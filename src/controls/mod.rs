//! Contains input related primitives.

mod orbit;

pub use self::orbit::{Orbit, OrbitBuilder};
pub use input::{Button, KeyAxis, Timer, Input,
                KEY_ESCAPE, KEY_SPACE, MOUSE_LEFT, MOUSE_RIGHT,
                AXIS_LEFT_RIGHT, AXIS_DOWN_UP};
