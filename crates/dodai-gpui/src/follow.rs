//! How instruments follow a changing value. Each animated instrument owns a [`Follow`]: a stable
//! id for its spring state and an optional [`NeedleResponse`] override. Without an override the
//! design's own feel (`ChartStyle::response`) applies, so a Cyberpunk needle snaps and a
//! Neo-Brutalism one overshoots.

use crate::motion::now;
use crate::theme_of;
use dodai_core::motion::{Spring, SpringState};
use dodai_core::NeedleResponse;
use gpui::{App, ElementId, SharedString, Window, WindowId};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    /// Per window: how many components claimed each call-site id during the current frame.
    static CLAIMS: RefCell<HashMap<WindowId, HashMap<ElementId, u32>>> = RefCell::default();
}

/// The next free instance number for `id` in this draw. The tally is cleared as soon as the draw
/// that started it ends, so the same component keeps the same number while the layout is stable.
/// (Clearing "before the next frame" is not enough: input events can draw several times between
/// two frames, and every extra draw would hand out new numbers, restarting every instrument.)
fn claim(w: &mut Window, cx: &mut App, id: &ElementId) -> u32 {
    let win = w.window_handle().window_id();
    let fresh = CLAIMS.with(|c| !c.borrow().contains_key(&win));
    if fresh {
        w.defer(cx, move |_, _| {
            CLAIMS.with(|c| {
                c.borrow_mut().remove(&win);
            });
        });
    }
    CLAIMS.with(|c| {
        let mut c = c.borrow_mut();
        let n = c.entry(win).or_default().entry(id.clone()).or_insert(0);
        let k = *n;
        *n += 1;
        k
    })
}

/// Seconds of animation time (follows the app's Motion speed): what scrolling and sweeping
/// instruments are driven with.
pub fn clock() -> f32 {
    crate::motion::time()
}

/// Asks for another frame, unless instruments are not animated.
pub fn next_frame(w: &Window) {
    if crate::motion::enabled() {
        crate::motion::next_frame(w);
    }
}

/// Called when the item under the pointer (else the selected one) changes: `Some(index)`, or
/// `None` when nothing has the focus any more.
pub type FocusFn = Rc<dyn Fn(Option<usize>, &mut Window, &mut App)>;

/// Animation identity and feel of one instrument.
#[derive(Clone)]
pub struct Follow {
    id: ElementId,
    /// Set for call-site ids: the instance number claimed this frame, so components built at
    /// the same spot (in a loop) get separate state.
    slot: Option<Rc<Cell<Option<u32>>>>,
    response: Option<NeedleResponse>,
    animated: bool,
    interactive: bool,
    on_focus: Option<FocusFn>,
}

impl Follow {
    /// An id derived from the call site of the instrument's constructor. Instruments built on
    /// the same source line (in a loop) are told apart by their order within the frame; give
    /// them an explicit `.id(..)` if that order can change between frames.
    #[track_caller]
    pub fn here() -> Self {
        let l = std::panic::Location::caller();
        Self {
            id: ElementId::from(SharedString::from(format!(
                "needle-{}:{}:{}",
                l.file(),
                l.line(),
                l.column()
            ))),
            slot: Some(Rc::default()),
            response: None,
            animated: true,
            interactive: true,
            on_focus: None,
        }
    }
    /// An id from a name, for components that already have one.
    pub fn named(id: &str) -> Self {
        Self {
            id: ElementId::from(SharedString::from(format!("follow-{id}"))),
            slot: None,
            response: None,
            animated: true,
            interactive: true,
            on_focus: None,
        }
    }
    /// Turns following on or off for this component; off, it shows its value at once.
    pub fn with_animated(mut self, on: bool) -> Self {
        self.animated = on;
        self
    }
    /// Whether the component reacts to the pointer (hover readouts, click selection,
    /// tooltips). Off, it is a plain picture.
    pub fn with_interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
    /// Whether the component reacts to the pointer.
    pub fn interactive(&self) -> bool {
        self.interactive
    }
    /// Sets the callback told which item has the focus.
    pub fn with_on_focus(mut self, f: FocusFn) -> Self {
        self.on_focus = Some(f);
        self
    }
    /// The callback told which item has the focus, if one is set.
    pub fn on_focus(&self) -> Option<FocusFn> {
        self.on_focus.clone()
    }
    /// A sibling identity for a second piece of state of the same component.
    pub fn sub(&self, name: &str) -> Self {
        Self {
            id: ElementId::from((self.id.clone(), SharedString::from(name.to_string()))),
            slot: self.slot.clone(),
            response: self.response,
            animated: self.animated,
            interactive: self.interactive,
            on_focus: self.on_focus.clone(),
        }
    }
    /// The state key of this component in the current frame: its id, plus the instance number
    /// for call-site ids.
    pub fn element_id(&self, w: &mut Window, cx: &mut App) -> ElementId {
        match &self.slot {
            Some(slot) => {
                let n = match slot.get() {
                    Some(n) => n,
                    None => {
                        let n = claim(w, cx, &self.id);
                        slot.set(Some(n));
                        n
                    }
                };
                ElementId::from((self.id.clone(), SharedString::from(format!("#{n}"))))
            }
            None => self.id.clone(),
        }
    }
    /// Replaces the id.
    pub fn with_id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self.slot = None;
        self
    }
    /// Overrides the design's response.
    pub fn with_response(mut self, r: NeedleResponse) -> Self {
        self.response = Some(r);
        self
    }
}

