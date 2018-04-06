//! Items in the scene heirarchy.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::mpsc;

use mint;

use hub::{Hub, Message, Operation, SubNode};
use node::NodePointer;

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
impl Object for Base {}

/// Groups are used to combine several other objects or groups to work with them
/// as with a single entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Group {
    pub(crate) object: Base,
}
three_object!(Group::object);

impl Group {
    pub(crate) fn new(hub: &mut Hub) -> Self {
        let sub = SubNode::Group { first_child: None };
        Group {
            object: hub.spawn(sub.into()),
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
