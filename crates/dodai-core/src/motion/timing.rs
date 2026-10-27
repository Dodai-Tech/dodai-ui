//! [`Motion`] (duration + easing + entry) and the entry effects.

use crate::motion::Easing;
use std::time::Duration;

/// Duration + easing, with a speed multiplier.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    pub duration: Duration,
    pub easing: Easing,
    /// Wait before the entry / value-in animation starts (stagger).
    pub delay: Duration,
    /// How components enter (see [`EntryKind`]).
    pub entry: EntrySet,
    /// How lively the design system is: drives springs and gesture sizes in mixins.
    pub personality: crate::motion::Personality,
}

/// How a component animates in when it first appears.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EntryKind {
    /// No entry animation (values also jump straight to their target).
    None,
    #[default]
    Fade,
    /// Fade while rising 14 px.
    Rise,
    /// Fade while dropping in 14 px.
    Drop,
    SlideLeft,
    SlideRight,
}

/// A composition of entry effects (opacity multiplies, offsets add), each
/// with an optional own duration in ms (`0` = the motion's default).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct EntrySet {
    mask: u8,
    ms: [u16; 5],
    grow: u16,
    stagger: u16,
}

impl EntrySet {
    pub const NONE: EntrySet = EntrySet {
        mask: 0,
        ms: [0; 5],
        grow: 0,
        stagger: 0,
    };
    pub fn of(k: EntryKind) -> Self {
        Self::NONE.with(k)
    }
    pub fn with(mut self, k: EntryKind) -> Self {
        self.mask |= k.bit();
        self
    }
    pub fn without(mut self, k: EntryKind) -> Self {
        self.mask &= !k.bit();
        self
    }
    pub fn toggled(mut self, k: EntryKind) -> Self {
        self.mask ^= k.bit();
        self
    }
    pub fn contains(self, k: EntryKind) -> bool {
        self.mask & k.bit() != 0
    }
    pub fn is_empty(self) -> bool {
        self.mask == 0
    }
    /// Duration of value-in (fill/sweep/grow) animations in ms (`0` = default).
    pub fn grow_ms(self) -> u16 {
        self.grow
    }
    pub fn with_grow_ms(mut self, ms: u16) -> Self {
        self.grow = ms;
        self
    }
    /// Delay between consecutive children of a scoped panel in ms.
    pub fn stagger_ms(self) -> u16 {
        self.stagger
    }
    pub fn with_stagger_ms(mut self, ms: u16) -> Self {
        self.stagger = ms;
        self
    }
    /// Own duration for `k` in ms (`0` = default).
    pub fn ms_of(self, k: EntryKind) -> u16 {
        k.slot().map_or(0, |i| self.ms[i])
    }
    /// Set `k`'s own duration (ms; `0` = default) and enable it.
    pub fn with_ms(mut self, k: EntryKind, ms: u16) -> Self {
        if let Some(i) = k.slot() {
            self.ms[i] = ms;
        }
        self.with(k)
    }
    fn dur_ms(self, k: EntryKind, default: Duration) -> f32 {
        match self.ms_of(k) {
            0 => default.as_secs_f32() * 1000.0,
            n => n as f32,
        }
    }
    /// Total animation length: the longest active effect.
    pub fn duration(self, default: Duration) -> Duration {
        let ms = EntryKind::ALL
            .iter()
            .filter(|k| self.contains(**k))
            .map(|k| self.dur_ms(*k, default))
            .fold(1.0, f32::max);
        Duration::from_secs_f32(ms / 1000.0)
    }
    /// Combined `(opacity, dx, dy)` at overall progress `p` (each effect
    /// finishes at its own duration). Pure.
    pub fn frame(self, p: f32, default: Duration) -> (f32, f32, f32) {
        let total = self.duration(default).as_secs_f32() * 1000.0;
        EntryKind::ALL.iter().filter(|k| self.contains(**k)).fold(
            (1.0, 0.0, 0.0),
            |(o, x, y), k| {
                let pk = (p * total / self.dur_ms(*k, default)).clamp(0.0, 1.0);
                let (ko, kx, ky) = k.frame(pk);
                (o * ko, x + kx, y + ky)
            },
        )
    }
}

