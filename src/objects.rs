use super::vector::Vec3;

pub trait HasDistanceFunction {
    fn sdf(&self, position: Vec3) -> f32;
}

impl HasDistanceFunction for Sphere {
    #[inline(always)]
    fn sdf(&self, position: Vec3) -> f32 {
        (position - self.position).mag() - self.radius
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

impl HasDistanceFunction for YPlane {
    #[inline(never)]
    fn sdf(&self, position: Vec3) -> f32 {
        position.y - self.y
    }
}
