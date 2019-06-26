//! Axes for handling input.

use glutin::VirtualKeyCode as KeyCode;

/// Two buttons responsible for opposite directions along specific axis.
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub struct Key {
    /// Key for "negative" direction
    pub neg: KeyCode,
    /// Key for "positive" direction
    pub pos: KeyCode,
}

/// Raw axis.
///
/// Usually you can get input from mouse using three axes:
/// - `id = 0` for moves along `X` axis.
/// - `id = 1` for moves along `Y` axis.
/// - `id = 2` for mouse wheel moves.
///
/// However, these `id`s depend on hardware and may vary on different machines.
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub struct Raw {
    /// Axis id.
    pub id: u8,
}

/// Axis for left and right arrow keys.
pub const AXIS_LEFT_RIGHT: Key = Key { neg: KeyCode::Left, pos: KeyCode::Right };
/// Axis for up and down arrow keys.
pub const AXIS_DOWN_UP: Key = Key { neg: KeyCode::Down, pos: KeyCode::Up };
