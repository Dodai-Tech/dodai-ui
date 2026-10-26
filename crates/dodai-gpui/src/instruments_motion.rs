//! How instruments follow a changing value. Each animated instrument owns a [`Follow`]: a stable
//! id for its spring state and an optional [`NeedleResponse`] override. Without an override the
//! design's own feel (`ChartStyle::response`) applies, so a Cyberpunk needle snaps and a
//! Neo-Brutalism one overshoots.

use crate::motion::now;
use crate::theme_of;
use dodai_core::NeedleResponse;
use dodai_motion::{Spring, SpringState};
use gpui::{App, ElementId, SharedString, Window};

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
    response: Option<NeedleResponse>,
}

impl Follow {
    /// An id derived from the call site of the instrument's constructor. Two instruments built
    /// on the same source line (in a loop) share it; give them an explicit `.id(..)`.
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
            response: None,
        }
    }
    /// Replaces the id.
    pub fn with_id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
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
        /// instruments are built from one source line (in a loop).
        pub fn id(mut self, id: impl Into<gpui::ElementId>) -> Self {
            self.follow = self.follow.with_id(id);
            self
        }
        /// How the needle follows a changing value; by default the design's own.
        pub fn response(mut self, r: dodai_core::NeedleResponse) -> Self {
            self.follow = self.follow.with_response(r);
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
    if !instruments_animated() {
        return None;
    }
    let (secs, bounce) = response.spring()?;
    let spring = Spring::from_response(secs / animation_speed(), bounce);
    let st = w.with_id(f.id.clone(), |w| {
        w.use_keyed_state(("needle", u32::from(part)), cx, move |_, _| Needle {
            response,
            spring: SpringState::new(spring, target),
            last: target,
        })
    });
    let t = now();
    let (v, settled) = st.update(cx, |n, _| {
        if n.response != response {
            n.spring = SpringState::new(spring, n.spring.value(t));
            n.response = response;
        }
        if n.last != target {
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

/// Like [`follow`] for a heading in degrees: turns the short way round and returns the angle
/// wrapped to `0..360`.
pub(crate) fn follow_angle(w: &mut Window, cx: &mut App, f: &Follow, part: u8, target: f32) -> f32 {
    let target = target.rem_euclid(360.0);
    state(w, cx, f, part, target, true)
        .unwrap_or(target)
        .rem_euclid(360.0)
}
