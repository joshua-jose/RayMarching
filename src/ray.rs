use crate::{
    engine::{MAX_MARCH_DISTANCE, MAX_SHAD_IT, SMALL_DISTANCE},
    objects::EngineObject,
    vector::Vec3,
};

type ObjectRef = Box<dyn EngineObject>;

pub struct Ray {
    pub position: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn march(&mut self, objects: &Vec<ObjectRef>, ignore_object: Option<usize>) -> Option<usize> {
        let mut distance_travelled = 0.0;

        while distance_travelled < MAX_MARCH_DISTANCE {
            let mut distance = f32::INFINITY;
            let mut closest_object: usize = usize::MAX;

            for (i, object) in objects.iter().enumerate() {
                if ignore_object.unwrap_or(usize::MAX) == i {
                    continue;
                }

                let obj_distance = object.sdf(self.position);
                if obj_distance < distance {
                    distance = obj_distance;
                    closest_object = i;
                };
            }
            if distance < SMALL_DISTANCE {
                return Some(closest_object);
            }

            distance_travelled += distance;
            self.position += self.direction * distance;
        }
        return None;
    }

    pub fn smooth_shadow_march(
        &mut self, objects: &Vec<ObjectRef>, ignore_obj_index: usize, light_dist: f32, shading_k: f32,
    ) -> f32 {
        let mut distance_travelled = 0.0;
        let mut shade: f32 = 1.0; // actually the amount of "not shade"

        let smoothstep = |x: f32| 3.0 * x.powi(2) - 2.0 * x.powi(3);

        for _ in 0..MAX_SHAD_IT {
            let mut distance = f32::INFINITY;

            for (i, object) in objects.iter().enumerate() {
                if ignore_obj_index == i {
                    continue;
                }

                let obj_distance = object.sdf(self.position);
                if obj_distance < distance {
                    distance = obj_distance;
                };
            }

            shade = shade.min(smoothstep((shading_k * distance / distance_travelled).clamp(0.0, 1.0)));
            //distance = distance.clamp(SMALL_DISTANCE, light_dist / MAX_SHAD_IT as f32);
            distance_travelled += distance; // could clamp this for better res
            self.position += self.direction * distance;
            if distance < SMALL_DISTANCE || distance_travelled > light_dist {
                break;
            }
        }
        shade.clamp(0.0, 1.0)
    }
}
