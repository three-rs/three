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

use animation::Clip;
use camera::{Orthographic, Perspective, Projection};
use gltf_utils::AccessorIter;
use std::path::{Path, PathBuf};

use {Group, Material, Mesh, Texture};
use geometry::{Geometry, Shape};
use object::Object;
use super::Factory;
use template::*;

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
    factory: &mut Factory,
    mesh: gltf::Mesh<'a>,
    buffers: &gltf_importer::Buffers,
    textures: &[Texture<[f32; 4]>],
) -> Vec<Mesh> {
    mesh
        .primitives()
        .map(|prim| load_primitive(factory, prim, buffers, textures))
        .collect()
}

fn load_primitive<'a>(
    factory: &mut Factory,
    primitive: gltf::Primitive<'a>,
    buffers: &gltf_importer::Buffers,
    textures: &[Texture<[f32; 4]>],
) -> Mesh {
    use gltf_utils::PrimitiveIterators;
    use itertools::Itertools;

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
    let shapes = load_morph_targets(primitive, buffers);
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

    let material = load_material(primitive.material(), textures);

    factory.mesh(geometry, material)
}

fn load_morph_targets<'a>(
    primitive: gltf::Primitive<'a>,
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
    nodes: HashMap<usize, HierarchyNode>,
    buffers: &gltf_importer::Buffers,
) -> Clip {
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

            // TODO: What do if node hasn't been instantiated for the current hierarchy?
            nodes[&node.index()].group.upcast(),
        ));
    }

    Clip {
        name,
        tracks,
    }
}

fn instantiate_node_hierarchy<'a>(
    factory: &mut Factory,
    node: gltf::Node<'a>,
    nodes: &mut HashMap<usize, HierarchyNode>,
    parent: &Group,
    buffers: &gltf_importer::Buffers,
    textures: &[Texture<[f32; 4]>],
) {
    let group = factory.group();
    parent.add(&group);

    // Get the transform from the source node and decompose it to get the translation, rotation,
    // and scale.
    // TODO: Groups do not handle non-uniform scaling, so for now we'll choose Y to be the
    // scale factor in all directions.
    let (translation, rotation, scale) = node.transform().decomposed();
    let scale = scale[1];

    // Apply the node's transformations to the root group of the node.
    group.set_position(translation);
    group.set_scale(scale);
    group.set_orientation(rotation);

    let meshes = node
        .mesh()
        .map(|mesh| load_mesh(factory, mesh, buffers, textures))
        .unwrap_or_default();
    let camera = node
        .camera()
        .map(load_camera)
        .map(|projection| factory.camera(projection));
    let children = node
        .children()
        .map(|child| child.index())
        .collect();

    for mesh in &meshes { group.add(mesh); }
    if let Some(ref camera) = camera { group.add(camera); }
    for child in node.children() {
        instantiate_node_hierarchy(factory, node, nodes, &group, buffers, textures);
    }

    // If the node has a camera associated with it, create a camera instance.
    if let Some(projection) = node.camera().map(load_camera) {
        let instance = factory.camera(projection);

        // Add the camera to the group that represents the node.
        group.add(&instance);

        camera = Some(instance);
    }

    let name = node.name().map(Into::into);

    let instance = HierarchyNode {
        name,

        group,

        meshes,
        skeleton: None,
        camera,
        children,
    };
    nodes.insert(node.index(), instance);
}

fn load_scene<'a>(
    factory: &mut Factory,
    scene: gltf::Scene<'a>,
    buffers: &gltf_importer::Buffers,
    textures: &[Texture<[f32; 4]>],
) -> Hierarchy {
    let mut nodes = HashMap::new();
    let group = factory.group();
    for node in scene.nodes() {
        instantiate_node_hierarchy(factory, node, &mut nodes, &group, buffers, textures);
    }

    let roots = scene
        .nodes()
        .map(|node| node.index())
        .collect();

    Hierarchy {
        group,
        roots,
        nodes,
        animations: Vec::new(),
    }
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
    ) -> Vec<Hierarchy> {
        info!("Loading {}", path_str);

        let path = Path::new(path_str);
        let base = path.parent().unwrap_or(&Path::new(""));
        let (gltf, buffers) = gltf_importer::import(path).expect("invalid glTF 2.0");

        let textures = load_textures(self, &gltf, base, &buffers);

        gltf
            .scenes()
            .map(|scene| load_scene(self, scene, &buffers, &textures))
            .collect()
    }
}