/// Adds `.id(..)` and `.response(..)` to an instrument with a `follow: Follow` field.
#[macro_export]
macro_rules! follow_api {
    () => {
        /// Stable id of this instrument's animation state. Needed only when several
        /// instruments are built from one source line and their order can change.
        pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
            self.follow = self.follow.with_id(id);
            self
        }
        /// How the needle follows a changing value; by default the design's own.
        pub fn response(mut self, r: dodai_core::NeedleResponse) -> Self {
            self.follow = self.follow.with_response(r);
            self
        }
        /// Animates towards new values (the default) or shows them at once. The global
        /// Animate switch still turns every component off.
        pub fn animate(mut self, on: bool) -> Self {
            self.follow = self.follow.with_animated(on);
            self
        }
        /// Whether the component reacts to the pointer: hover readouts, click selection and
        /// tooltips. Off, it is a plain picture. On by default.
        pub fn interactive(mut self, on: bool) -> Self {
            self.follow = self.follow.with_interactive(on);
            self
        }
        /// Called with the item that has the focus (hovered, else selected) each time it
        /// changes; `None` when it is lost.
        pub fn on_focus(
            mut self,
            f: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
        ) -> Self {
            self.follow = self.follow.with_on_focus(std::rc::Rc::new(f));
            self
        }
        /// Like `response` for an optional override (`None` keeps the design's own).
        pub fn response_opt(self, r: Option<dodai_core::NeedleResponse>) -> Self {
            match r {
                Some(r) => self.response(r),
                None => self,
            }
        }
    };
}

struct Needle {
    response: NeedleResponse,
    spring: SpringState,
    last: f32,
}

fn state(
    w: &mut Window,
    cx: &mut App,
    f: &Follow,
    part: u8,
    target: f32,
    angle: bool,
) -> Option<f32> {
    let response = f
        .response
        .unwrap_or_else(|| theme_of(cx).chart_style().response);
    if !crate::motion::enabled() || !f.animated {
        return None;
    }
    let (secs, bounce) = response.spring()?;
    let spring = Spring::from_response(secs / crate::motion::speed(), bounce);
    // A value starts at zero and sweeps up to its reading, like an instrument's self-test.
    let start = if angle { target } else { 0.0 };
    let key = f.element_id(w, cx);
    let st = w.with_id(key, |w| {
        w.use_keyed_state(("needle", u32::from(part)), cx, move |_, _| Needle {
            response,
            spring: SpringState::new(spring, start),
            last: f32::NAN,
        })
    });
    let t = now();
    let (v, settled) = st.update(cx, |n, _| {
        if n.response != response {
            n.spring = SpringState::new(spring, n.spring.value(t));
            n.response = response;
        }
        if n.last.is_nan() || n.last != target {
            n.last = target;
            let aim = if angle {
                let cur = n.spring.value(t);
                cur + (target - cur + 180.0).rem_euclid(360.0) - 180.0
            } else {
                target
            };
            n.spring.retarget(aim, t);
        }
        (n.spring.value(t), n.spring.settled(t, 0.01))
    });
    if !settled {
        next_frame(w);
    }
    Some(v)
}

/// `target` smoothed per the instrument's response. `part` tells several values of one
/// instrument apart.
pub fn follow(w: &mut Window, cx: &mut App, f: &Follow, part: u8, target: f32) -> f32 {
    state(w, cx, f, part, target, false).unwrap_or(target)
}

/// Like [`follow`] for a heading in degrees: turns the short way round and returns the angle
/// wrapped to `0..360`.
pub fn follow_angle(w: &mut Window, cx: &mut App, f: &Follow, part: u8, target: f32) -> f32 {
    let target = target.rem_euclid(360.0);
    state(w, cx, f, part, target, true)
        .unwrap_or(target)
        .rem_euclid(360.0)
}

struct Many {
    sig: (u32, u32),
    springs: Vec<SpringState>,
    last: Vec<f32>,
}

