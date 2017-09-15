use cgmath;
use mint;

use cgmath::Rotation3;
use input::{Input, Key, KeyAxis};
use object::Object;

#[derive(Clone, Debug)]
struct Axes {
    pub forward: Option<KeyAxis>,
    pub strafing: Option<KeyAxis>,
    pub vertical: Option<KeyAxis>,
}

impl Default for Axes {
    fn default() -> Self {
        Axes {
            forward: Some(KeyAxis {
                pos: Key::W,
                neg: Key::S,
            }),
            strafing: Some(KeyAxis {
                pos: Key::D,
                neg: Key::A,
            }),
            vertical: None,
        }
    }
}

/// Controls for first person camera.
pub struct FirstPerson {
    object: Object,
    position: mint::Point3<f32>,
    yaw: f32,
    pitch: f32,
    move_speed: f32,
    look_speed: f32,
    axes: Axes,
    vertical_move: bool,
    vertical_look: bool,
}

/// Constructs custom [`FirstPerson`](struct.FirstPerson.html) controls.
pub struct Builder {
    object: Object,
    position: mint::Point3<f32>,
    yaw: f32,
    pitch: f32,
    move_speed: f32,
    look_speed: f32,
    axes: Axes,
    vertical_move: bool,
    vertical_look: bool,
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
            look_speed: 0.5,
            axes: Axes::default(),
            vertical_move: true,
            vertical_look: true,
        }
    }

    /// Set the initial yaw angle in radians.
    ///
    /// Default is 0.0.
    pub fn yaw(
        &mut self,
        yaw: f32,
    ) -> &mut Self {
        self.yaw = yaw;
        self
    }

    /// Set the initial pitch angle in radians.
    ///
    /// Defaults to 0.0.
    pub fn pitch(
        &mut self,
        pitch: f32,
    ) -> &mut Self {
        self.pitch = pitch;
        self
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

    /// Setup the movement speed in world units per second.
    ///
    /// Defaults to 1.0 world units per second.
    pub fn move_speed(
        &mut self,
        speed: f32,
    ) -> &mut Self {
        self.move_speed = speed;
        self
    }

    /// Setup mouse sensitivity.
    ///
    /// Defaults to 0.5
    pub fn look_speed(
        &mut self,
        speed: f32,
    ) -> &mut Self {
        self.look_speed = speed;
        self
    }

    /// Setup whether controlled object should move along `y` axis when looking
    /// down or up.
    ///
    /// Defaults to true.
    pub fn vertical_movement(
        &mut self,
        value: bool,
    ) -> &mut Self {
        self.vertical_move = value;
        self
    }

    /// Setup whether controlled object can adjust pitch using mouse.
    ///
    /// Defaults to true.
    pub fn vertical_look(
        &mut self,
        value: bool,
    ) -> &mut Self {
        self.vertical_look = value;
        self
    }

    /// Setup key axis for moving forward/backward.
    ///
    /// Defaults to `W` and `S` keys.
    pub fn axis_forward(
        &mut self,
        axis: Option<KeyAxis>,
    ) -> &mut Self {
        self.axes.forward = axis;
        self
    }

    /// Setup button for "strafing" left/right.
    ///
    /// Defaults to `A` and `D` keys.
    pub fn axis_strafing(
        &mut self,
        axis: Option<KeyAxis>,
    ) -> &mut Self {
        self.axes.strafing = axis;
        self
    }

    /// Setup button for moving up/down.
    ///
    /// Defaults to `None`.
    pub fn axis_vertical(
        &mut self,
        axis: Option<KeyAxis>,
    ) -> &mut Self {
        self.axes.vertical = axis;
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
            axes: self.axes.clone(),
            vertical_move: self.vertical_move,
            vertical_look: self.vertical_look,
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

    /// Sets the yaw angle in radians.
    pub fn set_yaw(
        &mut self,
        yaw: f32,
    ) -> &mut Self {
        self.yaw = yaw;
        self
    }

    /// Sets the pitch angle in radians.
    pub fn set_pitch(
        &mut self,
        pitch: f32,
    ) -> &mut Self {
        self.pitch = pitch;
        self
    }

    /// Sets the object position.
    pub fn set_position<P>(
        &mut self,
        position: P,
    ) -> &mut Self
    where
        P: Into<mint::Point3<f32>>,
    {
        self.position = position.into();
        self
    }

    /// Sets the movement speed in world units per second.
    pub fn set_move_speed(
        &mut self,
        speed: f32,
    ) -> &mut Self {
        self.move_speed = speed;
        self
    }

    /// Sets the mouse sensitivity.
    pub fn set_look_speed(
        &mut self,
        speed: f32,
    ) -> &mut Self {
        self.look_speed = speed;
        self
    }

    /// Specifies whether controlled object should move along `y` axis when looking
    /// down or up.
    pub fn set_vertical_movement(
        &mut self,
        value: bool,
    ) -> &mut Self {
        self.vertical_move = value;
        self
    }

    /// Specifies whether controlled object can adjust pitch using mouse.
    pub fn set_vertical_look(
        &mut self,
        value: bool,
    ) -> &mut Self {
        self.vertical_look = value;
        self
    }

    /// Sets the key axis for moving forward/backward.
    pub fn set_axis_forward(
        &mut self,
        axis: Option<KeyAxis>,
    ) -> &mut Self {
        self.axes.forward = axis;
        self
    }

    /// Sets the button for "strafing" left/right.
    pub fn set_axis_strafing(
        &mut self,
        axis: Option<KeyAxis>,
    ) -> &mut Self {
        self.axes.strafing = axis;
        self
    }

    /// Setup button for moving up/down.
    ///
    /// Defaults to `None`.
    pub fn set_axis_vertical(
        &mut self,
        axis: Option<KeyAxis>,
    ) -> &mut Self {
        self.axes.vertical = axis;
        self
    }

    /// Updates the position, yaw, and pitch of the controlled object according to
    /// the last frame input.
    pub fn update(
        &mut self,
        input: &Input,
    ) {
        let dtime = input.delta_time();
        let dlook = dtime * self.look_speed;

        let mouse = input.mouse_delta_raw();

        self.yaw += dlook * mouse.x;
        if self.vertical_look {
            self.pitch += dlook * mouse.y;
        }

        self.axes
            .vertical
            .map(|a| if let Some(time) = a.timed(input) {
                self.position.y += self.move_speed * time;
            });

        self.axes
            .forward
            .map(|a| if let Some(time) = a.timed(input) {
                self.position.x += self.move_speed * time * self.yaw.sin();
                self.position.z -= self.move_speed * time * self.yaw.cos();
                if self.vertical_move {
                    self.position.y -= self.move_speed * time * self.pitch.sin();
                }
            });
        self.axes
            .strafing
            .map(|a| if let Some(time) = a.timed(input) {
                self.position.x += self.move_speed * time * self.yaw.cos();
                self.position.z += self.move_speed * time * self.yaw.sin();
            });

        let yrot = cgmath::Quaternion::from_angle_y(cgmath::Rad(-self.yaw));
        let xrot = cgmath::Quaternion::from_angle_x(cgmath::Rad(-self.pitch));
        self.object.set_transform(self.position, yrot * xrot, 1.0);
    }
}
