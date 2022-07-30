use super::colour::{phong_ds, ACESFilm, Colour, Pixel};
use super::objects::{EngineLight, EngineObject, PointLight};
use super::radiosity::{compute_direct_lighting, compute_object_radiosity, Lightmap, MAP_SIZE};
use super::ray::Ray;
use super::vector::Vec3;

use rayon::prelude::*;

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 600;

pub const MAX_MARCH_DISTANCE: f32 = 50.0;
pub const SMALL_DISTANCE: f32 = 0.001;
pub const MAX_SHAD_IT: u32 = 64;
pub const SKY_COLOUR: Vec3 = rgb![135, 206, 235];

pub type ObjectRef = Box<dyn EngineObject>;

static mut N: i32 = 0;

#[inline]
fn to_pixel_range(i: f32) -> u8 { (255.0 * i).round().clamp(0.0, 255.0) as u8 }

fn rot(vector: Vec3, sx: f32, cx: f32, sy: f32, cy: f32) -> Vec3 {
    let mut x = vector.x();
    let mut y = vector.y();
    let mut z = vector.z();

    (y, z) = (y * cy - z * sy, y * sy + z * cy);
    (x, z) = (x * cx - z * sx, x * sx + z * cx);
    Vec3::new(x, y, z)
}

#[repr(align(128))]
pub struct Aligned<T: ?Sized>(pub T);

impl Engine {
    pub fn render(
        &mut self, buffer: &mut [u8], directions: &mut Aligned<Vec<Vec<Vec3>>>, mouse_x: i32, mouse_y: i32,
        rel_move: Vec3, scroll: i32,
    ) {
        unsafe {
            N += 1;
            //self.camera_position.0[1] = 2.0 + 3.0 * (0.01 * N as f32).sin();
            //self.camera_position.0[0] = 2.0 * (0.1 * N as f32).cos();

            //zoffset = 3.0 + 6.0 * (0.1 * N as f32).sin();
            //xoffset = 3.0 + 6.0 * (0.1 * N as f32).cos();
        }

        /*
        self.light.position = Vec3 {
            x: -5.0,
            y: 5.0,
            z: -5.0,
        } + Vec3 {
            x: xoffset,
            y: 0.0,
            z: zoffset,
        };
        */

        let exposure: f32 = 1.0;

        let theta_x = -mouse_x as f32 / 600.0;
        let theta_y = mouse_y as f32 / 600.0;

        let (sx, cx) = theta_x.sin_cos();
        let (sy, cy) = theta_y.sin_cos();

        // move camera in direction we are facing
        self.camera_position += rot(rel_move, sx, cx, sy, cy);
        // start at 90 deg fov, change by 5 deg each scroll step
        let fov_deg = 90.0 - (scroll as f32 * 5.0);
        let zdepth = (fov_deg * 0.5).to_radians().tan().recip();

        directions.0.par_iter_mut().enumerate().for_each(|(y_inv, rows)| {
            for x in 0..WIDTH {
                let y = HEIGHT - y_inv;
                let u = 2.0 * (x as f32 - (0.5 * (WIDTH as f32))) / HEIGHT as f32; // divide u by height to account for aspect ratio
                let v = 2.0 * (y as f32 - (0.5 * (HEIGHT as f32))) / HEIGHT as f32;

                // array of vectors out of each pixel
                rows[x] = rot(Vec3::new(u, v, zdepth), sx, cx, sy, cy).normalized();
            }
        });

        let buffer_pixels =
            unsafe { std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut Pixel, WIDTH * HEIGHT) };

        let mut colours = vec![Vec::with_capacity(WIDTH); HEIGHT];

