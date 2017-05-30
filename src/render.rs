use cgmath::{Matrix4, Vector3};
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
use {SubLight, SubNode, Scene};

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;
pub type ConstantBuffer = gfx::handle::Buffer<back::Resources, Locals>;
const MAX_LIGHTS: usize = 4;

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

    constant LightParam {
        pos: [f32; 4] = "pos",
        dir: [f32; 4] = "dir",
        focus: [f32; 4] = "focus",
        color: [f32; 4] = "color",
        color_back: [f32; 4] = "color_back",
        intensity: [f32; 4] = "intensity",
    }

    constant Globals {
        mx_vp: [[f32; 4]; 4] = "u_ViewProj",
        num_lights: u32 = "u_NumLights",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        cb_locals: gfx::ConstantBuffer<Locals> = "b_Locals",
        cb_lights: gfx::ConstantBuffer<LightParam> = "b_Lights",
        cb_globals: gfx::ConstantBuffer<Globals> = "b_Globals",
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
    uniform b_Globals {
        mat4 u_ViewProj;
    };
    uniform b_Locals {
        mat4 u_World;
        vec4 u_Color;
    };
    void main() {
        gl_Position = u_ViewProj * u_World * a_Position;
    }
";
const BASIC_FS: &'static [u8] = b"
    #version 150 core
    uniform b_Locals {
        mat4 u_World;
        vec4 u_Color;
    };
    void main() {
        gl_FragColor = u_Color;
    }
";

const PHONG_VS: &'static [u8] = b"
    #version 150 core
    in vec4 a_Position;
    in vec4 a_Normal;
    out vec3 v_World;
    out vec3 v_Normal;
    out vec3 v_Half[4];
    struct Light {
        vec4 pos;
        vec4 dir;
        vec4 focus;
        vec4 color;
        vec4 color_back;
        vec4 intensity;
    };
    uniform b_Lights {
        Light u_Lights[4];
    };
    uniform b_Globals {
        mat4 u_ViewProj;
        uint u_NumLights;
    };
    uniform b_Locals {
        mat4 u_World;
        vec4 u_Color;
    };
    void main() {
        vec4 world = u_World * a_Position;
        v_World = world.xyz;
        v_Normal = normalize(mat3(u_World) * a_Normal.xyz);
        for(uint i=0U; i<4U && i < u_NumLights; ++i) {
            vec3 dir = u_Lights[i].pos.xyz - u_Lights[i].pos.w * world.xyz;
            v_Half[i] = normalize(v_Normal + normalize(dir));
        }
        gl_Position = u_ViewProj * world;
    }
";
const PHONG_FS: &'static [u8] = b"
    #version 150 core
    in vec3 v_World;
    in vec3 v_Normal;
    in vec3 v_Half[4];
    struct Light {
        vec4 pos;
        vec4 dir;
        vec4 focus;
        vec4 color;
        vec4 color_back;
        vec4 intensity;
    };
    uniform b_Lights {
        Light u_Lights[4];
    };
    uniform b_Globals {
        mat4 u_ViewProj;
        uint u_NumLights;
    };
    uniform b_Locals {
        mat4 u_World;
        vec4 u_Color;
    };
    void main() {
        vec4 color = vec4(0.0);
        vec3 normal = normalize(v_Normal);
        for(uint i=0U; i<4U && i < u_NumLights; ++i) {
            Light light = u_Lights[i];
            vec3 dir = light.pos.xyz - light.pos.w * v_World.xyz;
            float dot_nl = dot(normal, normalize(dir));
            // hemisphere light test
            if (dot(light.color_back, light.color_back) > 0.0) {
                vec4 irradiance = mix(light.color_back, light.color, dot_nl*0.5 + 0.5);
                color += light.intensity.y * u_Color * irradiance;
            } else {
                float kd = light.intensity.x + light.intensity.y * max(0.0, dot_nl);
                color += u_Color * light.color * kd;
            }
            if (dot_nl > 0.0 && light.intensity.z > 0.0) {
                float ks = dot(normal, normalize(v_Half[i]));
                if (ks > 0.0) {
                    color += light.color * pow(ks, light.intensity.z);
                }
            }
        }
        gl_FragColor = color;
    }
";

const SPRITE_VS: &'static [u8] = b"
    #version 150 core
    in vec4 a_Position;
    in vec2 a_TexCoord;
    out vec2 v_TexCoord;
    uniform b_Globals {
        mat4 u_ViewProj;
    };
    uniform b_Locals {
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


/// sRGB to linear conversion from:
/// https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_sRGB_decode.txt
fn decode_color(c: Color) -> [f32; 4] {
    let f = |xu: u32| {
        let x = (xu & 0xFF) as f32 / 255.0;
        if x > 0.04045 {
            ((x + 0.055) / 1.055).powf(2.4)
        } else {
            x / 12.92
        }
    };
    [f(c>>16), f(c>>8), f(c), 0.0]
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
    const_buf: gfx::handle::Buffer<back::Resources, Globals>,
    light_buf: gfx::handle::Buffer<back::Resources, LightParam>,
    out_color: gfx::handle::RenderTargetView<back::Resources, ColorFormat>,
    out_depth: gfx::handle::DepthStencilView<back::Resources, DepthFormat>,
    pso_line_basic: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_mesh_basic_fill: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_mesh_basic_wireframe: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_mesh_phong: gfx::PipelineState<back::Resources, pipe::Meta>,
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
        let prog_phong = gl_factory.link_program(PHONG_VS, PHONG_FS).unwrap();
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
            light_buf: gl_factory.create_constant_buffer(MAX_LIGHTS),
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
            pso_mesh_phong:  gl_factory.create_pipeline_from_program(&prog_phong,
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
        let mut hub = scene.hub.lock().unwrap();
        hub.process_messages();
        hub.update_graph();

        // gather lights
        let mut lights = Vec::new();
        for node in hub.nodes.iter_alive() {
            if node.scene_id != Some(scene.unique_id) {
                continue
            }
            if let SubNode::Light(ref light) = node.sub_node {
                if lights.len() == MAX_LIGHTS {
                    //error!("Max number of lights ({}) reached", MAX_LIGHTS);
                    break;
                }
                let mut color_back = 0;
                let mut p = node.world_transform.disp.extend(1.0);
                let d = node.world_transform.rot * Vector3::unit_z();
                let mut intensity = [0.0, light.intensity, 0.0, 0.0];
                match light.sub_light {
                    SubLight::Ambient => {
                        intensity = [light.intensity, 0.0, 0.0, 0.0];
                    }
                    SubLight::Directional => {
                        p = d.extend(0.0);
                    }
                    SubLight::Hemisphere{ ground } => {
                        color_back = ground | 0x010101; // can't be 0
                        p = d.extend(0.0);
                    }
                    SubLight::Point => {
                        //empty
                    }
                }
                lights.push(LightParam {
                    pos: p.into(),
                    dir: d.extend(0.0).into(),
                    focus: [0.0, 0.0, 0.0, 0.0],
                    color: decode_color(light.color),
                    color_back: decode_color(color_back),
                    intensity: intensity,
                });
            }
        }

        // prepare target and globals
        self.device.cleanup();
        self.encoder.clear(&self.out_color, [0.0, 0.0, 0.0, 1.0]);
        self.encoder.clear_depth(&self.out_depth, 1.0);
        self.encoder.update_constant_buffer(&self.const_buf, &Globals {
            mx_vp: cam.to_view_proj().into(),
            num_lights: lights.len() as u32,
        });
        self.encoder.update_buffer(&self.light_buf, &lights, 0).unwrap();

        // render everything
        for node in hub.nodes.iter_alive() {
            if node.scene_id != Some(scene.unique_id) {
                continue;
            }
            let visual = match node.sub_node {
                SubNode::Visual(ref data) => data,
                _ => continue
            };

            //TODO: batch per PSO
            let (pso, color, map) = match visual.material {
                Material::LineBasic { color } => (&self.pso_line_basic, color, None),
                Material::MeshBasic { color, wireframe: false } => (&self.pso_mesh_basic_fill, color, None),
                Material::MeshBasic { color, wireframe: true } => (&self.pso_mesh_basic_wireframe, color, None),
                Material::MeshLambert { color } => (&self.pso_mesh_phong, color, None),
                Material::Sprite { ref map } => (&self.pso_sprite, !0, Some(map)),
            };
            self.encoder.update_constant_buffer(&visual.payload, &Locals {
                mx_world: Matrix4::from(node.world_transform).into(),
                color: decode_color(color),
            });
            //TODO: avoid excessive cloning
            let data = pipe::Data {
                vbuf: visual.gpu_data.vertices.clone(),
                cb_locals: visual.payload.clone(),
                cb_lights: self.light_buf.clone(),
                cb_globals: self.const_buf.clone(),
                tex_map: map.unwrap_or(&self.map_default).to_param(),
                out_color: self.out_color.clone(),
                out_depth: self.out_depth.clone(),
            };
            self.encoder.draw(&visual.gpu_data.slice, pso, &data);
        }

        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().unwrap();
    }
}
