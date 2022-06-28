use super::vector::Vec3;

pub trait EngineObject {
    fn sdf(&self, position: Vec3) -> f32;
    fn colour(&self, position: Vec3) -> [u8; 3];
    fn reflectivity(&self) -> u8;
}

impl EngineObject for Sphere {
    #[inline(always)]
    fn sdf(&self, position: Vec3) -> f32 {
        (position - self.position).mag() - self.radius
    }

    fn colour(&self, _position: Vec3) -> [u8; 3] {
        [245, 104, 44]
    }

    fn reflectivity(&self) -> u8 {
        255
    }
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

impl EngineObject for YPlane {
    #[inline(never)]
    fn sdf(&self, position: Vec3) -> f32 {
        position.y - self.y
    }

    fn colour(&self, position: Vec3) -> [u8; 3] {
        if (position.x.round() as i32) % 2 == 0 || (position.z.round() as i32) % 2 == 0 {
            [102, 245, 66]
        } else {
            [2, 105, 240]
        }
    }

    fn reflectivity(&self) -> u8 {
        0
    }
}
