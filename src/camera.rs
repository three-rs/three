//! Contains useful [`Camera`](struct.Camera.html) struct and `Projections`.
use std::ops;

use cgmath::{ortho as cgmath_ortho, perspective as cgmath_perspective, Deg};
use mint;

use NodePointer;
use object::Object;

/// Camera is used to render Scene with specific `Projection`.
pub struct Camera<P> {
    pub(crate) object: Object,
    /// Projection parameters of this camera.
    pub projection: P,
}

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
    /// The center of the projection.
    pub center: mint::Point2<f32>,
    /// Vertical extent from the center point. The height is double the extent.
    /// The width is derived from the height based on the current aspect ratio.
    pub extent_y: f32,
    /// Distance to the near clip plane.
    pub near: f32,
    /// Distance to the far clip plane.
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
    /// Note: the horizontal FOV is computed based on the aspect.
    pub fov_y: f32,
    /// The distance to the near clip plane.
    pub near: f32,
    /// The distance to the far clip plane.
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
