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
use mint;

pub use self::back::Factory as BackendFactory;
pub use self::back::Resources as BackendResources;
use camera::Projection;
use factory::{Factory, ShadowMap, Texture};
use scene::{Color, Background, Material};
use {SubLight, SubNode, Scene, ShadowProjection, Camera};

/// The format of the back buffer color requested from the windowing system.
pub type ColorFormat = gfx::format::Srgba8;
/// The format of the depth stencil buffer requested from the windowing system.
pub type DepthFormat = gfx::format::DepthStencil;
pub type ShadowFormat = gfx::format::Depth32F;
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
        mat_params: [f32; 4] = "u_MatParams",
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


fn get_shader(root: &str, name: &str, variant: &str) -> String {
    use std::fs::File;
    use std::io::{BufRead, BufReader, Read};
    let mut code = String::new();
    let shader_path = format!("{}/{}_{}.glsl", root, name, variant);
    for line in BufReader::new(File::open(shader_path).unwrap()).lines() {
        let line = line.unwrap();
        if line.starts_with("#include") {
            for dep_name in line.split(' ').skip(1) {
                code += &format!("//!including {}:\n", dep_name);
                File::open(format!("{}/{}.glsl", root, dep_name)).unwrap()
                     .read_to_string(&mut code).unwrap();
            }
        } else {
            code.push_str(&line);
            code.push('\n');
        }
    }
    code
}

fn load_program<R, F>(root: &str, name: &str, factory: &mut F)
                      -> gfx::handle::Program<R> where
    R: gfx::Resources,
    F: gfx::Factory<R>,
{
    let code_vs = get_shader(root, name, "vs");
    let code_ps = get_shader(root, name, "ps");

    factory.link_program(code_vs.as_bytes(), code_ps.as_bytes()).unwrap()
}

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
#[derive(Clone, Debug)]
pub struct GpuData {
    pub slice: gfx::Slice<back::Resources>,
    pub vertices: gfx::handle::Buffer<back::Resources, Vertex>,
    pub constants: gfx::handle::Buffer<back::Resources, Locals>,
    pub pending: Option<DynamicData>,
}

#[derive(Clone, Debug)]
pub struct DynamicData {
    pub num_vertices: usize,
    pub buffer: gfx::handle::Buffer<back::Resources, Vertex>,
}

/// Shadow type is used to specify shadow's rendering algorithm.
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

/// Handle for additional viewport to render some relevant debug information.
/// See [`Renderer::debug_shadow_quad`](struct.Renderer.html#method.debug_shadow_quad).
pub struct DebugQuadHandle(froggy::Pointer<DebugQuad>);

/// Renders [`Scene`](struct.Scene.html) by [`Camera`](struct.Camera.html).
///
/// See [Window::render](struct.Window.html#method.render).
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
    pso_mesh_gouraud: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_mesh_phong: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_sprite: gfx::PipelineState<back::Resources, pipe::Meta>,
    pso_shadow: gfx::PipelineState<back::Resources, shadow_pipe::Meta>,
    pso_quad: gfx::PipelineState<back::Resources, quad_pipe::Meta>,
    map_default: Texture<[f32; 4]>,
    shadow_default: Texture<f32>,
    debug_quads: froggy::Storage<DebugQuad>,
    size: (u32, u32),
    /// `ShadowType` of this `Renderer`.
    pub shadow: ShadowType,
}

