#[cfg(feature = "gltf-loader")]
pub(crate) mod load_gltf;

use std::{cmp, fs, io, iter, ops};
use std::borrow::Cow;
use std::collections::hash_map::{Entry, HashMap};
use std::io::Read;
use std::path::{Path, PathBuf};

use cgmath::{Vector3};
use gfx;
use gfx::format::I8Norm;
use gfx::traits::{Factory as Factory_, FactoryExt};
use hub;
use image;
use itertools::Either;
use mint;
use obj;

use animation;
use audio;
use camera::{Camera, Projection, ZRange};
use color::{BLACK, Color};
use geometry::Geometry;
use hub::{Hub, HubPtr, LightData, SubLight, SubNode};
use light::{Ambient, Directional, Hemisphere, Point, ShadowMap};
use material::{self, Material};
use mesh::{DynamicMesh, Mesh};
use object::{self, Group, Object};
use render::{basic_pipe,
    BackendFactory, BackendResources, BasicPipelineState, DisplacementContribution,
    DynamicData, GpuData, Instance, InstanceCacheKey, PipelineCreationError, ShadowFormat, Source, Vertex,
    DEFAULT_VERTEX, VECS_PER_BONE, ZEROED_DISPLACEMENT_CONTRIBUTION,
};
use scene::{Background, Scene};
use sprite::Sprite;
use skeleton::{Bone, Skeleton};
use template::{AnimationTemplate, SkeletonTemplate, Template};
use text::{Font, Text, TextData};
use texture::{CubeMap, CubeMapPath, FilterMethod, Sampler, Texture, WrapMode};

const TANGENT_X: [I8Norm; 4] = [I8Norm(1), I8Norm(0), I8Norm(0), I8Norm(1)];
const NORMAL_Z: [I8Norm; 4] = [I8Norm(0), I8Norm(0), I8Norm(1), I8Norm(0)];

const QUAD: [Vertex; 4] = [
    Vertex {
        pos: [-1.0, -1.0, 0.0, 1.0],
        uv: [0.0, 0.0],
        .. DEFAULT_VERTEX
    },
    Vertex {
        pos: [1.0, -1.0, 0.0, 1.0],
        uv: [1.0, 0.0],
        .. DEFAULT_VERTEX
    },
    Vertex {
        pos: [-1.0, 1.0, 0.0, 1.0],
        uv: [0.0, 1.0],
        .. DEFAULT_VERTEX
    },
    Vertex {
        pos: [1.0, 1.0, 0.0, 1.0],
        uv: [1.0, 1.0],
        .. DEFAULT_VERTEX
    },
];

/// Mapping writer.
pub type MapVertices<'a> = gfx::mapping::Writer<'a, BackendResources, Vertex>;

/// `Factory` is used to instantiate game objects.
pub struct Factory {
    pub(crate) backend: BackendFactory,
    hub: HubPtr,
    quad_buf: gfx::handle::Buffer<BackendResources, Vertex>,
    texture_cache: HashMap<PathBuf, Texture<[f32; 4]>>,
    default_sampler: gfx::handle::Sampler<BackendResources>,
}

fn f2i(x: f32) -> I8Norm {
    I8Norm(cmp::min(cmp::max((x * 127.0) as isize, -128), 127) as i8)
}

impl Factory {
    fn create_instance_buffer(&mut self) -> gfx::handle::Buffer<BackendResources, Instance> {
        // TODO: Better error handling
        self.backend
            .create_buffer(
                1,
                gfx::buffer::Role::Vertex,
                gfx::memory::Usage::Dynamic,
                gfx::memory::Bind::TRANSFER_DST,
            )
            .unwrap()
    }

    pub(crate) fn new(mut backend: BackendFactory) -> Self {
        let quad_buf = backend.create_vertex_buffer(&QUAD);
        let default_sampler = backend.create_sampler_linear();
        Factory {
            backend: backend,
            hub: Hub::new(),
            quad_buf,
            texture_cache: HashMap::new(),
            default_sampler: default_sampler,
        }
    }

    /// Create new empty [`Scene`](struct.Scene.html).
    pub fn scene(&mut self) -> Scene {
        let hub = self.hub.clone();
        let background = Background::Color(BLACK);
        Scene {
            hub,
            first_child: None,
            background,
        }
    }

