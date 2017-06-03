use cgmath::{Matrix4, Vector3, Transform as Transform_};
use froggy;
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
use factory::{Factory, ShadowMap, Texture};
use scene::{Color, Background, Material};
use {SubLight, SubNode, Scene, ShadowProjection, Camera, Projection};

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;
pub type ShadowFormat = gfx::format::Depth32F;
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
        uv_range: [f32; 4] = "u_UvRange",
    }

    constant LightParam {
        projection: [[f32; 4]; 4] = "projection",
        pos: [f32; 4] = "pos",
        dir: [f32; 4] = "dir",
        focus: [f32; 4] = "focus",
        color: [f32; 4] = "color",
        color_back: [f32; 4] = "color_back",
        intensity: [f32; 4] = "intensity",
        shadow_params: [i32; 4] = "shadow_params",
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
        shadow_map0: gfx::TextureSampler<f32> = "t_Shadow0",
        shadow_map1: gfx::TextureSampler<f32> = "t_Shadow1",
        out_color: gfx::BlendTarget<ColorFormat> =
            ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::REPLACE),
        out_depth: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }

    pipeline shadow_pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        cb_locals: gfx::ConstantBuffer<Locals> = "b_Locals",
        cb_globals: gfx::ConstantBuffer<Globals> = "b_Globals",
        target: gfx::DepthTarget<ShadowFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }

    constant QuadParams {
        rect: [f32; 4] = "u_Rect",
    }

    pipeline quad_pipe {
        params: gfx::ConstantBuffer<QuadParams> = "b_Params",
        resource: gfx::RawShaderResource = "t_Input",
        sampler: gfx::Sampler = "t_Input",
        target: gfx::RenderTarget<ColorFormat> = "Target0",
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
        vec4 u_UvRange;
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
        vec4 u_UvRange;
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
        mat4 projection;
        vec4 pos;
        vec4 dir;
        vec4 focus;
        vec4 color;
        vec4 color_back;
        vec4 intensity;
        ivec4 shadow_params;
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
        vec4 u_UvRange;
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
    uniform sampler2DShadow t_Shadow0;
    uniform sampler2DShadow t_Shadow1;
    struct Light {
        mat4 projection;
        vec4 pos;
        vec4 dir;
        vec4 focus;
        vec4 color;
        vec4 color_back;
        vec4 intensity;
        ivec4 shadow_params;
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
        vec4 u_UvRange;
    };
    void main() {
        vec4 color = vec4(0.0);
        vec3 normal = normalize(v_Normal);
        for(uint i=0U; i<4U && i < u_NumLights; ++i) {
            Light light = u_Lights[i];
            float shadow = 1.0;
            vec4 lit_space = light.projection * vec4(v_World, 1.0);
            if (light.shadow_params[0] == 0) {
                shadow = texture(t_Shadow0, 0.5 * lit_space.xyz / lit_space.w + 0.5);
            }
            if (light.shadow_params[0] == 1) {
                shadow = texture(t_Shadow1, 0.5 * lit_space.xyz / lit_space.w + 0.5);
            }
            if (shadow == 0.0) {
                continue;
            }
            vec3 dir = light.pos.xyz - light.pos.w * v_World.xyz;
            float dot_nl = dot(normal, normalize(dir));
            // hemisphere light test
            if (dot(light.color_back, light.color_back) > 0.0) {
                vec4 irradiance = mix(light.color_back, light.color, dot_nl*0.5 + 0.5);
                color += shadow * light.intensity.y * u_Color * irradiance;
            } else {
                float kd = light.intensity.x + light.intensity.y * max(0.0, dot_nl);
                color += shadow * kd * u_Color * light.color;
            }
            if (dot_nl > 0.0 && light.intensity.z > 0.0) {
                float ks = dot(normal, normalize(v_Half[i]));
                if (ks > 0.0) {
                    color += shadow * pow(ks, light.intensity.z) * light.color;
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
        vec4 u_UvRange;
    };
    void main() {
        v_TexCoord = mix(u_UvRange.xy, u_UvRange.zw, a_TexCoord);
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

const SHADOW_VS: &'static [u8] = b"
    #version 150 core
    in vec4 a_Position;
    uniform b_Globals {
        mat4 u_ViewProj;
    };
    uniform b_Locals {
        mat4 u_World;
        vec4 u_Color;
        vec4 u_UvRange;
    };
    void main() {
        gl_Position = u_ViewProj * u_World * a_Position;
    }
";
const SHADOW_FS: &'static [u8] = b"
    #version 150 core
    void main() {}
";

const QUAD_VS: &'static [u8] = b"
    #version 150 core
    out vec2 v_TexCoord;
    uniform b_Params {
        vec4 u_Rect;
    };
    void main() {
        v_TexCoord = gl_VertexID==0 ? vec2(1.0, 0.0) :
                     gl_VertexID==1 ? vec2(0.0, 0.0) :
                     gl_VertexID==2 ? vec2(1.0, 1.0) :
                                      vec2(0.0, 1.0) ;
        vec2 pos = mix(u_Rect.xy, u_Rect.zw, v_TexCoord);
        gl_Position = vec4(pos, 0.0, 1.0);
    }
";
const QUAD_FS: &'static [u8] = b"
    #version 150 core
    in vec2 v_TexCoord;
    uniform sampler2D t_Input;
    void main() {
        gl_FragColor = texture(t_Input, v_TexCoord);
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

pub enum ShadowType {
    /// Force no shadows.
    Off,
    /// Basic (and fast) single-sample shadows.
    Basic,
    /// Percentage-closest filter (PCF).
    Pcf,
}

struct DebugQuad {
    resource: gfx::handle::RawShaderResourceView<back::Resources>,
    pos: [i32; 2],
    size: [i32; 2],
}

pub struct DebugQuadHandle(froggy::Pointer<DebugQuad>);

pub struct Renderer {
    device: back::Device,
    encoder: gfx::Encoder<back::Resources, back::CommandBuffer>,
    const_buf: gfx::handle::Buffer<back::Resources, Globals>,
    quad_buf: gfx::handle::Buffer<back::Resources, QuadParams>,
    light_buf: gfx::handle::Buffer<back::Resources, LightParam>,
    out_color: gfx::handle::RenderTargetView<back::Resources, ColorFormat>,
    out_depth: gfx::handle::DepthStencilView<back::Resources, DepthFormat>,
    pso_line_basic: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_mesh_basic_fill: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_mesh_basic_wireframe: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_mesh_phong: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_sprite: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_shadow: gfx::PipelineState<back::Resources, shadow_pipe::Meta>,
    pso_quad: gfx::PipelineState<back::Resources, quad_pipe::Meta>,
    map_default: Texture<[f32; 4]>,
    shadow_default: Texture<f32>,
    debug_quads: froggy::Storage<DebugQuad>,
    size: (u32, u32),
    pub shadow: ShadowType,
}

impl Renderer {
    #[cfg(feature = "opengl")]
    pub fn new(builder: glutin::WindowBuilder, event_loop: &glutin::EventsLoop)
               -> (Self, glutin::Window, Factory) {
        use gfx::texture as t;
        let (window, device, mut gl_factory, color, depth) =
            gfx_window_glutin::init(builder, event_loop);
        let prog_basic = gl_factory.link_program(BASIC_VS, BASIC_FS).unwrap();
        let prog_phong = gl_factory.link_program(PHONG_VS, PHONG_FS).unwrap();
        let prog_sprite = gl_factory.link_program(SPRITE_VS, SPRITE_FS).unwrap();
        let prog_shadow = gl_factory.link_program(SHADOW_VS, SHADOW_FS).unwrap();
        let prog_quad = gl_factory.link_program(QUAD_VS, QUAD_FS).unwrap();
        let rast_fill = gfx::state::Rasterizer::new_fill().with_cull_back();
        let rast_wire = gfx::state::Rasterizer {
            method: gfx::state::RasterMethod::Line(1),
            .. rast_fill
        };
        let rast_shadow = gfx::state::Rasterizer {
            offset: Some(gfx::state::Offset(2, 2)),
            .. rast_fill
        };
        let (_, srv_white) = gl_factory.create_texture_immutable::<gfx::format::Rgba8>(
            t::Kind::D2(1, 1, t::AaMode::Single), &[&[[0xFF; 4]]]
            ).unwrap();
        let (_, srv_shadow) = gl_factory.create_texture_immutable::<(gfx::format::R32, gfx::format::Float)>(
            t::Kind::D2(1, 1, t::AaMode::Single), &[&[0x3F800000]]
            ).unwrap();
        let sampler = gl_factory.create_sampler_linear();
        let sampler_shadow = gl_factory.create_sampler(t::SamplerInfo {
            comparison: Some(gfx::state::Comparison::Less),
            border: t::PackedColor(!0), // clamp to 1.0
            .. t::SamplerInfo::new(t::FilterMethod::Bilinear, t::WrapMode::Border)
        });
        let renderer = Renderer {
            device: device,
            encoder: gl_factory.create_command_buffer().into(),
            const_buf: gl_factory.create_constant_buffer(1),
            quad_buf: gl_factory.create_constant_buffer(1),
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
            pso_shadow: gl_factory.create_pipeline_from_program(&prog_shadow,
                gfx::Primitive::TriangleList, rast_shadow, shadow_pipe::new()
                ).unwrap(),
            pso_quad: gl_factory.create_pipeline_from_program(&prog_quad,
                gfx::Primitive::TriangleStrip, rast_fill, quad_pipe::new()
                ).unwrap(),
            map_default: Texture::new(srv_white, sampler, [1, 1]),
            shadow_default: Texture::new(srv_shadow, sampler_shadow, [1, 1]),
            shadow: ShadowType::Basic,
            debug_quads: froggy::Storage::new(),
            size: window.get_inner_size_pixels().unwrap(),
        };
        let factory = Factory::new(gl_factory);
        (renderer, window, factory)
    }

    pub fn resize(&mut self, window: &glutin::Window) {
        self.size = window.get_inner_size_pixels().unwrap();
        gfx_window_glutin::update_views(window, &mut self.out_color, &mut self.out_depth);
    }

    pub fn get_aspect(&self) -> f32 {
        self.size.0 as f32 / self.size.1 as f32
    }

    pub fn map_to_ndc(&self, x: i32, y: i32) -> (f32, f32) {
        (2.0 * x as f32 / self.size.0 as f32 - 1.0,
         1.0 - 2.0 * y as f32 / self.size.1 as f32)
    }

    pub fn render<P: Projection>(&mut self, scene: &Scene, camera: &Camera<P>) {
        self.device.cleanup();
        let mut hub = scene.hub.lock().unwrap();
        hub.process_messages();
        hub.update_graph();

        // gather lights
        struct ShadowRequest {
            target: gfx::handle::DepthStencilView<back::Resources, ShadowFormat>,
            resource: gfx::handle::ShaderResourceView<back::Resources, f32>,
            matrix: Matrix4<f32>,
        }
        let mut lights = Vec::new();
        let mut shadow_requests = Vec::new();
        for node in hub.nodes.iter_alive() {
            if !node.visible || node.scene_id != Some(scene.unique_id) {
                continue
            }
            if let SubNode::Light(ref light) = node.sub_node {
                if lights.len() == MAX_LIGHTS {
                    error!("Max number of lights ({}) reached", MAX_LIGHTS);
                    break;
                }
                let shadow_index = if let Some((ref map, ref projection)) = light.shadow {
                    let target = map.to_target();
                    let dim = target.get_dimensions();
                    let aspect = dim.0 as f32 / dim.1 as f32;
                    let mx_proj = match projection {
                        &ShadowProjection::Ortho(ref p) => p.get_matrix(aspect),
                    };
                    let mx_view = Matrix4::from(
                        node.world_transform.inverse_transform().unwrap());
                    shadow_requests.push(ShadowRequest {
                        target,
                        resource: map.to_resource(),
                        matrix: mx_proj * mx_view,
                    });
                    shadow_requests.len() as i32 - 1
                } else {
                    -1
                };
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
                let projection = if shadow_index >= 0 {
                    shadow_requests[shadow_index as usize].matrix.into()
                } else {
                    [[0.0; 4]; 4]
                };
                lights.push(LightParam {
                    projection,
                    pos: p.into(),
                    dir: d.extend(0.0).into(),
                    focus: [0.0, 0.0, 0.0, 0.0],
                    color: decode_color(light.color),
                    color_back: decode_color(color_back),
                    intensity,
                    shadow_params: [shadow_index, 0, 0, 0],
                });
            }
        }

        // render shadow maps
        for request in &shadow_requests {
            self.encoder.clear_depth(&request.target, 1.0);
            self.encoder.update_constant_buffer(&self.const_buf, &Globals {
                mx_vp: request.matrix.into(),
                num_lights: 0,
            });
            for node in hub.nodes.iter_alive() {
                if !node.visible || node.scene_id != Some(scene.unique_id) {
                    continue;
                }
                let visual = match node.sub_node {
                    SubNode::Visual(ref data) => data,
                    _ => continue
                };
                self.encoder.update_constant_buffer(&visual.payload, &Locals {
                    mx_world: Matrix4::from(node.world_transform).into(),
                    color: [0.0; 4],
                    uv_range: [0.0; 4],
                });
                //TODO: avoid excessive cloning
                let data = shadow_pipe::Data {
                    vbuf: visual.gpu_data.vertices.clone(),
                    cb_locals: visual.payload.clone(),
                    cb_globals: self.const_buf.clone(),
                    target: request.target.clone(),
                };
                self.encoder.draw(&visual.gpu_data.slice, &self.pso_shadow, &data);
            }
        }

        // prepare target and globals
        let mx_vp = {
            let p = camera.projection.get_matrix(self.get_aspect());
            let node = &hub.nodes[&camera.object.node];
            let w = match node.scene_id {
                Some(id) if id == scene.unique_id => node.world_transform,
                Some(_) => panic!("Camera does not belong to this scene"),
                None => node.transform,
            };
            p * Matrix4::from(w.inverse_transform().unwrap())
        };
        match scene.background {
            Background::Color(color) => {
                self.encoder.clear(&self.out_color, decode_color(color));
            }
        }
        self.encoder.clear_depth(&self.out_depth, 1.0);
        self.encoder.update_constant_buffer(&self.const_buf, &Globals {
            mx_vp: mx_vp.into(),
            num_lights: lights.len() as u32,
        });
        self.encoder.update_buffer(&self.light_buf, &lights, 0).unwrap();

        // render everything
        let (shadow_default, shadow_sampler) = self.shadow_default.to_param();
        let shadow0 = match shadow_requests.get(0) {
            Some(ref request) => request.resource.clone(),
            None => shadow_default.clone(),
        };
        let shadow1 = match shadow_requests.get(1) {
            Some(ref request) => request.resource.clone(),
            None => shadow_default.clone(),
        };
        for node in hub.nodes.iter_alive() {
            if !node.visible || node.scene_id != Some(scene.unique_id) {
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
            let uv_range = match map {
                Some(ref map) => map.get_uv_range(),
                None => [0.0; 4],
            };
            self.encoder.update_constant_buffer(&visual.payload, &Locals {
                mx_world: Matrix4::from(node.world_transform).into(),
                color: decode_color(color),
                uv_range,
            });
            //TODO: avoid excessive cloning
            let data = pipe::Data {
                vbuf: visual.gpu_data.vertices.clone(),
                cb_locals: visual.payload.clone(),
                cb_lights: self.light_buf.clone(),
                cb_globals: self.const_buf.clone(),
                tex_map: map.unwrap_or(&self.map_default).to_param(),
                shadow_map0: (shadow0.clone(), shadow_sampler.clone()),
                shadow_map1: (shadow1.clone(), shadow_sampler.clone()),
                out_color: self.out_color.clone(),
                out_depth: self.out_depth.clone(),
            };
            self.encoder.draw(&visual.gpu_data.slice, pso, &data);
        }

        // draw debug quads
        self.debug_quads.sync_pending();
        for quad in self.debug_quads.iter_alive() {
            let pos = [
                if quad.pos[0] >= 0 {
                    quad.pos[0]
                } else {
                    self.size.0 as i32 + quad.pos[0] - quad.size[0]
                },
                if quad.pos[1] >= 0 {
                    quad.pos[1]
                } else {
                    self.size.1 as i32 + quad.pos[1] - quad.size[1]
                },
            ];
            let (p0x, p0y) = self.map_to_ndc(pos[0], pos[1]);
            let (p1x, p1y) = self.map_to_ndc(pos[0] + quad.size[0], pos[1] + quad.size[1]);
            self.encoder.update_constant_buffer(&self.quad_buf, &QuadParams {
                rect: [p0x, p0y, p1x, p1y],
            });
            let slice = gfx::Slice {
                start: 0,
                end: 4,
                base_vertex: 0,
                instances: None,
                buffer: gfx::IndexBuffer::Auto,
            };
            let data = quad_pipe::Data {
                params: self.quad_buf.clone(),
                resource: quad.resource.clone(),
                sampler: self.map_default.to_param().1,
                target: self.out_color.clone(),
            };
            self.encoder.draw(&slice, &self.pso_quad, &data);
        }

        self.encoder.flush(&mut self.device);
    }

    pub fn debug_shadow_quad(&mut self, map: &ShadowMap, _num_components: u8,
                             pos: [i16; 2], size: [u16; 2]) -> DebugQuadHandle {
        use gfx::memory::Typed;
        DebugQuadHandle(self.debug_quads.create(DebugQuad {
            resource: map.to_resource().raw().clone(),
            pos: [pos[0] as i32, pos[1] as i32],
            size: [size[0] as i32, size[1] as i32],
        }))
    }
}
