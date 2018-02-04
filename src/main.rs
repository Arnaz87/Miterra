#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate cgmath;

mod camera;
mod voxel_source;
mod mesher;
mod surfnet;
mod blocky;
mod marching_cubes;
mod mesh;
mod chunk;
mod base;

use base::Base;
use camera::Camera;

use voxel_source::SphereSource;
use surfnet::SurfNet;
use blocky::Blocky;
use mesher::Mesher;

use marching_cubes::MarchingCubes;
use chunk::ChunkManager;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx_window_glutin as gfx_glutin;
use cgmath::{Matrix4, Vector3, InnerSpace};

pub struct World {
    camera: Camera,
    sun_angle: Vector3<f32>,
}

pub fn main() {

    let mut base = Base::new("MiTerra", 500, 500);

    let mut chunks = ChunkManager::new(
        SphereSource{x: 32, y: -64, z: 32, r: 96},
        //Blocky{size: 64},
        MarchingCubes{size: 64, smooth: true},
        //SurfNet{size: 64, smooth: 6},
    );

    chunks.generate(0, 0, 0);
    chunks.generate(0, 0, 64);
    chunks.generate(64, 0, 0);
    chunks.generate(64, 0, 64);

    // Rango aceptable de FOV: 45° - 120°
    // Mejor FOV: 100°
    let mut cam = Camera::new(45.0, 0.01, 500.0);
    cam.pos.x = 32.0;
    cam.pos.y = 32.0;
    cam.pos.z = 64.0 + 32.0;
    cam.sensitivity = 4.0;

    let mut running = true;
    let mut needs_update = false;

    let mut center = (0, 0);
    let mut mouse_pos = (0, 0);
    let mut active = false;

    fn try_center_mouse (
            w: &::glutin::Window,
            p: &mut (i32, i32),
            center: (i32, i32)
        ) {
        match w.set_cursor_position(center.0, center.1) {
            Ok(_) => *p = (center.0, center.1),
            Err(_) => println!("Could not set mouse position.")
        };
    }

    println!("In the game screen:");
    println!("- Press 1 to mine the craft.");
    println!("- Press 2 to net the surface.");
    println!("- Press 3 to march the cubes.");

    while running {
        match base {
            Base { ref mut event_loop, ref mut window, ref mut out_color, ref mut out_depth, .. } => {
                event_loop.poll_events(|_ev| { match _ev {
                    glutin::Event::WindowEvent{event, ..} => {
                        use glutin::WindowEvent::*;
                        use glutin::{MouseButton, ElementState, CursorState};

                        match event {
                            Closed => running = false,
                            Resized(w, h) => {
                                gfx_glutin::update_views(&window, out_color, out_depth);
                                center = (w as i32/2, h as i32/2);
                                cam.set_screen_size(w as f32, h as f32);
                                needs_update = true
                            },
                            CursorMoved{position: (x, y), ..} => {
                                if active {
                                    let xdif = x as i32- mouse_pos.0;
                                    let ydif = y as i32 - mouse_pos.1;
                                    cam.move_pixels(xdif as f32, ydif as f32);
                                    try_center_mouse(&window, &mut mouse_pos, center);
                                } else {
                                    mouse_pos = (x as i32, y as i32);
                                }
                            },
                            MouseInput{
                                    state: ElementState::Pressed,
                                    button: MouseButton::Left,
                                    .. } => {
                                active = true;
                                match window.set_cursor_state(CursorState::Hide) {
                                    Ok(_) => (),
                                    Err(msg) => println!("{:?}", msg)
                                };
                                try_center_mouse(&window, &mut mouse_pos, center);
                            },
                            KeyboardInput{
                                input: ::glutin::KeyboardInput {
                                    state, virtual_keycode: Some(key), ..
                                }, .. } => {

                                use glutin::{VirtualKeyCode as Key};

                                let pressed = match state {
                                    ElementState::Pressed => true,
                                    ElementState::Released => false,
                                };

                                if active { match key {
                                    Key::Escape if pressed => {
                                        active = false;
                                        match window.set_cursor_state(CursorState::Normal) {
                                            Ok(_) => (),
                                            Err(msg) => println!("{:?}", msg)
                                        };
                                    },
                                    Key::Space => cam.up = pressed,
                                    Key::LShift => cam.down = pressed,
                                    Key::W => cam.front = pressed,
                                    Key::A => cam.left = pressed,
                                    Key::S => cam.back = pressed,
                                    Key::D => cam.right = pressed,
                                    _ => ()
                                } }

                                if active && pressed { match key {
                                    Key::Key1 => chunks.set_mesher(Blocky{size: 64}),
                                    Key::Key2 => chunks.set_mesher(SurfNet{size: 64, smooth: 7}), // smooth 7 is best
                                    Key::Key3 => chunks.set_mesher(MarchingCubes{size: 64, smooth: true}),
                                    _ => {}
                                } }
                            }
                            _ => ()
                        }
                    }, _ => ()
                } });
            }
        }
        
        chunks.update(&mut base);

        cam.update();
        base.update_world(base::World {
            view: *cam.matrix().as_ref(),
            light_dir: *Vector3::new(-0.6, 1.0, 0.8).normalize().as_ref(),
        });

        base.begin();
        chunks.render(&mut base);
        base.end();
    }
}