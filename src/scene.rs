use std::ops;

use mint;

use {Object, Operation, NodePointer, SubNode,
     Mesh, DynamicMesh,
     Scene, ShadowProjection, Transform};
use camera::Orthographic;
use factory::{ShadowMap, Texture};
use render::BasicPipelineState;

/// Color represented by 4-bytes hex number.
pub type Color = u32;

/// Background type.
#[derive(Clone, Debug, PartialEq)]
pub enum Background {
    /// Basic solid color background.
    Color(Color),
    //TODO: texture, cubemap
}

/// Material is the enhancement of Texture that is used to setup appearance of [`Mesh`](struct.Mesh.html).
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum Material {
    /// Basic wireframe with specific `Color`.
    LineBasic { color: Color },
    /// Basic material with color, optional `Texture` and optional wireframe mode.
    MeshBasic {
        color: Color,
        map: Option<Texture<[f32; 4]>>,
        wireframe: bool,
    },
    /// Lambertian diffuse reflection. This technique causes all closed polygons
    /// (such as a triangle within a 3D mesh) to reflect light equally in all
    /// directions when rendered.
    MeshLambert { color: Color, flat: bool },
    /// Material that uses Phong reflection model.
    MeshPhong { color: Color, glossiness: f32 },
    /// Physically-based rendering material.
    MeshPbr {
        base_color_factor: [f32; 4],
        metallic_roughness: [f32; 2],
        occlusion_strength: f32,
        emissive_factor: [f32; 3],
        normal_scale: f32,

        base_color_map: Option<Texture<[f32; 4]>>,
        normal_map: Option<Texture<[f32; 4]>>,
        emissive_map: Option<Texture<[f32; 4]>>,
        metallic_roughness_map: Option<Texture<[f32; 4]>>,
        occlusion_map: Option<Texture<[f32; 4]>>,
    },
    /// 2D Sprite.
    Sprite { map: Texture<[f32; 4]> },
    /// Custom material.
    CustomBasicPipeline {
        color: Color,
        map: Option<Texture<[f32; 4]>>,
        pipeline: BasicPipelineState,
    },
}

/// Position, rotation and scale of the scene [`Node`](struct.Node.html).
#[derive(Clone, Debug)]
pub struct NodeTransform {
    /// Position.
    pub position: mint::Point3<f32>,
    /// Orientation.
    pub orientation: mint::Quaternion<f32>,
    /// Scale.
    pub scale: f32,
}

impl From<Transform> for NodeTransform {
    fn from(tf: Transform) -> Self {
        let p: [f32; 3] = tf.disp.into();
        let v: [f32; 3] = tf.rot.v.into();
        NodeTransform {
            position: p.into(),
            orientation: mint::Quaternion {
                v: v.into(),
                s: tf.rot.s,
            },
            scale: tf.scale,
        }
    }
}

/// General information about scene [`Node`](struct.Node.html).
#[derive(Clone, Debug)]
pub struct NodeInfo {
    /// Relative to parent transform.
    pub transform: NodeTransform,
    /// World transform (relative to the world's origin).
    pub world_transform: NodeTransform,
    /// Is `Node` visible by cameras or not?
    pub visible: bool,
    /// The same as `visible`, used internally.
    pub world_visible: bool,
    /// Material in case this `Node` has it.
    pub material: Option<Material>,
}


