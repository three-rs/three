//! Structures for creating and storing geometric primitives.
use mint;
use std::collections::HashMap;
use genmesh::{Vertex as GenVertex, EmitTriangles, Triangulate};
use genmesh::generators::{self, IndexedPolygon, SharedVertex};

/// A shape of geometry that is used for mesh blending.
#[derive(Clone, Debug)]
pub struct Shape {
    /// Vertices.
    pub vertices: Vec<mint::Point3<f32>>,
    /// Normals.
    pub normals: Vec<mint::Vector3<f32>>,
    /// Tangents.
    pub tangents: Vec<mint::Vector4<f32>>,
    /// Texture co-ordinates.
    pub tex_coords: Vec<mint::Point2<f32>>,
}

impl Shape {
    /// Create an empty shape.
    pub fn empty() -> Self {
        Shape {
            vertices: Vec::new(),
            normals: Vec::new(),
            tangents: Vec::new(),
            tex_coords: Vec::new(),
        }
    }
}

/// A collection of vertices, their normals, and faces that defines the
/// shape of a polyhedral object.
#[derive(Clone, Debug)]
pub struct Geometry {
    /// The original shape of geometry.
    pub base_shape: Shape,
    /// A map containing blend shapes and their names.
    pub shapes: HashMap<String, Shape>,
    /// Faces.
    pub faces: Vec<[u32; 3]>,
}

impl Geometry {
    /// Create new `Geometry` without any data in it.
    pub fn empty() -> Self {
        Geometry {
            base_shape: Shape::empty(),
            shapes: HashMap::new(),
            faces: Vec::new(),
        }
    }

    /// Create `Geometry` from vector of vertices.
    pub fn with_vertices(vertices: Vec<mint::Point3<f32>>) -> Self {
        Geometry {
            base_shape: Shape {
                vertices,
                normals: Vec::new(),
                .. Shape::empty()
            },
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
            base_shape: Shape {
                vertices: gen.shared_vertex_iter().map(fpos).collect(),
                normals: gen.shared_vertex_iter().map(fnor).collect(),
                // @alteous: TODO: Add similar functions for tangents and texture
                // co-ordinates
                .. Shape::empty()
            },
            shapes: HashMap::new(),
            faces: gen.indexed_polygon_iter()
                .triangulate()
                .map(|t| [t.x as u32, t.y as u32, t.z as u32])
                .collect(),
        }
    }

    /// Create new Plane with desired size.
    pub fn plane(sx: f32, sy: f32) -> Self {
        Self::generate(generators::Plane::new(),
                       |GenVertex{ pos, ..}| {
                           [pos[0] * 0.5 * sx, pos[1] * 0.5 * sy, 0.0].into()
                       },
                       |v| v.normal.into()
        )
    }

    /// Create new Box with desired size.
    pub fn cuboid(sx: f32, sy: f32, sz: f32) -> Self {
        Self::generate(generators::Cube::new(),
                       |GenVertex{ pos, ..}| {
                           [pos[0] * 0.5 * sx, pos[1] * 0.5 * sy, pos[2] * 0.5 * sz].into()
                       },
                       |v| v.normal.into()
        )
    }

    /// Create new Cylinder or Cone with desired top and bottom radius, height
    /// and number of segments.
    pub fn cylinder(radius_top: f32, radius_bottom: f32, height: f32,
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

    /// Create new Sphere with desired radius and number of segments.
    pub fn sphere(radius: f32, width_segments: usize,
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