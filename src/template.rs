//! Utilites for creating reusable templates for scene objects.

use mint;

use camera::{Projection};

use {Material};
use color::Color;
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
    pub meshes: Vec<MeshTemplate>,

    /// The scene nodes loaded from the glTF file.
    pub nodes: Vec<TemplateNode>,

    /// The animation clips loaded from the glTF file.
    pub animations: Vec<AnimationTemplate>,

    /// Light templates to be used as part of the template.
    pub lights: Vec<LightTemplate>,
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
    pub(crate) fn from_data(data: TemplateNodeData) -> TemplateNode {
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
    // TODO: Use a shared GPU resource, rather than keeping the geometry data in CPU memory.
    pub geometry: Geometry,

    /// The index for the material to use in the mesh, if specified.
    pub material: Option<usize>,
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
