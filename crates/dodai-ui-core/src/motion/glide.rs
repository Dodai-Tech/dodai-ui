//! Values that follow a moving target the way a [`Motion`] says: [`Glide`] for one number,
//! [`Stretch`] for a span whose edges lead and trail, and [`Glitch`] for a digital cut-in.
//! Components keep one of these per animated property and read it every frame.

use crate::motion::{Clip, Motion, SpringState, Tween};

/// How a selection moves from one item to the next (tabs, segmented controls).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SelectionMotion {
    /// The indicator glides across as one piece.
    #[default]
    Slide,
    /// The indicator's leading edge runs ahead and the trailing edge catches up
    /// (see [`Stretch`]).
    Stretch,
    /// The indicator cuts in at the new item with jittering slices and colour-split ghosts
    /// (see [`Glitch`]); the old one flickers out.
    Glitch,
    /// No travelling indicator: the chosen item rises off the page and the old one drops back.
    Lift,
}

impl SelectionMotion {
    pub const ALL: [SelectionMotion; 4] = [
        SelectionMotion::Slide,
        SelectionMotion::Stretch,
        SelectionMotion::Glitch,
        SelectionMotion::Lift,
    ];
    pub fn label(self) -> &'static str {
        match self {
            SelectionMotion::Slide => "Slide",
            SelectionMotion::Stretch => "Stretch",
            SelectionMotion::Glitch => "Glitch",
            SelectionMotion::Lift => "Lift",
        }
    }
}

/// How a value travels to a new target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Travel {
    /// Over the motion's duration on its easing curve.
    #[default]
    Eased,
    /// On the motion's spring (its personality, at its pace). Keeps its velocity when the
    /// target moves mid-flight, so quick changes never jerk.
    Spring,
}

#[derive(Clone, Copy, Debug)]
enum Drive {
    Eased(Tween),
    Spring(SpringState),
}

/// One number following a target as a [`Motion`] says (eased or on a spring).
///
/// ```
/// use dodai_ui_core::motion::{Glide, Motion, Travel};
/// let m = Motion::default().travel(Travel::Spring);
/// let mut g = Glide::new(0.0, &m);
/// g.to(100.0, &m, 0.0);
/// assert!(g.value(0.05) > 0.0 && !g.settled(0.05));
/// assert!(g.settled(5.0) && (g.value(5.0) - 100.0).abs() < 0.5);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Glide {
    drive: Drive,
}

impl Glide {
    /// At rest on `value`.
    pub fn new(value: f32, m: &Motion) -> Self {
        let drive = match m.travel {
            Travel::Eased => Drive::Eased(Tween::new(value, *m)),
            Travel::Spring => Drive::Spring(SpringState::new(m.spring(), value)),
        };
        Self { drive }
    }
    /// Head for `target` from wherever the value is at `now`, on `m` from the current position
    /// and velocity. The same target again keeps the move already going, whatever `m` says (so
    /// switching animation off mid-move does not snap it).
    pub fn to(&mut self, target: f32, m: &Motion, now: f32) {
        if (target - self.target()).abs() <= f32::EPSILON {
            return;
        }
        let (x, v) = (self.value(now), self.velocity(now));
        match (&mut self.drive, m.travel) {
            (Drive::Eased(t), Travel::Eased) => t.to(target, *m, now),
            (Drive::Spring(s), Travel::Spring) if s.spring() == m.spring() => {
                s.retarget(target, now)
            }
            (_, Travel::Eased) => {
                let mut t = Tween::new(x, *m);
                t.to(target, *m, now);
                self.drive = Drive::Eased(t);
            }
            (_, Travel::Spring) => {
                self.drive = Drive::Spring(SpringState::moving(m.spring(), x, v, target, now));
            }
        }
    }
    /// Let go at `x` moving `v` units per second, heading for `target`: a spring keeps the
    /// throw's momentum, an eased glide starts from rest at `x`.
    pub fn throw(&mut self, x: f32, v: f32, target: f32, m: &Motion, now: f32) {
        self.drive = match m.travel {
            Travel::Spring => Drive::Spring(SpringState::moving(m.spring(), x, v, target, now)),
            Travel::Eased => {
                let mut t = Tween::new(x, *m);
                t.to(target, *m, now);
                Drive::Eased(t)
            }
        };
    }
    /// The value at `now`.
    pub fn value(&self, now: f32) -> f32 {
        match &self.drive {
            Drive::Eased(t) => t.value(now),
            Drive::Spring(s) => s.value(now),
        }
    }
    /// Units per second at `now` (`0` for an eased glide: it starts from rest).
    pub fn velocity(&self, now: f32) -> f32 {
        match &self.drive {
            Drive::Eased(_) => 0.0,
            Drive::Spring(s) => s.velocity(now),
        }
    }
    /// The value it is heading for.
    pub fn target(&self) -> f32 {
        match &self.drive {
            Drive::Eased(t) => t.target(),
            Drive::Spring(s) => s.target(),
        }
    }
    /// At rest on its target: no need to draw another frame.
    pub fn settled(&self, now: f32) -> bool {
        match &self.drive {
            Drive::Eased(t) => t.is_done(now),
            Drive::Spring(s) => s.settled(now, 1e-3 * s.target().abs().max(1.0)),
        }
    }
}

