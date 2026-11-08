//! [`SnapStack`]: items stacked vertically, one shown at a time; scrolling snaps the next one
//! into place, like the roller on a steering wheel turning the pages of a dash display.
use crate::motion::{Glide, Motion, MotionExt};
use crate::{hsla, theme_of};
use dodai_core::motion::{overscroll, Easing, Travel};
use gpui::{
    div, px, AnyElement, App, DispatchPhase, ElementId, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, ScrollDelta,
    ScrollWheelEvent, Styled, TouchPhase, Window,
};
use std::time::Duration;

/// Share of a page a swipe must travel to turn it.
pub const TURN_AT: f32 = 0.15;

/// The page a swipe lands on: from page `from`, after moving `pages` (positive is onwards),
/// on a stack whose last page is `last`. A swipe past [`TURN_AT`] of a page turns at least one;
/// a long one turns as many as it covered. Pure.
pub fn snap_page(from: usize, pages: f32, last: usize) -> usize {
    let turned = if pages.abs() < TURN_AT {
        0.0
    } else {
        pages.signum() * (pages.abs() - TURN_AT).ceil().max(1.0)
    };
    (from as f32 + turned).clamp(0.0, last as f32) as usize
}

/// Whether a scroll `onwards` (else back) from page `at` of `0..=last` has a page to go to;
/// when it has not, the scroll passes to whatever scrolls around the stack. Pure.
pub fn can_turn(at: usize, last: usize, onwards: bool) -> bool {
    if onwards {
        at < last
    } else {
        at > 0
    }
}

/// Called with a page index.
type IndexFn = std::rc::Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The shortest time a page takes to settle.
pub const PAGE_TURN: Duration = Duration::from_millis(520);

/// Seconds after a gesture in which stray inertia is not read as a new notch.
const QUIET: f32 = 0.6;

/// The design's motion slowed to a page turn: a whole page is a long way to travel, so it
/// takes at least [`PAGE_TURN`] and decelerates smoothly into place. An instant motion (animation
/// off) stays instant. Pure.
pub fn page_motion(m: Motion) -> Motion {
    if m.duration.is_zero() {
        return m;
    }
    Motion {
        duration: m.duration.max(PAGE_TURN),
        easing: Easing::CubicBezier(0.22, 1.0, 0.36, 1.0),
        travel: Travel::Eased,
        ..m
    }
}

/// Lets go of a swipe of `s` pixels: the page it lands on, with the glide thrown from where the
/// fingers left it.
fn settle(r: &mut Roller, at: usize, s: f32, h: f32, last: usize, m: &Motion) -> usize {
    r.armed = false;
    r.grab = None;
    r.swipe = None;
    r.quiet_until = crate::motion::now() + QUIET;
    let to = snap_page(at, s / h, last);
    let from = (at as f32 + s / h).clamp(-0.5, last as f32 + 0.5);
    r.glide.throw(from, 0.0, to as f32, m, crate::motion::now());
    to
}

/// Where the stack rests and the swipe in progress.
struct Roller {
    glide: Glide,
    /// Pixels swiped so far (positive is onwards), while two fingers are down.
    swipe: Option<f32>,
    /// Fingers just landed: the first move says whether they turn the stack or scroll on.
    armed: bool,
    /// Last pointer height while the button is down (a drag, as on a phone).
    grab: Option<f32>,
    /// Until this time (see [`crate::motion::now`]) a touch-less pixel scroll is not a new notch:
    /// it is the tail of the last gesture. Held open while fingers are down.
    quiet_until: f32,
}

/// Items stacked vertically, one shown at a time. The wheel turns one page a notch; two fingers
/// drag the stack and snap the nearest page into place when lifted. At either end the scroll
/// passes on, so a page around the stack still scrolls. Small dots just past the trailing
/// edge show where.
#[derive(IntoElement)]
pub struct SnapStack {
    id: ElementId,
    items: Vec<AnyElement>,
    active: usize,
    height: f32,
    dots: bool,
    arrows: bool,
    motion: Option<Motion>,
    on_select: Option<IndexFn>,
}

impl SnapStack {
    /// An empty stack, 200 px tall.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            active: 0,
            height: 200.0,
            dots: true,
            arrows: false,
            motion: None,
            on_select: None,
        }
    }
    /// Add an item.
    pub fn item(mut self, e: impl IntoElement) -> Self {
        self.items.push(e.into_any_element());
        self
    }
    /// Add items.
    pub fn items(mut self, es: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.items
            .extend(es.into_iter().map(IntoElement::into_any_element));
        self
    }
    /// The page shown first, or the page shown when [`SnapStack::on_select`] keeps it.
    pub fn selected(mut self, i: usize) -> Self {
        self.active = i;
        self
    }
    /// The height of one page, in px (default 200).
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// Show the page dots (default on).
    pub fn dots(mut self, on: bool) -> Self {
        self.dots = on;
        self
    }
    /// Show chevron buttons at the top and bottom edges that turn a page (default off). Each
    /// appears only while there is a page in its direction.
    pub fn arrows(mut self, on: bool) -> Self {
        self.arrows = on;
        self
    }
    /// Pace and easing of the snap. Defaults to the design's motion.
    pub fn motion(mut self, m: Motion) -> Self {
        self.motion = Some(m);
        self
    }
    crate::callback!(
        /// Called when a page snaps into place, with its index.
        on_select, usize
    );
}