impl From<EntryKind> for EntrySet {
    fn from(k: EntryKind) -> Self {
        EntrySet::of(k)
    }
}

impl EntryKind {
    fn slot(self) -> Option<usize> {
        (self.bit() != 0).then(|| self.bit().trailing_zeros() as usize)
    }
    fn bit(self) -> u8 {
        match self {
            EntryKind::None => 0,
            EntryKind::Fade => 1,
            EntryKind::Rise => 2,
            EntryKind::Drop => 4,
            EntryKind::SlideLeft => 8,
            EntryKind::SlideRight => 16,
        }
    }
    pub const ALL: [EntryKind; 6] = [
        EntryKind::None,
        EntryKind::Fade,
        EntryKind::Rise,
        EntryKind::Drop,
        EntryKind::SlideLeft,
        EntryKind::SlideRight,
    ];
    pub fn label(self) -> &'static str {
        match self {
            EntryKind::None => "None",
            EntryKind::Fade => "Fade",
            EntryKind::Rise => "Rise",
            EntryKind::Drop => "Drop",
            EntryKind::SlideLeft => "Slide from left",
            EntryKind::SlideRight => "Slide from right",
        }
    }
    /// `(opacity, dx, dy)` at eased progress `p` (px offsets). Pure.
    pub fn frame(self, p: f32) -> (f32, f32, f32) {
        let q = 1.0 - p;
        match self {
            EntryKind::None => (1.0, 0.0, 0.0),
            EntryKind::Fade => (p, 0.0, 0.0),
            EntryKind::Rise => (p, 0.0, 14.0 * q),
            EntryKind::Drop => (p, 0.0, -14.0 * q),
            EntryKind::SlideLeft => (p, -28.0 * q, 0.0),
            EntryKind::SlideRight => (p, 28.0 * q, 0.0),
        }
    }
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(220),
            easing: Easing::EaseOut,
            delay: Duration::ZERO,
            entry: EntrySet::of(EntryKind::Fade),
            personality: crate::motion::Personality::Smooth,
        }
    }
}

impl Motion {
    pub fn new(duration: Duration, easing: Easing) -> Self {
        Self {
            duration,
            easing,
            delay: Duration::ZERO,
            entry: EntrySet::of(EntryKind::Fade),
            personality: crate::motion::Personality::Smooth,
        }
    }
    pub fn duration(mut self, d: Duration) -> Self {
        self.duration = d;
        self
    }
    pub fn millis(self, ms: u64) -> Self {
        self.duration(Duration::from_millis(ms))
    }
    pub fn personality(mut self, p: crate::motion::Personality) -> Self {
        self.personality = p;
        self
    }
    pub fn entry(mut self, k: impl Into<EntrySet>) -> Self {
        self.entry = k.into();
        self
    }
    /// Start the entry / value-in animation after `d`.
    pub fn delay(mut self, d: Duration) -> Self {
        self.delay = d;
        self
    }
    pub fn easing(mut self, e: Easing) -> Self {
        self.easing = e;
        self
    }
    /// Scale speed: `2.0` halves the duration, `0.5` doubles it.
    pub fn speed(mut self, factor: f32) -> Self {
        self.duration = self.duration.div_f32(factor.max(0.01));
        self
    }
    /// Linear progress `0..=1` after `elapsed` seconds (past `delay`).
    pub fn progress(&self, elapsed: f32) -> f32 {
        let d = self.duration.as_secs_f32().max(1e-3);
        ((elapsed - self.delay.as_secs_f32()) / d).clamp(0.0, 1.0)
    }
    /// The whole entry for a host that only has a clock: `(opacity, dx, dy)` after
    /// `elapsed` seconds, with this motion's easing, delay and entry effects applied.
    pub fn entry_frame(&self, elapsed: f32) -> (f32, f32, f32) {
        let total = self.entry.duration(self.duration).as_secs_f32().max(1e-3);
        let t = ((elapsed - self.delay.as_secs_f32()) / total).clamp(0.0, 1.0);
        self.entry.frame(self.eased(t), self.duration)
    }
    /// Eased progress for linear `t`.
    pub fn eased(&self, t: f32) -> f32 {
        self.easing.apply(t)
    }
}