/// A span `(x, width)` following a target span: the edge in the direction of travel leads and
/// the other trails, so the span stretches across the gap and then catches up.
#[derive(Clone, Copy, Debug)]
pub struct Stretch {
    left: Glide,
    right: Glide,
}

impl Stretch {
    /// The edges' speeds against the motion's own: the leading edge runs ahead, the trailing
    /// one just behind, so the stretch shows without the span arriving late.
    const LEAD: f32 = 1.6;
    const TRAIL: f32 = 1.0 / 1.15;

    /// At rest on `(x, w)`.
    pub fn new((x, w): (f32, f32), m: &Motion) -> Self {
        Self {
            left: Glide::new(x, m),
            right: Glide::new(x + w, m),
        }
    }
    /// Head for `(x, w)`, the leading edge first.
    pub fn to(&mut self, (x, w): (f32, f32), m: &Motion, now: f32) {
        if (x - self.left.target()).abs() < 1e-3 && (x + w - self.right.target()).abs() < 1e-3 {
            return;
        }
        let (lead, trail) = (m.speed(Self::LEAD), m.speed(Self::TRAIL));
        let right_ward = x + w > self.right.target();
        let (lm, rm) = if right_ward {
            (trail, lead)
        } else {
            (lead, trail)
        };
        self.left.to(x, &lm, now);
        self.right.to(x + w, &rm, now);
    }
    /// `(x, width)` at `now`.
    pub fn value(&self, now: f32) -> (f32, f32) {
        let l = self.left.value(now);
        (l, (self.right.value(now) - l).max(0.0))
    }
    pub fn settled(&self, now: f32) -> bool {
        self.left.settled(now) && self.right.settled(now)
    }
}

/// One frame of a digital glitch: slices of the shape jitter sideways and settle, with
/// colour-split ghosts and a flickering trace of what was there before.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glitch {
    /// How far the glitch has run, `0..=1`.
    pub progress: f32,
}

impl Glitch {
    /// Jitter steps over the whole glitch.
    const STEPS: f32 = 18.0;

    /// The jitter step (changes `STEPS` times over the glitch).
    pub fn step(self) -> u32 {
        (self.progress * Self::STEPS) as u32
    }
    /// How far slices and ghosts may stray, for a full-strength `max`.
    pub fn amplitude(self, max: f32) -> f32 {
        max * (1.0 - self.progress)
    }
    /// Sideways offset of slice `i` (deterministic jitter within `±amplitude(max)`).
    pub fn slice_dx(self, i: u32, max: f32) -> f32 {
        jitter(i, self.step()) * self.amplitude(max)
    }
    /// Whether slice `i` is blanked this frame (the cut-in starts torn).
    pub fn slice_hidden(self, i: u32) -> bool {
        self.progress < 0.15 && i == 1
    }
    /// Opacity of colour-split ghost `n` (`0..=1`).
    pub fn ghost(self, n: u32) -> f32 {
        let dim = if (self.step() + n) % 3 == 0 { 0.4 } else { 1.0 };
        0.45 * (1.0 - self.progress) * dim
    }
    /// Opacity of the flickering trace of the old shape (`0` when off this frame).
    pub fn trace(self) -> f32 {
        if self.step() % 2 == 0 && self.progress < 0.7 {
            0.35 * (1.0 - self.progress)
        } else {
            0.0
        }
    }
    /// A glitch as a clip lasting the motion's duration.
    pub fn clip(m: &Motion) -> Clip<Glitch> {
        let d = m.duration.as_secs_f32();
        Clip::from_fn(d, move |t| Glitch {
            progress: (t / d.max(1e-6)).min(1.0),
        })
    }
}

