use cgmath::{Decomposed, Point3, Quaternion, Rad, Vector3};
use cgmath::{EuclideanSpace, InnerSpace, Rotation, Rotation3, Transform as Transform_};
use mint;
use object;

use input::{Button, Input, MOUSE_LEFT};
use node::TransformInternal;
use object::Object;

/// Simple controls for Orbital Camera.
///
/// Camera is rotating around the fixed point without any restrictions.
/// By default, it uses left mouse button as control button (hold it to rotate) and mouse wheel
/// to adjust distance to the central point.
#[derive(Clone, Debug)]
pub struct Orbit {
    object: object::Base,
    transform: TransformInternal,
    initial_transform: TransformInternal,
    target: Point3<f32>,
    button: Button,
    speed: f32,
}

/// Helper struct to construct [`Orbit`](struct.Orbit.html) with desired settings.
#[derive(Clone, Debug)]
pub struct Builder {
    object: object::Base,
    position: mint::Point3<f32>,
    up: mint::Vector3<f32>,
    target: mint::Point3<f32>,
    button: Button,
    speed: f32,
}

impl Builder {
    /// Create new `Builder` with default values.
    pub fn new<T: Object>(object: &T) -> Self {
        Builder {
            object: object.upcast(),
            position: [0.0, 0.0, 0.0].into(),
            up: [0.0, 0.0, 1.0].into(),
            target: [0.0, 0.0, 0.0].into(),
            button: MOUSE_LEFT,
            speed: 1.0,
        }
    }

    /// Set the initial position.
    ///
    /// Defaults to the world origin.
    pub fn position<P>(&mut self, position: P) -> &mut Self
    where
        P: Into<mint::Point3<f32>>,
    {
        self.position = position.into();
        self
    }

    /// Sets the initial up direction.
    ///
    /// Defaults to the unit z axis.
    pub fn up<P>(&mut self, up: P) -> &mut Self
    where
        P: Into<mint::Vector3<f32>>,
    {
        self.up = up.into();
        self
    }

    /// Set the target position.
    ///
    /// Defaults to the world origin.
    pub fn target<P>(&mut self, target: P) -> &mut Self
    where
        P: Into<mint::Point3<f32>>,
    {
        self.target = target.into();
        self
    }

    /// Setup the speed of the movements. Default value is 1.0
    pub fn speed(&mut self, speed: f32) -> &mut Self {
        self.speed = speed;
        self
    }

    /// Setup control button. Default is left mouse button (`MOUSE_LEFT`).
    pub fn button(&mut self, button: Button) -> &mut Self {
        self.button = button;
        self
    }

    /// Finalize builder and create new `OrbitControls`.
    pub fn build(&mut self) -> Orbit {
        let dir = (Point3::from(self.position) - Point3::from(self.target)).normalize();
        let up = self.up;
        let q = Quaternion::look_at(dir, up.into()).invert();
        let object = self.object.clone();
        object.set_transform(self.position, q, 1.0);
        let transform = Decomposed {
            disp: mint::Vector3::from(self.position).into(),
            rot: q,
            scale: 1.0,
        };

        Orbit {
            object,
            transform,
            initial_transform: transform,
            target: self.target.into(),
            button: self.button,
            speed: self.speed,
        }
    }
}

impl Orbit {
    /// Create new `Builder` with default values.
    pub fn builder<T: Object>(object: &T) -> Builder {
        Builder::new(object)
    }

    /// Update current position and rotation of the controlled object according to the last frame input.
    pub fn update(&mut self, input: &Input) {
        let mouse_delta = if input.hit(self.button) {
            input.mouse_delta_ndc()
        } else {
            [0.0, 0.0].into()
        };
        let pre = Decomposed {
            disp: -self.target.to_vec(),
            ..Decomposed::one()
        };
        let q_ver = Quaternion::from_angle_y(Rad(self.speed * (mouse_delta.x)));
        let axis = self.transform.rot * Vector3::unit_x();
        let q_hor = Quaternion::from_axis_angle(axis, Rad(self.speed * (mouse_delta.y)));
        let post = Decomposed {
            scale: 1.0 + input.mouse_wheel() / 1000.0,
            rot: q_hor * q_ver,
            disp: self.target.to_vec(),
        };
        self.transform = post.concat(&pre.concat(&self.transform));
        let pf: mint::Vector3<f32> = self.transform.disp.into();
        self.object.set_transform(pf, self.transform.rot, 1.0);
    }

    /// Reset the current position and orientation of the controlled object to their initial values.
    pub fn reset(&mut self) {
        self.transform = self.initial_transform;
    }
}
