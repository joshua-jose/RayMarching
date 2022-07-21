#![feature(portable_simd)]
#![feature(core_intrinsics)]

#[macro_use]
mod colour;
mod engine;
mod material;
mod objects;
mod radiosity;
mod ray;
mod vector;

extern crate sdl2;

use colour::{SOFT_GRAY, SOFT_GREEN, SOFT_RED, SOFT_YELLOW, WHITE};
use engine::{Engine, HEIGHT, WIDTH};
use material::Material;
use objects::{EngineObject, PointLight, Sphere, XPlane, YPlane, ZPlane};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;
use vector::Vec3;

use crate::engine::Aligned;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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
    let mut canvas = window.into_canvas().build().unwrap(); // proxy for drawing

    // create a texture to plot pixels to
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            Some(sdl2::pixels::PixelFormatEnum::ARGB8888),
            WIDTH as u32,
            HEIGHT as u32,
        )
        .unwrap();

    // raw pixel buffer
    //let mut buffer = vec![0 as u8; 800 * 600 * 4];

    let objs = construct_objects();

    let mut engine = Engine {
        objects:         objs,
        camera_position: Vec3::new(0.0, 0.5, -3.5),
        light:           PointLight {
            position:  Vec3::new(0.0, 3.0, -2.5),
            intensity: 14.0,
        },
    };

    engine.compute_radiosity();
    let mut directions = Aligned(vec![vec![Vec3::default(); WIDTH]; HEIGHT]);

    'running: loop {
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
        let now = Instant::now();
        texture
            .with_lock(None, |buffer, _width| {
                engine.render(buffer, &mut directions);
            })
            .unwrap(); // update texture

        canvas.copy(&texture, None, None).unwrap();
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
        canvas.present();
    }
}

fn construct_objects() -> Vec<Box<dyn EngineObject>> {
    const BASIC_MAT: Material = Material::basic();

    vec![
        Box::new(Sphere {
            position: Vec3::new(-1.0, -1.0, 0.0),
            radius:   1.0,
            material: Material {
                ambient:      0.05,
                diffuse:      0.03,
                specular:     0.2,
                shininess:    16.0,
                reflectivity: 1.0,
            },
            colour:   WHITE,
        }),
        Box::new(Sphere {
            position: Vec3::new(1.0, -1.0, -0.8),
            radius:   1.0,
            material: Material {
                ambient:      0.1,
                diffuse:      1.0,
                specular:     0.9,
                shininess:    32.0,
                reflectivity: 0.3,
            },
            colour:   SOFT_YELLOW,
        }),
        Box::new(YPlane::new(-2.0, 1.0, BASIC_MAT, SOFT_GRAY)),
        Box::new(YPlane::new(4.0, -1.0, BASIC_MAT, SOFT_GRAY)),
        Box::new(XPlane::new(-3.0, 1.0, BASIC_MAT, SOFT_RED)),
        Box::new(XPlane::new(3.0, -1.0, BASIC_MAT, SOFT_GREEN)),
        Box::new(ZPlane::new(2.0, -1.0, BASIC_MAT, SOFT_GRAY)),
        Box::new(ZPlane::new(-4.0, 1.0, BASIC_MAT, SOFT_GRAY)),
    ]
}
