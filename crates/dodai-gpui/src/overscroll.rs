//! [`Overscroll`]: Material-style stretch / bounce when a scroll container is
//! pushed past either end, springing back on release.
//!
//! gpui clamps scrolling hard at the ends and exposes no gesture phases, so
//! this infers "pushed past the end" from wheel deltas that arrive while the
//! container is already at that end, and "released" from a short input gap.
//! Apply [`offset`](Overscroll::offset) as a translation of the scroll
//! container (e.g. `.relative().top(px(o.offset()))`) and call
//! [`settle`](Overscroll::settle) every frame while [`active`](Overscroll::active).

use gpui::{Pixels, ScrollHandle};
use std::time::{Duration, Instant};

/// Feel of the effect.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverscrollFeel {
    /// Maximum stretch in px (asymptote of the resistance curve).
    pub max: f32,
    /// Spring stiffness (1/s²) pulling back to rest.
    pub stiffness: f32,
    /// Spring damping (1/s). Below `2·√stiffness` it overshoots slightly.
    pub damping: f32,
}

impl OverscrollFeel {
    /// Material 3: soft, long stretch, critically damped (no wobble).
    pub const SOFT: Self = Self {
        max: 96.0,
        stiffness: 160.0,
        damping: 25.3,
    };
    /// Snappier, shorter, with a visible rebound.
    pub const SNAPPY: Self = Self {
        max: 56.0,
        stiffness: 420.0,
        damping: 22.0,
    };
}

/// How long after the last wheel event the gesture counts as released.
const RELEASE: Duration = Duration::from_millis(90);

/// Rubber-band resistance (pure): the share of a raw `delta` that moves the
/// content when it is already stretched by `stretch` (same sign = pushing
/// further). Pulling further gets harder as `|stretch|` nears `max`.
pub fn resist(delta: f32, stretch: f32, max: f32) -> f32 {
    if delta == 0.0 || max <= 0.0 {
        return 0.0;
    }
    if delta * stretch < 0.0 {
        // Moving back towards rest: no resistance.
        return delta;
    }
    let k = (1.0 - stretch.abs() / max).clamp(0.0, 1.0);
    delta * k * k * 0.5
}

#[derive(Clone, Copy, Debug)]
pub struct Overscroll {
    feel: OverscrollFeel,
    s: f32,
    v: f32,
    last_input: Option<Instant>,
}

impl Default for Overscroll {
    fn default() -> Self {
        Self::new(OverscrollFeel::SOFT)
    }
}

impl Overscroll {
    pub fn new(feel: OverscrollFeel) -> Self {
        Self {
            feel,
            s: 0.0,
            v: 0.0,
            last_input: None,
        }
    }

    /// Feel for a theme: soft for rounded (Material) shapes, snappy otherwise.
    pub fn for_theme(t: &dodai_core::Theme) -> OverscrollFeel {
        if t.shape.kind() == dodai_core::ShapeKind::Rounded {
            OverscrollFeel::SOFT
        } else {
            OverscrollFeel::SNAPPY
        }
    }

    pub fn set_feel(&mut self, feel: OverscrollFeel) {
        self.feel = feel;
    }

    /// Feed one wheel event. `dy` is the content delta in px (positive moves
    /// content down, i.e. scrolls towards the top). Returns true if the event
    /// was absorbed by the stretch (caller should repaint).
    pub fn on_wheel(&mut self, dy: f32, at_top: bool, at_bottom: bool, now: Instant) -> bool {
        let pushing_past = (dy > 0.0 && at_top) || (dy < 0.0 && at_bottom);
        let relaxing = self.s != 0.0 && dy * self.s < 0.0;
        if !pushing_past && !relaxing {
            return false;
        }
        self.s = (self.s + resist(dy, self.s, self.feel.max)).clamp(-self.feel.max, self.feel.max);
        self.v = 0.0;
        self.last_input = Some(now);
        true
    }

