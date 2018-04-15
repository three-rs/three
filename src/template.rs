//! Utilites for creating reusable templates for scene objects.

use mint;

use camera::{Projection};

use {Material};
use animation::Track;
use geometry::Geometry;

/// Raw data loaded from a glTF file with [`Factory::load_gltf`].
///
/// This is the raw data used as a template to instantiate three objects in the scene. Entire
/// glTF scenes can be instantiated using [`Factory::instantiate_gltf_scene`].
///
/// [`Factory::load_gltf`]: struct.Factory.html#method.load_gltf
#[derive(Debug, Clone)]
pub struct Template {
    /// The name of the scene.
    pub name: Option<String>,

    /// The nodes in `nodes` that act as the root nodes of the template.
    pub roots: Vec<usize>,

    /// The camera projections defined in the glTF file.
    pub cameras: Vec<Projection>,

    /// The materials defined in this template.
    pub materials: Vec<Material>,

    /// The meshes defined in this template.
    pub meshes: Vec<Vec<MeshTemplate>>,

    /// The scene nodes loaded from the glTF file.
    pub nodes: Vec<TemplateNode>,

    /// The skinned skeltons loaded from the glTF file.
    pub skeletons: Vec<SkeletonTemplate>,

    /// The animation clips loaded from the glTF file.
    pub animations: Vec<AnimationTemplate>,
}

/// The definition of a node used in a glTF file.
///
/// Nodes are composed to create a graph of elements in a glTF scene.
#[derive(Debug, Clone)]
pub struct TemplateNode {
    /// The name of the node.
    pub name: Option<String>,

    /// The index of the mesh associated with this node, if any.
    ///
    /// The index can be used to lookup the associated mesh definition in the `meshes` map of the
    /// parent [`GltfDefinitions`].
    pub mesh: Option<usize>,

    /// The index of the camera associated with this node, if any.
    ///
    /// The index can be used to lookup the associated camera projection in the `cameras` map of
    /// the parent [`GltfDefinitions`].
    pub camera: Option<usize>,

    /// The index of the skin attached to this node, if any.
    ///
    /// The index corresponds to a skin in the `skins` list of the parent [`GltfDefinitions`].
    ///
    /// Note that if `skin` has a value, then `mesh` will also have a value.
    pub skeleton: Option<usize>,

    /// The indices of this node's children. A node may have zero or more children.
    ///
    /// Each index corresponds to a node in the `nodes` map of the parent [`GltfDefinitions`].
    pub children: Vec<usize>,

    /// The node's local translation.
    ///
    /// This translation is relative to its parent node when instantiated.
    pub translation: mint::Point3<f32>,

    /// The node's local orientation.
    ///
    /// This rotation is relative to its parent node when instantiated.
    pub rotation: mint::Quaternion<f32>,

    /// The node's local scale.
    ///
    /// This scale is relative to its parent node when instantiated.
    pub scale: f32,
}

/// Information describing a mesh.
#[derive(Debug, Clone)]
pub struct MeshTemplate {
    /// The geometry used in the mesh.
    // TODO: Use a shared GPU resource, rather than keeping the geometry data in CPU memory.
    pub geometry: Geometry,

    /// The index for the material to use in the mesh, if specified.
    pub material: Option<usize>,
}

/// The definition for a skeleton used for vertex skinning in a glTF file.
///
/// When instantiated, this corresponds to a [`Skeleton`].
#[derive(Debug, Clone)]
pub struct SkeletonTemplate {
    /// The bones composing the skeleton.
    pub bones: Vec<BoneTemplate>,
}

/// The definition for a bone in a [`GltfSkinDefinition`].
///
/// When instantiated, this corresponds to a [`Bone`].
#[derive(Debug, Clone)]
pub struct BoneTemplate {
    /// The inverse bind matrix used to transform the mesh for this bone's joint.
    pub inverse_bind_matrix: mint::ColumnMatrix4<f32>,

    /// The index of the node that acts as the joint for this bone.
    ///
    /// This index corresponds to a node in the `nodes` list of the parent [`GltfDefinitions`].
    pub joint: usize,
}

/// The definition for an animation in a glTF file.
///
/// When instantiated, this corresponds to a [`Clip`].
#[derive(Debug, Clone)]
pub struct AnimationTemplate {
    /// The name of the animation.
    pub name: Option<String>,

    /// The tracks making up the animation.
    ///
    /// Each track is composed of a [`Track`] containing the data for the track, and an index
    /// of the node that the track targets. The node is an index into the `nodes` list of the
    /// parent [`GltfDefinitions`].
    pub tracks: Vec<(Track, usize)>,
}
