//! Utilites for creating reusable templates for scene objects.
//!
//! It is often the case that you will want to have multiple instances of the same model or
//! hierarchy of objects in your scene. While you could manually construct each instance yourself,
//! three-rs provides a templating system to allow you to describe your model's hierarchy ahead
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
//! [`NodeTemplate`] has a sub-type that determines which type of object it is instantiated as,
//! and references the type-specific data for that object. See [`NodeTemplateData`] for
//! information on the different node sub-types.
//!
//! Data for specific types of nodes are held in the other member arrays of [`Template`]. For
//! example, [`cameras`] contains projections for the different cameras in the template, and
//! [`meshes`] contains reusable GPU data for meshes. Note that this data is put into separate
//! arrays, rather than held directly by the variants of [`NodeTemplateData`], so that multiple
//! nodes can share the same data (e.g. all cameras defined in the template can easily reuse
//! the same projection without having to have separate copies of the projection).
//!
//! The nodes in the template create a hierarchy when nodes with the sub-type
//! [`NodeTemplateData::Group`] list other nodes as their children. Only
//! [`NodeTemplateData::Group`] is able to have children, and it does not carry any other data.
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
//! # Mesh Instancing
//!
//! When setting up a mesh in a template, you must first upload your [`Geometry`] to the GPU
//! using [`Factory::upload_geometry`]. This will give you an [`InstancedGeometry`] object
//! that acts as a shared handle to the GPU resources for that geometry. By uploading the
//! data to the GPU ahead of time, we can ensure that all mesh nodes that reference that
//! geometry, and all [`Mesh`] instances created from the template, will share a single copy
//! of the data on the GPU. This reduces GPU resource usage and, for any meshes that also share
//! a material, allows three to render many objects at once.
//!
//! [`Factory::instantiate_template`]: ../struct.Factory.html#method.instantiate_template
//! [`Factory::load_gltf`]: ../struct.Factory.html#method.load_gltf
//! [`Factory::upload_geometry`]: ../struct.Factory.html#method.upload_geometry
//! [`Object`]: ../trait.Object.html
//! [`Group`]: ../struct.Group.html
//! [`Geometry`]: ../struct.Geometry.html
//! [`Mesh`]: ../struct.Mesh.html
//! [`Template`]: ./struct.Template.html
//! [`NodeTemplate`]: ./struct.NodeTemplate.html
//! [`NodeTemplateData`]: ./enum.NodeTemplateData.html
//! [`NodeTemplateData::Group`]: ./enum.NodeTemplateData.html#variant.Group
//! [`nodes`]: ./struct.Template.html#structfield.nodes
//! [`cameras`]: ./struct.Template.html#structfield.cameras
//! [`meshes`]: ./struct.Template.html#structfield.meshes
//! [`roots`]: ./struct.Template.html#structfield.roots
//! [`InstancedGeometry`]: ./struct.InstancedGeometry.html

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
#[derive(Debug, Clone, Default)]
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
    pub nodes: Vec<NodeTemplate>,

    /// Data for the lights described by this template.
    pub lights: Vec<LightTemplate>,

    /// Templates for animation clips that target objects instantiated from this template.
    pub animations: Vec<AnimationTemplate>,
}

impl Template {
    /// Creates an empty template.
    ///
    /// # Examples
    ///
    /// Create an empty template and then instantiate it, effectively the most verbose way to
    /// call [`Factory::group`]:
    ///
    /// ```no_run
    /// use three::template::Template;
    ///
    /// # let mut window = three::Window::new("Three-rs");
    /// let template = Template::new();
    /// let (group, animations) = window.factory.instantiate_template(&template);
    /// ```
    ///
    /// [`Factory::group`]: ../struct.Factory.html#method.group
    pub fn new() -> Template { Default::default() }
}

/// An object with a transform that can be added to the scene or made the child of a [`Group`].
///
/// See the [module documentation] for more information on how template nodes are used to
/// describe objects and build templates.
///
/// [`Group`]: ../struct.Group.html
/// [module documentation]: ./index.html#node-templates
#[derive(Debug, Clone)]
pub struct NodeTemplate {
    /// An optional name for the node.
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

    /// The specific type of object that this node will be instantiated into.
    pub data: NodeTemplateData,
}

impl NodeTemplate {
    /// Creates a default `NodeTemplate` with the provided node data.
    ///
    /// The created [`Template`] node has no translation, no rotation, and a scale of 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use three::template::{NodeTemplate, NodeTemplateData};
    ///
    /// let camera_node = NodeTemplate::from_data(NodeTemplateData::Camera(0));
    /// ```
    ///
    /// [`Template`]: ./struct.Template.html
    pub fn from_data(data: NodeTemplateData) -> NodeTemplate {
        NodeTemplate {
            name: None,

            // Provide a default transformation with no translation, no rotation, and a scale of 1.
            translation: [0.0, 0.0, 0.0].into(),
            rotation: [0.0, 0.0, 0.0, 1.0].into(),
            scale: 1.0,

            data
        }
    }
}

