use glutin::{MouseButton, MouseScrollDelta, ElementState};
pub use glutin::VirtualKeyCode as Key;
use mint;

use std::collections::HashSet;
use std::time;

const PIXELS_PER_LINE: f32 = 38.0;

pub type TimerDuration = f32;

// TODO: Remove
#[allow(dead_code)]
struct State {
    time_moment: time::Instant,
    is_focused: bool,
    keys_pressed: HashSet<Key>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_pos: mint::Point2<f32>,
    mouse_pos_ndc: mint::Point2<f32>,
}

struct Delta {
    time_delta: TimerDuration,
    keys_hit: Vec<Key>,
    mouse_moves: Vec<mint::Vector2<f32>>,
    mouse_moves_ndc: Vec<mint::Vector2<f32>>,
    mouse_hit: Vec<MouseButton>,
    mouse_wheel: Vec<f32>,
}

/// Controls user and system input from keyboard, mouse and system clock.
pub struct Input(State, Delta);

impl Input {
    pub(crate) fn new() -> Self {
        let state = State {
            time_moment: time::Instant::now(),
            is_focused: true,
            keys_pressed: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_pos: [0.0; 2].into(),
            mouse_pos_ndc: [0.0; 2].into(),
        };
        let diff = Delta {
            time_delta: 0.0,
            keys_hit: Vec::new(),
            mouse_moves: Vec::new(),
            mouse_moves_ndc: Vec::new(),
            mouse_hit: Vec::new(),
            mouse_wheel: Vec::new(),
        };
        Input(state, diff)
    }

    pub(crate) fn reset(&mut self) {
        let now = time::Instant::now();
        let dt = now - self.0.time_moment;
        self.0.time_moment = now;
        self.1.time_delta = dt.as_secs() as f32 + 1e-9 * dt.subsec_nanos() as f32;
        self.1.keys_hit.clear();
        self.1.mouse_moves.clear();
        self.1.mouse_moves_ndc.clear();
        self.1.mouse_hit.clear();
        self.1.mouse_wheel.clear();
    }

    /// Create new timer.
    pub fn time(&self) -> Timer {
        Timer {
            start: self.0.time_moment
        }
    }

    /// Get current delta time (time since previous frame) in seconds.
    pub fn delta_time(&self) -> f32 {
        self.1.time_delta
    }

    /// Get current mouse pointer position in pixels from top-left
    pub fn mouse_pos(&self) -> mint::Point2<f32> {
        self.0.mouse_pos
    }

    /// Get current mouse pointer position in Normalized Display Coordinates.
    /// See [`map_to_ndc`](struct.Renderer.html#method.map_to_ndc).
    pub fn mouse_pos_ndc(&self) -> mint::Point2<f32> {
        self.0.mouse_pos_ndc
    }

    /// Get list of all mouse wheel movements since last frame.
    pub fn mouse_wheel_movements(&self) -> &[f32] {
        &self.1.mouse_wheel[..]
    }

    /// Get summarized mouse wheel movement (the sum of all movements since last frame).
    pub fn mouse_wheel(&self) -> f32 {
        self.1.mouse_wheel.iter().sum()
    }

    /// Get list of all mouse movements since last frame in pixels.
    pub fn mouse_movements(&self) -> &[mint::Vector2<f32>] {
        &self.1.mouse_moves[..]
    }

    /// Get list of all mouse movements since last frame in NDC.
    pub fn mouse_movements_ndc(&self) -> &[mint::Vector2<f32>] {
        &self.1.mouse_moves_ndc[..]
    }

    /// Get summarized mouse movements (the sum of all movements since last frame) in pixels.
    pub fn mouse_delta(&self) -> mint::Vector2<f32> {
        use cgmath::Vector2;
        self.1.mouse_moves.iter()
            .cloned()
            .map(Vector2::from)
            .sum::<Vector2<f32>>()
            .into()
    }

    /// Get summarized mouse movements (the sum of all movements since last frame) in NDC.
    pub fn mouse_delta_ndc(&self) -> mint::Vector2<f32> {
        use cgmath::Vector2;
        self.1.mouse_moves_ndc.iter()
            .cloned()
            .map(Vector2::from)
            .sum::<Vector2<f32>>()
            .into()
    }

    pub(crate) fn keyboard_input(&mut self, state: ElementState, key: Key) {
        match state {
            ElementState::Pressed => {
                self.0.keys_pressed.insert(key);
                self.1.keys_hit.push(key);
            }
            ElementState::Released => {
                self.0.keys_pressed.remove(&key);
            }
        }
    }

