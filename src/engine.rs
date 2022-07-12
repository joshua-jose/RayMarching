use super::objects::{EngineLight, EngineObject, PointLight};
use super::vector::Vec3;
use Vec3 as Colour;

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 800;

type ObjectRef<'a> = &'a dyn EngineObject;

static mut N: i32 = 0;
const SKY_COLOUR: Vec3 = rgb![135, 206, 235];

#[repr(C)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[inline(always)]
fn to_pixel_range(i: f32) -> u8 {
    (255.0 * i).round().clamp(0.0, 255.0) as u8
}

impl Engine<'_> {
    pub fn render(&mut self, buffer: &mut [u8], _width: usize) {
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

        let buffer_pixels = unsafe {
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut Pixel, WIDTH * HEIGHT * 4)
        };

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

                let colour_linear: Colour = self.cast_sight_ray(
                    self.camera_position,
                    (Vec3 { x: u, y: v, z: 1.0 }).normalized(),
                );

                // perform sRGB colour corrections
                let colour_tonemapped = Engine::ACESFilm(colour_linear);
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

    #[allow(non_snake_case)]
    fn ACESFilm(mut col: Colour) -> Colour {
        let a: f32 = 2.51;
        let b: f32 = 0.03;
        let c: f32 = 2.43;
        let d: f32 = 0.59;
        let e: f32 = 0.14;
        let map_colours = |x: f32| ((x * (a * x + b)) / (x * (c * x + d) + e));
        col.x = map_colours(col.x);
        col.y = map_colours(col.y);
        col.z = map_colours(col.z);
        col
    }

    fn cast_sight_ray(&self, position: Vec3, direction: Vec3) -> Colour {
        let colour: Colour;

        let mut ray = Ray {
            position,
            direction,
        };
        // object the sight ray hit
        let has_hit = self.march(&mut ray, None);

        // if we hit an object, colour this pixel
        //sky_colour[1] = sky_colour[1] * (direction.y.max(0.2));
        //sky_colour[0] = sky_colour[0] * (direction.y.max(0.2));
        match has_hit {
            None => colour = SKY_COLOUR,
            Some(object) => colour = self.shade_object(object, ray.position, direction),
        }
        colour
    }

    fn shade_object(&self, object: ObjectRef, position: Vec3, direction: Vec3) -> Colour {
        let mut final_colour: Colour;

        let diffuse: f32;
        let ambient: f32 = object.ambient();
        let specular: f32;

        let n = Engine::calculate_normal(position, object); // normal vector
        let object_colour = object.colour(position);

        let vector_to_light = self.light.get_position() - position;

        let distance_to_light_sqd = vector_to_light.mag_sqd();
        let distance_to_light = distance_to_light_sqd.sqrt();
        let vector_to_light = vector_to_light / distance_to_light;
        let light_reflection_vector = vector_to_light.reflect(n);
        let light_intensity = self.light.get_intensity() / (distance_to_light + 1.0).powi(2); // k/d^2

        // cast a shadow ray to see if this point is blocked by another object
        let mut shadow_ray = Ray {
            position,
            direction: vector_to_light,
        };
        let shade = self.smooth_shadow_march(&mut shadow_ray, object, distance_to_light, 16.0);

        // Phong shading algorithm
        diffuse = shade * object.diffuse() * light_intensity * vector_to_light.dot(n).max(0.0);
        if diffuse > 0.0 {
            specular = shade
                * object.specular()
                * light_intensity
                * light_reflection_vector
                    .dot(direction)
                    .max(0.0)
                    .powf(object.shininess());
        } else {
            specular = 0.0;
        }
        //}

        final_colour = object_colour * (diffuse + ambient + specular);

        // if the object is reflective, cast a reflection ray
        if object.reflectivity() > 1e-3 {
            // very cheap fresnel effect
            let fresnel = (1.0 - n.dot(-direction)).clamp(0.0, 1.0).powi(5);

            let reflection_vector = direction.reflect(n);
            let reflection_colour = self.cast_sight_ray(
                position + (reflection_vector * 2.0 * SMALL_DISTANCE),
                reflection_vector,
            );

            /*
            final_colour = final_colour
                + object.reflectivity()
                    * (object.reflectivity() * reflection_colour
                        + ((1.0 - object.reflectivity())
                            * (reflection_colour.element_mul(object_colour))));
            */
            final_colour = final_colour
                + (fresnel + object.reflectivity()).clamp(0.0, 1.0)
                    * reflection_colour.element_mul(object_colour);
        }
        final_colour
    }

    fn march(&self, ray: &mut Ray, ignore_object: Option<ObjectRef>) -> Option<ObjectRef> {
        let mut distance_travelled = 0.0;

        while distance_travelled < MAX_MARCH_DISTANCE {
            let mut distance = f32::INFINITY;
            let mut closest_object: Option<ObjectRef> = None;

            for object in &self.objects {
                if ignore_object.is_some() {
                    if std::ptr::eq(ignore_object.unwrap(), *object) {
                        continue;
                    }
                }

                let obj_distance = object.sdf(ray.position);
                if obj_distance < distance {
                    distance = obj_distance;
                    closest_object = Some(*object);
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

    fn smooth_shadow_march(
        &self,
        ray: &mut Ray,
        ignore_object: ObjectRef,
        light_dist: f32,
        shading_k: f32,
    ) -> f32 {
        let mut distance_travelled = 0.0;
        let mut shade: f32 = 1.0; // actually the amount of "not shade"
        const MAX_SHAD_IT: u32 = 12;
        let smoothstep = |x: f32| 3.0 * x.powi(2) - 2.0 * x.powi(3);

        for _ in 0..MAX_SHAD_IT {
            let mut distance = f32::INFINITY;

            for object in &self.objects {
                if std::ptr::eq(ignore_object, *object) {
                    continue;
                }

                let obj_distance = object.sdf(ray.position);
                if obj_distance < distance {
                    distance = obj_distance;
                };
            }

            shade = shade.min(smoothstep(
                (shading_k * distance / distance_travelled).clamp(0.0, 1.0),
            ));
            //distance = distance.clamp(SMALL_DISTANCE, light_dist / MAX_SHAD_IT as f32);
            distance_travelled += distance; // could clamp this for better res
            ray.position = ray.position + (ray.direction * distance);
            if distance < SMALL_DISTANCE || distance_travelled > light_dist {
                break;
            }
        }
        shade.clamp(0.0, 1.0)
    }

    fn calculate_normal(position: Vec3, object: ObjectRef) -> Vec3 {
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
    fn _read_from_buffer(
        buffer: &mut [u8],
        x: usize,
        y: usize,
        width: usize,
        _height: usize,
    ) -> Vec3 {
        let i = y * width + x;

        Vec3 {
            x: buffer[i * 4 + 2] as f32 / 255.0,
            y: buffer[i * 4 + 1] as f32 / 255.0,
            z: buffer[i * 4 + 0] as f32 / 255.0,
        }
    }
}

pub struct Engine<'a> {
    pub objects: Vec<ObjectRef<'a>>,
    pub camera_position: Vec3,
    pub light: PointLight,
}

struct Ray {
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

const MAX_MARCH_DISTANCE: f32 = 50.0;
const SMALL_DISTANCE: f32 = 0.001;
