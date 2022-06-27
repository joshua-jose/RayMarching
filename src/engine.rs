use super::objects::HasDistanceFunction;
use super::vector::Vec3;

pub const WIDTH: usize = 400;
pub const HEIGHT: usize = 400;

type ObjectRef<'a> = &'a dyn HasDistanceFunction;

static mut N: i32 = 0;

impl Engine<'_> {
    pub fn render(&mut self, buffer: &mut [u8], _width: usize) {
        let zoffset: f32;
        let xoffset: f32;
        unsafe {
            N += 1;
            //self.camera_position.y = 2.0 + 3.0 * (0.1 * N as f32).sin();
            //self.camera_position.x = 2.0 * (0.1 * N as f32).cos();
            zoffset = 3.0 + 6.0 * (0.1 * N as f32).sin();
            xoffset = 3.0 + 6.0 * (0.1 * N as f32).cos();
        }

        for y_inv in 0..HEIGHT {
            for x in 0..WIDTH {
                let y = HEIGHT - y_inv;
                let u = (2.0 * x as f32 / WIDTH as f32) - 1.0;
                let v = (2.0 * y as f32 / HEIGHT as f32) - 1.0;
                let i = y_inv * WIDTH + x;

                let mut ray = Ray {
                    position: self.camera_position,
                    direction: Vec3 { x: u, y: v, z: 1.0 }.normalized(),
                };

                let object = self.march(&mut ray);

                let mut r = 135;
                let mut g = 206;
                let mut b = 235;

                if object.is_some() {
                    let n = Engine::calculate_normal(ray.position, object.unwrap());

                    let vector_to_light = (LIGHT_POS
                        + Vec3 {
                            x: xoffset,
                            y: 0.0,
                            z: zoffset,
                        })
                        - ray.position;
                    let distance_to_light = vector_to_light.mag();
                    let vector_to_light = vector_to_light.normalized();

                    let light_intensity = 50.0 * distance_to_light.powi(2).recip();

                    let mut light_ray = Ray {
                        position: ray.position + (vector_to_light * 0.002),
                        direction: vector_to_light,
                    };
                    let light_intersection = self.march(&mut light_ray);
                    let diffuse: f32;
                    match light_intersection {
                        Some(o) => {
                            if !std::ptr::eq(o, object.unwrap()) {
                                diffuse = 0.05 * vector_to_light.dot(n);
                            } else {
                                diffuse = light_intensity * vector_to_light.dot(n).max(0.05);
                            }
                        }
                        None => diffuse = light_intensity * vector_to_light.dot(n).max(0.05),
                    }

                    if std::ptr::eq(self.objects[0], object.unwrap()) {
                        r = (245.0 * diffuse).floor() as u8;
                        g = (104.0 * diffuse).floor() as u8;
                        b = (44.0 * diffuse).floor() as u8;
                    } else {
                        r = (138.0 * diffuse).floor() as u8;
                        g = (245.0 * diffuse).floor() as u8;
                        b = (44.0 * diffuse).floor() as u8;
                    }
                }

                buffer[i * 4 + 0] = b;
                buffer[i * 4 + 1] = g;
                buffer[i * 4 + 2] = r;
                //buffer[i * 4 + 3] = 0;
            }
        }
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

        while distance_travelled < 50.0 {
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
}

pub struct Engine<'a> {
    pub objects: Vec<&'a dyn HasDistanceFunction>,
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
