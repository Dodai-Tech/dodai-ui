//! Shared animation primitives: [`Easing`] curves and a [`Motion`] setting
//! (duration + easing) that components accept via `.motion(..)`. A
//! app-wide default can be installed with [`Motion::set_default`].

use gpui::{
    div, px, Animation, AnimationElement, AnimationExt, App, Div, ElementId, Global,
    InteractiveElement, IntoElement, ParentElement, Stateful, Styled,
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
    /// Damped spring (mass 1): `stiffness` and `damping` as in physics
    /// engines. Under-damped springs overshoot `1.0`; entry effects clamp the
    /// overshoot, value-in animations (charts, sliders) show it.
    Spring {
        stiffness: f32,
        damping: f32,
    },
    /// Ball drop: falls to the target and bounces to rest.
    Bounce,
    /// Decaying wobble around the target (rubber band).
    Elastic,
    /// Overshoots the target by roughly `overshoot * 10%` and eases back (`1.7` ≈ 10%).
    Back(f32),
    /// Pulls back first, then accelerates past the start (windup).
    Anticipate(f32),
}

impl Easing {
    /// Bouncy spring preset (overshoots about 15%).
    pub const SPRING: Easing = Easing::Spring {
        stiffness: 170.0,
        damping: 12.0,
    };
    /// Fast and tight: settles quickly with a small overshoot.
    pub const SPRING_STIFF: Easing = Easing::Spring {
        stiffness: 400.0,
        damping: 26.0,
    };
    /// Slow and loose: large, lingering wobble.
    pub const SPRING_WOBBLY: Easing = Easing::Spring {
        stiffness: 120.0,
        damping: 6.0,
    };
    /// Heavily damped: no overshoot, soft landing.
    pub const SPRING_GENTLE: Easing = Easing::Spring {
        stiffness: 100.0,
        damping: 20.0,
    };

    /// Eased progress for linear `t` (clamped to `0..=1`). Pure.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => bezier(0.42, 0.0, 1.0, 1.0, t),
            Easing::EaseOut => bezier(0.0, 0.0, 0.58, 1.0, t),
            Easing::EaseInOut => bezier(0.42, 0.0, 0.58, 1.0, t),
            Easing::CubicBezier(a, b, c, d) => bezier(a, b, c, d, t),
            Easing::Spring { stiffness, damping } => spring(stiffness, damping, t),
            Easing::Bounce => bounce(t),
            Easing::Elastic => elastic(t),
            Easing::Back(o) => {
                let u = t - 1.0;
                1.0 + u * u * ((o + 1.0) * u + o)
            }
            Easing::Anticipate(o) => t * t * ((o + 1.0) * t - o),
        }
    }
}

/// Closed-form damped spring step response at time `t` (seconds), snapped to
/// exactly `1.0` at `t == 1`.
fn spring(k: f32, c: f32, t: f32) -> f32 {
    let k = k.max(1.0);
    let w0 = k.sqrt();
    let z = c.max(0.0) / (2.0 * w0);
    let x = if z < 1.0 {
        let wd = w0 * (1.0 - z * z).sqrt();
        let e = (-z * w0 * t).exp();
        1.0 - e * ((wd * t).cos() + (z * w0 / wd) * (wd * t).sin())
    } else {
        let e = (-w0 * t).exp();
        1.0 - e * (1.0 + w0 * t)
    };
    // Blend the last 5% to the target so the curve ends exactly at 1.
    let snap = ((t - 0.95) / 0.05).clamp(0.0, 1.0);
    x + (1.0 - x) * snap
}

/// Ball-drop bounce (easeOutBounce). Pure.
fn bounce(t: f32) -> f32 {
    let (n, d) = (7.5625, 2.75);
    if t < 1.0 / d {
        n * t * t
    } else if t < 2.0 / d {
        let t = t - 1.5 / d;
        n * t * t + 0.75
    } else if t < 2.5 / d {
        let t = t - 2.25 / d;
        n * t * t + 0.9375
    } else {
        let t = t - 2.625 / d;
        n * t * t + 0.984375
    }
}