impl Object {
    /// Invisible objects are not rendered by cameras.
    pub fn set_visible(&mut self, visible: bool) {
        let msg = Operation::SetVisible(visible);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    /// Rotates object in the specific direction of `target`.
    pub fn look_at<P>(&mut self, eye: P, target: P, up: Option<mint::Vector3<f32>>)
    where P: Into<[f32; 3]>
    {
        use cgmath::{InnerSpace, Point3, Quaternion, Rotation, Vector3};
        //TEMP
        let p: [[f32; 3]; 2] = [eye.into(), target.into()];
        let dir = (Point3::from(p[0]) - Point3::from(p[1])).normalize();
        let z = Vector3::unit_z();
        let up = match up {
            Some(v) => {
                let vf: [f32; 3] = v.into();
                Vector3::from(vf).normalize()
            },
            None if dir.dot(z).abs() < 0.99 => z,
            None => Vector3::unit_y(),
        };
        let q = Quaternion::look_at(dir, up).invert();
        let qv: [f32; 3] = q.v.into();
        let rot = mint::Quaternion {
            s: q.s,
            v: qv.into(),
        };
        self.set_transform(p[0], rot, 1.0);
    }

    /// Set both position, orientation and scale.
    pub fn set_transform<P, Q>(&mut self, pos: P, rot: Q, scale: f32) where
        P: Into<mint::Point3<f32>>,
        Q: Into<mint::Quaternion<f32>>,
    {
        let msg = Operation::SetTransform(Some(pos.into()), Some(rot.into()), Some(scale));
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    /// Set position.
    pub fn set_position<P>(&mut self, pos: P) where P: Into<mint::Point3<f32>> {
        let msg = Operation::SetTransform(Some(pos.into()), None, None);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    /// Set orientation.
    pub fn set_orientation<Q>(&mut self, rot: Q) where Q: Into<mint::Quaternion<f32>> {
        let msg = Operation::SetTransform(None, Some(rot.into()), None);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    /// Set scale.
    pub fn set_scale(&mut self, scale: f32) {
        let msg = Operation::SetTransform(None, None, Some(scale));
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    /// Get actual information about itself from the `scene`.
    /// # Panics
    /// Panics if `scene` doesn't have this `Object`.
    pub fn sync(&mut self, scene: &Scene) -> NodeInfo {
        let mut hub = scene.hub.lock().unwrap();
        hub.process_messages();
        hub.update_graph();
        let node = &hub.nodes[&self.node];
        assert_eq!(node.scene_id, Some(scene.unique_id));
        NodeInfo {
            transform: node.transform.into(),
            world_transform: node.world_transform.into(),
            visible: node.visible,
            world_visible: node.world_visible,
            material: match node.sub_node {
                SubNode::Visual(ref mat, _) => Some(mat.clone()),
                _ => None,
            },
        }
    }
}

/// Groups are used to combine several other objects or groups to work with them
/// as with a single entity.
#[derive(Debug)]
pub struct Group {
    object: Object,
}

impl Group {
    #[doc(hidden)]
    pub fn new(object: Object) -> Self {
        Group {
            object,
        }
    }

    /// Add new [`Object`](struct.Object.html) to the group.
    pub fn add<P: AsRef<NodePointer>>(&mut self, child: &P) {
        let msg = Operation::SetParent(self.object.node.clone());
        let _ = self.object.tx.send((child.as_ref().downgrade(), msg));
    }
}

impl Mesh {
    /// Set mesh material.
    pub fn set_material(&mut self, material: Material) {
        let msg = Operation::SetMaterial(material);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }
}

impl DynamicMesh {
    /// Set mesh material.
    pub fn set_material(&mut self, material: Material) {
        let msg = Operation::SetMaterial(material);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }
}

/// Two-dimensional bitmap that is integrated into a larger scene.
pub struct Sprite {
    object: Object,
}

impl Sprite {
    #[doc(hidden)]
    pub fn new(object: Object) -> Self {
        Sprite {
            object,
        }
    }

    /// Set area of the texture to render. It can be used in sequential animations.
    pub fn set_texel_range<P, S>(&mut self, base: P, size: S) where
        P: Into<mint::Point2<i16>>,
        S: Into<mint::Vector2<u16>>,
    {
        let msg = Operation::SetTexelRange(base.into(), size.into());
        let _ = self.object.tx.send((self.node.downgrade(), msg));
    }
}

/// Omni-directional, fixed-intensity and fixed-color light source that affects
/// all objects in the scene equally.
pub struct AmbientLight {
    object: Object,
}

impl AmbientLight {
    #[doc(hidden)]
    pub fn new(object: Object) -> Self {
        AmbientLight {
            object,
        }
    }
}

/// The light source that illuminates all objects equally from a given direction,
/// like an area light of infinite size and infinite distance from the scene;
/// there is shading, but cannot be any distance falloff.
pub struct DirectionalLight {
    object: Object,
    shadow: Option<ShadowMap>,
}

impl DirectionalLight {
    #[doc(hidden)]
    pub fn new(object: Object) -> Self {
        DirectionalLight {
            object,
            shadow: None,
        }
    }

    /// Returns `true` if it has [`ShadowMap`](struct.ShadowMap.html), `false` otherwise.
    pub fn has_shadow(&self) -> bool {
        self.shadow.is_some()
    }

    /// Adds shadow map for this light source.
    pub fn set_shadow(&mut self, map: ShadowMap,
                      extent_y: f32, near: f32, far: f32) {
        let sp = ShadowProjection::Ortho(Orthographic {
            center: [0.0; 2].into(),
            extent_y,
            near,
            far,
        });
        self.shadow = Some(map.clone());
        let msg = Operation::SetShadow(map, sp);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }
}

/// `HemisphereLight` uses two different colors in opposite to
/// [`AmbientLight`](struct.AmbientLight.html).
///
/// The color of each fragment is determined by direction of normal. If the
/// normal points in the direction of the upper hemisphere, the fragment has
/// color of the "sky". If the direction of the normal is opposite, then fragment
/// takes color of the "ground". In other cases, color is determined as
/// interpolation between colors of upper and lower hemispheres, depending on
/// how much the normal is oriented to the upper and the lower hemisphere.
pub struct HemisphereLight {
    object: Object,
}

impl HemisphereLight {
    #[doc(hidden)]
    pub fn new(object: Object) -> Self {
        HemisphereLight {
            object,
        }
    }
}

/// Light originates from a single point, and spreads outward in all directions.
pub struct PointLight {
    object: Object,
}

impl PointLight {
    #[doc(hidden)]
    pub fn new(object: Object) -> Self {
        PointLight {
            object,
        }
    }
}


impl Scene {
    /// Add new [`Object`](struct.Object.html) to the scene.
    pub fn add<P: AsRef<NodePointer>>(&mut self, child: &P) {
        let msg = Operation::SetParent(self.node.clone());
        let _ = self.tx.send((child.as_ref().downgrade(), msg));
    }
}

macro_rules! as_node {
    ($($name:ident),*) => {
        $(
            impl AsRef<NodePointer> for $name {
                fn as_ref(&self) -> &NodePointer {
                    &self.node
                }
            }
        )*
    }
}

as_node!(Object, Group, Mesh, DynamicMesh, Sprite,
         AmbientLight, DirectionalLight, HemisphereLight, PointLight);

macro_rules! deref_objects {
    ($( $name:ident ),*) => {
        $(
            impl ops::Deref for $name {
                type Target = Object;
                fn deref(&self) -> &Object {
                    &self.object
                }
            }

            impl ops::DerefMut for $name {
                fn deref_mut(&mut self) -> &mut Object {
                    &mut self.object
                }
            }
        )*
    }
}

deref_objects!(Group, Mesh, DynamicMesh, Sprite,
    AmbientLight, HemisphereLight, DirectionalLight, PointLight);
