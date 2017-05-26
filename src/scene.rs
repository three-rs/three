use std::ops;
use std::sync::mpsc;

use froggy::Pointer;

use {Object, VisualObject, Message, Operation,
     Node, Scene, Transform};
use factory::{Geometry, Texture};


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

pub type Color = u32;

#[derive(Clone)]
pub enum Material {
    LineBasic { color: Color },
    MeshBasic { color: Color },
    Sprite { map: Texture },
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

impl Object {
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

    pub fn attach<P: AsRef<Pointer<Node>>>(&mut self, parent: &P) {
        let msg = Operation::SetParent(parent.as_ref().clone());
        let _ = self.tx.send((self.node.downgrade(), msg));
    }
}

impl VisualObject {
    pub fn material(&self) -> &Material {
        &self.visual.material
    }

    pub fn material_mut(&mut self) -> MaterialProxy {
        MaterialProxy {
            value: &mut self.visual.material,
            node: &self.inner.node,
            tx: &self.inner.tx,
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
}

impl AsRef<Pointer<Node>> for Group {
    fn as_ref(&self) -> &Pointer<Node> {
        &self.object.node
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

deref!(VisualObject : inner = Object);
deref!(Group : object = Object);
deref!(Mesh : object = VisualObject);
deref!(Sprite : object = VisualObject);


impl Scene {
    /*fn make_node(&mut self, transform: Transform, group: Option<&Group>)
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
    }*/
}
