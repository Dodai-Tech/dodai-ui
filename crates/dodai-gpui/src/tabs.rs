//! [`Tabs`]: a row of selectable tabs driven by `theme.tab` tokens.

use crate::interact::{track, use_interact};
use crate::paint::{bottom_band_canvas, bottom_band_slide, shaped_id, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    div, px, App, ElementId, Hsla, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Resolved tab paint: shape style, text colour, optional indicator colour.
pub fn tab_style(theme: &Theme, active: bool, hovered: bool) -> (ShapeStyle, Hsla, Option<Hsla>) {
    let k = theme.tab(active, hovered);
    (
        ShapeStyle::from_tokens(
            theme,
            k.background,
            k.outline.then_some(theme.palette.border_strong),
            active,
            k.outline,
        ),
        hsla(k.foreground),
        k.indicator.map(hsla),
    )
}

type SelectHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Tab strip. Controlled: pass `selected` and update it in `on_select`.
#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    labels: Vec<SharedString>,
    active: usize,
    on_select: Option<SelectHandler>,
    corners: Option<dodai_core::Corners>,
    icons: Vec<crate::icons::IconName>,
    secondary: bool,
    scrollable: bool,
}

crate::corners_setter!(Tabs);

impl Tabs {
    /// Create a new [`Tabs`]. The active indicator slides from tab to tab.
    #[track_caller]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            labels: Vec::new(),
            active: 0,
            on_select: None,
            icons: Vec::new(),
            secondary: false,
            scrollable: false,
        }
    }
    /// Scrollable tabs: keep every tab at its natural width and scroll the strip sideways when
    /// it overflows (instead of squeezing the labels).
    pub fn scrollable(mut self, on: bool) -> Self {
        self.scrollable = on;
        self
    }
    /// One icon per tab, stacked above the label (icon-only when there are no labels).
    pub fn icons(mut self, icons: impl IntoIterator<Item = crate::icons::IconName>) -> Self {
        self.icons = icons.into_iter().collect();
        self
    }
    /// Secondary tabs: an on-surface label and a full-width underline, for sub-navigation.
    pub fn secondary(mut self, on: bool) -> Self {
        self.secondary = on;
        self
    }
    /// Tab labels in order.
    pub fn labels<S: Into<SharedString>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }
    /// Index of the active tab.
    pub fn selected(mut self, i: usize) -> Self {
        self.active = i;
        self
    }
    crate::callback!(
        /// Called with the clicked tab index.
        on_select, usize
    );
}

