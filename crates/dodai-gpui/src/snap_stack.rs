//! [`SnapStack`]: items stacked vertically, one shown at a time; scrolling snaps the next one
//! into place, like the roller on a steering wheel turning the pages of a dash display.
use crate::motion::{Glide, Motion, MotionExt};
use crate::{hsla, theme_of};
use dodai_core::motion::overscroll;
use gpui::{
    div, px, AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement,
    ScrollDelta, ScrollWheelEvent, Styled, TouchPhase, Window,
};

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

/// Where the stack rests and the swipe in progress.
struct Roller {
    glide: Glide,
    /// Pixels swiped so far (positive is onwards), while two fingers are down.
    swipe: Option<f32>,
    /// Fingers just landed: the first move says whether they turn the stack or scroll on.
    armed: bool,
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
        let select = move |i: usize, w: &mut Window, cx: &mut App| {
            crate::own::choose(&state, cx, i);
            if let Some(f) = &inner {
                f(i, w, cx);
            }
        };
        let t = theme_of(cx);
        let h = self.height.max(1.0);
        let m = crate::motion::pace(self.motion.unwrap_or_else(|| Motion::of(cx)));
        let roller = window.use_keyed_state((self.id.clone(), "roller"), cx, |_, _| Roller {
            glide: Glide::new(at as f32, &m),
            swipe: None,
            armed: false,
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
        let wheel = roller.clone();
        let on_wheel = move |e: &ScrollWheelEvent, w: &mut Window, cx: &mut App| {
            match e.delta {
                // A notch turns one page; past either end it scrolls on.
                ScrollDelta::Lines(d) => {
                    let onwards = d.y < 0.0;
                    if d.y == 0.0 || !can_turn(at, last, onwards) {
                        return;
                    }
                    select(if onwards { at + 1 } else { at - 1 }, w, cx);
                }
                ScrollDelta::Pixels(d) => {
                    let (dx, dy) = (f32::from(d.x), f32::from(d.y));
                    let lifted = wheel.update(cx, |r, _| match (e.touch_phase, r.swipe) {
                        (TouchPhase::Started, _) => {
                            r.armed = true;
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
                        (TouchPhase::Ended, Some(s)) => {
                            r.armed = false;
                            r.swipe = None;
                            let to = snap_page(at, s / h, last);
                            let from = (at as f32 + s / h).clamp(-0.5, last as f32 + 0.5);
                            r.glide
                                .throw(from, 0.0, to as f32, &m, crate::motion::now());
                            Some(Some(to))
                        }
                        (TouchPhase::Ended, None) => {
                            r.armed = false;
                            None
                        }
                        _ => None,
                    });
                    let Some(landed) = lifted else {
                        return;
                    };
                    if let Some(to) = landed.filter(|&to| to != at) {
                        select(to, w, cx);
                    }
                }
            }
            cx.stop_propagation();
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
