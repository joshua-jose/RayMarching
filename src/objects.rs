use super::radiosity::MAP_SIZE;

use super::colour::bilinear_interpolation;
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

    // for a given world position, return the uv coordinates on the texture
    fn sample_uv_from_pos(&self, _pos: Vec3) -> (f32, f32) {
        unimplemented!()
    }

    // for a given world position, sample the lightmap at that point
    fn sample(&self, _pos: Vec3) -> Colour {
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

        fn sample(&self, pos: Vec3) -> Colour {
            let (u, v) = self.sample_uv_from_pos(pos);

            let (u0, v0) = (u.floor() as usize, v.floor() as usize);
            let (u1, v1) = (u0 + 1, v0 + 1);

            let u1 = if u1 >= MAP_SIZE { u0 } else { u1 };
            let v1 = if v1 >= MAP_SIZE { v0 } else { v1 };

            let sample00 = self.lightmap.sample_map[u0][v0];
            let sample01 = self.lightmap.sample_map[u0][v1];
            let sample10 = self.lightmap.sample_map[u1][v0];
            let sample11 = self.lightmap.sample_map[u1][v1];

            /* when creating the array of points, always act as if the sample is from the right/up
               even if it wasn't
            */
            let (u1, v1) = (u0 + 1, v0 + 1);

            let points = [
                (u0 as f32, v0 as f32, sample00),
                (u0 as f32, v1 as f32, sample01),
                (u1 as f32, v0 as f32, sample10),
                (u1 as f32, v1 as f32, sample11),
            ];
            bilinear_interpolation(u, v, &points)
            //sample00
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
            x: (u as f32) - ((MAP_SIZE as f32) / 2.0),
            y: self.y,
            z: (v as f32) - ((MAP_SIZE as f32) / 2.0),
        }
    }
    fn sample_uv_from_pos(&self, pos: Vec3) -> (f32, f32) {
        (pos.x + (MAP_SIZE as f32 / 2.0), pos.z + (MAP_SIZE as f32 / 2.0))
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
            y: (u as f32) - ((MAP_SIZE as f32) / 2.0),
            z: (v as f32) - ((MAP_SIZE as f32) / 2.0),
        }
    }
    fn sample_uv_from_pos(&self, pos: Vec3) -> (f32, f32) {
        (pos.y + (MAP_SIZE as f32 / 2.0), pos.z + (MAP_SIZE as f32 / 2.0))
    }
}

impl EngineObject for ZPlane {
    fn sdf(&self, position: Vec3) -> f32 {
        self.dir * (position.z - self.z)
    }

    plane_funcs!();
    fn get_sample_pos(&self, u: usize, v: usize) -> Vec3 {
        Vec3 {
            x: (u as f32) - ((MAP_SIZE as f32) / 2.0),
            y: (v as f32) - ((MAP_SIZE as f32) / 2.0),
            z: self.z,
        }
    }
    fn sample_uv_from_pos(&self, pos: Vec3) -> (f32, f32) {
        (pos.x + (MAP_SIZE as f32 / 2.0), pos.y + (MAP_SIZE as f32 / 2.0))
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