impl Tabs {
    /// Tab `i`: styled for its active / hover state, with its icon, label and indicator.
    fn tab(
        &self,
        t: &dodai_core::Theme,
        i: usize,
        pan: Option<Rc<Cell<Pan>>>,
        slide: Option<&Rc<Slide>>,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        let label = self.labels.get(i);
        let id: ElementId = (self.id.clone(), i.to_string()).into();
        let st = use_interact(&id, window, cx);
        let hovered = st.read(cx).hovered;
        let active = i == self.active;
        let p = crate::motion::transition(window, cx, id.clone(), active);
        let (mut style, fg, indicator) = tab_style(t, active, hovered);
        let (on_s, on_fg, _) = tab_style(t, true, hovered);
        let (off_s, off_fg, _) = tab_style(t, false, hovered);
        style.fill = if slide.is_some() && t.tab(true, false).background.is_some() {
            // The active pill is one shared element that slides between tabs.
            if active {
                None
            } else {
                off_s.fill
            }
        } else {
            crate::motion::mix_opt(off_s.fill, on_s.fill, p)
        };
        let fg = if p > 0.0 && p < 1.0 {
            crate::motion::mix(off_fg, on_fg, p)
        } else {
            fg
        };
        let indicator = if slide.is_some() {
            None
        } else {
            indicator
                .or_else(|| tab_style(t, true, hovered).2.filter(|_| p > 0.01))
                .map(|c| c.opacity(p))
        };
        let fg = if self.secondary && active && t.tab(true, false).background.is_none() {
            hsla(t.palette.roles.on_surface)
        } else {
            fg
        };
        let stacked = label.is_some() && self.icons.get(i).is_some();
        let height = if stacked && !self.secondary {
            t.scale(dodai_core::Scale::Wide)
        } else if t.look().segmented_tabs {
            t.shape.control_height - 6.0
        } else {
            t.shape.control_height
        };
        let shape = if t.look().segmented_tabs {
            crate::paint::scaled_shape(&t.shape, t.shape.control_height)
        } else {
            t.shape.clone()
        };
        let mut tab = track(shaped_id(id, &shape, style), &st)
            .min_h(px(height))
            .px(px(t.spacing.md))
            .flex()
            .items_center()
            .cursor_pointer()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(fg);
        if t.look().tonal {
            tab = crate::text::typeset(tab, t, dodai_core::TypeRole::TitleSmall);
        }
        if stacked && !self.secondary {
            tab = tab.flex_col().justify_center().gap(px(2.0));
        } else {
            tab = tab.gap(px(t.spacing.sm));
        }
        if let Some(icon) = self.icons.get(i) {
            tab = tab.child(
                crate::icons::Icon::new(*icon)
                    .size(t.sizes().icon_sm)
                    .color(fg),
            );
        }
        if let Some(label) = label {
            tab = tab.child(format_label(&t.typography, label));
        }
        if let Some(c) = indicator {
            tab = tab.child(bottom_band_canvas(&t.shape, t.shape.border_width, c));
        }
        if let Some(s) = slide {
            // Report where this tab sits so the shared indicator can slide to it.
            let s = s.clone();
            tab = tab.child(
                gpui::canvas(
                    move |b, _, _| {
                        let mut r = s.rects.borrow_mut();
                        if r.len() <= i {
                            r.resize(i + 1, (0.0, 0.0));
                        }
                        r[i] = (f32::from(b.origin.x), f32::from(b.size.width));
                    },
                    |_, _, _, _| (),
                )
                .absolute()
                .size_full(),
            );
        }
        if let Some(f) = self.on_select.clone() {
            tab = tab.on_click(move |_, w, cx| {
                // A drag that ends over a tab is a pan, not a click.
                if pan.as_ref().is_some_and(|p| p.get().moved) {
                    return;
                }
                f(i, w, cx)
            });
        }
        if self.scrollable {
            tab = tab.flex_none().whitespace_nowrap();
        }
        tab
    }