/// Every value of `target` followed at once, for a chart's data. Values that appear start at
/// their target; the rest ease with the component's response. Requests frames until settled.
pub fn follow_all(w: &mut Window, cx: &mut App, f: &Follow, target: &[f32]) -> Vec<f32> {
    if !crate::motion::enabled() || !f.animated {
        return target.to_vec();
    }
    // Data follows the app's motion set (duration and personality), so the Motion controls
    // steer charts too; a component's own `.response()` overrides it.
    let (_, secs, bounce) = match f.response {
        Some(r) => match r.spring() {
            Some((s, b)) => (r, s, b),
            None => return target.to_vec(),
        },
        None => {
            let m = crate::motion::current(cx);
            if m.duration.is_zero() {
                return target.to_vec();
            }
            let bounce = match m.personality {
                dodai_core::motion::Personality::Snappy => 0.0,
                dodai_core::motion::Personality::Smooth => 0.05,
                dodai_core::motion::Personality::Playful => 0.15,
            };
            (
                theme_of(cx).chart_style().response,
                m.duration.as_secs_f32() * 0.6,
                bounce,
            )
        }
    };
    // Data does not overshoot: bars and areas keep a gentle settle at most.
    let spring = Spring::from_response(secs / crate::motion::speed(), bounce.min(0.15));
    let sig = (
        (secs * 1000.0 / crate::motion::speed()) as u32,
        (bounce.min(0.15) * 1000.0) as u32,
    );
    let key = f.element_id(w, cx);
    let st = w.with_id(key, |w| {
        w.use_keyed_state(("many", 0u32), cx, |_, _| Many {
            sig,
            springs: Vec::new(),
            last: Vec::new(),
        })
    });
    let t = now();
    let eps = (target.iter().fold(0.0_f32, |m, v| m.max(v.abs())) * 0.002).max(1e-4);
    let (out, settled) = st.update(cx, |m, _| {
        if m.sig != sig {
            m.sig = sig;
            for (sp, v) in m.springs.iter_mut().zip(&m.last) {
                *sp = SpringState::new(spring, *v);
            }
        }
        m.springs.truncate(target.len());
        m.last.truncate(target.len());
        for (i, v) in target.iter().enumerate() {
            if i >= m.springs.len() {
                m.springs.push(SpringState::new(spring, *v));
                m.last.push(*v);
            } else if m.last[i] != *v {
                m.last[i] = *v;
                m.springs[i].retarget(*v, t);
            }
        }
        (
            m.springs.iter().map(|s| s.value(t)).collect::<Vec<_>>(),
            m.springs.iter().all(|s| s.settled(t, eps)),
        )
    });
    if !settled {
        next_frame(w);
    }
    out
}

/// An axis range followed. A chart's nice-rounded range changes in steps as its data moves; easing
/// it keeps the whole plot from jumping when the data crosses a step.
pub fn follow_range(
    w: &mut Window,
    cx: &mut App,
    f: &Follow,
    name: &str,
    (lo, hi): (f32, f32),
) -> (f32, f32) {
    let v = follow_all(w, cx, &f.sub(name), &[lo, hi]);
    match v.as_slice() {
        [a, b] if (b - a).abs() > 1e-9 => (*a, *b),
        _ => (lo, hi),
    }
}

/// The `f32` of each `(label, value)` pair followed, in place.
pub fn follow_pairs<T>(w: &mut Window, cx: &mut App, f: &Follow, items: &mut [(T, f32)]) {
    let flat: Vec<f32> = items.iter().map(|p| p.1).collect();
    for (p, v) in items.iter_mut().zip(follow_all(w, cx, f, &flat)) {
        p.1 = v;
    }
}

/// The two numbers of each `(label, a, b)` row followed, in place.
pub fn follow_triples<T>(w: &mut Window, cx: &mut App, f: &Follow, items: &mut [(T, f32, f32)]) {
    let flat: Vec<f32> = items.iter().flat_map(|p| [p.1, p.2]).collect();
    let mut shown = follow_all(w, cx, f, &flat).into_iter();
    for p in items.iter_mut() {
        p.1 = shown.next().unwrap_or(p.1);
        p.2 = shown.next().unwrap_or(p.2);
    }
}

/// A grid of values followed, in place.
pub fn follow_rows(w: &mut Window, cx: &mut App, f: &Follow, rows: &mut [Vec<f32>]) {
    let flat: Vec<f32> = rows.iter().flatten().copied().collect();
    let mut shown = follow_all(w, cx, f, &flat).into_iter();
    for v in rows.iter_mut().flatten() {
        *v = shown.next().unwrap_or(*v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{div, prelude::*, px, size, Context, Render, TestAppContext};

    thread_local! {
        static KEYS: RefCell<Vec<Vec<String>>> = const { RefCell::new(Vec::new()) };
    }

    /// Builds three instruments from one source line each draw, like a loop in a showcase.
    struct Host;

    impl Render for Host {
        fn render(&mut self, w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let keys = (0..3)
                .map(|_| Follow::here().element_id(w, cx).to_string())
                .collect();
            KEYS.with(|k| k.borrow_mut().push(keys));
            div()
        }
    }

    /// Input events can draw several times between two frames. Every draw must hand the same
    /// instance numbers to the same instruments, or each one restarts from zero.
    #[gpui::test]
    fn instance_numbers_survive_extra_draws(cx: &mut TestAppContext) {
        KEYS.with(|k| k.borrow_mut().clear());
        let (_, vcx) = cx.add_window_view(|_, _| Host);
        vcx.simulate_resize(size(px(100.0), px(100.0)));
        vcx.run_until_parked();
        for _ in 0..3 {
            vcx.update(|w, cx| w.draw(cx).clear());
            vcx.run_until_parked();
        }
        let all = KEYS.with(|k| k.borrow().clone());
        assert!(all.len() >= 4, "drew {} times", all.len());
        for k in &all {
            assert_eq!(k, &all[0], "instance numbers changed between draws");
        }
    }
}
