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

use cgmath::Transform as Transform_;
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
    world_visible: bool,
    transform: Transform,
    world_transform: Transform,
    parent: Option<froggy::Pointer<Node>>,
    scene_id: Option<SceneId>,
    visual: Option<Visual>,
}

impl Node {
    fn new() -> Self {
        Node {
            visible: true,
            world_visible: false,
            transform: cgmath::Transform::one(),
            world_transform: cgmath::Transform::one(),
            parent: None,
            scene_id: None,
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
    SetVisible(bool),
    SetTransform(Transform),
    SetMaterial(Material),
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

    fn process_messages(&mut self) {
        while let Ok((pnode, operation)) = self.message_rx.try_recv() {
            let node = match pnode.upgrade() {
                Ok(ptr) => &mut self.nodes[&ptr],
                Err(_) => continue,
            };
            match operation {
                Operation::SetParent(parent) => {
                    node.parent = Some(parent);
                }
                Operation::SetVisible(visible) => {
                    node.visible = visible;
                }
                Operation::SetTransform(transform) => {
                    node.transform = transform;
                }
                Operation::SetMaterial(material) => {
                    node.visual.as_mut().unwrap().material = material;
                }
            }
        }
        self.nodes.sync_pending();
    }

    fn visualize<F>(&mut self, scene_id: SceneId, mut fun: F)
        where F: FnMut(&Visual, &Transform)
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
                if let Some(ref visual) = item.visual {
                    fun(visual, &item.world_transform);
                }
            }
        }
    }
}


pub struct Scene {
    unique_id: SceneId,
    node: froggy::Pointer<Node>,
    tx: mpsc::Sender<Message>,
    hub: HubPtr,
}
