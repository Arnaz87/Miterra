#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate cgmath;

mod camera;
mod chunk;

use camera::Camera;
use chunk::Chunk;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx_window_glutin as gfx_glutin;
use cgmath::{Matrix4, Deg, Vector3, Point3, InnerSpace};

// Esto debería ser Srgba8, todo el mundo usa eso, pero glutin da un error.
pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        normal: [f32; 3] = "a_Normal",
        color: [f32; 3] = "a_Color",
    }

    constant Transform {
        transform: [[f32; 4]; 4] = "u_Transform",
    }

    constant Light {
        dir: [f32; 3] = "u_LightDir",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        light: gfx::ConstantBuffer<Light> = "Light",
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        out_color: gfx::RenderTarget<ColorFormat> = "FragColor",
        out_depth: gfx::DepthTarget<::gfx::format::DepthStencil> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

impl Transform {
    pub fn new (matrix: Matrix4<f32>) -> Self {
        Transform { transform: *matrix.as_ref() }
    }
}

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

const U: f32 = 0.57735; // 1/sqrt(3)

const VERTICES: &[Vertex] = &[
    Vertex { pos: [-0.5, -0.5, -0.5], color: [0.1, 0.1, 0.1], normal: [-U,-U,-U] },
    Vertex { pos: [-0.5, -0.5,  0.5], color: [0.1, 0.1, 0.8], normal: [-U,-U, U] },
    Vertex { pos: [-0.5,  0.5, -0.5], color: [0.1, 0.8, 0.1], normal: [-U, U,-U] },
    Vertex { pos: [-0.5,  0.5,  0.5], color: [0.0, 0.8, 0.7], normal: [-U, U, U] },
    Vertex { pos: [ 0.5, -0.5, -0.5], color: [0.8, 0.1, 0.1], normal: [ U,-U,-U] },
    Vertex { pos: [ 0.5, -0.5,  0.5], color: [0.8, 0.1, 0.8], normal: [ U,-U, U] },
    Vertex { pos: [ 0.5,  0.5, -0.5], color: [0.7, 0.8, 0.0], normal: [ U, U,-U] },
    Vertex { pos: [ 0.5,  0.5,  0.5], color: [0.8, 0.6, 0.8], normal: [ U, U, U] },
];

const INDICES: &[u16] = &[
    0, 1, 2,  1, 2, 3,
    4, 5, 6,  5, 6, 7,
    0, 1, 4,  1, 4, 5,
    2, 3, 6,  3, 6, 7,
    0, 2, 4,  2, 4, 6,
    1, 3, 5,  3, 5, 7,
];

pub fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new()
        .with_title("3D toy".to_string())
        .with_dimensions(500, 500);
    let context = glutin::ContextBuilder::new()
        .with_depth_buffer(16)
        .with_vsync(true);
    let (window, mut device, mut factory, rtv, stv) =
        gfx_glutin::init::<ColorFormat, DepthFormat>(builder, context, &events_loop);

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let pso = {
        let vs = include_bytes!("../assets/shader_150_v.glsl");
        let ps = include_bytes!("../assets/shader_150_f.glsl");
        let init = pipe::new();

        // PointList, TriangleList
        let prim = ::gfx::Primitive::TriangleList;
        let raster = ::gfx::state::Rasterizer::new_fill();

        let set = factory.create_shader_set(vs, ps).unwrap();
        factory.create_pipeline_state(&set, prim, raster, init).unwrap()
    };

    /*let pso = factory.create_pipeline_simple(
        include_bytes!("../assets/shader_150.glslv"),
        include_bytes!("../assets/shader_150.glslf"),
        pipe::new()
    ).unwrap();*/

    let mut chunk = Chunk::new();
    chunk.create_mesh();

    let iterator = chunk.vertices.iter().zip(chunk.normals.iter());
    let vertices: Vec<Vertex> = iterator.map( |(p, n)|
        Vertex {
            pos: *p.as_ref(),
            color: *(p/64.0).as_ref(),
            normal: *n.as_ref()
        }
    ).collect();
    let indices: &[u16] = chunk.indices.as_ref();


    /*for v in vertices.iter() {
        println!("{:?} - {:?}", v.pos, v.normal);
    }
    println!("{:?}", indices);
    println!("{} faces, {} vertices", indices.len()/3, vertices.len());*/

    let (vertex_buffer, slice) =
        factory.create_vertex_buffer_with_slice(&vertices, indices);
    let transform_buffer = factory.create_constant_buffer(1);
    let light_buffer = factory.create_constant_buffer(1);

    let mut rot = Matrix4::from_angle_x(Deg(0.0));
    let pos = Matrix4::from_translation(Vector3{x: 0.0, y: 0.0, z: 0.0});

    // Rango aceptable de FOV: 45° - 120°
    // Mejor FOV: 100°
    let mut cam = Camera::new(45.0, 0.01, 500.0);
    cam.pos.x = 32.0;
    cam.pos.y = 32.0;
    cam.pos.z = 64.0 + 32.0;
    cam.sensitivity = 4.0;

    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        transform: transform_buffer,
        light: light_buffer,
        out_color: rtv,
        out_depth: stv,
    };

    {
        let dir = Vector3::new(-1.0, 1.0, 1.0).normalize();
        let light = Light{ dir: *dir.as_ref() };
        encoder.update_buffer(&data.light, &[light], 0).unwrap();
    }

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

    while running {
        use glutin::GlContext;

        /*if needs_update {
            let (vertices, indices) = cube.get_vertices_indices();
            let (vbuf, sl) = factory.create_vertex_buffer_with_slice(
                &vertices, &*indices
            );

            data.vbuf = vbuf;
            slice = sl;

            needs_update = false
        }*/

        events_loop.poll_events(|_ev| { match _ev {
            glutin::Event::WindowEvent{event, ..} => {
                use glutin::WindowEvent::*;
                use glutin::{MouseButton, ElementState, CursorState};

                match event {
                    Closed => running = false,
                    /*KeyboardInput(_, _, Some(VirtualKeyCode::Escape), _) => {
                        active = false;
                        window.set_cursor_state(CursorState::Normal).unwrap();
                    },*/
                    Resized(w, h) => {
                        gfx_glutin::update_views(&window, &mut data.out_color, &mut data.out_depth);
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
                        } };
                    }
                    _ => ()
                }
            }, _ => ()
        } });

        cam.update();

        //rot = Matrix4::from_angle_y(Deg(1.0)) * rot;

        let t = Transform::new(cam.matrix());

        encoder.update_buffer(&data.transform, &[t], 0).unwrap();
        encoder.clear(&data.out_color, BLACK);
        encoder.clear_depth(&data.out_depth, 1.0);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}