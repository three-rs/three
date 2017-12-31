use audio::{AudioData, Operation as AudioOperation};
use color::{self, Color};
use light::{ShadowMap, ShadowProjection};
use material::{Material};
use mesh::{DynamicMesh, MAX_TARGETS, Target, Weight};
use node::{NodeInternal, NodePointer, TransformInternal};
use object::{Base};
use render::{BackendResources, GpuData};
use skeleton::{Bone, Skeleton};
use text::{Operation as TextOperation, TextData};

use arrayvec::ArrayVec;
use cgmath::Transform;
use froggy;
use gfx;
use mint;

use std::{mem, ops};
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
    pub color: Color,
    pub intensity: f32,
    pub sub_light: SubLight,
    pub shadow: Option<(ShadowMap, ShadowProjection)>,
}

#[derive(Clone, Debug)]
pub(crate) struct SkeletonData {
    pub bones: Vec<Bone>,
    pub inverse_bind_matrices: Vec<mint::ColumnMatrix4<f32>>,

    pub gpu_buffer_view: gfx::handle::ShaderResourceView<BackendResources, [f32; 4]>,
    pub gpu_buffer: gfx::handle::Buffer<BackendResources, [f32; 4]>,
    pub cpu_buffer: Vec<[f32; 4]>,
}

#[derive(Clone, Debug)]
pub(crate) struct VisualData {
    pub material: Material,
    pub gpu: GpuData,
    pub skeleton: Option<Skeleton>,
}

#[derive(Debug)]
pub(crate) enum SubNode {
    /// No extra data.
    Empty,
    /// Group can be a parent to other objects.
    Group { first_child: Option<NodePointer> },
    /// Audio data.
    Audio(AudioData),
    /// Renderable text for 2D user interface.
    UiText(TextData),
    /// Renderable 3D content, such as a mesh.
    Visual(Material, GpuData, Option<Skeleton>),
    /// Lighting information for illumination and shadow casting.
    Light(LightData),
    /// Array of `Bone` instances that may be bound to a `Skinned` mesh.
    Skeleton(SkeletonData),
}

pub(crate) type Message = (froggy::WeakPointer<NodeInternal>, Operation);

#[derive(Debug)]
pub(crate) enum Operation {
    AddChild(NodePointer),
    RemoveChild(NodePointer),
    SetAudio(AudioOperation),
    SetVisible(bool),
    SetText(TextOperation),
    SetTransform(
        Option<mint::Point3<f32>>,
        Option<mint::Quaternion<f32>>,
        Option<f32>,
    ),
    SetMaterial(Material),
    SetSkeleton(Skeleton),
    SetShadow(ShadowMap, ShadowProjection),
    SetTargets(ArrayVec<[Target; MAX_TARGETS]>),
    SetTexelRange(mint::Point2<i16>, mint::Vector2<u16>),
    SetWeights([DisplacementContribution; MAX_TARGETS]),
}

pub(crate) type HubPtr = Arc<Mutex<Hub>>;

pub(crate) struct Hub {
    pub(crate) nodes: froggy::Storage<NodeInternal>,
    pub(crate) message_tx: mpsc::Sender<Message>,
    message_rx: mpsc::Receiver<Message>,
}

impl<T: AsRef<Base>> ops::Index<T> for Hub {
    type Output = NodeInternal;
    fn index(&self, i: T) -> &Self::Output {
        let base: &Base = i.as_ref();
        &self.nodes[&base.node]
    }
}

impl<T: AsRef<Base>> ops::IndexMut<T> for Hub {
    fn index_mut(&mut self, i: T) -> &mut Self::Output {
        let base: &Base = i.as_ref();
        &mut self.nodes[&base.node]
    }
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

    pub(crate) fn spawn(
        &mut self,
        sub: SubNode,
    ) -> Base {
        Base {
            node: self.nodes.create(sub.into()),
            tx: self.message_tx.clone(),
        }
    }

    pub(crate) fn spawn_visual(
        &mut self,
        mat: Material,
        gpu_data: GpuData,
        skeleton: Option<Skeleton>,
    ) -> Base {
        self.spawn(SubNode::Visual(mat, gpu_data, skeleton))
    }

    pub(crate) fn spawn_light(
        &mut self,
        data: LightData,
    ) -> Base {
        self.spawn(SubNode::Light(data))
    }

