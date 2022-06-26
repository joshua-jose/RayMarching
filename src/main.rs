pub mod vector;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use vector::Vec3;

const WIDTH: usize = 600;
const HEIGHT: usize = 600;

struct Sphere {
    pub position: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn sdf(&self, position: Vec3) -> f32 {

        (position - self.position).mag() - self.radius
    }
}

static THING: Sphere = Sphere {
    position: Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    radius: 1.0,
};

struct Ray {
    pub position: Vec3,
    pub direction: Vec3,
}

impl Ray {
    fn march(&mut self) -> bool {
        let mut distance_travelled = 0.0;

        while distance_travelled < 100.0 {
            let distance = THING.sdf(self.position);
            distance_travelled += distance;
            self.position = self.position + (self.direction * distance);

            if distance < 0.001 {
                return true;
            }
        }

        return false;
    }
}

const STEP_SIZE: f32 = 0.001;
const X_STEP: Vec3 = Vec3 {
    x: STEP_SIZE,
    y: 0.0,
    z: 0.0,
};
const Y_STEP: Vec3 = Vec3 {
    x: 0.0,
    y: STEP_SIZE,
    z: 0.0,
};
const Z_STEP: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: STEP_SIZE,
};

fn calculate_normal(position: Vec3) -> Vec3 {
    let gradient_x = THING.sdf(position + X_STEP) - THING.sdf(position - X_STEP);
    let gradient_y = THING.sdf(position + Y_STEP) - THING.sdf(position - Y_STEP);
    let gradient_z = THING.sdf(position + Z_STEP) - THING.sdf(position - Z_STEP);

    Vec3 {
        x: gradient_x,
        y: gradient_y,
        z: gradient_z,
    }
    .normalized()
}

static mut N: i32 = 0;

const LIGHT_POS: Vec3 = Vec3 {
    x: 2.0,
    y: -5.0,
    z: 3.0,
};

fn render(buffer: &mut [u8], _width: usize) {
    let sin_n: f32;
    let cos_n: f32;
    unsafe {
        sin_n = (0.1 * N as f32).sin();
        cos_n = (0.1 * N as f32).cos();
        N += 1;
    }
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let u = (2.0 * x as f32 / WIDTH as f32) - 1.0;
            let v = (2.0 * y as f32 / HEIGHT as f32) - 1.0;
            let i = y * WIDTH + x;

            let mut ray = Ray {
                position: Vec3 {
                    x: sin_n,
                    y: cos_n,
                    z: -5.0,
                },
                direction: Vec3 { x: u, y: v, z: 1.0 },
            };

            let hit = ray.march();

            let mut r = 0;
            let mut g = 0;
            let mut b = 0;

            if hit {
                let n = calculate_normal(ray.position);
                let diffuse = (ray.position - LIGHT_POS).normalized().dot(n).max(0.05);

                r = (255.0 * diffuse).floor() as u8;
                g = (255.0 * diffuse).floor() as u8;
                b = (255.0 * diffuse).floor() as u8;
            }

            buffer[i * 4 + 0] = b;
            buffer[i * 4 + 1] = g;
            buffer[i * 4 + 2] = r;
            //buffer[i * 4 + 3] = 0;
        }
    }
}

fn main() {
    println!("Hello, world!");

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // create window
    let window = video_subsystem
        .window("ray marching", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap(); // lets us handle events
    let mut canvas = window.into_canvas().accelerated().build().unwrap(); // proxy for drawing

    // create a texture to plot pixels to
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(None, WIDTH as u32, HEIGHT as u32)
        .unwrap();

    // raw pixel buffer
    //let mut buffer = vec![0 as u8; 800 * 600 * 4];

    'running: loop {
        use std::time::Instant;
        let now = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        texture.with_lock(None, render).unwrap(); // update texture
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
    }
}
