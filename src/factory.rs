use std::cmp;
use std::collections::hash_map::{HashMap, Entry};
use std::io::BufReader;
use std::fs::File;
use std::path::Path;

use cgmath::{self, Transform as Transform_};
use genmesh::{Polygon, EmitTriangles, Triangulate, Vertex as GenVertex};
use genmesh::generators::{self, IndexedPolygon, SharedVertex};
use gfx;
use gfx::format::I8Norm;
use gfx::handle as h;
use gfx::traits::{Factory as Factory_, FactoryExt};
use image;
use mint;
use obj;

use render::{BackendFactory, BackendResources, ConstantBuffer, GpuData, Vertex, ShadowFormat};
use scene::{Color, Background, Group, Mesh, Sprite, Material,
            AmbientLight, DirectionalLight, HemisphereLight, PointLight};
use {Hub, HubPtr, SubLight, Node, SubNode,
     VisualData, LightData, Object, VisualObject, LightObject, Scene,
     Camera, OrthographicCamera, PerspectiveCamera};


const NORMAL_Z: [I8Norm; 4] = [I8Norm(0), I8Norm(0), I8Norm(1), I8Norm(0)];

const QUAD: [Vertex; 4] = [
    Vertex {
        pos: [-1.0, -1.0, 0.0, 1.0],
        uv: [0.0, 0.0],
        normal: NORMAL_Z,
    },
    Vertex {
        pos: [1.0, -1.0, 0.0, 1.0],
        uv: [1.0, 0.0],
        normal: NORMAL_Z,
    },
    Vertex {
        pos: [-1.0, 1.0, 0.0, 1.0],
        uv: [0.0, 1.0],
        normal: NORMAL_Z,
    },
    Vertex {
        pos: [1.0, 1.0, 0.0, 1.0],
        uv: [1.0, 1.0],
        normal: NORMAL_Z,
    },
];


impl From<SubNode> for Node {
    fn from(sub: SubNode) -> Self {
        Node {
            visible: true,
            world_visible: false,
            transform: Transform_::one(),
            world_transform: Transform_::one(),
            parent: None,
            scene_id: None,
            sub_node: sub,
        }
    }
}

impl Hub {
    fn spawn(&mut self) -> Object {
        Object {
            visible: true,
            transform: Transform_::one(),
            node: self.nodes.create(SubNode::Empty.into()),
            tx: self.message_tx.clone(),
        }
    }

    fn spawn_visual(&mut self, data: VisualData<ConstantBuffer>)
                    -> VisualObject
    {
        VisualObject {
            data: data.drop_payload(),
            inner: Object {
                visible: true,
                transform: Transform_::one(),
                node: self.nodes.create(SubNode::Visual(data).into()),
                tx: self.message_tx.clone(),
            },
        }
    }

    fn spawn_light(&mut self, data: LightData) -> LightObject {
        LightObject {
            inner: Object {
                visible: true,
                transform: Transform_::one(),
                node: self.nodes.create(SubNode::Light(data.clone()).into()),
                tx: self.message_tx.clone(),
            },
            data,
        }
    }
}


#[derive(Clone, Debug)]
pub struct ShadowMap {
    resource: gfx::handle::ShaderResourceView<BackendResources, f32>,
    target: gfx::handle::DepthStencilView<BackendResources, ShadowFormat>,
}

impl ShadowMap {
    #[doc(hidden)]
    pub fn to_target(&self) -> gfx::handle::DepthStencilView<BackendResources, ShadowFormat> {
        self.target.clone()
    }

    #[doc(hidden)]
    pub fn to_resource(&self) -> gfx::handle::ShaderResourceView<BackendResources, f32> {
        self.resource.clone()
    }
}


pub type SceneId = usize;

pub struct Factory {
    backend: BackendFactory,
    scene_id: SceneId,
    hub: HubPtr,
    quad: GpuData,
    texture_cache: HashMap<String, Texture<[f32; 4]>>,
}

