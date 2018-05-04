use cgmath;
use froggy;
use mint;

use hub::SubNode;
use material::Material;

use std::marker::PhantomData;


/// Pointer to a Node
pub(crate) type NodePointer = froggy::Pointer<NodeInternal>;
pub(crate) type TransformInternal = cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>;

// Fat node of the scene graph.
//
// `NodeInternal` is used by `three-rs` to represent an object in our scene graph,
// shaped as a node tree. Client code uses [`object::Base`](struct.Base.html) instead.
#[derive(Debug)]
pub(crate) struct NodeInternal {
    /// `true` if this node (and its children) are visible to cameras.
    pub(crate) visible: bool,

    /// A user-defined name for the node.
    ///
    /// Not used internally to implement functionality. This is used by users to identify nodes
    /// programatically, and to act as a utility when debugging.
    pub(crate) name: Option<String>,

    /// The transform relative to the node's parent.
    pub(crate) transform: TransformInternal,

    /// The transform relative to the scene root.
    pub(crate) world_transform: TransformInternal,

    /// Pointer to the next sibling.
    pub(crate) next_sibling: Option<NodePointer>,

    /// Context specific-data, for example, `UiText`, `Visual` or `Light`.
    pub(crate) sub_node: SubNode,
}

impl NodeInternal {
    pub(crate) fn to_node(&self) -> Node<Local> {
        Node {
            transform: self.transform.into(),
            visible: self.visible,
            name: self.name.clone(),
            material: match self.sub_node {
                SubNode::Visual(ref mat, _, _) => Some(mat.clone()),
                _ => None,
            },
            _space: PhantomData,
        }
    }
}

impl From<SubNode> for NodeInternal {
    fn from(sub: SubNode) -> Self {
        NodeInternal {
            visible: true,
            name: None,
            transform: cgmath::Transform::one(),
            world_transform: cgmath::Transform::one(),
            next_sibling: None,
            sub_node: sub,
        }
    }
}

/// Position, rotation, and scale of the scene node.
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

/// Local space, defined relative to the parent node.
pub enum Local {}
/// World space, defined relative to the scene root.
pub enum World {}

/// General information about an object in a scene.
#[derive(Clone, Debug, PartialEq)]
pub struct Node<Space> {
    /// Is `Node` visible by cameras or not?
    pub visible: bool,

    /// The name of the node, if any.
    pub name: Option<String>,

    /// Transformation in `Space`.
    // NOTE: this really begs for `euclid`-style parametrized math types.
    pub transform: Transform,

    /// Material in case this `Node` has it.
    pub material: Option<Material>,

    ///
    pub(crate) _space: PhantomData<Space>,
}
