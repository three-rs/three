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

mod camera;
mod factory;
mod render;
mod scene;
#[cfg(feature = "opengl")]
mod window;

pub use camera::{Camera, OrthographicCamera, PerspectiveCamera};
pub use factory::{Factory, Geometry, Texture};
pub use render::{ColorFormat, DepthFormat, Renderer};
pub use scene::{Color, Material, Group, Mesh, Sprite};
#[cfg(feature = "opengl")]
pub use window::{Events, Window};
#[cfg(feature = "opengl")]
pub use glutin::VirtualKeyCode as Key;

use std::sync::{mpsc, Arc, Mutex};

use factory::SceneId;
use render::GpuData;


pub type Position = cgmath::Point3<f32>;
pub type Normal = cgmath::Vector3<f32>;
pub type Orientation = cgmath::Quaternion<f32>;
pub type Transform = cgmath::Decomposed<Normal, Orientation>;


struct Visual {
    material: Material,
    gpu_data: GpuData,
}

/// Fat node of the scene graph.
pub struct Node {
    visible: bool,
    local: Transform,
    world: Transform,
    parent: Option<froggy::Pointer<Node>>,
    scene: Option<SceneId>,
    visual: Option<Visual>,
}

impl Node {
    fn new() -> Self {
        Node {
            visible: true,
            local: cgmath::Transform::one(),
            world: cgmath::Transform::one(),
            parent: None,
            scene: None,
            visual: None,
        }
    }
}

pub struct Object {
    visible: bool,
    transform: Transform,
    node: froggy::Pointer<Node>,
    tx: mpsc::Sender<Message>,
}

pub struct VisualObject {
    inner: Object,
    visual: Visual,
}

type Message = (froggy::WeakPointer<Node>, Operation);
enum Operation {
    SetParent(froggy::Pointer<Node>),
    SetTransform(Transform),
    SetMaterial(Material),
    //Delete,
}

struct VisualIter<'a> {
    _dummy: &'a (),
}

impl<'a> Iterator for VisualIter<'a> {
    type Item = (&'a Visual, Transform);
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

type HubPtr = Arc<Mutex<Hub>>;
struct Hub {
    nodes: froggy::Storage<Node>,
    message_tx: mpsc::Sender<Message>,
    message_rx: mpsc::Receiver<Message>,
}

impl Hub {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Hub {
            nodes: froggy::Storage::new(),
            message_tx: tx,
            message_rx: rx,
        }
    }

    fn spawn(&mut self) -> Object {
        Object {
            visible: true,
            transform: cgmath::Transform::one(),
            node: self.nodes.create(Node::new()),
            tx: self.message_tx.clone(),
        }
    }

    fn into_ptr(self) -> HubPtr {
        Arc::new(Mutex::new(self))
    }

    fn visualize(&mut self, _scene_id: SceneId) -> VisualIter {
        unimplemented!()
    }
}


pub struct Scene {
    unique_id: SceneId,
    node: froggy::Pointer<Node>,
    hub: HubPtr,
}

impl AsRef<froggy::Pointer<Node>> for Scene {
    fn as_ref(&self) -> &froggy::Pointer<Node> {
        &self.node
    }
}