        colours.par_iter_mut().enumerate().for_each(|(y_inv, rows)| {
            for x in 0..WIDTH {
                /*
                let mut ssaa_colours: [Vec3; 4] = [Default::default(); 4];
                let offsets: [Vec3; 4] = [
                    Vec3::new(0.5 / WIDTH as f32, 0.0 / HEIGHT as f32, 0.0),
                    Vec3::new(-0.5 / WIDTH as f32, 0.0 / HEIGHT as f32, 0.0),
                    Vec3::new(0.0 / WIDTH as f32, 0.5 / HEIGHT as f32, 0.0),
                    Vec3::new(0.0 / WIDTH as f32, -0.5 / HEIGHT as f32, 0.0),
                ];
                for i in 0..4 {
                    ssaa_colours[i] =
                        self.cast_sight_ray(self.camera_position, (directions.0[y_inv][x] + offsets[i]).normalized());
                }
                let colour_linear = (ssaa_colours[0] + ssaa_colours[1] + ssaa_colours[2] + ssaa_colours[3]) / 4.0;
                */
                let colour_linear: Colour = self.cast_sight_ray(self.camera_position, directions.0[y_inv][x]);
                let colour_srgb = ACESFilm(colour_linear * exposure).sqrt();

                rows.push(colour_srgb);
            }
        });

