//! A scalar that eases towards a target.

use crate::{lerp, Motion};

/// Animated scalar that eases from its current value to a new target.
/// Retarget with [`Tween::to`] on every frame; read with [`Tween::value`].
#[derive(Clone, Copy, Debug)]
pub struct Tween {
    from: f32,
    to: f32,
    start: std::time::Instant,
    motion: Motion,
}

impl Tween {
    pub fn new(v: f32, motion: Motion) -> Self {
        Self {
            from: v,
            to: v,
            start: std::time::Instant::now(),
            motion,
        }
    }
    /// Current eased value.
    pub fn value(&self) -> f32 {
        let t = self.start.elapsed().as_secs_f32() / self.motion.duration.as_secs_f32().max(1e-3);
        lerp(self.from, self.to, self.motion.eased(t))
    }
    pub fn is_done(&self) -> bool {
        self.start.elapsed() >= self.motion.duration
    }
    /// Move towards `to` from wherever the value is now (no-op if unchanged).
    pub fn to(&mut self, to: f32, motion: Motion) {
        if (to - self.to).abs() > f32::EPSILON {
            self.from = self.value();
            self.to = to;
            self.start = std::time::Instant::now();
            self.motion = motion;
        }
    }
}
