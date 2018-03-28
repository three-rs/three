use mint;
use std::collections::HashMap;

use camera::{Camera, Projection};
use skeleton::Skeleton;

use animation::Track;
use {Group, Material, Mesh, Texture};
use geometry::Geometry;
use object;

/// A glTF scene that has been instantiated and can be added to a [`Scene`].
///
/// Created by instantiating a scene defined in a [`GltfDefinitions`] with
/// [`Factory::instantiate_gltf_scene`]. A `Hierarchy` can be added to a [`Scene`] with
/// [`Scene::add`].
///
/// # Examples
///
/// ```no_run
/// # let mut window = three::Window::new("three-rs");
/// let definitions = window.factory.load_gltf("my_data.gltf");
///
/// let gltf_scene = window.factory.instantiate_gltf_scene(&definitions, 0);
/// window.scene.add(&gltf_scene);
/// ```
///
/// [`Scene`]: struct.Scene.html
/// [`Scene::add`]: struct.Scene.html#method.add
/// [`GltfDefinitions`]: struct.GltfDefinitions.html
/// [`Factory::instantiate_gltf_scene`]: struct.Factory.html#method.instantiate_gltf_scene
#[derive(Debug, Clone)]
pub struct Hierarchy {
    /// A group containing all of the root nodes of the scene.
    ///
    /// While the glTF format allows scenes to have an arbitrary number of root nodes, all scene
    /// roots are added to a single root group to make it easier to manipulate the scene as a
    /// whole. See [`roots`] for the full list of root nodes for the scene.
    ///
    /// [`roots`]: #structfield.roots
    pub group: object::Group,

    /// The indices of the root nodes of the scene.
    ///
    /// Each index corresponds to a node in [`nodes`].
    ///
    /// [`nodes`]: #structfield.nodes
    pub roots: Vec<usize>,

    /// The nodes that are part of the scene.
    ///
    /// Node instances are stored in a [`HashMap`] where the key is the node's index in the source
    /// [`GltfDefinitions::nodes`]. Note that a [`HashMap`] is used instead of a [`Vec`] because
    /// not all nodes in the source file are guaranteed to be used in the scene, and so node
    /// indices in the scene instance may not be contiguous.
    ///
    /// [`HashMap`]: https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html
    /// [`Vec`]: https://doc.rust-lang.org/stable/std/vec/struct.Vec.html
    /// [`GltfDefinitions::nodes`]: struct.GltfDefinitions.html#structfield.nodes
    pub nodes: HashMap<usize, Node>,
}

impl Hierarchy {
    /// Finds the first node in the scene with the specified name, using a [`GltfDefinitions`]
    /// to lookup the name for each node.
    ///
    /// Name matching is case-sensitive. Returns the first node with a matching name, otherwise
    /// returns `None`.
    pub fn find_node_by_name(
        &self,
        name: &str,
        definitions: &GltfDefinitions,
    ) -> Option<&Node> {
        for (index, node) in &self.nodes {
            if let Some(node_def) = definitions.nodes.get(*index) {
                if node_def.name.as_ref().map(|node_name| node_name == name).unwrap_or(false) {
                    return Some(node);
                }
            }
        }

        None
    }
}

impl AsRef<object::Base> for Hierarchy {
    fn as_ref(&self) -> &object::Base {
        self.group.as_ref()
    }
}

impl object::Object for Hierarchy {}

/// A node in a scene from a glTF file that has been instantiated.
#[derive(Debug, Clone)]
pub struct Node {
    /// The group that represents this node.
    pub group: Group,

    /// The meshes associated with this node.
    pub meshes: Vec<Mesh>,

    /// The skeleton associated with this node.
    ///
    /// If `skeleton` has a value, then there will be at least one mesh in `meshes`.
    pub skeleton: Option<Skeleton>,

    /// The camera associated with this node.
    pub camera: Option<Camera>,

    /// The indices of the children of this node.
    pub children: Vec<usize>,
}

impl AsRef<object::Base> for Node {
    fn as_ref(&self) -> &object::Base {
        self.group.as_ref()
    }
}

impl object::Object for Node {}

/// Raw data loaded from a glTF file with [`Factory::load_gltf`].
///
/// This is the raw data used as a template to instantiate three objects in the scene. Entire
/// glTF scenes can be instantiated using [`Factory::instantiate_gltf_scene`].
///
/// [`Factory::load_gltf`]: struct.Factory.html#method.load_gltf
#[derive(Debug, Clone)]
pub struct GltfDefinitions {
    /// The materials loaded from the glTF file.
    pub materials: Vec<Material>,

