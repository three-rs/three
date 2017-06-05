extern crate cgmath;
extern crate froggy;
extern crate genmesh;
#[macro_use]
extern crate gfx;
extern crate image;
#[macro_use]
extern crate log;
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
pub use scene::{Color, Background, Material, WorldNode,
                Group, Mesh, Sprite,
                AmbientLight, DirectionalLight, HemisphereLight, PointLight};
#[cfg(feature = "opengl")]
pub use window::{Events, Window};
#[cfg(feature = "opengl")]
pub use glutin::VirtualKeyCode as Key;

use std::sync::{mpsc, Arc, Mutex};

use cgmath::Transform as Transform_;
use factory::SceneId;
use render::{ConstantBuffer, GpuData};


pub type Position = cgmath::Point3<f32>;
pub type Vector = cgmath::Vector3<f32>;
pub type Normal = cgmath::Vector3<f32>;
pub type Orientation = cgmath::Quaternion<f32>;
pub type Transform = cgmath::Decomposed<Vector, Orientation>;


#[derive(Debug)]
struct VisualData<T> {
    material: Material,
    gpu_data: GpuData,
    payload: T,
}

impl<T> VisualData<T> {
    fn drop_payload(&self) -> VisualData<()> {
        VisualData {
            material: self.material.clone(),
            gpu_data: self.gpu_data.clone(),
            payload: (),
        }
    }
}

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
    Visual(VisualData<ConstantBuffer>),
    Light(LightData),
}

/// Fat node of the scene graph.
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

pub struct Object {
    visible: bool,
    transform: Transform,
    node: froggy::Pointer<Node>,
    tx: mpsc::Sender<Message>,
}

pub struct VisualObject {
    inner: Object,
    data: VisualData<()>,
}

pub struct LightObject {
    inner: Object,
    data: LightData,
}

pub struct Camera<P> {
    object: Object,
    projection: P,
}

pub type OrthographicCamera = Camera<cgmath::Ortho<f32>>;
pub type PerspectiveCamera = Camera<cgmath::PerspectiveFov<f32>>;

pub trait Projection {
    fn get_matrix(&self, aspect: f32) -> cgmath::Matrix4<f32>;
}

type Message = (froggy::WeakPointer<Node>, Operation);
enum Operation {
    SetParent(froggy::Pointer<Node>),
    SetVisible(bool),
    SetTransform(Transform),
    SetMaterial(Material),
    SetTexelRange([i16; 2], [u16; 2]),
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
                Operation::SetTransform(transform) => {
                    node.transform = transform;
                }
                Operation::SetMaterial(material) => {
                    if let SubNode::Visual(ref mut data) = node.sub_node {
                        data.material = material;
                    }
                }
                Operation::SetTexelRange(base, size) => {
                    if let SubNode::Visual(ref mut data) = node.sub_node {
                        match data.material {
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

pub struct Scene {
    unique_id: SceneId,
    node: froggy::Pointer<Node>,
    tx: mpsc::Sender<Message>,
    hub: HubPtr,
    pub background: scene::Background,
}
