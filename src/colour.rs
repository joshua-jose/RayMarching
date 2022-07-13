use super::Vec3;

#[macro_export]
macro_rules! rgb {
    [$r:expr, $g:expr, $b:expr] => {
        Vec3::new(
             ($r as f32 / 255.0) * ($r as f32 / 255.0),
             ($g as f32 / 255.0) * ($g as f32 / 255.0),
             ($b as f32 / 255.0) * ($b as f32 / 255.0),
        )
    };
}

pub const WHITE: Vec3 = rgb![255, 255, 255];
pub const SOFT_RED: Vec3 = rgb![214, 81, 81];
pub const SOFT_GREEN: Vec3 = rgb![81, 214, 81];
pub const SOFT_GRAY: Vec3 = rgb![214, 214, 214];
pub const SOFT_YELLOW: Vec3 = rgb![230, 230, 127];

#[repr(C)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[allow(non_snake_case)]
pub fn ACESFilm(mut col: Vec3) -> Vec3 {
    let a: f32 = 2.51;
    let b: f32 = 0.03;
    let c: f32 = 2.43;
    let d: f32 = 0.59;
    let e: f32 = 0.14;
    let map_colours = |x: f32| ((x * (a * x + b)) / (x * (c * x + d) + e));
    col.x = map_colours(col.x);
    col.y = map_colours(col.y);
    col.z = map_colours(col.z);
    col
}

