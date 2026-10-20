//! Shared animation primitives: [`Easing`] curves and a [`Motion`] setting
//! (duration + easing) that components accept via `.motion(..)`. A
//! app-wide default can be installed with [`Motion::set_default`].

use gpui::{
    div, px, Animation, AnimationElement, AnimationExt, App, Div, ElementId, Global,
    InteractiveElement, IntoElement, ParentElement, Stateful, Styled,
};
use std::time::Duration;

pub use dodai_motion::{
    lerp, reveal, stagger_window, Curve, Easing, EntryKind, EntrySet, Motion, Tween,
};

/// The app-wide motion override installed by [`MotionExt::set_default`].
struct DefaultMotion(Motion);
impl Global for DefaultMotion {}

/// Seconds on gpui's monotonic clock (since first use): what [`Tween`] is driven with.
pub fn now() -> f32 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_secs_f32()
}

/// The motion in force (see [`MotionExt::of`]).
pub fn current(cx: &App) -> Motion {
    Motion::of(cx)
}

/// gpui glue for [`Motion`]: native animations and the app-wide default. Import it
/// (`use dodai_gpui::MotionExt`) to call `crate::motion::current(cx)`, `.animation()` and friends.
pub trait MotionExt: Sized {
    /// One-shot gpui animation with this duration and easing.
    fn animation(&self) -> Animation;
    /// One-shot gpui animation driven by a composed [`Curve`] (the motion's own easing is
    /// ignored; put it in the curve).
    fn animation_with(&self, curve: Curve) -> Animation;
    /// Looping gpui animation with this duration and easing per cycle.
    fn looping(&self) -> Animation;
    /// The motion in force: the one installed with [`set_default`](Self::set_default), or the
    /// active theme's own.
    fn of(cx: &App) -> Self;
    /// Install this motion app-wide (overrides the theme's).
    fn set_default(self, cx: &mut App);
}

impl MotionExt for Motion {
    fn animation(&self) -> Animation {
        let e = self.easing;
        Animation::new(self.duration).with_easing(move |t| e.apply(t).clamp(0.0, 1.0))
    }
    fn animation_with(&self, curve: Curve) -> Animation {
        Animation::new(self.duration).with_easing(move |t| curve.apply(t).clamp(0.0, 1.0))
    }
    fn looping(&self) -> Animation {
        self.animation().repeat()
    }
    fn of(cx: &App) -> Motion {
        match cx.try_global::<DefaultMotion>() {
            Some(m) => m.0,
            None if cx.has_global::<crate::DodaiTheme>() => crate::theme_of(cx).motion,
            None => Motion::default(),
        }
    }
    fn set_default(self, cx: &mut App) {
        cx.set_global(DefaultMotion(self));
    }
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
    let m = crate::motion::current(cx);
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
        crate::motion::next_frame(window);
    }
    m.eased(t.max(0.0))
}

/// Ask for another frame (animations call this while they are running).
/// With `DODAI_PROF=1` it counts who keeps the window redrawing, once a second.
#[track_caller]
pub fn next_frame(window: &gpui::Window) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *ON.get_or_init(|| std::env::var_os("DODAI_PROF").is_some()) {
        type Tally = (
            Option<std::time::Instant>,
            std::collections::HashMap<String, u32>,
        );
        thread_local! { static T: std::cell::RefCell<Tally> = Default::default(); }
        let l = std::panic::Location::caller();
        T.with(|t| {
            let mut t = t.borrow_mut();
            let t0 = *t.0.get_or_insert_with(std::time::Instant::now);
            *t.1.entry(format!("{}:{}", l.file(), l.line())).or_default() += 1;
            if t0.elapsed().as_secs_f32() >= 1.0 {
                let mut v: Vec<_> = t.1.drain().collect();
                v.sort_by_key(|x| std::cmp::Reverse(x.1));
                eprintln!("frames requested by: {v:?}");
                t.0 = Some(std::time::Instant::now());
            }
        });
    }
    window.request_animation_frame();
}

/// Eased `0..1` value following `on` (retargets from the current value, so
/// quick toggles reverse smoothly). Keyed under `id`; keeps requesting frames
/// until settled.
pub fn transition(window: &mut gpui::Window, cx: &mut App, id: gpui::ElementId, on: bool) -> f32 {
    smooth(window, cx, id, if on { 1.0 } else { 0.0 })
}

/// Eased follower of an arbitrary `target`, keyed under `id`.
pub fn smooth(window: &mut gpui::Window, cx: &mut App, id: gpui::ElementId, target: f32) -> f32 {
    let m = crate::motion::current(cx);
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
    st.update(cx, |t, _| t.to(target, m, now()));
    let (v, done) = {
        let t = st.read(cx);
        (t.value(now()), t.is_done(now()))
    };
    if !done {
        crate::motion::next_frame(window);
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
        let prev = crate::motion::current(cx);
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
