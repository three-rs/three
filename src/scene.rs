use std::ops;
use std::sync::mpsc;

use froggy::Pointer;

use {Object, VisualObject, LightObject, Message, Operation,
     Node, SubNode, Scene, Transform, ShadowConfig};
use camera::OrthographicCamera;
use factory::{Geometry, Texture};


pub type Color = u32;

#[derive(Clone)]
pub enum Material {
    LineBasic { color: Color },
    MeshBasic { color: Color, wireframe: bool },
    MeshLambert { color: Color },
    Sprite { map: Texture },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Shadow<C> {
    pub camera: C,
    pub resolution: [u16; 2],
}

macro_rules! def_proxy {
    ($name:ident<$target:ty> = $message:ident($key:ident)) => {
        pub struct $name<'a> {
            value: &'a mut $target,
            node: &'a Pointer<Node>,
            tx: &'a mpsc::Sender<Message>,
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
                let msg = Operation::$message(self.value.clone());
                let _ = self.tx.send((self.node.downgrade(), msg));
            }
        }
    }
}

def_proxy!(TransformProxy<Transform> = SetTransform(node));
def_proxy!(MaterialProxy<Material> = SetMaterial(visual));

impl<'a> TransformProxy<'a> {
    pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
        use cgmath::{Euler, Quaternion, Rad};
        let rot = Euler::new(Rad(x), Rad(y), Rad(z));
        self.value.rot = Quaternion::from(rot) * self.value.rot;
    }
}

impl Object {
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        let msg = Operation::SetVisible(visible);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> TransformProxy {
        TransformProxy {
            value: &mut self.transform,
            node: &self.node,
            tx: &self.tx,
        }
    }

    pub fn sync(&mut self, scene: &Scene) {
        let mut hub = scene.hub.lock().unwrap();
        hub.process_messages();
        self.transform = hub.nodes[&self.node].transform;
    }
}

impl VisualObject {
    pub fn material(&self) -> &Material {
        &self.data.material
    }

    pub fn material_mut(&mut self) -> MaterialProxy {
        MaterialProxy {
            value: &mut self.data.material,
            node: &self.inner.node,
            tx: &self.inner.tx,
        }
    }

    pub fn sync(&mut self, scene: &Scene) {
        let mut hub = scene.hub.lock().unwrap();
        hub.process_messages();
        let node = &hub.nodes[&self.node];
        self.inner.transform = node.transform;
        if let SubNode::Visual(ref data) = node.sub_node {
            self.data = data.drop_payload();
        }
    }
}


pub struct Group {
    object: Object,
}

impl Group {
    #[doc(hidden)]
    pub fn new(object: Object) -> Self {
        Group {
            object,
        }
    }

    pub fn add<P: AsRef<Pointer<Node>>>(&mut self, child: &P) {
        let msg = Operation::SetParent(self.object.node.clone());
        let _ = self.object.tx.send((child.as_ref().downgrade(), msg));
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
            object,
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
            object,
        }
    }
}


pub struct AmbientLight {
    object: LightObject,
}

impl AmbientLight {
    #[doc(hidden)]
    pub fn new(object: LightObject) -> Self {
        AmbientLight {
            object,
        }
    }
}

pub struct DirectionalLight {
    object: LightObject,
    has_shadow: bool,
}

impl DirectionalLight {
    #[doc(hidden)]
    pub fn new(object: LightObject) -> Self {
        DirectionalLight {
            object,
            has_shadow: false,
        }
    }

    pub fn has_shadow(&self) -> bool {
        self.has_shadow
    }

    pub fn set_shadow(&mut self, shadow: Shadow<OrthographicCamera>) {
        self.has_shadow = true;
        let msg = Operation::SetShadow(ShadowConfig::Ortho(shadow));
        let _ = self.tx.send((self.node.downgrade(), msg));
    }
}

pub struct HemisphereLight {
    object: LightObject,
}

impl HemisphereLight {
    #[doc(hidden)]
    pub fn new(object: LightObject) -> Self {
        HemisphereLight {
            object,
        }
    }
}

pub struct PointLight {
    object: LightObject,
}

impl PointLight {
    #[doc(hidden)]
    pub fn new(object: LightObject) -> Self {
        PointLight {
            object,
        }
    }
}


impl Scene {
    pub fn add<P: AsRef<Pointer<Node>>>(&mut self, child: &P) {
        let msg = Operation::SetParent(self.node.clone());
        let _ = self.tx.send((child.as_ref().downgrade(), msg));
    }
}

macro_rules! as_node {
    ($($name:ident),*) => {
        $(
            impl AsRef<Pointer<Node>> for $name {
                fn as_ref(&self) -> &Pointer<Node> {
                    &self.object.node
                }
            }
        )*
    }
}

as_node!(Group, Mesh, Sprite, AmbientLight, DirectionalLight, HemisphereLight, PointLight);

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

deref!(VisualObject : inner = Object);
deref!(LightObject : inner = Object);
deref!(Group : object = Object);
deref!(Mesh : object = VisualObject);
deref!(Sprite : object = VisualObject);
deref!(AmbientLight : object = LightObject);
deref!(HemisphereLight : object = LightObject);
deref!(DirectionalLight : object = LightObject);
deref!(PointLight : object = LightObject);