    /// The scrolling strip around `row`: keeps the selected tab in view and pans with the wheel
    /// and the pointer.
    fn scroll_strip(
        &self,
        row: gpui::Div,
        state: &gpui::Entity<ScrollState>,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::AnyElement {
        // Keep the selected tab in view, but only when the selection changes so the user
        // can still scroll the strip freely. gpui only knows the strip's size after its
        // first layout, so the request is repeated for a few frames after a change.
        let (handle, pan) = {
            let s = state.read(cx);
            (s.handle.clone(), s.pan.clone())
        };
        let (seen, tries) = (state.read(cx).seen, state.read(cx).tries);
        if seen != self.active || tries < 3 {
            handle.scroll_to_item(self.active);
            let fresh = seen != self.active;
            state.update(cx, |s, _| {
                s.seen = self.active;
                s.tries = if fresh { 1 } else { s.tries + 1 };
            });
            crate::motion::next_frame(window);
        }
        // A mouse wheel only scrolls vertically; over an overflowing strip, turn it into a
        // sideways scroll (trackpads already send horizontal deltas, which gpui handles).
        let (wheel, down, drag, up) = (
            handle.clone(),
            (handle.clone(), pan.clone()),
            (handle.clone(), pan.clone()),
            pan,
        );
        let el = gpui::StatefulInteractiveElement::track_scroll(
            gpui::InteractiveElement::id(row.w_full(), (self.id.clone(), "scroll")),
            &handle,
        )
        .overflow_x_scroll();
        let el = gpui::InteractiveElement::on_scroll_wheel(el, move |ev, window, cx| {
            let d = ev.delta.pixel_delta(window.line_height());
            let max = wheel.max_offset().width;
            if max > px(0.0) && d.y.abs() > d.x.abs() {
                let mut o = wheel.offset();
                o.x = (o.x - d.y).clamp(-max, px(0.0));
                wheel.set_offset(o);
                window.refresh();
                cx.stop_propagation();
            }
        });
        let el = gpui::InteractiveElement::on_mouse_down(
            el,
            gpui::MouseButton::Left,
            move |ev, _, _| {
                down.1.set(Pan {
                    from: Some((f32::from(ev.position.x), f32::from(down.0.offset().x))),
                    moved: false,
                });
            },
        );
        let el = gpui::InteractiveElement::on_mouse_move(el, move |ev, window, _| {
            let mut p = drag.1.get();
            let Some((x0, o0)) = p.from else {
                return;
            };
            if ev.pressed_button != Some(gpui::MouseButton::Left) {
                drag.1.set(Pan::default());
                return;
            }
            let dx = f32::from(ev.position.x) - x0;
            if dx.abs() > 4.0 {
                p.moved = true;
            }
            if p.moved {
                let max = f32::from(drag.0.max_offset().width);
                let mut o = drag.0.offset();
                o.x = px((o0 + dx).clamp(-max, 0.0));
                drag.0.set_offset(o);
                window.refresh();
            }
            drag.1.set(p);
        });
        gpui::InteractiveElement::on_mouse_up(el, gpui::MouseButton::Left, move |_, _, _| {
            let p = up.get();
            // Keep `moved` through the click that follows the release, clear it after.
            up.set(Pan {
                from: None,
                moved: p.moved,
            });
        })
        .into_any_element()
    }

    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let strip = t.look().segmented_tabs;
        let mut row = div().relative().flex().gap(px(t.spacing.xs));
        // One indicator slides between tabs when the design underlines the active tab.
        let tokens = t.tab(true, false);
        let pill = tokens.background.is_some() && !tokens.outline;
        let slide = (pill || (!tokens.outline && tokens.indicator.is_some())).then(|| {
            window
                .use_keyed_state((self.id.clone(), "slide"), cx, |_, _| {
                    SlideState(Rc::default())
                })
                .read(cx)
                .0
                .clone()
        });
        // Scrollable strips pan with the pointer too: a swipe or drag moves them sideways.
        let scroll = self.scrollable.then(|| {
            window.use_keyed_state((self.id.clone(), "scroll"), cx, |_, _| ScrollState {
                handle: gpui::ScrollHandle::new(),
                seen: usize::MAX,
                tries: 0,
                pan: Rc::new(Cell::new(Pan::default())),
            })
        });
        if let (true, Some(s)) = (pill, &slide) {
            // The filled tab glides to the active one (measured last frame), at the design's pace.
            let at = s.rects.borrow().get(self.active).copied();
            let first = s.rects.borrow().first().map_or(0.0, |r| r.0);
            if let Some((x, w)) = at.filter(|r| r.1 > 0.0) {
                let (style, _, _) = tab_style(&t, true, false);
                let shape = if strip {
                    crate::paint::scaled_shape(&t.shape, t.shape.control_height)
                } else {
                    t.shape.clone()
                };
                if t.look().glitch_tabs {
                    let old = s.rects.borrow().get(self.last_active(window, cx)).copied();
                    row = row.child(glitch_pill(
                        &self,
                        &t,
                        (x - first, w),
                        old.map(|o| (o.0 - first, o.1)),
                        style,
                        shape,
                        window,
                        cx,
                    ));
                } else {
                    let (x, w) = stretch(self.id.clone(), (x - first, w), window, cx);
                    row = row.child(
                        shaped_id((self.id.clone(), "pill"), &shape, style)
                            .absolute()
                            .top_0()
                            .left(px(x))
                            .w(px(w))
                            .h_full(),
                    );
                }
            } else {
                crate::motion::next_frame(window);
            }
        }
        let count = self.labels.len().max(self.icons.len());
        for i in 0..count {
            let pan = scroll.as_ref().map(|s| s.read(cx).pan.clone());
            row = row.child(self.tab(&t, i, pan, slide.as_ref(), window, cx));
        }
        if let (false, Some(s)) = (pill, &slide) {
            let at = s.rects.borrow().get(self.active).copied();
            let first = s.rects.borrow().first().map_or(0.0, |r| r.0);
            match at {
                Some((x, w)) if w > 0.0 => {
                    let v = [
                        crate::motion::smooth(
                            window,
                            cx,
                            (self.id.clone(), "bar-x").into(),
                            x - first,
                        ),
                        crate::motion::smooth(window, cx, (self.id.clone(), "bar-w").into(), w),
                    ];
                    let c = tab_style(&t, true, false).2.unwrap_or_default();
                    row = row.child(bottom_band_slide(
                        &t.shape,
                        t.shape.border_width,
                        c,
                        v[0],
                        v[1],
                    ));
                }
                // Tabs have not been measured yet: paint once more after layout.
                _ => crate::motion::next_frame(window),
            }
        }
        let inner = match scroll {
            Some(state) => self.scroll_strip(row, &state, window, cx),
            None => row.into_any_element(),
        };
        if !strip {
            return inner;
        }
        // The same bordered capsule as a segmented control; a row parent keeps it as wide as its
        // tabs, and a scrolling strip pans inside it.
        let p = &t.palette;
        let style =
            ShapeStyle::from_tokens(&t, Some(p.background), Some(p.border_strong), false, false);
        let capsule = crate::paint::shaped(&t.shape, style)
            .flex()
            .min_w_0()
            .max_w_full()
            .p(px(2.0))
            .child(inner);
        div().flex().child(capsule).into_any_element()
    }
}

