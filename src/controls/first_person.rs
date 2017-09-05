use cgmath;
use mint;
use std;

use cgmath::Rotation3;
use input::{Button, Input, Key};
use object::Object;

/// Controls for first person camera.
pub struct FirstPerson {
    object: Object,
    position: mint::Point3<f32>,
    yaw: f32,
    pitch: f32,
    move_speed: f32,
    look_speed: f32,
}

/// Constructs custom [`FirstPerson`](struct.FirstPerson.html) controls.
pub struct Builder {
    object: Object,
    position: mint::Point3<f32>,
    yaw: f32,
    pitch: f32,
    move_speed: f32,
    look_speed: f32,
}

impl Builder {
    /// Create new `Builder` with default parameters.
    pub fn new(object: &Object) -> Self {
        Builder {
            object: object.clone(),
            position: [0.0, 0.0, 0.0].into(),
            yaw: 0.0,
            pitch: 0.0,
            move_speed: 1.0,
            look_speed: std::f32::consts::PI / 2.0,
        }
    }

    /// Set the initial yaw angle in radians. Default is 0.0.
    pub fn yaw(&mut self, yaw: f32) -> &mut Self {
        self.yaw = yaw;
        self
    }

    /// Set the initial pitch angle in radians.
    ///
    /// Defaults to 0.0.
    pub fn pitch(&mut self, pitch: f32) -> &mut Self {
        self.pitch = pitch;
        self
    }

    /// Set the initial position.
    ///
    /// Defaults to the world origin.
    pub fn position<P>(&mut self, position: P) -> &mut Self
        where P: Into<mint::Point3<f32>>
    {
        self.position = position.into();
        self
    }

    /// Setup the movement speed in world units per second.
    ///
    /// Defaults to 1.0 world units per second.
    pub fn move_speed(&mut self, speed: f32) -> &mut Self {
        self.move_speed = speed;
        self
    }

    /// Setup the yaw and pitch movement speed in radians per second.
    ///
    /// Defaults to PI/2 radians per second.
    pub fn look_speed(&mut self, speed: f32) -> &mut Self {
        self.look_speed = speed;
        self
    }

    /// Finalize builder and create new `FirstPerson` controls.
    pub fn build(&mut self) -> FirstPerson {
        FirstPerson {
            object: self.object.clone(),
            position: self.position,
            yaw: self.yaw,
            pitch: self.pitch,
            move_speed: self.move_speed,
            look_speed: self.look_speed,
        }
    }
}

impl FirstPerson {
    /// Create a `Builder`.
    pub fn builder(object: &Object) -> Builder {
        Builder::new(object)
    }

    /// Create `FirstPerson` controls with default parameters.
    pub fn default(object: &Object) -> Self {
        Self::builder(object).build()
    }
 
    /// Updates the position, yaw, and pitch of the controlled object according to
    /// the last frame input.
    pub fn update(&mut self, input: &Input) {
        let dtime = input.delta_time();
        let dlook = dtime * self.look_speed;
        let dmove = dtime * self.move_speed;

        if Button::Key(Key::Q).is_hit(input) {
            self.yaw -= dlook;
        }
        if Button::Key(Key::E).is_hit(input) {
            self.yaw += dlook;
        }
        if Button::Key(Key::R).is_hit(input) {
            self.pitch -= dlook;
        }
        if Button::Key(Key::F).is_hit(input) {
            self.pitch += dlook;
        }
        if Button::Key(Key::X).is_hit(input) {
            self.position.y += dmove;
        }
        if Button::Key(Key::Z).is_hit(input) {
            self.position.y -= dmove;
        }
        if Button::Key(Key::W).is_hit(input) {
            self.position.x += dmove * self.yaw.sin();
            self.position.z -= dmove * self.yaw.cos();
        }
        if Button::Key(Key::S).is_hit(input) {
            self.position.x -= dmove * self.yaw.sin();
            self.position.z += dmove * self.yaw.cos();
        }
        if Button::Key(Key::D).is_hit(input) {
            self.position.x += dmove * self.yaw.cos();
            self.position.z += dmove * self.yaw.sin();
        }
        if Button::Key(Key::A).is_hit(input) {
            self.position.x -= dmove * self.yaw.cos();
            self.position.z -= dmove * self.yaw.sin();
        }

        let yrot = cgmath::Quaternion::from_angle_y(cgmath::Rad(-self.yaw));
        let xrot = cgmath::Quaternion::from_angle_x(cgmath::Rad(-self.pitch));
        self.object.set_transform(self.position, yrot * xrot, 1.0);
    }
}
