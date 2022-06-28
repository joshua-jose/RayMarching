use super::objects::EngineObject;
use super::vector::Vec3;

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 800;

type ObjectRef<'a> = &'a dyn EngineObject;

static mut N: i32 = 0;

impl Engine<'_> {
    pub fn render(&mut self, buffer: &mut [u8], _width: usize) {
        unsafe {
            N += 1;
            //self.camera_position.y = 2.0 + 3.0 * (0.1 * N as f32).sin();
            //self.camera_position.x = 2.0 * (0.1 * N as f32).cos();
        }

        for y_inv in 0..HEIGHT {
            for x in 0..WIDTH {
                // calculate proper y value and pixel uvs
                let y = HEIGHT - y_inv;
                let u = ((2 * x) as f32 / WIDTH as f32) - 1.0;
                let v = ((2 * y) as f32 / HEIGHT as f32) - 1.0;
                let i = y_inv * WIDTH + x;

                let colour: [u8; 3] = self.cast_sight_ray(
                    self.camera_position,
                    Vec3 { x: u, y: v, z: 1.0 }.normalized(),
                );

                buffer[i * 4 + 0] = colour[2];
                buffer[i * 4 + 1] = colour[1];
                buffer[i * 4 + 2] = colour[0];
                //buffer[i * 4 + 3] = 0;
            }
        }
    }

    fn cast_sight_ray(&self, position: Vec3, direction: Vec3) -> [u8; 3] {
        let colour: [u8; 3];

        let mut ray = Ray {
            position: position,
            direction: direction,
        };
        // object the sight ray hit
        let object = self.march(&mut ray);

        // if we hit an object, colour this pixel
        if object.is_some() {
            colour = self.object_pixel_colour(object.unwrap(), ray.position, direction);
        } else {
            // sky colour
            colour = [135, 206, 235];
        }
        colour
    }

    fn object_pixel_colour(&self, object: ObjectRef, position: Vec3, direction: Vec3) -> [u8; 3] {
        let n = Engine::calculate_normal(position, object);
        let r;
        let g;
        let b;

        let zoffset: f32;
        let xoffset: f32;
        unsafe {
            zoffset = 3.0 + 6.0 * (0.1 * N as f32).sin();
            xoffset = 3.0 + 6.0 * (0.1 * N as f32).cos();
        }

        let object_colour = object.colour(position);

        if object.reflectivity() == 0 {
            let vector_to_light = (LIGHT_POS
                + Vec3 {
                    x: xoffset,
                    y: 0.0,
                    z: zoffset,
                })
                - position;

            let distance_to_light = vector_to_light.mag();
            let vector_to_light = vector_to_light.normalized();

            let light_intensity = 50.0 * distance_to_light.powi(2).recip();

            let mut light_ray = Ray {
                position: position,
                direction: vector_to_light,
            };
            let light_intersection = self.march_light(&mut light_ray, object);
            let diffuse: f32;

            match light_intersection {
                Some(_) => diffuse = 0.05 * vector_to_light.dot(n),
                None => diffuse = light_intensity * vector_to_light.dot(n).max(0.05),
            }

            r = (object_colour[0] as f32 * diffuse).floor() as u8;
            g = (object_colour[1] as f32 * diffuse).floor() as u8;
            b = (object_colour[2] as f32 * diffuse).floor() as u8;
        } else {
            let vector_to_light = (LIGHT_POS
                + Vec3 {
                    x: xoffset,
                    y: 0.0,
                    z: zoffset,
                })
                - position;

            let vector_to_light = vector_to_light.normalized();

            let reflection_vector = direction - n * 2.0 * (n.dot(direction));
            let reflection_colour =
                self.cast_sight_ray(position + (reflection_vector * 0.002), reflection_vector);

            r = (vector_to_light.dot(n).max(0.5) * 0.8 * reflection_colour[0] as f32).round() as u8;
            g = (vector_to_light.dot(n).max(0.5) * 0.8 * reflection_colour[1] as f32).round() as u8;
            b = (vector_to_light.dot(n).max(0.5) * 0.8 * reflection_colour[2] as f32).round() as u8;
        }
        [r, g, b]
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

    fn march(&self, ray: &mut Ray) -> Option<ObjectRef> {
        let mut distance_travelled = 0.0;

        while distance_travelled < MAX_MARCH_DISTANCE {
            let mut distance = f32::INFINITY;
            let mut closest_object: Option<ObjectRef> = None;

            for object in &self.objects {
                let obj_distance = object.sdf(ray.position);
                if obj_distance < distance {
                    distance = obj_distance;
                    closest_object = Some(*object);
                };
            }

            assert_eq!(closest_object.is_some(), true);

            distance_travelled += distance;
            ray.position = ray.position + (ray.direction * distance);

            if distance < 0.001 {
                return closest_object;
            }
        }
        return None;
    }

    fn march_light(&self, ray: &mut Ray, this_object: ObjectRef) -> Option<ObjectRef> {
        let mut distance_travelled = 0.0;

        while distance_travelled < MAX_MARCH_DISTANCE {
            let mut distance = f32::INFINITY;
            let mut closest_object: Option<ObjectRef> = None;

            for object in &self.objects {
                if std::ptr::eq(this_object, *object) {
                    continue;
                } else {
                    let obj_distance = object.sdf(ray.position);
                    if obj_distance < distance {
                        distance = obj_distance;
                        closest_object = Some(*object);
                    };
                }
            }

            distance_travelled += distance;
            ray.position = ray.position + (ray.direction * distance);

            if distance < 0.001 {
                return closest_object;
            }
        }
        return None;
    }
}

pub struct Engine<'a> {
    pub objects: Vec<ObjectRef<'a>>,
    pub camera_position: Vec3,
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

const LIGHT_POS: Vec3 = Vec3 {
    x: -5.0,
    y: 5.0,
    z: -5.0,
};

const MAX_MARCH_DISTANCE: f32 = 50.0;
