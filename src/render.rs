use cgmath;
use gfx;
use gfx::traits::{Device, Factory as Factory_, FactoryExt};
#[cfg(feature = "opengl")]
use gfx_device_gl as back;
#[cfg(feature = "opengl")]
use gfx_window_glutin;
#[cfg(feature = "opengl")]
use glutin;

pub use self::back::Factory as BackendFactory;
pub use self::back::Resources as BackendResources;
use camera::Camera;
use factory::{Factory, Texture};
use scene::{Color, Material};
use {Scene};


pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_vertex_struct!(Vertex {
    pos: [f32; 4] = "a_Position",
    uv: [f32; 2] = "a_TexCoord",
    normal: [gfx::format::I8Norm; 4] = "a_Normal",
});

gfx_pipeline!(pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    mx_vp: gfx::Global<[[f32; 4]; 4]> = "u_ViewProj",
    mx_world: gfx::Global<[[f32; 4]; 4]> = "u_World",
    color: gfx::Global<[f32; 4]> = "u_Color",
    tex_map: gfx::TextureSampler<[f32; 4]> = "t_Map",
    out_color: gfx::BlendTarget<ColorFormat> =
        ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::REPLACE),
});

const LINE_VS: &'static [u8] = b"
    #version 150 core
    in vec4 a_Position;
    uniform mat4 u_ViewProj;
    uniform mat4 u_World;
    void main() {
        gl_Position = u_ViewProj * u_World * a_Position;
    }
";
const LINE_FS: &'static [u8] = b"
    #version 150 core
    uniform vec4 u_Color;
    void main() {
        gl_FragColor = u_Color;
    }
";

const MESH_VS: &'static [u8] = b"
    #version 150 core
    in vec4 a_Position;
    in vec2 a_TexCoord;
    out vec2 v_TexCoord;
    uniform mat4 u_ViewProj;
    uniform mat4 u_World;
    void main() {
        gl_Position = u_ViewProj * u_World * a_Position;
    }
";
const MESH_FS: &'static [u8] = b"
    #version 150 core
    in vec2 v_TexCoord; //TODO
    uniform vec4 u_Color;
    void main() {
        gl_FragColor = u_Color;
    }
";

const SPRITE_VS: &'static [u8] = b"
    #version 150 core
    in vec4 a_Position;
    in vec2 a_TexCoord;
    out vec2 v_TexCoord;
    uniform mat4 u_ViewProj;
    uniform mat4 u_World;
    void main() {
        v_TexCoord = a_TexCoord;
        gl_Position = u_ViewProj * u_World * a_Position;
    }
";
const SPRITE_FS: &'static [u8] = b"
    #version 150 core
    in vec2 v_TexCoord;
    uniform sampler2D t_Map;
    void main() {
        gl_FragColor = texture(t_Map, v_TexCoord);
    }
";


fn color_to_f32(c: Color) -> [f32; 4] {
    [((c>>16)&0xFF) as f32 / 255.0,
     ((c>>8) &0xFF) as f32 / 255.0,
     (c&0xFF) as f32 / 255.0,
     1.0]
}

//TODO: private fields?
#[derive(Clone)]
pub struct GpuData {
    pub slice: gfx::Slice<back::Resources>,
    pub vertices: gfx::handle::Buffer<back::Resources, Vertex>,
}


pub struct Renderer {
    device: back::Device,
    encoder: gfx::Encoder<back::Resources, back::CommandBuffer>,
    out_color: gfx::handle::RenderTargetView<back::Resources, ColorFormat>,
    out_depth: gfx::handle::DepthStencilView<back::Resources, DepthFormat>,
    pso_line_basic: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_mesh_basic: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_sprite: gfx::PipelineState<back::Resources, pipe::Meta>,
    map_default: Texture,
    size: (u32, u32),
    #[cfg(feature = "opengl")]
    window: glutin::Window,
}

impl Renderer {
    #[cfg(feature = "opengl")]
    pub fn new(builder: glutin::WindowBuilder, event_loop: &glutin::EventsLoop)
               -> (Self, Factory) {
        let (window, device, mut gl_factory, color, depth) =
            gfx_window_glutin::init(builder, event_loop);
        let prog_line = gl_factory.link_program(LINE_VS, LINE_FS).unwrap();
        let prog_mesh = gl_factory.link_program(MESH_VS, MESH_FS).unwrap();
        let prog_sprite = gl_factory.link_program(SPRITE_VS, SPRITE_FS).unwrap();
        let rast_fill = gfx::state::Rasterizer::new_fill().with_cull_back();
        let (_, srv_white) = gl_factory.create_texture_immutable::<gfx::format::Rgba8>(
            gfx::texture::Kind::D2(1, 1, gfx::texture::AaMode::Single), &[&[[0xFF; 4]]]
            ).unwrap();
        let sampler = gl_factory.create_sampler_linear();
        let renderer = Renderer {
            device: device,
            encoder: gl_factory.create_command_buffer().into(),
            out_color: color,
            out_depth: depth,
            pso_line_basic: gl_factory.create_pipeline_from_program(&prog_line,
                gfx::Primitive::LineStrip, rast_fill, pipe::new()
                ).unwrap(),
            pso_mesh_basic: gl_factory.create_pipeline_from_program(&prog_mesh,
                gfx::Primitive::TriangleList, rast_fill, pipe::new()
                ).unwrap(),
            pso_sprite: gl_factory.create_pipeline_from_program(&prog_sprite,
                gfx::Primitive::TriangleStrip, rast_fill, pipe::Init {
                    out_color: ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
                    .. pipe::new()
                }).unwrap(),
            map_default: Texture::new(srv_white, sampler),
            size: window.get_inner_size_pixels().unwrap(),
            window: window,
        };
        let factory = Factory::new(gl_factory);
        (renderer, factory)
    }

    pub fn resize(&mut self) {
        self.size = self.window.get_inner_size_pixels().unwrap();
        gfx_window_glutin::update_views(&self.window, &mut self.out_color, &mut self.out_depth);
    }

    pub fn get_aspect(&self) -> f32 {
        self.size.0 as f32 / self.size.1 as f32
    }

    pub fn map_to_ndc(&self, x: i32, y: i32) -> (f32, f32) {
        (2.0 * x as f32 / self.size.0 as f32 - 1.0,
         1.0 - 2.0 * y as f32 / self.size.1 as f32)
    }

    pub fn render<C: Camera>(&mut self, scene: &Scene, cam: &C) {
        self.device.cleanup();
        self.encoder.clear(&self.out_color, [0.0, 0.0, 0.0, 1.0]);
        self.encoder.clear_depth(&self.out_depth, 1.0);

        let mx_vp = cam.to_view_proj();
        for visual in &scene.visuals {
            let (pso, color, map) = match visual.material {
                Material::LineBasic { color } => (&self.pso_line_basic, color, None),
                Material::MeshBasic { color } => (&self.pso_mesh_basic, color, None),
                Material::Sprite { ref map } => (&self.pso_sprite, !0, Some(map)),
            };
            let mx_world = cgmath::Matrix4::from(scene.nodes[&visual.node].world);
            let data = pipe::Data {
                vbuf: visual.gpu_data.vertices.clone(),
                mx_vp: mx_vp.into(),
                mx_world: mx_world.into(),
                color: color_to_f32(color),
                tex_map: map.unwrap_or(&self.map_default).to_param(),
                out_color: self.out_color.clone(),
            };
            self.encoder.draw(&visual.gpu_data.slice, pso, &data);
        }

        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().unwrap();
    }
}