impl Factory {
    #[doc(hidden)]
    pub fn new(mut backend: BackendFactory) -> Self {
        let (vbuf, slice) = backend.create_vertex_buffer_with_slice(&QUAD, ());
        Factory {
            backend: backend,
            scene_id: 0,
            hub: Hub::new(),
            quad: GpuData {
                slice: slice,
                vertices: vbuf,
            },
            texture_cache: HashMap::new(),
        }
    }

    pub fn scene(&mut self) -> Scene {
        self.scene_id += 1;
        let mut hub = self.hub.lock().unwrap();
        let node = hub.nodes.create(Node {
            scene_id: Some(self.scene_id),
            .. SubNode::Empty.into()
        });
        Scene {
            unique_id: self.scene_id,
            node: node,
            tx: hub.message_tx.clone(),
            hub: self.hub.clone(),
            background: Background::Color(0),
        }
    }

    pub fn orthographic_camera(&mut self, left: f32, right: f32, top: f32, bottom: f32,
                               near: f32, far: f32) -> OrthographicCamera {
        Camera {
            object: self.hub.lock().unwrap().spawn(),
            projection: cgmath::Ortho{ left, right, bottom, top, near, far },
        }
    }

    pub fn perspective_camera(&mut self, fov: f32, aspect: f32,
                              near: f32, far: f32) -> PerspectiveCamera {
        Camera {
            object: self.hub.lock().unwrap().spawn(),
            projection: cgmath::PerspectiveFov {
                fovy: cgmath::Deg(fov).into(),
                aspect: aspect,
                near: near,
                far: far,
            },
        }
    }

    pub fn group(&mut self) -> Group {
        Group::new(self.hub.lock().unwrap().spawn())
    }

    pub fn mesh(&mut self, geom: Geometry, mat: Material) -> Mesh {
        let vertices: Vec<_> = if geom.normals.is_empty() {
            geom.vertices.iter().map(|v| Vertex {
                pos: [v.x, v.y, v.z, 1.0],
                uv: [0.0, 0.0], //TODO
                normal: NORMAL_Z,
            }).collect()
        } else {
            let f2i = |x: f32| I8Norm(cmp::min(cmp::max((x * 127.) as isize, -128), 127) as i8);
            geom.vertices.iter().zip(geom.normals.iter()).map(|(v, n)| Vertex {
                pos: [v.x, v.y, v.z, 1.0],
                uv: [0.0, 0.0], //TODO
                normal: [f2i(n.x), f2i(n.y), f2i(n.z), I8Norm(0)],
            }).collect()
        };
        //TODO: dynamic geometry
        let cbuf = self.backend.create_constant_buffer(1);
        let (vbuf, slice) = if geom.faces.is_empty() {
            self.backend.create_vertex_buffer_with_slice(&vertices, ())
        } else {
            let faces: &[u16] = gfx::memory::cast_slice(&geom.faces);
            self.backend.create_vertex_buffer_with_slice(&vertices, faces)
        };
        Mesh::new(self.hub.lock().unwrap().spawn_visual(VisualData {
            material: mat,
            payload: cbuf,
            gpu_data: GpuData {
                slice: slice,
                vertices: vbuf,
            },
        }))
    }

    pub fn sprite(&mut self, mat: Material) -> Sprite {
        let cbuf = self.backend.create_constant_buffer(1);
        Sprite::new(self.hub.lock().unwrap().spawn_visual(VisualData {
            material: mat,
            payload: cbuf,
            gpu_data: self.quad.clone(),
        }))
    }

    pub fn ambient_light(&mut self, color: Color, intensity: f32) -> AmbientLight {
        AmbientLight::new(self.hub.lock().unwrap().spawn_light(LightData {
            color,
            intensity,
            sub_light: SubLight::Ambient,
            shadow: None,
        }))
    }

