//! How instruments follow a changing value. Each animated instrument owns a [`Follow`]: a stable
//! id for its spring state and an optional [`NeedleResponse`] override. Without an override the
//! design's own feel (`ChartStyle::response`) applies, so a Cyberpunk needle snaps and a
//! Neo-Brutalism one overshoots.

use crate::motion::now;
use crate::theme_of;
use dodai_core::NeedleResponse;
use dodai_motion::{Spring, SpringState};
use gpui::{App, ElementId, SharedString, Window, WindowId};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

thread_local! {
    /// Per window: how many components claimed each call-site id during the current frame.
    static CLAIMS: RefCell<HashMap<WindowId, HashMap<ElementId, u32>>> = RefCell::default();
}

/// The next free instance number for `id` in this frame. The tally is cleared before the next
/// frame is drawn, so the same component keeps the same number while the layout is stable.
fn claim(w: &mut Window, id: &ElementId) -> u32 {
    let win = w.window_handle().window_id();
    let fresh = CLAIMS.with(|c| !c.borrow().contains_key(&win));
    if fresh {
        w.on_next_frame(move |_, _| {
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

thread_local! {
    static ANIMATED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Whether instruments animate: needles follow values with a spring, sweeps and flashes run.
/// On by default.
pub fn instruments_animated() -> bool {
    ANIMATED.with(std::cell::Cell::get)
}

/// Switches every instrument's animation on or off and repaints. Off, needles jump to their
/// value and sweeps, flashes and live traces stand still.
pub fn set_instruments_animated(cx: &mut App, on: bool) {
    ANIMATED.with(|c| c.set(on));
    cx.refresh_windows();
}

thread_local! {
    static SPEED: std::cell::Cell<f32> = const { std::cell::Cell::new(1.0) };
    /// `(virtual seconds, real seconds at the last read)`.
    static CLOCK: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((0.0, 0.0)) };
}

/// How fast instrument motion runs: 1 is real time, 2 twice as fast.
pub fn animation_speed() -> f32 {
    SPEED.with(std::cell::Cell::get)
}

/// Sets the animation speed (clamped to 0.1..=4) and repaints. It scales sweeps, flashes, live
/// traces, the showcase demo values and how quickly needles follow.
pub fn set_animation_speed(cx: &mut App, speed: f32) {
    SPEED.with(|c| c.set(speed.clamp(0.1, 4.0)));
    cx.refresh_windows();
}

/// Seconds on the animation clock: it runs at [`animation_speed`] and stands still when
/// animation is off. Changing the speed never makes it jump.
pub fn animation_time() -> f32 {
    let real = now();
    CLOCK.with(|c| {
        let (virt, last) = c.get();
        let virt = if instruments_animated() {
            virt + (real - last).max(0.0) * animation_speed()
        } else {
            virt
        };
        c.set((virt, real));
        virt
    })
}

#[cfg(feature = "charts")]
pub(crate) fn clock() -> f32 {
    animation_time()
}

/// Asks for another frame, unless instruments are not animated.
pub(crate) fn next_frame(w: &Window) {
    if instruments_animated() {
        crate::motion::next_frame(w);
    }
}

/// Animation identity and feel of one instrument.
#[derive(Clone)]
pub struct Follow {
    id: ElementId,
    /// Set for call-site ids: the instance number claimed this frame, so components built at
    /// the same spot (in a loop) get separate state.
    slot: Option<Rc<Cell<Option<u32>>>>,
    response: Option<NeedleResponse>,
    animated: bool,
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
        }
    }
    /// An id from a name, for components that already have one.
    #[cfg(feature = "charts")]
    pub fn named(id: &str) -> Self {
        Self {
            id: ElementId::from(SharedString::from(format!("follow-{id}"))),
            slot: None,
            response: None,
            animated: true,
        }
    }
    /// Turns following on or off for this component; off, it shows its value at once.
    pub fn with_animated(mut self, on: bool) -> Self {
        self.animated = on;
        self
    }
    /// A sibling identity for a second piece of state of the same component.
    #[cfg(feature = "charts")]
    pub(crate) fn sub(&self, name: &str) -> Self {
        Self {
            id: ElementId::from((self.id.clone(), SharedString::from(name.to_string()))),
            slot: self.slot.clone(),
            response: self.response,
            animated: self.animated,
        }
    }
    /// The state key of this component in the current frame: its id, plus the instance number
    /// for call-site ids.
    pub(crate) fn element_id(&self, w: &mut Window) -> ElementId {
        match &self.slot {
            Some(slot) => {
                let n = match slot.get() {
                    Some(n) => n,
                    None => {
                        let n = claim(w, &self.id);
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
        /// Like `response` for an optional override (`None` keeps the design's own).
        pub fn response_opt(self, r: Option<dodai_core::NeedleResponse>) -> Self {
            match r {
                Some(r) => self.response(r),
                None => self,
            }
        }
    };
}
pub(crate) use follow_api;

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
    if !instruments_animated() || !f.animated {
        return None;
    }
    let (secs, bounce) = response.spring()?;
    let spring = Spring::from_response(secs / animation_speed(), bounce);
    // A value starts at zero and sweeps up to its reading, like an instrument's self-test.
    let start = if angle { target } else { 0.0 };
    let key = f.element_id(w);
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
pub(crate) fn follow(w: &mut Window, cx: &mut App, f: &Follow, part: u8, target: f32) -> f32 {
    state(w, cx, f, part, target, false).unwrap_or(target)
}

#[cfg(feature = "charts")]
/// Like [`follow`] for a heading in degrees: turns the short way round and returns the angle
/// wrapped to `0..360`.
pub(crate) fn follow_angle(w: &mut Window, cx: &mut App, f: &Follow, part: u8, target: f32) -> f32 {
    let target = target.rem_euclid(360.0);
    state(w, cx, f, part, target, true)
        .unwrap_or(target)
        .rem_euclid(360.0)
}

#[cfg(any(feature = "charts", feature = "components"))]
struct Many {
    sig: (u32, u32),
    springs: Vec<SpringState>,
    last: Vec<f32>,
}

/// Every value of `target` followed at once, for a chart's data. Values that appear start at
/// their target; the rest ease with the component's response. Requests frames until settled.
#[cfg(any(feature = "charts", feature = "components"))]
pub(crate) fn follow_all(w: &mut Window, cx: &mut App, f: &Follow, target: &[f32]) -> Vec<f32> {
    if !instruments_animated() || !f.animated {
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
                dodai_motion::Personality::Snappy => 0.0,
                dodai_motion::Personality::Smooth => 0.05,
                dodai_motion::Personality::Playful => 0.15,
            };
            (
                theme_of(cx).chart_style().response,
                m.duration.as_secs_f32() * 0.6,
                bounce,
            )
        }
    };
    // Data does not overshoot: bars and areas keep a gentle settle at most.
    let spring = Spring::from_response(secs / animation_speed(), bounce.min(0.15));
    let sig = (
        (secs * 1000.0 / animation_speed()) as u32,
        (bounce.min(0.15) * 1000.0) as u32,
    );
    let key = f.element_id(w);
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

/// A chart's series with their values followed. Live series take their window's newest samples
/// and are not eased: they already move continuously.
#[cfg(feature = "charts")]
pub(crate) fn follow_series(
    w: &mut Window,
    cx: &mut App,
    f: &Follow,
    series: &[crate::charts::Series],
) -> Vec<crate::charts::Series> {
    let series: Vec<crate::charts::Series> = series
        .iter()
        .map(|s| {
            let mut s = s.clone();
            if let Some(l) = &s.live {
                s.values = l.values();
            }
            s
        })
        .collect();
    let flat: Vec<f32> = series
        .iter()
        .filter(|s| s.live.is_none())
        .flat_map(|s| s.values.iter().copied())
        .collect();
    let mut shown = follow_all(w, cx, f, &flat).into_iter();
    series
        .into_iter()
        .map(|mut s| {
            if s.live.is_none() {
                for v in &mut s.values {
                    *v = shown.next().unwrap_or(*v);
                }
            }
            s
        })
        .collect()
}

/// The `f32` of each `(label, value)` pair followed, in place.
#[cfg(feature = "charts")]
pub(crate) fn follow_pairs<T>(w: &mut Window, cx: &mut App, f: &Follow, items: &mut [(T, f32)]) {
    let flat: Vec<f32> = items.iter().map(|p| p.1).collect();
    for (p, v) in items.iter_mut().zip(follow_all(w, cx, f, &flat)) {
        p.1 = v;
    }
}

/// The two numbers of each `(label, a, b)` row followed, in place.
#[cfg(feature = "charts")]
pub(crate) fn follow_triples<T>(
    w: &mut Window,
    cx: &mut App,
    f: &Follow,
    items: &mut [(T, f32, f32)],
) {
    let flat: Vec<f32> = items.iter().flat_map(|p| [p.1, p.2]).collect();
    let mut shown = follow_all(w, cx, f, &flat).into_iter();
    for p in items.iter_mut() {
        p.1 = shown.next().unwrap_or(p.1);
        p.2 = shown.next().unwrap_or(p.2);
    }
}

/// A grid of values followed, in place.
#[cfg(feature = "charts")]
pub(crate) fn follow_rows(w: &mut Window, cx: &mut App, f: &Follow, rows: &mut [Vec<f32>]) {
    let flat: Vec<f32> = rows.iter().flatten().copied().collect();
    let mut shown = follow_all(w, cx, f, &flat).into_iter();
    for v in rows.iter_mut().flatten() {
        *v = shown.next().unwrap_or(*v);
    }
}
