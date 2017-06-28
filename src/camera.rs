#![allow(missing_docs)] //TODO

use std::ops;

use cgmath::{ortho as cgmath_ortho, perspective as cgmath_perspective,
             Deg, Rad, Decomposed, Point3, Vector3, Quaternion,
             EuclideanSpace, InnerSpace, Rotation, Rotation3,
             Transform as Transform_};
use mint;

use {Camera, NodePointer, Object, Transform};
use input::{MOUSE_LEFT, Input, Button};


impl<P> AsRef<NodePointer> for Camera<P> {
    fn as_ref(&self) -> &NodePointer {
        &self.object.node
    }
}

impl<P> ops::Deref for Camera<P> {
    type Target = Object;
    fn deref(&self) -> &Object {
        &self.object
    }
}
impl<P> ops::DerefMut for Camera<P> {
    fn deref_mut(&mut self) -> &mut Object {
        &mut self.object
    }
}

/// Generic trait for different graphics projections.
pub trait Projection {
    /// Represents projection as projection matrix.
    fn get_matrix(&self, aspect: f32) -> mint::ColumnMatrix4<f32>;
}


/// Orthographic projection parameters.
/// See [`Orthographic projection`](https://en.wikipedia.org/wiki/3D_projection#Orthographic_projection).
#[derive(Clone, Debug, PartialEq)]
pub struct Orthographic {
    pub center: mint::Point2<f32>,
    /// Vertical extent from the center point. The height is double the extent.
    /// The width is derived from the height based on the current aspect ratio.
    pub extent_y: f32,
    pub near: f32,
    pub far: f32,
}

impl Projection for Orthographic {
    fn get_matrix(&self, aspect: f32) -> mint::ColumnMatrix4<f32> {
        let extent_x = aspect * self.extent_y;
        let m: [[f32; 4]; 4];
        m = cgmath_ortho(self.center.x - extent_x,
                         self.center.x + extent_x,
                         self.center.y - self.extent_y,
                         self.center.y + self.extent_y,
                         self.near, self.far
                         ).into();
        m.into()
    }
}


/// Perspective projection parameters.
/// See [`Perspective projection`](https://en.wikipedia.org/wiki/3D_projection#Perspective_projection).
#[derive(Clone, Debug, PartialEq)]
pub struct Perspective {
    /// Vertical field of view in degrees.
    ///Note: the horizontal FOV is computed based on the aspect.
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,
}

impl Projection for Perspective {
    fn get_matrix(&self, aspect: f32) -> mint::ColumnMatrix4<f32> {
        let m: [[f32; 4]; 4];
        m = cgmath_perspective(Deg(self.fov_y),
                               aspect, self.near, self.far
                               ).into();
        m.into()
    }
}


pub struct OrbitControls {
    object: Object,
    transform: Transform,
    target: Point3<f32>,
    mouse_base: Option<mint::Point2<f32>>,
    button: Button,
    speed: f32,
}

impl OrbitControls {
    pub fn new<P>(object: &Object, position: P, target: P) -> Self
    where P: Into<[f32; 3]>,
    {
        let pf: [f32; 3] = position.into();
        let tf: [f32; 3] = target.into();

        let dir = (Point3::from(pf) - Point3::from(tf)).normalize();
        let up = Vector3::unit_z();
        let q = Quaternion::look_at(dir, up).invert();
        //TEMP
        let qv: [f32; 3] = q.v.into();
        let rot = mint::Quaternion {
            s: q.s,
            v: qv.into(),
        };
        let mut object = object.clone();
        object.set_transform(pf, rot, 1.0);

        OrbitControls {
            object,
            transform: Decomposed {
                disp: pf.into(),
                rot: q,
                scale: 1.0,
            },
            target: tf.into(),
            mouse_base: None,
            button: MOUSE_LEFT,
            speed: 1.0,
        }
    }

    pub fn update(&mut self, input: &Input) {
        if !self.button.is_hit(input) {
            self.mouse_base = None;
            return
        }

        let cur = input.get_mouse_pos();
        if let Some(base) = self.mouse_base {
            let pre = Decomposed {
                disp: -self.target.to_vec(),
                .. Decomposed::one()
            };
            let q_ver = Quaternion::from_angle_y(Rad(self.speed * (base.x - cur.x)));
            let axis = self.transform.rot * Vector3::unit_x();
            let q_hor = Quaternion::from_axis_angle(axis, Rad(self.speed * (cur.y - base.y)));
            let post = Decomposed {
                scale: 1.0,
                rot: q_hor * q_ver,
                disp: self.target.to_vec(),
            };
            self.transform = post.concat(&pre.concat(&self.transform));
            //TEMP
            let pf: [f32; 3] = self.transform.disp.into();
            let qv: [f32; 3] = self.transform.rot.v.into();
            let rot = mint::Quaternion {
                s: self.transform.rot.s,
                v: qv.into(),
            };
            self.object.set_transform(pf, rot, 1.0);
        }
        self.mouse_base = Some(cur);
    }
}
