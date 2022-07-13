#[macro_use]
mod colour;
mod engine;
mod material;
mod objects;
mod vector;

extern crate sdl2;

use engine::{Engine, HEIGHT, WIDTH};
use material::Material;
use objects::{EngineObject, PointLight,Sphere, XPlane, YPlane, ZPlane};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;
use vector::Vec3;
use colour::{WHITE,SOFT_YELLOW,SOFT_GRAY, SOFT_RED, SOFT_GREEN};

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

    let engine = Engine {
        objects: construct_objects(),
        camera_position: Vec3 { x: 0.0, y: 0.5, z: -4.0 },
        light: PointLight {position: Vec3 { x: 0.0, y: 3.0, z: -3.0 }, intensity: 12.0,},
    };

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                }
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

fn construct_objects() -> Vec<&'static dyn EngineObject> {
    const BASIC_MAT: Material = Material::basic();
    
    vec![
        &Sphere {
            position: Vec3 { x: -1.0, y: -0.85, z: 0.5 },
            radius:   1.0,
            material: Material {
                ambient:      0.05,
                diffuse:      0.03,
                specular:     0.2,
                shininess:    16.0,
                reflectivity: 1.0,
            },
            colour: WHITE
        },
        &Sphere {
            position: Vec3 { x: 1.0, y: -0.85, z: -0.8 },
            radius:   1.0,
            material: Material {
                ambient:      0.1,
                diffuse:      1.0,
                specular:     0.9,
                shininess:    32.0,
                reflectivity: 0.3,
            },
            colour: SOFT_YELLOW
        },
        &YPlane { y: -2.0, dir: 1.0, material: BASIC_MAT, colour: SOFT_GRAY },
        &YPlane { y: 4.0, dir: -1.0, material: BASIC_MAT, colour: SOFT_GRAY },
        &XPlane { x: -3.0, dir: 1.0, material: BASIC_MAT, colour: SOFT_RED },
        &XPlane { x: 3.0, dir: -1.0, material: BASIC_MAT, colour: SOFT_GREEN },
        &ZPlane { z: 2.0, dir: -1.0, material: BASIC_MAT, colour: SOFT_GRAY },
        &ZPlane { z: -4.5, dir: 1.0, material: BASIC_MAT, colour: SOFT_GRAY },
    ]
}
