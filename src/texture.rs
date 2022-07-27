use super::colour::Colour;

#[derive(Debug, Clone)]
pub struct Texture {
    image:      Vec<Colour>,
    width:      u32,
    height:     u32,
    pub uscale: f32,
    pub vscale: f32,
}

impl Texture {
    pub fn new(path: &str, uscale: f32, vscale: f32) -> Self {
        let raw_image = image::open(path).unwrap().into_rgb8();
        let (width, height) = (raw_image.width(), raw_image.height());
        let mut image = Vec::with_capacity((raw_image.width() * raw_image.height()) as usize);

        for pixel in raw_image.pixels() {
            let [r, g, b] = pixel.0;
            image.push(rgb!(r, g, b))
        }
        Self {
            image,
            width,
            height,
            uscale,
            vscale,
        }
    }

    pub fn sample(&self, u: f32, v: f32) -> Colour {
        let x = (u * self.uscale).floor() as u32 % self.width;
        let y = (v * self.vscale).floor() as u32 % self.height;
        let i = x + (y * self.width);

        self.image[i as usize]
    }
}
