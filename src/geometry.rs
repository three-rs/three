//! Structures for creating and storing geometric primitives.

use genmesh::{EmitTriangles, Triangulate, Vertex as GenVertex};
use genmesh::generators::{self, IndexedPolygon, SharedVertex};
use mint;
use vec_map::VecMap;

/// A collection of vertices, their normals, and faces that defines the
/// shape of a polyhedral object.
///
/// # Examples
///
/// Tetrahedron of unit height and base radius.
///
/// ```rust
/// # extern crate three;
/// # fn make_tetrahedron() -> three::Geometry {
/// use std::f32::consts::PI;
///
/// let vertices = vec![
///     [0.0, 1.0, 0.0].into(),
///     [0.0, 0.0, 1.0].into(),
///     [(2.0 * PI / 3.0).sin(), 0.0, (2.0 * PI / 3.0).cos()].into(),
///     [(4.0 * PI / 3.0).sin(), 0.0, (4.0 * PI / 3.0).cos()].into(),
/// ];
///
/// let faces = vec![
///     [0, 1, 2],
///     [0, 2, 3],
///     [0, 3, 1],
///     [1, 3, 2],
/// ];
///
/// three::Geometry {
///     faces,
///     vertices,
///     .. three::Geometry::empty()
/// }
/// # }
/// # fn main() { let _ = make_tetrahedron(); }
/// ```
#[derive(Clone, Debug, Default)]
pub struct Geometry {
    /// Vertex positions.
    pub vertices: Vec<mint::Point3<f32>>,
    /// Vertex normals.
    pub normals: Vec<mint::Vector3<f32>>,
    /// Vertex tangents.
    pub tangents: Vec<mint::Vector4<f32>>,
    /// Vertex texture co-ordinates.
    pub tex_coords: Vec<mint::Point2<f32>>,
    /// Face indices.
    ///
    /// When omitted, the vertex order `[[0, 1, 2], [3, 4, 5], ...]` is
    /// assumed.
    pub faces: Vec<[u32; 3]>,
    /// Properties for vertex skinning.
    pub joints: Joints,
    /// Properties for morph target animation.
    pub morph_targets: MorphTargets,
}

/// Properties for vertex skinning.
#[derive(Clone, Debug, Default)]
pub struct Joints {
    /// Joint indices, encoded as floats.
    pub indices: Vec<[f32; 4]>,
    /// Joint weights.
    pub weights: Vec<[f32; 4]>,
}

/// Properties for morph target animation
#[derive(Clone, Debug, Default)]
pub struct MorphTargets {
    /// Morph target names, one per morph target index.
    pub names: VecMap<String>,
    ///Contiguous sets of vertex displacements.
    pub vertices: Vec<mint::Point3<f32>>,
    ///Contiguous sets of normal displacements.
    pub normals: Vec<mint::Vector3<f32>>,
    /// Contiguous sets of tangent displacements.
    pub tangents: Vec<mint::Vector4<f32>>,
}

impl Geometry {
    /// Create empty `Geometry`.
    ///
    /// # Examples
    ///
    /// Basic usage.
    ///
    /// ```rust
    /// let geometry = three::Geometry::empty();
    /// assert!(geometry.base_shape.vertices.is_empty());
    /// assert!(geometry.shapes.is_empty());
    /// assert!(geometry.faces.is_empty());
    /// ```
    ///
    /// Completing geometry for a triangle.
    ///
    /// ```rust
    /// # extern crate three;
    /// fn make_triangle() -> three::Geometry {
    ///    let vertices = vec![
    ///        [-0.5, -0.5, 0.0].into(),
    ///        [ 0.5, -0.5, 0.0].into(),
    ///        [ 0.5, -0.5, 0.0].into(),
    ///    ];
    ///    three::Geometry {
    ///        base_shape: three::geometry::Shape {
    ///            vertices,
    ///            .. three::geometry::Shape::empty()
    ///        },
    ///        .. three::Geometry::empty()
    ///    }
    /// }
    /// # fn main() { let _ = make_triangle(); }
    /// ```
    pub fn empty() -> Self {
        Default::default()
    }

    /// Create `Geometry` from vector of vertices.
    ///
    /// # Examples
    ///
    /// Triangle in the XY plane.
    ///
    /// ```rust
    /// let vertices = vec![
    ///     [-0.5, -0.5, 0.0].into(),
    ///     [ 0.5, -0.5, 0.0].into(),
    ///     [ 0.5, -0.5, 0.0].into(),
    /// ];
    /// let geometry = three::Geometry::with_vertices(vertices);
    /// ```
    pub fn with_vertices(vertices: Vec<mint::Point3<f32>>) -> Self {
        Geometry {
            vertices,
            normals: Vec::new(),
            ..Geometry::empty()
        }
    }

