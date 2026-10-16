//! Shared animation primitives: [`Easing`] curves and a [`Motion`] setting
//! (duration + easing) that components accept via `.motion(..)`. A
//! global default can be installed with [`Motion::set_global`].

use gpui::{
    div, Animation, AnimationElement, AnimationExt, App, Div, ElementId, Global, IntoElement,
    ParentElement, SharedString, Styled,
};
use std::rc::Rc;
use std::time::Duration;

/// Timing curve mapping linear progress `0..1` to eased progress `0..1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    /// Slow start, accelerates.
    EaseIn,
    /// Fast start, decelerates.
    EaseOut,
    /// Accelerates then decelerates.
    EaseInOut,
    /// CSS-style `cubic-bezier(x1, y1, x2, y2)`.
    CubicBezier(f32, f32, f32, f32),
}

impl Easing {
    /// Eased progress for linear `t` (clamped to `0..=1`). Pure.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => bezier(0.42, 0.0, 1.0, 1.0, t),
            Easing::EaseOut => bezier(0.0, 0.0, 0.58, 1.0, t),
            Easing::EaseInOut => bezier(0.42, 0.0, 0.58, 1.0, t),
            Easing::CubicBezier(a, b, c, d) => bezier(a, b, c, d, t),
        }
    }
}

/// Solve a CSS cubic-bezier for `y` at time `x` by bisection.
fn bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    let at = |s: f32, a: f32, b: f32| {
        let u = 1.0 - s;
        3.0 * u * u * s * a + 3.0 * u * s * s * b + s * s * s
    };
    let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        if at(mid, x1.clamp(0.0, 1.0), x2.clamp(0.0, 1.0)) < x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    at((lo + hi) / 2.0, y1, y2)
}

/// Composable progress curve `0..1 -> 0..1`. Cheap to clone; build from an
/// [`Easing`] or closure, then combine: [`window`](Curve::window) (stagger /
/// delay), [`reverse`](Curve::reverse), [`ping_pong`](Curve::ping_pong),
/// [`then`](Curve::then), [`pipe`](Curve::pipe).
#[derive(Clone)]
pub struct Curve(Rc<dyn Fn(f32) -> f32>);

impl From<Easing> for Curve {
    fn from(e: Easing) -> Self {
        Curve(Rc::new(move |t| e.apply(t)))
    }
}

impl Curve {
    pub fn new(f: impl Fn(f32) -> f32 + 'static) -> Self {
        Curve(Rc::new(f))
    }
    pub fn apply(&self, t: f32) -> f32 {
        (self.0)(t.clamp(0.0, 1.0))
    }
    /// Run only within `[start, end]` of the timeline: `0` before, `1` after.
    pub fn window(self, start: f32, end: f32) -> Curve {
        let span = (end - start).max(1e-6);
        Curve::new(move |t| self.apply(((t - start) / span).clamp(0.0, 1.0)))
    }
    /// Play backwards.
    pub fn reverse(self) -> Curve {
        Curve::new(move |t| self.apply(1.0 - t))
    }
    /// Forward in the first half, back in the second.
    pub fn ping_pong(self) -> Curve {
        Curve::new(move |t| self.apply(1.0 - (2.0 * t - 1.0).abs()))
    }
    /// `self` over the first half (output `0..0.5`), then `next` over the
    /// second (output `0.5..1`).
    pub fn then(self, next: Curve) -> Curve {
        Curve::new(move |t| {
            if t < 0.5 {
                self.apply(2.0 * t) * 0.5
            } else {
                0.5 + next.apply(2.0 * t - 1.0) * 0.5
            }
        })
    }
    /// Feed this curve's output through `next`.
    pub fn pipe(self, next: Curve) -> Curve {
        Curve::new(move |t| next.apply(self.apply(t)))
    }
    /// Curve for item `i` of `n` in a staggered group; `overlap` in `0..1`
    /// is how much neighbouring items run concurrently.
    pub fn stagger(self, i: usize, n: usize, overlap: f32) -> Curve {
        let (start, end) = stagger_window(i, n, overlap);
        self.window(start, end)
    }
    /// Map the eased progress onto `a..b`.
    pub fn value(&self, t: f32, a: f32, b: f32) -> f32 {
        lerp(a, b, self.apply(t))
    }
}

