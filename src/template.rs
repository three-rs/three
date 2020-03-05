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
//! # Object Relations
//!
//! Often, one object needs to reference another object in the template, e.g. a bone needs
//! to specify which skeleton it belongs to, and any object can specify that it belongs to
//! a group in the template. When doing so, objects reference each other by their index in
//! their respective arrays in [`Template`]. When such indices are used, the documentation
//! will specify which array the index refers to.
//!
//! # Object Templates
//!
//! The [`objects`] field of [`Template`] provides a flattened, type-erased list of all objects
//! defined in the template. Each type of object provides its type-specific data in that type's
//! array, and then specifies the index of an [`ObjectTemplate`] in [`objects`]. Every object
//! in the template must be represented in [`objects`] exactly once.
//!
//! The full, flattened list of objects is primarily used by [`AnimationTemplate`] to allow
//! tracks in the animation to reference the object targeted by the track regardless of the
//! target object's concrete type.
//!
//! # Animations
//!
//! Templates can also describe animations that apply to the objects in a template.
//! When instantiated, the resulting animation clips will be unique to that instance of of the
//! template. This allows for instances of the template to be animated independently of each
//! other, without requiring you to manually setup animations for each instance.
//!
//! An animation in a template can target any of the objects described in the template. It does
//! this by specifying the index of the objects in [`objects`]. See
//! [`AnimationTemplate::tracks`] for more information.
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
//! [`ObjectTemplate`]: ./struct.ObjectTemplate.html
//! [`AnimationTemplate`]: ./struct.AnimationTemplate.html
//! [`AnimationTemplate::tracks`]: ./struct.AnimationTemplate.html#structfield.tracks
//! [`nodes`]: ./struct.Template.html#structfield.nodes
//! [`cameras`]: ./struct.Template.html#structfield.cameras
//! [`meshes`]: ./struct.Template.html#structfield.meshes
//! [`roots`]: ./struct.Template.html#structfield.roots
//! [`objects`]: ./struct.Template.html#structfield.objects
//! [`InstancedGeometry`]: ./struct.InstancedGeometry.html

use animation::Track;
use camera::Projection;
use color::Color;
use material::Material;
use node::Transform;
use render::GpuData;
use skeleton::InverseBindMatrix;

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

    /// The base object data for all objects defined in the template.
    ///
    /// The index into this array is used to uniquely identify each object in the template. Each
    /// object, regardless of its concrete type, will be represented in this array exactly once.
    /// These indices are primarily used in [`AnimationTemplate`] to define the target of each
    /// track of the animation.
    ///
    /// [`AnimationTemplate`]: ./struct.AnimationTemplate.html
    pub objects: Vec<ObjectTemplate>,

    /// Definitions for all [`Group`] objects in the template, given as indices into [`objects`].
    ///
    /// Groups carry no data beyond the common object data, so groups are defined soley by their
    /// [`ObjectTemplate`].
    ///
    /// [`objects`]: #structfield.objects
    /// [`Group`]: ../struct.Group.html
    /// [`ObjectTemplate`]: ./struct.ObjectTemplate.html
    pub groups: Vec<usize>,

    /// Projection data used by cameras defined in the template.
    pub cameras: Vec<CameraTemplate>,

    /// The meshes defined in this template.
    pub meshes: Vec<MeshTemplate>,

    /// Data for the lights described by this template.
    pub lights: Vec<LightTemplate>,

    /// Data for the bones described by this template.
    pub bones: Vec<BoneTemplate>,

    /// Definitions for all [`Skeleton`] objects in the template, given as indices into
    /// [`objects`].
    ///
    /// Skeletons carry no data beyond the common object data, so groups are defined soley by
    /// their [`ObjectTemplate`].
    ///
    /// [`objects`]: #structfield.objects
    /// [`Skeleton`]: ../skeleton/struct.Skeleton.html
    /// [`ObjectTemplate`]: ./struct.ObjectTemplate.html
    pub skeletons: Vec<usize>,

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
    pub fn new() -> Template {
        Default::default()
    }
}