    pub(crate) fn spawn_skeleton(
        &mut self,
        data: SkeletonData,
    ) -> Base {
        self.spawn(SubNode::Skeleton(data))
    }

    pub(crate) fn process_messages(&mut self) {
        while let Ok((weak_ptr, operation)) = self.message_rx.try_recv() {
            let ptr = match weak_ptr.upgrade() {
                Ok(ptr) => ptr,
                Err(_) => continue,
            };
            match operation {
                Operation::SetAudio(operation) => {
                    if let SubNode::Audio(ref mut data) = self.nodes[&ptr].sub_node {
                        Hub::process_audio(operation, data);
                    }
                },
                Operation::SetParent(parent) => {
                    self.nodes[&ptr].parent = Some(parent);
                }
                Operation::SetVisible(visible) => {
                    self.nodes[&ptr].visible = visible;
                }
                Operation::SetTransform(pos, rot, scale) => {
                    let transform = &mut self.nodes[&ptr].transform;
                    if let Some(pos) = pos {

                        transform.disp = mint::Vector3::from(pos).into();
                    }
                    if let Some(rot) = rot {
                        transform.rot = rot.into();
                    }
                    if let Some(scale) = scale {
                        transform.scale = scale;
                    }
                }
                Operation::AddChild(child_ptr) => {
                    let sibling = match self.nodes[&ptr].sub_node {
                        SubNode::Group { ref mut first_child } =>
                            mem::replace(first_child, Some(child_ptr.clone())),
                        _ => unreachable!(),
                    };
                    let child = &mut self.nodes[&child_ptr];
                    if child.next_sibling.is_some() {
                        error!("Element {:?} is added to a group while still having old parent - {}",
                            child.sub_node, "discarding siblings");
                    }
                    child.next_sibling = sibling;
                }
                Operation::RemoveChild(child_ptr) => {
                    let next_sibling = self.nodes[&child_ptr].next_sibling.clone();
                    let target_maybe = Some(child_ptr);
                    let mut cur_ptr = match self.nodes[&ptr].sub_node {
                        SubNode::Group { ref mut first_child } => {
                            if *first_child == target_maybe {
                                *first_child = next_sibling;
                                continue;
                            }
                            first_child.clone()
                        }
                        _ => unreachable!()
                    };

                    //TODO: consolidate the code with `Scene::remove()`
                    loop {
                        let node = match cur_ptr.take() {
                            Some(next_ptr) => &mut self.nodes[&next_ptr],
                            None => {
                                error!("Unable to find child for removal");
                                break;
                            }
                        };
                        if node.next_sibling == target_maybe {
                            node.next_sibling = next_sibling;
                            break;
                        }
                        cur_ptr = node.next_sibling.clone(); //TODO: avoid clone
                    }
                }
                Operation::SetAudio(operation) => {
                    match self.nodes[&ptr].sub_node {
                        SubNode::Audio(ref mut data) => {
                            Hub::process_audio(operation, data);
                        }
                        _ => unreachable!()
                    }
                },
                Operation::SetSkeleton(skeleton) => {
                    if let SubNode::Visual(_, _, ref mut s) = self.nodes[&ptr].sub_node {
                        *s = Some(skeleton);
                    }
                },
                Operation::SetShadow(map, proj) => {
                    if let SubNode::Light(ref mut data) = self.nodes[&ptr].sub_node {
                        data.shadow = Some((map, proj));
                    }
                },
                Operation::SetTargets(targets) => {
                    println!("Not yet implemented!");
                },
                Operation::SetWeights(weights) => {
                    match self.nodes[&ptr].sub_node {
                        SubNode::Visual(_, ref mut gpu_data, _) => {
                            gpu_data.displacement_contributions = weights;
                        }
                        _ => println!("Not yet implemented!"),
                    }
                }
                Operation::SetText(operation) => {
                    match self.nodes[&ptr].sub_node {
                        SubNode::UiText(ref mut data) => {
                            Hub::process_text(operation, data);
                        }
                        _ => unreachable!()
                    }
                }
                Operation::SetMaterial(material) => {
                    match self.nodes[&ptr].sub_node {
                        SubNode::Visual(ref mut mat, _, _) => {
                            *mat = material;
                        }
                        _ => unreachable!()
                    }
                }
                Operation::SetSkeleton(sleketon) => {
                    match self.nodes[&ptr].sub_node {
                        SubNode::Visual(_, _, ref mut skel) => {
                            *skel = Some(sleketon);
                        }
                        _ => unreachable!()
                    }
                }
                Operation::SetShadow(map, proj) => {
                    match self.nodes[&ptr].sub_node {
                        SubNode::Light(ref mut data) => {
                            data.shadow = Some((map, proj));
                        },
                    _ => unreachable!()
                    }
                }
                Operation::SetTexelRange(base, size) => {
                    match self.nodes[&ptr].sub_node {
                        SubNode::Visual(Material::Sprite(ref mut params), _, _) => {
                            params.map.set_texel_range(base, size);
                        }
                        _ => unreachable!()
                    }
                }
                Operation::SetTargets(targets) => unimplemented!(),
                Operation::SetWeights(weights) => unimplemented!(),
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
                let rgb = color::to_linear_rgb(color);
                data.section.text[0].color = [rgb[0], rgb[1], rgb[2], 0.0];
            }
            TextOperation::Font(font) => data.font = font,
            TextOperation::Layout(layout) => data.layout = layout,
            TextOperation::Opacity(opacity) => data.section.text[0].color[3] = opacity,
            TextOperation::Pos(point) => data.section.screen_position = (point.x, point.y),
            // TODO: somehow grab window::hdpi_factor and multiply size
            TextOperation::Scale(scale) => data.section.text[0].scale = Scale::uniform(scale),
            TextOperation::Size(size) => data.section.bounds = (size.x, size.y),
            TextOperation::Text(text) => data.section.text[0].text = text,
        }
    }

