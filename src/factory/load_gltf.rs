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

use camera::{Camera, Orthographic, Perspective, Projection};
use gltf_utils::AccessorIter;
use object::Object;
use skeleton::Skeleton;
use std::path::{Path, PathBuf};
use vec_map::VecMap;

use animation::Clip;
use {Group, Material, Mesh, Texture};
use geometry::{Geometry, Shape};
use object;
use super::Factory;

/// Loaded glTF 2.0 returned by [`Factory::load_gltf`].
///
/// [`Factory::load_gltf`]: struct.Factory.html#method.load_gltf
#[derive(Debug, Clone)]
pub struct Gltf {
    /// Imported camera views.
    pub cameras: Vec<Camera>,

    /// Imported animation clips.
    pub clips: Vec<Clip>,

    /// The node heirarchy of the default scene.
    ///
    /// If the `glTF` contained no default scene then this
    /// container will be empty.
    pub heirarchy: VecMap<object::Group>,

    /// Imported mesh instances.
    ///
    /// ### Notes
    ///
    /// * Must be kept alive in order to be displayed.
    pub instances: Vec<Mesh>,

    /// Imported mesh materials.
    pub materials: Vec<Material>,

    /// Imported mesh templates.
    pub meshes: VecMap<Vec<Mesh>>,

    /// The root node of the default scene.
    ///
    /// If the `glTF` contained no default scene then this group
    /// will have no children.
    pub root: object::Group,

    /// Imported skeletons.
    pub skeletons: Vec<Skeleton>,

    /// Imported textures.
    pub textures: Vec<Texture<[f32; 4]>>,
}

impl AsRef<object::Base> for Gltf {
    fn as_ref(&self) -> &object::Base {
        self.root.as_ref()
    }
}

impl object::Object for Gltf {}

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

    /// The scenes described from the glTF file.
    pub scenes: Vec<GltfSceneDefinition>,
}

/// A template for a glTF mesh instance.
///
/// Note that a glTF mesh doesn't map directly to three's concept of a [`Mesh`] (see
/// [`GltfPrimitiveDefinition`] for a more direct analogy). Rather, `GltfMeshDefinition` can be instantated
/// into a [`Group`] with one or more [`Mesh`] instances added to the group.
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
    /// parent [`GltfData`].
    pub mesh: Option<usize>,

    /// The index of the camera associated with this node, if any.
    ///
    /// The index can be used to lookup the associated camera projection in the `cameras` map of
    /// the parent [`GltfData`].
    pub camera: Option<usize>,

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

fn build_scene_hierarchy(
    factory: &mut Factory,
    gltf: &gltf::Gltf,
    scene: &gltf::Scene,
    root: &Group,
    heirarchy: &mut VecMap<Group>,
) {
    struct Item<'a> {
        group: Group,
        node: gltf::Node<'a>,
    }

    let mut stack = Vec::with_capacity(gltf.nodes().len());

    for node in scene.nodes() {
        let group = factory.group();
        root.add(&group);
        stack.push(Item { group, node });
    }

    while let Some(Item { group, node }) = stack.pop() {
        for child_node in node.children() {
            let child_group = factory.group();
            group.add(&child_group);
            stack.push(Item {
                group: child_group,
                node: gltf.nodes().nth(child_node.index()).unwrap()
            });
        }
        heirarchy.insert(node.index(), group);
    }
}

