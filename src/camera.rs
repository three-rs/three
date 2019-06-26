//! Cameras are used to view scenes from any point in the world.
//!
//! ## Projections
//!
//! ### Finite perspective
//!
//! Finite persepective projections are often used for 3D rendering. In a finite
//! perspective projection, objects moving away from the camera appear smaller and
//! are occluded by objects that are closer to the camera.
//!
//! Finite [`Perspective`] projections are created with the
//! [`Factory::perspective_camera`] method with a bounded range.
//!
//! ```rust,no_run
//! # let mut window = three::Window::new("");
//! # let _ = {
//! window.factory.perspective_camera(60.0, 0.1 .. 1.0);
//! # };
//! ```
//!
//! ### Infinite perspective
//!
//! Infinite perspective projections are perspective projections with `zfar` planes
//! at infinity. This means objects are never considered to be 'too far away' to be
//! visible by the camera.
//!
//! Infinite [`Perspective`] projections are created with the
//! [`Factory::perspective_camera`] method with an unbounded range.
//!
//! ```rust,no_run
//! # let mut window = three::Window::new("");
//! # let _ = {
//! window.factory.perspective_camera(60.0, 0.1 ..);
//! # };
//! ```
//!
//! ### Orthographic
//!
//! Orthographic projections are often used for 2D rendering. In an orthographic
//! projection, objects moving away from the camera retain their size but are
//! occluded by objects that are closer to the camera.
//!
//! [`Orthographic`] projections are created with the
//! [`Factory::orthographic_camera`] method.
//!
//! ```rust,no_run
//! # let mut window = three::Window::new("");
//! # let _ = {
//! window.factory.orthographic_camera([0.0, 0.0], 1.0, -1.0 .. 1.0)
//! # };
//! ```
//!
//! [`Factory::orthographic_camera`]: ../factory/struct.Factory.html#method.orthographic_camera
//! [`Factory::perspective_camera`]: ../factory/struct.Factory.html#method.perspective_camera
//! [`object::Base`]: ../object/struct.Base.html
//! [`Orthographic`]: struct.Orthographic.html
//! [`Perspective`]: struct.Perspective.html

use cgmath;
use mint;

use hub::{Hub, Operation, SubNode};
use object::{Base, DowncastObject, Object, ObjectType};
use scene::SyncGuard;

use std::ops;

/// The Z values of the near and far clipping planes of a camera's projection.
#[derive(Clone, Debug, PartialEq)]
pub enum ZRange {
    /// Z range for a finite projection.
    Finite(ops::Range<f32>),

    /// Z range for an infinite projection.
    Infinite(ops::RangeFrom<f32>),
}

impl From<ops::Range<f32>> for ZRange {
    fn from(range: ops::Range<f32>) -> ZRange {
        ZRange::Finite(range)
    }
}

impl From<ops::RangeFrom<f32>> for ZRange {
    fn from(range: ops::RangeFrom<f32>) -> ZRange {
        ZRange::Infinite(range)
    }
}

/// A camera's projection.
#[derive(Clone, Debug, PartialEq)]
pub enum Projection {
    /// An orthographic projection.
    Orthographic(Orthographic),
    /// A perspective projection.
    Perspective(Perspective),
}

/// Camera is used to render Scene with specific [`Projection`].
///
/// [`Projection`]: enum.Projection.html
#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    pub(crate) object: Base,
}

impl AsRef<Base> for Camera {
    fn as_ref(&self) -> &Base {
        &self.object
    }
}

impl Object for Camera {
    type Data = Projection;

    fn resolve_data(
        &self,
        sync_guard: &SyncGuard,
    ) -> Self::Data {
        match &sync_guard.hub[self].sub_node {
            SubNode::Camera(ref projection) => projection.clone(),
            sub_node @ _ => panic!("`Group` had a bad sub node type: {:?}", sub_node),
        }
    }
}

impl Camera {
    pub(crate) fn new(
        hub: &mut Hub,
        projection: Projection,
    ) -> Self {
        Camera { object: hub.spawn(SubNode::Camera(projection)) }
    }

    /// Sets the projection used by the camera.
    pub fn set_projection<P: Into<Projection>>(
        &self,
        projection: P,
    ) {
        self.as_ref().send(Operation::SetProjection(projection.into()));
    }
}

impl DowncastObject for Camera {
    fn downcast(object_type: ObjectType) -> Option<Self> {
        match object_type {
            ObjectType::Camera(camera) => Some(camera),
            _ => None,
        }
    }
}

impl Projection {
    /// Constructs an orthographic projection.
    pub fn orthographic<P>(
        center: P,
        extent_y: f32,
        range: ops::Range<f32>,
    ) -> Self
    where
        P: Into<mint::Point2<f32>>,
    {
        let center = center.into();
        Projection::Orthographic(Orthographic { center, extent_y, range })
    }

    /// Constructs a perspective projection.
    pub fn perspective<R>(
        fov_y: f32,
        range: R,
    ) -> Self
    where
        R: Into<ZRange>,
    {
        Projection::Perspective(Perspective { fov_y, zrange: range.into() })
    }

    /// Computes the projection matrix representing the camera's projection.
    pub fn matrix(
        &self,
        aspect_ratio: f32,
    ) -> mint::ColumnMatrix4<f32> {
        match *self {
            Projection::Orthographic(ref x) => x.matrix(aspect_ratio),
            Projection::Perspective(ref x) => x.matrix(aspect_ratio),
        }
    }
}

/// Orthographic projection parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Orthographic {
    /// The center of the projection.
    pub center: mint::Point2<f32>,
    /// Vertical extent from the center point. The height is double the extent.
    /// The width is derived from the height based on the current aspect ratio.
    pub extent_y: f32,
    /// Distance to the clipping planes.
    pub range: ops::Range<f32>,
}

impl Orthographic {
    /// Computes the projection matrix representing the camera's projection.
    pub fn matrix(
        &self,
        aspect_ratio: f32,
    ) -> mint::ColumnMatrix4<f32> {
        let extent_x = aspect_ratio * self.extent_y;
        cgmath::ortho(self.center.x - extent_x, self.center.x + extent_x, self.center.y - self.extent_y, self.center.y + self.extent_y, self.range.start, self.range.end).into()
    }
}

/// Perspective projection parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Perspective {
    /// Vertical field of view in degrees.
    /// Note: the horizontal FOV is computed based on the aspect.
    pub fov_y: f32,
    /// The distance to the clipping planes.
    pub zrange: ZRange,
}

impl Perspective {
    /// Computes the projection matrix representing the camera's projection.
    pub fn matrix(
        &self,
        aspect_ratio: f32,
    ) -> mint::ColumnMatrix4<f32> {
        match self.zrange {
            ZRange::Finite(ref range) => cgmath::perspective(cgmath::Deg(self.fov_y), aspect_ratio, range.start, range.end).into(),
            ZRange::Infinite(ref range) => {
                let f = 1.0 / (0.5 * self.fov_y.to_radians()).tan();

                let m00 = f / aspect_ratio;
                let m11 = f;
                let m22 = -1.0;
                let m23 = -1.0;
                let m32 = -2.0 * range.start;

                let m = [[m00, 0.0, 0.0, 0.0], [0.0, m11, 0.0, 0.0], [0.0, 0.0, m22, m23], [0.0, 0.0, m32, 0.0]];

                m.into()
            }
        }
    }
}
