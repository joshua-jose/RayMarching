use std::fmt;
use std::intrinsics::sqrtf32;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};
use std::simd::{f32x4, StdFloat};

#[derive(Debug, Default, Clone, Copy)]
#[repr(transparent)]
pub struct Vec3(pub f32x4);

impl Vec3 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self { Self(f32x4::from_array([x, y, z, 0.0])) }

    #[inline(always)]
    pub fn x(&self) -> f32 { self.0[0] }
    #[inline(always)]
    pub fn y(&self) -> f32 { self.0[1] }
    #[inline(always)]
    pub fn z(&self) -> f32 { self.0[2] }

    /// Returns the mag of this [`Vec3`].
    pub fn mag(&self) -> f32 { unsafe { sqrtf32(self.mag_sqd()) } }

    pub fn mag_sqd(&self) -> f32 { (self.0 * self.0).reduce_sum() }

    /// Returns the normalized vector of this [`Vec3`].
    pub fn normalized(&self) -> Vec3 {
        let inv_mag = self.mag().recip();
        *self * inv_mag
    }

    pub fn dot(&self, other: Vec3) -> f32 { self.x() * other.x() + self.y() * other.y() + self.z() * other.z() }

    pub fn sqrt(&self) -> Vec3 { Vec3(self.0.sqrt()) }

    pub fn element_mul(&self, other: Vec3) -> Vec3 { Vec3(self.0 * other.0) }

    pub fn reflect(&self, normal: Vec3) -> Vec3 { *self - normal * (2.0 * normal.dot(*self)) }

    pub fn refract(&self, normal: Vec3, ior: f32) -> Vec3 {
        //! ior is actually the ratio of index of refractions of the two materials
        let k = 1.0 - ior.powi(2) * (1.0 - self.dot(normal).powi(2));
        if k < 0.0 {
            Vec3::new(0.0, 0.0, 0.0)
        } else {
            ior * (*self) - (ior * self.dot(normal) + k.sqrt()) * normal
        }
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vec3(x: {},y: {}, z: {})", self.x(), self.y(), self.z())
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output { Vec3(self.0 + rhs.0) }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) { self.0 += rhs.0; }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output { Vec3(self.0 - rhs.0) }
}

impl Add<f32> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: f32) -> Self::Output { Vec3(self.0 + f32x4::splat(rhs)) }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output { Vec3(self.0 * f32x4::splat(rhs)) }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output { Vec3(self.0 / f32x4::splat(rhs)) }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output { Vec3(-self.0) }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output { rhs * self }
}