fn load_cameras(
    factory: &mut Factory,
    gltf: &gltf::Gltf,
) -> Vec<Camera> {
    let mut cameras = Vec::new();
    for entry in gltf.cameras() {
        match entry.projection() {
            gltf::camera::Projection::Orthographic(values) => {
                let center = mint::Point2::<f32>::from([0.0, 0.0]);
                let yextent = values.ymag();
                let range = values.znear() .. values.zfar();
                let camera = factory.orthographic_camera(center, yextent, range);
                cameras.push(camera);
            }
            gltf::camera::Projection::Perspective(values) => {
                let yfov = values.yfov().to_degrees();
                let near = values.znear();
                let camera = if let Some(far) = values.zfar() {
                    factory.perspective_camera(yfov, near .. far)
                } else {
                    factory.perspective_camera(yfov, near ..)
                };
                cameras.push(camera);
            }
        }
    }
    cameras
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

fn load_materials(
    gltf: &gltf::Gltf,
    textures: &[Texture<[f32; 4]>],
) -> Vec<Material> {
    let mut materials = Vec::new();
    for mat in gltf.materials() {
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
        let material = if false {// is_basic_material {
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
        };
        materials.push(material);
    }
    materials
}

/// ### Implementation Notes
///
/// * Multiple 'sub-meshes' are returned since the concept of a
///   mesh is different in `glTF` to `three`.
/// * A `glTF` mesh consists of one or more _primitives_, which are
///   equivalent to `three` meshes.
fn load_meshes(
    factory: &mut Factory,
    gltf: &gltf::Gltf,
    materials: &[Material],
    buffers: &gltf_importer::Buffers,
) -> VecMap<Vec<Mesh>> {
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

    let mut meshes = VecMap::new();
    for mesh in gltf.meshes() {
        let mut primitives = Vec::new();
        for primitive in mesh.primitives() {
            use gltf_utils::PrimitiveIterators;
            use itertools::Itertools;
            let mut faces = vec![];
            if let Some(mut iter) = primitive.indices_u32(buffers) {
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
            let material = if let Some(index) = primitive.material().index() {
                materials[index].clone()
            } else {
                material::Basic {
                    color: 0xFFFFFF,
                    map: None,
                }.into()
            };
            let mesh = factory.mesh(geometry, material);
            primitives.push(mesh);
        }
        meshes.insert(mesh.index(), primitives);
    }
    meshes
}

fn load_skeletons(
    factory: &mut Factory,
    gltf: &gltf::Gltf,
    heirarchy: &VecMap<Group>,
    buffers: &gltf_importer::Buffers,
) -> Vec<Skeleton> {
    use std::iter::repeat;

    let mut skeletons = Vec::new();
    for skin in gltf.skins() {
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
        for (joint, ibm) in skin.joints().zip(ibm_iter) {
            let bone = factory.bone(bones.len(), ibm);
            heirarchy[&joint.index()].add(&bone);
            bones.push(bone);
        }
        let skeleton = factory.skeleton(bones);
        skeletons.push(skeleton);
    }
    skeletons
}

fn load_clips(
    gltf: &gltf::Gltf,
    heirarchy: &VecMap<Group>,
    buffers: &gltf_importer::Buffers,
) -> Vec<animation::Clip> {
    use gltf::animation::InterpolationAlgorithm::*;
    let mut clips = Vec::new();
    for animation in gltf.animations() {
        let mut tracks = Vec::new();
        let name = animation.name().map(str::to_string);
        for channel in animation.channels() {
            let sampler = channel.sampler();
            let target = channel.target();
            let node = target.node();
            let object = match heirarchy.get(node.index()) {
                Some(object) => object.upcast(),
                // This animation does not correspond to any loaded node.
                None => continue,
            };
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
                    // write all values for target[0] first, then all values for target[1], etc
                    let num_targets = node.mesh().unwrap().primitives().next().unwrap().morph_targets().len();
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
                object,
            ));
        }
        clips.push(animation::Clip { name, tracks });
    }
    clips
}

fn bind_objects(
    factory: &mut Factory,
    gltf: &gltf::Gltf,
    heirarchy: &VecMap<Group>,
    meshes: &VecMap<Vec<Mesh>>,
    skeletons: &[Skeleton],
) -> Vec<Mesh> {
    let mut instances = Vec::new();
    for node in gltf.nodes() {
        if let Some(ref group) = heirarchy.get(node.index()) {
            if let Some(mesh) = node.mesh() {
                let mut primitives = meshes[mesh.index()]
                    .iter()
                    .map(|template| factory.mesh_instance(template))
                    .collect::<Vec<Mesh>>();
                for primitive in &primitives {
                    group.add(primitive);
                }
                if let Some(skin) = node.skin() {
                    let skeleton = &skeletons[skin.index()];
                    group.add(skeleton);
                    for primitive in &mut primitives {
                        primitive.set_skeleton(skeleton.clone());
                    }
                }
                for primitive in primitives {
                    instances.push(primitive);
                }
            }
        }
    }
    instances
}

impl super::Factory {
    /// Load a scene from glTF 2.0 format.
    pub fn load_gltf(
        &mut self,
        path_str: &str,
    ) -> Gltf {
        info!("Loading {}", path_str);
        let path = Path::new(path_str);
        let base = path.parent().unwrap_or(&Path::new(""));
        let (gltf, buffers) = gltf_importer::import(path).expect("invalid glTF 2.0");

        let mut cameras = Vec::new();
        let mut clips = Vec::new();
        let mut heirarchy = VecMap::new();
        let mut instances = Vec::new();
        let mut materials = Vec::new();
        let mut meshes = VecMap::new();
        let root = self.group();
        let mut skeletons = Vec::new();
        let mut textures = Vec::new();

        if let Some(scene) = gltf.default_scene() {
            build_scene_hierarchy(self, &gltf, &scene, &root, &mut heirarchy);
            cameras = load_cameras(self, &gltf);
            textures = load_textures(self, &gltf, base, &buffers);
            materials = load_materials(&gltf, &textures);
            meshes = load_meshes(self, &gltf, &materials, &buffers);
            skeletons = load_skeletons(self, &gltf, &heirarchy, &buffers);
            clips = load_clips(&gltf, &heirarchy, &buffers);
            instances = bind_objects(self, &gltf, &heirarchy, &meshes, &skeletons);
        }

        Gltf {
            cameras,
            clips,
            heirarchy,
            instances,
            materials,
            meshes,
            root,
            skeletons,
            textures,
        }
    }
}
