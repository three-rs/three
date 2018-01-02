use cgmath;
use froggy;
use mint;

use hub::SubNode;
use material::Material;

/// Pointer to a Node
pub(crate) type NodePointer = froggy::Pointer<NodeInternal>;
pub(crate) type TransformInternal = cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>;

// Fat node of the scene graph.
//
// `NodeInternal` is used by `three-rs` internally,
// client code uses [`object::Base`](struct.Base.html) instead.
#[derive(Debug)]
pub(crate) struct NodeInternal {
    /// `true` if this node (and its children) are visible to cameras.
    pub(crate) visible: bool,
    /// The transform relative to the node's parent.
    pub(crate) transform: TransformInternal,
    /// Pointer to the next sibling.
    pub(crate) next_sibling: Option<NodePointer>,
    /// Context specific-data, for example, `UiText`, `Visual` or `Light`.
    pub(crate) sub_node: SubNode,
}

impl NodeInternal {
    pub(crate) fn to_node(&self) -> Node {
        Node {
            transform: self.transform.into(),
            visible: self.visible,
            material: match self.sub_node {
                SubNode::Visual(ref mat, _) => Some(mat.clone()),
                _ => None,
            },
        }
    }
}

impl From<SubNode> for NodeInternal {
    fn from(sub: SubNode) -> Self {
        NodeInternal {
            visible: true,
            transform: cgmath::Transform::one(),
            next_sibling: None,
            sub_node: sub,
        }
    }
}

/// Position, rotation, and scale of the scene `Node`.
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    /// Is `Node` visible by cameras or not?
    pub visible: bool,
    /// Relative to parent transform.
    pub transform: Transform,
    /// Material in case this `Node` has it.
    pub material: Option<Material>,
}
