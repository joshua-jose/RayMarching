use std::f32::consts::PI;

use crate::colour::phong_ds;

use super::colour::{ACESFilm, Pixel};
use super::objects::{EngineLight, EngineObject, PointLight};
use super::radiosity::{Lightmap, MAP_SIZE};
use super::ray::Ray;
use super::vector::Vec3;
use Vec3 as Colour;

pub const WIDTH: usize = 700;
pub const HEIGHT: usize = 700;

pub const MAX_MARCH_DISTANCE: f32 = 50.0;
pub const SMALL_DISTANCE: f32 = 0.001;
pub const MAX_SHAD_IT: u32 = 16;
pub const SKY_COLOUR: Vec3 = rgb![135, 206, 235];

type ObjectRef = Box<dyn EngineObject>;

static mut N: i32 = 0;

#[inline(always)]
fn to_pixel_range(i: f32) -> u8 {
    (255.0 * i).round().clamp(0.0, 255.0) as u8
}

impl Engine {
    pub fn render(&self, buffer: &mut [u8], _width: usize) {
        unsafe {
            N += 1;
            //self.camera_position.y = 2.0 + 3.0 * (0.01 * N as f32).sin();
            //self.camera_position.x = 2.0 * (0.01 * N as f32).cos();

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

        let buffer_pixels =
            unsafe { std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut Pixel, WIDTH * HEIGHT * 4) };

        let mut directions = vec![Vec::with_capacity(WIDTH); HEIGHT];

        for y_inv in 0..HEIGHT {
            for x in 0..WIDTH {
                // calculate proper y value and pixel uvs
                let y = HEIGHT - y_inv;
                let u = ((2 * x) as f32 / WIDTH as f32) - 1.0;
                let v = ((2 * y) as f32 / HEIGHT as f32) - 1.0;

                // array of vectors out of each pixel
                let direction_vector = Vec3 { x: u, y: v, z: 1.0 }.normalized();
                directions[y_inv].push(direction_vector);
            }
        }

        let mut colours: Vec<Vec<Colour>> = vec![Vec::with_capacity(WIDTH); HEIGHT];

        // calculate the linear colour of each pixel
        for y_inv in 0..HEIGHT {
            for x in 0..WIDTH {
                let colour_linear: Colour = self.cast_sight_ray(self.camera_position, directions[y_inv][x]);
                colours[y_inv].push(colour_linear);
            }
        }

        // perform sRGB colour corrections and tone mapping
        for y_inv in 0..HEIGHT {
            for x in 0..WIDTH {
                colours[y_inv][x] = ACESFilm(colours[y_inv][x]).sqrt();
            }
        }

        // write the colour to the frame buffer
        for y_inv in 0..HEIGHT {
            for x in 0..WIDTH {
                let i = y_inv * WIDTH + x;

                /*
                let mut colours: [Vec3; 5] = [Default::default(); 5];
                let offsets: [Vec3; 5] = [
                    Vec3::new(0.5 / WIDTH as f32, 0.0 / HEIGHT as f32, 0.0),
                    Vec3::new(-0.5 / WIDTH as f32, 0.0 / HEIGHT as f32, 0.0),
                    Vec3::new(0.0 / WIDTH as f32, 0.5 / HEIGHT as f32, 0.0),
                    Vec3::new(0.0 / WIDTH as f32, -0.5 / HEIGHT as f32, 0.0),
                    Vec3::new(0.0 / WIDTH as f32, 0.0 / HEIGHT as f32, 0.0),
                ];
                for i in 0..5 {
                    colours[i] = self.cast_sight_ray(
                        self.camera_position,
                        (Vec3 { x: u, y: v, z: 1.0 } + offsets[i]).normalized(),
                    );
                }
                let colour_linear =
                    (colours[0] + colours[1] + colours[2] + colours[3] + colours[4]) / 5.0;

                */
                let colour_tonemapped = colours[y_inv][x];

                // transform 0..1 to 0..255
                buffer_pixels[i] = Pixel {
                    r: to_pixel_range(colour_tonemapped.z),
                    g: to_pixel_range(colour_tonemapped.y),
                    b: to_pixel_range(colour_tonemapped.x),
                    a: 0,
                };
            }
        }
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

        let ambient: Colour;
        match object.get_lightmap() {
            None => ambient = object_colour * object_mat.ambient,
            Some(_) => ambient = object.sample(position),
        }
        //let ambient = object_colour * object_mat.ambient;

        let n = object.calculate_normal(position); // normal vector

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

    pub fn compute_radiosity(&mut self) {
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
        let mut colour_cloud: Vec<[[Colour; MAP_SIZE]; MAP_SIZE]> = vec![];
        colour_cloud.resize(obj_indexes.len(), [[Colour::default(); MAP_SIZE]; MAP_SIZE]);

        // get point cloud (world pos of all points)
        for (cloud_index, &obj_index) in obj_indexes.iter().enumerate() {
            for x in 0..MAP_SIZE {
                for y in 0..MAP_SIZE {
                    let sample_pos = self.objects[obj_index].get_sample_pos(x, y);
                    point_cloud[cloud_index][x][y] = sample_pos;
                    colour_cloud[cloud_index][x][y] = self.objects[obj_index].colour(sample_pos);
                }
            }
        }

        let light_pos = self.light.get_position();
        let light_intensity = self.light.get_intensity();

        let mut emissive_maps: Vec<Lightmap> = Vec::new();
        emissive_maps.reserve(obj_indexes.len());

        // direct lighting stage
        for (cloud_index, &obj_index) in obj_indexes.iter().enumerate() {
            let object = &self.objects[obj_index];
            let mut emissive_map = Lightmap::default();

            for x in 0..MAP_SIZE {
                for y in 0..MAP_SIZE {
                    let origin = point_cloud[cloud_index][x][y];

                    let vector_to_light = light_pos - origin;
                    let distance_to_light = vector_to_light.mag();
                    let vector_to_light = vector_to_light / distance_to_light;

                    /*
                    let mut shadow_ray = Ray {
                        position: origin,
                        direction: vector_to_light,
                    };
                    let _ = self.march(&mut shadow_ray, Some(obj_index));

                    if (shadow_ray.position - origin).mag() >= distance_to_light {
                        let n = Engine::calculate_normal(origin, object);
                        let diffuse =
                            n.dot(shadow_ray.direction).max(0.0) * light_intensity / (distance_to_light + 1.0).powi(2);

                        lightmap.sample_map[x][y] = object.colour(origin) * diffuse;
                    }
                    */

                    let n = object.calculate_normal(origin);
                    let diffuse = n.dot(vector_to_light).max(0.0) * light_intensity / (distance_to_light + 1.0).powi(2);

                    emissive_map.sample_map[x][y] = object.colour(origin) * diffuse;
                }
            }
            emissive_maps.push(emissive_map);
            //self.objects[obj_index].set_lightmap(emissive_map);
        }

        // generate occlusion matrix from point cloud
        // light bounces
        for _ in 0..2 {
            let mut lightmaps: Vec<Lightmap> = Vec::new();
            lightmaps.reserve(obj_indexes.len());

            let mut new_emissive_maps: Vec<Lightmap> = Vec::new();
            new_emissive_maps.reserve(obj_indexes.len());

            for (lit_cloud_index, &lit_obj_index) in obj_indexes.iter().enumerate() {
                let lit_object = &self.objects[lit_obj_index];
                let mut lit_lightmap = lit_object.get_lightmap().unwrap().clone();
                let mut new_emissive_map = Lightmap::default();

                for x in 0..MAP_SIZE {
                    for y in 0..MAP_SIZE {
                        // patch we are lighting
                        let origin = point_cloud[lit_cloud_index][x][y];
                        let lit_point_colour = colour_cloud[lit_cloud_index][x][y];
                        let n_lit = lit_object.calculate_normal(origin);
                        let mut incident: Colour = Colour::new(0.0, 0.0, 0.0);

                        for (lighting_cloud_index, &lighting_obj_index) in obj_indexes.iter().enumerate() {
                            // get light output from this patch
                            if lit_obj_index == lighting_obj_index {
                                continue;
                            }
                            //let lighting_object = &self.objects[lighting_obj_index];
                            //let lighting_lightmap = lighting_object.get_lightmap().unwrap().clone();
                            let lighting_lightmap = emissive_maps[lighting_cloud_index];

                            for a in 0..MAP_SIZE {
                                for b in 0..MAP_SIZE {
                                    // patch we are lighting
                                    let light_source = point_cloud[lighting_cloud_index][a][b];
                                    let light_colour = lighting_lightmap.sample_map[a][b];

                                    let vector_to_light = light_source - origin;
                                    let distance_to_light = vector_to_light.mag();
                                    let vector_to_light = vector_to_light / distance_to_light;

                                    /*
                                    let mut shadow_ray = Ray {
                                        position: origin,
                                        direction: vector_to_light,
                                    };
                                    let hit = self.march(&mut shadow_ray, Some(lit_obj_index));


                                    if hit.is_some() {
                                        if hit.unwrap() == lighting_obj_index {
                                            let diffuse = n_lit.dot(shadow_ray.direction).max(0.0) * 1.0
                                                / (distance_to_light + 1.0).powi(2);
                                            incident += light_colour.element_mul(lit_point_colour) * diffuse;
                                        }
                                    }
                                    */
                                    let diffuse =
                                        n_lit.dot(vector_to_light).max(0.0) * 1.0 / (distance_to_light + 1.0).powi(2);
                                    incident += light_colour.element_mul(lit_point_colour) * diffuse;
                                }
                            }
                            // scale by surface area of the patches
                            //incident = incident / ((MAP_SIZE * MAP_SIZE) as f32);
                        }
                        // "because calculus"
                        lit_lightmap.sample_map[x][y] += incident.element_mul(incident) / PI;
                        new_emissive_map.sample_map[x][y] = incident.element_mul(incident) / PI;
                        //emissive_maps[lit_cloud_index].sample_map[x][y] += incident / PI;
                        //println!("{}", lit_lightmap.sample_map[x][y]);
                    }
                }
                lightmaps.push(lit_lightmap);
                new_emissive_maps.push(new_emissive_map);
            }

            for (cloud_index, &obj_index) in obj_indexes.iter().enumerate() {
                self.objects[obj_index].set_lightmap(lightmaps[cloud_index]);
                emissive_maps[cloud_index] = new_emissive_maps[cloud_index];
            }
        }

        // for each bounce, copy all light maps and recompute them
    }
}

pub struct Engine {
    pub objects: Vec<ObjectRef>,
    pub camera_position: Vec3,
    pub light: PointLight,
}
