
use gfx;
use glutin;

pub use gfx::traits::FactoryExt;
use gfx::{Device, handle};
use gfx_window_glutin as gfx_glutin;
use gfx_device_gl as backend;
use self::backend::Resources;
use glutin::GlContext;

// Esto debería ser Srgba8, pero glutin da un error.
// Con srgb, se aplica automáticamente correción gamma (tengo entendido).
// https://www.khronos.org/opengl/wiki/Image_Format#sRGB_colorspace
pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        normal: [f32; 3] = "a_Normal",
        material: i32 = "a_Material",
    }

    constant World {
        view: [[f32; 4]; 4] = "u_View",
        light_dir: [f32; 3] = "u_LightDir",
    }

    pipeline terrain_pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        world: gfx::ConstantBuffer<World> = "World",
        out_color: gfx::RenderTarget<ColorFormat> = "FragColor",
        grass: gfx::TextureSampler<[f32; 4]> = "t_Grass",
        soilsand: gfx::TextureSampler<[f32; 4]> = "t_SoilSand",
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

pub type Slice = gfx::Slice<Resources>;
pub type Texture = handle::ShaderResourceView<Resources, [f32; 4]>;

pub type VertexBuffer = handle::Buffer<Resources, Vertex>;
pub type WorldBuffer = handle::Buffer<Resources, World>;
pub type Sampler = handle::Sampler<Resources>;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

pub struct Base {
    pub device: backend::Device,
    pub encoder: gfx::Encoder<Resources, backend::CommandBuffer>,
    pub factory: backend::Factory,

    pub event_loop: glutin::EventsLoop,
    pub window: glutin::GlWindow,

    pub out_color: handle::RenderTargetView<Resources, ColorFormat>,
    pub out_depth: handle::DepthStencilView<Resources, DepthFormat>,

    pub terrain_pso: gfx::PipelineState<Resources, terrain_pipe::Meta>,

    pub world_buffer: WorldBuffer,

    pub sampler: handle::Sampler<Resources>,
}

impl Base {
    pub fn new (title: &str, width: u32, height: u32) -> Self {
        let mut event_loop = glutin::EventsLoop::new();
        let builder = glutin::WindowBuilder::new()
            .with_title(title.to_string())
            .with_dimensions(width, height);
        let context = glutin::ContextBuilder::new()
            .with_depth_buffer(16)
            .with_vsync(true);
        let (window, mut device, mut factory, rtv, stv) =
            gfx_glutin::init::<ColorFormat, DepthFormat>(builder, context, &event_loop);

        let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

        let pso = {
            let vs = include_bytes!("../assets/shader_150_v.glsl");
            let gs = include_bytes!("../assets/shader_150_g.glsl");
            let ps = include_bytes!("../assets/shader_150_f.glsl");
            let shader_set = factory.create_shader_set_geometry(vs, gs, ps).unwrap();

            let init = terrain_pipe::new();

            // PointList, TriangleList
            let prim = gfx::Primitive::TriangleList;
            let raster = gfx::state::Rasterizer::new_fill().with_cull_back();

            factory.create_pipeline_state(&shader_set, prim, raster, init).unwrap()
        };

        let w_buff = factory.create_constant_buffer(1);
        let sampler = factory.create_sampler_linear();

        Base {
            device: device,
            encoder: encoder,
            factory: factory,

            window: window,
            event_loop: event_loop,

            terrain_pso: pso,
            world_buffer: w_buff,

            out_color: rtv,
            out_depth: stv,

            sampler: sampler,
        }
    }

    pub fn begin (&mut self) {
        let &mut Base {ref mut encoder, ref mut out_color, ref mut out_depth, ..} = self;
        encoder.clear(out_color, BLACK);
        encoder.clear_depth(out_depth, 1.0);
    }

    pub fn end (&mut self) {
        let &mut Base {ref mut encoder, ref mut device, ref mut window, ..} = self;
        encoder.flush(device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }

    pub fn update_world (&mut self, w: World) {
        let &mut Base {ref mut encoder, ref mut world_buffer, ..} = self;
        encoder.update_buffer(&world_buffer, &[w], 0).unwrap();
    }

    pub fn load_texture (&mut self, path: &str) -> Texture {
        use gfx::Factory;
        let img = ::image::open(path).unwrap().to_rgba();
        let (width, height) = img.dimensions();
        let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
        let mipmap = gfx::texture::Mipmap::Allocated;
        let (_, view) = self.factory.create_texture_immutable_u8::<ColorFormat>(kind, mipmap, &[&img]).unwrap();
        self.encoder.generate_mipmap(&view);
        view
    }
}
