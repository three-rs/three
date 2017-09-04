//! Contains input related primitives.

/// First person camera controls.
pub mod first_person;

/// Mouse orbit camera controls.
pub mod orbit;

pub use self::first_person::FirstPerson;
pub use self::orbit::Orbit;
pub use input::{Button, Key, KeyAxis, Timer, Input,
                KEY_ESCAPE, KEY_SPACE, MOUSE_LEFT, MOUSE_RIGHT,
                AXIS_LEFT_RIGHT, AXIS_DOWN_UP};
