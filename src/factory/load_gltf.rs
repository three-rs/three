//! `glTF` loading sub-module.
//!
//! ### Implementation Notes
//!
//! * The order of function declarations matches the order of usage.
//! * The entry point is `Factory::load_gltf`, at the end of the file.

use animation;
use color;
use geometry;
use gltf;
use gltf_importer;
use image;
use material;
use mint;
use std::{fs, io};
use std::collections::HashMap;

use camera::{Camera, Orthographic, Perspective, Projection};
use gltf_utils::AccessorIter;
use skeleton::Skeleton;
use std::path::{Path, PathBuf};

use animation::{Clip, Track};
use {Group, Material, Mesh, Texture};
use geometry::{Geometry, Shape};
use object::{self, Object};
use super::Factory;

/// A glTF scene that has been instantiated and can be added to a [`Scene`].
///
/// Created by instantiating a scene defined in a [`GltfDefinitions`] with
/// [`Factory::instantiate_gltf_scene`]. A `GltfScene` can be added to a [`Scene`] with
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
pub struct GltfScene {
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
    pub nodes: HashMap<usize, GltfNode>,
}

impl GltfScene {
    /// Finds the first node in the scene with the specified name, using a [`GltfDefinitions`]
    /// to lookup the name for each node.
    ///
    /// Name matching is case-sensitive. Returns the first node with a matching name, otherwise
    /// returns `None`.
    pub fn find_node_by_name(
        &self,
        name: &str,
        definitions: &GltfDefinitions,
    ) -> Option<&GltfNode> {
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

impl AsRef<object::Base> for GltfScene {
    fn as_ref(&self) -> &object::Base {
        self.group.as_ref()
    }
}

impl object::Object for GltfScene {}

/// A node in a scene from a glTF file that has been instantiated.
#[derive(Debug, Clone)]
pub struct GltfNode {
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

impl AsRef<object::Base> for GltfNode {
    fn as_ref(&self) -> &object::Base {
        self.group.as_ref()
    }
}

impl object::Object for GltfNode {}

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
    pub nodes: Vec<GltfNodeDefinition>,

    /// The scenes described in the glTF file.
    pub scenes: Vec<GltfSceneDefinition>,

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
pub struct GltfNodeDefinition {
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
pub struct GltfSceneDefinition {
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

fn load_camera<'a>(
    entry: gltf::Camera<'a>,
) -> Projection {
    match entry.projection() {
        gltf::camera::Projection::Orthographic(values) => {
            let center = mint::Point2::<f32>::from([0.0, 0.0]);
            let extent_y = values.ymag();
            let range = values.znear() .. values.zfar();
            Projection::Orthographic(Orthographic { center, extent_y, range })
        }

        gltf::camera::Projection::Perspective(values) => {
            let fov_y = values.yfov().to_degrees();
            let near = values.znear();
            let zrange = match values.zfar() {
                Some(far) => (near .. far).into(),
                None => (near ..).into(),
            };
            Projection::Perspective(Perspective { fov_y, zrange })
        }
    }
}

fn load_textures(
    factory: &mut Factory,
    gltf: &gltf::Gltf,
    base: &Path,
    buffers: &gltf_importer::Buffers,
) -> Vec<Texture<[f32; 4]>> {
    let mut textures = Vec::new();
    for texture in gltf.textures() {
        use image::ImageFormat::{JPEG as Jpeg, PNG as Png};
        let image = match texture.source().data() {
            gltf::image::Data::View { view, mime_type } => {
                let format = match mime_type {
                    "image/png" => Png,
                    "image/jpeg" => Jpeg,
                    _ => unreachable!(),
                };
                let data = buffers.view(&view).unwrap();
                if data.starts_with(b"data:") {
                    // Data URI decoding not yet implemented
                    unimplemented!()
                } else {
                    image::load_from_memory_with_format(&data, format)
                        .unwrap()
                        .to_rgba()
                }
            }
            gltf::image::Data::Uri { uri, mime_type } => {
                let path: PathBuf = base.join(uri);
                if let Some(ty) = mime_type {
                    let format = match ty {
                        "image/png" => Png,
                        "image/jpeg" => Jpeg,
                        _ => unreachable!(),
                    };
                    let file = fs::File::open(&path).unwrap();
                    let reader = io::BufReader::new(file);
                    image::load(reader, format).unwrap().to_rgba()
                } else {
                    image::open(&path).unwrap().to_rgba()
                }
            }
        };
        let (width, height) = (image.width() as u16, image.height() as u16);
        use {FilterMethod, WrapMode};
        use gltf::texture::{MagFilter, WrappingMode};
        let params = texture.sampler();
        // gfx does not support separate min / mag
        // filters yet, so for now we'll use `mag_filter` for both.
        let mag_filter = match params.mag_filter() {
            None | Some(MagFilter::Nearest) => FilterMethod::Scale,
            Some(MagFilter::Linear) => FilterMethod::Bilinear,
        };
        let wrap_s = match params.wrap_s() {
            WrappingMode::ClampToEdge => WrapMode::Clamp,
            WrappingMode::MirroredRepeat => WrapMode::Mirror,
            WrappingMode::Repeat => WrapMode::Tile,
        };
        let wrap_t = match params.wrap_t() {
            WrappingMode::ClampToEdge => WrapMode::Clamp,
            WrappingMode::MirroredRepeat => WrapMode::Mirror,
            WrappingMode::Repeat => WrapMode::Tile,
        };
        let sampler = factory.sampler(mag_filter, wrap_s, wrap_t);
        let texture = factory.load_texture_from_memory(width, height, &image, sampler);
        textures.push(texture);
    }
    textures
}

fn load_material<'a>(
    mat: gltf::Material<'a>,
    textures: &[Texture<[f32; 4]>],
) -> Material {
    let pbr = mat.pbr_metallic_roughness();
    let mut is_basic_material = true;
    let base_color_map = pbr.base_color_texture()
        .map(|t| textures[t.as_ref().index()].clone());
    let normal_map = mat.normal_texture().map(|t| {
        is_basic_material = false;
        textures[t.as_ref().index()].clone()
    });
    let emissive_map = mat.emissive_texture().map(|t| {
        is_basic_material = false;
        textures[t.as_ref().index()].clone()
    });
    let metallic_roughness_map = pbr.metallic_roughness_texture().map(|t| {
        is_basic_material = false;
        textures[t.as_ref().index()].clone()
    });
    let occlusion_map = mat.occlusion_texture().map(|t| {
        is_basic_material = false;
        textures[t.as_ref().index()].clone()
    });
    let (base_color_factor, base_color_alpha) = {
        let x = pbr.base_color_factor();
        (color::from_linear_rgb([x[0], x[1], x[2]]), x[3])
    };

    if false {// is_basic_material {
        material::Basic {
            color: base_color_factor,
            map: base_color_map,
        }.into()
    } else {
        material::Pbr {
            base_color_factor,
            base_color_alpha,
            metallic_factor: pbr.metallic_factor(),
            roughness_factor: pbr.roughness_factor(),
            occlusion_strength: mat.occlusion_texture().map_or(1.0, |t| {
                t.strength()
            }),
            emissive_factor: color::from_linear_rgb(mat.emissive_factor()),
            normal_scale: mat.normal_texture().map_or(1.0, |t| {
                t.scale()
            }),
            base_color_map,
            normal_map,
            emissive_map,
            metallic_roughness_map,
            occlusion_map,
        }.into()
    }
}

/// ### Implementation Notes
///
/// * Multiple 'sub-meshes' are returned since the concept of a
///   mesh is different in `glTF` to `three`.
/// * A `glTF` mesh consists of one or more _primitives_, which are
///   equivalent to `three` meshes.
fn load_mesh<'a>(
    mesh: gltf::Mesh<'a>,
    buffers: &gltf_importer::Buffers,
) -> GltfMeshDefinition {
    let name = mesh.name().map(Into::into);
    let primitives = mesh
        .primitives()
        .map(|prim| load_primitive(prim, buffers))
        .collect();
    GltfMeshDefinition {
        name,
        primitives
    }
}

fn load_primitive<'a>(
    primitive: gltf::Primitive<'a>,
    buffers: &gltf_importer::Buffers,
) -> GltfPrimitiveDefinition {
    use gltf_utils::PrimitiveIterators;
    use itertools::Itertools;

    fn load_morph_targets(
        primitive: &gltf::Primitive,
        buffers: &gltf_importer::Buffers,
    ) -> Vec<Shape> {
        primitive
            .morph_targets()
            .map(|target| {
                let mut shape = Shape::default();
                if let Some(accessor) = target.positions() {
                    let iter = AccessorIter::<[f32; 3]>::new(accessor, buffers);
                    shape.vertices.extend(iter.map(mint::Point3::<f32>::from));
                }
                if let Some(accessor) = target.normals() {
                    let iter = AccessorIter::<[f32; 3]>::new(accessor, buffers);
                    shape.normals.extend(iter.map(mint::Vector3::<f32>::from));
                }
                if let Some(accessor) = target.tangents() {
                    let iter = AccessorIter::<[f32; 3]>::new(accessor, buffers);
                    shape.tangents.extend(iter.map(|v| mint::Vector4{ x: v[0], y: v[1], z: v[2], w: 1.0 }));
                }
                shape
            })
            .collect()
    }

    let mut faces = vec![];
    if let Some(iter) = primitive.indices_u32(buffers) {
        faces.extend(iter.tuples().map(|(a, b, c)| [a, b, c]));
    }
    let vertices: Vec<mint::Point3<f32>> = primitive
        .positions(buffers)
        .unwrap()
        .map(|x| x.into())
        .collect();
    let normals = if let Some(iter) = primitive.normals(buffers) {
        iter.map(|x| x.into()).collect()
    } else {
        Vec::new()
    };
    let tangents = if let Some(iter) = primitive.tangents(buffers) {
        iter.map(|x| x.into()).collect()
    } else {
        Vec::new()
    };
    let tex_coords = if let Some(iter) = primitive.tex_coords_f32(0, buffers) {
        iter.map(|x| x.into()).collect()
    } else {
        Vec::new()
    };
    let joint_indices = if let Some(iter) = primitive.joints_u16(0, buffers) {
        iter.map(|x| [x[0] as i32, x[1] as i32, x[2] as i32, x[3] as i32]).collect()
    } else {
        Vec::new()
    };
    let joint_weights = if let Some(iter) = primitive.weights_f32(0, buffers) {
        iter.collect()
    } else {
        Vec::new()
    };
    let shapes = load_morph_targets(&primitive, &buffers);
    let geometry = Geometry {
        base: Shape {
            vertices,
            normals,
            tangents,
        },
        tex_coords,
        faces,
        shapes,
        joints: geometry::Joints {
            indices: joint_indices,
            weights: joint_weights,
        },
    };

    let material = primitive.material().index();

    GltfPrimitiveDefinition {
        geometry,
        material,
    }
}

fn load_skin<'a>(
    skin: gltf::Skin<'a>,
    buffers: &gltf_importer::Buffers,
) -> GltfSkinDefinition {
    use std::iter::repeat;

    let mut ibms = Vec::new();
    let mut bones = Vec::new();
    if let Some(accessor) = skin.inverse_bind_matrices() {
        for ibm in AccessorIter::<[[f32; 4]; 4]>::new(accessor, buffers) {
            ibms.push(ibm.into());
        }
    }
    let mx_id = mint::ColumnMatrix4::from([
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]);
    let ibm_iter = ibms.
        into_iter().
        chain(repeat(mx_id));
    for (joint, inverse_bind_matrix) in skin.joints().zip(ibm_iter) {
        let joint = joint.index();
        bones.push(GltfBoneDefinition {
            inverse_bind_matrix,
            joint,
        });
    }

    GltfSkinDefinition {
        bones,
    }
}

