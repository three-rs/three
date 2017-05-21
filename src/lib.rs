extern crate cgmath;
extern crate froggy;
extern crate genmesh;
#[macro_use]
extern crate gfx;
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
#[cfg(feature = "opengl")]
mod window;

pub use factory::{Factory, Geometry};
pub use render::{ColorFormat, DepthFormat, Renderer};
#[cfg(feature = "opengl")]
pub use window::{Events, Window};
#[cfg(feature = "opengl")]
pub use glutin::VirtualKeyCode as Key;

use cgmath::prelude::*;
use cgmath::Transform as Transform_;
use std::ops;
use std::sync::mpsc;

use factory::SceneId;
use render::GpuData;


pub type Position = cgmath::Point3<f32>;
pub type Normal = cgmath::Vector3<f32>;
pub type Orientation = cgmath::Quaternion<f32>;
pub type Transform = cgmath::Decomposed<Normal, Orientation>;

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


enum Message {
    SetTransform(froggy::WeakPointer<Node>, Transform),
    SetMaterial(froggy::WeakPointer<Visual>, Material),
    //Delete,
}

type NodePtr = froggy::Pointer<Node>;
type VisualPtr = froggy::Pointer<Visual>;

pub type Color = u32;

#[derive(Clone)]
pub enum Material {
    LineBasic { color: Color },
    MeshBasic { color: Color },
}

struct SceneLink<V> {
    id: SceneId,
    node: NodePtr,
    visual: V,
    tx: mpsc::Sender<Message>,
}

pub struct Object {
    transform: Transform,
    scenes: Vec<SceneLink<()>>,
}

pub struct VisualObject {
    _visible: bool,
    transform: Transform,
    material: Material,
    gpu_data: GpuData,
    scenes: Vec<SceneLink<VisualPtr>>,
}

macro_rules! def_proxy {
    ($name:ident<$visual:ty, $target:ty> = $message:ident($key:ident)) => {
        pub struct $name<'a> {
            value: &'a mut $target,
            links: &'a [SceneLink<$visual>],
        }

        impl<'a> ops::Deref for $name<'a> {
            type Target = $target;
            fn deref(&self) -> &Self::Target {
                self.value
            }
        }

        impl<'a> ops::DerefMut for $name<'a> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.value
            }
        }

        impl<'a> Drop for $name<'a> {
            fn drop(&mut self) {
                for link in self.links {
                    let msg = Message::$message(link.$key.downgrade(), self.value.clone());
                    let _ = link.tx.send(msg);
                }
            }
        }
    }
}

def_proxy!(TransformProxy<(), Transform> = SetTransform(node));
def_proxy!(TransformProxyVisual<VisualPtr, Transform> = SetTransform(node));
def_proxy!(MaterialProxy<VisualPtr, Material> = SetMaterial(visual));

impl Object {
    fn new() -> Self {
        Object {
            transform: Transform::one(),
            scenes: Vec::with_capacity(1),
        }
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> TransformProxy {
        TransformProxy {
            value: &mut self.transform,
            links: &self.scenes,
        }
    }

    pub fn attach(&mut self, scene: &mut Scene, group: Option<&Group>) {
        assert!(!self.scenes.iter().any(|link| link.id == scene.unique_id),
            "Object is already in the scene");
        let node_ptr = scene.make_node(self.transform.clone(), group);
        self.scenes.push(SceneLink {
            id: scene.unique_id,
            node: node_ptr,
            visual: (),
            tx: scene.message_tx.clone(),
        });
    }
}

impl VisualObject {
    fn new(material: Material, gpu_data: GpuData) -> Self {
        VisualObject {
            _visible: true,
            transform: Transform::one(),
            material: material,
            gpu_data: gpu_data,
            scenes: Vec::with_capacity(1),
        }
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> TransformProxyVisual {
        TransformProxyVisual {
            value: &mut self.transform,
            links: &self.scenes,
        }
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    pub fn material_mut(&mut self) -> MaterialProxy {
        MaterialProxy {
            value: &mut self.material,
            links: &self.scenes,
        }
    }

    pub fn attach(&mut self, scene: &mut Scene, group: Option<&Group>) {
        assert!(!self.scenes.iter().any(|link| link.id == scene.unique_id),
            "VisualObject is already in the scene");
        let node_ptr = scene.make_node(self.transform.clone(), group);
        let visual_ptr = scene.visuals.create(Visual {
            material: self.material.clone(),
            gpu_data: self.gpu_data.clone(),
            node: node_ptr.clone(),
        });
        self.scenes.push(SceneLink {
            id: scene.unique_id,
            node: node_ptr,
            visual: visual_ptr,
            tx: scene.message_tx.clone(),
        });
    }
}


pub struct Group {
    object: Object,
}

pub struct Mesh {
    object: VisualObject,
    _geometry: Option<Geometry>,
}

macro_rules! deref {
    ($name:ty = $object:ty) => {
        impl ops::Deref for $name {
            type Target = $object;
            fn deref(&self) -> &Self::Target {
                &self.object
            }
        }

        impl ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.object
            }
        }
    }
}

deref!(Group = Object);
deref!(Mesh = VisualObject);


struct Node {
    local: Transform,
    world: Transform,
    parent: Option<NodePtr>,
}

struct Visual {
    material: Material,
    gpu_data: GpuData,
    node: NodePtr,
}

pub struct Scene {
    nodes: froggy::Storage<Node>,
    visuals: froggy::Storage<Visual>,
    unique_id: SceneId,
    message_tx: mpsc::Sender<Message>,
    message_rx: mpsc::Receiver<Message>,
}

impl Scene {
    fn make_node(&mut self, transform: Transform, group: Option<&Group>) -> NodePtr {
        let parent = group.map(|g| {
            g.scenes.iter().find(|link| link.id == self.unique_id)
             .expect("Parent group is not in the scene")
             .node.clone()
        });
        self.nodes.create(Node {
            local: transform,
            world: Transform::one(),
            parent: parent,
        })
    }

    pub fn process_messages(&mut self) {
        while let Ok(message) = self.message_rx.try_recv() {
            match message {
                Message::SetTransform(pnode, transform) => {
                    if let Ok(ref ptr) = pnode.upgrade() {
                        self.nodes[ptr].local = transform;
                    }
                }
                Message::SetMaterial(pvisual, material) => {
                    if let Ok(ref ptr) = pvisual.upgrade() {
                        self.visuals[ptr].material = material;
                    }
                }
            }
        }
    }

    pub fn compute_transforms(&mut self) {
        let mut cursor = self.nodes.cursor();
        while let Some(mut item) = cursor.next() {
            item.world = match item.parent {
                Some(ref parent) => item.look_back(parent).unwrap().world.concat(&item.local),
                None => item.local,
            };
        }
    }

    pub fn update(&mut self) {
        self.process_messages();
        self.compute_transforms();
    }
}
