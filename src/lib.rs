extern crate cgmath;
extern crate froggy;
#[macro_use]
extern crate gfx;
extern crate winit;
// OpenGL
#[cfg(feature = "opengl")]
extern crate gfx_device_gl as back;
#[cfg(feature = "opengl")]
extern crate gfx_window_glutin;
#[cfg(feature = "opengl")]
extern crate glutin;

use cgmath::prelude::*;
use cgmath::Transform as Transform_;
use gfx::traits::{Device, FactoryExt};
use std::sync::mpsc;


pub type Position = cgmath::Point3<f32>;
pub type Normal = cgmath::Vector3<f32>;
pub type Orientation = cgmath::Quaternion<f32>;
pub type Transform = cgmath::Decomposed<Normal, Orientation>;
type ObjectId = usize;
pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_vertex_struct!(Vertex {
    pos: [f32; 4] = "a_Position",
});

gfx_pipeline!(pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
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

type NodePtr = froggy::Pointer<Node>;
struct Node {
    transform: Transform,
    children: Vec<NodePtr>,
}

impl Node {
    fn new() -> Self {
        Node {
            transform: Transform::one(),
            children: Vec::new(),
        }
    }
}

pub struct Factory {
    graphics: back::Factory,
    node_store: froggy::Storage<Node>,
    object_id: ObjectId,
}

pub struct Renderer {
    device: back::Device,
    encoder: gfx::Encoder<back::Resources, back::CommandBuffer>,
    out_color: gfx::handle::RenderTargetView<back::Resources, ColorFormat>,
    out_depth: gfx::handle::DepthStencilView<back::Resources, DepthFormat>,
    pso_line: gfx::PipelineState<back::Resources, pipe::Meta>,
    size: (u32, u32),
    #[cfg(feature = "opengl")]
    window: glutin::Window,
}

#[cfg(feature = "opengl")]
impl Renderer {
    pub fn new(builder: glutin::WindowBuilder, event_loop: &glutin::EventsLoop)
               -> (Renderer, Factory) {
        let (window, device, mut gl_factory, color, depth) =
            gfx_window_glutin::init(builder, event_loop);
        let renderer = Renderer {
            device: device,
            encoder: gl_factory.create_command_buffer().into(),
            out_color: color,
            out_depth: depth,
            pso_line: {
                let prog = gl_factory.link_program(LINE_VS, LINE_FS).unwrap();
                let rast = gfx::state::Rasterizer::new_fill();
                gl_factory.create_pipeline_from_program(&prog,
                    gfx::Primitive::LineStrip, rast, pipe::new()
                ).unwrap()
            },
            size: window.get_inner_size_pixels().unwrap(),
            window: window,
        };
        let factory = Factory {
            graphics: gl_factory,
            node_store: froggy::Storage::new(),
            object_id: 0,
        };
        (renderer, factory)
    }

    pub fn resize(&mut self) {
        self.size = self.window.get_inner_size_pixels().unwrap();
        gfx_window_glutin::update_views(&self.window, &mut self.out_color, &mut self.out_depth);
    }
}

pub trait Camera {
    fn to_view_proj(&self) -> cgmath::Matrix4<f32>;
}

pub struct PerspectiveCamera {
    pub projection: cgmath::PerspectiveFov<f32>,
    pub position: Position,
    pub orientation: Orientation,
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
            position: Position::origin(),
            orientation: Orientation::one(),
        }
    }

    pub fn look_at(&mut self, target: cgmath::Point3<f32>) {
        let dir = (self.position - target).normalize();
        let z = cgmath::Vector3::unit_z();
        let up = if dir.dot(z).abs() < 0.99 { z } else {
            cgmath::Vector3::unit_y()
        };
        self.orientation = Orientation::look_at(dir, up);
    }
}

impl Camera for PerspectiveCamera {
    fn to_view_proj(&self) -> cgmath::Matrix4<f32> {
        let mx_proj = cgmath::perspective(self.projection.fovy,
            self.projection.aspect, self.projection.near, self.projection.far);
        let transform = cgmath::Decomposed {
            disp: self.position.to_vec(),
            rot: self.orientation,
            scale: 1.0,
        };

        let mx_view = cgmath::Matrix4::from(transform.inverse_transform().unwrap());
        mx_proj * mx_view
    }
}

