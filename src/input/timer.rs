use input::Input;
use input::TimerDuration;
use std::time;

/// Timer can be used to find the time difference between the moment of timer creation and the
/// moment of calling [`get`](struct.Timer.html#method.get).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timer {
    pub(crate) start: time::Instant,
}

impl Timer {
    /// Get period of time since timer creation in seconds.
    pub fn get(
        &self,
        input: &Input,
    ) -> TimerDuration {
        let dt = input.state.time_moment - self.start;
        dt.as_secs() as f32 + 1e-9 * dt.subsec_nanos() as f32
    }
}
