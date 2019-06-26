use input::TimerDuration;
use std::time;

/// Timer can be used to find the time difference between the moment of timer creation and the
/// moment of calling [`elapsed`](struct.Timer.html#method.get).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timer {
    pub(crate) start: time::Instant,
}

impl Timer {
    /// Create new timer based on current system time.
    pub fn new() -> Self {
        Self { start: time::Instant::now() }
    }

    /// Reset time of creation to current time.
    pub fn reset(&mut self) {
        self.start = time::Instant::now();
    }

    /// Get period of time since timer creation in seconds.
    pub fn elapsed(&self) -> TimerDuration {
        let dt = self.start.elapsed();
        dt.as_secs() as f32 + 1e-9 * dt.subsec_nanos() as f32
    }
}
