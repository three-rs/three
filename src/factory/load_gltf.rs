use animation;
use color;
use geometry;
use gltf;
use gltf_importer;
use image;
use material;
use mint;
use object;
use std::{fs, io};

use camera::Camera;
use gltf::Gltf;
use gltf_utils::AccessorIter;
use object::Object;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use vec_map::VecMap;

use {Geometry, Group, Material, Mesh, Texture};

type GltfNodeIndex = usize;

impl super::Factory {
    /// Loads a `glTF` texture.
    fn load_gltf_texture(
        &mut self,
        texture: &AsRef<gltf::Texture>,
        buffers: &gltf_importer::Buffers,
        base: &Path,
    ) -> Texture<[f32; 4]> {
        use image::ImageFormat::{JPEG as Jpeg, PNG as Png};
        let texture = texture.as_ref();
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
        let sampler = self.sampler(mag_filter, wrap_s, wrap_t);
        self.load_texture_from_memory(width, height, &image, sampler)
    }

    /// Loads a `glTF` material.
    fn load_gltf_material(
        &mut self,
        mat: &gltf::Material,
        buffers: &gltf_importer::Buffers,
        base: &Path,
    ) -> Material {
        let pbr = mat.pbr_metallic_roughness();
        let mut is_basic_material = true;
        let base_color_map = pbr.base_color_texture()
            .map(|t| self.load_gltf_texture(&t, buffers, base));
        let normal_map = mat.normal_texture().map(|t| {
            is_basic_material = false;
            self.load_gltf_texture(&t, buffers, base)
        });
        let emissive_map = mat.emissive_texture().map(|t| {
            is_basic_material = false;
            self.load_gltf_texture(&t, buffers, base)
        });
        let metallic_roughness_map = pbr.metallic_roughness_texture().map(|t| {
            is_basic_material = false;
            self.load_gltf_texture(&t, buffers, base)
        });
        let occlusion_map = mat.occlusion_texture().map(|t| {
            is_basic_material = false;
            self.load_gltf_texture(&t, buffers, base)
        });
        let (base_color_factor, base_color_alpha) = {
            let x = pbr.base_color_factor();
            (color::from_linear_rgb([x[0], x[1], x[2]]), x[3])
        };
        if is_basic_material {
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

    /// Loads a `glTF` mesh.
    ///
    /// Note that multiple meshes are returned, since the concept of a mesh is
    /// different in `glTF` to `three`. A glTF mesh consists of one or more
    /// *primitives*, which are equivalent to `three` meshes.
    fn load_gltf_mesh(
        &mut self,
        mesh: &gltf::Mesh,
        buffers: &gltf_importer::Buffers,
        base: &Path,
    ) -> Vec<Mesh> {
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
            let geometry = Geometry {
                base_shape: geometry::Shape {
                    vertices: vertices,
                    normals: normals,
                    tangents: tangents,
                    tex_coords: tex_coords,
                },
                faces: faces,
                ..Geometry::empty()
            };
            let material = self.load_gltf_material(&primitive.material(), buffers, base);
            primitives.push(self.mesh(geometry, material));
        }
        primitives
    }

    /// Loads a single `glTF` node.
    fn load_gltf_node(
        &mut self,
        gltf: &gltf::Gltf,
        the_node: &gltf::Node,
        buffers: &gltf_importer::Buffers,
        base: &Path,
        cameras: &mut Vec<Camera>,
        meshes: &mut VecMap<Vec<Mesh>>,
        instances: &mut Vec<Mesh>,
        node_map: &mut HashMap<GltfNodeIndex, object::Base>,
    ) -> Group {
        fn clone_child<'a>(
            gltf: &'a Gltf,
            node: &gltf::Node,
        ) -> gltf::Node<'a> {
            gltf.nodes().nth(node.index()).unwrap()
        }