        buffer_pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
            let x = i % WIDTH;
            let y_inv = i / WIDTH;
            let colour_srgb = colours[y_inv][x];
            *pixel = Pixel {
                r: to_pixel_range(colour_srgb.z()),
                g: to_pixel_range(colour_srgb.y()),
                b: to_pixel_range(colour_srgb.x()),
                a: 0,
            };
        });
    }

    fn cast_sight_ray(&self, position: Vec3, direction: Vec3) -> Colour {
        let colour: Colour;

        let mut ray = Ray { position, direction };
        // object the sight ray hit
        let has_hit = ray.march(&self.objects, None);

        // if we hit an object, colour this pixel
        //sky_colour[1] = sky_colour[1] * (direction.y.max(0.2));
        //sky_colour[0] = sky_colour[0] * (direction.y.max(0.2));

        match has_hit {
            None => colour = SKY_COLOUR,
            Some(obj_index) => colour = self.shade_object(obj_index, ray.position, direction),
        }
        colour
    }

    fn shade_object(&self, obj_index: usize, position: Vec3, direction: Vec3) -> Colour {
        let mut final_colour: Colour;
        let object = &self.objects[obj_index];

        let object_colour = object.colour(position);
        let object_mat = object.material();
        let n = object.calculate_normal(position); // normal vector

        let mut ambient: Colour;

        match object.get_lightmap() {
            None => ambient = object_colour * object_mat.ambient,
            Some(_) => ambient = object.sample_lightmap(position),
        }

        // set a minimum intensity
        if ambient.mag_sqd() < object_mat.ambient.powi(2) {
            ambient = ambient.normalized() * object_mat.ambient;
        }

        // colour the pixel correctly
        ambient = ambient.element_mul(object_colour);

        //let ambient = object_colour * object_mat.ambient;

        // get normalised vector to light, and distance
        let vector_to_light = self.light.get_position() - position;
        let distance_to_light = vector_to_light.mag();
        let vector_to_light = vector_to_light / distance_to_light;

        // get the diffuse and specular lighting of this object
        let (diffuse, specular) = phong_ds(
            n,
            vector_to_light,
            distance_to_light,
            self.light.get_intensity(),
            object_mat,
            direction,
        );

        // cast a shadow ray to see if this point is blocked by another object
        let shade = Ray {
            position,
            direction: vector_to_light,
        }
        .smooth_shadow_march(&self.objects, obj_index, distance_to_light, 16.0);

        final_colour = ambient + object_colour * (shade * (diffuse + specular));

        // if object_mat.refract
        /*
        if object_mat.reflectivity > 1e-3 {
            let ior = 2.5;
            let refraction_vector = direction.refract(n, 1.0 / ior);
            let mut enter_ray = Ray {
                position:  position + refraction_vector * (3.0 * SMALL_DISTANCE),
                direction: refraction_vector,
            };

            enter_ray.internal_march(object);

            let exit_n = object.calculate_normal(enter_ray.position);
            let mut exit_direction = refraction_vector.refract(-exit_n, ior);
            let exit_pos = enter_ray.position;

            // TIR
            if exit_direction.mag_sqd() == 0.0 {
                exit_direction = refraction_vector.reflect(-exit_n);
            }

            let mut exit_ray = Ray {
                position:  exit_pos,
                direction: exit_direction,
            };

            let refr_hit = exit_ray.march(&self.objects, Some(obj_index));

            if refr_hit.is_some() {
                final_colour += self.shade_object(refr_hit.unwrap(), exit_ray.position, exit_ray.direction);
            }
        }
        */

        // if the object is reflective, cast a reflection ray
        if object_mat.reflectivity > 1e-3 {
            // very cheap fresnel effect
            let fresnel = (1.0 - n.dot(-direction)).clamp(0.0, 1.0).powi(5);

            let reflection_vector = direction.reflect(n);
            let reflection_colour =
                self.cast_sight_ray(position + (reflection_vector * 3.0 * SMALL_DISTANCE), reflection_vector);

            final_colour +=
                (fresnel + object_mat.reflectivity).clamp(0.0, 1.0) * reflection_colour.element_mul(object_colour);
        }

        final_colour
    }

    pub fn compute_lightmaps(&mut self) {
        self.objects.iter_mut().for_each(|x| x.clear_lightmap());

        // get all objects with a lightmap
        let obj_indexes = self
            .objects
            .iter()
            .enumerate()
            .filter(|(_, x)| x.get_lightmap().is_some())
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        // map of every sample point's position in the world
        let mut point_cloud: Vec<[[Vec3; MAP_SIZE]; MAP_SIZE]> = vec![];
        point_cloud.resize(obj_indexes.len(), [[Vec3::default(); MAP_SIZE]; MAP_SIZE]);
        // map of every sample point's colour
        let mut colour_cloud: Vec<[[Colour; MAP_SIZE]; MAP_SIZE]> = vec![];
        colour_cloud.resize(obj_indexes.len(), [[Colour::default(); MAP_SIZE]; MAP_SIZE]);
        // map of every sample point's normal
        let mut normal_cloud: Vec<[[Vec3; MAP_SIZE]; MAP_SIZE]> = vec![];
        normal_cloud.resize(obj_indexes.len(), [[Vec3::default(); MAP_SIZE]; MAP_SIZE]);

        // get point cloud (world pos of all points)
        for (cloud_index, &obj_index) in obj_indexes.iter().enumerate() {
            for x in 0..MAP_SIZE {
                for y in 0..MAP_SIZE {
                    let sample_pos = self.objects[obj_index].get_sample_pos(x, y);
                    let colour = self.objects[obj_index].colour(sample_pos);
                    let normal = self.objects[obj_index].calculate_normal(sample_pos);

                    colour_cloud[cloud_index][x][y] = colour;
                    normal_cloud[cloud_index][x][y] = normal;
                    point_cloud[cloud_index][x][y] = sample_pos;
                }
            }
        }

        let (mut emissive_maps, new_lightmaps) =
            compute_direct_lighting(&self.light, &obj_indexes, &self.objects, &point_cloud, &colour_cloud);

        for (cloud_index, &obj_index) in obj_indexes.iter().enumerate() {
            self.objects[obj_index].set_lightmap(new_lightmaps[cloud_index]);
        }

        // TODO: generate occlusion matrix from point cloud

        let mut lightmaps: Vec<Lightmap> = Vec::new();
        lightmaps.reserve(obj_indexes.len());

        let mut new_emissive_maps: Vec<Lightmap> = Vec::new();
        new_emissive_maps.reserve(obj_indexes.len());

        // light bounces
        for _ in 0..4 {
            lightmaps.clear();
            new_emissive_maps.clear();

            for (obj_cloud_index, &obj_eng_index) in obj_indexes.iter().enumerate() {
                let object = &self.objects[obj_eng_index];

                let (lit_lightmap, new_emissive_map) = compute_object_radiosity(
                    &self.objects,
                    object,
                    obj_cloud_index,
                    obj_eng_index,
                    &point_cloud,
                    &colour_cloud,
                    &normal_cloud,
                    &emissive_maps,
                );

                lightmaps.push(lit_lightmap);
                new_emissive_maps.push(new_emissive_map);
            }

            for (cloud_index, &obj_index) in obj_indexes.iter().enumerate() {
                self.objects[obj_index].set_lightmap(lightmaps[cloud_index]);
                emissive_maps[cloud_index] = new_emissive_maps[cloud_index];
            }
        }
    }
}

pub struct Engine {
    pub objects:         Vec<ObjectRef>,
    pub camera_position: Vec3,
    pub light:           PointLight,
}

unsafe impl Sync for Engine {}
unsafe impl Send for Engine {}