    /// Advance the spring by `dt` seconds at `now`. Holds while input is
    /// still arriving, then springs back to rest.
    pub fn settle(&mut self, now: Instant, dt: f32) {
        if self.s == 0.0 && self.v == 0.0 {
            return;
        }
        if self
            .last_input
            .is_some_and(|t| now.duration_since(t) < RELEASE)
        {
            return;
        }
        let n = ((dt / (1.0 / 240.0)).ceil() as u32).clamp(1, 24);
        let h = dt / n as f32;
        for _ in 0..n {
            let a = -self.feel.stiffness * self.s - self.feel.damping * self.v;
            self.v += a * h;
            self.s += self.v * h;
        }
        if self.s.abs() < 0.2 && self.v.abs() < 4.0 {
            self.s = 0.0;
            self.v = 0.0;
            self.last_input = None;
        }
    }

    /// Current translation in px (positive = content pulled down).
    pub fn offset(&self) -> f32 {
        self.s
    }

    /// True while stretched or springing: keep requesting frames.
    pub fn active(&self) -> bool {
        self.s != 0.0 || self.v != 0.0
    }
}

/// `(at_top, at_bottom)` of a scroll handle (a container with no range is
/// both, so either direction stretches it).
pub fn edges(handle: &ScrollHandle) -> (bool, bool) {
    let off = f32::from(handle.offset().y);
    let max: Pixels = handle.max_offset().height;
    let max = f32::from(max);
    (off >= -0.5, -off >= max - 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t0() -> Instant {
        Instant::now()
    }

    #[test]
    fn resistance_grows_with_stretch_and_is_free_when_relaxing() {
        let a = resist(10.0, 0.0, 100.0);
        let b = resist(10.0, 80.0, 100.0);
        assert!(a > b && b > 0.0, "{a} {b}");
        assert_eq!(resist(-10.0, 50.0, 100.0), -10.0, "relaxing is 1:1");
        assert_eq!(resist(10.0, 100.0, 100.0), 0.0, "wall at max");
    }

    #[test]
    fn stretch_never_exceeds_max() {
        let mut o = Overscroll::new(OverscrollFeel::SOFT);
        let now = t0();
        for _ in 0..500 {
            o.on_wheel(40.0, true, false, now);
        }
        assert!(o.offset() <= OverscrollFeel::SOFT.max + 1e-3);
        assert!(o.offset() > 0.0);
    }

    #[test]
    fn only_stretches_when_pushing_past_an_end() {
        let mut o = Overscroll::default();
        assert!(!o.on_wheel(20.0, false, false, t0()), "mid-page ignored");
        assert!(
            !o.on_wheel(-20.0, true, false, t0()),
            "away from the top ignored"
        );
        assert!(o.on_wheel(20.0, true, false, t0()));
        assert!(o.offset() > 0.0, "pulled down past the top");
        let mut b = Overscroll::default();
        assert!(b.on_wheel(-20.0, false, true, t0()));
        assert!(b.offset() < 0.0, "pulled up past the bottom");
    }

    #[test]
    fn holds_while_input_flows_then_springs_back_to_zero() {
        let mut o = Overscroll::new(OverscrollFeel::SOFT);
        let t = t0();
        o.on_wheel(60.0, true, false, t);
        let held = o.offset();
        o.settle(t + Duration::from_millis(30), 0.016);
        assert_eq!(o.offset(), held, "no recoil mid-gesture");
        let mut now = t + Duration::from_millis(100);
        for _ in 0..240 {
            o.settle(now, 1.0 / 60.0);
            now += Duration::from_millis(16);
        }
        assert_eq!(o.offset(), 0.0);
        assert!(!o.active());
    }

    #[test]
    fn soft_feel_does_not_overshoot_but_snappy_rebounds() {
        let run = |feel| {
            let mut o = Overscroll::new(feel);
            let t = t0();
            o.on_wheel(200.0, true, false, t);
            let mut now = t + Duration::from_millis(100);
            let mut min = f32::MAX;
            for _ in 0..300 {
                o.settle(now, 1.0 / 120.0);
                now += Duration::from_millis(8);
                min = min.min(o.offset());
            }
            min
        };
        assert!(run(OverscrollFeel::SOFT) >= -0.5, "critically damped");
        assert!(run(OverscrollFeel::SNAPPY) < -0.5, "underdamped rebounds");
    }

    #[test]
    fn relaxing_wheel_unwinds_the_stretch() {
        let mut o = Overscroll::default();
        let t = t0();
        o.on_wheel(40.0, true, false, t);
        let before = o.offset();
        assert!(o.on_wheel(-10.0, false, false, t));
        assert!(o.offset() < before);
    }
}