#[derive(Clone)]
pub struct Geometry {
    pub vertices: Vec<Position>,
    pub normals: Vec<Normal>,
    pub faces: Vec<[u16; 3]>,
    pub is_dynamic: bool,
}

impl Geometry {
    pub fn new() -> Geometry {
        Geometry {
            vertices: Vec::new(),
            normals: Vec::new(),
            faces: Vec::new(),
            is_dynamic: false,
        }
    }
    pub fn from_vertices(verts: Vec<Position>) -> Geometry {
        Geometry {
            vertices: verts,
            .. Geometry::new()
        }
    }
}

/*
enum Message {
    UpdateTransform(Transform),
    Delete,
}

type ObjectMessage = (ObjectId, Message);
*/

pub type Color = u32;

#[derive(Clone)]
pub enum Material {
    LineBasic { color: Color },
}

#[derive(Clone)]
struct GpuData {
    slice: gfx::Slice<back::Resources>,
    vertices: gfx::handle::Buffer<back::Resources, Vertex>,
}

#[derive(Clone)]
pub struct Object {
    pub geometry: Option<Geometry>,
    pub material: Material,
    transform: Transform,
    gpu_data: GpuData,
}

impl Drop for Object {
    fn drop(&mut self) {
 //       for tx in &self.message_tx {
 //           let _ = tx.send((self.id, Message::Delete));
 //       }
    }
}

pub struct Scene {
    root_node: Node,
    lines: Vec<Object>,
//    message_tx: mpsc::Sender<ObjectMessage>,
//    message_rx: mpsc::Receiver<ObjectMessage>,
//    nodes: froggy::Storage<SceneNode>,
}

impl Scene {
    pub fn new() -> Self {
//        let (tx, rx) = mpsc::channel();
        Scene {
            root_node: Node::new(),
            lines: Vec::new(),
//            message_tx: tx,
//            message_rx: rx,
        }
    }

    pub fn add(&mut self, object: &Object) {
//        object.message_tx.push(self.message_tx.clone());
        match object.material {
            Material::LineBasic {..} => {
                self.lines.push(object.clone());
            },
        }
    }

    fn update(&mut self) {
//        while let Ok(message) = self.message_rx.try_recv() {
//            match message {
//                Message::UpdateTransform(_, _) => (),
//                Message::Delete(_) => (),
//            }
//        }
    }
}


impl Factory {
    pub fn line(&mut self, geom: Geometry, mat: Material) -> Object {
        self.object_id += 1;
        let vertices: Vec<_> = geom.vertices.iter().map(|v| Vertex {
            pos: [v.x, v.y, v.z, 1.0],
        }).collect();
        //TODO: dynamic geometry
        let (vbuf, slice) = self.graphics.create_vertex_buffer_with_slice(&vertices, ());
        Object {
            geometry: if geom.is_dynamic { Some(geom) } else { None },
            material: mat,
            transform: Transform::one(),
            gpu_data: GpuData {
                slice: slice,
                vertices: vbuf,
            },
//            node: self.node_store.create(Node::new()),
//            id: self.object_id,
//            message_tx: Vec::with_capacity(1),
        }
    }

    // pub fn update(&self, ) //TODO: update dynamic geometry
}


impl Renderer {
    pub fn get_aspect(&self) -> f32 {
        self.size.0 as f32 / self.size.1 as f32
    }

    pub fn render<C: Camera>(&mut self, scene: &Scene, cam: &C) {
        //scene.update();

        self.device.cleanup();
        self.encoder.clear(&self.out_color, [0.0, 0.0, 0.0, 1.0]);
        self.encoder.clear_depth(&self.out_depth, 1.0);

        let mx_vp = cam.to_view_proj();
        for line in &scene.lines {
            let color = match line.material {
                Material::LineBasic { color } => {
                    [((color>>16)&0xFF) as f32 / 255.0,
                     ((color>>8) &0xFF) as f32 / 255.0,
                     (color&0xFF) as f32 / 255.0,
                     1.0]
                },
            };
            let data = pipe::Data {
                vbuf: line.gpu_data.vertices.clone(),
                mx_vp: mx_vp.into(),
                color: color,
                out_color: self.out_color.clone(),
            };
            self.encoder.draw(&line.gpu_data.slice, &self.pso_line, &data);
        }

        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().unwrap();
    }
}