/// Deterministic jitter in `-1.0..=1.0` for slice `i` at glitch step `step`.
fn jitter(i: u32, step: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9E37_79B1) ^ step.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h % 2001) as f32 / 1000.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn eased() -> Motion {
        Motion::default().duration(Duration::from_millis(200))
    }

    #[test]
    fn an_eased_glide_lands_on_time() {
        let m = eased();
        let mut g = Glide::new(0.0, &m);
        g.to(10.0, &m, 1.0);
        assert!(g.value(1.1) > 0.0 && g.value(1.1) < 10.0);
        assert!(!g.settled(1.1));
        assert_eq!(g.value(1.2), 10.0);
        assert!(g.settled(1.2));
    }

    #[test]
    fn a_spring_glide_keeps_its_velocity_when_retargeted() {
        let m = eased().travel(Travel::Spring);
        let mut g = Glide::new(0.0, &m);
        g.to(100.0, &m, 0.0);
        let v = g.velocity(0.05);
        assert!(v > 0.0);
        g.to(-100.0, &m, 0.05);
        // Still moving forwards the instant after the target flipped.
        assert!((g.velocity(0.05) - v).abs() < 1e-3);
        assert!(g.settled(10.0) && (g.value(10.0) + 100.0).abs() < 0.2);
    }

    #[test]
    fn a_faster_motion_settles_sooner() {
        let m = eased().travel(Travel::Spring);
        let settle = |m: &Motion| {
            let mut g = Glide::new(0.0, m);
            g.to(1.0, m, 0.0);
            (0..2000)
                .map(|i| i as f32 / 200.0)
                .find(|t| g.settled(*t))
                .unwrap()
        };
        assert!(settle(&m.speed(2.0)) < settle(&m) * 0.6);
    }

    #[test]
    fn switching_travel_continues_from_the_current_value() {
        let m = eased();
        let mut g = Glide::new(0.0, &m);
        g.to(10.0, &m, 0.0);
        let mid = g.value(0.1);
        g.to(20.0, &m.travel(Travel::Spring), 0.1);
        assert!((g.value(0.1) - mid).abs() < 1e-3);
    }

    #[test]
    fn a_stretch_leads_with_the_edge_in_the_direction_of_travel() {
        let m = eased();
        let mut s = Stretch::new((0.0, 10.0), &m);
        s.to((100.0, 10.0), &m, 0.0);
        let (x, w) = s.value(0.1);
        assert!(w > 10.0, "stretched across the gap: {w}");
        assert!(x > 0.0);
        assert!(s.settled(1.0));
        assert_eq!(s.value(1.0), (100.0, 10.0));
    }

    #[test]
    fn a_sprung_stretch_lands_promptly_at_speed_one() {
        // A tab move on a smooth spring: within 1 px of the target in half a second.
        let m = Motion::default().travel(Travel::Spring);
        let mut s = Stretch::new((0.0, 80.0), &m);
        s.to((200.0, 80.0), &m, 0.0);
        let (x, w) = s.value(0.5);
        assert!((x - 200.0).abs() < 1.0 && (w - 80.0).abs() < 1.0, "{x} {w}");
        assert!(
            s.value(0.08).1 > 85.0,
            "still stretches: {:?}",
            s.value(0.08)
        );
    }

    #[test]
    fn a_glitch_runs_then_settles() {
        let c = Glitch::clip(&Motion::default().duration(Duration::from_millis(320)));
        assert_eq!(c.at(0.0).progress, 0.0);
        assert!(c.at(0.32).progress >= 1.0 - 1e-6);
        let g = c.at(0.1);
        for i in 0..3 {
            assert!(g.slice_dx(i, 14.0).abs() <= 14.0);
        }
        assert!(jitter(0, 3) != jitter(1, 3) || jitter(1, 3) != jitter(2, 3));
        assert_eq!(c.at(0.32).amplitude(14.0), 0.0);
        assert!(Glitch { progress: 0.0 }.slice_hidden(1));
        assert!(Glitch { progress: 0.0 }.trace() > 0.0);
        assert!(Glitch { progress: 0.0 }.ghost(1) > 0.0);
    }
}