/// Common data used by all object types.
///
/// All objects (i.e. three-rs types that implement the [`Object`] trait) have common data
/// that the user can set at runtime. `ObjectTemplate` encapsultes these fields, and the
/// various template types have a way to reference an `ObjectTemplate` to specify the object
/// data for that template.
///
/// See the [module documentation] for more information on how object data is defined in
/// templates.
///
/// [`Object`]: ../trait.Object.html
/// [module documentation]: ./index.html#object-templates
#[derive(Debug, Clone, Default)]
pub struct ObjectTemplate {
    /// An optional name for the object.
    pub name: Option<String>,

    /// The parent [`Group`] of the object, given as an index into the [`groups`] array of the
    /// parent [`Template`].
    ///
    /// If `parent` is `None`, then the object is added to the root [`Group`] returned from
    /// [`Factory::instantiate_template`].
    ///
    /// [`Group`]: ../struct.Group.html
    /// [`Template`]: ./struct.Template.html
    pub parent: Option<usize>,

    /// The local transform for the object.
    pub transform: Transform,
}

impl ObjectTemplate {
    /// Creates a new `ObjectTemplate` with default values.
    ///
    /// The new object template will have no name, no parent (i.e. it will be treated as a root
    /// object of the template), and a default transform.
    ///
    /// # Examples
    ///
    /// ```
    /// use three::template::{ObjectTemplate, Template};
    ///
    /// let mut template = Template::new();
    ///
    /// let mut object = ObjectTemplate::new();
    /// object.name = Some("My Node".into());
    /// object.transform.position = [1.0, 2.0, 3.0].into();
    ///
    /// template.objects.push(object);
    /// ```
    pub fn new() -> ObjectTemplate {
        Default::default()
    }
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
    /// The object data for the mesh, given as an index in the [`objects`] array of the parent
    /// [`Template`].
    ///
    /// [`Template`]: ./struct.Template.html
    /// [`objects`]: ./struct.Template.html#structfield.objects
    pub object: usize,

    /// The geometry used in the mesh.
    pub geometry: InstancedGeometry,

    /// The index of the material for the mesh in the [`meshes`] array of the parent [`Template`].
    ///
    /// [`Template`]: ./struct.Template.html
    /// [`meshes`]: ./struct.Template.html#structfield.meshes
    pub material: Material,

    /// The skeleton used to render the mesh, if it's a skinned mesh.
    pub skeleton: Option<usize>,
}

/// A template for a [`Camera`] object.
///
/// [`Camera`]: ../struct.Camera.html
#[derive(Debug, Clone)]
pub struct CameraTemplate {
    /// The object data for the camera, given as an index in the [`objects`] array of the parent
    /// [`Template`].
    ///
    /// [`Template`]: ./struct.Template.html
    /// [`objects`]: ./struct.Template.html#structfield.objects
    pub object: usize,

    /// The projection used by the camera.
    pub projection: Projection,
}

/// A template for a [`Bone`] object.
///
/// For more information about creating a [`Bone`], see [`Factory::bone`].
///
/// [`Bone`]: ../skeleton/struct.Bone.html
/// [`Factory::bone`]: ../struct.Factory.html#method.bone
#[derive(Debug, Clone)]
pub struct BoneTemplate {
    /// The object data for the bone, given as an index in the [`objects`] array of the parent
    /// [`Template`].
    ///
    /// [`Template`]: ./struct.Template.html
    /// [`objects`]: ./struct.Template.html#structfield.objects
    pub object: usize,

    /// The index of the bone within its skeleton.
    pub index: usize,

    /// The inverse bind matrix used to bind vertices of the mesh to the bone.
    pub inverse_bind_matrix: InverseBindMatrix,

