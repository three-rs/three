use std::sync::mpsc;

use froggy;
use genmesh::Triangulate;
use genmesh::generators::{self, IndexedPolygon, SharedVertex};
use gfx;
use gfx::traits::FactoryExt;

use render::{BackendFactory, GpuData, Vertex};
use {Normal, Position, Material, Scene};
use {Object, VisualObject, Group, Mesh};


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
        Group {
            object: Object::new(),
        }
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
        Mesh {
            object: VisualObject::new(mat, GpuData {
                slice: slice,
                vertices: vbuf,
            }),
            _geometry: if geom.is_dynamic { Some(geom) } else { None },
        }
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
