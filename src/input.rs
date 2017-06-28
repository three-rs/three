#![allow(missing_docs)] //TODO

use glutin::{MouseButton, ElementState, VirtualKeyCode as Key};
use mint;

use std::collections::HashSet;
use std::time;


pub type TimerDuration = f32;

#[allow(dead_code)]
struct InputState {
    time_moment: time::Instant,
    is_focused: bool,
    keys_pressed: HashSet<Key>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_pos: mint::Point2<f32>,
}

#[allow(dead_code)]
struct InputDiff {
    time_delta: TimerDuration,
    keys_hit: Vec<Key>,
    mouse_moves: Vec<mint::Vector2<f32>>,
    mouse_hit: Vec<MouseButton>,
}

pub struct Input(InputState, InputDiff);

impl Input {
    pub fn new() -> Self {
        let state = InputState {
            time_moment: time::Instant::now(),
            is_focused: true,
            keys_pressed: HashSet::new(),
            mouse_pressed: HashSet::new(),
            mouse_pos: [0.0; 2].into(),
        };
        let diff = InputDiff {
            time_delta: 0.0,
            keys_hit: Vec::new(),
            mouse_moves: Vec::new(),
            mouse_hit: Vec::new(),
        };
        Input(state, diff)
    }

    pub fn reset(&mut self) {
        let now = time::Instant::now();
        let dt = now - self.0.time_moment;
        self.0.time_moment = now;
        self.1.time_delta = dt.as_secs() as f32 + 1e-9 * dt.subsec_nanos() as f32;
        self.1.keys_hit.clear();
        self.1.mouse_moves.clear();
        self.1.mouse_hit.clear();
    }

    pub fn time(&self) -> Timer {
        Timer {
            start: self.0.time_moment
        }
    }

    pub fn get_time_delta(&self) -> f32 {
        self.1.time_delta
    }

    pub fn get_mouse_pos(&self) -> mint::Point2<f32> {
        self.0.mouse_pos
    }

    pub fn keyboard_input(&mut self, state: ElementState, key: Key) {
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

    pub fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
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

    pub fn mouse_moved(&mut self, pos: mint::Point2<f32>) {
        self.1.mouse_moves.push(mint::Vector2 {
            x: pos.x - self.0.mouse_pos.x,
            y: pos.y - self.0.mouse_pos.y,
        });
        self.0.mouse_pos = pos;
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timer {
    start: time::Instant,
}

impl Timer {
    pub fn get(&self, input: &Input) -> TimerDuration {
        let dt = input.0.time_moment - self.start;
        dt.as_secs() as f32 + 1e-9 * dt.subsec_nanos() as f32
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Button {
    Key(Key),
    Mouse(MouseButton),
}

pub const KEY_ESCAPE: Button = Button::Key(Key::Escape);
pub const KEY_SPACE: Button = Button::Key(Key::Space);
pub const MOUSE_LEFT: Button = Button::Mouse(MouseButton::Left);
pub const MOUSE_RIGHT: Button = Button::Mouse(MouseButton::Right);

impl Button {
    pub fn count_hits(&self, input: &Input) -> u8 {
        use std::u8::MAX;
        match *self {
            Button::Key(button) => input.1.keys_hit.iter().filter(|&&key| key == button).take(MAX as usize).count() as u8,
            Button::Mouse(button) => input.1.mouse_hit.iter().filter(|&&key| key == button).take(MAX as usize).count() as u8,
        }
    }

    pub fn is_hit(&self, input: &Input) -> bool {
        match *self {
            Button::Key(button) => input.0.keys_pressed.contains(&button),
            Button::Mouse(button) => input.0.mouse_pressed.contains(&button),
        }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyAxis {
    pub neg: Key,
    pub pos: Key,
}

pub const AXIS_LEFT_RIGHT: KeyAxis = KeyAxis{ neg: Key::Left, pos: Key::Right };
pub const AXIS_DOWN_UP: KeyAxis = KeyAxis{ neg: Key::Down, pos: Key::Up };

impl KeyAxis {
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
