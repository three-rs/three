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

use camera::{Orthographic, Perspective, Projection};
use gltf_utils::AccessorIter;
use std::path::{Path, PathBuf};

use {Material, Texture};
use geometry::{Geometry, Shape};
use node::Transform;
use super::Factory;
use template::{
    AnimationTemplate,
    BoneTemplate,
    CameraTemplate,
    InstancedGeometry,
    MeshTemplate,
    ObjectTemplate,
    Template,
};

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

fn load_primitive<'a>(
    factory: &mut Factory,
    primitive: gltf::Primitive<'a>,
    buffers: &gltf_importer::Buffers,
    textures: &[Texture<[f32; 4]>],
) -> (InstancedGeometry, Material) {
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

    let geometry = factory.upload_geometry(geometry);
    let material = load_material(primitive.material(), textures);
    (geometry, material)
}

/// Creates bone and skeleton templates from a glTF skin.
///
/// Returns two values:
///
/// * The index of the template node created for the skeleton.
/// * The glTF index of the node used as the skeleton root (if any).
///
/// Additionally, this will add any newly created nodes to `nodes`, and will add any joints
/// loaded to `joints`, allowing the bone node created to represent the joint to later be added
/// as a child to the group that represents the original node.
fn load_skin<'a>(
    skin: gltf::Skin<'a>,
    objects: &mut Vec<ObjectTemplate>,
    bones: &mut Vec<BoneTemplate>,
    buffers: &gltf_importer::Buffers,
) -> usize {
    use std::iter::repeat;

    let mut ibms = Vec::new();
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
        into_iter()
        .chain(repeat(mx_id));

    let joint_iter = skin
        .joints()
        .map(|joint| joint.index());
    for (index, (joint_index, inverse_bind_matrix)) in joint_iter.zip(ibm_iter).enumerate() {
        // Create a bone node corresponding to the joint.
        let object = objects.len();
        objects.push(ObjectTemplate {
            parent: Some(joint_index),
            .. Default::default()
        });
        bones.push(BoneTemplate {
            object,
            index,
            inverse_bind_matrix,
            skeleton: skin.index(),
        });
    }

    // Create a skeleton template (which is really just an object template) for the skin.
    let object = objects.len();
    objects.push(ObjectTemplate {
        parent: skin.skeleton().map(|node| node.index()),
        .. Default::default()
    });

    object
}

fn load_animation<'a>(
    animation: gltf::Animation<'a>,
    buffers: &gltf_importer::Buffers,
    groups: &[usize],
) -> AnimationTemplate {
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

            // Target the object for the group that corresponds to the target node.
            groups[node.index()],
        ));
    }

    AnimationTemplate {
        name,
        tracks,
    }
}

/// Partially loads a single glTF node and creates template nodes from its data.
///
/// Adds a `Group` template node to `nodes`, which directly represents `node`. The following
/// additional nodes may also be added:
///
/// * One `Mesh` template node will be added for each mesh primitive in the mesh referenced by
///   `node`, if any.
/// * One `Camera` template node will be added if `node` references a camera, using the
///   projection data for the camera referenced.
///
/// Any additional nodes will be listed as children of the initial `Group` template node.
///
/// # Warning
///
/// The `Group` template node corresponding to `node` will *only* list the mesh and camera
/// templates as its children, any children that `node` specifies will not be added by this
/// function. We can't yet add the children declared in the original document because we don't
/// know the indices that the corresponding template nodes will have until we've loaded and
/// processed all nodes declared in the document. Those children are added in a final pass after
/// all glTF nodes have been added to the template (see `Factory::load_gltf`).
fn load_node<'a>(
    node: gltf::Node<'a>,
    objects: &mut Vec<ObjectTemplate>,
    meshes: &mut Vec<MeshTemplate>,
    cameras: &mut Vec<CameraTemplate>,
    mesh_map: &HashMap<usize, Vec<usize>>,
    primitives: &[(InstancedGeometry, Material)],
) -> usize {
    let name = node.name().map(Into::into);

    // Decompose the transform to get the translation, rotation, and scale.
    let (translation, rotation, scale) = node.transform().decomposed();

    // TODO: Groups do not handle non-uniform scaling, so for now we'll choose Y to be the
    // scale factor in all directions.
    let scale = scale[1];

    // Create a `Group` node to directly represent the original glTF node, listing any extra
    // nodes we needed to create as its children.
    let object_index = objects.len();
    objects.push(ObjectTemplate {
        name,

        transform: Transform {
            position: translation.into(),
            orientation: rotation.into(),
            scale,
        },

        // NOTE: Since glTF has parents list their children, and three-rs templates do the
        // opposite, we wait to hook up parent/child relationships until all group templates
        // have been created. Group templates are hooked up to their parent in a pass immediately
        // following loading all nodes from the glTF data.
        parent: None,
    });

    // Create mesh/skinned mesh nodes for any meshes associated with this glTF node.
    let skeleton = node.skin().map(|skin| skin.index());
    if let Some(gltf_mesh) = node.mesh() {
        for &geometry_index in &mesh_map[&gltf_mesh.index()] {
            let (geometry, material) = primitives[geometry_index].clone();
            let object = objects.len();
            objects.push(ObjectTemplate {
                parent: Some(node.index()),
                .. Default::default()
            });
            meshes.push(MeshTemplate {
                object,
                geometry,
                material,
                skeleton,
            });
        }
    }

    // Create a camera node as a child if there's a camera associated with this glTF node.
    if let Some(camera) = node.camera() {
        let object = objects.len();
        objects.push(ObjectTemplate {
            parent: Some(node.index()),
            .. Default::default()
        });
        cameras.push(CameraTemplate {
            object,
            projection: load_camera(camera),
        })
    }

    object_index
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

fn load_scene<'a>(scene: gltf::Scene<'a>, raw: &Template) -> Template {
    // TODO: Create a new template that just contains the objects for the specified scene.

    Template {
        name: scene.name().map(Into::into),
        .. raw.clone()
    }
}

