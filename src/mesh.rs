use object;

use arrayvec::ArrayVec;
use geometry::Geometry;
use hub::Operation;
use material::Material;
use render::DynamicData;
use skeleton::Skeleton;

use std::hash::{Hash, Hasher};

pub const MAX_TARGETS: usize = 9;

/// Defines a weight target.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Target {
    /// Position displacements.
    Position,

    /// Normal displacements.
    Normal,

    /// Tangent displacements.
    Tangent,
}

pub enum Weight {
    Position(f32),
    Normal(f32),
    Tangent(f32),
}

/// [`Geometry`](struct.Geometry.html) with some [`Material`](struct.Material.html).
///
/// # Examples
///
/// Creating a solid red triangle.
///
/// ```rust,no_run
/// # let mut win = three::Window::new("Example");
/// # let factory = &mut win.factory;
/// let vertices = vec![
///     [-0.5, -0.5, 0.0].into(),
///     [ 0.5, -0.5, 0.0].into(),
///     [ 0.5, -0.5, 0.0].into(),
/// ];
/// let geometry = three::Geometry::with_vertices(vertices);
/// let red_material = three::material::Basic { color: three::color::RED, map: None };
/// let mesh = factory.mesh(geometry, red_material);
/// # let _ = mesh;
/// ```
///
/// Duplicating a mesh.
///
/// ```rust,no_run
/// # let mut win = three::Window::new("Example");
/// # let factory = &mut win.factory;
/// # let vertices = vec![
/// #     [-0.5, -0.5, 0.0].into(),
/// #     [ 0.5, -0.5, 0.0].into(),
/// #     [ 0.5, -0.5, 0.0].into(),
/// # ];
/// # let geometry = three::Geometry::with_vertices(vertices);
/// # let red_material = three::material::Basic { color: three::color::RED, map: None };
/// # let mesh = factory.mesh(geometry, red_material);
/// use three::Object;
/// let mut duplicate = factory.mesh_instance(&mesh);
/// // Duplicated meshes share their geometry but may be transformed individually.
/// duplicate.set_position([1.2, 3.4, 5.6]);
/// ```
///
/// Duplicating a mesh with a different material.
///
/// ```rust,no_run
/// # let mut win = three::Window::new("Example");
/// # let factory = &mut win.factory;
/// # let vertices = vec![
/// #     [-0.5, -0.5, 0.0].into(),
/// #     [ 0.5, -0.5, 0.0].into(),
/// #     [ 0.5, -0.5, 0.0].into(),
/// # ];
/// # let geometry = three::Geometry::with_vertices(vertices);
/// # let red_material = three::material::Basic { color: three::color::RED, map: None };
/// # let mesh = factory.mesh(geometry, red_material);
/// let yellow_material = three::material::Wireframe { color: three::color::YELLOW };
/// # use three::Object;
/// let mut duplicate = factory.mesh_instance_with_material(&mesh, yellow_material);
/// duplicate.set_position([1.2, 3.4, 5.6]);
/// ```
///
/// # Notes
///
/// * Meshes are removed from the scene when dropped.
/// * Hence, meshes must be kept in scope in order to be displayed.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Mesh {
    pub(crate) object: object::Base,
}
three_object!(Mesh::object);

/// A dynamic version of a mesh allows changing the geometry on CPU side
/// in order to animate the mesh.
#[derive(Clone, Debug)]
pub struct DynamicMesh {
    pub(crate) object: object::Base,
    pub(crate) geometry: Geometry,
    pub(crate) dynamic: DynamicData,
}
three_object!(DynamicMesh::object);

impl PartialEq for DynamicMesh {
    fn eq(
        &self,
        other: &DynamicMesh,
    ) -> bool {
        self.object == other.object
    }
}

impl Eq for DynamicMesh {}

impl Hash for DynamicMesh {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        self.object.hash(state);
    }
}

impl Mesh {
    /// Set mesh material.
    pub fn set_material(
        &mut self,
        material: Material,
    ) {
        let msg = Operation::SetMaterial(material);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }

    /// Bind a skeleton to the mesh.
    pub fn set_skeleton(
        &mut self,
        skeleton: Skeleton,
    ) {
        let msg = Operation::SetSkeleton(skeleton);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }

    /// Bind a set of morph targets to the mesh.
    pub fn set_targets(
        &mut self,
        targets: ArrayVec<[Target; MAX_TARGETS]>,
    ) {
        let msg = Operation::SetTargets(targets);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }
}

impl DynamicMesh {
    /// Returns the number of vertices of the geometry base shape.
    pub fn vertex_count(&self) -> usize {
        self.geometry.base_shape.vertices.len()
    }

    /// Set mesh material.
    pub fn set_material(
        &mut self,
        material: Material,
    ) {
        let msg = Operation::SetMaterial(material);
        let _ = self.object.tx.send((self.object.node.downgrade(), msg));
    }
}
