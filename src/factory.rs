use std::cmp;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;

use cgmath::Transform as Transform_;
use genmesh::{Triangulate, Vertex as GenVertex};
use genmesh::generators::{self, IndexedPolygon, SharedVertex};
use gfx;
use gfx::format::I8Norm;
use gfx::handle as h;
use gfx::traits::{Factory as Factory_, FactoryExt};
use image;

use render::{BackendFactory, BackendResources, ConstantBuffer, GpuData, Vertex};
use scene::{Color, Group, Mesh, Sprite, Material,
            AmbientLight, DirectionalLight, HemisphereLight, PointLight};
use {Hub, HubPtr, SubLight, Node, SubNode, Normal, Position, Transform,
     VisualData, LightData, Object, VisualObject, LightObject, Scene};


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
            transform: Transform::one(),
            world_transform: Transform::one(),
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
            transform: Transform::one(),
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
                transform: Transform::one(),
                node: self.nodes.create(SubNode::Visual(data).into()),
                tx: self.message_tx.clone(),
            },
        }
    }

    fn spawn_light(&mut self, data: LightData) -> LightObject {
        LightObject {
            inner: Object {
                visible: true,
                transform: Transform::one(),
                node: self.nodes.create(SubNode::Light(data.clone()).into()),
                tx: self.message_tx.clone(),
            },
            _data: data,
        }
    }
}


pub type SceneId = usize;

pub struct Factory {
    backend: BackendFactory,
    scene_id: SceneId,
    hub: HubPtr,
    quad: GpuData,
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
        }))
    }

    pub fn directional_light(&mut self, color: Color, intensity: f32) -> DirectionalLight {
        DirectionalLight::new(self.hub.lock().unwrap().spawn_light(LightData {
            color,
            intensity,
            sub_light: SubLight::Directional,
        }))
    }

    pub fn hemisphere_light(&mut self, sky_color: Color, ground_color: Color,
                            intensity: f32) -> HemisphereLight {
        HemisphereLight::new(self.hub.lock().unwrap().spawn_light(LightData {
            color: sky_color,
            intensity,
            sub_light: SubLight::Hemisphere{ ground: ground_color },
        }))
    }

    pub fn point_light(&mut self, color: Color, intensity: f32) -> PointLight {
        PointLight::new(self.hub.lock().unwrap().spawn_light(LightData {
            color,
            intensity,
            sub_light: SubLight::Point,
        }))
    }
}


#[derive(Clone, Debug)]
pub struct Geometry {
    pub vertices: Vec<Position>,
    pub normals: Vec<Normal>,
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

    pub fn from_vertices(verts: Vec<Position>) -> Geometry {
        Geometry {
            vertices: verts,
            .. Geometry::empty()
        }
    }

    pub fn new_box(sx: f32, sy: f32, sz: f32) -> Self {
        let gen = generators::Cube::new();
        let function = |GenVertex{ pos, ..}| {
            Position::new(pos[0] * 0.5 * sx, pos[1] * 0.5 * sy, pos[2] * 0.5 * sz)
        };
        Geometry {
            vertices: gen.shared_vertex_iter()
                         .map(function)
                         .collect(),
            normals: gen.shared_vertex_iter()
                        .map(|v| Normal::from(v.normal))
                        .collect(),
            faces: gen.indexed_polygon_iter()
                       .triangulate()
                       .map(|t| [t.x as u16, t.y as u16, t.z as u16])
                       .collect(),
            is_dynamic: false,
        }
    }

    pub fn new_cylinder(radius_top: f32, radius_bottom: f32, height: f32,
                        radius_segments: usize) -> Self
    {
        let gen = generators::Cylinder::new(radius_segments);
        //Three.js has height along the Y axis for some reason
        let f_pos = |GenVertex{ pos, ..}| {
            let scale = (pos[2] + 1.0) * 0.5 * radius_top +
                        (1.0 - pos[2]) * 0.5 * radius_bottom;
            Position::new(pos[1] * scale, pos[2] * 0.5 * height, pos[0] * scale)
        };
        let f_normal = |GenVertex{ normal, ..}| {
            Normal::from([normal[1], normal[2], normal[0]])
        };
        Geometry {
            vertices: gen.shared_vertex_iter()
                         .map(f_pos)
                         .collect(),
            normals: gen.shared_vertex_iter()
                        .map(f_normal)
                        .collect(),
            faces: gen.indexed_polygon_iter()
                       .triangulate()
                       .map(|t| [t.x as u16, t.y as u16, t.z as u16])
                       .collect(),
            is_dynamic: false,
        }
    }

    pub fn new_sphere(radius: f32, width_segments: usize,
                      height_segments: usize) -> Self
    {
        let gen = generators::SphereUV::new(width_segments, height_segments);
        let function = |GenVertex{ pos, ..}| {
            Position::new(pos[0] * radius, pos[1] * radius, pos[2] * radius)
        };
        Geometry {
            vertices: gen.shared_vertex_iter()
                         .map(function)
                         .collect(),
            normals: gen.shared_vertex_iter()
                        .map(|v| Normal::from(v.normal))
                        .collect(),
            faces: gen.indexed_polygon_iter()
                       .triangulate()
                       .map(|t| [t.x as u16, t.y as u16, t.z as u16])
                       .collect(),
            is_dynamic: false,
        }
    }
}


#[derive(Clone)]
pub struct Texture {
    view: h::ShaderResourceView<BackendResources, [f32; 4]>,
    sampler: h::Sampler<BackendResources>,
}

impl Texture {
    #[doc(hidden)]
    pub fn new(view: h::ShaderResourceView<BackendResources, [f32; 4]>,
               sampler: h::Sampler<BackendResources>) -> Self {
        Texture {
            view: view,
            sampler: sampler,
        }
    }

    #[doc(hidden)]
    pub fn to_param(&self) -> (h::ShaderResourceView<BackendResources, [f32; 4]>, h::Sampler<BackendResources>) {
        (self.view.clone(), self.sampler.clone())
    }
}

impl Factory {
    pub fn load_texture(&mut self, path_str: &str) -> Texture {
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
        let (_, view) = self.backend.create_texture_immutable_u8::<gfx::format::Srgba8>(kind, &[&img])
                                    .unwrap_or_else(|e| panic!("Unable to create GPU texture for {}: {:?}", path_str, e));

        Texture {
            view: view,
            sampler: self.backend.create_sampler_linear(),
        }
    }
}
