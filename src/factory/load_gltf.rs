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
    MeshTemplate,
    Template,
    NodeTemplate,
    NodeTemplateData,
};

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
    default_material_index: usize,
) -> Vec<MeshTemplate> {
    mesh
        .primitives()
        .map(|prim| load_primitive(factory, prim, buffers, default_material_index))
        .collect()
}

fn load_primitive<'a>(
    factory: &mut Factory,
    primitive: gltf::Primitive<'a>,
    buffers: &gltf_importer::Buffers,
    default_material_index: usize,
) -> MeshTemplate {
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
    let material = primitive.material().index().unwrap_or(default_material_index);

    MeshTemplate {
        geometry,
        material,
    }
}

/// Helper for binding bones to their parent node in the template.
#[derive(Debug, Clone, Copy)]
struct Joint {
    /// The template index for the bone template for this joint.
    node_index: usize,

    /// The glTF index for the node that the joint targets.
    joint_index: usize
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
    buffers: &gltf_importer::Buffers,
    nodes: &mut Vec<NodeTemplate>,
    joints: &mut Vec<Joint>,
) -> (usize, Option<usize>) {
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
        into_iter()
        .chain(repeat(mx_id));

    let joint_iter = skin
        .joints()
        .map(|joint| joint.index());
    for (bone_index, (joint_index, inverse_bind_matrix)) in joint_iter.zip(ibm_iter).enumerate() {
        // Create a bone node corresponding to the joint.
        let node_index = nodes.len();
        bones.push(node_index);
        nodes.push(NodeTemplate::from_data(
            NodeTemplateData::Bone(bone_index, inverse_bind_matrix)),
        );
        joints.push(Joint { node_index, joint_index });
    }

    let skeleton_index = nodes.len();
    nodes.push(NodeTemplate::from_data(NodeTemplateData::Skeleton(bones)));

    (skeleton_index, skin.skeleton().map(|node| node.index()))
}

