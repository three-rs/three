#![allow(missing_docs)] //TODO

use std::ops;

use cgmath;
use froggy::Pointer;
use mint;

use {Camera, Node, Object};


impl<P> AsRef<Pointer<Node>> for Camera<P> {
    fn as_ref(&self) -> &Pointer<Node> {
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
        m = cgmath::ortho(self.center.x - extent_x,
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
        m = cgmath::perspective(cgmath::Deg(self.fov_y),
                                aspect, self.near, self.far
                                ).into();
        m.into()
    }
}
