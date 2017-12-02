//! `Scene` and `SyncGuard` structures.

use color::Color;
use hub::{Hub, HubPtr};
use node::Node;
use object::Object;
use texture::{CubeMap, Texture};

use std::sync::MutexGuard;

/// Unique identifier for a scene.
pub type Uid = usize;

/// Background type.
#[derive(Clone, Debug, PartialEq)]
pub enum Background {
    /// Basic solid color background.
    Color(Color),
    /// Texture background, covers the whole screen.
    // TODO: different wrap modes?
    Texture(Texture<[f32; 4]>),
    /// Skybox
    Skybox(CubeMap<[f32; 4]>),
}

/// The root node of a tree of game objects that may be rendered by a [`Camera`].
///
/// [`Camera`]: ../camera/struct.Camera.html
pub struct Scene {
    pub(crate) object: Object,
    pub(crate) hub: HubPtr,
    pub(crate) uid: Uid,
    /// See [`Background`](struct.Background.html).
    pub background: Background,
}

/// `SyncGuard` is used to obtain information about scene nodes in the most effective way.
///
/// # Examples
///
/// Imagine that you have your own helper type `Enemy`:
///
/// ```rust
/// # #[macro_use]
/// # extern crate three;
/// struct Enemy {
///     mesh: three::Mesh,
///     is_visible: bool,
/// }
/// three_object!(Enemy::mesh);
/// # fn main() {}
/// ```
///
/// You need this wrapper around `three::Mesh` to cache some information - in our case, visibility.
///
/// In your game you contain all your enemy objects in `Vec<Enemy>`. In the main loop you need
/// to iterate over all the enemies and make them visible or not, basing on current position.
/// The most obvious way is to use [`Object::sync`], but it's not the best idea from the side of
/// performance. Instead, you can create `SyncGuard` and use its `resolve` method to effectively
/// walk through every enemy in your game:
///
/// ```rust,no_run
/// # #[macro_use]
/// # extern crate three;
/// # #[derive(Clone)]
/// # struct Enemy {
/// #     mesh: three::Mesh,
/// #     is_visible: bool,
/// # }
/// # three_object!(Enemy::mesh);
/// # fn main() {
/// # let mut win = three::Window::new("SyncGuard example");
/// # let geometry = three::Geometry::default();
/// # let material = three::material::Basic { color: three::color::RED, map: None };
/// # let mesh = win.factory.mesh(geometry, material);
/// # let mut enemy = Enemy { mesh, is_visible: true };
/// # enemy.set_parent(&win.scene);
/// # let mut enemies = vec![enemy];
/// # loop {
/// let mut sync = win.scene.sync_guard();
/// for enemy in &mut enemies {
///     let node = sync.resolve(enemy);
///     let position = node.world_transform.position;
///     if position.x > 10.0 {
///         enemy.is_visible = false;
///         enemy.set_visible(false);
///     } else {
///         enemy.is_visible = true;
///         enemy.set_visible(true);
///     }
/// }
/// # }}
/// ```
///
/// [`Object::sync`]: ../struct.Object.html#method.sync
pub struct SyncGuard<'a> {
    hub: MutexGuard<'a, Hub>,
    scene_id: Option<Uid>,
}

impl<'a> SyncGuard<'a> {
    /// Obtains `objects`'s [`Node`] in an effective way.
    ///
    /// # Panics
    /// Panics if `scene` doesn't have this `Object`.
    ///
    /// [`Node`]: ../node/struct.Node.html
    pub fn resolve<T: AsRef<Object> + 'a>(
        &mut self,
        object: &T,
    ) -> Node {
        let node_internal = self.hub.get(object);
        assert_eq!(node_internal.scene_id, self.scene_id);
        node_internal.to_node()
    }
}

impl Scene {
    /// Create new [`SyncGuard`](struct.SyncGuard.html).
    ///
    /// This is performance-costly operation, you should not use it many times per frame.
    pub fn sync_guard<'a>(&'a mut self) -> SyncGuard<'a> {
        let mut hub = self.hub.lock().unwrap();
        hub.process_messages();
        hub.update_graph();
        let scene_id = Some(self.uid);
        SyncGuard { hub, scene_id }
    }
}

three_object_internal!(Scene);