fn load_animation<'a>(
    animation: gltf::Animation<'a>,
    buffers: &gltf_importer::Buffers,
    node_map: &HashMap<usize, usize>,
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
            node_map[&node.index()],
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
    nodes: &mut Vec<NodeTemplate>,
    node_map: &mut HashMap<usize, usize>,
    mesh_map: &HashMap<usize, Vec<usize>>,
    skeleton_map: &[usize],
) {
    let name = node.name().map(Into::into);

    // Create a list of the children that are under the resulting template node. Note that this
    // won't necessarily match the list of children in the original glTF document since meshes
    // and cameras become separate nodes in the template.
    let mut children = Vec::new();

    // Create mesh/skinned mesh nodes for any meshes associated with this glTF node.
    let skeleton = node.skin().map(|skin| skeleton_map[skin.index()]);
    if let Some(gltf_index) = node.mesh().map(|mesh| mesh.index()) {
        for &mesh_index in &mesh_map[&gltf_index] {
            children.push(nodes.len());

            // The node will either be a mesh or a skinned mesh, depending on whether or not
            // there's a skeleton associated with the glTF node.
            let data = match skeleton {
                Some(skeleton_index) => NodeTemplateData::SkinnedMesh {
                    mesh: mesh_index,
                    skeleton: skeleton_index,
                },

                None => NodeTemplateData::Mesh(mesh_index),
            };

            nodes.push(NodeTemplate::from_data(data));
        }
    }

    // Create a camera node as a child if there's a camera associated with this glTF node.
    if let Some(camera_index) = node.camera().map(|camera| camera.index()) {
        children.push(nodes.len());
        nodes.push(NodeTemplate::from_data(NodeTemplateData::Camera(camera_index)));
    }

    // Decompose the transform to get the translation, rotation, and scale.
    let (translation, rotation, scale) = node.transform().decomposed();

    // TODO: Groups do not handle non-uniform scaling, so for now we'll choose Y to be the
    // scale factor in all directions.
    let scale = scale[1];

    // Create a `Group` node to directly represent the original glTF node, listing any extra
    // nodes we needed to create as its children.
    //
    // NOTE: At this point the list of children only includes the nodes generated for any
    // meshes and cameras attached to the original node. We can't yet add the children declared
    // in the original document because we don't know the indices that the corresponding
    // template nodes will have until we've created them all. Those children are added in a
    // final pass after all glTF nodes have been added to the template (see `Factory::load_gltf`).
    node_map.insert(node.index(), nodes.len());
    nodes.push(NodeTemplate {
        name,

        transform: Transform {
            position: translation.into(),
            orientation: rotation.into(),
            scale,
        },

        data: NodeTemplateData::Group(children),
    });
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

        let cameras: Vec<_> = gltf.cameras().map(load_camera).collect();

        let textures = load_textures(self, &gltf, base, &buffers);
        let mut materials: Vec<_> = gltf
            .materials()
            .map(|material| load_material(material, &textures))
            .collect();

        // Create a material to represent the glTF default material.
        let default_material_index = materials.len();
        materials.push(material::Basic {
            color: 0xFFFFFF,
            map: None,
        }.into());

        // Mappings that allow us to convert from indices in the glTF document to the indices in
        // the resulting template, for objects where the two don't necessarily line up.
        let mut mesh_map = HashMap::new();
        let mut node_map = HashMap::new();
        let mut skeleton_map = Vec::with_capacity(gltf.skins().len());

        // Load the meshes declared in the glTF file. Each glTF mesh declaration can potentially
        // result in multiple Three meshes, so in doing so we flatten them to a single list of
        // meshes, and populate `mesh_map` with information on how to lookup meshes in the
        // flattened list given the index in the original glTF document.
        let mut meshes = Vec::new();
        for gltf_mesh in gltf.meshes() {
            // Save the index within the glTF document so that we can add an entry to the mesh map.
            let gltf_index = gltf_mesh.index();

            // Add all of the meshes to the flattened list of meshes, and generate a list of new
            // indices that can be used to map from the glTF index to the flattened indices.
            let mut indices = Vec::new();
            for mesh in load_mesh(self, gltf_mesh, &buffers, default_material_index) {
                indices.push(meshes.len());
                meshes.push(mesh);
            }

            // Add the list of mesh indices to the mesh map.
            mesh_map.insert(gltf_index, indices);
        }

        // The full list of template nodes created from the glTF file. We know there will be at
        // least as many template nodes as nodes in the original glTF file, but there will likely
        // be many since many things in the glTF format end up as their own template nodes.
        let mut nodes = Vec::with_capacity(gltf.nodes().len());

        // Create a skeleton template for each of the skins in the glTF document. Since both
        // bones and skeletons are treated as nodes within `Template`, we need to load these
        // first so that we can reference the new nodes when loading the glTF nodes next.
        let mut skeleton_roots = Vec::with_capacity(gltf.skins().len());
        let mut joints = Vec::new();
        for skin in gltf.skins() {
            let (skeleton_index, root) = load_skin(skin, &buffers, &mut nodes, &mut joints);
            skeleton_map.push(skeleton_index);
            skeleton_roots.push(root);
        }

        // Create template nodes from each of the glTF nodes.
        for node in gltf.nodes() {
            load_node(node, &mut nodes, &mut node_map, &mesh_map, &skeleton_map);
        }

        // Fix-up any group nodes in the template by adding their original children to their
        // list of children.
        for gltf_node in gltf.nodes() {
            // Lookup the template node corresponding to this glTF node.
            let template_node_index = node_map[&gltf_node.index()];
            let template_node = &mut nodes[template_node_index];

            // Get the list of children for this node.
            let children = match template_node.data {
                NodeTemplateData::Group(ref mut children) => children,
                _ => panic!(
                    "Node corresponding to glTF node {} is not a group: {:?}",
                    gltf_node.index(),
                    template_node,
                ),
            };

            // For each of the children originally declared, lookup the index of the node in the
            // final template and add it to the group's list of children.
            for gltf_index in gltf_node.children().map(|child| child.index()) {
                let template_index = node_map[&gltf_index];
                children.push(template_index);
            }
        }

        // Once all nodes have been created, make each bone a child of the joint it targets.
        for Joint { node_index, joint_index } in joints {
            match nodes[node_map[&joint_index]].data {
                NodeTemplateData::Group(ref mut children) => children.push(node_index),
                _ => panic!("Joint index referenced by skin did not point to a group template"),
            }
        }

        // Make each skeleton a child of its root node.
        // NOTE: This needs to happen last so that skeletons are always the last child to be
        // added to their parent group. The way that Three handles rendering, it needs find the
        // skeleton before it finds any of the skeleton's bones when traversing the scene. The
        // renderer uses a depth-first traversal when walking the scene, so the skeleton needs
        // to be an earlier child than any of the bones of sub-nodes of the skeleton. Since
        // children are listed internally in the reverse order in which they are added, we need
        // to make sure that the skeleton is the last child added for its parent.
        for (index, &root) in skeleton_roots.iter().enumerate() {
            if let Some(parent_index) = root {
                match nodes[node_map[&parent_index]].data {
                    NodeTemplateData::Group(ref mut children) => children.push(skeleton_map[index]),
                    _ => panic!("Root node for skeleton wasn't a group"),
                }
            }
        }

        // Create an animation template from any animations in the glTF file.
        let animations: Vec<_> = gltf
            .animations()
            .map(|anim| load_animation(anim, &buffers, &node_map))
            .collect();

        gltf
            .scenes()
            .map(|scene| {
                let mut roots: Vec<usize> = scene
                    .nodes()
                    .map(|node| node_map[&node.index()])
                    .collect();

                // If there are any skeletons that don't have a root specified, then they become
                // root nodes of the template.
                //
                // NOTE: It's possible that a skeleton node without a root was already marked as
                // a root node by the glTF document, in which case we would have added a
                // duplicate root node to `roots`. To handle that case, we call `dedup` after
                // adding the root skeletons.
                let root_skeletons = skeleton_roots
                    .iter()
                    .enumerate()
                    .filter_map(|(index, &root)| match root {
                        Some(..) => None,
                        None => Some(skeleton_map[index]),
                    });
                roots.extend(root_skeletons);
                roots.dedup();

                let name = scene.name().map(Into::into);

                // TODO: Optimize template to only include objects that end up being instantiated.
                // Each template currently gets a copy of all objects templates loaded from the
                // source file, but may only use a subset of those depending on what the original
                // glTF scene referenced. This is safe to do because any unused objects that get
                // instantiated will immediately be destroyed, but that adds a potentially
                // unnecessary cost each time the template is instantiated. To avoid this, we can
                // optime the template ahead of time by reducing it to only objects that are
                // referenced within that template.
                Template {
                    name,
                    roots,
                    cameras: cameras.clone(),
                    materials: materials.clone(),
                    meshes: meshes.clone(),
                    nodes: nodes.clone(),
                    animations: animations.clone(),
                    lights: Vec::new(),
                }
            })
            .collect()
    }
}
