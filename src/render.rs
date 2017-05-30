use cgmath::{Matrix4, Transform as Transform_};
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
use {Hub, SubNode, VisualData, Scene, SceneId, Transform};

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;
pub type ConstantBuffer = gfx::handle::Buffer<back::Resources, Locals>;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 4] = "a_Position",
        uv: [f32; 2] = "a_TexCoord",
        normal: [gfx::format::I8Norm; 4] = "a_Normal",
    }

    constant Locals {
        mx_world: [[f32; 4]; 4] = "u_World",
        color: [f32; 4] = "u_Color",
    }

    constant Globals {
        mx_vp: [[f32; 4]; 4] = "u_ViewProj",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        cb_locals: gfx::ConstantBuffer<Locals> = "cb_Locals",
        cb_globals: gfx::ConstantBuffer<Globals> = "cb_Globals",
        tex_map: gfx::TextureSampler<[f32; 4]> = "t_Map",
        out_color: gfx::BlendTarget<ColorFormat> =
            ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::REPLACE),
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

const BASIC_VS: &'static [u8] = b"
    #version 150 core
    in vec4 a_Position;
    in vec4 a_Normal;
    uniform cb_Globals {
        mat4 u_ViewProj;
    };
    uniform cb_Locals {
        mat4 u_World;
        vec4 u_Color;
    };
    void main() {
        gl_Position = u_ViewProj * u_World * a_Position;
    }
";
const BASIC_FS: &'static [u8] = b"
    #version 150 core
    uniform cb_Locals {
        mat4 u_World;
        vec4 u_Color;
    };
    void main() {
        gl_FragColor = u_Color;
    }
";

const SPRITE_VS: &'static [u8] = b"
    #version 150 core
    in vec4 a_Position;
    in vec2 a_TexCoord;
    out vec2 v_TexCoord;
    uniform cb_Globals {
        mat4 u_ViewProj;
    };
    uniform cb_Locals {
        mat4 u_World;
        vec4 u_Color;
    };
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


impl Hub {
    fn visualize<F>(&mut self, scene_id: SceneId, mut fun: F)
        where F: FnMut(&VisualData<ConstantBuffer>, &Transform)
    {
        let mut cursor = self.nodes.cursor_alive();
        while let Some(mut item) = cursor.next() {
            if !item.visible {
                item.world_visible = false;
                continue
            }
            let (visibility, affilation, transform) = match item.parent {
                Some(ref parent_ptr) => {
                    let parent = item.look_back(parent_ptr).unwrap();
                    (parent.world_visible, parent.scene_id,
                     parent.world_transform.concat(&item.transform))
                },
                None => (true, item.scene_id, item.transform),
            };
            item.world_visible = visibility;
            item.scene_id = affilation;
            item.world_transform = transform;

            if visibility && affilation == Some(scene_id) {
                if let SubNode::Visual(ref data) = item.sub_node {
                    fun(data, &item.world_transform);
                }
            }
        }
    }
}


pub struct Renderer {
    device: back::Device,
    encoder: gfx::Encoder<back::Resources, back::CommandBuffer>,
    const_buf: gfx::handle::Buffer<back::Resources, Globals>,
    out_color: gfx::handle::RenderTargetView<back::Resources, ColorFormat>,
    out_depth: gfx::handle::DepthStencilView<back::Resources, DepthFormat>,
    pso_line_basic: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_mesh_basic_fill: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_mesh_basic_wireframe: gfx::PipelineState<back::Resources, pipe::Meta>,
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
        let prog_basic = gl_factory.link_program(BASIC_VS, BASIC_FS).unwrap();
        let prog_sprite = gl_factory.link_program(SPRITE_VS, SPRITE_FS).unwrap();
        let rast_fill = gfx::state::Rasterizer::new_fill().with_cull_back();
        let rast_wire = gfx::state::Rasterizer {
            method: gfx::state::RasterMethod::Line(1),
            .. rast_fill
        };
        let (_, srv_white) = gl_factory.create_texture_immutable::<gfx::format::Rgba8>(
            gfx::texture::Kind::D2(1, 1, gfx::texture::AaMode::Single), &[&[[0xFF; 4]]]
            ).unwrap();
        let sampler = gl_factory.create_sampler_linear();
        let renderer = Renderer {
            device: device,
            encoder: gl_factory.create_command_buffer().into(),
            const_buf: gl_factory.create_constant_buffer(1),
            out_color: color,
            out_depth: depth,
            pso_line_basic: gl_factory.create_pipeline_from_program(&prog_basic,
                gfx::Primitive::LineStrip, rast_fill, pipe::new()
                ).unwrap(),
            pso_mesh_basic_fill: gl_factory.create_pipeline_from_program(&prog_basic,
                gfx::Primitive::TriangleList, rast_fill, pipe::new()
                ).unwrap(),
            pso_mesh_basic_wireframe: gl_factory.create_pipeline_from_program(&prog_basic,
                gfx::Primitive::TriangleList, rast_wire, pipe::new(),
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
        self.encoder.update_constant_buffer(&self.const_buf, &Globals {
            mx_vp: cam.to_view_proj().into(),
        });

        let mut hub = scene.hub.lock().unwrap();
        hub.process_messages();
        hub.visualize(scene.unique_id, |visual, transform| {
            //TODO: batch per PSO
            let (pso, color, map) = match visual.material {
                Material::LineBasic { color } => (&self.pso_line_basic, color, None),
                Material::MeshBasic { color, wireframe: false } => (&self.pso_mesh_basic_fill, color, None),
                Material::MeshBasic { color, wireframe: true } => (&self.pso_mesh_basic_wireframe, color, None),
                Material::MeshLambert { color } => (&self.pso_mesh_basic_fill, color, None), //TEMP
                Material::Sprite { ref map } => (&self.pso_sprite, !0, Some(map)),
            };
            self.encoder.update_constant_buffer(&visual.payload, &Locals {
                mx_world: Matrix4::from(*transform).into(),
                color: color_to_f32(color),
            });
            //TODO: avoid excessive cloning
            let data = pipe::Data {
                vbuf: visual.gpu_data.vertices.clone(),
                cb_locals: visual.payload.clone(),
                cb_globals: self.const_buf.clone(),
                tex_map: map.unwrap_or(&self.map_default).to_param(),
                out_color: self.out_color.clone(),
                out_depth: self.out_depth.clone(),
            };
            self.encoder.draw(&visual.gpu_data.slice, pso, &data);
        });

        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().unwrap();
    }
}