        struct Item<'a> {
            group: Group,
            node: gltf::Node<'a>,
        }

        let mut groups = Vec::<Group>::new();
        let mut stack = vec![
            Item {
                group: self.group(),
                node: the_node.clone(),
            },
        ];

        while let Some(mut item) = stack.pop() {
            // TODO: Groups do not handle non-uniform scaling, so for now
            // we'll choose Y to be the scale factor in all directions.
            let (translation, rotation, scale) = item.node.transform().decomposed();
            item.group.set_transform(translation, rotation, scale[1]);

            if let Some(entry) = item.node.mesh() {
                let index = entry.index();
                let has_entry = meshes.contains_key(index);
                if has_entry {
                    let mesh = meshes.get(index).unwrap();
                    for primitive in mesh.iter() {
                        let mut instance = self.mesh_instance(primitive);
                        instance.set_parent(&item.group);
                        instances.push(instance);
                    }
                } else {
                    let mut primitives = self.load_gltf_mesh(&entry, buffers, base);
                    for primitive in &mut primitives {
                        primitive.set_parent(&item.group);
                    }
                    meshes.insert(index, primitives);
                }
            }

            if let Some(entry) = item.node.camera() {
                match entry.projection() {
                    gltf::camera::Projection::Orthographic(values) => {
                        let center: mint::Point2<f32> = [0.0, 0.0].into();
                        let extent_y = values.ymag();
                        let range = values.znear() .. values.zfar();
                        let mut camera = self.orthographic_camera(center, extent_y, range);
                        camera.set_parent(&item.group);
                        cameras.push(camera);
                    }
                    gltf::camera::Projection::Perspective(values) => {
                        let fov_y = values.yfov().to_degrees();
                        let near = values.znear();
                        let mut camera = if let Some(far) = values.zfar() {
                            self.perspective_camera(fov_y, near .. far)
                        } else {
                            self.perspective_camera(fov_y, near ..)
                        };
                        camera.set_parent(&item.group);
                        cameras.push(camera);
                    }
                }
            }

            for child in item.node.children() {
                let mut child_group = self.group();
                child_group.set_parent(&item.group);
                stack.push(Item {
                    node: clone_child(&gltf, &child),
                    group: child_group,
                });
            }

            node_map.insert(item.node.index(), item.group.upcast());
            groups.push(item.group.clone());
        }

        groups.swap_remove(0)
    }

    /// Loads animations from glTF 2.0.
    pub fn load_gltf_animations(
        &mut self,
        gltf: &Gltf,
        node_map: &HashMap<GltfNodeIndex, object::Base>,
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
                let object = match node_map.get(&node.index()) {
                    Some(object) => object.clone(),
                    // This animation does not correspond to any loaded node.
                    None => continue,
                };
                let input = sampler.input();
                let output = sampler.output();
                let interpolation = match sampler.interpolation() {
                    Linear => animation::Interpolation::Linear,
                    Step => animation::Interpolation::Discrete,
                    CubicSpline => animation::Interpolation::Cubic,
                    CatmullRomSpline => animation::Interpolation::CatmullRom,
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
                    gltf::animation::TrsProperty::Weights => unimplemented!(),
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

    /// Load a scene from glTF 2.0 format.
    pub fn load_gltf(
        &mut self,
        path_str: &str,
    ) -> super::Gltf {
        info!("Loading {}", path_str);
        let path = Path::new(path_str);
        let default = Path::new("");
        let base = path.parent().unwrap_or(&default);
        let (gltf, buffers) = gltf_importer::import(path).expect("invalid glTF 2.0");
        let mut cameras = Vec::new();
        let mut meshes = VecMap::new();
        let mut instances = Vec::new();
        let mut node_map = HashMap::new();
        let mut clips = Vec::new();
        let group = self.group();

        if let Some(scene) = gltf.default_scene() {
            for root in scene.nodes() {
                let mut node = self.load_gltf_node(
                    &gltf,
                    &root,
                    &buffers,
                    base,
                    &mut cameras,
                    &mut meshes,
                    &mut instances,
                    &mut node_map,
                );
                node.set_parent(&group);
            }
            clips = self.load_gltf_animations(&gltf, &node_map, &buffers);
        }

        // Put the instances in any empty spot in the mesh map.
        {
            let mut i = 0;
            while meshes.contains_key(i) {
                i += 1;
            }
            meshes.insert(i, instances);
        }

        super::Gltf {
            group,
            cameras,
            clips,
            meshes,
        }
    }
}
