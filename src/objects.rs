use std::f32::consts::{PI, TAU};

use super::radiosity::MAP_SIZE;

use super::colour::bilinear_interpolation;
use super::material::Material;
use super::radiosity::Lightmap;
use super::vector::Vec3;
use Vec3 as Colour;

const STEP_SIZE: f32 = 0.0001;
const X_STEP: Vec3 = Vec3::new(STEP_SIZE, 0.0, 0.0);
const Y_STEP: Vec3 = Vec3::new(0.0, STEP_SIZE, 0.0);
const Z_STEP: Vec3 = Vec3::new(0.0, 0.0, STEP_SIZE);

/*
const TETRA_STEP: f32 = 0.0015;
const STEP_A: Vec3 = Vec3::new(TETRA_STEP, -TETRA_STEP, -TETRA_STEP);
const STEP_B: Vec3 = Vec3::new(-TETRA_STEP, -TETRA_STEP, TETRA_STEP);
const STEP_C: Vec3 = Vec3::new(-TETRA_STEP, TETRA_STEP, -TETRA_STEP);
const STEP_D: Vec3 = Vec3::new(TETRA_STEP, TETRA_STEP, TETRA_STEP);
*/

pub trait EngineObject {
    fn sdf(&self, position: Vec3) -> f32;
    fn colour(&self, position: Vec3) -> Colour;
    fn material(&self) -> &Material;

    fn radiosity_collide(&self) -> bool { false }

    // all objects have a default implementation of no lightmap
    fn get_lightmap(&self) -> Option<&Lightmap> { None }
    fn set_lightmap(&mut self, _new_lightmap: Lightmap) {}
    fn clear_lightmap(&mut self) {}

    // for a given uv coordinate, get the world space coordinate
    fn get_sample_pos(&self, _u: usize, _v: usize) -> Vec3 { unimplemented!() }

    // for a given world position, return the uv coordinates on the texture
    fn sample_uv_from_pos(&self, _pos: Vec3) -> (f32, f32) { unimplemented!() }

    // for a given world position, sample the lightmap at that point
    fn sample_lightmap(&self, pos: Vec3) -> Colour {
        let (u, v) = self.sample_uv_from_pos(pos);

        let (mut u0, mut v0) = ((u.floor().max(0.0)) as usize, (v.floor().max(0.0)) as usize);
        let (mut u1, mut v1) = (u0 + 1, v0 + 1);

        u1 = if u1 >= MAP_SIZE { u0 } else { u1 };
        v1 = if v1 >= MAP_SIZE { v0 } else { v1 };

        // if UV coords exceed lightmap boundaries, extrapolate from previous luxel and current one.
        /*
        if u1 >= MAP_SIZE {
            u0 -= 1;
            u1 -= 1;
        }
        if v1 >= MAP_SIZE {
            v0 -= 1;
            v1 -= 1;
        }
        */

        let lightmap = self.get_lightmap().unwrap();

        let mut sample00 = lightmap.sample_map[u0][v0];
        let mut sample01 = lightmap.sample_map[u0][v1];
        let mut sample10 = lightmap.sample_map[u1][v0];
        let mut sample11 = lightmap.sample_map[u1][v1];

        let mut resample = false;

        if sample00.mag_sqd() == 0.0 {
            resample = true;
            if sample10.mag_sqd() == 0.0 {
                v0 += 1;
                v1 += 1;
            } else if sample11.mag_sqd() == 0.0 {
                v0 -= 1;
                v1 -= 1;
            }

            if sample01.mag_sqd() == 0.0 {
                u0 += 1;
                u1 += 1;
            } else if sample11.mag_sqd() == 0.0 {
                u0 -= 1;
                u1 -= 1;
            }
        } else {
            // right side luxel is empty, so it is obstructed
            if sample10.mag_sqd() == 0.0 {
                resample = true;
                u1 -= 1;
            }
            // above luxel is empty, so it is obstructed
            if sample01.mag_sqd() == 0.0 {
                resample = true;
                v1 -= 1;
            }
        }

        if resample {
            sample00 = lightmap.sample_map[u0][v0];
            sample01 = lightmap.sample_map[u0][v1];
            sample10 = lightmap.sample_map[u1][v0];
            sample11 = lightmap.sample_map[u1][v1];
        }

        /* when creating the array of points, always act as if the sample is from the right/up
           even if it wasn't
        */
        if u0 == u1 {
            u1 += 1
        };
        if v0 == v1 {
            v1 += 1
        };

        let mut points = [
            (u0 as f32, v0 as f32, sample00),
            (u0 as f32, v1 as f32, sample01),
            (u1 as f32, v0 as f32, sample10),
            (u1 as f32, v1 as f32, sample11),
        ];

        bilinear_interpolation(u, v, &mut points)
        //sample00
    }