impl Renderer {
    #[cfg(feature = "opengl")]
    #[doc(hidden)]
    pub fn new(builder: glutin::WindowBuilder, event_loop: &glutin::EventsLoop,
               shader_path: &str) -> (Self, glutin::Window, Factory) {
        use gfx::texture as t;
        let (window, device, mut gl_factory, color, depth) =
            gfx_window_glutin::init(builder, event_loop);
        let prog_basic = load_program(shader_path, "basic", &mut gl_factory);
        let prog_gouraud = load_program(shader_path, "gouraud", &mut gl_factory);
        let prog_phong = load_program(shader_path, "phong", &mut gl_factory);
        let prog_sprite = load_program(shader_path, "sprite", &mut gl_factory);
        let prog_shadow = load_program(shader_path, "shadow", &mut gl_factory);
        let prog_quad = load_program(shader_path, "quad", &mut gl_factory);
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
            pso_mesh_gouraud: gl_factory.create_pipeline_from_program(&prog_gouraud,
                gfx::Primitive::TriangleList, rast_fill, pipe::new()
                ).unwrap(),
            pso_mesh_phong: gl_factory.create_pipeline_from_program(&prog_phong,
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

    #[doc(hidden)]
    pub fn resize(&mut self, window: &glutin::Window) {
        self.size = window.get_inner_size_pixels().unwrap();
        gfx_window_glutin::update_views(window, &mut self.out_color, &mut self.out_depth);
    }

    /// Returns current viewport aspect, i.e. width / height.
    pub fn get_aspect(&self) -> f32 {
        self.size.0 as f32 / self.size.1 as f32
    }

    /// Map screen pixel coordinates to Normalized Display Coordinates.
    /// The lower left corner corresponds to (-1,-1), and the upper right corner
    /// corresponds to (1,1).
    pub fn map_to_ndc(&self, x: i32, y: i32) -> mint::Point2<f32> {
        mint::Point2 {
            x: 2.0 * x as f32 / self.size.0 as f32 - 1.0,
            y: 1.0 - 2.0 * y as f32 / self.size.1 as f32,
        }
    }

    /// See [`Window::render`](struct.Window.html#method.render).
    pub fn render<P: Projection>(&mut self, scene: &Scene, camera: &Camera<P>) {
        self.device.cleanup();
        let mut hub = scene.hub.lock().unwrap();
        hub.process_messages();
        hub.update_graph();

        // update dynamic meshes
        for mut node in hub.nodes.iter_alive_mut() {
            if !node.visible || node.scene_id != Some(scene.unique_id) {
                continue
            }
            if let SubNode::Visual(_, ref mut gpu_data) = node.sub_node {
                if let Some(dynamic) = gpu_data.pending.take() {
                    self.encoder.copy_buffer(&dynamic.buffer, &gpu_data.vertices, 0, 0, dynamic.num_vertices)
                                .unwrap();
                }
            }
        }

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
                    let mx_proj: [[f32; 4]; 4] = match projection {
                        &ShadowProjection::Ortho(ref p) => p.get_matrix(aspect),
                    }.into();
                    let mx_view = Matrix4::from(
                        node.world_transform.inverse_transform().unwrap());
                    shadow_requests.push(ShadowRequest {
                        target,
                        resource: map.to_resource(),
                        matrix: Matrix4::from(mx_proj) * mx_view,
                    });
                    shadow_requests.len() as i32 - 1
                } else {
                    -1
                };
                let mut color_back = 0;
                let mut p = node.world_transform.disp.extend(1.0);
                let d = node.world_transform.rot * Vector3::unit_z();
                let intensity = match light.sub_light {
                    SubLight::Ambient => {
                        [light.intensity, 0.0, 0.0, 0.0]
                    }
                    SubLight::Directional => {
                        p = d.extend(0.0);
                        [0.0, light.intensity, 0.0, 0.0]
                    }
                    SubLight::Hemisphere{ ground } => {
                        color_back = ground | 0x010101; // can't be 0
                        p = d.extend(0.0);
                        [light.intensity, 0.0, 0.0, 0.0]
                    }
                    SubLight::Point => {
                        [0.0, light.intensity, 0.0, 0.0]
                    }
                };
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
                let gpu_data = match node.sub_node {
                    SubNode::Visual(_, ref data) => data,
                    _ => continue
                };
                self.encoder.update_constant_buffer(&gpu_data.constants, &Locals {
                    mx_world: Matrix4::from(node.world_transform).into(),
                    color: [0.0; 4],
                    mat_params: [0.0; 4],
                    uv_range: [0.0; 4],
                });
                //TODO: avoid excessive cloning
                let data = shadow_pipe::Data {
                    vbuf: gpu_data.vertices.clone(),
                    cb_locals: gpu_data.constants.clone(),
                    cb_globals: self.const_buf.clone(),
                    target: request.target.clone(),
                };
                self.encoder.draw(&gpu_data.slice, &self.pso_shadow, &data);
            }
        }

        // prepare target and globals
        let mx_vp = {
            let p: [[f32; 4]; 4] = camera.projection.get_matrix(self.get_aspect()).into();
            let node = &hub.nodes[&camera.object.node];
            let w = match node.scene_id {
                Some(id) if id == scene.unique_id => node.world_transform,
                Some(_) => panic!("Camera does not belong to this scene"),
                None => node.transform,
            };
            Matrix4::from(p) * Matrix4::from(w.inverse_transform().unwrap())
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
            let (material, gpu_data) = match node.sub_node {
                SubNode::Visual(ref mat, ref data) => (mat, data),
                _ => continue
            };

            //TODO: batch per PSO
            let (pso, color, param0, map) = match *material {
                Material::LineBasic { color } => (&self.pso_line_basic, color, 0.0, None),
                Material::MeshBasic { color, ref map, wireframe: false } => (&self.pso_mesh_basic_fill, color, 0.0, map.as_ref()),
                Material::MeshBasic { color, map: ref _map, wireframe: true } => (&self.pso_mesh_basic_wireframe, color, 0.0, None),
                Material::MeshLambert { color, flat } => (&self.pso_mesh_gouraud, color, if flat {0.0} else {1.0}, None),
                Material::MeshPhong { color, glossiness } => (&self.pso_mesh_phong, color, glossiness, None),
                Material::Sprite { ref map } => (&self.pso_sprite, !0, 0.0, Some(map)),
            };
            let uv_range = match map {
                Some(ref map) => map.get_uv_range(),
                None => [0.0; 4],
            };
            self.encoder.update_constant_buffer(&gpu_data.constants, &Locals {
                mx_world: Matrix4::from(node.world_transform).into(),
                color: decode_color(color),
                mat_params: [param0, 0.0, 0.0, 0.0],
                uv_range,
            });
            //TODO: avoid excessive cloning
            let data = pipe::Data {
                vbuf: gpu_data.vertices.clone(),
                cb_locals: gpu_data.constants.clone(),
                cb_lights: self.light_buf.clone(),
                cb_globals: self.const_buf.clone(),
                tex_map: map.unwrap_or(&self.map_default).to_param(),
                shadow_map0: (shadow0.clone(), shadow_sampler.clone()),
                shadow_map1: (shadow1.clone(), shadow_sampler.clone()),
                out_color: self.out_color.clone(),
                out_depth: self.out_depth.clone(),
            };
            self.encoder.draw(&gpu_data.slice, pso, &data);
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
            let p0 = self.map_to_ndc(pos[0], pos[1]);
            let p1 = self.map_to_ndc(pos[0] + quad.size[0], pos[1] + quad.size[1]);
            self.encoder.update_constant_buffer(&self.quad_buf, &QuadParams {
                rect: [p0.x, p0.y, p1.x, p1.y],
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

    /// Draw [`ShadowMap`](struct.ShadowMap.html) for debug purposes.
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
