//! Utilites for creating reusable templates for scene objects.
//!
//! It is often the case that you will want to have multiple instances of the same model or
//! hierarchy of object in your scene. While you could manually construct each instance yourself,
//! three provides a templating system to allow you to describe your model's hierarchy ahead
//! of time, and then quickly create instances that three can efficiently batch render.
//! [`Template`] describes the objects for a single model, and can be instantiated with
//! [`Factory::instantiate_template`].
//!
//! The easiest way to create a template is to load one from a glTF file using
//! [`Factory::load_gltf`].
//!
//! # Node Templates
//!
//! Templates hold their data in flat array, and objects in the template reference each other
//! using their respective indices. The base descriptions of all objects that can be added to the
//! scene (i.e. that implement [`Object`] and have transforms) are represented in [`nodes`]. Each
//! [`TemplateNode`] has a sub-type that determines which type of object it is instantiated as,
//! and references the type-specific data for that object. See [`TemplateNodeData`] for
//! information on the different node sub-types.
//!
//! Data for specific types of nodes are held in the other member arrays of [`Template`]. For
//! example, [`cameras`] contains projections for the different cameras in the template, and
//! [`meshes`] contains reusable GPU data for meshes. Note that this data is put into separate
//! arrays, rather than held directly by the variants of [`TemplateNodeData`], so that multiple
//! nodes can share the same data (e.g. all cameras defined in the template can easily reuse
//! the same projection without having to have separate copies of the projection).
//!
//! The nodes in the template create a hierarchy when nodes with the sub-type
//! [`TemplateNodeData::Group`] list other nodes as their children. Only
//! [`TemplateNodeData::Group`] is able to have children, and it does not carry any other data.
//!
//! The root nodes of the template are specified in [`roots`]. The nodes specified by [`roots`]
//! will be the direct children of the [`Group`] returned from [`Factory::instantiate_template`],
//! and all other nodes will be children of those nodes.
//!
//! # Animations
//!
//! Templates can also describe animations that apply to the objects described by the template.
//! When instantiated, the resulting animation clips will be unique to that instance of of the
//! template. This allows for all animations for the template to be described once, while still
//! allowing all instances of the template to be animated independently of each other.
//!
//! [`Factory::instantiate_template`]: ../struct.Factory.html#method.instantiate_template
//! [`Factory::load_gltf`]: ../struct.Factory.html#method.load_gltf
//! [`Object`]: ../trait.Object.html
//! [`Group`]: ../struct.Group.html
//! [`Template`]: ./struct.Template.html
//! [`TemplateNode`]: ./struct.TemplateNode.html
//! [`TemplateNodeData`]: ./enum.TemplateNodeData.html
//! [`TemplateNodeData::Group`]: ./enum.TemplateNodeData.html#variant.Group
//! [`nodes`]: ./struct.Template.html#structfield.nodes
//! [`cameras`]: ./struct.Template.html#structfield.cameras
//! [`meshes`]: ./struct.Template.html#structfield.meshes
//! [`roots`]: ./struct.Template.html#structfield.roots

use mint;

use camera::{Projection};

use {Material};
use color::Color;
use animation::Track;
use render::GpuData;

/// A template representing a hierarchy of objects.
///
/// To create an instance of the template that can be added to your scene, use
/// [`Factory::instantiate_template`]. For more information about the templating system and how
/// to use it, see the [module documentation].
///
/// [`Factory::instantiate_template`]: ../struct.Factory.html#method.instantiate_template
/// [module documentation]: ./index.html
#[derive(Debug, Clone)]
pub struct Template {
    /// An optional name for the template.
    pub name: Option<String>,

    /// The indices of the nodes in [`nodes`] that act as the root nodes of the template.
    ///
    /// When the template is instantiated, the indicated nodes will be the direct children of
    /// the resulting [`Group`].
    ///
    /// [`nodes`]: #structfield.nodes
    /// [`Group`]: ../struct.Group.html
    pub roots: Vec<usize>,

    /// Projection data used by cameras defined in the template.
    pub cameras: Vec<Projection>,

    /// The materials used by the meshes defined in [`meshes`].
    ///
    /// [`meshes`]: #structfield.meshes
    pub materials: Vec<Material>,

    /// The meshes defined in this template.
    pub meshes: Vec<MeshTemplate>,

    /// All objects defined by this template.
    pub nodes: Vec<TemplateNode>,

    /// Data for the lights described by this template.
    pub lights: Vec<LightTemplate>,

    /// Templates for animation clips that target objects instantiated from this template.
    pub animations: Vec<AnimationTemplate>,
}

