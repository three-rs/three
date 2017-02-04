extern crate cgmath;
#[macro_use]
extern crate gfx;
extern crate winit;
#[cfg(feature = "opengl")]
extern crate gfx_device_gl as back;
#[cfg(feature = "opengl")]
extern crate gfx_window_glutin;
#[cfg(feature = "opengl")]
extern crate glutin;

use cgmath::prelude::*;
use gfx::traits::{Device, FactoryExt};


pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_vertex_struct!(LineVertex {
    pos: [f32; 4] = "a_Position",
});

gfx_pipeline!(pipe_line {
    vbuf: gfx::VertexBuffer<LineVertex> = (),
    mx_vp: gfx::Global<[[f32; 4]; 4]> = "u_ViewProj",
    color: gfx::Global<[f32; 4]> = "u_Color",
    out_color: gfx::RenderTarget<ColorFormat> = "Target0",
});

const LINE_VS: &'static [u8] = b"
    #version 150 core
    in vec4 a_Position;
    uniform mat4 u_ViewProj;
    void main() {
        gl_Position = u_ViewProj * a_Position;
    }
";
const LINE_FS: &'static [u8] = b"
    #version 150 core
    uniform vec4 u_Color;
    void main() {
        gl_FragColor = u_Color;
    }
";

pub struct Factory(back::Factory);

pub struct Renderer {
    device: back::Device,
    encoder: gfx::Encoder<back::Resources, back::CommandBuffer>,
    out_color: gfx::handle::RenderTargetView<back::Resources, ColorFormat>,
    out_depth: gfx::handle::DepthStencilView<back::Resources, DepthFormat>,
    pso_line: gfx::PipelineState<back::Resources, pipe_line::Meta>,
}

impl Renderer {
    #[cfg(feature = "opengl")]
    pub fn new(builder: glutin::WindowBuilder) -> (glutin::Window, Renderer, Factory) {
        let (window, device, mut factory, color, depth) = gfx_window_glutin::init(builder);
        let renderer = Renderer {
            device: device,
            encoder: factory.create_command_buffer().into(),
            out_color: color,
            out_depth: depth,
            pso_line: {
                let prog = factory.link_program(LINE_VS, LINE_FS).unwrap();
                let rast = gfx::state::Rasterizer::new_fill();
                factory.create_pipeline_from_program(&prog,
                    gfx::Primitive::LineStrip, rast, pipe_line::new()
                ).unwrap()
            },
        };
        (window, renderer, Factory(factory))
    }
}

pub trait Camera {
    fn to_view_proj(&self) -> cgmath::Matrix4<f32>;
}

pub struct PerspectiveCamera {
    pub projection: cgmath::PerspectiveFov<f32>,
    pub position: cgmath::Point3<f32>,
    pub orientation: cgmath::Quaternion<f32>,
}

impl PerspectiveCamera {
    pub fn new(fov: f32, aspect: f32, near: f32, far: f32) -> PerspectiveCamera {
        PerspectiveCamera {
            projection: cgmath::PerspectiveFov {
                fovy: cgmath::Deg(fov).into(),
                aspect: aspect,
                near: near,
                far: far,
            },
            position: cgmath::Point3::origin(),
            orientation: cgmath::Quaternion::one(),
        }
    }

    pub fn look_at(&mut self, target: cgmath::Point3<f32>) {
        let dir = (self.position - target).normalize();
        let z = cgmath::Vector3::unit_z();
        let up = if dir.dot(z).abs() < 0.99 { z } else {
            cgmath::Vector3::unit_y()
        };
        self.orientation = cgmath::Quaternion::look_at(dir, up);
    }
}

impl Camera for PerspectiveCamera {
    fn to_view_proj(&self) -> cgmath::Matrix4<f32> {
        let mx_proj = cgmath::perspective(self.projection.fovy, self.projection.aspect, self.projection.near, self.projection.far);
        let decomposed = cgmath::Decomposed {
            disp: self.position.to_vec(),
            rot: self.orientation,
            scale: 1.0,
        };
        let mx_view = cgmath::Matrix4::from(decomposed.inverse_transform().unwrap());
        mx_proj * mx_view
    }
}

pub struct Geometry {
    pub vertices: Vec<cgmath::Vector3<f32>>,
}

impl Geometry {
    pub fn new() -> Geometry {
        Geometry {
            vertices: Vec::new(),
        }
    }
}

pub type Color = u32;

pub enum Material {
    LineBasic { color: Color },
}

pub struct Object {
    slice: gfx::Slice<back::Resources>,
    vertices: gfx::handle::Buffer<back::Resources, LineVertex>,
    material: Material,
}

impl Factory {
    pub fn line(&mut self, geom: &Geometry, mat: Material) -> Object {
        let vertices: Vec<_> = geom.vertices.iter().map(|v| LineVertex {
            pos: [v.x, v.y, v.z, 1.0],
        }).collect();
        let (vbuf, slice) = self.0.create_vertex_buffer_with_slice(&vertices, ());
        Object {
            slice: slice,
            vertices: vbuf,
            material: mat,
        }
    }
}


pub struct Scene {
    lines: Vec<Object>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            lines: Vec::new(),
        }
    }

    pub fn add(&mut self, object: Object) {
        match object.material {
            Material::LineBasic {..} => {
                self.lines.push(object);
            },
        }
    }
}

impl Renderer {
    pub fn render<C: Camera>(&mut self, scene: &Scene, cam: &C) {
        self.device.cleanup();
        self.encoder.clear(&self.out_color, [0.0, 0.0, 0.0, 1.0]);
        self.encoder.clear_depth(&self.out_depth, 1.0);

        let mx_vp = cam.to_view_proj();
        for line in scene.lines.iter() {
            let color = match line.material {
                Material::LineBasic { color } => {
                    [((color>>16)&0xFF) as f32 / 255.0,
                     ((color>>8) &0xFF) as f32 / 255.0,
                     (color&0xFF) as f32 / 255.0,
                     1.0]
                },
            };
            let data = pipe_line::Data {
                vbuf: line.vertices.clone(),
                mx_vp: mx_vp.into(),
                color: color,
                out_color: self.out_color.clone(),
            };
            self.encoder.draw(&line.slice, &self.pso_line, &data);
        }

        self.encoder.flush(&mut self.device);
    }
}