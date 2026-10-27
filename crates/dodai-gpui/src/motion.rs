//! Shared animation primitives: [`Easing`] curves and a [`Motion`] setting
//! (duration + easing) that components accept via `.motion(..)`. A
//! app-wide default can be installed with [`Motion::set_default`].

use gpui::{
    div, px, Animation, AnimationElement, AnimationExt, App, Div, ElementId, Global,
    InteractiveElement, IntoElement, ParentElement, Stateful, Styled,
};
use std::time::Duration;

pub use dodai_core::motion::{
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
    match frame_cap() {
        None => window.request_animation_frame(),
        Some(gap) => {
            // Every request inside the budget joins one tick, so they redraw together
            // (separate timers would each cause their own frame).
            let view = window.current_view();
            let first = PENDING.with(|p| {
                let mut p = p.borrow_mut();
                let first = p.is_empty();
                p.insert(view);
                first
            });
            if first {
                window.on_next_frame(move |window, cx| {
                    window
                        .spawn(cx, async move |cx| {
                            cx.background_executor().timer(gap).await;
                            cx.update(|_, cx| {
                                for v in PENDING.with(|p| std::mem::take(&mut *p.borrow_mut())) {
                                    cx.notify(v);
                                }
                            })
                            .ok();
                        })
                        .detach();
                });
            }
        }
    }
}

thread_local! {
    static PENDING: std::cell::RefCell<std::collections::HashSet<gpui::EntityId>> = Default::default();
    static FRAME_CAP: std::cell::Cell<Option<std::time::Duration>> =
        const { std::cell::Cell::new(None) };
}

/// Cap how fast running animations redraw (`None`: every display refresh). Looping animations
/// on a 120 Hz display cost twice the frames for no visible gain; 60 is plenty.
pub fn set_max_fps(fps: Option<u32>) {
    FRAME_CAP.with(|c| {
        c.set(fps.filter(|f| *f > 0).map(|f| {
            // Minus one display refresh of latency already spent waiting for the callback.
            std::time::Duration::from_secs_f32((1.0 / f as f32 - 1.0 / 120.0).max(0.0))
        }));
    });
}

fn frame_cap() -> Option<std::time::Duration> {
    FRAME_CAP.with(std::cell::Cell::get)
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

/// Drive reactive mixins from interaction state and return the layered [`dodai_core::motion::Pose`]
/// for this frame. Keyed under `id`; keeps requesting frames until the springs settle. The
/// personality comes from the active theme's motion, so each design system moves its own way.
pub fn react(
    window: &mut gpui::Window,
    cx: &mut App,
    id: gpui::ElementId,
    mixins: &[dodai_core::motion::Reactive],
    states: &[(dodai_core::motion::Trigger, bool)],
) -> dodai_core::motion::Pose {
    let personality = crate::motion::current(cx).personality;
    let mixins = mixins.to_vec();
    let st = window.with_id(id, |w| {
        w.use_keyed_state(("react", 0u32), cx, move |_, _| {
            mixins
                .into_iter()
                .fold(dodai_core::motion::Reactions::new(personality), |r, m| {
                    r.with(m)
                })
        })
    });
    let t = now();
    st.update(cx, |r, _| {
        for &(trigger, on) in states {
            r.set(trigger, on, t);
        }
    });
    let (pose, settled) = {
        let r = st.read(cx);
        (r.pose(t), r.settled(t))
    };
    if !settled {
        crate::motion::next_frame(window);
    }
    pose
}

/// Vertical hover-lift offset in px for the element keyed `id`: a spring in the theme's
/// personality, `0.0` at rest. Apply as `.mt(lift).mb(-lift)` so layout doesn't move.
pub fn hover_lift(
    window: &mut gpui::Window,
    cx: &mut App,
    id: gpui::ElementId,
    hovered: bool,
) -> f32 {
    use dodai_core::motion::{Reactive, Trigger};
    react(
        window,
        cx,
        id,
        &[Reactive::HoverLift],
        &[(Trigger::Hover, hovered)],
    )
    .dy
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

    /// Create a new [`EntryScope`].
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

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb(r: f32, g: f32, b: f32) -> gpui::Hsla {
        gpui::Rgba { r, g, b, a: 1.0 }.into()
    }

    #[test]
    fn mix_hits_both_ends_and_the_middle() {
        let (a, b) = (rgb(0.0, 0.0, 0.0), rgb(1.0, 0.5, 0.0));
        let close = |x: gpui::Hsla, y: gpui::Hsla| {
            let (x, y) = (gpui::Rgba::from(x), gpui::Rgba::from(y));
            (x.r - y.r).abs() + (x.g - y.g).abs() + (x.b - y.b).abs() < 1e-3
        };
        assert!(close(mix(a, b, 0.0), a));
        assert!(close(mix(a, b, 1.0), b));
        assert!(close(mix(a, b, 0.5), rgb(0.5, 0.25, 0.0)));
    }

    #[test]
    fn mixing_with_nothing_fades_to_transparent() {
        let c = rgb(0.2, 0.4, 0.6);
        assert_eq!(mix_opt(None, None, 0.5), None);
        let half = mix_opt(Some(c), None, 1.0).expect("a colour fades, it does not vanish");
        assert!(half.a < 1e-3);
    }
}
