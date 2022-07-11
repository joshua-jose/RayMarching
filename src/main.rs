#[macro_use]
mod colour;
mod engine;
mod objects;
mod vector;

extern crate sdl2;

use engine::{Engine, HEIGHT, WIDTH};
use objects::{PointLight, Sphere, XPlane, YPlane, ZPlane};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;
use vector::Vec3;

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
        .create_texture_streaming(
            Some(sdl2::pixels::PixelFormatEnum::ARGB8888),
            WIDTH as u32,
            HEIGHT as u32,
        )
        .unwrap();

    // raw pixel buffer
    //let mut buffer = vec![0 as u8; 800 * 600 * 4];

    let mut engine = Engine {
        objects: vec![
            &Sphere {
                position: Vec3 {
                    x: 0.0,
                    y: -0.85,
                    z: 0.0,
                },
                radius: 1.0,
            },
            &YPlane { y: -2.0, dir: 1.0 },
            &YPlane { y: 4.0, dir: -1.0 },
            &XPlane { x: -3.0, dir: 1.0 },
            &XPlane { x: 3.0, dir: -1.0 },
            &ZPlane { z: 2.0, dir: -1.0 },
            &ZPlane { z: -4.5, dir: 1.0 },
        ],
        camera_position: Vec3 {
            x: 0.0,
            y: 0.0,
            z: -4.0,
        },
        light: PointLight {
            position: Vec3 {
                x: 0.0,
                y: 3.0,
                z: 0.0,
            },
            intensity: 12.0,
        },
    };

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
            .with_lock(None, |buffer, width| {
                engine.render(buffer, width);
            })
            .unwrap(); // update texture

        canvas.copy(&texture, None, None).unwrap();
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
        canvas.present();
    }
}