    pub(crate) fn update_mesh(
        &mut self,
        mesh: &DynamicMesh,
    ) {
        match self[mesh].sub_node {
            SubNode::Visual(_, ref mut gpu_data, _) => gpu_data.pending = Some(mesh.dynamic.clone()),
            _ => unreachable!(),
        }
    }

    fn walk_impl(
        &self, base: &Option<NodePointer>, only_visible: bool
    ) -> TreeWalker {
        let default_stack_size = 10;
        let mut walker = TreeWalker {
            hub: self,
            only_visible,
            stack: Vec::with_capacity(default_stack_size),
        };
        walker.descend(base);
        walker
    }

    pub(crate) fn walk(&self, base: &Option<NodePointer>) -> TreeWalker {
        self.walk_impl(base, true)
    }

    pub(crate) fn walk_all(&self, base: &Option<NodePointer>) -> TreeWalker {
        self.walk_impl(base, false)
    }
}

#[derive(Debug)]
pub(crate) struct WalkedNode<'a> {
    pub(crate) node: &'a NodeInternal,
    pub(crate) world_visible: bool,
    pub(crate) world_transform: TransformInternal,
}

pub(crate) struct TreeWalker<'a> {
    hub: &'a Hub,
    only_visible: bool,
    stack: Vec<WalkedNode<'a>>,
}

impl<'a> TreeWalker<'a> {
    fn descend(&mut self, base: &Option<NodePointer>) -> Option<&NodeInternal> {
        // Note: this is a CPU hotspot, presumably for copying stuff around
        // TODO: profile carefully and optimize
        let mut node = &self.hub.nodes[base.as_ref()?];

        loop {
            let wn = match self.stack.last() {
                Some(parent) => WalkedNode {
                    node,
                    world_visible: parent.world_visible && node.visible,
                    world_transform: parent.world_transform.concat(&node.transform),
                },
                None => WalkedNode {
                    node,
                    world_visible: node.visible,
                    world_transform: node.transform,
                },
            };
            self.stack.push(wn);

            if self.only_visible && !node.visible {
                break;
            }

            node = match node.sub_node {
                SubNode::Group { first_child: Some(ref ptr) } => &self.hub.nodes[ptr],
                _ => break,
            };
        }

        Some(node)
    }
}

impl<'a> Iterator for TreeWalker<'a> {
    type Item = WalkedNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(top) = self.stack.pop() {
            self.descend(&top.node.next_sibling);
            if !self.only_visible || top.world_visible {
                return Some(top)
            }
        }
        None
    }
}