    /// The camera projections defined in the glTF file.
    pub cameras: Vec<Projection>,

    /// The meshes loaded from the glTF file.
    pub meshes: Vec<GltfMeshDefinition>,

    /// The scene nodes loaded from the glTF file.
    pub nodes: Vec<NodeDefinition>,

    /// The scenes described in the glTF file.
    pub scenes: Vec<HierarchyDefinition>,

    /// The index of the default scene, if specified by the glTF file.
    ///
    /// The index corresponds to an element in `scenes`.
    pub default_scene: Option<usize>,

    /// The skinned skeltons loaded from the glTF file.
    pub skins: Vec<GltfSkinDefinition>,

    /// The animation clips loaded from the glTF file.
    pub animations: Vec<GltfAnimationDefinition>,

    /// Imported textures.
    pub textures: Vec<Texture<[f32; 4]>>,
}

/// A template for a glTF mesh instance.
///
/// Note that a glTF mesh doesn't map directly to three's [`Mesh`] type (see
/// [`GltfPrimitiveDefinition`] for a more direct analogy). Rather, `GltfMeshDefinition` can
/// be instantated into a [`Group`] with one or more [`Mesh`] instances added to the group.
///
/// [`Mesh`]: struct.Mesh.html
/// [`GltfPrimitiveDefinition`]: struct.GltfPrimitiveDefinition.html
/// [`Group`]: struct.Group.html
#[derive(Debug, Clone)]
pub struct GltfMeshDefinition {
    /// The name of the mesh template.
    pub name: Option<String>,

    /// The primitives included in the mesh template.
    ///
    /// When the mesh template is instantiated, each primitive is instantiated as a [`Mesh`].
    pub primitives: Vec<GltfPrimitiveDefinition>,
}

/// A template for a glTF mesh primitive.
///
/// A `GltfPrimitiveDefinition` can be converted directly into a [`Mesh`] using [`Factory::mesh`]. Note that
/// to do this, the material must first be retrieved by index from the parent [`GltfDefinitions`].
#[derive(Debug, Clone)]
pub struct GltfPrimitiveDefinition {
    /// The geometric data described by this primitive.
    pub geometry: Geometry,

    /// The index of the material associated with this mesh primitive, if any.
    ///
    /// The index can be used to lookup the material data from the `materials` map of the parent
    /// [`GltfDefinitions`].
    ///
    /// If no material is specified, then the glTF default material (an unlit, flat black material)
    /// will be used when instantiating the primitive.
    pub material: Option<usize>,
}

/// The definition of a node used in a glTF file.
///
/// Nodes are composed to create a graph of elements in a glTF scene.
#[derive(Debug, Clone)]
pub struct NodeDefinition {
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
    pub skin: Option<usize>,

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

/// The definition of a scene from a glTF file.
///
/// A glTF scene is a hierarchy of nodes, begining with one or more root nodes. Note that glTF
/// scenes are *not* the same as three [`Scene`]s, and must be explicity added to a [`Scene`]
/// when instantiated.
#[derive(Debug, Clone)]
pub struct HierarchyDefinition {
    /// The name of the scene.
    pub name: Option<String>,

    /// The indices of the root nodes of the scene.
    ///
    /// These indices correspond to elements in the
    pub roots: Vec<usize>,
}

/// The definition for a skeleton used for vertex skinning in a glTF file.
///
/// When instantiated, this corresponds to a [`Skeleton`].
#[derive(Debug, Clone)]
pub struct GltfSkinDefinition {
    /// The bones composing the skeleton.
    pub bones: Vec<GltfBoneDefinition>,
}

/// The definition for a bone in a [`GltfSkinDefinition`].
///
/// When instantiated, this corresponds to a [`Bone`].
#[derive(Debug, Clone)]
pub struct GltfBoneDefinition {
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
pub struct GltfAnimationDefinition {
    /// The name of the animation.
    pub name: Option<String>,

    /// The tracks making up the animation.
    ///
    /// Each track is composed of a [`Track`] containing the data for the track, and an index
    /// of the node that the track targets. The node is an index into the `nodes` list of the
    /// parent [`GltfDefinitions`].
    pub tracks: Vec<(Track, usize)>,
}