/// Defines which type of object a [`NodeTemplate`] will be instantiated into.
///
/// See the [module documentation] for more information on how template nodes are used to
/// describe objects and build templates.
///
/// [`NodeTemplate`]: ./struct.NodeTemplate.html
/// [module documentation]: ./index.html#node-templates
#[derive(Debug, Clone)]
pub enum NodeTemplateData {
    /// The node represents a [`Group`].
    ///
    /// Contains a list of nodes that will be added to the resulting group, given as indices into
    /// the [`nodes`] array in the parent [`Template`].
    ///
    /// [`Group`]: ../struct.Group.html
    /// [`nodes`]: ./struct.Template.html#structfield.nodes
    /// [`Template`]: ./struct.Template.html
    Group(Vec<usize>),

    /// The node represents a [`Mesh`].
    ///
    /// Specifies the index of the mesh data to be used in the [`meshes`] array of the parent
    /// [`Template`].
    ///
    /// [`Mesh`]: ../struct.Mesh.html
    /// [`meshes`]: ./struct.Template.html#structfield.meshes
    /// [`Template`]: ./struct.Template.html
    Mesh(usize),

    /// The node represents a skinned [`Mesh`] with a [`Skeleton`] attached.
    ///
    /// [`Mesh`]: ../struct.Mesh.html
    /// [`Skeleton`]: ../skeleton/struct.Skeleton.html
    SkinnedMesh {
        /// The index of the mesh in the [`meshes`] array of the parent [`Template`].
        ///
        /// [`meshes`]: ./struct.Template.html#structfield.meshes
        /// [`Template`]: ./struct.Template.html
        mesh: usize,

        /// The index of the skeleton node in the [`nodes`] array of the parent [`Template`].
        ///
        /// Note that this index must reference a node that has a [`NodeTemplateData::Skeleton`]
        /// for its [`data`] field.
        ///
        /// [`nodes`]: ./struct.Template.html#structfield.nodes
        /// [`Template`]: ./struct.Template.html
        /// [`data`]: ./struct.Template.html#structfield.data
        /// [`NodeTemplateData::Skeleton`]: #variant.Skeleton
        skeleton: usize,
    },

    /// The node represents one of the light types defined in the [`light`] module.
    ///
    /// Specifies the index of the light data in the [`lights`] array of the parent [`Template`].
    ///
    /// [`light`]: ../light/index.html
    /// [`lights`]: ./struct.Template.html#structfield.lights
    /// [`Template`]: ./struct.Template.html
    Light(usize),

    /// The node represents a [`Bone`].
    ///
    /// Contains the bone's index within its skeleton, and the inverse bind matrix for
    /// the bone. See [`Factory::bone`] for more information on these parameters.
    ///
    /// [`Bone`]: ../skeleton/struct.Bone.html
    /// [`Factory::bone`]: ../struct.Factory.html#method.bone
    Bone(usize, mint::ColumnMatrix4<f32>),

    /// The node represents a [`Skeleton`].
    ///
    /// Contains a list of the indices of the bone nodes in the [`nodes`] array of the parent
    /// [`Template`]. Note that the nodes referenced must have a [`NodeTemplateData::Bone`]
    /// for their [`data`] field.
    ///
    /// [`Skeleton`]: ../skeleton/struct.Skeleton.html
    /// [`nodes`]: ./struct.Template.html#structfield.nodes
    /// [`Template`]: ./struct.Template.html
    /// [`data`]: ./struct.Template.html#structfield.data
    /// [`NodeTemplateData::Bone`]: #variant.Bone
    Skeleton(Vec<usize>),

    /// The node represents a [`Camera`].
    ///
    /// Specifies the index of the projection in the [`cameras`] array of the parent [`Template`].
    ///
    /// [`Camera`]: ../camera/struct.Camera.html
    /// [`cameras`]: ./struct.Template.html#structfield.cameras
    /// [`Template`]: ./struct.Template.html
    Camera(usize),
}

/// Information for instantiating a [`Mesh`].
///
/// See the [module documentation] for more information on mesh instancing and how mesh
/// data is setup for templates.
///
/// [`Mesh`]: ../struct.Mesh.html
/// [module documentation]: ./index.html#mesh-instancing
#[derive(Debug, Clone)]
pub struct MeshTemplate {
    /// The geometry used in the mesh.
    pub geometry: InstancedGeometry,

    /// The index of the material for the mesh in the [`meshes`] array of the parent [`Template`].
    ///
    /// [`Template`]: ./struct.Template.html
    /// [`meshes`]: ./struct.Template.html#structfield.meshes
    pub material: usize,
}

/// The definition for an animation in a glTF file, corresponds to an [`animation::Clip`].
///
/// See the [module documentation] for more information on template animations and how they
/// are used.
///
/// [`animation::Clip`]: ../animation/struct.Clip.html
/// [module documentation]: ./index.html#animations
#[derive(Debug, Clone)]
pub struct AnimationTemplate {
    /// An optional name for the animation.
    pub name: Option<String>,

    /// The tracks making up the animation.
    ///
    /// Each track is composed of a [`Track`], containing the data for the track, and the node
    /// that the track targetes, specified as an index into the [`nodes`] array of the
    /// parent [`Template`].
    ///
    /// [`Track`]: ../animation/struct.Track.html
    /// [`Template`]: ./struct.Template.html
    /// [`nodes`]: ./struct.Template.html#structfield.nodes
    pub tracks: Vec<(Track, usize)>,
}