    /// Creates an instance of all the objects described in the template, returning the root
    /// `Group` of the resulting hierarchy.
    pub fn instantiate_template(&mut self, template: &Template) -> (Group, Vec<animation::Clip>) {
        let default_material: Material = material::Basic {
            color: 0xFFFFFF,
            map: None,
        }.into();

        // Create group to act as the root node of the instantiated hierarchy.
        let root = self.group();

        // Create a group for each node in the template.
        let mut groups = Vec::with_capacity(template.nodes.len());
        while groups.len() < template.nodes.len() {
            groups.push(self.group());
        }

        // Instantiate all skeletons in the template.
        let skeletons: Vec<_> = template
            .skeletons
            .iter()
            .map(|skeleton| instantiate_skeleton(skeleton, self, &groups))
            .collect();

        // Instantiate all animation clips in the template.
        let animations = template
            .animations
            .iter()
            .map(|animation| instantiate_animation(animation, &groups))
            .collect();

        // For each of the root nodes, add the node's group to the root group.
        for &root_index in &template.roots {
            root.add(&groups[root_index]);
        }

        // For each of the nodes, instantiate an attach the various objects assoicated with
        // the node.
        for (index, node) in template.nodes.iter().enumerate() {
            let group = &groups[index];

            // Set the node's transform.
            group.set_transform(node.translation, node.rotation, node.scale);

            // Add the groups for the node's children.
            for &child_index in &node.children {
                group.add(&groups[child_index]);
            }

            // Instantiate and add meshes.
            if let Some(mesh_index) = node.mesh {
                let mesh_templates = &template.meshes[mesh_index];
                for mesh_template in mesh_templates {
                    let material = mesh_template
                        .material
                        .map(|index| template.materials[index].clone())
                        .unwrap_or(default_material.clone());
                    let mut mesh = self.mesh(mesh_template.geometry.clone(), material);
                    group.add(&mesh);

                    // Set skeletonf for mesh, if applicable.
                    if let Some(skeleton_index) = node.skeleton {
                        let skeleton = &skeletons[skeleton_index];
                        mesh.set_skeleton(skeleton.clone());
                    }
                }
            }

            // Instantiate and add camera.
            if let Some(camera_index) = node.camera {
                let projection = template.cameras[camera_index].clone();
                let camera = self.camera(projection);
                group.add(&camera);
            }

            // Attach skeleton.
            if let Some(skeleton_index) = node.skeleton {
                let skeleton = &skeletons[skeleton_index];
                group.add(skeleton);
            }
        }

        (root, animations)
    }

    /// Create a new [`Bone`], one component of a [`Skeleton`].
    ///
    /// [`Bone`]: ../skeleton/struct.Bone.html
    /// [`Skeleton`]: ../skeleton/struct.Skeleton.html
    pub fn bone(
        &mut self,
        index: usize,
        inverse_bind_matrix: mint::ColumnMatrix4<f32>,
    ) -> Bone {
        let data = SubNode::Bone { index, inverse_bind_matrix };
        let object = self.hub.lock().unwrap().spawn(data);
        Bone { object }
    }

    /// Create a new [`Skeleton`] from a set of [`Bone`] instances.
    ///
    /// * `bones` is the array of bones that form the skeleton.
    /// * `inverses` is an optional array of inverse bind matrices for each bone.
    /// [`Skeleton`]: ../skeleton/struct.Skeleton.html
    /// [`Bone`]: ../skeleton/struct.Bone.html
    pub fn skeleton(
        &mut self,
        bones: Vec<Bone>,
    ) -> Skeleton {
        let gpu_buffer = self.backend
            .create_buffer(
                bones.len() * VECS_PER_BONE,
                gfx::buffer::Role::Constant,
                gfx::memory::Usage::Dynamic,
                gfx::memory::Bind::SHADER_RESOURCE,
            )
            .expect("create GPU target buffer");
        let gpu_buffer_view = self.backend
            .view_buffer_as_shader_resource(&gpu_buffer)
            .expect("create shader resource view for GPU target buffer");
        let data = hub::SkeletonData { bones, gpu_buffer, gpu_buffer_view };
        let object = self.hub.lock().unwrap().spawn_skeleton(data);
        Skeleton { object }
    }

    /// Create a new camera using the provided projection.
    ///
    /// This allows you to create a camera from a predefined projection, which is useful if you
    /// e.g. load projection data from a file and don't necessarily know ahead of time what type
    /// of projection the camera uses. If you're manually creating a camera, you should use
    /// [`perspective_camera`] or [`orthographic_camera`].
    pub fn camera<P: Into<Projection>>(&mut self, projection: P) -> Camera {
        Camera::new(
            &mut *self.hub.lock().unwrap(),
            projection.into(),
        )
    }

    /// Create new [Orthographic] Camera.
    /// It's used to render 2D.
    ///
    /// [Orthographic]: https://en.wikipedia.org/wiki/Orthographic_projection
    pub fn orthographic_camera<P: Into<mint::Point2<f32>>>(
        &mut self,
        center: P,
        extent_y: f32,
        range: ops::Range<f32>,
    ) -> Camera {
        Camera::new(
            &mut *self.hub.lock().unwrap(),
            Projection::orthographic(center, extent_y, range),
        )
    }

