use std::ops;

use cgmath::Ortho;
use froggy::Pointer;
use mint;

use {Object, Operation, Node, SubNode,
     Scene, ShadowProjection, Transform};
use factory::{Geometry, ShadowMap, Texture};


pub type Color = u32;

#[derive(Clone, Debug, PartialEq)]
pub enum Background {
    Color(Color),
    //TODO: texture, cubemap
}

#[derive(Clone, Debug)]
pub enum Material {
    LineBasic { color: Color },
    MeshBasic { color: Color, map: Option<Texture<[f32; 4]>>, wireframe: bool },
    MeshLambert { color: Color, flat: bool },
    MeshPhong { color: Color, glossiness: f32 },
    Sprite { map: Texture<[f32; 4]> },
}

/// Position, rotation and scale of the scene [`Node`](struct.Node.html).
#[derive(Clone, Debug)]
pub struct NodeTransform {
    pub position: mint::Point3<f32>,
    pub orientation: mint::Quaternion<f32>,
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

    pub fn set_transform<P, Q>(&mut self, pos: P, rot: Q, scale: f32) where
        P: Into<mint::Point3<f32>>,
        Q: Into<mint::Quaternion<f32>>,
    {
        let msg = Operation::SetTransform(Some(pos.into()), Some(rot.into()), Some(scale));
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    pub fn set_position<P>(&mut self, pos: P) where P: Into<mint::Point3<f32>> {
        let msg = Operation::SetTransform(Some(pos.into()), None, None);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

    pub fn set_orientation<Q>(&mut self, rot: Q) where Q: Into<mint::Quaternion<f32>> {
        let msg = Operation::SetTransform(None, Some(rot.into()), None);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }

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

    pub fn add<P: AsRef<Pointer<Node>>>(&mut self, child: &P) {
        let msg = Operation::SetParent(self.object.node.clone());
        let _ = self.object.tx.send((child.as_ref().downgrade(), msg));
    }
}

/// [`Geometry`](struct.Geometry.html) with some [`Material`](struct.Material.html).
pub struct Mesh {
    object: Object,
    _geometry: Option<Geometry>,
}

impl Mesh {
    #[doc(hidden)]
    pub fn new(object: Object) -> Self {
        Mesh {
            object,
            _geometry: None,
        }
    }

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
    /// # Example
    /// To render only the upper-left quater of the texture with size `256`x`256`
    /// you should write something similar:
    ///
    /// ```rust
    /// // Create new `Window`
    /// let mut win = three::Window::new("Three-rs sprite example", "data/shaders");
    /// // Create sprite map by loading it from file
    /// let material = three::Material::Sprite {
    ///    map: win.factory.load_texture("my_data/some_sprite.png"),
    /// };
    /// // Create sprite and add it to the scene
    /// let mut sprite = win.factory.sprite(material);
    /// win.scene.add(&sprite);
    /// // Set it to render only upper-left quater.
    /// sprite.set_texel_size([0, 0], [128, 128]);
    /// ```
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
                      width: f32, height: f32, near: f32, far: f32) {
        let sp = ShadowProjection::Ortho(Ortho {
            left: -0.5 * width,
            right: 0.5 * width,
            bottom: -0.5 * height,
            top: 0.5 * height,
            near,
            far,
        });
        self.shadow = Some(map.clone());
        let msg = Operation::SetShadow(map, sp);
        let _ = self.tx.send((self.node.downgrade(), msg));
    }
}

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
    pub fn add<P: AsRef<Pointer<Node>>>(&mut self, child: &P) {
        let msg = Operation::SetParent(self.node.clone());
        let _ = self.tx.send((child.as_ref().downgrade(), msg));
    }
}

macro_rules! as_node {
    ($($name:ident),*) => {
        $(
            impl AsRef<Pointer<Node>> for $name {
                fn as_ref(&self) -> &Pointer<Node> {
                    &self.node
                }
            }
        )*
    }
}

as_node!(Object, Group, Mesh, Sprite,
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

deref_objects!(Group, Mesh, Sprite,
    AmbientLight, HemisphereLight, DirectionalLight, PointLight);
