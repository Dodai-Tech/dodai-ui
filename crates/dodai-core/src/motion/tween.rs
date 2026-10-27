//! A scalar that eases towards a target.

use crate::motion::{lerp, Motion};

/// Animated scalar that eases from its current value to a new target.
///
/// Time is passed in (seconds on any monotonic clock the host has: `Instant`, a frame
/// counter, `performance.now()`), so it works on every platform including wasm.
/// Retarget with [`Tween::to`], read with [`Tween::value`].
#[derive(Clone, Copy, Debug)]
pub struct Tween {
    from: f32,
    to: f32,
    start: f32,
    motion: Motion,
}

impl Tween {
    pub fn new(v: f32, motion: Motion) -> Self {
        Self {
            from: v,
            to: v,
            start: 0.0,
            motion,
        }
    }
    /// Linear progress `0..=1` at `now`.
    fn progress(&self, now: f32) -> f32 {
        ((now - self.start) / self.motion.duration.as_secs_f32().max(1e-3)).clamp(0.0, 1.0)
    }
    /// Eased value at `now`.
    pub fn value(&self, now: f32) -> f32 {
        lerp(self.from, self.to, self.motion.eased(self.progress(now)))
    }
    pub fn is_done(&self, now: f32) -> bool {
        self.progress(now) >= 1.0
    }
    /// The value it is heading for.
    pub fn target(&self) -> f32 {
        self.to
    }
    /// Move towards `to` from wherever the value is at `now` (no-op if unchanged).
    pub fn to(&mut self, to: f32, motion: Motion, now: f32) {
        if (to - self.to).abs() > f32::EPSILON {
            self.from = self.value(now);
            self.to = to;
            self.start = now;
            self.motion = motion;
        }
    }
}