    /// Create new [Perspective] Camera.
    ///
    /// It's used to render 3D.
    ///
    /// # Examples
    ///
    /// Creating a finite perspective camera.
    ///
    /// ```rust,no_run
    /// # #![allow(unreachable_code, unused_variables)]
    /// # let mut factory: three::Factory = unimplemented!();
    /// let camera = factory.perspective_camera(60.0, 0.1 .. 1.0);
    /// ```
    ///
    /// Creating an infinite perspective camera.
    ///
    /// ```rust,no_run
    /// # #![allow(unreachable_code, unused_variables)]
    /// # let mut factory: three::Factory = unimplemented!();
    /// let camera = factory.perspective_camera(60.0, 0.1 ..);
    /// ```
    ///
    /// [Perspective]: https://en.wikipedia.org/wiki/Perspective_(graphical)
    pub fn perspective_camera<R: Into<ZRange>>(
        &mut self,
        fov_y: f32,
        range: R,
    ) -> Camera {
        Camera::new(
            &mut *self.hub.lock().unwrap(),
            Projection::perspective(fov_y, range),
        )
    }

    /// Create empty [`Group`](struct.Group.html).
    pub fn group(&mut self) -> object::Group {
        object::Group::new(&mut *self.hub.lock().unwrap())
    }

    fn mesh_vertices(geometry: &Geometry) -> Vec<Vertex> {
        let position_iter = geometry.base.vertices.iter();
        let normal_iter = if geometry.base.normals.is_empty() {
            Either::Left(iter::repeat(NORMAL_Z))
        } else {
            Either::Right(
                geometry.base.normals
                    .iter()
                    .map(|n| [f2i(n.x), f2i(n.y), f2i(n.z), I8Norm(0)]),
            )
        };
        let uv_iter = if geometry.tex_coords.is_empty() {
            Either::Left(iter::repeat([0.0, 0.0]))
        } else {
            Either::Right(geometry.tex_coords.iter().map(|uv| [uv.x, uv.y]))
        };
        let tangent_iter = if geometry.base.tangents.is_empty() {
            // TODO: Generate tangents if texture coordinates are provided.
            // (Use mikktspace algorithm or otherwise.)
            Either::Left(iter::repeat(TANGENT_X))
        } else {
            Either::Right(
                geometry.base.tangents
                    .iter()
                    .map(|t| [f2i(t.x), f2i(t.y), f2i(t.z), f2i(t.w)]),
            )
        };
        let joint_indices_iter = if geometry.joints.indices.is_empty() {
            Either::Left(iter::repeat([0, 0, 0, 0]))
        } else {
            Either::Right(geometry.joints.indices.iter().cloned())
        };
        let joint_weights_iter = if geometry.joints.weights.is_empty() {
            Either::Left(iter::repeat([1.0, 1.0, 1.0, 1.0]))
        } else {
            Either::Right(geometry.joints.weights.iter().cloned())
        };

        izip!(
            position_iter,
            normal_iter,
            tangent_iter,
            uv_iter,
            joint_indices_iter,
            joint_weights_iter,
        )
            .map(|(pos, normal, tangent, uv, joint_indices, joint_weights)| {
                Vertex {
                    pos: [pos.x, pos.y, pos.z, 1.0],
                    normal,
                    uv,
                    tangent,
                    joint_indices,
                    joint_weights,
                }
            })
            .collect()
    }

    /// Create new `Mesh` with desired `Geometry` and `Material`.
    pub fn mesh<M: Into<Material>>(
        &mut self,
        geometry: Geometry,
        material: M,
    ) -> Mesh {
        let vertices = Self::mesh_vertices(&geometry);
        let (vbuf, mut slice) = if geometry.faces.is_empty() {
            self.backend.create_vertex_buffer_with_slice(&vertices, ())
        } else {
            let faces: &[u32] = gfx::memory::cast_slice(&geometry.faces);
            self.backend
                .create_vertex_buffer_with_slice(&vertices, faces)
        };
        slice.instances = Some((1, 0));
        let num_shapes = geometry.shapes.len();
        let mut displacement_contributions = Vec::with_capacity(num_shapes);
        let instances = self.create_instance_buffer();
        let displacements = if num_shapes != 0 {
            let num_vertices = geometry.base.vertices.len();
            let mut contents = vec![[0.0; 4]; num_shapes * 3 * num_vertices];
            for (content_chunk, shape) in contents.chunks_mut(3 * num_vertices).zip(&geometry.shapes) {
                let mut contribution = DisplacementContribution::ZERO;
                if !shape.vertices.is_empty() {
                    contribution.position = 1.0;
                    for (out, v) in content_chunk[0 * num_vertices .. 1 * num_vertices].iter_mut().zip(&shape.vertices) {
                        *out = [v.x, v.y, v.z, 1.0];
                    }
                }
                if !shape.normals.is_empty() {
                    contribution.normal = 1.0;
                    for (out, v) in content_chunk[1 * num_vertices .. 2 * num_vertices].iter_mut().zip(&shape.normals) {
                        *out = [v.x, v.y, v.z, 0.0];
                    }
                }
                if !shape.tangents.is_empty() {
                    contribution.tangent = 1.0;
                    for (out, &v) in content_chunk[2 * num_vertices .. 3 * num_vertices].iter_mut().zip(&shape.tangents) {
                        *out = v.into();
                    }
                }
                displacement_contributions.push(contribution);
            }

            let texture_and_view = self.backend
                .create_texture_immutable::<[f32; 4]>(
                    gfx::texture::Kind::D2(
                        num_vertices as _,
                        3 * num_shapes as gfx::texture::Size,
                        gfx::texture::AaMode::Single,
                    ),
                    gfx::texture::Mipmap::Provided,
                    &[gfx::memory::cast_slice(&contents)],
                )
                .unwrap();
            Some(texture_and_view)
        } else {
            None
        };

        Mesh {
            object: self.hub.lock().unwrap().spawn_visual(
                material.into(),
                GpuData {
                    slice,
                    vertices: vbuf,
                    instances,
                    displacements,
                    pending: None,
                    instance_cache_key: None,
                    displacement_contributions,
                },
                None,
            ),
        }
    }