    pub(crate) fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        match state {
            ElementState::Pressed => {
                self.0.mouse_pressed.insert(button);
                self.1.mouse_hit.push(button);
            }
            ElementState::Released => {
                self.0.mouse_pressed.remove(&button);
            }
        }
    }

    pub(crate) fn mouse_moved(&mut self, pos: mint::Point2<f32>, pos_ndc: mint::Point2<f32>) {
        use cgmath::Point2;
        self.1.mouse_moves.push((Point2::from(pos) - Point2::from(self.0.mouse_pos)).into());
        self.1.mouse_moves_ndc.push((Point2::from(pos_ndc) - Point2::from(self.0.mouse_pos_ndc)).into());
        self.0.mouse_pos = pos;
        self.0.mouse_pos_ndc = pos_ndc;
    }

    pub(crate) fn mouse_wheel_input(&mut self, delta: MouseScrollDelta) {
        self.1.mouse_wheel.push(match delta {
            MouseScrollDelta::LineDelta(_, y) => y * PIXELS_PER_LINE,
            MouseScrollDelta::PixelDelta(_, y) => y,
        });
    }
}

/// Timer can be used to find the time difference between the moment of timer creation and the
/// moment of calling [`get`](struct.Timer.html#method.get).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timer {
    start: time::Instant,
}

impl Timer {
    /// Get period of time since timer creation in seconds.
    pub fn get(&self, input: &Input) -> TimerDuration {
        let dt = input.0.time_moment - self.start;
        dt.as_secs() as f32 + 1e-9 * dt.subsec_nanos() as f32
    }
}

/// Keyboard or mouse button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Button {
    /// Keyboard button.
    Key(Key),
    /// Mouse button.
    Mouse(MouseButton),
}

/// `Escape` keyboard button.
pub const KEY_ESCAPE: Button = Button::Key(Key::Escape);
/// `Space` keyboard button.
pub const KEY_SPACE: Button = Button::Key(Key::Space);
/// Left mouse button.
pub const MOUSE_LEFT: Button = Button::Mouse(MouseButton::Left);
/// Right mouse button.
pub const MOUSE_RIGHT: Button = Button::Mouse(MouseButton::Right);

impl Button {
    /// Get the amount of hits (moves down) for this button.
    /// You typically need to pass `window.input` as `input` parameter.
    pub fn hit_count(&self, input: &Input) -> u8 {
        use std::u8::MAX;
        match *self {
            Button::Key(button) => input.1.keys_hit.iter().filter(|&&key| key == button).take(MAX as usize).count() as u8,
            Button::Mouse(button) => input.1.mouse_hit.iter().filter(|&&key| key == button).take(MAX as usize).count() as u8,
        }
    }

    /// Whether this button is pressed or not at the moment.
    pub fn is_hit(&self, input: &Input) -> bool {
        match *self {
            Button::Key(button) => input.0.keys_pressed.contains(&button),
            Button::Mouse(button) => input.0.mouse_pressed.contains(&button),
        }
    }
}

/// Two buttons responsible for opposite directions along specific axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyAxis {
    /// Key for "negative" direction
    pub neg: Key,
    /// Key for "positive" direction
    pub pos: Key,
}

/// Axis for left and right arrow keys.
pub const AXIS_LEFT_RIGHT: KeyAxis = KeyAxis{ neg: Key::Left, pos: Key::Right };
/// Axis for up and down arrow keys.
pub const AXIS_DOWN_UP: KeyAxis = KeyAxis{ neg: Key::Down, pos: Key::Up };

impl KeyAxis {
    /// Returns `Some(positive value)` if "positive" button was pressed more times since last frame.
    /// Returns `Some(negative value)` if "negative" button was pressed more times.
    /// Otherwise returns `None`.
    pub fn delta_hits(&self, input: &Input) -> Option<i8> {
        let (mut pos, mut neg) = (0, 0);
        for &key in input.1.keys_hit.iter() {
            if key == self.neg { neg += 1 }
            if key == self.pos { pos += 1 }
        }
        if pos + neg != 0 {
            Some(pos - neg)
        } else {
            None
        }
    }

    /// Returns `Some(value)` where `value` is positive if "positive" button is pressed now and
    /// negative otherwise. `value` itself represents the amount of time button was pressed.
    /// If both buttons weren't pressed, return `None`.
    pub fn timed(&self, input: &Input) -> Option<TimerDuration> {
        let is_pos = input.0.keys_pressed.contains(&self.pos);
        let is_neg = input.0.keys_pressed.contains(&self.neg);
        if is_pos && is_neg {
            Some(0.0)
        } else if is_pos {
            Some(input.1.time_delta)
        } else if is_neg {
            Some(-input.1.time_delta)
        } else {
            None
        }
    }
}