fn load_animation<'a>(
    animation: gltf::Animation<'a>,
    buffers: &gltf_importer::Buffers,
) -> GltfAnimationDefinition {
    use gltf::animation::InterpolationAlgorithm::*;

    let mut tracks = Vec::new();
    let name = animation.name().map(str::to_string);
    for channel in animation.channels() {
        let sampler = channel.sampler();
        let target = channel.target();
        let node = target.node();
        let input = sampler.input();
        let output = sampler.output();
        let interpolation = match sampler.interpolation() {
            Linear => animation::Interpolation::Linear,
            Step => animation::Interpolation::Discrete,
            CubicSpline => animation::Interpolation::Cubic,
            CatmullRomSpline => animation::Interpolation::Cubic,
        };
        use animation::{Binding, Track, Values};
        let times: Vec<f32> = AccessorIter::new(input, buffers).collect();
        let (binding, values) = match target.path() {
            gltf::animation::TrsProperty::Translation => {
                let values = AccessorIter::<[f32; 3]>::new(output, buffers)
                    .map(|v| mint::Vector3::from(v))
                    .collect::<Vec<_>>();
                assert_eq!(values.len(), times.len());
                (Binding::Position, Values::Vector3(values))
            }
            gltf::animation::TrsProperty::Rotation => {
                let values = AccessorIter::<[f32; 4]>::new(output, buffers)
                    .map(|r| mint::Quaternion::from(r))
                    .collect::<Vec<_>>();
                assert_eq!(values.len(), times.len());
                (Binding::Orientation, Values::Quaternion(values))
            }
            gltf::animation::TrsProperty::Scale => {
                // TODO: Groups do not handle non-uniform scaling, so for now
                // we'll choose Y to be the scale factor in all directions.
                let values = AccessorIter::<[f32; 3]>::new(output, buffers)
                    .map(|s| s[1])
                    .collect::<Vec<_>>();
                assert_eq!(values.len(), times.len());
                (Binding::Scale, Values::Scalar(values))
            }
            gltf::animation::TrsProperty::Weights => {
                // Write all values for target[0] first, then all values for target[1], etc.
                let num_targets = node
                    .mesh()
                    .unwrap()
                    .primitives()
                    .next()
                    .unwrap()
                    .morph_targets()
                    .len();
                let mut values = vec![0.0; times.len() * num_targets];
                let raw = AccessorIter::<f32>::new(output, buffers).collect::<Vec<_>>();
                for (i, chunk) in raw.chunks(num_targets).enumerate() {
                    for (j, value) in chunk.iter().enumerate() {
                        values[j * times.len() + i] = *value;
                    }
                }
                (Binding::Weights, Values::Scalar(values))
            }
        };
        tracks.push((
            Track {
                binding,
                interpolation,
                times,
                values,
            },
            node.index(),
        ));
    }

    GltfAnimationDefinition {
        name,
        tracks,
    }
}

