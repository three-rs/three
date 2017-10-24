use cgmath;
use froggy;
use mint;
use scene;

use hub::SubNode;
use material::Material;

/// Pointer to a Node
pub(crate) type NodePointer = froggy::Pointer<NodeInternal>;
pub(crate) type TransformInternal = cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>;

// Fat node of the scene graph.
//
// `Node` is used by `three-rs` internally,
// client code uses [`Object`](struct.Object.html) instead.
#[derive(Debug)]
pub(crate) struct NodeInternal {
    /// `true` if this node (and its subnodes) are visible to cameras.
    pub(crate) visible: bool,
    /// For internal use.
    pub(crate) world_visible: bool,
    /// The transform relative to the node's parent.
    pub(crate) transform: TransformInternal,
    /// The transform relative to the world origin.
    pub(crate) world_transform: TransformInternal,
    /// Pointer to node's parent.
    pub(crate) parent: Option<NodePointer>,
    /// The ID of the scene this node belongs to.
    pub(crate) scene_id: Option<scene::Uid>,
    /// Context specific-data, for example, `UiText`, `Visual` or `Light`.
    pub(crate) sub_node: SubNode,
}

/// Position, rotation, and scale of the scene `Node`.
#[derive(Clone, Debug)]
pub struct Transform {
    /// Position.
    pub position: mint::Point3<f32>,
    /// Orientation.
    pub orientation: mint::Quaternion<f32>,
    /// Scale.
    pub scale: f32,
}

impl From<TransformInternal> for Transform {
    fn from(tf: TransformInternal) -> Self {
        let pos: mint::Vector3<f32> = tf.disp.into();
        Transform {
            position: pos.into(),
            orientation: tf.rot.into(),
            scale: tf.scale,
        }
    }
}

/// General information about scene `Node`.
#[derive(Clone, Debug)]
pub struct Node {
    /// Relative to parent transform.
    pub transform: Transform,
    /// World transform (relative to the world's origin).
    pub world_transform: Transform,
    /// Is `Node` visible by cameras or not?
    pub visible: bool,
    /// The same as `visible`, used internally.
    pub world_visible: bool,
    /// Material in case this `Node` has it.
    pub material: Option<Material>,
}

impl From<SubNode> for NodeInternal {
    fn from(sub: SubNode) -> Self {
        NodeInternal {
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
