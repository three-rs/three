use std::ops;
use std::sync::mpsc;

use cgmath::prelude::*;
use cgmath::{self, Transform as Transform_};

use froggy::{Pointer};

use {Position, Orientation, Object, VisualObject, Message,
     Node, Visual, Scene, Transform};
use factory::{Geometry, SceneId, Texture};


macro_rules! deref {
    ($name:ty : $field:ident = $object:ty) => {
        impl ops::Deref for $name {
            type Target = $object;
            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }

        impl ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    }
}


pub trait Camera {
    //TODO: combine into a single method?
    fn to_view_proj(&self) -> cgmath::Matrix4<f32>;
    fn set_aspect(&mut self, f32);
}

pub struct PerspectiveCamera {
    projection: cgmath::PerspectiveFov<f32>,
    pub position: Position,
    pub orientation: Orientation,
}

impl PerspectiveCamera {
    pub fn new(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
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

    fn set_aspect(&mut self, aspect: f32) {
        self.projection.aspect = aspect;
    }
}

pub struct OrthographicCamera {
    projection: cgmath::Ortho<f32>,
    base_aspect: f32,
    pub position: Position,
    pub orientation: Orientation,
}

impl OrthographicCamera {
    pub fn new(left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32) -> Self {
        OrthographicCamera {
            projection: cgmath::Ortho{ left, right, bottom, top, near, far },
            base_aspect: (right - left) / (top - bottom),
            position: Position::origin(),
            orientation: Orientation::one(),
        }
    }
}

impl Camera for OrthographicCamera {
    fn to_view_proj(&self) -> cgmath::Matrix4<f32> {
        let mx_proj = cgmath::ortho(self.projection.left, self.projection.right,
            self.projection.bottom, self.projection.top,
            self.projection.near, self.projection.far);
        let transform = cgmath::Decomposed {
            disp: self.position.to_vec(),
            rot: self.orientation,
            scale: 1.0,
        };

        let mx_view = cgmath::Matrix4::from(transform.inverse_transform().unwrap());
        mx_proj * mx_view
    }

    fn set_aspect(&mut self, aspect: f32) {
        let center = 0.5 * (self.projection.left + self.projection.right);
        let scale = aspect / self.base_aspect;
        self.projection.left = scale * (self.projection.left - center) + center;
        self.projection.right = scale * (self.projection.right - center) + center;
        self.base_aspect = aspect;
    }
}

deref!(PerspectiveCamera : projection = cgmath::PerspectiveFov<f32>);
deref!(OrthographicCamera : projection = cgmath::Ortho<f32>);


pub type Color = u32;

#[derive(Clone)]
pub enum Material {
    LineBasic { color: Color },
    MeshBasic { color: Color },
    Sprite { map: Texture },
}

pub struct SceneLink<V> {
    id: SceneId,
    node: Pointer<Node>,
    visual: V,
    tx: mpsc::Sender<Message>,
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
def_proxy!(TransformProxyVisual<Pointer<Visual>, Transform> = SetTransform(node));
def_proxy!(MaterialProxy<Pointer<Visual>, Material> = SetMaterial(visual));

impl Object {
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

impl Group {
    #[doc(hidden)]
    pub fn new() -> Self {
        Group {
            object: Object::new(),
        }
    }
}

pub struct Mesh {
    object: VisualObject,
    _geometry: Option<Geometry>,
}

impl Mesh {
    #[doc(hidden)]
    pub fn new(object: VisualObject) -> Self {
        Mesh {
            object: object,
            _geometry: None,
        }
    }
}

pub struct Sprite {
    object: VisualObject,
}

impl Sprite {
    #[doc(hidden)]
    pub fn new(object: VisualObject) -> Self {
        Sprite {
            object: object,
        }
    }
}

deref!(Group : object = Object);
deref!(Mesh : object = VisualObject);
deref!(Sprite : object = VisualObject);


impl Scene {
    fn make_node(&mut self, transform: Transform, group: Option<&Group>)
                 -> Pointer<Node> {
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
