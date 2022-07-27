use super::material::Material;
use super::vector::Vec3;

pub type Colour = Vec3;

#[macro_export]
macro_rules! rgb {
    [$r:expr, $g:expr, $b:expr] => {
        Colour::new(
             ($r as f32 / 255.0) * ($r as f32 / 255.0),
             ($g as f32 / 255.0) * ($g as f32 / 255.0),
             ($b as f32 / 255.0) * ($b as f32 / 255.0),
        )
    };
}

pub const WHITE: Colour = rgb![255, 255, 255];
pub const SOFT_RED: Colour = rgb![214, 81, 81];
pub const SOFT_GREEN: Colour = rgb![81, 214, 81];
//pub const SOFT_BLUE: Colour = rgb![81, 81, 214];
pub const SOFT_GRAY: Colour = rgb![214, 214, 214];
pub const SOFT_YELLOW: Colour = rgb![230, 230, 127];

#[repr(C)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[allow(non_snake_case)]
pub fn ACESFilm(mut col: Colour) -> Colour {
    let a: f32 = 2.51;
    let b: f32 = 0.03;
    let c: f32 = 2.43;
    let d: f32 = 0.59;
    let e: f32 = 0.14;
    let map_colours = |x: f32| ((x * (a * x + b)) / (x * (c * x + d) + e));
    col.0[0] = map_colours(col.x());
    col.0[1] = map_colours(col.y());
    col.0[2] = map_colours(col.z());
    col
}

// interpolates (x,y) between the 4 points. The 4 points should form a rectangle
pub fn bilinear_interpolation(x: f32, y: f32, points: &mut [(f32, f32, Colour); 4]) -> Colour {
    /*sort by y values, then x, to get 00,01,10,11 order */
    points.sort_by(|a, b| (&a.1).partial_cmp(&b.1).unwrap());
    points.sort_by(|a, b| (&a.0).partial_cmp(&b.0).unwrap());

    let (x1, y1, q11) = points[0];
    let (_x1, y2, q12) = points[1];
    let (x2, _y1, q21) = points[2];
    let (_x2, _y2, q22) = points[3];

    assert_eq!(x1, _x1, "Points do not form a rectangle");
    assert_eq!(x2, _x2, "Points do not form a rectangle");
    assert_eq!(y1, _y1, "Points do not form a rectangle");
    assert_eq!(y2, _y2, "Points do not form a rectangle");

    /*
    assert!(x1 <= x, "Point not within rectangle");
    assert!(x2 >= x, "Point not within rectangle");
    assert!(y1 <= y, "Point not within rectangle");
    assert!(y2 >= y, "Point not within rectangle");
    */

    (q11 * (x2 - x) * (y2 - y) + q21 * (x - x1) * (y2 - y) + q12 * (x2 - x) * (y - y1) + q22 * (x - x1) * (y - y1))
        / ((x2 - x1) * (y2 - y1))
}

// Phong diffuse and specular shading
pub fn phong_ds(
    n: Colour, vector_to_light: Colour, distance_to_light: f32, light_intensity: f32, object_mat: &Material,
    view_direction: Colour,
) -> (f32, f32) {
    let diffuse: f32;
    let specular: f32;

    let light_reflection_vector = vector_to_light.reflect(n);
    let light_intensity = light_intensity / (distance_to_light).powi(2); // k/d^2

    // Phong shading algorithm
    diffuse = object_mat.diffuse * light_intensity * vector_to_light.dot(n).max(0.0);
    if diffuse > 0.0 {
        specular = object_mat.specular
            * light_intensity
            * light_reflection_vector
                .dot(view_direction)
                .max(0.0)
                .powf(object_mat.shininess);
    } else {
        specular = 0.0;
    }

    (diffuse, specular)
}
