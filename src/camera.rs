use std::ops;

use cgmath::prelude::*;
use cgmath;

use {Position, Orientation};


pub trait Camera {
    //TODO: combine into a single method?
    fn to_view_proj(&self) -> cgmath::Matrix4<f32>;
    fn set_aspect(&mut self, f32);
}

#[derive(Clone)]
pub struct PerspectiveCamera {
    projection: cgmath::PerspectiveFov<f32>,
    pub position: Position,
    pub orientation: Orientation,
}

impl PerspectiveCamera {
    pub fn new(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        PerspectiveCamera {
            projection: cgmath::PerspectiveFov {
                fovy: cgmath::Deg(fov).into(),
                aspect: aspect,
                near: near,
                far: far,
            },
            position: Position::origin(),
            orientation: Orientation::one(),
        }
    }

    pub fn look_at(&mut self, target: cgmath::Point3<f32>) {
        let dir = (self.position - target).normalize();
        let z = cgmath::Vector3::unit_z();
        let up = if dir.dot(z).abs() < 0.99 { z } else {
            cgmath::Vector3::unit_y()
        };
        self.orientation = Orientation::look_at(dir, up);
    }
}

impl Camera for PerspectiveCamera {
    fn to_view_proj(&self) -> cgmath::Matrix4<f32> {
        let mx_proj = cgmath::perspective(self.projection.fovy,
            self.projection.aspect, self.projection.near, self.projection.far);
        let transform = cgmath::Decomposed {
            disp: self.position.to_vec(),
            rot: self.orientation,
            scale: 1.0,
        };

        let mx_view = cgmath::Matrix4::from(transform.inverse_transform().unwrap());
        mx_proj * mx_view
    }

    fn set_aspect(&mut self, aspect: f32) {
        self.projection.aspect = aspect;
    }
}

#[derive(Clone)]
pub struct OrthographicCamera {
    projection: cgmath::Ortho<f32>,
    base_aspect: f32,
    pub position: Position,
    pub orientation: Orientation,
}

impl OrthographicCamera {
    pub fn new(left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32) -> Self {
        OrthographicCamera {
            projection: cgmath::Ortho{ left, right, bottom, top, near, far },
            base_aspect: (right - left) / (top - bottom),
            position: Position::origin(),
            orientation: Orientation::one(),
        }
    }
}

impl Camera for OrthographicCamera {
    fn to_view_proj(&self) -> cgmath::Matrix4<f32> {
        let mx_proj = cgmath::ortho(self.projection.left, self.projection.right,
            self.projection.bottom, self.projection.top,
            self.projection.near, self.projection.far);
        let transform = cgmath::Decomposed {
            disp: self.position.to_vec(),
            rot: self.orientation,
            scale: 1.0,
        };

        let mx_view = cgmath::Matrix4::from(transform.inverse_transform().unwrap());
        mx_proj * mx_view
    }

    fn set_aspect(&mut self, aspect: f32) {
        let center = 0.5 * (self.projection.left + self.projection.right);
        let scale = aspect / self.base_aspect;
        self.projection.left = scale * (self.projection.left - center) + center;
        self.projection.right = scale * (self.projection.right - center) + center;
        self.base_aspect = aspect;
    }
}

//TODO: share the macro with `scene`
macro_rules! deref {
    ($name:ty : $field:ident = $object:ty) => {
        impl ops::Deref for $name {
            type Target = $object;
            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }

        impl ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    }
}

deref!(PerspectiveCamera : projection = cgmath::PerspectiveFov<f32>);
deref!(OrthographicCamera : projection = cgmath::Ortho<f32>);
