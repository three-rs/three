use glutin::{ElementState, MouseButton, MouseScrollDelta};
pub use glutin::VirtualKeyCode as Key;
use mint;

use std::collections::HashSet;
use std::time;

mod timer;
pub mod axis;

pub use self::axis::{AXIS_DOWN_UP, AXIS_LEFT_RIGHT};

pub use self::timer::Timer;

const PIXELS_PER_LINE: f32 = 38.0;

pub type TimerDuration = f32;

struct State {
    time_moment: time::Instant,
    is_focused: bool,
    keys_pressed: HashSet<Key>,
    mouse_pressed: HashSet<MouseButton>,
    mouse_pos: mint::Point2<f32>,
    mouse_pos_ndc: mint::Point2<f32>,
}

struct Diff {
    time_delta: TimerDuration,
    keys_hit: Vec<Key>,
    mouse_moves: Vec<mint::Vector2<f32>>,
    mouse_moves_ndc: Vec<mint::Vector2<f32>>,
    axes_raw: Vec<(u8, f32)>,
    mouse_hit: Vec<MouseButton>,
    mouse_wheel: Vec<f32>,
}

/// Controls user and system input from keyboard, mouse and system clock.
pub struct Input {
    state: State,
    delta: Diff,
}

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
        let delta = Diff {
            time_delta: 0.0,
            keys_hit: Vec::new(),
            mouse_moves: Vec::new(),
            mouse_moves_ndc: Vec::new(),
            axes_raw: Vec::new(),
            mouse_hit: Vec::new(),
            mouse_wheel: Vec::new(),
        };
        Input { state, delta }
    }

    /// Manually reset current `Input` state.
    ///
    /// Usually there is no need in using this method, because [`Window`](struct.Window.html)
    /// resets `Input` on each [`update`](struct.Window.html#method.update) method by default.
    ///
    /// It will discard all mouse or raw axes movements and also all keyboard hits.
    /// Moreover, delta time will be recalculated.
    pub fn reset(&mut self) {
        let now = time::Instant::now();
        let dt = now - self.state.time_moment;
        self.state.time_moment = now;
        self.delta.time_delta = dt.as_secs() as TimerDuration + 1e-9 * dt.subsec_nanos() as TimerDuration;
        self.delta.keys_hit.clear();
        self.delta.mouse_moves.clear();
        self.delta.mouse_moves_ndc.clear();
        self.delta.axes_raw.clear();
        self.delta.mouse_hit.clear();
        self.delta.mouse_wheel.clear();
    }

    /// Get current delta time (time since previous frame) in seconds.
    pub fn delta_time(&self) -> TimerDuration {
        self.delta.time_delta
    }

    /// Get list of all keys hit since the last frame.
    pub fn keys_hit(&self) -> &[Key] {
        &self.delta.keys_hit
    }

    /// Get current mouse pointer position in pixels from top-left.
    pub fn mouse_pos(&self) -> mint::Point2<f32> {
        self.state.mouse_pos
    }

    /// Get current mouse pointer position in Normalized Display Coordinates.
    /// See [`map_to_ndc`](struct.Renderer.html#method.map_to_ndc).
    pub fn mouse_pos_ndc(&self) -> mint::Point2<f32> {
        self.state.mouse_pos_ndc
    }

    /// Get list of all mouse wheel movements since last frame.
    pub fn mouse_wheel_movements(&self) -> &[f32] {
        &self.delta.mouse_wheel[..]
    }

    /// Get summarized mouse wheel movement (the sum of all movements since last frame).
    pub fn mouse_wheel(&self) -> f32 {
        self.delta.mouse_wheel.iter().sum()
    }

    /// Get list of all mouse movements since last frame in pixels.
    pub fn mouse_movements(&self) -> &[mint::Vector2<f32>] {
        &self.delta.mouse_moves[..]
    }

    /// Get list of all mouse movements since last frame in NDC.
    pub fn mouse_movements_ndc(&self) -> &[mint::Vector2<f32>] {
        &self.delta.mouse_moves_ndc[..]
    }

    /// Get list of all raw inputs since last frame.
    pub fn axes_movements(&self) -> &[(u8, f32)] {
        &self.delta.axes_raw[..]
    }

    fn calculate_delta(moves: &[mint::Vector2<f32>]) -> mint::Vector2<f32> {
        use cgmath::Vector2;
        moves
            .iter()
            .cloned()
            .map(Vector2::from)
            .sum::<Vector2<f32>>()
            .into()
    }

    /// Get summarized mouse movements (the sum of all movements since last frame) in pixels.
    pub fn mouse_delta(&self) -> mint::Vector2<f32> {
        Input::calculate_delta(self.mouse_movements())
    }

    /// Get summarized mouse movements (the sum of all movements since last frame) in NDC.
    pub fn mouse_delta_ndc(&self) -> mint::Vector2<f32> {
        Input::calculate_delta(self.mouse_movements_ndc())
    }

    /// Get summarized raw input along `0` and `1` axes since last frame.
    /// It usually corresponds to mouse movements.
    pub fn mouse_delta_raw(&self) -> mint::Vector2<f32> {
        use cgmath::Vector2;
        self.delta
            .axes_raw
            .iter()
            .filter(|&&(axis, _)| axis == 0 || axis == 1)
            .map(|&(axis, value)| {
                if axis == 0 {
                    (value, 0.0)
                } else {
                    (0.0, value)
                }
            })
            .map(|t| Vector2 { x: t.0, y: t.1 })
            .sum::<Vector2<f32>>()
            .into()
    }

    /// Return whether [`Window`](struct.Window.html) is in focus or not.
    pub fn is_focused(&self) -> bool {
        self.state.is_focused
    }

    pub(crate) fn window_focus(
        &mut self,
        state: bool,
    ) {
        self.state.is_focused = state;
    }

    pub(crate) fn keyboard_input(
        &mut self,
        state: ElementState,
        key: Key,
    ) {
        match state {
            ElementState::Pressed => {
                self.state.keys_pressed.insert(key);
                self.delta.keys_hit.push(key);
            }
            ElementState::Released => {
                self.state.keys_pressed.remove(&key);
            }
        }
    }

    pub(crate) fn mouse_input(
        &mut self,
        state: ElementState,
        button: MouseButton,
    ) {
        match state {
            ElementState::Pressed => {
                self.state.mouse_pressed.insert(button);
                self.delta.mouse_hit.push(button);
            }
            ElementState::Released => {
                self.state.mouse_pressed.remove(&button);
            }
        }
    }

    pub(crate) fn mouse_moved(
        &mut self,
        pos: mint::Point2<f32>,
        pos_ndc: mint::Point2<f32>,
    ) {
        use cgmath::Point2;
        self.delta
            .mouse_moves
            .push((Point2::from(pos) - Point2::from(self.state.mouse_pos)).into());
        self.delta
            .mouse_moves_ndc
            .push((Point2::from(pos_ndc) - Point2::from(self.state.mouse_pos_ndc)).into());
        self.state.mouse_pos = pos;
        self.state.mouse_pos_ndc = pos_ndc;
    }

    pub(crate) fn axis_moved_raw(
        &mut self,
        axis: u8,
        value: f32,
    ) {
        self.delta.axes_raw.push((axis, value));
    }

    pub(crate) fn mouse_wheel_input(
        &mut self,
        delta: MouseScrollDelta,
    ) {
        self.delta.mouse_wheel.push(match delta {
            MouseScrollDelta::LineDelta(_, y) => y * PIXELS_PER_LINE,
            MouseScrollDelta::PixelDelta(_, y) => y,
        });
    }

    /// Returns `true` there is any input info from [`Button`](struct.Button.html),
    /// [`axis::Key`](struct.Key.html) or [`axis::Raw`](struct.Raw.html). Otherwise returns `false`.
    pub fn hit<H: Hit>(
        &self,
        hit: H,
    ) -> bool {
        hit.hit(self)
    }

    /// Returns the change ('delta') in input state since the last call to
    /// [`Window::update`].
    ///
    /// This value depends on the type of input device is given.
    ///
    /// [`axis::Key`]
    ///
    ///  * `None` when no updates to the axis are received.
    ///  * `Some(1)` when only positive input to the axis is received.
    ///  * `Some(-1)` when only negative input to the axis is received.
    ///  * `Some(0)` when both positive and negative input to the axis is received.
    ///
    /// [`axis::Raw`]
    ///
    ///  * `None` when no updates to the axis are received and
    ///  * `Some(x)` where `x` is the sum of positive and negative inputs otherwise.
    ///
    /// [`Window::update`]: window/struct.Window.html#method.update
    /// [`axis::Key`]: input/axis/struct.Key.html
    /// [`axis::Raw`]: input/axis/struct.Raw.html
    pub fn delta<D: Delta>(
        &self,
        delta: D,
    ) -> <D as Delta>::Output {
        delta.delta(self)
    }

    /// Shortcut for [`delta`] `*` [`delta_time`].
    ///
    /// [`delta`]: struct.Input.html#method.delta
    /// [`delta_time`]: struct.Input.html#method.delta_time
    pub fn timed<D: Delta>(
        &self,
        delta: D,
    ) -> Option<TimerDuration> {
        delta.timed(self)
    }

    /// Returns the amount of:
    ///
    /// - Hits for [`Button`](enum.Button.html) as `u8`.
    ///
    /// - Hits for [`axis::Key`](struct.Key.html) as `(u8, u8)` where first number is for `positive`
    /// direction and the second one is for `negative`.
    pub fn hit_count<C: HitCount>(
        &self,
        hit_count: C,
    ) -> <C as HitCount>::Output {
        hit_count.hit_count(self)
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

/// Trait for [`Buttons`](enum.Button.html).
pub trait Hit {
    /// See [`Input::hit`](struct.Input.html#method.hit).
    fn hit(
        &self,
        input: &Input,
    ) -> bool;
}

impl Hit for Button {
    fn hit(
        &self,
        input: &Input,
    ) -> bool {
        match *self {
            Button::Key(button) => button.hit(input),
            Button::Mouse(button) => button.hit(input),
        }
    }
}

impl Hit for Key {
    fn hit(
        &self,
        input: &Input,
    ) -> bool {
        input.state.keys_pressed.contains(self)
    }
}

impl Hit for MouseButton {
    fn hit(
        &self,
        input: &Input,
    ) -> bool {
        input.state.mouse_pressed.contains(self)
    }
}

impl Hit for axis::Key {
    fn hit(
        &self,
        input: &Input,
    ) -> bool {
        let pos_hit = input.state.keys_pressed.contains(&self.pos);
        let neg_hit = input.state.keys_pressed.contains(&self.neg);
        pos_hit || neg_hit
    }
}

impl Hit for axis::Raw {
    fn hit(
        &self,
        input: &Input,
    ) -> bool {
        input
            .delta
            .axes_raw
            .iter()
            .filter(|&&(id, _)| id == self.id)
            .count() > 0
    }
}

/// Trait for [`Buttons`](enum.Button.html) and [`axis::Key`](struct.Key.html).
pub trait HitCount {
    /// Output type.
    type Output;
    /// See [`Input::hit_count`](struct.Input.html#method.hit_count).
    fn hit_count(
        &self,
        input: &Input,
    ) -> Self::Output;
}

impl HitCount for Button {
    type Output = u8;
    fn hit_count(
        &self,
        input: &Input,
    ) -> Self::Output {
        use std::u8::MAX;
        match *self {
            Button::Key(button) => input
                .delta
                .keys_hit
                .iter()
                .filter(|&&key| key == button)
                .take(MAX as usize)
                .count() as Self::Output,
            Button::Mouse(button) => input
                .delta
                .mouse_hit
                .iter()
                .filter(|&&key| key == button)
                .take(MAX as usize)
                .count() as Self::Output,
        }
    }
}

impl HitCount for axis::Key {
    type Output = (u8, u8);

    fn hit_count(
        &self,
        input: &Input,
    ) -> Self::Output {
        use std::u8::MAX;
        let pos = input
            .delta
            .keys_hit
            .iter()
            .filter(|&&k| k == self.pos)
            .take(MAX as usize)
            .count() as u8;
        let neg = input
            .delta
            .keys_hit
            .iter()
            .filter(|&&k| k == self.neg)
            .take(MAX as usize)
            .count() as u8;
        (pos, neg)
    }
}

/// Trait for [`axis::Key`](struct.Key.html) and [`axis::Raw`](struct.Raw.html).
pub trait Delta {
    /// Output type.
    type Output;

    /// See [`Input::delta`](struct.Input.html#method.delta).
    fn delta(
        &self,
        input: &Input,
    ) -> Self::Output;
    /// See [`Input::timed`](struct.Input.html#method.timed).
    fn timed(
        &self,
        input: &Input,
    ) -> Option<TimerDuration>;
}

impl Delta for axis::Key {
    type Output = Option<i8>;

    fn delta(
        &self,
        input: &Input,
    ) -> Self::Output {
        match (input.delta.keys_hit.contains(&self.pos), input.delta.keys_hit.contains(&self.neg)) {
            (true, true) => Some(0),
            (true, false) => Some(1),
            (false, true) => Some(-1),
            (false, false) => None,
        }
    }

    fn timed(
        &self,
        input: &Input,
    ) -> Option<TimerDuration> {
        self.delta(input)
            .map(|delta| delta as TimerDuration * input.delta_time())
    }
}

impl Delta for axis::Raw {
    type Output = Option<f32>;

    fn delta(
        &self,
        input: &Input,
    ) -> Self::Output {
        let moves = input
            .delta
            .axes_raw
            .iter()
            .filter(|&&(id, _)| id == self.id)
            .map(|&(_, value)| value)
            .collect::<Vec<_>>();
        if moves.len() == 0 {
            None
        } else {
            Some(moves.iter().sum::<f32>())
        }
    }

    fn timed(
        &self,
        input: &Input,
    ) -> Option<TimerDuration> {
        self.delta(input)
            .map(|v| v as TimerDuration * input.delta_time())
    }
}

/// `Escape` keyboard button.
pub const KEY_ESCAPE: Button = Button::Key(Key::Escape);
/// `Space` keyboard button.
pub const KEY_SPACE: Button = Button::Key(Key::Space);
/// Left mouse button.
pub const MOUSE_LEFT: Button = Button::Mouse(MouseButton::Left);
/// Right mouse button.
pub const MOUSE_RIGHT: Button = Button::Mouse(MouseButton::Right);
