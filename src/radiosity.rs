use std::f32::consts::PI;

use super::vector::Vec3;
use crate::{
    engine::ObjectRef,
    objects::{EngineLight, PointLight},
};

use Vec3 as Colour;

pub const MAP_SIZE: usize = 8;

#[derive(Copy, Clone, Debug, Default)]
pub struct Lightmap {
    pub sample_map: [[Colour; MAP_SIZE]; MAP_SIZE], // 2D array of colour, dim MAP_SIZE*MAP_SIZE
}

#[allow(dead_code)]
pub struct SampleCloud {
    /// position of each sample point
    point_cloud:  Vec<[[Vec3; MAP_SIZE]; MAP_SIZE]>,
    /// map of every sample point's colour
    colour_cloud: Vec<[[Colour; MAP_SIZE]; MAP_SIZE]>,
}

impl SampleCloud {
    #[allow(dead_code)]
    pub fn new(len: usize) -> Self {
        let mut point_cloud: Vec<[[Vec3; MAP_SIZE]; MAP_SIZE]> = vec![];
        point_cloud.resize(len, [[Vec3::default(); MAP_SIZE]; MAP_SIZE]);

        let mut colour_cloud: Vec<[[Colour; MAP_SIZE]; MAP_SIZE]> = vec![];
        colour_cloud.resize(len, [[Colour::default(); MAP_SIZE]; MAP_SIZE]);

        Self {
            point_cloud,
            colour_cloud,
        }
    }
}

pub struct Sample {
    pub pos:             Vec3,
    pub colour:          Vec3,
    pub normal:          Vec3,
    pub obj_cloud_index: usize,
}

/// Compute the direct lighting on a lightmap
pub fn compute_direct_lighting(
    light: &PointLight, obj_indexes: &Vec<usize>, objects: &Vec<ObjectRef>,
    point_cloud: &Vec<[[Vec3; MAP_SIZE]; MAP_SIZE]>, colour_cloud: &Vec<[[Colour; MAP_SIZE]; MAP_SIZE]>,
) -> Vec<Lightmap> {
    let light_pos = light.get_position();
    let light_intensity = light.get_intensity();

    let mut emissive_maps: Vec<Lightmap> = Vec::new();
    emissive_maps.reserve(obj_indexes.len());

    for (cloud_index, &obj_index) in obj_indexes.iter().enumerate() {
        let object = &objects[obj_index];
        let mut emissive_map = Lightmap::default();

        for x in 0..MAP_SIZE {
            for y in 0..MAP_SIZE {
                let origin = point_cloud[cloud_index][x][y];
                let colour = colour_cloud[cloud_index][x][y];

                let vector_to_light = light_pos - origin;
                let distance_to_light = vector_to_light.mag();
                let vector_to_light = vector_to_light / distance_to_light;

                /*
                let mut shadow_ray = Ray {
                    position:  origin,
                    direction: vector_to_light,
                };
                let _ = shadow_ray.march(&self.objects, Some(obj_index));

                if (shadow_ray.position - origin).mag() >= distance_to_light {
                    let n = object.calculate_normal(origin);
                    let diffuse =
                        n.dot(shadow_ray.direction).max(0.0) * light_intensity / (distance_to_light + 1.0).powi(2);

                    emissive_map.sample_map[x][y] = object.colour(origin) * diffuse;
                }
                */

                let n = object.calculate_normal(origin);
                let diffuse = n.dot(vector_to_light).max(0.0) * light_intensity / (distance_to_light + 1.0).powi(2);

                emissive_map.sample_map[x][y] = colour * diffuse;
            }
        }
        emissive_maps.push(emissive_map);
    }
    emissive_maps
}

pub fn compute_patch_radiosity(
    point_cloud: &Vec<[[Vec3; MAP_SIZE]; MAP_SIZE]>, emissive_maps: &Vec<Lightmap>, sample: Sample,
) -> Colour {
    let mut incident: Colour = Colour::new(0.0, 0.0, 0.0);

    // go through all other lightmaps
    for lighting_cloud_index in 0..point_cloud.len() {
        // no self illumination
        if sample.obj_cloud_index == lighting_cloud_index {
            continue;
        }
        // get light output from this lightmap
        let lighting_lightmap = emissive_maps[lighting_cloud_index];

        // go through each patch in the lightmap
        for a in 0..MAP_SIZE {
            for b in 0..MAP_SIZE {
                // get the position and emission of the patch
                let light_source = point_cloud[lighting_cloud_index][a][b];
                let light_colour = lighting_lightmap.sample_map[a][b];

                let vector_to_light = light_source - sample.pos;
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
                // compute lambertian attenuation of light from one patch to another
                let diffuse = sample.normal.dot(vector_to_light).max(0.0) * 1.0 / (distance_to_light + 1.0).powi(2);
                incident += light_colour.element_mul(sample.colour) * diffuse;
            }
        }
    }
    incident
}

pub fn compute_object_radiosity(
    object: &ObjectRef, obj_cloud_index: usize, point_cloud: &Vec<[[Vec3; MAP_SIZE]; MAP_SIZE]>,
    colour_cloud: &Vec<[[Colour; MAP_SIZE]; MAP_SIZE]>, emissive_maps: &Vec<Lightmap>,
) -> (Lightmap, Lightmap) {
    let mut obj_lightmap = object.get_lightmap().unwrap().clone();
    let mut new_emissive_map = Lightmap::default();

    // go through each patch in the object's lightmap
    for x in 0..MAP_SIZE {
        for y in 0..MAP_SIZE {
            // get the position, colour, normal of the patch
            let patch_pos = point_cloud[obj_cloud_index][x][y];
            let obj_pos_colour = colour_cloud[obj_cloud_index][x][y];
            let n = object.calculate_normal(patch_pos);

            let sample = Sample {
                pos: patch_pos,
                colour: obj_pos_colour,
                normal: n,
                obj_cloud_index,
            };

            // compute the radiosity of this patch
            let incident: Colour = compute_patch_radiosity(&point_cloud, &emissive_maps, sample);
            // "because calculus"
            obj_lightmap.sample_map[x][y] += incident.element_mul(incident) / PI;
            new_emissive_map.sample_map[x][y] = incident.element_mul(incident) / PI;
        }
    }
    (obj_lightmap, new_emissive_map)
}
