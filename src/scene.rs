
use color::Color;
use hub::HubPtr;
use object::Object;
use texture::{CubeMap, Texture};

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
    /// See [`Background`](struct.Background.html).
    pub background: Background,
}
three_object_wrapper!(Scene);
