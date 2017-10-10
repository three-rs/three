use audio::{AudioData, Operation as AudioOperation};
use light::{ShadowMap, ShadowProjection};
use material::Material;
use mesh::DynamicMesh;
use node::{Node, NodePointer};
use object::Object;
use render::GpuData;
use scene::Color;
use text::{Operation as TextOperation, TextData};

use cgmath::Transform;
use froggy;
use mint;

use std::sync::{Arc, Mutex};
use std::sync::mpsc;

#[derive(Clone, Debug)]
pub(crate) enum SubLight {
    Ambient,
    Directional,
    Hemisphere { ground: Color },
    Point,
}

#[derive(Clone, Debug)]
pub(crate) struct LightData {
    pub(crate) color: Color,
    pub(crate) intensity: f32,
    pub(crate) sub_light: SubLight,
    pub(crate) shadow: Option<(ShadowMap, ShadowProjection)>,
}

/// A sub-node specifies and contains the context-specific data owned by a `Node`.
#[derive(Debug)]
pub(crate) enum SubNode {
    /// No extra data, such as in the case of `Group`.
    Empty,
    /// Audio data.
    Audio(AudioData),
    /// Renderable text for 2D user interface.
    UiText(TextData),
    /// Renderable 3D content, such as a mesh.
    Visual(Material, GpuData),
    /// Lighting information for illumination and shadow casting.
    Light(LightData),
}

pub(crate) type Message = (froggy::WeakPointer<Node>, Operation);
pub(crate) enum Operation {
    SetAudio(AudioOperation),
    SetParent(NodePointer),
    SetVisible(bool),
    SetText(TextOperation),
    SetTransform(Option<mint::Point3<f32>>, Option<mint::Quaternion<f32>>, Option<f32>),
    SetMaterial(Material),
    SetTexelRange(mint::Point2<i16>, mint::Vector2<u16>),
    SetShadow(ShadowMap, ShadowProjection),
}

pub(crate) type HubPtr = Arc<Mutex<Hub>>;

pub(crate) struct Hub {
    pub(crate) nodes: froggy::Storage<Node>,
    pub(crate) message_tx: mpsc::Sender<Message>,
    message_rx: mpsc::Receiver<Message>,
}

impl Hub {
    pub(crate) fn new() -> HubPtr {
        let (tx, rx) = mpsc::channel();
        let hub = Hub {
            nodes: froggy::Storage::new(),
            message_tx: tx,
            message_rx: rx,
        };
        Arc::new(Mutex::new(hub))
    }

    fn spawn(
        &mut self,
        sub: SubNode,
    ) -> Object {
        Object {
            node: self.nodes.create(sub.into()),
            tx: self.message_tx.clone(),
        }
    }

    pub(crate) fn spawn_empty(&mut self) -> Object {
        self.spawn(SubNode::Empty)
    }

    pub(crate) fn spawn_visual(
        &mut self,
        mat: Material,
        gpu_data: GpuData,
    ) -> Object {
        self.spawn(SubNode::Visual(mat, gpu_data))
    }

    pub(crate) fn spawn_light(
        &mut self,
        data: LightData,
    ) -> Object {
        self.spawn(SubNode::Light(data))
    }

    pub(crate) fn spawn_ui_text(
        &mut self,
        text: TextData,
    ) -> Object {
        self.spawn(SubNode::UiText(text))
    }

    pub(crate) fn spawn_audio_source(
        &mut self,
        data: AudioData,
    ) -> Object {
        self.spawn(SubNode::Audio(data))
    }

    pub(crate) fn process_messages(&mut self) {
        while let Ok((pnode, operation)) = self.message_rx.try_recv() {
            let node = match pnode.upgrade() {
                Ok(ptr) => &mut self.nodes[&ptr],
                Err(_) => continue,
            };
            match operation {
                Operation::SetAudio(operation) => {
                    if let SubNode::Audio(ref mut data) = node.sub_node {
                        Hub::process_audio(operation, data);
                    }
                }
                Operation::SetParent(parent) => {
                    node.parent = Some(parent);
                }
                Operation::SetVisible(visible) => {
                    node.visible = visible;
                }
                Operation::SetTransform(pos, rot, scale) => {
                    if let Some(pos) = pos {
                        node.transform.disp = mint::Vector3::from(pos).into();
                    }
                    if let Some(rot) = rot {
                        node.transform.rot = rot.into();
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
                            _ => panic!("Unsupported material for texel range request"),
                        }
                    }
                }
                Operation::SetText(operation) => {
                    if let SubNode::UiText(ref mut data) = node.sub_node {
                        Hub::process_text(operation, data);
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

    fn process_audio(
        operation: AudioOperation,
        data: &mut AudioData,
    ) {
        match operation {
            AudioOperation::Append(clip) => data.source.append(clip),
            AudioOperation::Pause => data.source.pause(),
            AudioOperation::Resume => data.source.resume(),
            AudioOperation::Stop => data.source.stop(),
            AudioOperation::SetVolume(volume) => data.source.set_volume(volume),
        }
    }

    fn process_text(
        operation: TextOperation,
        data: &mut TextData,
    ) {
        use gfx_glyph::Scale;
        match operation {
            TextOperation::Color(color) => {
                use util::decode_color;
                let mut color = decode_color(color);
                color[3] = data.section.color[3];
                data.section.color = color;
            }
            TextOperation::Font(font) => data.font = font,
            TextOperation::Layout(layout) => data.layout = layout,
            TextOperation::Opacity(opacity) => data.section.color[3] = opacity,
            TextOperation::Pos(point) => data.section.screen_position = (point.x, point.y),
            // TODO: somehow grab window::hdpi_factor and multiply size
            TextOperation::Scale(scale) => data.section.scale = Scale::uniform(scale),
            TextOperation::Size(size) => data.section.bounds = (size.x, size.y),
            TextOperation::Text(text) => data.section.text = text,
        }
    }

    pub(crate) fn update_graph(&mut self) {
        let mut cursor = self.nodes.cursor();
        while let Some((left, mut item, _)) = cursor.next() {
            if !item.visible {
                item.world_visible = false;
                continue;
            }
            let (visibility, affilation, transform) = match item.parent {
                Some(ref parent_ptr) => {
                    match left.get(parent_ptr) {
                        Some(parent) => (
                            parent.world_visible,
                            parent.scene_id,
                            parent.world_transform.concat(&item.transform),
                        ),
                        None => {
                            error!("Parent node was created after the child, ignoring");
                            (false, item.scene_id, item.transform)
                        }
                    }
                }
                None => (true, item.scene_id, item.transform),
            };
            item.world_visible = visibility;
            item.scene_id = affilation;
            item.world_transform = transform;
        }
    }

    pub(crate) fn update_mesh(
        &mut self,
        mesh: &DynamicMesh,
    ) {
        match self.nodes[&mesh.node].sub_node {
            SubNode::Visual(_, ref mut gpu_data) => gpu_data.pending = Some(mesh.dynamic.clone()),
            _ => unreachable!(),
        }
    }
}
