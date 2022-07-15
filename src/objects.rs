use crate::radiosity::MAP_SIZE;

use super::material::Material;
use super::radiosity::Lightmap;
use super::vector::Vec3;
use Vec3 as Colour;

pub trait EngineObject {
    fn sdf(&self, position: Vec3) -> f32;
    fn colour(&self, position: Vec3) -> Colour;
    fn material(&self) -> Material;

    // all objects have a default implementation of no lightmap
    fn get_lightmap(&self) -> Option<&Lightmap> {
        None
    }
    fn set_lightmap(&mut self, _new_lightmap: Lightmap) {}
    fn clear_lightmap(&mut self) {}

    // for a given uv coordinate, get the world space coordinate
    fn get_sample_pos(&self, _u: usize, _v: usize) -> Vec3 {
        unimplemented!()
    }
    // for a given world position, sample the lightmap at that point
    fn sample(&self, _pos: Vec3) -> (usize, usize) {
        unimplemented!()
    }
}

pub trait EngineLight {
    fn get_position(&self) -> Vec3;
    fn get_intensity(&self) -> f32;
}

#[derive(Clone, Copy)]
pub struct Sphere {
    pub position: Vec3,
    pub radius: f32,
    pub material: Material,
    pub colour: Colour,
}

#[derive(Clone, Copy, Default)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,

    pub material: Material,
    pub colour: Colour,
    pub lightmap: Lightmap,
}

#[derive(Clone, Copy, Default)]
pub struct XPlane {
    pub x: f32,
    pub dir: f32,
    pub material: Material,
    pub colour: Colour,
    pub lightmap: Lightmap,
}

#[derive(Clone, Copy, Default)]
pub struct YPlane {
    pub y: f32,
    pub dir: f32,
    pub material: Material,
    pub colour: Colour,
    pub lightmap: Lightmap,
}

#[derive(Clone, Copy, Default)]
pub struct ZPlane {
    pub z: f32,
    pub dir: f32,
    pub material: Material,
    pub colour: Colour,
    pub lightmap: Lightmap,
}

#[derive(Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub intensity: f32,
}

macro_rules! plane_funcs {
    () => {
        fn colour(&self, _position: Vec3) -> Colour {
            self.colour
        }
        fn material(&self) -> Material {
            self.material
        }

        // all objects have a default implementation of no lightmap
        fn get_lightmap(&self) -> Option<&Lightmap> {
            Some(&self.lightmap)
        }
        fn set_lightmap(&mut self, new_lightmap: Lightmap) {
            self.lightmap = new_lightmap;
        }
        fn clear_lightmap(&mut self) {
            self.lightmap = Lightmap::default()
        }
    };
}

impl XPlane {
    pub fn new(x: f32, dir: f32, material: Material, colour: Colour) -> Self {
        Self {
            x,
            dir,
            material,
            colour,
            lightmap: Lightmap::default(),
        }
    }
}
impl YPlane {
    pub fn new(y: f32, dir: f32, material: Material, colour: Colour) -> Self {
        Self {
            y,
            dir,
            material,
            colour,
            lightmap: Lightmap::default(),
        }
    }
}
impl ZPlane {
    pub fn new(z: f32, dir: f32, material: Material, colour: Colour) -> Self {
        Self {
            z,
            dir,
            material,
            colour,
            lightmap: Lightmap::default(),
        }
    }
}

impl EngineObject for Plane {
    fn sdf(&self, position: Vec3) -> f32 {
        position.dot(self.normal) - self.distance
    }

    plane_funcs!();
}

impl EngineObject for YPlane {
    fn sdf(&self, position: Vec3) -> f32 {
        self.dir * (position.y - self.y)
    }

    plane_funcs!();
    fn get_sample_pos(&self, u: usize, v: usize) -> Vec3 {
        Vec3 {
            x: (u as f32) - ((MAP_SIZE as f32) / 2.0) + 0.5,
            y: self.y,
            z: (v as f32) - ((MAP_SIZE as f32) / 2.0) + 0.5,
        }
    }
    fn sample(&self, pos: Vec3) -> (usize, usize) {
        (
            (pos.x.floor() + (MAP_SIZE as f32 / 2.0)) as usize,
            (pos.z.floor() + (MAP_SIZE as f32 / 2.0)) as usize,
        )
    }
}

impl EngineObject for XPlane {
    fn sdf(&self, position: Vec3) -> f32 {
        self.dir * (position.x - self.x)
    }

    plane_funcs!();
    fn get_sample_pos(&self, u: usize, v: usize) -> Vec3 {
        Vec3 {
            x: self.x,
            y: (u as f32) - ((MAP_SIZE as f32) / 2.0) + 0.5,
            z: (v as f32) - ((MAP_SIZE as f32) / 2.0) + 0.5,
        }
    }
    fn sample(&self, pos: Vec3) -> (usize, usize) {
        (
            (pos.y.floor() + (MAP_SIZE as f32 / 2.0)) as usize,
            (pos.z.floor() + (MAP_SIZE as f32 / 2.0)) as usize,
        )
    }
}

impl EngineObject for ZPlane {
    fn sdf(&self, position: Vec3) -> f32 {
        self.dir * (position.z - self.z)
    }

    plane_funcs!();
    fn get_sample_pos(&self, u: usize, v: usize) -> Vec3 {
        Vec3 {
            x: (u as f32) - ((MAP_SIZE as f32) / 2.0) + 0.5,
            y: (v as f32) - ((MAP_SIZE as f32) / 2.0) + 0.5,
            z: self.z,
        }
    }
    fn sample(&self, pos: Vec3) -> (usize, usize) {
        (
            (pos.x.floor() + (MAP_SIZE as f32 / 2.0)) as usize,
            (pos.y.floor() + (MAP_SIZE as f32 / 2.0)) as usize,
        )
    }
}

impl EngineObject for Sphere {
    fn sdf(&self, position: Vec3) -> f32 {
        (position - self.position).mag() - self.radius
    }

    fn colour(&self, _position: Vec3) -> Colour {
        self.colour
    }
    fn material(&self) -> Material {
        self.material
    }
}

// box
/*
let p = Vec3 {
    x: position.x.abs(),
    y: position.y.abs(),
    z: position.z.abs(),
};
p.x.max(p.y.max(p.z)) - 1.0 + p.dot(p) * 0.2
*/

impl EngineLight for PointLight {
    fn get_position(&self) -> Vec3 {
        self.position
    }

    fn get_intensity(&self) -> f32 {
        self.intensity
    }
}