fn load_scene<'a>(scene: gltf::Scene<'a>) -> GltfSceneDefinition {
    let roots = scene.nodes().map(|node| node.index()).collect();

    GltfSceneDefinition {
        name: scene.name().map(Into::into),
        roots,
    }
}

fn load_node<'a>(node: gltf::Node<'a>) -> GltfNodeDefinition {
    let name = node.name().map(Into::into);

    let mesh = node.mesh().map(|mesh| mesh.index());
    let camera = node.camera().map(|camera| camera.index());
    let skin = node.skin().map(|skin| skin.index());
    let children = node.children().map(|node| node.index()).collect();

    // Decompose the transform to get the translation, rotation, and scale.
    let (translation, rotation, scale) = node.transform().decomposed();

    // TODO: Groups do not handle non-uniform scaling, so for now we'll choose Y to be the
    // scale factor in all directions.
    let scale = scale[1];

    GltfNodeDefinition {
        name,

        mesh,
        skin,
        camera,
        children,

        translation: translation.into(),
        rotation: rotation.into(),
        scale,
    }
}

fn instantiate_node_hierarchy(
    factory: &mut Factory,
    gltf: &GltfDefinitions,
    nodes: &mut HashMap<usize, GltfNode>,
    parent: &Group,
    node_index: usize,
) {
    let node = &gltf.nodes[node_index];

    let group = factory.group();
    parent.add(&group);

    // Apply the node's transformations to the root group of the node.
    group.set_position(node.translation);
    group.set_scale(node.scale);
    group.set_orientation(node.rotation);

    let mut meshes = Vec::new();
    let mut camera = None;
    let children = node.children.clone();

    // If the node has a mesh associated with it, instantiate each of the primitives as a mesh.
    if let Some(mesh_index) = node.mesh {
        let gltf_mesh = &gltf.meshes[mesh_index];
        for primitive in &gltf_mesh.primitives {
            let material = primitive
                .material
                .map(|index| gltf.materials[index].clone())
                .unwrap_or(material::Basic {
                    color: 0xFFFFFF,
                    map: None,
                }.into());
            let mesh = factory.mesh(primitive.geometry.clone(), material);
            group.add(&mesh);
            meshes.push(mesh);
        }
    }

    // If the node has a camera associated with it, create a camera instance.
    if let Some(camera_index) = node.camera {
        let projection = gltf.cameras[camera_index].clone();
        let instance = match projection {
            Projection::Perspective(Perspective { fov_y, zrange }) => {
                factory.perspective_camera(fov_y, zrange)
            }

            Projection::Orthographic(Orthographic { center, extent_y, range }) => {
                factory.orthographic_camera(center, extent_y, range)
            }
        };

        // Add the camera to the group that represents the node.
        group.add(&instance);

        camera = Some(instance);
    }

    // Recursively instantiate the node's children.
    for &child_index in &node.children {
        instantiate_node_hierarchy(factory, gltf, nodes, &group, child_index);
    }

    let instance = GltfNode {
        group,
        meshes,
        skeleton: None,
        camera,
        children,
    };
    nodes.insert(node_index, instance);
}