    fn calculate_normal(&self, position: Vec3) -> Vec3 {
        let gradient_x = self.sdf(position + X_STEP) - self.sdf(position - X_STEP);
        let gradient_y = self.sdf(position + Y_STEP) - self.sdf(position - Y_STEP);
        let gradient_z = self.sdf(position + Z_STEP) - self.sdf(position - Z_STEP);

        Vec3::new(gradient_x, gradient_y, gradient_z).normalized()

        /*
        let norm = STEP_A * self.sdf(position + STEP_A)
            + STEP_B * self.sdf(position + STEP_B)
            + STEP_C * self.sdf(position + STEP_C)
            + STEP_D * self.sdf(position + STEP_D);

        norm.normalized()
        */
    }
}

pub trait EngineLight {
    fn get_position(&self) -> Vec3;
    fn get_intensity(&self) -> f32;
}

#[derive(Clone, Copy)]
pub struct Sphere {
    pub position: Vec3,
    pub radius:   f32,
    pub material: Material,
    pub colour:   Colour,
    pub lightmap: Lightmap,
}

#[derive(Clone, Copy, Default)]
pub struct Plane {
    pub normal:   Vec3,
    pub distance: f32,

    pub material: Material,
    pub colour:   Colour,
    pub lightmap: Lightmap,
}

#[derive(Clone, Copy, Default)]
pub struct XPlane {
    pub x:        f32,
    pub dir:      f32,
    pub material: Material,
    pub colour:   Colour,
    pub lightmap: Lightmap,
}

#[derive(Clone, Copy, Default)]
pub struct YPlane {
    pub y:        f32,
    pub dir:      f32,
    pub material: Material,
    pub colour:   Colour,
    pub lightmap: Lightmap,
}

#[derive(Clone, Copy, Default)]
pub struct ZPlane {
    pub z:        f32,
    pub dir:      f32,
    pub material: Material,
    pub colour:   Colour,
    pub lightmap: Lightmap,
}

#[derive(Clone, Copy)]
pub struct PointLight {
    pub position:  Vec3,
    pub intensity: f32,
}