    /// The skeleton that this bone is a part of, given as an index into the [`skeletons`]
    /// array of the parent [`Template`].
    ///
    /// [`Template`]: ./struct.Template.html
    /// [`skeletons`]: ./struct.Template.html#structfield.skeletons
    pub skeleton: usize,
}

/// The definition for an animation targeting objects in a [`Template`].
///
/// See the [module documentation] for more information on template animations and how they
/// are used.
///
/// [`Template`]: ./struct.Template.html
/// [module documentation]: ./index.html#animations
#[derive(Debug, Clone)]
pub struct AnimationTemplate {
    /// An optional name for the animation.
    pub name: Option<String>,

    /// The tracks making up the animation.
    ///
    /// Each track is composed of a [`Track`], containing the data for the track, and the node
    /// that the track targetes, specified as an index into the [`objects`] array of the
    /// parent [`Template`].
    ///
    /// [`Track`]: ../animation/struct.Track.html
    /// [`Template`]: ./struct.Template.html
    /// [`objects`]: ./struct.Template.html#structfield.nodes
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
    /// The object data for the light, given as an index into the [`objects`] array of the parent
    /// [`Template`].
    ///
    /// [`Template`]: ./struct.Template.html
    /// [`objects`]: ./struct.Template.html#structfield.objects
    pub object: usize,

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
    /// use three::template::{LightTemplate, ObjectTemplate, Template};
    ///
    /// let mut template = Template::new();
    /// template.objects.push(ObjectTemplate::new());
    /// let light = LightTemplate::ambient(
    ///     template.objects.len() - 1,
    ///     three::color::RED,
    ///     0.5,
    /// );
    /// template.lights.push(light);
    /// ```
    ///
    /// [`Factory::ambient_light`]: ../struct.Factory.html#method.ambient_light
    pub fn ambient(object: usize, color: Color, intensity: f32) -> LightTemplate {
        LightTemplate {
            object,
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
    /// use three::template::{LightTemplate, ObjectTemplate, Template};
    ///
    /// let mut template = Template::new();
    /// template.objects.push(ObjectTemplate::new());
    /// let light = LightTemplate::directional(
    ///     template.objects.len() - 1,
    ///     three::color::RED,
    ///     0.5,
    /// );
    /// template.lights.push(light);
    /// ```
    ///
    /// [`Factory::directional_light`]: ../struct.Factory.html#method.directional_light
    pub fn directional(object: usize, color: Color, intensity: f32) -> LightTemplate {
        LightTemplate {
            object,
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
    /// use three::template::{LightTemplate, ObjectTemplate, Template};
    ///
    /// let mut template = Template::new();
    /// template.objects.push(ObjectTemplate::new());
    /// let light = LightTemplate::point(
    ///     template.objects.len() - 1,
    ///     three::color::RED,
    ///     0.5,
    /// );
    /// template.lights.push(light);
    /// ```
    ///
    /// [`Factory::point_light`]: ../struct.Factory.html#method.point_light
    pub fn point(object: usize, color: Color, intensity: f32) -> LightTemplate {
        LightTemplate {
            object,
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
    /// use three::template::{LightTemplate, ObjectTemplate, Template};
    ///
    /// let mut template = Template::new();
    /// template.objects.push(ObjectTemplate::new());
    /// let light = LightTemplate::hemisphere(
    ///     template.objects.len() - 1,
    ///     three::color::RED,
    ///     three::color::BLUE,
    ///     0.5,
    /// );
    /// template.lights.push(light);
    /// ```
    ///
    /// [`Factory::hemisphere_light`]: ../struct.Factory.html#method.hemisphere_light
    pub fn hemisphere(
        object: usize,
        sky_color: Color,
        ground_color: Color,
        intensity: f32,
    ) -> LightTemplate {
        LightTemplate {
            object,
            color: sky_color,
            intensity,
            sub_light: SubLightTemplate::Hemisphere {
                ground: ground_color,
            },
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
