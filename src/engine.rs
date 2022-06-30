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

            zoffset = 3.0 + 6.0 * (0.1 * N as f32).sin();
            xoffset = 3.0 + 6.0 * (0.1 * N as f32).cos();
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
                let i = y_inv * WIDTH + x;

                let colour: [u8; 3] = self.cast_sight_ray(
                    self.camera_position,
                    Vec3 { x: u, y: v, z: 1.0 }.normalized(),
                );

                // perform sRGB colour corrections
                buffer[i * 4 + 0] = GAMMA_LUT[colour[2] as usize];
                buffer[i * 4 + 1] = GAMMA_LUT[colour[1] as usize];
                buffer[i * 4 + 2] = GAMMA_LUT[colour[0] as usize];
                //buffer[i * 4 + 3] = 0;
            }
        }
    }

    fn cast_sight_ray(&self, position: Vec3, direction: Vec3) -> [u8; 3] {
        let colour: [u8; 3];

        let mut ray = Ray {
            position,
            direction,
        };
        // object the sight ray hit
        let object = self.march(&mut ray, None);

        // if we hit an object, colour this pixel
        if object.is_some() {
            colour = self.shade_object(object.unwrap(), ray.position, direction);
        } else {
            // sky colour
            colour = [135, 206, 235];
        }
        colour
    }

    fn shade_object(&self, object: ObjectRef, position: Vec3, direction: Vec3) -> [u8; 3] {
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
        let vector_to_light = vector_to_light / distance_to_light_sqd.sqrt();
        let light_reflection_vector = (-vector_to_light).reflect(n);
        let light_intensity = self.light.get_intensity() / distance_to_light_sqd; // k/d^2

        // cast a shadow ray to see if this point is blocked by another object
        let mut shadow_ray = Ray {
            position,
            direction: vector_to_light,
        };
        let light_intersection = self.march(&mut shadow_ray, Some(object));

        // if this point is blocked by some other object, do not light it.
        match light_intersection {
            Some(_) => {
                diffuse = 0.0;
                specular = 0.0;
            }
            None => {
                // Phong shading algorithm
                diffuse = object.diffuse() * light_intensity * vector_to_light.dot(n).max(0.0);
                if diffuse > 0.0 {
                    specular = object.specular()
                        * light_intensity
                        * light_reflection_vector
                            .dot(-direction)
                            .max(0.0)
                            .powf(object.shininess());
                } else {
                    specular = 0.0;
                }
            }
        }

        r = object_colour[0] as f32 * (diffuse + ambient + specular);
        g = object_colour[1] as f32 * (diffuse + ambient + specular);
        b = object_colour[2] as f32 * (diffuse + ambient + specular);

        // if the object is reflective, cast a reflection ray
        if object.reflectivity() > 1e-3 {
            let reflection_vector = direction.reflect(n);
            let reflection_colour =
                self.cast_sight_ray(position + (reflection_vector * 0.002), reflection_vector);

            r += 0.9 * reflection_colour[0] as f32;
            g += 0.9 * reflection_colour[1] as f32;
            b += 0.9 * reflection_colour[2] as f32;
        }

        [r.round() as u8, g.round() as u8, b.round() as u8]
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

            if distance < 0.001 {
                return closest_object;
            }
        }
        return None;
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

// mapping of linear luminosity to sRGB colour space
const GAMMA_LUT: [u8; 256] = [
    0, 21, 28, 34, 39, 43, 46, 50, 53, 56, 59, 61, 64, 66, 68, 70, 72, 74, 76, 78, 80, 82, 84, 85,
    87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 103, 105, 106, 107, 109, 110, 111, 112, 114, 115,
    116, 117, 118, 119, 120, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 135,
    136, 137, 138, 139, 140, 141, 142, 143, 144, 144, 145, 146, 147, 148, 149, 150, 151, 151, 152,
    153, 154, 155, 156, 156, 157, 158, 159, 160, 160, 161, 162, 163, 164, 164, 165, 166, 167, 167,
    168, 169, 170, 170, 171, 172, 173, 173, 174, 175, 175, 176, 177, 178, 178, 179, 180, 180, 181,
    182, 182, 183, 184, 184, 185, 186, 186, 187, 188, 188, 189, 190, 190, 191, 192, 192, 193, 194,
    194, 195, 195, 196, 197, 197, 198, 199, 199, 200, 200, 201, 202, 202, 203, 203, 204, 205, 205,
    206, 206, 207, 207, 208, 209, 209, 210, 210, 211, 212, 212, 213, 213, 214, 214, 215, 215, 216,
    217, 217, 218, 218, 219, 219, 220, 220, 221, 221, 222, 223, 223, 224, 224, 225, 225, 226, 226,
    227, 227, 228, 228, 229, 229, 230, 230, 231, 231, 232, 232, 233, 233, 234, 234, 235, 235, 236,
    236, 237, 237, 238, 238, 239, 239, 240, 240, 241, 241, 242, 242, 243, 243, 244, 244, 245, 245,
    246, 246, 247, 247, 248, 248, 249, 249, 249, 250, 250, 251, 251, 252, 252, 253, 253, 254, 254,
    255, 255,
];
