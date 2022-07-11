use super::vector::Vec3;
use Vec3 as Colour;

pub trait EngineObject {
    fn sdf(&self, position: Vec3) -> f32;
    fn colour(&self, position: Vec3) -> Colour;

    fn diffuse(&self) -> f32 {
        1.0
    }
    fn specular(&self) -> f32 {
        0.0
    }
    fn reflectivity(&self) -> f32 {
        0.0
    }

    fn ambient(&self) -> f32 {
        0.03
    }

    fn shininess(&self) -> f32 {
        4.0
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
}

#[derive(Clone, Copy)]
pub struct YPlane {
    pub y: f32,
    pub dir: f32,
}

#[derive(Clone, Copy)]
pub struct XPlane {
    pub x: f32,
    pub dir: f32,
}

#[derive(Clone, Copy)]
pub struct ZPlane {
    pub z: f32,
    pub dir: f32,
}

#[derive(Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub intensity: f32,
}

const WHITE: Colour = rgb![255, 255, 255];
const SOFT_RED: Colour = rgb![214, 81, 81];
const SOFT_GREEN: Colour = rgb![81, 214, 81];
const SOFT_GRAY: Colour = rgb![214, 214, 214];

impl EngineObject for YPlane {
    fn sdf(&self, position: Vec3) -> f32 {
        self.dir * (position.y - self.y)
    }

    fn colour(&self, _position: Vec3) -> Colour {
        rgb![179, 179, 179]
    }
}

impl EngineObject for XPlane {
    fn sdf(&self, position: Vec3) -> f32 {
        self.dir * (position.x - self.x)
    }

    fn colour(&self, position: Vec3) -> Colour {
        if position.x < 0.0 {
            SOFT_RED
        } else {
            SOFT_GREEN
        }
    }
}

impl EngineObject for ZPlane {
    fn sdf(&self, position: Vec3) -> f32 {
        self.dir * (position.z - self.z)
    }

    fn colour(&self, _position: Vec3) -> Colour {
        SOFT_GRAY
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

impl EngineObject for Sphere {
    fn sdf(&self, position: Vec3) -> f32 {
        (position - self.position).mag() - self.radius
    }

    fn colour(&self, _position: Vec3) -> Colour {
        WHITE
    }

    fn reflectivity(&self) -> f32 {
        1.0
    }

    fn diffuse(&self) -> f32 {
        0.03
    }

    fn shininess(&self) -> f32 {
        16.0
    }

    fn specular(&self) -> f32 {
        0.2
    }
}

impl EngineLight for PointLight {
    fn get_position(&self) -> Vec3 {
        self.position
    }

    fn get_intensity(&self) -> f32 {
        self.intensity
    }
}