/// Elastic ease-out: decaying sine around the target. Pure.
fn elastic(t: f32) -> f32 {
    if t <= 0.0 || t >= 1.0 {
        return t;
    }
    let c = std::f32::consts::TAU / 3.0;
    2f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c).sin() + 1.0
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
pub fn entry(
    el: impl IntoElement,
    name: &'static str,
    motion: Motion,
) -> AnimationElement<Stateful<Div>> {
    let kind = motion.entry;
    let default = motion.duration;
    let delay = motion.delay.as_secs_f32();
    let run = kind.duration(default).as_secs_f32();
    let anim = if kind.is_empty() {
        Animation::new(Duration::from_millis(1))
    } else {
        Animation::new(Duration::from_secs_f32(run + delay))
    };
    let e = motion.easing;
    animate(
        div().flex().flex_col().child(el),
        name,
        anim,
        kind,
        delay,
        run,
        e,
        default,
    )
}

/// Like [`entry`] but animates `d` itself: no wrapper node.
pub fn entry_div(
    d: gpui::Div,
    name: &'static str,
    motion: Motion,
) -> AnimationElement<Stateful<Div>> {
    let kind = motion.entry;
    let default = motion.duration;
    let delay = motion.delay.as_secs_f32();
    let run = kind.duration(default).as_secs_f32();
    let anim = if kind.is_empty() {
        Animation::new(Duration::from_millis(1))
    } else {
        Animation::new(Duration::from_secs_f32(run + delay))
    };
    animate(d, name, anim, kind, delay, run, motion.easing, default)
}

#[allow(clippy::too_many_arguments)]
fn animate(
    d: gpui::Div,
    name: &'static str,
    anim: Animation,
    kind: EntrySet,
    delay: f32,
    run: f32,
    e: Easing,
    default: Duration,
) -> AnimationElement<Stateful<Div>> {
    d.relative().id((name, 0u32)).with_animation(
        ElementId::from((name, 1u32)),
        anim,
        move |el, p| {
            // Linear time -> local progress after the stagger delay -> eased.
            let local = ((p * (run + delay) - delay) / run.max(1e-3)).clamp(0.0, 1.0);
            let (o, dx, dy) = kind.frame(e.apply(local).clamp(0.0, 1.0), default);
            el.opacity(o).left(px(dx)).top(px(dy))
        },
    )
}

/// Eased `0..1` progress since this component first appeared; components
/// multiply their displayed value by it so progress, sliders, gauges and
/// charts animate in. `1.0` when entry animation is [`EntryKind::None`].
/// Replays whenever the surrounding keyed subtree is rebuilt.
pub fn grow(window: &mut gpui::Window, cx: &mut App, name: &'static str) -> f32 {
    let m = Motion::of(cx);
    // A set fill duration enables value-in even with no entry effect on.
    if m.entry.is_empty() && m.entry.grow_ms() == 0 {
        return 1.0;
    }
    let st = window.use_keyed_state((name, 2u32), cx, |_, _| std::time::Instant::now());
    let start = *st.read(cx);
    let dur = match m.entry.grow_ms() {
        0 => m.duration.as_secs_f32(),
        ms => ms as f32 / 1000.0,
    };
    let t = (start.elapsed().as_secs_f32() - m.delay.as_secs_f32()) / dur.max(1e-3);
    if t < 1.0 {
        window.request_animation_frame();
    }
    m.eased(t.max(0.0))
}

/// Staggered reveal: progress `0..=1` of item `i` of `n` while the whole chart
/// is at `g` (items overlap; later items start later). Pure.
pub fn reveal(g: f32, i: usize, n: usize) -> f32 {
    let start = 0.6 * i as f32 / n.max(1) as f32;
    ((g - start) / 0.4).clamp(0.0, 1.0)
}

