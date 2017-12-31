//! Items in the scene heirarchy.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::mpsc;

use mint;

use hub::{Message, Operation};
use mesh::MAX_TARGETS;
use node::{Node, NodePointer};
use scene::Scene;

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
pub trait Object: AsRef<Base> + AsMut<Base> {
    /// Converts into the base type.
    fn upcast(&self) -> Base {
        self.as_ref().clone()
    }

    /// Invisible objects are not rendered by cameras.
    fn set_visible(
        &mut self,
        visible: bool,
    ) {
        self.as_mut().set_visible(visible)
    }

    /// Rotates object in the specific direction of `target`.
    fn look_at<E, T>(
        &mut self,
        eye: E,
        target: T,
        up: Option<mint::Vector3<f32>>,
    ) where
        Self: Sized,
        E: Into<mint::Point3<f32>>,
        T: Into<mint::Point3<f32>>,
    {
        self.as_mut().look_at(eye, target, up)
    }

    /// Set both position, orientation and scale.
    fn set_transform<P, Q>(
        &mut self,
        pos: P,
        rot: Q,
        scale: f32,
    ) where
        Self: Sized,
        P: Into<mint::Point3<f32>>,
        Q: Into<mint::Quaternion<f32>>,
    {
        self.as_mut().set_transform(pos, rot, scale)
    }

    /// Add new [`Base`](struct.Base.html) to the group.
    fn set_parent<P>(
        &mut self,
        parent: P,
    ) where
        Self: Sized,
        P: AsRef<Base>,
    {
        self.as_mut().set_parent(parent)
    }

    /// Set position.
    fn set_position<P>(
        &mut self,
        pos: P,
    ) where
        Self: Sized,
        P: Into<mint::Point3<f32>>,
    {
        self.as_mut().set_position(pos)
    }

    /// Set orientation.
    fn set_orientation<Q>(
        &mut self,
        rot: Q,
    ) where
        Self: Sized,
        Q: Into<mint::Quaternion<f32>>,
    {
        self.as_mut().set_orientation(rot)
    }

    /// Set scale.
    fn set_scale(
        &mut self,
        scale: f32,
    ) {
        self.as_mut().set_scale(scale)
    }

    /// Get actual information about itself from the `scene`.
    /// # Panics
    /// Panics if `scene` doesn't have this `Base`.
    fn sync(
        &mut self,
        scene: &Scene,
    ) -> Node {
        self.as_mut().sync(scene)
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
    /// Invisible objects are not rendered by cameras.
    pub fn set_visible(
        &mut self,
        visible: bool,
    ) {
        let msg = Operation::SetVisible(visible);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    /// Rotates object in the specific direction of `target`.
    pub fn look_at<E, T>(
        &mut self,
        eye: E,
        target: T,
        up: Option<mint::Vector3<f32>>,
    ) where
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

    /// Set both position, orientation and scale.
    pub fn set_transform<P, Q>(
        &mut self,
        pos: P,
        rot: Q,
        scale: f32,
    ) where
        P: Into<mint::Point3<f32>>,
        Q: Into<mint::Quaternion<f32>>,
    {
        let msg = Operation::SetTransform(Some(pos.into()), Some(rot.into()), Some(scale));
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    /// Add new [`Base`](struct.Base.html) to the group.
    pub fn set_parent<P>(
        &mut self,
        parent: P,
    ) where
        P: AsRef<Self>,
    {
        let msg = Operation::SetParent(parent.as_ref().node.clone());
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    /// Set position.
    pub fn set_position<P>(
        &mut self,
        pos: P,
    ) where
        P: Into<mint::Point3<f32>>,
    {
        let msg = Operation::SetTransform(Some(pos.into()), None, None);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    /// Set orientation.
    pub fn set_orientation<Q>(
        &mut self,
        rot: Q,
    ) where
        Q: Into<mint::Quaternion<f32>>,
    {
        let msg = Operation::SetTransform(None, Some(rot.into()), None);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    /// Set scale.
    pub fn set_scale(
        &mut self,
        scale: f32,
    ) {
        let msg = Operation::SetTransform(None, None, Some(scale));
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    /// Set weights.
    pub fn set_weights(
        &mut self,
        weights: [f32; MAX_TARGETS],
    ) {
        let msg = Operation::SetWeights(weights);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }
    
    /// Get actual information about itself from the `scene`.
    /// # Panics
    /// Panics if `scene` doesn't have this `Base`.
    pub fn sync(
        &mut self,
        scene: &Scene,
    ) -> Node {
        let mut hub = scene.hub.lock().unwrap();
        hub.process_messages();
        hub.update_graph();
        let node = &hub.nodes[&self.node];
        let root = &hub.nodes[&scene.object.node];
        assert_eq!(node.scene_id, root.scene_id);
        node.to_node()
    }
}

impl AsRef<Base> for Base {
    fn as_ref(&self) -> &Base {
        self
    }
}

impl AsMut<Base> for Base {
    fn as_mut(&mut self) -> &mut Base {
        self
    }
}