impl super::Factory {
    /// Loads templates from a glTF 2.0 file.
    ///
    /// The returned [`Template`] objects cannot be added to the scene directly, rather they
    /// contain definitions for meshes, node hierarchies, skinned meshes and their skeletons,
    /// animations, and other things that can be instantiated and added to the scene. Use
    /// [`Factory::instantiate_template`] to create an instance of the template that can be
    /// added to your scene. See the module documentation for [`template`] for more information
    /// on templates and how they are used.
    ///
    /// Each scene in the glTF file results in a separate [`Template`]. Any animations that
    /// reference nodes in a scene will be included in that scene's [`Template`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use three::animation;
    ///
    /// # let mut window = three::Window::new("Three-rs");
    /// // Load all templates from a glTF file.
    /// let mut templates = window.factory.load_gltf("my-model.gltf");
    ///
    /// // Instantiate the first template loaded and add the root group to the scene.
    /// let (root, animations) = window.factory.instantiate_template(&templates[0]);
    /// window.scene.add(&root);
    ///
    /// // Start playing all the animations instantiated from the template.
    /// let mut mixer = animation::Mixer::new();
    /// for animation in animations {
    ///     mixer.action(animation);
    /// }
    /// ```
    ///
    /// [`template`]: ./template/index.html
    /// [`Template`]: ./template/struct.Template.html
    /// [`Factory::instantiate_template`]: #method.instantiate_template
    pub fn load_gltf(
        &mut self,
        path_str: &str,
    ) -> Vec<Template> {
        info!("Loading glTF file {}", path_str);

        let path = Path::new(path_str);
        let base = path.parent().unwrap_or(&Path::new(""));
        let (gltf, buffers) = gltf_importer::import(path).expect("invalid glTF 2.0");

        let textures = load_textures(self, &gltf, base, &buffers);

        // Mappings that allow us to convert from indices in the glTF document to the indices in
        // the resulting template, for objects where the two don't necessarily line up.
        let mut mesh_map = HashMap::new();

        // Load the meshes declared in the glTF file. Each glTF mesh declaration can potentially
        // result in multiple Three meshes, so in doing so we flatten them to a single list of
        // meshes, and populate `mesh_map` with information on how to lookup meshes in the
        // flattened list given the index in the original glTF document.
        let mut primitives = Vec::new();
        for gltf_mesh in gltf.meshes() {
            // Save the index within the glTF document so that we can add an entry to the mesh map.
            let gltf_index = gltf_mesh.index();

            // Add all of the meshes to the flattened list of meshes, and generate a list of new
            // indices that can be used to map from the glTF index to the flattened indices.
            let mut indices = Vec::new();
            let prim_iter = gltf_mesh
                .primitives()
                .map(|prim| load_primitive(self, prim, &buffers, &textures));
            for primitive in prim_iter {
                indices.push(primitives.len());
                primitives.push(primitive);
            }

            // Add the list of mesh indices to the mesh map.
            mesh_map.insert(gltf_index, indices);
        }

        // The full list of template nodes created from the glTF file. We know there will be at
        // least as many template nodes as nodes in the original glTF file, but there will likely
        // be many since many things in the glTF format end up as their own template nodes.
        let mut objects = Vec::with_capacity(gltf.nodes().len());
        let mut meshes = Vec::new();
        let mut cameras = Vec::new();

        // Create template nodes from each of the glTF nodes.
        let groups: Vec<_> = gltf
            .nodes()
            .map(|node| {
                load_node(node, &mut objects, &mut meshes, &mut cameras, &mesh_map, &primitives)
            })
            .collect();

        // Fix-up any group nodes in the template by adding their original children to their
        // list of children.
        for gltf_node in gltf.nodes() {
            // For each of the children originally declared, lookup the index of the node in the
            // final template and add it to the group's list of children.
            for child_index in gltf_node.children().map(|child| child.index()) {
                let object = &mut objects[groups[child_index]];

                assert!(object.parent.is_none(), "Object template already had a parent specified");
                object.parent = Some(gltf_node.index());
            }
        }

        // Create a skeleton template for each of the skins in the glTF document.
        let mut bones = Vec::new();
        let skeletons = gltf
            .skins()
            .map(|skin| load_skin(skin, &mut objects, &mut bones, &buffers))
            .collect();

        // Create an animation template from any animations in the glTF file.
        let animations = gltf
            .animations()
            .map(|anim| load_animation(anim, &buffers, &groups))
            .collect();

        let raw_template = Template {
            name: None,
            objects,
            groups,
            cameras,
            meshes,
            lights: Vec::new(),
            bones,
            skeletons,
            animations,
        };

        if gltf.scenes().len() > 1 {
            warn!("Mutliple scenes found in {}, glTF loading does not currently work correctly for glTF files with multiple scenes", path.display());
        }

        gltf
            .scenes()
            .map(|scene| load_scene(scene, &raw_template))
            .collect()
    }
}