    pub fn directional_light(&mut self, color: Color, intensity: f32) -> DirectionalLight {
        DirectionalLight::new(self.hub.lock().unwrap().spawn_light(LightData {
            color,
            intensity,
            sub_light: SubLight::Directional,
            shadow: None,
        }))
    }

    pub fn hemisphere_light(&mut self, sky_color: Color, ground_color: Color,
                            intensity: f32) -> HemisphereLight {
        HemisphereLight::new(self.hub.lock().unwrap().spawn_light(LightData {
            color: sky_color,
            intensity,
            sub_light: SubLight::Hemisphere{ ground: ground_color },
            shadow: None,
        }))
    }

    pub fn point_light(&mut self, color: Color, intensity: f32) -> PointLight {
        PointLight::new(self.hub.lock().unwrap().spawn_light(LightData {
            color,
            intensity,
            sub_light: SubLight::Point,
            shadow: None,
        }))
    }

    pub fn shadow_map(&mut self, width: u16, height: u16) -> ShadowMap {
        let (_, resource, target) = self.backend.create_depth_stencil::<ShadowFormat>(
            width, height).unwrap();
        ShadowMap {
            resource,
            target,
        }
    }
}


#[derive(Clone, Debug)]
pub struct Geometry {
    pub vertices: Vec<mint::Point3<f32>>,
    pub normals: Vec<mint::Vector3<f32>>,
    pub faces: Vec<[u16; 3]>,
    pub is_dynamic: bool,
}

impl Geometry {
    pub fn empty() -> Geometry {
        Geometry {
            vertices: Vec::new(),
            normals: Vec::new(),
            faces: Vec::new(),
            is_dynamic: false,
        }
    }

    pub fn from_vertices(verts: Vec<mint::Point3<f32>>) -> Geometry {
        Geometry {
            vertices: verts,
            .. Geometry::empty()
        }
    }

    fn generate<P, G, Fpos, Fnor>(gen: G, fpos: Fpos, fnor: Fnor) -> Self where
        P: EmitTriangles<Vertex=usize>,
        G: IndexedPolygon<P> + SharedVertex<GenVertex>,
        Fpos: Fn(GenVertex) -> mint::Point3<f32>,
        Fnor: Fn(GenVertex) -> mint::Vector3<f32>,
    {
        Geometry {
            vertices: gen.shared_vertex_iter()
                         .map(fpos)
                         .collect(),
            normals: gen.shared_vertex_iter()
                        .map(fnor)
                        .collect(),
            faces: gen.indexed_polygon_iter()
                       .triangulate()
                       .map(|t| [t.x as u16, t.y as u16, t.z as u16])
                       .collect(),
            is_dynamic: false,
        }
    }

    pub fn new_plane(sx: f32, sy: f32) -> Self {
        Self::generate(generators::Plane::new(),
            |GenVertex{ pos, ..}| {
                [pos[0] * 0.5 * sx, pos[1] * 0.5 * sy, 0.0].into()
            },
            |v| v.normal.into()
        )
    }

    pub fn new_box(sx: f32, sy: f32, sz: f32) -> Self {
        Self::generate(generators::Cube::new(),
            |GenVertex{ pos, ..}| {
                [pos[0] * 0.5 * sx, pos[1] * 0.5 * sy, pos[2] * 0.5 * sz].into()
            },
            |v| v.normal.into()
        )
    }

    pub fn new_cylinder(radius_top: f32, radius_bottom: f32, height: f32,
                        radius_segments: usize) -> Self
    {
        Self::generate(generators::Cylinder::new(radius_segments),
            //Three.js has height along the Y axis for some reason
            |GenVertex{ pos, ..}| {
                let scale = (pos[2] + 1.0) * 0.5 * radius_top +
                            (1.0 - pos[2]) * 0.5 * radius_bottom;
                [pos[1] * scale, pos[2] * 0.5 * height, pos[0] * scale].into()
            },
            |GenVertex{ normal, ..}| {
                [normal[1], normal[2], normal[0]].into()
            },
        )
    }