    /// Create a new `DynamicMesh` with desired `Geometry` and `Material`.
    pub fn mesh_dynamic<M: Into<Material>>(
        &mut self,
        geometry: Geometry,
        material: M,
    ) -> DynamicMesh {
        let slice = {
            let data: &[u32] = gfx::memory::cast_slice(&geometry.faces);
            gfx::Slice {
                start: 0,
                end: data.len() as u32,
                base_vertex: 0,
                instances: Some((1, 0)),
                buffer: self.backend.create_index_buffer(data),
            }
        };
        let (num_vertices, vertices, upload_buf) = {
            let data = Self::mesh_vertices(&geometry);
            let dest_buf = self.backend
                .create_buffer_immutable(&data, gfx::buffer::Role::Vertex, gfx::memory::Bind::TRANSFER_DST)
                .unwrap();
            let upload_buf = self.backend.create_upload_buffer(data.len()).unwrap();
            // TODO: Workaround for not having a 'write-to-slice' capability.
            // Reason: The renderer copies the entire staging buffer upon updates.
            {
                self.backend
                    .write_mapping(&upload_buf)
                    .unwrap()
                    .copy_from_slice(&data);
            }
            (data.len(), dest_buf, upload_buf)
        };
        let instances = self.create_instance_buffer();
        DynamicMesh {
            object: self.hub.lock().unwrap().spawn_visual(
                material.into(),
                GpuData {
                    slice,
                    vertices,
                    instances,
                    displacements: None,
                    pending: None,
                    instance_cache_key: None,
                    displacement_contributions: ZEROED_DISPLACEMENT_CONTRIBUTION.to_vec(),
                },
                None,
            ),
            geometry,
            dynamic: DynamicData {
                num_vertices,
                buffer: upload_buf,
            },
        }
    }

    /// Create a `Mesh` sharing the geometry with another one.
    /// Rendering a sequence of meshes with the same geometry is faster.
    /// The material is duplicated from the template.
    pub fn mesh_instance(
        &mut self,
        template: &Mesh,
    ) -> Mesh {
        let instances = self.create_instance_buffer();
        let mut hub = self.hub.lock().unwrap();
        let (material, gpu_data) = match hub[template].sub_node {
            SubNode::Visual(ref mat, ref gpu, _) => {
                (mat.clone(), GpuData {
                    instances,
                    instance_cache_key: Some(InstanceCacheKey {
                        material: mat.clone(),
                        geometry: gpu.vertices.clone(),
                    }),
                    ..gpu.clone()
                })
            }
            _ => unreachable!(),
        };
        Mesh {
            object: hub.spawn_visual(material, gpu_data, None),
        }
    }

    /// Create a `Mesh` sharing the geometry with another one but with a different material.
    /// Rendering a sequence of meshes with the same geometry is faster.
    pub fn mesh_instance_with_material<M: Into<Material>>(
        &mut self,
        template: &Mesh,
        material: M,
    ) -> Mesh {
        let instances = self.create_instance_buffer();
        let material = material.into();
        let mut hub = self.hub.lock().unwrap();
        let gpu_data = match hub[template].sub_node {
            SubNode::Visual(_, ref gpu, _) => GpuData {
                instances,
                instance_cache_key: Some(InstanceCacheKey {
                    material: material.clone(),
                    geometry: gpu.vertices.clone(),
                }),
                ..gpu.clone()
            },
            _ => unreachable!(),
        };
        Mesh {
            object: hub.spawn_visual(material, gpu_data, None),
        }
    }

    /// Create new sprite from `Material`.
    pub fn sprite(
        &mut self,
        material: material::Sprite,
    ) -> Sprite {
        let instances = self.create_instance_buffer();
        let mut slice = gfx::Slice::new_match_vertex_buffer(&self.quad_buf);
        slice.instances = Some((1, 0));
        let material = Material::from(material);
        Sprite::new(self.hub.lock().unwrap().spawn_visual(
            material,
            GpuData {
                slice,
                vertices: self.quad_buf.clone(),
                instances,
                displacements: None,
                pending: None,
                instance_cache_key: None,
                displacement_contributions: ZEROED_DISPLACEMENT_CONTRIBUTION.to_vec(),
            },
            None,
        ))
    }