impl super::Factory {
    /// Loads definitions from a glTF 2.0 file.
    ///
    /// The returned [`GltfDefinitions`] cannot be added to the scene directly, rather it
    /// contains definitions for meshes, node hierarchies, skinned skeletons, animations, and
    /// other things that can be instantiated and added to the scene. See the [`GltfDefinitions`]
    /// for more information on how to instantiate the various definitions in the glTF file.
    pub fn load_gltf(
        &mut self,
        path_str: &str,
    ) -> GltfDefinitions {
        info!("Loading {}", path_str);

        let path = Path::new(path_str);
        let base = path.parent().unwrap_or(&Path::new(""));
        let (gltf, buffers) = gltf_importer::import(path).expect("invalid glTF 2.0");

        let cameras = gltf.cameras().map(load_camera).collect();

        let textures = load_textures(self, &gltf, base, &buffers);
        let materials: Vec<_> = gltf
            .materials()
            .map(|material| load_material(material, &textures))
            .collect();

        let meshes: Vec<_> = gltf
            .meshes()
            .map(|mesh| load_mesh(mesh, &buffers))
            .collect();

        let nodes = gltf.nodes().map(load_node).collect();
        let scenes = gltf.scenes().map(load_scene).collect();
        let default_scene = gltf.default_scene().map(|scene| scene.index());

        let animations = gltf
            .animations()
            .map(|anim| load_animation(anim, &buffers))
            .collect();

        let skins = gltf
            .skins()
            .map(|skin| load_skin(skin, &buffers))
            .collect();

        GltfDefinitions {
            materials,
            cameras,
            meshes,
            scenes,
            default_scene,
            nodes,
            skins,
            animations,
            textures,
        }
    }

