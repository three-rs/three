use cgmath::{Decomposed, Point3, Quaternion, Rad, Vector3};
use cgmath::{EuclideanSpace, InnerSpace, Rotation, Rotation3, Transform as Transform_};
use mint;

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
    object: Object,
    transform: TransformInternal,
    target: Point3<f32>,
    button: Button,
    speed: f32,
}

/// Helper struct to construct [`Orbit`](struct.Orbit.html) with desired settings.
#[derive(Clone, Debug)]
pub struct Builder {
    object: Object,
    position: mint::Point3<f32>,
    target: mint::Point3<f32>,
    button: Button,
    speed: f32,
}

impl Builder {
    /// Create new `Builder` with default values.
    pub fn new<T>(object: T) -> Self
        where T: AsRef<Object>
    {
        Builder {
            object: object.as_ref().clone(),
            position: [0.0, 0.0, 0.0].into(),
            target: [0.0, 0.0, 0.0].into(),
            button: MOUSE_LEFT,
            speed: 1.0,
        }
    }

    /// Set the initial position.
    ///
    /// Defaults to the world origin.
    pub fn position<P>(
        &mut self,
        position: P,
    ) -> &mut Self
    where
        P: Into<mint::Point3<f32>>,
    {
        self.position = position.into();
        self
    }

    /// Set the target position.
    ///
    /// Defaults to the world origin.
    pub fn target<P>(
        &mut self,
        target: P,
    ) -> &mut Self
    where
        P: Into<mint::Point3<f32>>,
    {
        self.target = target.into();
        self
    }

    /// Setup the speed of the movements. Default value is 1.0
    pub fn speed(
        &mut self,
        speed: f32,
    ) -> &mut Self {
        self.speed = speed;
        self
    }

    /// Setup control button. Default is left mouse button (`MOUSE_LEFT`).
    pub fn button(
        &mut self,
        button: Button,
    ) -> &mut Self {
        self.button = button;
        self
    }

    /// Finalize builder and create new `OrbitControls`.
    pub fn build(&mut self) -> Orbit {
        let dir = (Point3::from(self.position) - Point3::from(self.target)).normalize();
        let up = Vector3::unit_z();
        let q = Quaternion::look_at(dir, up).invert();
        let mut object = self.object.clone();
        object.set_transform(self.position, q, 1.0);

        Orbit {
            object,
            transform: Decomposed {
                disp: mint::Vector3::from(self.position).into(),
                rot: q,
                scale: 1.0,
            },
            target: self.target.into(),
            button: self.button,
            speed: self.speed,
        }
    }
}

impl Orbit {
    /// Create new `Builder` with default values.
    pub fn builder<T>(object: T) -> Builder
        where T: AsRef<Object>
    {
        Builder::new(object)
    }

    /// Update current position and rotation of the controlled object according to the last frame input.
    pub fn update(
        &mut self,
        input: &Input,
    ) {
        if !input.hit(self.button) && input.mouse_wheel().abs() < 1e-6 {
            return;
        }

        if input.mouse_movements().len() > 0 {
            let mouse_delta = input.mouse_delta_ndc();
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
    }
}