macro_rules! plane_funcs {
    () => {
        fn material(&self) -> &Material { &self.material }

        // all objects have a default implementation of no lightmap
        fn get_lightmap(&self) -> Option<&Lightmap> { Some(&self.lightmap) }
        fn set_lightmap(&mut self, new_lightmap: Lightmap) { self.lightmap = new_lightmap; }
        fn clear_lightmap(&mut self) { self.lightmap = Lightmap::default() }

        fn radiosity_collide(&self) -> bool { true }
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
    fn colour(&self, _position: Vec3) -> Colour { self.colour }
    fn sdf(&self, position: Vec3) -> f32 { position.dot(self.normal) - self.distance }

    plane_funcs!();
}

impl EngineObject for YPlane {
    fn colour(&self, _position: Vec3) -> Colour {
        /*
        if self.y > 0.0 {
            self.colour
        } else {
            let (u, v) = self.sample_uv_from_pos(_position);
            // texture is 128x128, and we want 16 pixels per world unit
            let (u, v) = (
                ((u * 32.0).floor() % 128.0) as usize,
                ((v * 32.0).floor() % 128.0) as usize,
            );

            // 5 bit texture
            let col: u16 = WOOD_TEX[v][u];
            let (r, g, b) = (col >> 11 & 0x1F, col >> 6 & 0x1F, col & 0x1F);
            //let (r, g, b) = (r as f32 / 31.0, g as f32 / 31.0, b as f32 / 31.0);
            let (r, g, b) = (
                255.0 * r as f32 / 31.0,
                255.0 * g as f32 / 31.0,
                255.0 * b as f32 / 31.0,
            );

            rgb!(r, g, b)
        }
        */
        self.colour
    }
    fn sdf(&self, position: Vec3) -> f32 { self.dir * (position.y() - self.y) }

    plane_funcs!();
    fn get_sample_pos(&self, u: usize, v: usize) -> Vec3 {
        Vec3::new(
            (u as f32) - ((MAP_SIZE as f32) / 2.0) + 0.5,
            self.y,
            (v as f32) - ((MAP_SIZE as f32) / 2.0) + 0.5,
        )
    }
    fn sample_uv_from_pos(&self, pos: Vec3) -> (f32, f32) {
        (
            pos.x() + (MAP_SIZE as f32 / 2.0) - 0.5,
            pos.z() + (MAP_SIZE as f32 / 2.0) - 0.5,
        )
    }
}

impl EngineObject for XPlane {
    fn colour(&self, _position: Vec3) -> Colour { self.colour }
    fn sdf(&self, position: Vec3) -> f32 { self.dir * (position.x() - self.x) }

    plane_funcs!();
    fn get_sample_pos(&self, u: usize, v: usize) -> Vec3 {
        Vec3::new(
            self.x,
            (u as f32) - ((MAP_SIZE as f32) / 2.0) + 0.5,
            (v as f32) - ((MAP_SIZE as f32) / 2.0) + 0.5,
        )
    }
    fn sample_uv_from_pos(&self, pos: Vec3) -> (f32, f32) {
        (
            pos.y() + (MAP_SIZE as f32 / 2.0) - 0.5,
            pos.z() + (MAP_SIZE as f32 / 2.0) - 0.5,
        )
    }
}

impl EngineObject for ZPlane {
    fn colour(&self, _position: Vec3) -> Colour { self.colour }
    fn sdf(&self, position: Vec3) -> f32 { self.dir * (position.z() - self.z) }

    plane_funcs!();
    fn get_sample_pos(&self, u: usize, v: usize) -> Vec3 {
        Vec3::new(
            (u as f32) - ((MAP_SIZE as f32) / 2.0) + 0.5,
            (v as f32) - ((MAP_SIZE as f32) / 2.0) + 0.5,
            self.z,
        )
    }
    fn sample_uv_from_pos(&self, pos: Vec3) -> (f32, f32) {
        (
            pos.x() + (MAP_SIZE as f32 / 2.0) - 0.5,
            pos.y() + (MAP_SIZE as f32 / 2.0) - 0.5,
        )
    }
}

impl EngineObject for Sphere {
    fn sdf(&self, position: Vec3) -> f32 { (position - self.position).mag() - self.radius }

    fn colour(&self, _position: Vec3) -> Colour { self.colour }
    fn material(&self) -> &Material { &self.material }

    fn get_lightmap(&self) -> Option<&Lightmap> { Some(&self.lightmap) }
    fn set_lightmap(&mut self, new_lightmap: Lightmap) { self.lightmap = new_lightmap; }
    fn clear_lightmap(&mut self) { self.lightmap = Lightmap::default() }

    fn radiosity_collide(&self) -> bool { true }

    fn get_sample_pos(&self, u: usize, v: usize) -> Vec3 {
        let theta = (((u as f32 / MAP_SIZE as f32) * 2.0) - 1.0) * PI;
        let phi = ((v as f32 / MAP_SIZE as f32) - 0.5) * PI;

        let n = Vec3::new(theta.sin(), phi.sin(), -theta.cos());

        self.position + n * self.radius
    }
    fn sample_uv_from_pos(&self, pos: Vec3) -> (f32, f32) {
        let n = (pos - self.position).normalized();
        let u = 0.5 + f32::atan2(n.x(), -n.z()) / TAU;
        let v = 0.5 + n.y().asin() / PI;
        (u * MAP_SIZE as f32, v * MAP_SIZE as f32)
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
    fn get_position(&self) -> Vec3 { self.position }

    fn get_intensity(&self) -> f32 { self.intensity }
}