/// Eased `0..1` value following `on` (retargets from the current value, so
/// quick toggles reverse smoothly). Keyed under `id`; keeps requesting frames
/// until settled.
pub fn transition(window: &mut gpui::Window, cx: &mut App, id: gpui::ElementId, on: bool) -> f32 {
    smooth(window, cx, id, if on { 1.0 } else { 0.0 })
}

/// Eased follower of an arbitrary `target`, keyed under `id`.
pub fn smooth(window: &mut gpui::Window, cx: &mut App, id: gpui::ElementId, target: f32) -> f32 {
    let m = Motion::of(cx);
    smooth_with(window, cx, id, target, m)
}

/// Like [`smooth`] with an explicit [`Motion`] (e.g. a spring for physical feel).
pub fn smooth_with(
    window: &mut gpui::Window,
    cx: &mut App,
    id: gpui::ElementId,
    target: f32,
    m: Motion,
) -> f32 {
    let st = window.with_id(id, |w| {
        w.use_keyed_state(("smooth", 0u32), cx, move |_, _| Tween::new(target, m))
    });
    st.update(cx, |t, _| t.to(target, m));
    let (v, done) = {
        let t = st.read(cx);
        (t.value(), t.is_done())
    };
    if !done {
        window.request_animation_frame();
    }
    v
}

/// Component-wise mix of two colours (`t` in `0..=1`). Pure.
pub fn mix(a: gpui::Hsla, b: gpui::Hsla, t: f32) -> gpui::Hsla {
    let (a, b) = (gpui::Rgba::from(a), gpui::Rgba::from(b));
    gpui::Rgba {
        r: lerp(a.r, b.r, t),
        g: lerp(a.g, b.g, t),
        b: lerp(a.b, b.b, t),
        a: lerp(a.a, b.a, t),
    }
    .into()
}

/// Mix two optional colours; `None` counts as the other colour, fully transparent.
pub fn mix_opt(a: Option<gpui::Hsla>, b: Option<gpui::Hsla>, t: f32) -> Option<gpui::Hsla> {
    match (a, b) {
        (Some(a), Some(b)) => Some(mix(a, b, t)),
        (Some(a), None) => Some(a.opacity(1.0 - t)),
        (None, Some(b)) => Some(b.opacity(t)),
        (None, None) => None,
    }
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
    /// Wait before the entry / value-in animation starts (stagger).
    pub delay: Duration,
    /// How components enter (see [`EntryKind`]).
    pub entry: EntrySet,
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
        }
    }
}

impl Global for Motion {}

impl Motion {
    pub fn new(duration: Duration, easing: Easing) -> Self {
        Self {
            duration,
            easing,
            delay: Duration::ZERO,
            entry: EntrySet::of(EntryKind::Fade),
        }
    }
    pub fn duration(mut self, d: Duration) -> Self {
        self.duration = d;
        self
    }
    pub fn millis(self, ms: u64) -> Self {
        self.duration(Duration::from_millis(ms))
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
    /// Eased progress for linear `t`.
    pub fn eased(&self, t: f32) -> f32 {
        self.easing.apply(t)
    }
    /// One-shot gpui animation with this duration and easing.
    pub fn animation(&self) -> Animation {
        let e = self.easing;
        Animation::new(self.duration).with_easing(move |t| e.apply(t).clamp(0.0, 1.0))
    }
    /// One-shot gpui animation driven by a composed [`Curve`] (the motion's
    /// own easing is ignored; put it in the curve).
    pub fn animation_with(&self, curve: Curve) -> Animation {
        Animation::new(self.duration).with_easing(move |t| curve.apply(t).clamp(0.0, 1.0))
    }
    /// Looping gpui animation with this duration and easing per cycle.
    pub fn looping(&self) -> Animation {
        self.animation().repeat()
    }
    /// The app-wide default (set via [`Motion::set_default`]).
    pub fn of(cx: &App) -> Motion {
        cx.try_global::<Motion>().copied().unwrap_or_default()
    }
    pub fn set_default(self, cx: &mut App) {
        cx.set_global(self);
    }
}

/// Wraps a subtree so every component inside uses `kind` for its entry and
/// value-in animation instead of the default [`Motion::entry`]. Components
/// render while their parent lays out, so the override is installed around the
/// child's `request_layout` and restored afterwards.
pub struct EntryScope {
    kind: EntrySet,
    delay: Duration,
    child: gpui::AnyElement,
    key: Option<ElementId>,
}

impl EntryScope {
    /// Give the scope an element id so replaying (new id) resets the subtree.
    pub fn key(mut self, id: impl Into<ElementId>) -> Self {
        self.key = Some(id.into());
        self
    }

