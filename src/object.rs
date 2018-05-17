//! Items in the scene heirarchy.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::mpsc;

use mint;

use audio;
use hub::{Hub, Message, Operation, SubLight, SubNode};
use light;
use mesh::Mesh;
use node::NodePointer;
use scene::SyncGuard;
use skeleton::{Bone, Skeleton};
use sprite::Sprite;
use text::Text;

//Note: no local state should be here, only remote links
/// `Base` represents a concrete entity that can be added to the scene.
///
/// One cannot construct `Base` directly. Wrapper types such as [`Camera`],
/// [`Mesh`], and [`Group`] are provided instead.
///
/// Any type that implements [`Object`] may be converted to its concrete
/// `Base` type with the method [`Object::upcast`]. This is useful for
/// storing generic objects in containers.
///
/// [`Camera`]: ../camera/struct.Camera.html
/// [`Mesh`]: ../mesh/struct.Mesh.html
/// [`Group`]: ../object/struct.Group.html
/// [`Object`]: ../object/trait.Object.html
/// [`Object::upcast`]: ../object/trait.Object.html#method.upcast
#[derive(Clone)]
pub struct Base {
    pub(crate) node: NodePointer,
    pub(crate) tx: mpsc::Sender<Message>,
}

/// Marks data structures that are able to added to the scene graph.
pub trait Object: AsRef<Base> {
    /// The internal data for the object.
    ///
    /// Three-rs objects normally expose a write-only interface, making it possible to change
    /// an object's internal values but not possible to read those values.
    /// [`SyncGuard::resolve_data`] allows for that data to be read in a controlled way, with.
    /// the data for the specific object type determined by the `Data` trait member.
    ///
    /// Each object type has its own internal data, and not all object types can provide access
    /// to meaningful data. Types that cannot provide user-facing data will specify `()`
    /// for `Data`.
    type Data;

    /// Retrieves the internal data for the object.
    ///
    /// Prefer to use [`SyncGuard::resolve_data`] instead.
    fn resolve_data(&self, sync_guard: &mut SyncGuard) -> Self::Data;

    /// Converts into the base type.
    fn upcast(&self) -> Base {
        self.as_ref().clone()
    }

    /// Invisible objects are not rendered by cameras.
    fn set_visible(
        &self,
        visible: bool,
    ) {
        self.as_ref().send(Operation::SetVisible(visible));
    }

    /// Sets the name of the object.
    fn set_name<S: Into<String>>(
        &self,
        name: S,
    ) {
        self.as_ref().send(Operation::SetName(name.into()));
    }

    /// Set both position, orientation and scale.
    fn set_transform<P, Q>(
        &self,
        pos: P,
        rot: Q,
        scale: f32,
    ) where
        Self: Sized,
        P: Into<mint::Point3<f32>>,
        Q: Into<mint::Quaternion<f32>>,
    {
        self.as_ref().send(Operation::SetTransform(Some(pos.into()), Some(rot.into()), Some(scale)));
    }

    /// Set position.
    fn set_position<P>(
        &self,
        pos: P,
    ) where
        Self: Sized,
        P: Into<mint::Point3<f32>>,
    {
        self.as_ref().send(Operation::SetTransform(Some(pos.into()), None, None));
    }

    /// Set orientation.
    fn set_orientation<Q>(
        &self,
        rot: Q,
    ) where
        Self: Sized,
        Q: Into<mint::Quaternion<f32>>,
    {
        self.as_ref().send(Operation::SetTransform(None, Some(rot.into()), None));
    }

    /// Set scale.
    fn set_scale(
        &self,
        scale: f32,
    ) {
        self.as_ref().send(Operation::SetTransform(None, None, Some(scale)));
    }

    /// Set weights.
    //Note: needed for animations
    fn set_weights(
        &self,
        weights: Vec<f32>,
    ) {
        self.as_ref().send(Operation::SetWeights(weights));
    }

    /// Rotates object in the specific direction of `target`.
    fn look_at<E, T>(
        &self,
        eye: E,
        target: T,
        up: Option<mint::Vector3<f32>>,
    ) where
        Self: Sized,
        E: Into<mint::Point3<f32>>,
        T: Into<mint::Point3<f32>>,
    {
        use cgmath::{InnerSpace, Point3, Quaternion, Rotation, Vector3};
        let p: [mint::Point3<f32>; 2] = [eye.into(), target.into()];
        let dir = (Point3::from(p[0]) - Point3::from(p[1])).normalize();
        let z = Vector3::unit_z();
        let up = match up {
            Some(v) => Vector3::from(v).normalize(),
            None if dir.dot(z).abs() < 0.99 => z,
            None => Vector3::unit_y(),
        };
        let q = Quaternion::look_at(dir, up).invert();
        self.set_transform(p[0], q, 1.0);
    }
}

