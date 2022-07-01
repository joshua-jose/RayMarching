use super::objects::{EngineLight, EngineObject, PointLight};
use super::vector::Vec3;

pub const WIDTH: usize = 400;
pub const HEIGHT: usize = 400;

type ObjectRef<'a> = &'a dyn EngineObject;

static mut N: i32 = 0;

impl Engine<'_> {
    pub fn render(&mut self, buffer: &mut [u8], _width: usize) {
        let zoffset: f32;
        let xoffset: f32;
        unsafe {
            N += 1;
            //self.camera_position.y = 2.0 + 3.0 * (0.01 * N as f32).sin();
            //self.camera_position.x = 2.0 * (0.01 * N as f32).cos();

            //zoffset = 3.0 + 6.0 * (0.1 * N as f32).sin();
            //xoffset = 3.0 + 6.0 * (0.1 * N as f32).cos();
            zoffset = 0.0;
            xoffset = 0.0;
        }

        self.light.position = Vec3 {
            x: -5.0,
            y: 5.0,
            z: -5.0,
        } + Vec3 {
            x: xoffset,
            y: 0.0,
            z: zoffset,
        };

        for y_inv in 0..HEIGHT {
            for x in 0..WIDTH {
                // calculate proper y value and pixel uvs
                let y = HEIGHT - y_inv;
                let u = ((2 * x) as f32 / WIDTH as f32) - 1.0;
                let v = ((2 * y) as f32 / HEIGHT as f32) - 1.0;

                let colour_linear: [f32; 3] = self.cast_sight_ray(
                    self.camera_position,
                    Vec3 { x: u, y: v, z: 1.0 }.normalized(),
                );

                // perform sRGB colour corrections
                let colour_srgb = colour_linear.map(|i| (i).sqrt());
                // transform 0..1 to 0..255
                let colour_scaled =
                    colour_srgb.map(|i| (255.0 * i).round().clamp(0.0, 255.0) as u8);

                Engine::write_to_buffer(buffer, x, y_inv, WIDTH, HEIGHT, colour_scaled);
            }
        }
    }

    fn cast_sight_ray(&self, position: Vec3, direction: Vec3) -> [f32; 3] {
        let colour: [f32; 3];

        let mut ray = Ray {
            position,
            direction,
        };
        // object the sight ray hit
        let has_hit = self.march(&mut ray, None);

        // if we hit an object, colour this pixel
        let mut sky_colour = rgb![135, 206, 235];
        sky_colour[1] = sky_colour[1] * (direction.y.max(0.2));
        sky_colour[0] = sky_colour[0] * (direction.y.max(0.2));
        match has_hit {
            None => colour = sky_colour,
            Some(object) => colour = self.shade_object(object, ray.position, direction),
        }
        colour
    }

    fn shade_object(&self, object: ObjectRef, position: Vec3, direction: Vec3) -> [f32; 3] {
        let mut r: f32;
        let mut g: f32;
        let mut b: f32;

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
        let light_intensity = self.light.get_intensity() / distance_to_light_sqd; // k/d^2

        // cast a shadow ray to see if this point is blocked by another object
        //let light_blocked;
        let mut shadow_ray = Ray {
            position,
            direction: vector_to_light,
        };
        //let light_intersection = self.march(&mut shadow_ray, Some(object));
        let shade = self.smooth_shadow_march(&mut shadow_ray, object, distance_to_light, 16.0);

        // if light source is closer than the hit world object, then there is a line of sight
        /*
        match light_intersection {
            None => light_blocked = false,
            Some(_) => {
                light_blocked = distance_to_light_sqd > (shadow_ray.position - position).mag_sqd();
            }
        }
        */

        // if this point is blocked by some other object, do not light it.
        /*
        if light_blocked {
            diffuse = 0.0;
            specular = 0.0;
        } else {
            */
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

        r = object_colour[0] * (diffuse + ambient + specular);
        g = object_colour[1] * (diffuse + ambient + specular);
        b = object_colour[2] * (diffuse + ambient + specular);

        // if the object is reflective, cast a reflection ray
        if object.reflectivity() > 1e-3 {
            let reflection_vector = direction.reflect(n);
            let reflection_colour = self.cast_sight_ray(
                position + (reflection_vector * 2.0 * SMALL_DISTANCE),
                reflection_vector,
            );

            r += object.reflectivity() * reflection_colour[0];
            g += object.reflectivity() * reflection_colour[1];
            b += object.reflectivity() * reflection_colour[2];
        }

        [r, g, b]
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
            distance_travelled += distance;
            ray.position = ray.position + (ray.direction * distance);

            if distance < SMALL_DISTANCE {
                return closest_object;
            }
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
        const MAX_SHAD_IT: u32 = 8;
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

    fn write_to_buffer(
        buffer: &mut [u8],
        x: usize,
        y: usize,
        width: usize,
        _height: usize,
        colour: [u8; 3],
    ) {
        let i = y * width + x;
        buffer[i * 4 + 0] = colour[2];
        buffer[i * 4 + 1] = colour[1];
        buffer[i * 4 + 2] = colour[0];
    }

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
