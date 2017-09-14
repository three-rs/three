//! Contains input related primitives.

/// First person camera controls.
pub mod first_person;

/// Mouse orbit camera controls.
pub mod orbit;

pub use self::first_person::FirstPerson;
pub use self::orbit::Orbit;
pub use input::{Button, Input, Key, KeyAxis, Timer, AXIS_DOWN_UP, AXIS_LEFT_RIGHT, KEY_ESCAPE, KEY_SPACE, MOUSE_LEFT, MOUSE_RIGHT};