impl PartialEq for Base {
    fn eq(
        &self,
        other: &Base,
    ) -> bool {
        self.node == other.node
    }
}

impl Eq for Base {}

impl Hash for Base {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.node.hash(state);
    }
}

impl fmt::Debug for Base {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        self.node.fmt(f)
    }
}

impl Base {
    pub(crate) fn send(
        &self,
        operation: Operation,
    ) {
        let _ = self.tx.send((self.node.downgrade(), operation));
    }
}

// Required for `Base` to implement `trait Object`.
impl AsRef<Base> for Base {
    fn as_ref(&self) -> &Base {
        self
    }
}

impl Object for Base {
    type Data = ObjectType;

    fn resolve_data(&self, sync_guard: &mut SyncGuard) -> Self::Data {
        let node = &sync_guard.hub[self];
        match &node.sub_node {
            // TODO: Handle resolving cameras better (`Empty` is only used for cameras).
            SubNode::Empty => unimplemented!("Cameras need to be changed my dude"),

            SubNode::Group { .. } => ObjectType::Group(Group {
                object: self.clone(),
            }),

            SubNode::Audio(..) => ObjectType::AudioSource(audio::Source {
                object: self.clone(),
            }),

            SubNode::UiText(..) => ObjectType::Text(Text {
                object: self.clone(),
            }),

            // TODO: Differentiate between `Mesh` and `DynamicMesh`.
            SubNode::Visual(..) => ObjectType::Mesh(Mesh {
                object: self.clone(),
            }),

            SubNode::Bone { .. } => ObjectType::Bone(Bone {
                object: self.clone(),
            }),

            SubNode::Skeleton(..) => ObjectType::Skeleton(Skeleton {
                object: self.clone(),
            }),

            SubNode::Light(light) => match light.sub_light {
                SubLight::Ambient => ObjectType::AmbientLight(light::Ambient {
                    object: self.clone(),
                }),

                SubLight::Directional => ObjectType::DirectionalLight(light::Directional {
                    object: self.clone(),
                    shadow: light.shadow.as_ref().map(|&(ref map, _)| map.clone()),
                }),

                SubLight::Point => ObjectType::PointLight(light::Point {
                    object: self.clone(),
                }),

                SubLight::Hemisphere { .. } => ObjectType::HemisphereLight(light::Hemisphere {
                    object: self.clone(),
                }),
            },
        }
    }
}

/// The possible concrete types that a [`Base`] can be resolved to.
///
/// When using [`SyncGuard::resolve_data`] with a [`Base`], it will resolve to the concrete
/// object type that the base represents.
pub enum ObjectType {
    /// An audio source.
    AudioSource(audio::Source),

    /// An ambient light.
    AmbientLight(light::Ambient),

    /// A directional light.
    DirectionalLight(light::Directional),

    /// A hemisphere light.
    HemisphereLight(light::Hemisphere),

    /// A point light.
    PointLight(light::Point),

    /// A mesh.
    Mesh(Mesh),

    /// A group.
    Group(Group),

    /// A skeleton.
    Skeleton(Skeleton),

    /// A bone in a skeleton.
    Bone(Bone),

    /// A 2D sprite.
    Sprite(Sprite),

    /// A UI text object.
    Text(Text),
}

/// Groups are used to combine several other objects or groups to work with them
/// as with a single entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Group {
    object: Base,
}

impl AsRef<Base> for Group {
    fn as_ref(&self) -> &Base { &self.object }
}

impl Object for Group {
    type Data = Vec<Base>;

    fn resolve_data(&self, sync_guard: &mut SyncGuard) -> Vec<Base> {
        let mut children = Vec::new();
        let node = &sync_guard.hub[self];

        let mut child = match node.sub_node {
            SubNode::Group { ref first_child } => first_child.clone(),
            _ => panic!("`Group` had a bad sub node type: {:?}", node.sub_node),
        };

        while let Some(child_pointer) = child {
            child = sync_guard.hub.nodes[&child_pointer].next_sibling.clone();

            children.push(Base {
                node: child_pointer,
                tx: sync_guard.hub.message_tx.clone(),
            });
        }

        children
    }
}

impl Group {
    pub(crate) fn new(hub: &mut Hub) -> Self {
        let sub = SubNode::Group { first_child: None };
        Group {
            object: hub.spawn(sub),
        }
    }

    /// Add new [`Object`](trait.Object.html) to the group.
    pub fn add<T: Object>(
        &self,
        child: &T,
    ) {
        let node = child.as_ref().node.clone();
        self.as_ref().send(Operation::AddChild(node));
    }

    /// Removes a child [`Object`](trait.Object.html) from the group.
    pub fn remove<T: Object>(
        &self,
        child: &T,
    ) {
        let node = child.as_ref().node.clone();
        self.as_ref().send(Operation::RemoveChild(node));
    }
}
