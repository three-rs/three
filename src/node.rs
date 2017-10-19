use cgmath;
use froggy;
use mint;
use scene;

use hub::SubNode;
use material::Material;

/// Pointer to a Node
pub type NodePointer = froggy::Pointer<Node>;
pub(crate) type Transform = cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>;

/// Fat node of the scene graph.
///
/// `Node` is used by `three-rs` internally,
/// client code uses [`Object`](struct.Object.html) instead.
#[derive(Debug)]
pub struct Node {
    /// `true` if this node (and its subnodes) are visible to cameras.
    pub(crate) visible: bool,
    /// For internal use.
    pub(crate) world_visible: bool,
    /// The transform relative to the node's parent.
    pub(crate) transform: Transform,
    /// The transform relative to the world origin.
    pub(crate) world_transform: Transform,
    /// Pointer to node's parent.
    pub(crate) parent: Option<NodePointer>,
    /// The ID of the scene this node belongs to.
    pub(crate) scene_id: Option<scene::Uid>,
    /// Context specific-data, for example, `UiText`, `Visual` or `Light`.
    pub(crate) sub_node: SubNode,
}

/// Position, rotation, and scale of the scene [`Node`](struct.Node.html).
#[derive(Clone, Debug)]
pub struct NodeTransform {
    /// Position.
    pub position: mint::Point3<f32>,
    /// Orientation.
    pub orientation: mint::Quaternion<f32>,
    /// Scale.
    pub scale: f32,
}

impl From<Transform> for NodeTransform {
    fn from(tf: Transform) -> Self {
        let pos: mint::Vector3<f32> = tf.disp.into();
        NodeTransform {
            position: pos.into(),
            orientation: tf.rot.into(),
            scale: tf.scale,
        }
    }
}

/// General information about scene [`Node`](struct.Node.html).
#[derive(Clone, Debug)]
pub struct NodeInfo {
    /// Relative to parent transform.
    pub transform: NodeTransform,
    /// World transform (relative to the world's origin).
    pub world_transform: NodeTransform,
    /// Is `Node` visible by cameras or not?
    pub visible: bool,
    /// The same as `visible`, used internally.
    pub world_visible: bool,
    /// Material in case this `Node` has it.
    pub material: Option<Material>,
}

impl From<SubNode> for Node {
    fn from(sub: SubNode) -> Self {
        Node {
            visible: true,
            world_visible: false,
            transform: cgmath::Transform::one(),
            world_transform: cgmath::Transform::one(),
            parent: None,
            scene_id: None,
            sub_node: sub,
        }
    }
}
