use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc;

use froggy;
use genmesh::Triangulate;
use genmesh::generators::{self, IndexedPolygon, SharedVertex};
use gfx;
use gfx::traits::{Factory as Factory_, FactoryExt};
use image;

use render::{BackendFactory, BackendResources, GpuData, Vertex};
use scene::{Group, Mesh, Material};
use {Normal, Position, Scene, VisualObject};


pub type SceneId = usize;

pub struct Factory {
    backend: BackendFactory,
    scene_id: SceneId,
}

impl Factory {
    pub fn new(backend: BackendFactory) -> Self {
        Factory {
            backend: backend,
            scene_id: 0,
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

    pub fn new_box(sx: f32, sy: f32, sz: f32) -> Geometry {
        let cube = generators::Cube::new();
        Geometry {
            vertices: cube.shared_vertex_iter()
                          .map(|(x, y, z)| Position::new(x * sx, y * sy, z * sz))
                          .collect(),
            normals: Vec::new(),
            faces: cube.indexed_polygon_iter()
                       .triangulate()
                       .map(|t| [t.x as u16, t.y as u16, t.z as u16])
                       .collect(),
            is_dynamic: false,
        }
    }
}


pub struct Texture {
    view: gfx::handle::ShaderResourceView<BackendResources, [f32; 4]>
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
                        .to_rgba();
        let (width, height) = img.dimensions();
        let kind = t::Kind::D2(width as t::Size, height as t::Size, t::AaMode::Single);
        let (_, view) = self.backend.create_texture_immutable_u8::<gfx::format::Rgba8>(kind, &[&img])
                                    .unwrap_or_else(|e| panic!("Unable to create GPU texture for {}: {:?}", path_str, e));

        Texture {
            view:view
        }
    }
}
