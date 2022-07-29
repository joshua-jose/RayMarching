#![feature(portable_simd)]
#![feature(core_intrinsics)]

#[macro_use]
mod colour;
mod engine;
mod material;
mod objects;
mod radiosity;
mod ray;
mod texture;
mod vector;

extern crate sdl2;

use colour::{SOFT_GRAY, SOFT_GREEN, SOFT_RED, SOFT_YELLOW, WHITE};
use engine::{Engine, HEIGHT, WIDTH};
use material::Material;
use objects::{EngineObject, PointLight, Sphere, XPlane, YPlane, ZPlane};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::MouseButton;
use std::time::Instant;
use vector::Vec3;

use crate::engine::Aligned;
use crate::texture::Texture;

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

    sdl_context.mouse().capture(true);
    sdl_context.mouse().set_relative_mouse_mode(true);
    sdl_context.mouse().show_cursor(false);

    // create a texture to plot pixels to
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            Some(sdl2::pixels::PixelFormatEnum::ARGB8888),
            WIDTH as u32,
            HEIGHT as u32,
        )
        .unwrap();

    let objs = construct_objects();

    let mut engine = Engine {
        objects:         objs,
        camera_position: Vec3::new(0.0, 0.5, -3.5),
        light:           PointLight {
            position:  Vec3::new(2.0, -1.0, 1.5),
            intensity: 3.5,
        },
    };

    engine.compute_lightmaps();
    let mut directions = Aligned(vec![vec![Vec3::default(); WIDTH]; HEIGHT]);

    rayon::ThreadPoolBuilder::new().num_threads(10).build_global().unwrap();

    let mut mouse_x: i32 = 0;
    let mut mouse_y: i32 = 0;
    let mut scroll: i32 = 0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,

                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    sdl_context.mouse().capture(false);
                    sdl_context.mouse().set_relative_mouse_mode(false);
                    sdl_context.mouse().show_cursor(true);
                }

                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    sdl_context.mouse().capture(true);
                    sdl_context.mouse().set_relative_mouse_mode(true);
                    sdl_context.mouse().show_cursor(false);
                }

                Event::MouseMotion { xrel, yrel, .. } => {
                    //if relative mouse mode true, mouse is captured
                    if sdl_context.mouse().relative_mouse_mode() {
                        mouse_x += xrel;
                        mouse_y += yrel;
                    }
                }
                Event::MouseWheel { y, .. } => {
                    if sdl_context.mouse().relative_mouse_mode() {
                        scroll += y;
                    }
                }

                _ => {}
            }
        }

        let mut rel_move = Vec3::default();
        let speed;
        if event_pump.keyboard_state().is_scancode_pressed(Scancode::LShift) {
            speed = 0.15;
        } else {
            speed = 0.05;
        }

        for pressed_key in event_pump.keyboard_state().pressed_scancodes() {
            match pressed_key {
                Scancode::W => rel_move.0[2] += speed,
                Scancode::S => rel_move.0[2] -= speed,
                Scancode::A => rel_move.0[0] -= speed,
                Scancode::D => rel_move.0[0] += speed,
                _ => {}
            }
        }

        let now = Instant::now();
        texture
            .with_lock(None, |buffer, _width| {
                engine.render(buffer, &mut directions, mouse_x, mouse_y, rel_move, scroll);
            })
            .unwrap(); // update texture

        canvas.copy(&texture, None, None).unwrap();
        let elapsed = now.elapsed();
        println!("Elapsed: {:.2?}", elapsed);
        canvas.present();
    }
}

fn construct_objects() -> Vec<Box<dyn EngineObject>> {
    let wood_tex = Texture::new("assets/textures/Floor128.bmp", 32.0, 32.0);
    const BASIC_MAT: Material = Material::basic();
    vec![
        Box::new(Sphere {
            position: Vec3::new(-1.2, -1.0, 0.1),
            radius:   1.0,
            material: Material {
                ambient:      0.05,
                diffuse:      0.03,
                specular:     0.2,
                shininess:    16.0,
                reflectivity: 1.0,
                emissive:     0.0,
            },
            colour:   WHITE,
            lightmap: Default::default(),
        }),
        Box::new(Sphere {
            position: Vec3::new(1.0, -1.0, -0.7),
            radius:   1.0,
            material: Material {
                ambient:      0.1,
                diffuse:      1.0,
                specular:     0.9,
                shininess:    32.0,
                reflectivity: 0.25,
                emissive:     0.0,
            },
            colour:   SOFT_YELLOW,
            lightmap: Default::default(),
        }),
        Box::new(YPlane::new(-2.0, 1.0, BASIC_MAT, SOFT_GRAY, &wood_tex)),
        Box::new(YPlane::new(4.0, -1.0, BASIC_MAT, SOFT_GRAY, &wood_tex)),
        Box::new(XPlane::new(-3.0, 1.0, BASIC_MAT, SOFT_RED)),
        Box::new(XPlane::new(3.0, -1.0, BASIC_MAT, SOFT_GREEN)),
        Box::new(ZPlane::new(2.0, -1.0, BASIC_MAT, SOFT_GRAY)),
        Box::new(ZPlane::new(-4.0, 1.0, BASIC_MAT, SOFT_GRAY)),
    ]
}
