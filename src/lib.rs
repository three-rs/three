#![warn(missing_docs)]
//! # Motivation and Goals
//!
//! 1. Ergonomics is first priority. Being able to prototype quickly and code
//! intuitively is more important than capturing all the 3D features. We already
//! have a solid foundation with [gfx-rs](https://github.com/gfx-rs/gfx), so
//! let's make some use of it by providing a nice higher-level abstraction.
//! 2. Follow "Three.JS" - this is simpler than coming up with a brand new API
//! (like [kiss3d](https://github.com/sebcrozet/kiss3d)), yet instantly familiar
//! to a large portion of Web develper. Some deviations from the original API are imminent.
//! 3. Explore the capabilities of Rust to mimic deep object-oriented nature of
//! JavaScript. This is a tough challenge, involving channels, defer
//! implementations, blood, and sweat.
//! 4. Battle-test the [genmesh](https://github.com/gfx-rs/genmesh) library.
//! Being able to create cubes, spheres, cylinders (and more) with one-liners
//! allows for nice proceduraly generated demos.
//! 5.Play with Component-Graph System concept, provided by [froggy](https://github.com/kvark/froggy).
//! It's a good fit for the scene graph implementation that is fast and usable.

extern crate cgmath;
extern crate froggy;
extern crate genmesh;
#[macro_use]
extern crate gfx;
extern crate image;
#[macro_use]
extern crate log;
extern crate mint;
extern crate obj;
extern crate winit;
// OpenGL
#[cfg(feature = "opengl")]
extern crate gfx_device_gl;
#[cfg(feature = "opengl")]
extern crate gfx_window_glutin;
#[cfg(feature = "opengl")]
extern crate glutin;

mod camera;
mod factory;
mod render;
mod scene;
#[cfg(feature = "opengl")]
mod window;

pub use factory::{Factory, Geometry, ShadowMap, Texture};
pub use render::{ColorFormat, DepthFormat, Renderer, ShadowType, DebugQuadHandle};
pub use scene::{Color, Background, Material, NodeTransform, NodeInfo,
                Group, Mesh, Sprite,
                AmbientLight, DirectionalLight, HemisphereLight, PointLight};
#[cfg(feature = "opengl")]
pub use window::{Events, Window};
#[cfg(feature = "opengl")]
pub use glutin::VirtualKeyCode as Key;

use std::sync::{mpsc, Arc, Mutex};

use cgmath::Transform as Transform_;
use factory::SceneId;
use render::GpuData;


type Transform = cgmath::Decomposed<cgmath::Vector3<f32>, cgmath::Quaternion<f32>>;

#[derive(Clone, Debug)]
enum SubLight {
    Ambient,
    Directional,
    Hemisphere{ ground: Color },
    Point,
}

#[derive(Clone, Debug)]
enum ShadowProjection {
    Ortho(cgmath::Ortho<f32>),
}

#[derive(Clone, Debug)]
struct LightData {
    color: Color,
    intensity: f32,
    sub_light: SubLight,
    shadow: Option<(ShadowMap, ShadowProjection)>,
}

#[derive(Debug)]
enum SubNode {
    Empty,
    Visual(Material, GpuData),
    Light(LightData),
}

/// Fat node of the scene graph.
///
/// `Node` is used by `three-rs` internally,
/// client code uses [`Object`](struct.Object.html) instead.
#[derive(Debug)]
pub struct Node {
    visible: bool,
    world_visible: bool,
    transform: Transform,
    world_transform: Transform,
    parent: Option<froggy::Pointer<Node>>,
    scene_id: Option<SceneId>,
    sub_node: SubNode,
}

//Note: no local state should be here, only remote links
/// `Object` represents an entity that can be added to the scene.
///
/// There is no need to use `Object` directly, there are specific wrapper types
/// for each case (e.g. [`Camera`](struct.Camera.html),
/// [`AmbientLight`](struct.AmbientLight.html),
/// [`Mesh`](struct.Mesh.html), ...).
#[derive(Clone)]
pub struct Object {
    node: froggy::Pointer<Node>,
    tx: mpsc::Sender<Message>,
}

/// Camera is used to render Scene with specific Projection.
/// See [`OrthographicCamera`](type.OrthographicCamera.html),
/// [`PerspectiveCamera`](type.PerspectiveCamera.html).
pub struct Camera<P> {
    object: Object,
    projection: P,
}