    /// Instantiates the contents of a scene defined in a glTF file.
    pub fn instantiate_gltf_scene(
        &mut self,
        definitions: &GltfDefinitions,
        scene: usize,
    ) -> GltfScene {
        // Get the scene definition.
        //
        // NOTE: We use `get` here (instead of indexing into the scenes list normally) so that
        // we can panic with a more meaningful error message if the scene index is invalid.
        let scene = definitions.scenes.get(scene).expect("Invalid scene index");

        let group = self.group();
        let roots = scene.roots.clone();

        // Instantiate the node hiercharies beginning with each of the root nodes.
        let mut nodes = HashMap::new();
        for &node_index in &scene.roots {
            instantiate_node_hierarchy(
                self,
                definitions,
                &mut nodes,
                &group,
                node_index,
            );
        }

        // Instantiate the skeletons.
        {
            for (node_index, node_def) in definitions.nodes.iter().enumerate() {
                // Ignore any nodes that aren't in the scene.
                if !nodes.contains_key(&node_index) { continue; }

                let skin_index = match node_def.skin {
                    Some(index) => index,
                    None => continue,
                };

                let skin = &definitions.skins[skin_index];
                let mut bones = Vec::with_capacity(skin.bones.len());
                for (bone_index, bone_def) in skin.bones.iter().enumerate() {
                    // Instantiate the bone and add it to the corresponding node in the scene.
                    let bone = self.bone(bone_index, bone_def.inverse_bind_matrix);
                    nodes[&bone_def.joint].group.add(&bone);

                    bones.push(bone);
                }

                // Create the skeleton from the bones.
                let skeleton = self.skeleton(bones);

                // Get the node and attach the skeleton to it.
                let node = nodes.get_mut(&node_index).unwrap();
                node.group.add(&skeleton);

                // Set the skeleton for all the meshes on the node.
                for mesh in &mut node.meshes {
                    mesh.set_skeleton(skeleton.clone());
                }

                node.skeleton = Some(skeleton);
            }
        }

        GltfScene {
            group,
            nodes,
            roots,
        }
    }

    /// Instantiate an animation from a glTF file and apply it to the contents of a glTF scene.
    ///
    /// Returns a [`Clip`] for the animation if it was successfully instantiated. If the the
    /// animation references a node that is not in `scene`, then `Err(())` is returned.
    pub fn instantiate_gltf_animation(
        &mut self,
        scene: &GltfScene,
        anim_def: &GltfAnimationDefinition,
    ) -> Result<Clip, ()> {
        // Apply each track in the animation definition to its target node in the scene.
        let mut tracks = Vec::with_capacity(anim_def.tracks.len());
        for &(ref track, target_index) in &anim_def.tracks {
            match scene.nodes.get(&target_index) {
                Some(node) => {
                    let target = node.upcast();
                    tracks.push((track.clone(), target));
                }

                None => return Err(()),
            }
        }

        Ok(Clip {
            name: anim_def.name.clone(),
            tracks,
        })
    }
}
