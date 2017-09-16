use geometry::Geometry;
use hub::Operation;
use material::Material;
use object::Object;
use render::DynamicData;

/// [`Geometry`](struct.Geometry.html) with some [`Material`](struct.Material.html).
///
/// # Examples
///
/// Creating a red triangle.
///
/// ```rust,no_run
/// # let shaders_path = format!("{}/data/shaders", env!("CARGO_MANIFEST_DIR"));
/// # let mut win = three::Window::builder("Example", &shaders_path).build();
/// # let factory = &mut win.factory;
/// let vertices = vec![
///     [-0.5, -0.5, 0.0].into(),
///     [ 0.5, -0.5, 0.0].into(),
///     [ 0.5, -0.5, 0.0].into(),
/// ];
/// let geometry = three::Geometry::with_vertices(vertices);
/// let red_material = three::Material::MeshBasic {
///     color: 0xFF0000,
///     wireframe: false,
///     map: None,
/// };
/// let mesh = factory.mesh(geometry, red_material);
/// # let _ = mesh;
/// ```
///
/// Duplicating a mesh.
///
/// ```rust,no_run
/// # let shaders_path = format!("{}/data/shaders", env!("CARGO_MANIFEST_DIR"));
/// # let mut win = three::Window::builder("Example", &shaders_path).build();
/// # let factory = &mut win.factory;
/// # let vertices = vec![
/// #     [-0.5, -0.5, 0.0].into(),
/// #     [ 0.5, -0.5, 0.0].into(),
/// #     [ 0.5, -0.5, 0.0].into(),
/// # ];
/// # let geometry = three::Geometry::with_vertices(vertices);
/// # let red_material = three::Material::MeshBasic {
/// #     color: 0xFF0000,
/// #     wireframe: false,
/// #     map: None,
/// # };
/// # let mesh = factory.mesh(geometry, red_material);
/// let yellow_material = three::Material::MeshBasic {
///    color: 0xFFFF00,
///    wireframe: false,
///    map: None,
/// };
/// let mut duplicate = factory.mesh_instance(&mesh, Some(yellow_material));
/// // Duplicated meshes share their geometry but can be transformed individually
/// // and be rendered with different materials.
/// duplicate.set_position([1.2, 3.4, 5.6]);
/// ```
///
/// # Notes
///
/// * Meshes are removed from the scene when dropped.
/// * Hence, meshes must be kept in scope in order to be displayed.
pub struct Mesh {
    pub(crate) object: Object,
}

/// A dynamic version of a mesh allows changing the geometry on CPU side
/// in order to animate the mesh.
pub struct DynamicMesh {
    pub(crate) object: Object,
    pub(crate) geometry: Geometry,
    pub(crate) dynamic: DynamicData,
}

impl Mesh {
    /// Set mesh material.
    pub fn set_material(
        &mut self,
        material: Material,
    ) {
        let msg = Operation::SetMaterial(material);
        let _ = self.tx.send((self.node.downgrade(), msg));
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
        let _ = self.tx.send((self.node.downgrade(), msg));
    }
}