/// Where each tab sat in the last frame (window x and width), shared with the tabs' measuring
/// canvases.
#[derive(Default)]
struct Slide {
    rects: RefCell<Vec<(f32, f32)>>,
}

struct SlideState(Rc<Slide>);

/// Pointer pan state of a scrollable strip.
#[derive(Clone, Copy, Default)]
struct Pan {
    /// Pointer x and scroll offset when the button went down.
    from: Option<(f32, f32)>,
    /// The pointer travelled far enough that this press is a drag, not a click.
    moved: bool,
}

struct ScrollState {
    handle: gpui::ScrollHandle,
    seen: usize,
    tries: u8,
    pan: Rc<Cell<Pan>>,
}

crate::entry_render!(Tabs);

/// The sliding pill's edges `(x, w)`, eased towards `(x, w)` of the active tab. The edge in the
/// direction of travel leads and the other trails, so the pill stretches across the gap and
/// then catches up instead of sliding as a rigid block. Pace and easing are the design's own.
fn stretch(id: ElementId, (x, w): (f32, f32), window: &mut Window, cx: &mut App) -> (f32, f32) {
    let m = crate::motion::current(cx);
    let last = window.use_keyed_state((id.clone(), "pill-dir"), cx, |_, _| (x + w, 1.0f32));
    let (prev_right, mut dir) = *last.read(cx);
    if (x + w - prev_right).abs() > 0.5 {
        dir = (x + w - prev_right).signum();
        last.update(cx, |l, _| *l = (x + w, dir));
    }
    let scaled = |k: f32| dodai_core::Motion {
        duration: m.duration.mul_f32(k),
        ..m
    };
    let (lead, trail) = (scaled(1.4), scaled(2.2));
    let (lm, rm) = if dir > 0.0 {
        (trail, lead)
    } else {
        (lead, trail)
    };
    let left = crate::motion::smooth_with(window, cx, (id.clone(), "pill-l").into(), x, lm);
    let right = crate::motion::smooth_with(window, cx, (id, "pill-r").into(), x + w, rm);
    (left, (right - left).max(0.0))
}

/// When the active tab last changed: the tab it left and the time of the switch.
#[derive(Clone, Copy)]
struct Switch {
    active: usize,
    from: usize,
    at: f32,
}

impl Tabs {
    /// Notes a change of the active tab and returns the tab it came from.
    fn last_active(&self, window: &mut Window, cx: &mut App) -> usize {
        let st = window.use_keyed_state((self.id.clone(), "switch"), cx, |_, _| Switch {
            active: self.active,
            from: self.active,
            at: -10.0,
        });
        let cur = *st.read(cx);
        if cur.active != self.active {
            st.update(cx, |s, _| {
                *s = Switch {
                    active: self.active,
                    from: cur.active,
                    at: crate::motion::now(),
                }
            });
        }
        st.read(cx).from
    }
}

/// Deterministic jitter in `-1.0..=1.0` for slice `i` at glitch step `step`.
fn jitter(i: u32, step: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9E37_79B1) ^ step.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h % 2001) as f32 / 1000.0 - 1.0
}

/// How far the glitch has run, `0.0..1.0`, or `None` once settled.
fn glitch_progress(since: f32, now: f32) -> Option<f32> {
    let k = (now - since) / GLITCH_SECS;
    (0.0..1.0).contains(&k).then_some(k)
}

const GLITCH_SECS: f32 = 0.32;