    /// Create a `Sprite` sharing the material with another one.
    /// Rendering a sequence of instanced sprites is much faster.
    pub fn sprite_instance(
        &mut self,
        template: &Sprite,
    ) -> Sprite {
        let instances = self.create_instance_buffer();
        let mut hub = self.hub.lock().unwrap();
        let (material, gpu_data) = match hub[template].sub_node {
            SubNode::Visual(ref mat, ref gpu, _) => {
                (mat.clone(), GpuData {
                    instances,
                    instance_cache_key: Some(InstanceCacheKey {
                        material: mat.clone(),
                        geometry: self.quad_buf.clone(),
                    }),
                    ..gpu.clone()
                })
            }
            _ => unreachable!(),
        };
        Sprite::new(hub.spawn_visual(material, gpu_data, None))
    }

    /// Create new `AmbientLight`.
    pub fn ambient_light(
        &mut self,
        color: Color,
        intensity: f32,
    ) -> Ambient {
        Ambient::new(self.hub.lock().unwrap().spawn_light(LightData {
            color,
            intensity,
            sub_light: SubLight::Ambient,
            shadow: None,
        }))
    }

    /// Create new `DirectionalLight`.
    pub fn directional_light(
        &mut self,
        color: Color,
        intensity: f32,
    ) -> Directional {
        Directional::new(self.hub.lock().unwrap().spawn_light(LightData {
            color,
            intensity,
            sub_light: SubLight::Directional,
            shadow: None,
        }))
    }

    /// Create new `HemisphereLight`.
    pub fn hemisphere_light(
        &mut self,
        sky_color: Color,
        ground_color: Color,
        intensity: f32,
    ) -> Hemisphere {
        Hemisphere::new(self.hub.lock().unwrap().spawn_light(LightData {
            color: sky_color,
            intensity,
            sub_light: SubLight::Hemisphere {
                ground: ground_color,
            },
            shadow: None,
        }))
    }

    /// Create new `PointLight`.
    pub fn point_light(
        &mut self,
        color: Color,
        intensity: f32,
    ) -> Point {
        Point::new(self.hub.lock().unwrap().spawn_light(LightData {
            color,
            intensity,
            sub_light: SubLight::Point,
            shadow: None,
        }))
    }

    /// Create a `Sampler` with default properties.
    ///
    /// The default sampler has `Clamp` as its horizontal and vertical
    /// wrapping mode and `Scale` as its filtering method.
    pub fn default_sampler(&self) -> Sampler {
        Sampler(self.default_sampler.clone())
    }

    /// Create new `Sampler`.
    pub fn sampler(
        &mut self,
        filter_method: FilterMethod,
        horizontal_wrap_mode: WrapMode,
        vertical_wrap_mode: WrapMode,
    ) -> Sampler {
        use gfx::texture::Lod;
        let info = gfx::texture::SamplerInfo {
            filter: filter_method,
            wrap_mode: (horizontal_wrap_mode, vertical_wrap_mode, WrapMode::Clamp),
            lod_bias: Lod::from(0.0),
            lod_range: (Lod::from(-8000.0), Lod::from(8000.0)),
            comparison: None,
            border: gfx::texture::PackedColor(0),
        };
        let inner = self.backend.create_sampler(info);
        Sampler(inner)
    }

    /// Create new `ShadowMap`.
    pub fn shadow_map(
        &mut self,
        width: u16,
        height: u16,
    ) -> ShadowMap {
        let (_, resource, target) = self.backend
            .create_depth_stencil::<ShadowFormat>(width, height)
            .unwrap();
        ShadowMap { resource, target }
    }

    /// Create a basic mesh pipeline using a custom shader.
    pub fn basic_pipeline<P: AsRef<Path>>(
        &mut self,
        dir: P,
        name: &str,
        primitive: gfx::Primitive,
        rasterizer: gfx::state::Rasterizer,
        color_mask: gfx::state::ColorMask,
        blend_state: gfx::state::Blend,
        depth_state: gfx::state::Depth,
        stencil_state: gfx::state::Stencil,
    ) -> Result<BasicPipelineState, PipelineCreationError> {
        use gfx::traits::FactoryExt;
        let vs = Source::user(&dir, name, "vs")?;
        let ps = Source::user(&dir, name, "ps")?;
        let shaders = self.backend
            .create_shader_set(vs.0.as_bytes(), ps.0.as_bytes())?;
        let init = basic_pipe::Init {
            out_color: ("Target0", color_mask, blend_state),
            out_depth: (depth_state, stencil_state),
            ..basic_pipe::new()
        };
        let pso = self.backend
            .create_pipeline_state(&shaders, primitive, rasterizer, init)?;
        Ok(pso)
    }

