use std::collections::HashMap;

use {Group, Mesh};
use animation;
use camera::Camera;
use object;
use skeleton::Skeleton;

/// A glTF scene that has been instantiated and can be added to a [`Scene`].
///
/// Created by instantiating a scene defined in a [`GltfDefinitions`] with
/// [`Factory::instantiate_gltf_scene`]. A `Hierarchy` can be added to a [`Scene`] with
/// [`Scene::add`].
///
/// # Examples
///
/// ```no_run
/// # let mut window = three::Window::new("three-rs");
/// let definitions = window.factory.load_gltf("my_data.gltf");
///
/// let gltf_scene = window.factory.instantiate_gltf_scene(&definitions, 0);
/// window.scene.add(&gltf_scene);
/// ```
///
/// [`Scene`]: struct.Scene.html
/// [`Scene::add`]: struct.Scene.html#method.add
/// [`GltfDefinitions`]: struct.GltfDefinitions.html
/// [`Factory::instantiate_gltf_scene`]: struct.Factory.html#method.instantiate_gltf_scene
#[derive(Debug, Clone)]
pub struct Hierarchy {
    /// A group containing all of the root nodes of the scene.
    ///
    /// While the glTF format allows scenes to have an arbitrary number of root nodes, all scene
    /// roots are added to a single root group to make it easier to manipulate the scene as a
    /// whole. See [`roots`] for the full list of root nodes for the scene.
    ///
    /// [`roots`]: #structfield.roots
    pub group: object::Group,

    /// The indices of the root nodes of the scene.
    ///
    /// Each index corresponds to a node in [`nodes`].
    ///
    /// [`nodes`]: #structfield.nodes
    pub roots: Vec<usize>,

    /// The nodes that are part of the scene.
    ///
    /// Node instances are stored in a [`HashMap`] where the key is the node's index in the source
    /// [`GltfDefinitions::nodes`]. Note that a [`HashMap`] is used instead of a [`Vec`] because
    /// not all nodes in the source file are guaranteed to be used in the scene, and so node
    /// indices in the scene instance may not be contiguous.
    ///
    /// [`HashMap`]: https://doc.rust-lang.org/stable/std/collections/struct.HashMap.html
    /// [`Vec`]: https://doc.rust-lang.org/stable/std/vec/struct.Vec.html
    /// [`GltfDefinitions::nodes`]: struct.GltfDefinitions.html#structfield.nodes
    pub nodes: HashMap<usize, HierarchyNode>,

    /// Animations tied to this hierarchy.
    pub animations: Vec<animation::Clip>,
}

impl Hierarchy {
    /// Finds the first node in the scene with the specified name, using a [`GltfDefinitions`]
    /// to lookup the name for each node.
    ///
    /// Name matching is case-sensitive. Returns the first node with a matching name, otherwise
    /// returns `None`.
    pub fn find_by_name(
        &self,
        name: &str,
    ) -> Option<&HierarchyNode> {
        self.nodes
            .values()
            .find(|node| node.name.as_ref().map(|n| n == name).unwrap_or(false))
    }
}

impl AsRef<object::Base> for Hierarchy {
    fn as_ref(&self) -> &object::Base {
        self.group.as_ref()
    }
}

impl object::Object for Hierarchy {}

/// A node in a scene from a glTF file that has been instantiated.
#[derive(Debug, Clone)]
pub struct HierarchyNode {
    /// The name of the node.
    ///
    /// Names are not guaranteed to be unique, but can be used to help identify nodes.
    pub name: Option<String>,

    /// The group that represents this node.
    pub group: Group,

    /// The meshes associated with this node.
    pub meshes: Vec<Mesh>,

    /// The skeleton associated with this node.
    ///
    /// If `skeleton` has a value, then there will be at least one mesh in `meshes`.
    pub skeleton: Option<Skeleton>,

    /// The camera associated with this node.
    pub camera: Option<Camera>,

    /// The indices of the children of this node.
    pub children: Vec<usize>,
}

impl AsRef<object::Base> for HierarchyNode {
    fn as_ref(&self) -> &object::Base {
        self.group.as_ref()
    }
}

impl object::Object for HierarchyNode {}
