//! High-level input handling.
//!
//! ## Controllers
//!
//! Controllers are used to orient objects in the scene using input devices.
//! Any [`Object`] can be the target of a controller, including cameras.
//!
//! ### Orbital
//!
//!  * Uses mouse movement to rotate the object around its target.
//!  * Uses the mouse scroll wheel to move the object closer to or further
//!    from its target.
//!
//! ### First-person
//!
//!  * Uses the W and S keys to move forward or backward.
//!  * Uses the A and D keys to strafe left or right.
//!  * Uses mouse movement to rotate the object when the right mouse button
//!    is held down.
//!
//! [`Object`]: ../object/trait.Object.html

/// First person controls.
pub mod first_person;

/// Mouse orbit controls.
pub mod orbit;

#[doc(inline)]
pub use self::first_person::FirstPerson;

#[doc(inline)]
pub use self::orbit::Orbit;

pub use input::{axis, Button, Delta, Hit, HitCount, Input, Key, MouseButton, Timer, AXIS_DOWN_UP, AXIS_LEFT_RIGHT, KEY_ESCAPE, KEY_SPACE, MOUSE_LEFT, MOUSE_RIGHT};