impl SnapStack {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let own = crate::own::Own::new(window, cx, self.id.clone(), self.active);
        let last = self.items.len().saturating_sub(1);
        let at = own.get().min(last);
        let (state, inner) = (own.setter(), self.on_select.take());
        let select = std::rc::Rc::new(move |i: usize, w: &mut Window, cx: &mut App| {
            crate::own::choose(&state, cx, i);
            if let Some(f) = &inner {
                f(i, w, cx);
            }
        });
        let t = theme_of(cx);
        let h = self.height.max(1.0);
        let m = crate::motion::pace(match self.motion {
            Some(m) => m,
            None => page_motion(Motion::of(cx)),
        });
        let roller = window.use_keyed_state((self.id.clone(), "roller"), cx, |_, _| Roller {
            glide: Glide::new(at as f32, &m),
            swipe: None,
            armed: false,
            grab: None,
            quiet_until: 0.0,
        });
        let now = crate::motion::now();
        // Under a swipe the stack follows the fingers, resisting past either end.
        let slide = match roller.read(cx).swipe {
            Some(px_) => {
                let raw = at as f32 + px_ / h;
                let band = |past: f32| overscroll(past * h, h * 0.5) / h;
                if raw < 0.0 {
                    -band(-raw)
                } else if raw > last as f32 {
                    last as f32 + band(raw - last as f32)
                } else {
                    raw
                }
            }
            None => {
                roller.update(cx, |r, _| r.glide.to(at as f32, &m, now));
                let g = &roller.read(cx).glide;
                if !g.settled(now) {
                    crate::motion::next_frame(window);
                }
                g.value(now)
            }
        };
        let (wheel, wheel_select) = (roller.clone(), select.clone());
        let on_wheel = move |e: &ScrollWheelEvent, w: &mut Window, cx: &mut App| {
            match e.delta {
                // A notch turns one page; past either end it scrolls on.
                ScrollDelta::Lines(d) => {
                    let onwards = d.y < 0.0;
                    if d.y == 0.0 || !can_turn(at, last, onwards) {
                        return;
                    }
                    wheel_select(if onwards { at + 1 } else { at - 1 }, w, cx);
                }
                ScrollDelta::Pixels(d) => {
                    let (dx, dy) = (f32::from(d.x), f32::from(d.y));
                    let lifted = wheel.update(cx, |r, _| match (e.touch_phase, r.swipe) {
                        (TouchPhase::Started, _) => {
                            r.armed = true;
                            r.quiet_until = f32::MAX;
                            None
                        }
                        (TouchPhase::Moved, Some(s)) => {
                            r.swipe = Some(s - dy);
                            Some(None)
                        }
                        (TouchPhase::Moved, None) if r.armed && dy != 0.0 => {
                            r.armed = false;
                            let ours = dy.abs() > dx.abs() && can_turn(at, last, dy < 0.0);
                            ours.then(|| {
                                r.swipe = Some(-dy);
                                None
                            })
                        }
                        // A pixel wheel without touch phases (a smooth mouse wheel): one
                        // notch a flick, ignoring the rest of its inertia.
                        (TouchPhase::Moved, None)
                            if dy != 0.0 && crate::motion::now() > r.quiet_until =>
                        {
                            let onwards = dy < 0.0;
                            r.quiet_until = crate::motion::now() + QUIET;
                            can_turn(at, last, onwards)
                                .then(|| Some(if onwards { at + 1 } else { at - 1 }))
                        }
                        (TouchPhase::Ended, Some(s)) => Some(Some(settle(r, at, s, h, last, &m))),
                        (TouchPhase::Ended, None) => {
                            r.armed = false;
                            r.quiet_until = crate::motion::now() + QUIET;
                            None
                        }
                        _ => None,
                    });
                    let Some(landed) = lifted else {
                        return;
                    };
                    if let Some(to) = landed.filter(|&to| to != at) {
                        wheel_select(to, w, cx);
                    }
                }
            }
            cx.stop_propagation();
            w.refresh();
        };
        let (grab, moved, up) = (roller.clone(), roller.clone(), roller.clone());
        let dragging = roller.read(cx).grab.is_some();
        let on_down = move |e: &MouseDownEvent, w: &mut Window, cx: &mut App| {
            if last == 0 {
                return;
            }
            grab.update(cx, |r, _| {
                r.grab = Some(f32::from(e.position.y));
                r.swipe = Some(0.0);
            });
            w.refresh();
        };
        let first = (slide.floor().max(0.0) as usize).min(last);
        let frac = slide - first as f32;
        let shown = std::mem::take(&mut self.items)
            .into_iter()
            .skip(first)
            .take(if frac.abs() > 0.001 { 2 } else { 1 })
            .map(|e| div().h(px(h)).flex_none().overflow_hidden().child(e));
        let dot = (t.sizes().bar_thickness * 0.75).max(2.0);
        let ink = hsla(t.palette.text);
        let dots = (0..=last).map(|i| {
            let on = i == at;
            div()
                .w(px(dot))
                .h(px(if on { dot * 3.0 } else { dot }))
                .rounded(px(dot))
                .bg(crate::paint::scale_alpha(ink, if on { 0.7 } else { 0.25 }))
        });
        let mut root = div()
            .id(self.id.clone())
            .relative()
            .h(px(h))
            .on_scroll_wheel(on_wheel)
            .on_mouse_down(MouseButton::Left, on_down)
            .child(
                div().h(px(h)).overflow_hidden().child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .mt(px(-frac * h))
                        .children(shown),
                ),
            );
        if self.arrows {
            for (onwards, glyph, name) in [(false, "⌃", "up"), (true, "⌄", "down")] {
                if !can_turn(at, last, onwards) {
                    continue;
                }
                let go = select.clone();
                let to = if onwards {
                    at + 1
                } else {
                    at.saturating_sub(1)
                };
                let button = crate::IconButton::new((self.id.clone(), name), glyph)
                    .on_click(move |_, w, cx| go(to, w, cx));
                let edge = div().absolute().left_0().right_0().flex().justify_center();
                root = root.child(if onwards {
                    edge.bottom_0().child(button)
                } else {
                    edge.top_0().child(button)
                });
            }
        }
        if dragging {
            // While the button is down the whole window reports to the stack, so a drag that
            // leaves it keeps going and is let go wherever the button comes up.
            root = root.child(gpui::canvas(
                |_, _, _| (),
                move |_, _, window, _| {
                    window.on_mouse_event(move |e: &MouseMoveEvent, phase, w, cx| {
                        if phase != DispatchPhase::Bubble {
                            return;
                        }
                        let y = f32::from(e.position.y);
                        moved.update(cx, |r, _| {
                            if let (Some(from), Some(s)) = (r.grab, r.swipe) {
                                r.swipe = Some(s + from - y);
                                r.grab = Some(y);
                            }
                        });
                        w.refresh();
                    });
                    window.on_mouse_event(move |_: &MouseUpEvent, phase, w, cx| {
                        if phase != DispatchPhase::Bubble {
                            return;
                        }
                        let to =
                            up.update(cx, |r, _| r.swipe.map(|s| settle(r, at, s, h, last, &m)));
                        if let Some(to) = to.filter(|&to| to != at) {
                            select(to, w, cx);
                        }
                        w.refresh();
                    });
                },
            ));
        }
        if self.dots && last > 0 {
            root = root.child(
                div()
                    .absolute()
                    .right(px(-(dot + t.spacing.xs)))
                    .top_0()
                    .bottom_0()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap(px(t.spacing.xs))
                    .children(dots),
            );
        }
        root
    }
}

crate::entry_render!(SnapStack);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_page_turn_is_slow_and_never_wakes_a_still_design() {
        assert!(page_motion(Motion::default()).duration >= PAGE_TURN);
        assert_eq!(
            page_motion(Motion::default().instant()).duration,
            Duration::ZERO
        );
    }

    #[test]
    fn a_short_swipe_stays_and_a_long_one_turns_as_far_as_it_went() {
        assert_eq!(snap_page(2, 0.1, 5), 2);
        assert_eq!(snap_page(2, -0.1, 5), 2);
        assert_eq!(snap_page(2, 0.2, 5), 3);
        assert_eq!(snap_page(2, -0.2, 5), 1);
        assert_eq!(snap_page(2, 1.6, 5), 4);
        assert_eq!(snap_page(4, 3.0, 5), 5);
        assert_eq!(snap_page(0, -2.0, 5), 0);
    }

    #[test]
    fn the_ends_let_the_scroll_pass_on() {
        assert!(!can_turn(0, 3, false));
        assert!(can_turn(0, 3, true));
        assert!(!can_turn(3, 3, true));
        assert!(!can_turn(0, 0, true));
    }
}