    fn generate<P, G, Fpos, Fnor>(
        gen: G,
        fpos: Fpos,
        fnor: Fnor,
    ) -> Self
    where
        P: EmitTriangles<Vertex = usize>,
        G: IndexedPolygon<P> + SharedVertex<GenVertex>,
        Fpos: Fn(GenVertex) -> mint::Point3<f32>,
        Fnor: Fn(GenVertex) -> mint::Vector3<f32>,
    {
        Geometry {
            vertices: gen.shared_vertex_iter().map(fpos).collect(),
            normals: gen.shared_vertex_iter().map(fnor).collect(),
            // @alteous: TODO: Add similar functions for tangents and texture
            // co-ordinates
            faces: gen.indexed_polygon_iter()
                .triangulate()
                .map(|t| [t.x as u32, t.y as u32, t.z as u32])
                .collect(),
            .. Default::default()
        }
    }

    /// Creates planar geometry in the XY plane.
    ///
    /// The `width` and `height` parameters specify the total length of the
    /// geometry along the X and Y axes respectively.
    ///
    /// # Examples
    ///
    /// Unit square in the XY plane, centered at the origin.
    ///
    /// ```rust
    /// # extern crate three;
    /// fn make_square() -> three::Geometry {
    ///     three::Geometry::plane(1.0, 1.0)
    /// }
    /// # fn main() { let _ = make_square(); }
    /// ```
    pub fn plane(
        width: f32,
        height: f32,
    ) -> Self {
        Self::generate(
            generators::Plane::new(),
            |GenVertex { pos, .. }| [pos[0] * 0.5 * width, pos[1] * 0.5 * height, 0.0].into(),
            |v| v.normal.into(),
        )
    }

    /// Creates cuboidal geometry.
    ///
    /// The `width`, `height`, and `depth` parameters specify the total length of
    /// the geometry along the X, Y, and Z axes respectively.
    ///
    /// # Examples
    ///
    /// Unit cube, centered at the origin.
    ///
    /// ```rust
    /// # extern crate three;
    /// fn make_cube() -> three::Geometry {
    ///     three::Geometry::cuboid(1.0, 1.0, 1.0)
    /// }
    /// # fn main() { let _ = make_cube(); }
    /// ```
    pub fn cuboid(
        width: f32,
        height: f32,
        depth: f32,
    ) -> Self {
        Self::generate(
            generators::Cube::new(),
            |GenVertex { pos, .. }| {
                [
                    pos[0] * 0.5 * width,
                    pos[1] * 0.5 * height,
                    pos[2] * 0.5 * depth,
                ].into()
            },
            |v| v.normal.into(),
        )
    }

    /// Creates cylindrial geometry.
    ///
    /// # Examples
    ///
    /// Cylinder of unit height and radius, using 12 segments at each end.
    ///
    /// ```rust
    /// # extern crate three;
    /// fn make_cylinder() -> three::Geometry {
    ///     three::Geometry::cylinder(1.0, 1.0, 1.0, 12)
    /// }
    /// # fn main() { let _ = make_cylinder(); }
    /// ```
    ///
    /// Cone of unit height and unit radius at the bottom.
    ///
    /// ```rust
    /// # extern crate three;
    /// fn make_cone() -> three::Geometry {
    ///     three::Geometry::cylinder(0.0, 1.0, 1.0, 12)
    /// }
    /// # fn main() { let _ = make_cone(); }
    /// ```
    pub fn cylinder(
        radius_top: f32,
        radius_bottom: f32,
        height: f32,
        radius_segments: usize,
    ) -> Self {
        Self::generate(
            generators::Cylinder::new(radius_segments),
            //Three.js has height along the Y axis for some reason
            |GenVertex { pos, .. }| {
                let scale = (pos[2] + 1.0) * 0.5 * radius_top + (1.0 - pos[2]) * 0.5 * radius_bottom;
                [pos[1] * scale, pos[2] * 0.5 * height, pos[0] * scale].into()
            },
            |GenVertex { normal, .. }| [normal[1], normal[2], normal[0]].into(),
        )
    }

    /// Creates geometry for a sphere, using the UV method.
    ///
    /// * `equatorial_divisions` specifies the number of segments about
    ///    the sphere equator that lies in the XZ plane.
    /// * `meridional_segments` specifies the number of segments around
    ///    the sphere meridian that lies in the YZ plane.
    ///
    /// ```rust
    /// # extern crate three;
    /// fn make_sphere() -> three::Geometry {
    ///     three::Geometry::uv_sphere(1.0, 12, 12)
    /// }
    /// # fn main() { let _ = make_sphere(); }
    /// ```
    pub fn uv_sphere(
        radius: f32,
        equatorial_segments: usize,
        meridional_segments: usize,
    ) -> Self {
        Self::generate(
            generators::SphereUV::new(equatorial_segments, meridional_segments),
            |GenVertex { pos, .. }| [pos[0] * radius, pos[1] * radius, pos[2] * radius].into(),
            |v| v.normal.into(),
        )
    }
}