// warning: public exposure of `cgmath` here
/// See [`Orthographic projection`](https://en.wikipedia.org/wiki/3D_projection#Orthographic_projection).
pub type OrthographicCamera = Camera<cgmath::Ortho<f32>>;
/// See [`Perspective projection`](https://en.wikipedia.org/wiki/3D_projection#Perspective_projection).
pub type PerspectiveCamera = Camera<cgmath::PerspectiveFov<f32>>;

/// Generic trait for different graphics projections.
pub trait Projection {
    /// Represents projection as projection matrix.
    fn get_matrix(&self, aspect: f32) -> mint::ColumnMatrix4<f32>;
}

type Message = (froggy::WeakPointer<Node>, Operation);
enum Operation {
    SetParent(froggy::Pointer<Node>),
    SetVisible(bool),
    SetTransform(Option<mint::Point3<f32>>, Option<mint::Quaternion<f32>>, Option<f32>),
    SetMaterial(Material),
    SetTexelRange(mint::Point2<i16>, mint::Vector2<u16>),
    SetShadow(ShadowMap, ShadowProjection),
}

type HubPtr = Arc<Mutex<Hub>>;
struct Hub {
    nodes: froggy::Storage<Node>,
    message_tx: mpsc::Sender<Message>,
    message_rx: mpsc::Receiver<Message>,
}

impl Hub {
    fn new() -> HubPtr {
        let (tx, rx) = mpsc::channel();
        let hub = Hub {
            nodes: froggy::Storage::new(),
            message_tx: tx,
            message_rx: rx,
        };
        Arc::new(Mutex::new(hub))
    }

    fn process_messages(&mut self) {
        while let Ok((pnode, operation)) = self.message_rx.try_recv() {
            let node = match pnode.upgrade() {
                Ok(ptr) => &mut self.nodes[&ptr],
                Err(_) => continue,
            };
            match operation {
                Operation::SetParent(parent) => {
                    node.parent = Some(parent);
                }
                Operation::SetVisible(visible) => {
                    node.visible = visible;
                }
                Operation::SetTransform(pos, rot, scale) => {
                    //TEMP! until mint integration is done in cgmath
                    if let Some(pos) = pos {
                        let p: [f32; 3] = pos.into();
                        node.transform.disp = p.into();
                    }
                    if let Some(rot) = rot {
                        let q: [f32; 3] = rot.v.into();
                        node.transform.rot = cgmath::Quaternion {
                            s: rot.s,
                            v: q.into(),
                        };
                    }
                    if let Some(scale) = scale {
                        node.transform.scale = scale;
                    }
                }
                Operation::SetMaterial(material) => {
                    if let SubNode::Visual(ref mut mat, _) = node.sub_node {
                        *mat = material;
                    }
                }
                Operation::SetTexelRange(base, size) => {
                    if let SubNode::Visual(ref mut material, _) = node.sub_node {
                        match *material {
                            Material::Sprite { ref mut map } => map.set_texel_range(base, size),
                            _ => panic!("Unsupported material for texel range request")
                        }
                    }
                }
                Operation::SetShadow(map, proj) => {
                    if let SubNode::Light(ref mut data) = node.sub_node {
                        data.shadow = Some((map, proj));
                    }
                }
            }
        }
        self.nodes.sync_pending();
    }

    fn update_graph(&mut self) {
        let mut cursor = self.nodes.cursor_alive();
        while let Some(mut item) = cursor.next() {
            if !item.visible {
                item.world_visible = false;
                continue
            }
            let (visibility, affilation, transform) = match item.parent {
                Some(ref parent_ptr) => {
                    let parent = item.look_back(parent_ptr).unwrap();
                    (parent.world_visible, parent.scene_id,
                     parent.world_transform.concat(&item.transform))
                },
                None => (true, item.scene_id, item.transform),
            };
            item.world_visible = visibility;
            item.scene_id = affilation;
            item.world_transform = transform;
        }
    }
}

/// Game scene contains game objects and can be rendered by [`Camera`](struct.Camera.html).
pub struct Scene {
    unique_id: SceneId,
    node: froggy::Pointer<Node>,
    tx: mpsc::Sender<Message>,
    hub: HubPtr,
    /// See [`Background`](struct.Background.html).
    pub background: scene::Background,
}