/// The active fill of a glitching design: the new tab appears in horizontal slices that jitter
/// sideways and settle, with colour-split ghosts and a flickering trace of the tab it left.
#[allow(clippy::too_many_arguments)]
fn glitch_pill(
    tabs: &Tabs,
    t: &Theme,
    (x, w): (f32, f32),
    old: Option<(f32, f32)>,
    style: ShapeStyle,
    shape: dodai_core::Shape,
    window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    let id = tabs.id.clone();
    let k = glitch_progress(
        window
            .use_keyed_state((tabs.id.clone(), "switch"), cx, |_, _| Switch {
                active: tabs.active,
                from: tabs.active,
                at: -10.0,
            })
            .read(cx)
            .at,
        crate::motion::now(),
    );
    let Some(k) = k else {
        return shaped_id((id, "pill"), &shape, style)
            .absolute()
            .top_0()
            .left(px(x))
            .w(px(w))
            .h_full()
            .into_any_element();
    };
    crate::motion::next_frame(window);
    let step = (k * 18.0) as u32;
    let amp = 14.0 * (1.0 - k);
    let fill = style.fill.unwrap_or_else(|| hsla(t.palette.primary));
    // The whole contour (cut corners included) in `colour`, `dx` px sideways.
    let contour = |colour, dx: f32, x: f32, w: f32, tag: &'static str| {
        let mut st = style;
        st.fill = Some(colour);
        shaped_id((id.clone(), tag), &shape, st)
            .absolute()
            .top_0()
            .left(px(x + dx))
            .w(px(w))
            .h_full()
    };
    let mut layer = div().absolute().top_0().left_0().size_full();
    // Colour-split ghosts behind the slices.
    for (n, (c, dx, tag)) in [
        (t.palette.info, -amp * 0.5, "ghost-a"),
        (t.palette.danger, amp * 0.5, "ghost-b"),
    ]
    .into_iter()
    .enumerate()
    {
        let dim = if (step + n as u32) % 3 == 0 { 0.4 } else { 1.0 };
        layer = layer.child(contour(
            hsla(c).opacity(0.45 * (1.0 - k) * dim),
            dx,
            x,
            w,
            tag,
        ));
    }
    // The tab it left, flickering out.
    if let (Some((ox, ow)), true) = (old, step % 2 == 0 && k < 0.7) {
        layer = layer.child(contour(
            fill.opacity(0.35 * (1.0 - k)),
            0.0,
            ox,
            ow,
            "trace",
        ));
    }
    // The new tab in three slices that jitter sideways and settle. Each slice clips the full
    // contour, so the cut corners stay cut.
    for i in 0..3u32 {
        let dx = jitter(i, step) * amp;
        if k < 0.15 && i == 1 {
            continue;
        }
        let tag: &'static str = ["slice-0", "slice-1", "slice-2"][i as usize];
        let mut st = style;
        st.fill = Some(fill);
        layer = layer.child(
            div()
                .absolute()
                .top(gpui::relative(i as f32 / 3.0))
                .h(gpui::relative(1.0 / 3.0))
                .left(px(x + dx))
                .w(px(w))
                .overflow_hidden()
                .child(
                    shaped_id((id.clone(), tag), &shape, st)
                        .absolute()
                        .top(gpui::relative(-(i as f32)))
                        .left_0()
                        .w_full()
                        .h(gpui::relative(3.0)),
                ),
        );
    }
    layer.into_any_element()
}

#[cfg(test)]
mod tests {
    #[test]
    fn glitch_runs_then_settles() {
        assert_eq!(glitch_progress(1.0, 1.0), Some(0.0));
        assert!(glitch_progress(1.0, 1.1).is_some());
        assert_eq!(glitch_progress(1.0, 1.0 + GLITCH_SECS + 0.01), None);
        assert_eq!(glitch_progress(-10.0, 0.5), None);
        for i in 0..3 {
            for step in 0..20 {
                assert!(jitter(i, step).abs() <= 1.0);
            }
        }
        // The slices do not all move together.
        assert!(jitter(0, 3) != jitter(1, 3) || jitter(1, 3) != jitter(2, 3));
    }

    use super::*;
    use crate::testing::light_rounded;

    #[test]
    fn active_tab_uses_indicator_tokens() {
        let t = light_rounded();
        let (s, fg, ind) = tab_style(&t, true, false);
        // M3 primary tab: no fill, `primary` label and indicator.
        assert!(s.fill.is_none());
        assert_eq!(fg, hsla(t.palette.primary));
        assert_eq!(ind, Some(hsla(t.palette.primary)));
        let (s, fg, ind) = tab_style(&t, false, false);
        assert!(s.fill.is_none() && ind.is_none());
        assert_eq!(fg, hsla(t.palette.roles.on_surface_variant));
        let (s, _, _) = tab_style(&t, false, true);
        assert_eq!(
            s.fill,
            Some(hsla(
                t.palette
                    .roles
                    .on_surface
                    .with_alpha(dodai_core::StateLayer::HOVER)
            ))
        );
    }
}
