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

use camera::Camera;
use gltf::Gltf;
use gltf_utils::AccessorIter;
use mesh::{Target, MAX_TARGETS};
use object::Object;
use skeleton::Skeleton;
use std::path::{Path, PathBuf};
use vec_map::VecMap;

use {Geometry, Group, Material, Mesh, Texture};
use super::Factory;

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

    fn clone_child_node<'a>(gltf: &'a gltf::Gltf, node: &gltf::Node) -> gltf::Node<'a> {
        gltf.nodes().nth(node.index()).unwrap()
    }
    
    let nr_nodes = gltf.nodes().len();
    let mut stack = Vec::with_capacity(nr_nodes);

    for node in scene.nodes() {
        let mut group = factory.group();
        group.set_parent(root);
        stack.push(Item { group, node });
    }

    while let Some(Item { group, node }) = stack.pop() {
        for child_node in node.children() {
            let mut child_group = factory.group();
            child_group.set_parent(&group);
            stack.push(Item { group: child_group, node: clone_child_node(gltf, &child_node) });
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
    gltf: &Gltf,
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
                occlusion_strength: mat.occlusion_texture().map_or(1.0, |t| t.strength()),
                emissive_factor: color::from_linear_rgb(mat.emissive_factor()),
                normal_scale: mat.normal_texture().map_or(1.0, |t| t.scale()),
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
    ) -> (geometry::MorphTargets, [Target; MAX_TARGETS]) {
        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut tangents = Vec::new();
        let mut targets = [Target::None; MAX_TARGETS];
        let mut i = 0;
        for target in primitive.morph_targets() {
            if let Some(accessor) = target.positions() {
                targets[i] = Target::Position;
                i += 1;
                let iter = AccessorIter::<[f32; 3]>::new(accessor, buffers);
                vertices.extend(iter.map(|x| mint::Vector3::<f32>::from(x)));
            }

            if let Some(accessor) = target.normals() {
                targets[i] = Target::Normal;
                i += 1;
                let iter = AccessorIter::<[f32; 3]>::new(accessor, buffers);
                normals.extend(iter.map(|x| mint::Vector3::<f32>::from(x)));
            }

            if let Some(accessor) = target.tangents() {
                targets[i] = Target::Tangent;
                i += 1;
                let iter = AccessorIter::<[f32; 3]>::new(accessor, buffers);
                tangents.extend(iter.map(|x| mint::Vector3::<f32>::from(x)));
            }
        }

        (geometry::MorphTargets {
            names: VecMap::new(),
            vertices,
            normals,
            tangents,
        }, targets)
    }
    
    let mut meshes = VecMap::new();
    for mesh in gltf.meshes() {
        let mut primitives = Vec::new();
        for primitive in mesh.primitives() {
            use gltf_utils::PrimitiveIterators;
            let mut faces = vec![];
            if let Some(mut iter) = primitive.indices_u32(buffers) {
                while let (Some(a), Some(b), Some(c)) = (iter.next(), iter.next(), iter.next()) {
                    faces.push([a, b, c]);
                }
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
                iter.map(|x| [x[0] as f32, x[1] as f32, x[2] as f32, x[3] as f32]).collect()
            } else {
                Vec::new()
            };
            let joint_weights = if let Some(iter) = primitive.weights_f32(0, buffers) {
                iter.collect()
            } else {
                Vec::new()
            };
            let (morph_targets, mesh_targets) = load_morph_targets(&primitive, &buffers);
            let geometry = Geometry {
                vertices,
                normals,
                tangents,
                tex_coords,
                joints: geometry::Joints {
                    indices: joint_indices,
                    weights: joint_weights,
                },
                morph_targets,
                faces,
                ..Geometry::empty()
            };
            let material = if let Some(index) = primitive.material().index() {
                materials[index].clone()
            } else {
                material::Basic {
                    color: 0xFFFFFF,
                    map: None,
                }.into()
            };
            let mut mesh = factory.mesh_with_targets(geometry, material, mesh_targets);
            primitives.push(mesh);
        }
        meshes.insert(mesh.index(), primitives);
    }
    meshes
}


fn load_skeletons(
    factory: &mut Factory,
    gltf: &Gltf,
    heirarchy: &VecMap<Group>,
    buffers: &gltf_importer::Buffers,
) -> Vec<Skeleton> {
    let mut skeletons = Vec::new();
    for skin in gltf.skins() {
        let mut ibms = Vec::new();
        let mut bones = Vec::new();
        if let Some(accessor) = skin.inverse_bind_matrices() {
            for ibm in AccessorIter::<[[f32; 4]; 4]>::new(accessor, buffers) {
                ibms.push(ibm.into());
            }
        }
        for joint in skin.joints() {
            let mut bone = factory.bone();
            bone.set_parent(&heirarchy[&joint.index()]);
            bones.push(bone);
        }
        let skeleton = factory.skeleton(bones, ibms);
        skeletons.push(skeleton);
    }
    skeletons
}

fn load_clips(
    gltf: &Gltf,
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
                    // Note
                    //
                    // The number of morph targets in glTF usually differs from the number of
                    // morph targets in three-rs since where glTF sets a single weight for one set
                    // of targets { POSITION, NORMAL, TANGENT }, three-rs expects separate weights
                    // for each of POSITION, NORMAL, TANGENT in the same set.
                    //
                    // This array keeps track of how many times a weight needs to be duplicated
                    // to reflect the above.
                    let mut nr_three_targets_per_gltf_target = [0; MAX_TARGETS];
                    let mut nr_three_targets = 0;
                    let nr_gltf_targets;
                    {
                        let mesh = node.mesh().unwrap();
                        let first_primitive = mesh.primitives().next().unwrap();
                        nr_gltf_targets = first_primitive.morph_targets().len();
                        for (i, target) in first_primitive.morph_targets().enumerate() {
                            if target.positions().is_some() {
                                nr_three_targets_per_gltf_target[i] += 1;
                                nr_three_targets += 1;
                            }
                            if target.normals().is_some() {
                                nr_three_targets_per_gltf_target[i] += 1;
                                nr_three_targets += 1;
                            }
                            if target.tangents().is_some() {
                                nr_three_targets_per_gltf_target[i] += 1;
                                nr_three_targets += 1;
                            }
                        }
                    }
                    let mut values = Vec::with_capacity(times.len() * MAX_TARGETS);
                    let raw = AccessorIter::<f32>::new(output, buffers).collect::<Vec<_>>();
                    for chunk in raw.chunks(nr_gltf_targets) {
                        for (i, value) in chunk.iter().enumerate() {
                            for _ in 0 .. nr_three_targets_per_gltf_target[i] {
                                values.push(*value)
                            }
                        }
                        for _ in nr_three_targets .. MAX_TARGETS {
                            values.push(0.0);
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
                for primitive in &mut primitives {
                    primitive.set_parent(group);
                }
                if let Some(skin) = node.skin() {
                    let mut skeleton = skeletons[skin.index()].clone();
                    skeleton.set_parent(group);
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
    ) -> super::Gltf {
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

        super::Gltf {
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
