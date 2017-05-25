use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc;

use froggy;
use genmesh::Triangulate;
use genmesh::generators::{self, IndexedPolygon, SharedVertex};
use gfx;
use gfx::format::I8Norm;
use gfx::handle as h;
use gfx::traits::{Factory as Factory_, FactoryExt};
use image;

use render::{BackendFactory, BackendResources, GpuData, Vertex};
use scene::{Group, Mesh, Sprite, Material};
use {Normal, Position, Scene, VisualObject};


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

pub type SceneId = usize;

pub struct Factory {
    backend: BackendFactory,
    scene_id: SceneId,
    quad: GpuData,
}

impl Factory {
    #[doc(hidden)]
    pub fn new(mut backend: BackendFactory) -> Self {
        let (vbuf, slice) = backend.create_vertex_buffer_with_slice(&QUAD, ());
        Factory {
            backend: backend,
            scene_id: 0,
            quad: GpuData {
                slice: slice,
                vertices: vbuf,
            }
        }
    }

    pub fn scene(&mut self) -> Scene {
        self.scene_id += 1;
        let (tx, rx) = mpsc::channel();
        Scene {
            nodes: froggy::Storage::new(),
            visuals: froggy::Storage::new(),
            unique_id: self.scene_id,
            message_tx: tx,
            message_rx: rx,
        }
    }

    pub fn group(&mut self) -> Group {
        Group::new()
    }

    pub fn mesh(&mut self, geom: Geometry, mat: Material) -> Mesh {
        let vertices: Vec<_> = geom.vertices.iter().map(|v| Vertex {
            pos: [v.x, v.y, v.z, 1.0],
            uv: [0.0, 0.0], //TODO
            normal: NORMAL_Z, //TODO
        }).collect();
        //TODO: dynamic geometry
        let (vbuf, slice) = if geom.faces.is_empty() {
            self.backend.create_vertex_buffer_with_slice(&vertices, ())
        } else {
            let faces: &[u16] = gfx::memory::cast_slice(&geom.faces);
            self.backend.create_vertex_buffer_with_slice(&vertices, faces)
        };
        Mesh::new(VisualObject::new(mat, GpuData {
            slice: slice,
            vertices: vbuf,
        }))
    }

    pub fn sprite(&mut self, mat: Material) -> Sprite {
        Sprite::new(VisualObject::new(mat, self.quad.clone()))
    }
}


#[derive(Clone)]
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
        let function = |(x, y, z)| {
            Position::new(x * 0.5 * sx, y * 0.5 * sy, z * 0.5 * sz)
        };
        Geometry {
            vertices: gen.shared_vertex_iter()
                          .map(function)
                          .collect(),
            normals: Vec::new(),
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
        let function = |(x, y, z)| {
            let scale = (z + 1.0) * 0.5 * radius_top +
                        (1.0 - z) * 0.5 * radius_bottom;
            //three,js has height along the Y axis for some reason
            Position::new(y * scale, z * 0.5 * height, x * scale)
        };
        Geometry {
            vertices: gen.shared_vertex_iter()
                          .map(function)
                          .collect(),
            normals: Vec::new(),
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
        let (_, view) = self.backend.create_texture_immutable_u8::<gfx::format::Rgba8>(kind, &[&img])
                                    .unwrap_or_else(|e| panic!("Unable to create GPU texture for {}: {:?}", path_str, e));

        Texture {
            view: view,
            sampler: self.backend.create_sampler_linear(),
        }
    }
}