    pub fn new_sphere(radius: f32, width_segments: usize,
                      height_segments: usize) -> Self
    {
        Self::generate(generators::SphereUV::new(width_segments, height_segments),
            |GenVertex{ pos, ..}| {
                [pos[0] * radius, pos[1] * radius, pos[2] * radius].into()
            },
            |v| v.normal.into()
        )
    }
}


#[derive(Clone, Debug)]
pub struct Texture<T> {
    view: h::ShaderResourceView<BackendResources, T>,
    sampler: h::Sampler<BackendResources>,
    total_size: [u32; 2],
    tex0: [f32; 2],
    tex1: [f32; 2],
}

impl<T> Texture<T> {
    #[doc(hidden)]
    pub fn new(view: h::ShaderResourceView<BackendResources, T>,
               sampler: h::Sampler<BackendResources>,
               total_size: [u32; 2]) -> Self {
        Texture {
            view,
            sampler,
            total_size,
            tex0: [0.0; 2],
            tex1: [0.0; 2],
        }
    }

    #[doc(hidden)]
    pub fn to_param(&self) -> (h::ShaderResourceView<BackendResources, T>, h::Sampler<BackendResources>) {
        (self.view.clone(), self.sampler.clone())
    }

    pub fn set_texel_range(&mut self, base: [i16; 2], size: [u16; 2]) {
        self.tex0 = [
            base[0] as f32,
            self.total_size[1] as f32 - base[1] as f32 - size[1] as f32,
        ];
        self.tex1 = [
            base[0] as f32 + size[0] as f32,
            self.total_size[1] as f32 - base[1] as f32,
        ];
    }

    pub fn get_uv_range(&self) -> [f32; 4] {
        [self.tex0[0] / self.total_size[0] as f32,
         self.tex0[1] / self.total_size[1] as f32,
         self.tex1[0] / self.total_size[0] as f32,
         self.tex1[1] / self.total_size[1] as f32]
    }
}


impl Factory {
    fn load_texture_impl(path_str: &str, factory: &mut BackendFactory) -> Texture<[f32; 4]> {
        use gfx::texture as t;
        use image::ImageFormat as F;

        let path = Path::new(path_str);
        let extension = path.extension()
                            .expect("no extension for an image?")
                            .to_string_lossy()
                            .to_lowercase();
        let format = match extension.as_str() {
            "png" => F::PNG,
            "jpg" | "jpeg" => F::JPEG,
            "gif" => F::GIF,
            "webp" => F::WEBP,
            "ppm" => F::PPM,
            "tiff" => F::TIFF,
            "tga" => F::TGA,
            "bmp" => F::BMP,
            "ico" => F::ICO,
            "hdr" => F::HDR,
            _ => panic!("Unrecognized image extension: {}", extension),
        };

        let file = File::open(&path)
                        .unwrap_or_else(|e| panic!("Unable to open {}: {:?}", path_str, e));
        let img = image::load(BufReader::new(file), format)
                        .unwrap_or_else(|e| panic!("Unable to decode {}: {:?}", path_str, e))
                        .flipv().to_rgba();
        let (width, height) = img.dimensions();
        let kind = t::Kind::D2(width as t::Size, height as t::Size, t::AaMode::Single);
        let (_, view) = factory.create_texture_immutable_u8::<gfx::format::Srgba8>(kind, &[&img])
                               .unwrap_or_else(|e| panic!("Unable to create GPU texture for {}: {:?}", path_str, e));

        Texture::new(view, factory.create_sampler_linear(), [width, height])
    }

    fn request_texture(&mut self, path: &str) -> Texture<[f32; 4]> {
        match self.texture_cache.entry(path.to_string()) {
            Entry::Occupied(e) => e.get().clone(),
            Entry::Vacant(e) => {
                let tex = Self::load_texture_impl(path, &mut self.backend);
                e.insert(tex.clone());
                tex
            }
        }
    }