/// `(start, end)` timeline window of item `i` of `n`. Pure.
pub fn stagger_window(i: usize, n: usize, overlap: f32) -> (f32, f32) {
    if n <= 1 {
        return (0.0, 1.0);
    }
    let o = overlap.clamp(0.0, 0.95);
    let len = 1.0 / (1.0 + (n - 1) as f32 * (1.0 - o));
    let start = i as f32 * len * (1.0 - o);
    (start, start + len)
}

/// Fade `el` in the first time it appears (one-shot; replays after the
/// element leaves and re-enters the tree). Shared by every chart so all
/// components enter the same way; honours [`Motion::of`] unless overridden.
pub fn entry(el: impl IntoElement, name: &'static str, motion: Motion) -> AnimationElement<Div> {
    div().child(el).with_animation(
        ElementId::from(SharedString::from(format!("entry-{name}"))),
        motion.animation(),
        |el, p| el.opacity(p),
    )
}

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

/// Linear interpolation. Pure.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Duration + easing, with a speed multiplier.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motion {
    pub duration: Duration,
    pub easing: Easing,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(220),
            easing: Easing::EaseOut,
        }
    }
}

impl Global for Motion {}

impl Motion {
    pub fn new(duration: Duration, easing: Easing) -> Self {
        Self { duration, easing }
    }
    pub fn duration(mut self, d: Duration) -> Self {
        self.duration = d;
        self
    }
    pub fn millis(self, ms: u64) -> Self {
        self.duration(Duration::from_millis(ms))
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
    /// Eased progress for linear `t`.
    pub fn eased(&self, t: f32) -> f32 {
        self.easing.apply(t)
    }
    /// One-shot gpui animation with this duration and easing.
    pub fn animation(&self) -> Animation {
        let e = self.easing;
        Animation::new(self.duration).with_easing(move |t| e.apply(t))
    }
    /// One-shot gpui animation driven by a composed [`Curve`] (the motion's
    /// own easing is ignored; put it in the curve).
    pub fn animation_with(&self, curve: Curve) -> Animation {
        Animation::new(self.duration).with_easing(move |t| curve.apply(t))
    }
    /// Looping gpui animation with this duration and easing per cycle.
    pub fn looping(&self) -> Animation {
        self.animation().repeat()
    }
    /// The app-wide default (set via [`Motion::set_global`]).
    pub fn of(cx: &App) -> Motion {
        cx.try_global::<Motion>().copied().unwrap_or_default()
    }
    pub fn set_global(self, cx: &mut App) {
        cx.set_global(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn easing_endpoints_and_shape() {
        for e in [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::CubicBezier(0.2, 0.8, 0.2, 1.0),
        ] {
            assert!(e.apply(0.0).abs() < 1e-3);
            assert!((e.apply(1.0) - 1.0).abs() < 1e-3);
        }
        assert!(Easing::EaseIn.apply(0.3) < 0.3);
        assert!(Easing::EaseOut.apply(0.3) > 0.3);
        assert!((Easing::EaseInOut.apply(0.5) - 0.5).abs() < 1e-2);
    }

    #[test]
    fn curves_compose() {
        let lin = || Curve::from(Easing::Linear);
        let w = lin().window(0.5, 1.0);
        assert_eq!(w.apply(0.25), 0.0);
        assert!((w.apply(0.75) - 0.5).abs() < 1e-5);
        assert!((lin().reverse().apply(0.25) - 0.75).abs() < 1e-5);
        assert!((lin().ping_pong().apply(0.5) - 1.0).abs() < 1e-5);
        assert!(lin().ping_pong().apply(1.0).abs() < 1e-5);
        let seq = lin().then(lin().reverse());
        assert!((seq.apply(0.5) - 1.0).abs() < 1e-5);
        assert!((seq.apply(1.0) - 0.5).abs() < 1e-5);
        assert!(
            (lin().pipe(Easing::EaseIn.into()).apply(0.3) - Easing::EaseIn.apply(0.3)).abs() < 1e-5
        );
    }

    #[test]
    fn stagger_windows_cover_timeline() {
        let (s0, _) = stagger_window(0, 4, 0.5);
        let (_, e3) = stagger_window(3, 4, 0.5);
        assert!(s0.abs() < 1e-6 && (e3 - 1.0).abs() < 1e-5);
        assert!(stagger_window(1, 4, 0.0).0 > stagger_window(0, 4, 0.0).0);
    }

    #[test]
    fn speed_scales_duration() {
        let m = Motion::default().millis(200).speed(2.0);
        assert_eq!(m.duration, Duration::from_millis(100));
    }
}
