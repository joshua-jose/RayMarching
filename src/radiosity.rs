use super::vector::Vec3;

use Vec3 as Colour;

pub const MAP_SIZE:usize = 8;

#[derive(Copy, Clone, Debug, Default)]
pub struct Lightmap {
    pub sample_map: [[Colour;MAP_SIZE];MAP_SIZE] // 2D array of colour, dim MAP_SIZE*MAP_SIZE
}