    /// Create new UI (on-screen) text. See [`Text`](struct.Text.html) for default settings.
    pub fn ui_text<S: Into<String>>(
        &mut self,
        font: &Font,
        text: S,
    ) -> Text {
        let sub = SubNode::UiText(TextData::new(font, text));
        let object = self.hub.lock().unwrap().spawn(sub);
        Text::with_object(object)
    }

    /// Create new audio source.
    pub fn audio_source(&mut self) -> audio::Source {
        let sub = SubNode::Audio(audio::AudioData::new());
        let object = self.hub.lock().unwrap().spawn(sub);
        audio::Source::with_object(object)
    }

    /// Map vertices for updating their data.
    pub fn map_vertices<'a>(
        &'a mut self,
        mesh: &'a mut DynamicMesh,
    ) -> MapVertices<'a> {
        self.hub.lock().unwrap().update_mesh(mesh);
        self.backend.write_mapping(&mesh.dynamic.buffer).unwrap()
    }

    /// Interpolate between the shapes of a `DynamicMesh`.
    pub fn mix(
        &mut self,
        mesh: &DynamicMesh,
        shapes: &[(usize, f32)],
    ) {
        self.hub.lock().unwrap().update_mesh(mesh);
        let mut mapping = self.backend.write_mapping(&mesh.dynamic.buffer).unwrap();

        let n = mesh.geometry.base.vertices.len();
        for i in 0 .. n {
            let (mut pos, ksum) = shapes.iter().fold(
                (Vector3::new(0.0, 0.0, 0.0), 0.0),
                |(pos, ksum), &(idx, k)| {
                    let p: [f32; 3] = mesh.geometry.shapes[idx].vertices[i].into();
                    (pos + k * Vector3::from(p), ksum + k)
                },
            );
            if ksum != 1.0 {
                let p: [f32; 3] = mesh.geometry.base.vertices[i].into();
                pos += (1.0 - ksum) * Vector3::from(p);
            }
            mapping[i] = Vertex {
                pos: [pos.x, pos.y, pos.z, 1.0],
                .. mapping[i]
            };
        }
    }

    /// Load TrueTypeFont (.ttf) from file.
    /// #### Panics
    /// Panics if I/O operations with file fails (e.g. file not found or corrupted)
    pub fn load_font<P: AsRef<Path>>(
        &mut self,
        file_path: P,
    ) -> Font {
        use self::io::Read;
        let file_path = file_path.as_ref();
        let mut buffer = Vec::new();
        let file = fs::File::open(&file_path).expect(&format!(
            "Can't open font file:\nFile: {}",
            file_path.display()
        ));
        io::BufReader::new(file)
            .read_to_end(&mut buffer)
            .expect(&format!(
                "Can't read font file:\nFile: {}",
                file_path.display()
            ));
        Font::new(buffer, file_path.to_owned(), self.backend.clone())
    }

    fn parse_texture_format(path: &Path) -> image::ImageFormat {
        use image::ImageFormat as F;
        let extension = path.extension()
            .expect("no extension for an image?")
            .to_string_lossy()
            .to_lowercase();
        match extension.as_str() {
            "png" => F::PNG,
            "jpg" | "jpeg" => F::JPEG,
            "gif" => F::GIF,
            "webp" => F::WEBP,
            "ppm" => F::PNM,
            "tiff" => F::TIFF,
            "tga" => F::TGA,
            "bmp" => F::BMP,
            "ico" => F::ICO,
            "hdr" => F::HDR,
            _ => panic!("Unrecognized image extension: {}", extension),
        }
    }

    fn load_texture_impl(
        path: &Path,
        sampler: Sampler,
        factory: &mut BackendFactory,
    ) -> Texture<[f32; 4]> {
        use gfx::texture as t;
        //TODO: generate mipmaps
        let format = Factory::parse_texture_format(path);
        let file = fs::File::open(path).unwrap_or_else(|e| panic!("Unable to open {}: {:?}", path.display(), e));
        let img = image::load(io::BufReader::new(file), format)
            .unwrap_or_else(|e| panic!("Unable to decode {}: {:?}", path.display(), e))
            .flipv()
            .to_rgba();
        let (width, height) = img.dimensions();
        let kind = t::Kind::D2(width as t::Size, height as t::Size, t::AaMode::Single);
        let (_, view) = factory
            .create_texture_immutable_u8::<gfx::format::Srgba8>(kind, t::Mipmap::Provided, &[&img])
            .unwrap_or_else(|e| {
                panic!(
                    "Unable to create GPU texture for {}: {:?}",
                    path.display(),
                    e
                )
            });
        Texture::new(view, sampler.0, [width, height])
    }

    fn load_cubemap_impl<P: AsRef<Path>>(
        paths: &CubeMapPath<P>,
        sampler: Sampler,
        factory: &mut BackendFactory,
    ) -> CubeMap<[f32; 4]> {
        use gfx::texture as t;
        let images = paths
            .as_array()
            .iter()
            .map(|path| {
                let format = Factory::parse_texture_format(path.as_ref());
                let file = fs::File::open(path).unwrap_or_else(|e| panic!("Unable to open {}: {:?}", path.as_ref().display(), e));
                image::load(io::BufReader::new(file), format)
                    .unwrap_or_else(|e| panic!("Unable to decode {}: {:?}", path.as_ref().display(), e))
                    .to_rgba()
            })
            .collect::<Vec<_>>();
        let data: [&[u8]; 6] = [
            &images[0], &images[1], &images[2], &images[3], &images[4], &images[5]
        ];
        let size = images[0].dimensions().0;
        let kind = t::Kind::Cube(size as t::Size);
        let (_, view) = factory
            .create_texture_immutable_u8::<gfx::format::Srgba8>(kind, t::Mipmap::Provided, &data)
            .unwrap_or_else(|e| {
                panic!("Unable to create GPU texture for cubemap: {:?}", e);
            });
        CubeMap::new(view, sampler.0)
    }

    fn request_texture<P: AsRef<Path>>(
        &mut self,
        path: P,
        sampler: Sampler,
    ) -> Texture<[f32; 4]> {
        match self.texture_cache.entry(path.as_ref().to_owned()) {
            Entry::Occupied(e) => e.get().clone(),
            Entry::Vacant(e) => {
                let tex = Self::load_texture_impl(path.as_ref(), sampler, &mut self.backend);
                e.insert(tex.clone());
                tex
            }
        }
    }

    fn load_obj_material(
        &mut self,
        mat: &obj::Material,
        has_normals: bool,
        has_uv: bool,
        obj_dir: Option<&Path>,
    ) -> Material {
        let cf2u = |c: [f32; 3]| {
            c.iter()
                .fold(0, |u, &v| (u << 8) + cmp::min((v * 255.0) as u32, 0xFF))
        };
        match *mat {
            obj::Material {
                kd: Some(color),
                ns: Some(glossiness),
                ..
            } if has_normals =>
            {
                material::Phong {
                    color: cf2u(color),
                    glossiness,
                }.into()
            }
            obj::Material {
                kd: Some(color), ..
            } if has_normals =>
            {
                material::Lambert {
                    color: cf2u(color),
                    flat: false,
                }.into()
            }
            obj::Material {
                kd: Some(color),
                ref map_kd,
                ..
            } => material::Basic {
                color: cf2u(color),
                map: match (has_uv, map_kd) {
                    (true, &Some(ref name)) => {
                        let sampler = self.default_sampler();
                        Some(self.request_texture(&concat_path(obj_dir, name), sampler))
                    },
                    _ => None,
                },
            }.into(),
            _ => material::Basic {
                color: 0xffffff,
                map: None,
            }.into(),
        }
    }

    /// Load texture from pre-loaded data.
    pub fn load_texture_from_memory(
        &mut self,
        width: u16,
        height: u16,
        pixels: &[u8],
        sampler: Sampler,
    ) -> Texture<[f32; 4]> {
        use gfx::texture as t;
        let kind = t::Kind::D2(width, height, t::AaMode::Single);
        let (_, view) = self.backend
            .create_texture_immutable_u8::<gfx::format::Srgba8>(kind, t::Mipmap::Provided, &[pixels])
            .unwrap_or_else(|e| {
                panic!("Unable to create GPU texture from memory: {:?}", e);
            });
        Texture::new(view, sampler.0, [width as u32, height as u32])
    }

    /// Load texture from file, with default `Sampler`.
    /// Supported file formats are: PNG, JPEG, GIF, WEBP, PPM, TIFF, TGA, BMP, ICO, HDR.
    pub fn load_texture<P: AsRef<Path>>(
        &mut self,
        path_str: P,
    ) -> Texture<[f32; 4]> {
        let sampler = self.default_sampler();
        self.request_texture(path_str, sampler)
    }

    /// Load texture from file, with custom `Sampler`.
    /// Supported file formats are: PNG, JPEG, GIF, WEBP, PPM, TIFF, TGA, BMP, ICO, HDR.
    pub fn load_texture_with_sampler<P: AsRef<Path>>(
        &mut self,
        path_str: P,
        sampler: Sampler,
    ) -> Texture<[f32; 4]> {
        self.request_texture(path_str, sampler)
    }

    /// Load cubemap from files.
    /// Supported file formats are: PNG, JPEG, GIF, WEBP, PPM, TIFF, TGA, BMP, ICO, HDR.
    pub fn load_cubemap<P: AsRef<Path>>(
        &mut self,
        paths: &CubeMapPath<P>,
    ) -> CubeMap<[f32; 4]> {
        Factory::load_cubemap_impl(paths, self.default_sampler(), &mut self.backend)
    }

    /// Load mesh from Wavefront Obj format.
    pub fn load_obj(
        &mut self,
        path_str: &str,
    ) -> (HashMap<String, object::Group>, Vec<Mesh>) {
        use genmesh::{Indexer, LruIndexer, Polygon, Triangulate, Vertices};

        info!("Loading {}", path_str);
        let path = Path::new(path_str);
        let path_parent = path.parent();
        let mut obj: obj::Obj<Polygon<_>> = obj::Obj::load(path).unwrap();
        obj.load_mtls().unwrap();

        let hub_ptr = self.hub.clone();
        let mut hub = hub_ptr.lock().unwrap();
        let mut groups = HashMap::new();
        let mut meshes = Vec::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for object in &obj.objects {
            let group = object::Group::new(&mut *hub);
            for gr in &object.groups {
                let (mut num_normals, mut num_uvs) = (0, 0);
                {
                    // separate scope for LruIndexer
                    let f2i = |x: f32| I8Norm(cmp::min(cmp::max((x * 127.) as isize, -128), 127) as i8);
                    vertices.clear();
                    let mut lru = LruIndexer::new(10, |_, obj::IndexTuple(ipos, iuv, inor)| {
                        let p: [f32; 3] = obj.position[ipos];
                        vertices.push(Vertex {
                            pos: [p[0], p[1], p[2], 1.0],
                            uv: match iuv {
                                Some(i) => {
                                    num_uvs += 1;
                                    obj.texture[i]
                                }
                                None => [0.0, 0.0],
                            },
                            normal: match inor {
                                Some(id) => {
                                    num_normals += 1;
                                    let n: [f32; 3] = obj.normal[id];
                                    [f2i(n[0]), f2i(n[1]), f2i(n[2]), I8Norm(0)]
                                }
                                None => [I8Norm(0), I8Norm(0), I8Norm(0x7f), I8Norm(0)],
                            },
                            .. DEFAULT_VERTEX
                        });
                    });

                    indices.clear();
                    indices.extend(
                        gr.polys
                            .iter()
                            .cloned()
                            .triangulate()
                            .vertices()
                            .map(|tuple| lru.index(tuple) as u16),
                    );
                };

                info!(
                    "\tmaterial {} with {} normals and {} uvs",
                    gr.name, num_normals, num_uvs
                );
                let material = match gr.material {
                    Some(ref rc_mat) => self.load_obj_material(&*rc_mat, num_normals != 0, num_uvs != 0, path_parent),
                    None => material::Basic {
                        color: 0xFFFFFF,
                        map: None,
                    }.into(),
                };
                info!("\t{:?}", material);

                let (vertices, mut slice) = self.backend
                    .create_vertex_buffer_with_slice(&vertices, &indices[..]);
                slice.instances = Some((1, 0));
                let instances = self.backend
                    .create_buffer(
                        1,
                        gfx::buffer::Role::Vertex,
                        gfx::memory::Usage::Dynamic,
                        gfx::memory::Bind::TRANSFER_DST,
                    )
                    .unwrap();
                let mesh = Mesh {
                    object: hub.spawn_visual(
                        material,
                        GpuData {
                            slice,
                            vertices,
                            instances,
                            displacements: None,
                            pending: None,
                            instance_cache_key: None,
                            displacement_contributions: ZEROED_DISPLACEMENT_CONTRIBUTION.to_vec(),
                        },
                        None,
                    ),
                };
                group.add(&mesh);
                meshes.push(mesh);
            }

            groups.insert(object.name.clone(), group);
        }

        (groups, meshes)
    }

    /// Load audio from file. Supported formats are Flac, Vorbis and WAV.
    pub fn load_audio<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> audio::Clip {
        let mut buffer = Vec::new();
        let mut file = fs::File::open(&path).expect(&format!(
            "Can't open audio file:\nFile: {}",
            path.as_ref().display()
        ));
        file.read_to_end(&mut buffer).expect(&format!(
            "Can't read audio file:\nFile: {}",
            path.as_ref().display()
        ));
        audio::Clip::new(buffer)
    }
}

fn concat_path<'a>(
    base: Option<&Path>,
    name: &'a str,
) -> Cow<'a, Path> {
    match base {
        Some(base) => Cow::Owned(base.join(name)),
        None => Cow::Borrowed(Path::new(name)),
    }
}

fn instantiate_skeleton(
    template: &SkeletonTemplate,
    factory: &mut Factory,
    groups: &[Group],
) -> Skeleton {
    let bones = template
        .bones
        .iter()
        .enumerate()
        .map(|(index, template)| {
            let bone = factory.bone(index, template.inverse_bind_matrix);
            groups[template.joint].add(&bone);
            bone
        })
        .collect();
    factory.skeleton(bones)
}

fn instantiate_animation(
    template: &AnimationTemplate,
    groups: &[Group],
) -> animation::Clip {
    let tracks = template
        .tracks
        .iter()
        .map(|&(ref track, target_index)| (track.clone(), groups[target_index].upcast()))
        .collect();

    animation::Clip {
        name: template.name.clone(),
        tracks,
    }
}
