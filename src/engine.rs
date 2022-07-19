use std::f32::consts::PI;

use crate::radiosity::{Lightmap, MAP_SIZE};

use super::colour::{ACESFilm, Pixel};
use super::objects::{EngineLight, EngineObject, PointLight};
use super::vector::Vec3;
use Vec3 as Colour;

pub const WIDTH: usize = 700;
pub const HEIGHT: usize = 700;

const MAX_MARCH_DISTANCE: f32 = 50.0;
const SMALL_DISTANCE: f32 = 0.001;
const MAX_SHAD_IT: u32 = 16;
const SKY_COLOUR: Vec3 = rgb![135, 206, 235];

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

        for y_inv in 0..HEIGHT {
            for x in 0..WIDTH {
                // calculate proper y value and pixel uvs
                let y = HEIGHT - y_inv;
                let u = ((2 * x) as f32 / WIDTH as f32) - 1.0;
                let v = ((2 * y) as f32 / HEIGHT as f32) - 1.0;
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
                let colour_linear: Colour =
                    self.cast_sight_ray(self.camera_position, (Vec3 { x: u, y: v, z: 1.0 }).normalized());

                let colour_tonemapped = ACESFilm(colour_linear);

                // perform sRGB colour corrections
                // transform 0..1 to 0..255
                buffer_pixels[i] = Pixel {
                    r: to_pixel_range(colour_tonemapped.z.sqrt()),
                    g: to_pixel_range(colour_tonemapped.y.sqrt()),
                    b: to_pixel_range(colour_tonemapped.x.sqrt()),
                    a: 255,
                };
            }
        }
    }

    fn cast_sight_ray(&self, position: Vec3, direction: Vec3) -> Colour {
        let colour: Colour;

        let mut ray = Ray { position, direction };
        // object the sight ray hit
        let has_hit = self.march(&mut ray, None);

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
        let object_mat = &object.material();

        let ambient: Colour;
        if object.get_lightmap().is_some() {
            let sample = object.sample(position);

            ambient = sample;
        } else {
            ambient = object_colour * object_mat.ambient;
        }

        //let ambient = object_colour * object_mat.ambient;
        let diffuse: f32;
        let specular: f32;

        let n = Engine::calculate_normal(position, object); // normal vector

        let vector_to_light = self.light.get_position() - position;
        let distance_to_light = vector_to_light.mag();
        let vector_to_light = vector_to_light / distance_to_light;

        let light_reflection_vector = vector_to_light.reflect(n);
        let light_intensity = self.light.get_intensity() / (distance_to_light + 1.0).powi(2); // k/d^2

        // cast a shadow ray to see if this point is blocked by another object
        let mut shadow_ray = Ray {
            position,
            direction: vector_to_light,
        };
        let shade = self.smooth_shadow_march(&mut shadow_ray, obj_index, distance_to_light, 16.0);

        // Phong shading algorithm
        diffuse = object_mat.diffuse * light_intensity * vector_to_light.dot(n).max(0.0);
        if diffuse > 0.0 {
            specular = object_mat.specular
                * light_intensity
                * light_reflection_vector
                    .dot(direction)
                    .max(0.0)
                    .powf(object_mat.shininess);
        } else {
            specular = 0.0;
        }
        //}

        final_colour = ambient + object_colour * (shade * (diffuse + specular));

        // if the object is reflective, cast a reflection ray
        if object_mat.reflectivity > 1e-3 {
            // very cheap fresnel effect
            let fresnel = (1.0 - n.dot(-direction)).clamp(0.0, 1.0).powi(5);

            let reflection_vector = direction.reflect(n);
            let reflection_colour =
                self.cast_sight_ray(position + (reflection_vector * 2.0 * SMALL_DISTANCE), reflection_vector);

            /*
            final_colour = final_colour
                + object.reflectivity()
                    * (object.reflectivity() * reflection_colour
                        + ((1.0 - object.reflectivity())
                            * (reflection_colour.element_mul(object_colour))));
            */
            final_colour = final_colour
                + (fresnel + object_mat.reflectivity).clamp(0.0, 1.0) * reflection_colour.element_mul(object_colour);
        }
        final_colour
    }

    pub fn march(&self, ray: &mut Ray, ignore_object: Option<usize>) -> Option<usize> {
        let mut distance_travelled = 0.0;
        let has_ignore = ignore_object.is_some();

        while distance_travelled < MAX_MARCH_DISTANCE {
            let mut distance = f32::INFINITY;
            let mut closest_object: Option<usize> = None;

            for (i, object) in self.objects.iter().enumerate() {
                if has_ignore {
                    if ignore_object.unwrap() == i {
                        continue;
                    }
                }

                let obj_distance = object.sdf(ray.position);
                if obj_distance < distance {
                    distance = obj_distance;
                    closest_object = Some(i);
                };
            }
            if distance < SMALL_DISTANCE {
                return closest_object;
            }

            distance_travelled += distance;
            ray.position = ray.position + (ray.direction * distance);
        }
        return None;
    }

    fn smooth_shadow_march(&self, ray: &mut Ray, ignore_obj_index: usize, light_dist: f32, shading_k: f32) -> f32 {
        let mut distance_travelled = 0.0;
        let mut shade: f32 = 1.0; // actually the amount of "not shade"

        let smoothstep = |x: f32| 3.0 * x.powi(2) - 2.0 * x.powi(3);

        for _ in 0..MAX_SHAD_IT {
            let mut distance = f32::INFINITY;

            for (i, object) in self.objects.iter().enumerate() {
                if ignore_obj_index == i {
                    continue;
                }

                let obj_distance = object.sdf(ray.position);
                if obj_distance < distance {
                    distance = obj_distance;
                };
            }

            shade = shade.min(smoothstep((shading_k * distance / distance_travelled).clamp(0.0, 1.0)));
            //distance = distance.clamp(SMALL_DISTANCE, light_dist / MAX_SHAD_IT as f32);
            distance_travelled += distance; // could clamp this for better res
            ray.position = ray.position + (ray.direction * distance);
            if distance < SMALL_DISTANCE || distance_travelled > light_dist {
                break;
            }
        }
        shade.clamp(0.0, 1.0)
    }

    pub fn calculate_normal(position: Vec3, object: &ObjectRef) -> Vec3 {
        let gradient_x = object.sdf(position + X_STEP) - object.sdf(position - X_STEP);
        let gradient_y = object.sdf(position + Y_STEP) - object.sdf(position - Y_STEP);
        let gradient_z = object.sdf(position + Z_STEP) - object.sdf(position - Z_STEP);

        Vec3 {
            x: gradient_x,
            y: gradient_y,
            z: gradient_z,
        }
        .normalized()
    }

    /*
    // cheaper tetrahedral normal
    vec2 e = vec2(.0015, -.0015);
    return normalize(
        e.xyy * map(p + e.xyy) +
        e.yyx * map(p + e.yyx) +
        e.yxy * map(p + e.yxy) +
        e.xxx * map(p + e.xxx));
    */

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

        // get point cloud (world pos of all points)
        for (cloud_index, &obj_index) in obj_indexes.iter().enumerate() {
            for x in 0..MAP_SIZE {
                for y in 0..MAP_SIZE {
                    point_cloud[cloud_index][x][y] = self.objects[obj_index].get_sample_pos(x, y);
                }
            }
        }

        let light_pos = self.light.get_position();
        let light_intensity = self.light.get_intensity();

        // direct lighting stage
        for (cloud_index, &obj_index) in obj_indexes.iter().enumerate() {
            let object = &self.objects[obj_index];
            let mut lightmap = object.get_lightmap().unwrap().clone();

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

                    let n = Engine::calculate_normal(origin, object);
                    let diffuse = n.dot(vector_to_light).max(0.0) * light_intensity / (distance_to_light + 1.0).powi(2);

                    lightmap.sample_map[x][y] = object.colour(origin) * diffuse;
                }
            }
            self.objects[obj_index].set_lightmap(lightmap);
        }

        // generate occlusion matrix from point cloud
        // light bounces
        for _ in 0..1 {
            let mut lightmaps: Vec<Lightmap> = Vec::new();
            lightmaps.reserve(obj_indexes.len());

            for (lit_cloud_index, &lit_obj_index) in obj_indexes.iter().enumerate() {
                let lit_object = &self.objects[lit_obj_index];
                let mut lit_lightmap = lit_object.get_lightmap().unwrap().clone();

                for x in 0..MAP_SIZE {
                    for y in 0..MAP_SIZE {
                        // patch we are lighting
                        let origin = point_cloud[lit_cloud_index][x][y];
                        let n_lit = Engine::calculate_normal(origin, lit_object);
                        let mut incident: Colour = Colour::new(0.0, 0.0, 0.0);

                        for (lighting_cloud_index, &lighting_obj_index) in obj_indexes.iter().enumerate() {
                            // get light output from this patch
                            if lit_obj_index == lighting_obj_index {
                                continue;
                            }
                            let lighting_object = &self.objects[lighting_obj_index];
                            let lighting_lightmap = lighting_object.get_lightmap().unwrap().clone();

                            for a in 0..MAP_SIZE {
                                for b in 0..MAP_SIZE {
                                    // patch we are lighting
                                    let light_source = point_cloud[lighting_cloud_index][a][b];
                                    let light_colour = lighting_lightmap.sample_map[a][b];
                                    // TODO: remove this... it's physically inaccurate but looks better
                                    if (light_colour.x - light_colour.y).abs() < 0.1 {
                                        continue;
                                    }

                                    let vector_to_light = light_source - origin;
                                    let distance_to_light = vector_to_light.mag();
                                    let vector_to_light = vector_to_light / distance_to_light;

                                    let mut shadow_ray = Ray {
                                        position: origin,
                                        direction: vector_to_light,
                                    };
                                    let hit = self.march(&mut shadow_ray, Some(lit_obj_index));

                                    if hit.is_some() {
                                        if hit.unwrap() == lighting_obj_index {
                                            let diffuse = n_lit.dot(shadow_ray.direction).max(0.0) * 1.0
                                                / (distance_to_light + 1.0).powi(2);
                                            incident += light_colour * diffuse;
                                        }
                                    }
                                }
                            }
                            // scale by surface area of the patches
                            //incident = incident / MAP_SIZE.pow(2) as f32;
                        }
                        // "because calculus"
                        lit_lightmap.sample_map[x][y] += incident / PI;
                        //println!("{}", lit_lightmap.sample_map[x][y]);
                    }
                }
                lightmaps.push(lit_lightmap);
            }

            for (cloud_index, &obj_index) in obj_indexes.iter().enumerate() {
                self.objects[obj_index].set_lightmap(lightmaps[cloud_index]);
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

pub struct Ray {
    pub position: Vec3,
    pub direction: Vec3,
}

const STEP_SIZE: f32 = 0.0001;
const X_STEP: Vec3 = Vec3 {
    x: STEP_SIZE,
    y: 0.0,
    z: 0.0,
};
const Y_STEP: Vec3 = Vec3 {
    x: 0.0,
    y: STEP_SIZE,
    z: 0.0,
};
const Z_STEP: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: STEP_SIZE,
};