    pub fn new(kind: impl Into<EntrySet>, child: impl IntoElement) -> Self {
        Self {
            kind: kind.into(),
            delay: Duration::ZERO,
            child: child.into_any_element(),
            key: None,
        }
    }
}

impl EntryScope {
    /// Start the scoped components' animations after `d` (stagger).
    pub fn delay(mut self, d: Duration) -> Self {
        self.delay = d;
        self
    }
}

impl IntoElement for EntryScope {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl gpui::Element for EntryScope {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        self.key.clone()
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut gpui::Window,
        cx: &mut App,
    ) -> (gpui::LayoutId, ()) {
        let prev = Motion::of(cx);
        prev.entry(self.kind).delay(self.delay).set_default(cx);
        let id = self.child.request_layout(window, cx);
        prev.set_default(cx);
        (id, ())
    }
    fn prepaint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        _: &mut (),
        window: &mut gpui::Window,
        cx: &mut App,
    ) {
        self.child.prepaint(window, cx);
    }
    fn paint(
        &mut self,
        _: Option<&gpui::GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        _: gpui::Bounds<gpui::Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut gpui::Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn entry_set_per_effect_duration() {
        use super::*;
        let d = Duration::from_millis(200);
        let s = EntrySet::NONE
            .with(EntryKind::Fade)
            .with_ms(EntryKind::Rise, 400);
        assert!((s.duration(d).as_secs_f32() * 1000.0 - 400.0).abs() < 0.01);
        let (o, _, y) = s.frame(0.5, d); // fade done, rise half-way
        assert!((o - 0.5).abs() < 1e-6 && (y - 7.0).abs() < 1e-6);
        let (o1, _, _) = s.frame(1.0, d);
        assert!((o1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn entry_set_composes() {
        use super::*;
        let s = EntrySet::NONE.with(EntryKind::Fade).with(EntryKind::Rise);
        let (o, x, y) = s.frame(0.5, Duration::from_millis(200));
        assert!((o - 0.25).abs() < 1e-6 && x == 0.0 && (y - 7.0).abs() < 1e-6);
        assert!(s
            .toggled(EntryKind::Fade)
            .toggled(EntryKind::Rise)
            .is_empty());
    }

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
    fn physics_easings_hit_endpoints() {
        for e in [
            Easing::Bounce,
            Easing::Elastic,
            Easing::Back(1.7),
            Easing::Anticipate(1.7),
            Easing::SPRING_STIFF,
            Easing::SPRING_WOBBLY,
            Easing::SPRING_GENTLE,
        ] {
            assert!(e.apply(0.0).abs() < 1e-3, "{e:?} start");
            assert!((e.apply(1.0) - 1.0).abs() < 1e-3, "{e:?} end");
        }
        assert!(Easing::Back(1.7).apply(0.7) > 1.0, "back overshoots");
        assert!(
            Easing::Anticipate(1.7).apply(0.2) < 0.0,
            "anticipate pulls back"
        );
        assert!(Easing::Bounce.apply(0.9) <= 1.0);
    }

    #[test]
    fn spring_overshoots_and_settles() {
        let e = Easing::SPRING;
        assert!(e.apply(0.0).abs() < 1e-4);
        assert!((e.apply(1.0) - 1.0).abs() < 1e-4);
        assert!((0..100).any(|i| e.apply(i as f32 / 100.0) > 1.02));
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