/// The definition of a node used in a glTF file.
///
/// Nodes are composed to create a graph of elements in a glTF scene.
#[derive(Debug, Clone)]
pub struct TemplateNode {
    /// The name of the node.
    pub name: Option<String>,

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

    /// The specific type of Three object that this node will be instantiated into, and its
    /// associated data.
    pub data: TemplateNodeData,
}

impl TemplateNode {
    /// Creates a default `TemplateNode` with the provided node data.
    ///
    /// This is used by `Factory::load_gltf`, which needs to generate new nodes on the fly with
    /// a default transform.
    pub fn from_data(data: TemplateNodeData) -> TemplateNode {
        TemplateNode {
            name: None,

            // Provide a default transformation with no translation, no rotation, and a scale of 1.
            translation: [0.0, 0.0, 0.0].into(),
            rotation: [0.0, 0.0, 0.0, 1.0].into(),
            scale: 1.0,

            data
        }
    }
}

/// The specific type of Three object that a `TemplateNode` will become when instantiated.
#[derive(Debug, Clone)]
pub enum TemplateNodeData {
    /// A node representing a [`Group`].
    ///
    /// Contains a list of the indices of the nodes that are in the group.
    Group(Vec<usize>),

    /// A node representing a [`Mesh`].
    ///
    /// Contains the index of the mesh in [`meshes`].
    Mesh(usize),

    /// A node representing a [`Mesh`] with an attached [`Skeleton`].
    ///
    /// The first `usize` is the index of the mesh in [`meshes`], the second `usize` is the
    /// index of the skeleton node in [`nodes`]. Note that the second index must reference a
    /// node that has a [`TemplateNodeData::Skeleton`] for its [`data`] field.
    SkinnedMesh(usize, usize),

    /// A node representing a [`Light`].
    Light(usize),

    /// A node representing a [`Bone`].
    ///
    /// Contains the index of the bone within its skeleton, and the inverse bind matrix for
    /// the bone.
    Bone(usize, mint::ColumnMatrix4<f32>),

    /// A node representing a [`Skeleton`].
    ///
    /// Contains the indices of the bones nodes in the scene that are the bones in this skeleton.
    /// These indices correspond to elements in [`nodes`] in the parent [`Template`]. Note that
    /// the nodes references must have a [`TemplateNodeData::Bone`] for their [`data`] field.
    Skeleton(Vec<usize>),

    /// A node representing a [`Camera`].
    Camera(usize),
}

/// Information describing a mesh.
#[derive(Debug, Clone)]
pub struct MeshTemplate {
    /// The geometry used in the mesh.
    pub geometry: InstancedGeometry,

    /// The index for the material to use in the mesh.
    pub material: usize,
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

/// Template for a light in the scene.
#[derive(Clone, Copy, Debug)]
pub struct LightTemplate {
    /// The base color of the light.
    pub color: Color,

    /// The intensity of the light.
    pub intensity: f32,

    /// The specific type of light represented by the template.
    pub sub_light: SubLightTemplate,
}

impl LightTemplate {
    /// Creates a new template for an ambient light.
    pub fn ambient(color: Color, intensity: f32) -> LightTemplate {
        LightTemplate {
            color,
            intensity,
            sub_light: SubLightTemplate::Ambient,
        }
    }

    /// Creates a new template for a directional light.
    pub fn directional(color: Color, intensity: f32) -> LightTemplate {
        LightTemplate {
            color,
            intensity,
            sub_light: SubLightTemplate::Directional,
        }
    }

    /// Creates a new template for a point light.
    pub fn point(color: Color, intensity: f32) -> LightTemplate {
        LightTemplate {
            color,
            intensity,
            sub_light: SubLightTemplate::Point,
        }
    }

    /// Creates a new template for a hemisphere light.
    pub fn hemisphere(sky_color: Color, ground_color: Color, intensity: f32) -> LightTemplate {
        LightTemplate {
            color: sky_color,
            intensity,
            sub_light: SubLightTemplate::Hemisphere { ground: ground_color },
        }
    }
}

/// Template information about the different sub-types for light.
#[derive(Clone, Copy, Debug)]
pub enum SubLightTemplate {
    /// Represents an ambient light.
    Ambient,

    /// Represents a directional light.
    Directional,

    /// Represents a hemisphere light.
    Hemisphere {
        /// The ground color for the light.
        ground: Color,
    },

    /// Represents a point light.
    Point,
}

/// Geometry data that has been loaded to the GPU.
///
/// [`Mesh`] objects instanted with this data will share GPU resources, allowing for more
/// efficient instanced rendering.
#[derive(Debug, Clone)]
pub struct InstancedGeometry {
    pub(crate) gpu_data: GpuData,
}
