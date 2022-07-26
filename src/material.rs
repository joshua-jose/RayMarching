#[derive(Clone, Copy)]
pub struct Material {
    pub ambient:      f32,
    pub diffuse:      f32, // aka albedo
    pub specular:     f32,
    pub shininess:    f32, // aka gloss
    pub reflectivity: f32,
}

impl Material {
    pub const fn basic() -> Self {
        Self {
            ambient:      0.25,
            diffuse:      1.0,
            specular:     0.0,
            shininess:    4.0,
            reflectivity: 0.0,
        }
    }
}

impl Default for Material {
    fn default() -> Self { Material::basic() }
}
