use super::vector::Vec3;

pub trait EngineObject {
    fn sdf(&self, position: Vec3) -> f32;
    fn colour(&self, position: Vec3) -> [f32; 3];

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
}

#[derive(Clone, Copy)]
pub struct XPlane {
    pub x: f32,
}

#[derive(Clone, Copy)]
pub struct ZPlane {
    pub z: f32,
}

#[derive(Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub intensity: f32,
}

impl EngineObject for YPlane {
    #[inline(never)]
    fn sdf(&self, position: Vec3) -> f32 {
        position.y - self.y
    }

    fn colour(&self, position: Vec3) -> [f32; 3] {
        if (position.x.round() as i32) % 2 == 0 || (position.z.round() as i32) % 2 == 0 {
            rgb![168, 250, 138]
        } else {
            rgb![28, 170, 248]
        }
    }
}

impl EngineObject for Sphere {
    #[inline(always)]
    fn sdf(&self, position: Vec3) -> f32 {
        (position - self.position).mag() - self.radius
    }

    fn colour(&self, _position: Vec3) -> [f32; 3] {
        rgb![255, 255, 255]
    }

    fn reflectivity(&self) -> f32 {
        0.9
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
