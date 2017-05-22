extern crate cgmath;
extern crate froggy;
extern crate genmesh;
#[macro_use]
extern crate gfx;
extern crate image;
extern crate winit;
// OpenGL
#[cfg(feature = "opengl")]
extern crate gfx_device_gl;
#[cfg(feature = "opengl")]
extern crate gfx_window_glutin;
#[cfg(feature = "opengl")]
extern crate glutin;

mod factory;
mod render;
mod scene;
#[cfg(feature = "opengl")]
mod window;

pub use factory::{Factory, Geometry, Texture};
pub use render::{ColorFormat, DepthFormat, Renderer};
pub use scene::{Camera, OrthographicCamera, PerspectiveCamera,
                Color, Material, Group, Mesh, Sprite};
#[cfg(feature = "opengl")]
pub use window::{Events, Window};
#[cfg(feature = "opengl")]
pub use glutin::VirtualKeyCode as Key;

use std::sync::mpsc;

use factory::SceneId;
use render::GpuData;
use scene::SceneLink;


pub type Position = cgmath::Point3<f32>;
pub type Normal = cgmath::Vector3<f32>;
pub type Orientation = cgmath::Quaternion<f32>;
pub type Transform = cgmath::Decomposed<Normal, Orientation>;

struct Node {
    local: Transform,
    world: Transform,
    parent: Option<froggy::Pointer<Node>>,
}

struct Visual {
    material: Material,
    gpu_data: GpuData,
    node: froggy::Pointer<Node>,
}

pub struct Object {
    transform: Transform,
    scenes: Vec<SceneLink<()>>,
}

impl Object {
    fn new() -> Self {
        Object {
            transform: cgmath::Transform::one(),
            scenes: Vec::with_capacity(1),
        }
    }
}

pub struct VisualObject {
    _visible: bool,
    transform: Transform,
    material: Material,
    gpu_data: GpuData,
    scenes: Vec<SceneLink<froggy::Pointer<Visual>>>,
}

impl VisualObject {
    fn new(material: Material, gpu_data: GpuData) -> Self {
        VisualObject {
            _visible: true,
            transform: cgmath::Transform::one(),
            material: material,
            gpu_data: gpu_data,
            scenes: Vec::with_capacity(1),
        }
    }
}

enum Message {
    SetTransform(froggy::WeakPointer<Node>, Transform),
    SetMaterial(froggy::WeakPointer<Visual>, Material),
    //Delete,
}

pub struct Scene {
    nodes: froggy::Storage<Node>,
    visuals: froggy::Storage<Visual>,
    unique_id: SceneId,
    message_tx: mpsc::Sender<Message>,
    message_rx: mpsc::Receiver<Message>,
}