    fn load_obj_material(&mut self, mat: &obj::Material, has_normals: bool, has_uv: bool) -> Material {
        let cf2u = |c: [f32; 3]| { c.iter().fold(0, |u, &v|
            (u << 8) + cmp::min((v * 255.0) as u32, 0xFF)
        )};
        match *mat {
            obj::Material { kd: Some(color), ns: Some(glossiness), .. } if has_normals =>
                Material::MeshPhong { color: cf2u(color), glossiness },
            obj::Material { kd: Some(color), .. } if has_normals =>
                Material::MeshLambert { color: cf2u(color) },
            obj::Material { kd: Some(color), ref map_kd, .. } =>
                Material::MeshBasic {
                    color: cf2u(color),
                    map: match (has_uv, map_kd) {
                        (true, &Some(ref name)) => Some(self.request_texture(name)),
                        _ => None,
                    },
                    wireframe: false,
                },
            _ => Material::MeshBasic { color: 0xffffff, map: None, wireframe: true },
        }
    }

    pub fn load_texture(&mut self, path_str: &str) -> Texture<[f32; 4]> {
        self.request_texture(path_str)
    }

    pub fn load_obj(&mut self, path_str: &str) -> (HashMap<String, Group>, Vec<Mesh>) {
        use std::path::Path;
        use genmesh::{LruIndexer, Indexer, Vertices};

        info!("Loading {}", path_str);
        let obj = obj::load::<Polygon<obj::IndexTuple>>(Path::new(path_str)).unwrap();

        let hub_ptr = self.hub.clone();
        let mut hub = hub_ptr.lock().unwrap();
        let mut groups = HashMap::new();
        let mut meshes = Vec::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for object in obj.object_iter() {
            let mut group = Group::new(hub.spawn());
            for gr in object.group_iter() {
                let (mut num_normals, mut num_uvs) = (0, 0);
                {   // separate scope for LruIndexer
                    let f2i = |x: f32| I8Norm(cmp::min(cmp::max((x * 127.) as isize, -128), 127) as i8);
                    vertices.clear();
                    let mut lru = LruIndexer::new(10, |_, (ipos, iuv, inor)| {
                        let p: [f32; 3] = obj.position()[ipos];
                        vertices.push(Vertex {
                            pos: [p[0], p[1], p[2], 1.0],
                            uv: match iuv {
                                Some(i) => {
                                    num_uvs += 1;
                                    obj.texture()[i]
                                },
                                None => [0.0, 0.0],
                            },
                            normal: match inor {
                                Some(id) => {
                                    num_normals += 1;
                                    let n: [f32; 3] = obj.normal()[id];
                                    [f2i(n[0]), f2i(n[1]), f2i(n[2]), I8Norm(0)]
                                },
                                None => [I8Norm(0), I8Norm(0), I8Norm(0x7f), I8Norm(0)],
                            },
                        });
                    });

                    indices.clear();
                    indices.extend(gr.indices.iter().cloned()
                                             .triangulate()
                                             .vertices()
                                             .map(|tuple| lru.index(tuple) as u16));
                };

                info!("\tmaterial {} with {} normals and {} uvs", gr.name, num_normals, num_uvs);
                let material = match gr.material {
                    Some(ref rc_mat) => self.load_obj_material(&*rc_mat, num_normals!=0, num_uvs!=0),
                    None => Material::MeshBasic { color: 0xffffff, map: None, wireframe: true },
                };
                info!("\t{:?}", material);

                let (vbuf, slice) = self.backend.create_vertex_buffer_with_slice(&vertices, &indices[..]);
                let mesh = Mesh::new(hub.spawn_visual(VisualData {
                    material,
                    payload: self.backend.create_constant_buffer(1),
                    gpu_data: GpuData {
                        slice,
                        vertices: vbuf,
                    },
                }));
                group.add(&mesh);
                meshes.push(mesh);
            }

            groups.insert(object.name.clone(), group);
        }

        (groups, meshes)
    }
}