/// Common information for instantiating the various types of lights.
///
/// See the [module documentation] for information on how templates are setup and how objects
/// are added to the template.
///
/// [module documentation]: ./index.html
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
    /// Creates a new template for an ambient light, analogous to [`Factory::ambient_light`].
    ///
    /// # Examples
    ///
    /// ```
    /// use three::template::{LightTemplate, Template};
    ///
    /// let mut template = Template::new();
    /// let light = LightTemplate::ambient(three::color::RED, 0.5);
    /// template.lights.push(light);
    /// ```
    ///
    /// [`Factory::ambient_light`]: ../struct.Factory.html#method.ambient_light
    pub fn ambient(color: Color, intensity: f32) -> LightTemplate {
        LightTemplate {
            color,
            intensity,
            sub_light: SubLightTemplate::Ambient,
        }
    }

    /// Creates a new template for a directional light, analogous to [`Factory::directional_light`].
    ///
    /// # Examples
    ///
    /// ```
    /// use three::template::{LightTemplate, Template};
    ///
    /// let mut template = Template::new();
    /// let light = LightTemplate::directional(three::color::RED, 0.5);
    /// template.lights.push(light);
    /// ```
    ///
    /// [`Factory::directional_light`]: ../struct.Factory.html#method.directional_light
    pub fn directional(color: Color, intensity: f32) -> LightTemplate {
        LightTemplate {
            color,
            intensity,
            sub_light: SubLightTemplate::Directional,
        }
    }

    /// Creates a new template for a point light, analogous to [`Factory::point_light`].
    ///
    /// # Examples
    ///
    /// ```
    /// use three::template::{LightTemplate, Template};
    ///
    /// let mut template = Template::new();
    /// let light = LightTemplate::point(three::color::RED, 0.5);
    /// template.lights.push(light);
    /// ```
    ///
    /// [`Factory::point_light`]: ../struct.Factory.html#method.point_light
    pub fn point(color: Color, intensity: f32) -> LightTemplate {
        LightTemplate {
            color,
            intensity,
            sub_light: SubLightTemplate::Point,
        }
    }

    /// Creates a new template for a hemisphere light, analogous to [`Factory::hemisphere_light`].
    ///
    /// # Examples
    ///
    /// ```
    /// use three::template::{LightTemplate, Template};
    ///
    /// let mut template = Template::new();
    /// let light = LightTemplate::hemisphere(three::color::RED, three::color::BLUE, 0.5);
    /// template.lights.push(light);
    /// ```
    ///
    /// [`Factory::hemisphere_light`]: ../struct.Factory.html#method.hemisphere_light
    pub fn hemisphere(sky_color: Color, ground_color: Color, intensity: f32) -> LightTemplate {
        LightTemplate {
            color: sky_color,
            intensity,
            sub_light: SubLightTemplate::Hemisphere { ground: ground_color },
        }
    }
}

/// Template information about the different sub-types for light.
///
/// See [`LightTemplate`] for more more information on settings up light templates, and
/// utilities for doing so.
///
/// [`LightTemplate`]: ./struct.LightTemplate.html
#[derive(Clone, Copy, Debug)]
pub enum SubLightTemplate {
    /// Represents an ambient light, instantiated as an [`Ambient`].
    ///
    /// [`Ambient`]: ../light/struct.Ambient.html
    Ambient,

    /// Represents a directional light, instantiated as a [`Directional`].
    ///
    /// [`Directional`]: ../light/struct.Directional.html
    Directional,

    /// Represents a hemisphere light, instantiated as a [`Hemisphere`].
    ///
    /// [`Hemisphere`]: ../light/struct.Hemisphere.html
    Hemisphere {
        /// The ground color for the light.
        ground: Color,
    },

    /// Represents a point light, instantiated as a [`Point`].
    ///
    /// [`Point`]: ../light/struct.Point.html
    Point,
}

/// Geometry data that has been loaded to the GPU.
///
/// [`Mesh`] objects instantiated with this data will share GPU resources, allowing for more
/// efficient instanced rendering. Use [`Factory::upload_geometry`] to upload [`Geometry`]
/// to the GPU and get an `InstancedGeometry`. You can use an `InstancedGeometry` to create
/// a [`MeshTemplate`] for use in a [`Template`], or you can use [`Factory::create_instanced_mesh`]
/// to create a [`Mesh`] directly.
///
/// [`Factory::upload_geometry`]: ../struct.Factory.html#method.upload_geometry
/// [`Factory::create_instanced_mesh`]: ../struct.Factory.html#method.create_instanced_mesh
/// [`Mesh`]: ../struct.Mesh.html
/// [`Geometry`]: ../struct.Geometry.html
/// [`Template`]: ./struct.Template.html
/// [`MeshTemplate`]: ./struct.MeshTemplate.html
#[derive(Debug, Clone)]
pub struct InstancedGeometry {
    pub(crate) gpu_data: GpuData,
}
