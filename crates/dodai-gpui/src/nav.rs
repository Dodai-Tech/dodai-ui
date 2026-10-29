//! Navigation: [`Breadcrumb`], [`Pagination`], [`Stepper`], [`Segmented`], [`Accordion`].

use crate::interact::{track, use_interact, IndexHandler};
use crate::logic::{page_window, step_status, PageItem, StepStatus};
use crate::paint::{round_shape, scaled_shape, shaped, shaped_id, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::{ButtonVariant, ControlState, Theme};
use gpui::prelude::FluentBuilder as _;
use gpui::{
    div, px, App, ElementId, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Path trail; the last crumb is the current page. Earlier crumbs are clickable.
#[derive(IntoElement)]
pub struct Breadcrumb {
    id: ElementId,
    items: Vec<SharedString>,
    on_select: Option<IndexHandler>,
}

impl Breadcrumb {
    /// Create a new [`Breadcrumb`].
    pub fn new<S: Into<SharedString>>(
        id: impl Into<ElementId>,
        items: impl IntoIterator<Item = S>,
    ) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().map(Into::into).collect(),
            on_select: None,
        }
    }
    crate::callback!(
        /// Called when an item is chosen, with the index.
        on_select, usize
    );
}

impl Breadcrumb {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let mut row = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.sm))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_sm));
        let last = self.items.len().saturating_sub(1);
        for (i, label) in self.items.iter().enumerate() {
            if i > 0 {
                row = row.child(div().text_color(hsla(t.palette.text_disabled)).child("/"));
            }
            let id: ElementId = (self.id.clone(), i.to_string()).into();
            let st = use_interact(&id, window, cx);
            let hovered = st.read(cx).hovered;
            let current = i == last;
            let color = if current {
                t.palette.text
            } else if hovered {
                t.palette.primary
            } else {
                t.palette.link
            };
            let mut c = div()
                .id(id)
                .text_color(hsla(color))
                .child(format_label(&t.typography, label));
            if !current {
                c = c.cursor_pointer();
                if let Some(f) = self.on_select.clone() {
                    c = c.on_click(move |_, w, cx| f(i, w, cx));
                }
            }
            row = row.child(crate::interact::track(c, &st));
        }
        row
    }
}

crate::entry_render!(Breadcrumb);

/// Square-ish page/step button used by [`Pagination`].
fn page_button(
    t: &Theme,
    id: ElementId,
    label: SharedString,
    variant: ButtonVariant,
    disabled: bool,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Stateful<gpui::Div> {
    let st = use_interact(&id, window, cx);
    let state = st.read(cx).control_state(disabled);
    let (style, fg) = crate::button::button_style(t, variant, state);
    let el = track(shaped_id(id, &t.shape, style), &st)
        .min_h(px(t.shape.control_height))
        .min_w(px(t.shape.control_height))
        .px(px(t.spacing.sm))
        .flex()
        .items_center()
        .justify_center()
        .font_family(first_family(t.typography.body))
        .text_size(px(t.typography.size_md))
        .text_color(fg)
        .child(label);
    if disabled {
        el
    } else {
        el.cursor_pointer()
    }
}

/// Page selector: `‹ 1 … 4 [5] 6 … 20 ›`. Pages are 1-based. Controlled.
#[derive(IntoElement)]
pub struct Pagination {
    id: ElementId,
    page: usize,
    total: usize,
    on_change: Option<IndexHandler>,
}

impl Pagination {
    /// Create a new [`Pagination`].
    pub fn new(id: impl Into<ElementId>, page: usize, total: usize) -> Self {
        Self {
            id: id.into(),
            page,
            total,
            on_change: None,
        }
    }
    crate::callback!(
        /// Called with the requested 1-based page.
        on_change, usize
    );
}

impl Pagination {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let own = crate::own::Own::new(window, cx, self.id.clone(), self.page);
        self.page = own.get();
        let (state, inner) = (own.setter(), self.on_change.take());
        self.on_change = Some(std::rc::Rc::new(move |i, w, cx| {
            crate::own::choose(&state, cx, i);
            if let Some(f) = &inner {
                f(i, w, cx);
            }
        }));
        let t = theme_of(cx);
        let mut row = div().flex().items_center().gap(px(t.spacing.xs));
        let (page, total) = (self.page.clamp(1, self.total.max(1)), self.total);
        let mk = |row: gpui::Div,
                  key: String,
                  label: &str,
                  variant,
                  disabled: bool,
                  target: usize,
                  window: &mut Window,
                  cx: &mut App| {
            let mut b = page_button(
                &t,
                (self.id.clone(), key).into(),
                label.to_string().into(),
                variant,
                disabled,
                window,
                cx,
            );
            if let (false, Some(f)) = (disabled, self.on_change.clone()) {
                b = b.on_click(move |_, w, cx| f(target, w, cx));
            }
            row.child(b)
        };
        row = mk(
            row,
            "prev".into(),
            "‹",
            ButtonVariant::Outline,
            page <= 1,
            page.saturating_sub(1),
            window,
            cx,
        );
        for (i, item) in page_window(page, total, 7).into_iter().enumerate() {
            row = match item {
                PageItem::Page(p) => mk(
                    row,
                    format!("p{p}"),
                    &p.to_string(),
                    if p == page {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Ghost
                    },
                    false,
                    p,
                    window,
                    cx,
                ),
                PageItem::Ellipsis => row.child(
                    div()
                        .id((self.id.clone(), format!("e{i}")))
                        .px(px(t.spacing.xs))
                        .text_color(hsla(t.palette.text_muted))
                        .child("…"),
                ),
            };
        }
        mk(
            row,
            "next".into(),
            "›",
            ButtonVariant::Outline,
            page >= total,
            page + 1,
            window,
            cx,
        )
    }
}

crate::entry_render!(Pagination);

/// Multi-step progress: numbered nodes joined by lines. Done steps show ✓.
#[derive(IntoElement)]
pub struct Stepper {
    follow: crate::instruments_motion::Follow,
    steps: Vec<SharedString>,
    current: usize,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Stepper);

impl Stepper {
    /// Create a new [`Stepper`]; the joining lines fill as the current step advances.
    #[track_caller]
    pub fn new<S: Into<SharedString>>(steps: impl IntoIterator<Item = S>, current: usize) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            corners: None,
            steps: steps.into_iter().map(Into::into).collect(),
            current,
        }
    }
    crate::instruments_motion::follow_api!();
}

impl Stepper {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let p = &t.palette;
        let side = 26.0;
        let node_shape = round_shape(&t.shape, side);
        let mut row = div()
            .flex()
            .items_center()
            .w_full()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_sm));
        let n = self.steps.len();
        let eased =
            crate::instruments_motion::follow(window, cx, &self.follow, 0, self.current as f32);
        for (i, label) in self.steps.iter().enumerate() {
            let status = step_status(i, self.current);
            let k = t.step(status);
            let (fill, stroke, fg, text) = (k.fill, k.stroke, k.mark, k.text);
            let style = ShapeStyle::from_tokens(&t, fill, Some(stroke), k.glow, false);
            row = row.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(t.spacing.sm))
                    // Steps give way when the row is short: the other labels truncate, the
                    // current one and every node stay whole.
                    .min_w(px(side))
                    .when_else(
                        status == StepStatus::Current,
                        |d| d.flex_none(),
                        |d| d.flex_shrink(),
                    )
                    .child(
                        shaped(&node_shape, style)
                            .flex_none()
                            .w(px(side))
                            .h(px(side))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(hsla(fg))
                            .child(if status == StepStatus::Done {
                                "✓".to_string()
                            } else {
                                (i + 1).to_string()
                            }),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .text_color(hsla(text))
                            .child(format_label(&t.typography, label)),
                    ),
            );
            if i + 1 < n {
                row = row.child(
                    div()
                        .flex_1()
                        .min_w(px(t.spacing.md))
                        .mx(px(t.spacing.sm))
                        .h(px(2.0))
                        .bg(hsla(p.border))
                        .child(
                            div()
                                .h_full()
                                .w(gpui::relative((eased - i as f32).clamp(0.0, 1.0)))
                                .bg(hsla(p.primary)),
                        ),
                );
            }
        }
        row
    }
}

crate::entry_render!(Stepper);

/// Exclusive option group in one bordered strip. Controlled.
#[derive(IntoElement)]
pub struct Segmented {
    id: ElementId,
    options: Vec<SharedString>,
    selected: usize,
    disabled: bool,
    on_select: Option<IndexHandler>,
    corners: Option<dodai_core::Corners>,
    animation: Option<crate::motion::SelectionMotion>,
    motion: Option<crate::motion::Motion>,
}

crate::corners_setter!(Segmented);

impl Segmented {
    /// Create a new [`Segmented`].
    pub fn new<S: Into<SharedString>>(
        id: impl Into<ElementId>,
        options: impl IntoIterator<Item = S>,
    ) -> Self {
        Self {
            corners: None,
            id: id.into(),
            options: options.into_iter().map(Into::into).collect(),
            selected: 0,
            disabled: false,
            on_select: None,
            animation: None,
            motion: None,
        }
    }
    /// How the selection moves between segments. `None` (the default) is the design's own
    /// ([`Look::selection`](dodai_core::Look::selection)). Material segmented buttons switch in
    /// place, as their spec says.
    pub fn animation(mut self, a: impl Into<Option<crate::motion::SelectionMotion>>) -> Self {
        self.animation = a.into();
        self
    }
    /// Pace, easing and travel of the selection. Defaults to the design's motion.
    pub fn motion(mut self, m: crate::motion::Motion) -> Self {
        self.motion = Some(m);
        self
    }
    /// The selected item (controlled).
    pub fn selected(mut self, i: usize) -> Self {
        self.selected = i;
        self
    }
    /// Disable the control.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    crate::callback!(
        /// Called when an item is chosen, with the index.
        on_select, usize
    );
}

impl Segmented {
    fn build(mut self, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
        let own = crate::own::Own::new(window, cx, self.id.clone(), self.selected);
        self.selected = own.get();
        let (state, inner) = (own.setter(), self.on_select.take());
        self.on_select = Some(std::rc::Rc::new(move |i, w, cx| {
            crate::own::choose(&state, cx, i);
            if let Some(f) = &inner {
                f(i, w, cx);
            }
        }));
        let t = theme_of(cx);
        if t.look().tonal {
            return self.build_m3(t, window, cx).into_any_element();
        }
        self.build_classic(window, cx).into_any_element()
    }

    /// Material 3 segmented button: a 40 px pill outlined in `outline`, segments split by
    /// 1 px dividers, the selected one filled `secondary-container` with a check mark.
    fn build_m3(self, t: Theme, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let r = t.palette.roles.clone();
        let n = self.options.len();
        let outline = if self.disabled {
            r.on_surface.with_alpha(dodai_core::StateLayer::SELECTED)
        } else {
            r.outline
        };
        let strip = ShapeStyle::from_tokens(&t, None, Some(outline), false, false);
        let strip_shape = t.role_shape(dodai_core::ShapeRole::Button);
        // The selected / hovered fills follow the strip's own end radius.
        let end_r = strip_shape.radius.min(t.sizes().segmented_height / 2.0);
        let mut row = shaped(&strip_shape, strip)
            .flex()
            .h(px(t.sizes().segmented_height))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md));
        for (i, label) in self.options.iter().enumerate() {
            let id: ElementId = (self.id.clone(), i.to_string()).into();
            let st = use_interact(&id, window, cx);
            let hovered = st.read(cx).hovered && !self.disabled;
            let on = i == self.selected;
            let (fill, fg) = if self.disabled {
                (None, r.on_surface.with_alpha(0.38))
            } else if on {
                (Some(r.secondary_container), r.on_secondary_container)
            } else if hovered {
                (
                    Some(r.on_surface.with_alpha(dodai_core::StateLayer::HOVER)),
                    r.on_surface,
                )
            } else {
                (None, r.on_surface)
            };
            let mut seg = track(gpui::InteractiveElement::id(div(), id), &st)
                .h_full()
                .px(px(t.spacing.sm * 1.5))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(t.spacing.sm))
                .text_color(hsla(fg));
            if let Some(f) = fill {
                seg = seg.bg(hsla(f));
            }
            if i == 0 {
                seg = seg.rounded_tl(px(end_r)).rounded_bl(px(end_r));
            }
            if i + 1 == n {
                seg = seg.rounded_tr(px(end_r)).rounded_br(px(end_r));
            }
            if i > 0 {
                seg = seg.border_l_1().border_color(hsla(outline));
            }
            if on {
                seg = seg.child("✓");
            }
            seg = seg.child(format_label(&t.typography, label));
            if !self.disabled {
                seg = seg.cursor_pointer();
                if let Some(f) = self.on_select.clone() {
                    seg = seg.on_click(move |_, w, cx| f(i, w, cx));
                }
            }
            row = row.child(seg);
        }
        // Hug the segments: in a column the strip would stretch to the full width and leave the
        // last segment's fill short of the outline. Inside a row it keeps its content width.
        div().flex().child(row.flex_none())
    }

    fn build_classic(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use crate::motion::{MotionExt, SelectionMotion};
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let p = &t.palette;
        let strip =
            ShapeStyle::from_tokens(&t, Some(p.background), Some(p.border_strong), false, false);
        let inner = scaled_shape(&t.shape, t.shape.control_height);
        let kind = self.animation.unwrap_or(t.look().selection);
        let motion = self.motion.unwrap_or_else(|| crate::motion::Motion::of(cx));
        let (on_fill, on_fg) = if t.look().tonal {
            (p.roles.secondary_container, p.roles.on_secondary_container)
        } else {
            (p.primary, p.on_primary)
        };
        // One fill travels between the segments, unless the design lifts them in place.
        let track = (kind != SelectionMotion::Lift && !self.disabled)
            .then(|| crate::selection::SelectionTrack::new(self.id.clone(), window, cx));
        let mut row = shaped(&t.shape, strip)
            .relative()
            .flex()
            .p(px(2.0))
            .gap(px(2.0))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md));
        if let Some(track) = &track {
            let style = ShapeStyle::from_tokens(&t, Some(on_fill), None, true, false);
            let marker = crate::selection::SelectionMarker::fill(inner.clone(), style);
            row = row.child(track.container()).child(track.indicator(
                self.selected,
                kind,
                motion,
                marker,
                window,
                cx,
            ));
        }
        for (i, label) in self.options.iter().enumerate() {
            let id: ElementId = (self.id.clone(), i.to_string()).into();
            let st = use_interact(&id, window, cx);
            let hovered = st.read(cx).hovered && !self.disabled;
            let on = i == self.selected;
            let k = crate::motion::smooth_with(
                window,
                cx,
                id.clone(),
                if on { 1.0 } else { 0.0 },
                motion,
            );
            let (fill, fg) = if self.disabled {
                (on.then_some(p.surface_raised), hsla(p.text_disabled))
            } else {
                let rest = if hovered && !on {
                    Some(p.surface_raised)
                } else {
                    None
                };
                // With a travelling fill the segment itself only shows hover.
                let fill = match (&track, on) {
                    (None, true) => Some(on_fill),
                    _ => rest,
                };
                let off = if hovered { p.text } else { p.text_muted };
                (fill, crate::motion::mix(hsla(off), hsla(on_fg), k))
            };
            let style = ShapeStyle::from_tokens(&t, fill, None, on && !self.disabled, false);
            let mut seg = crate::interact::track(shaped_id(id, &inner, style), &st)
                .relative()
                .h(px(t.shape.control_height - 6.0))
                .px(px(t.spacing.md))
                .flex()
                .items_center()
                .text_color(fg)
                .child(format_label(&t.typography, label));
            if let Some(track) = &track {
                seg = seg.child(track.item(i));
            }
            if kind == SelectionMotion::Lift {
                seg = seg.top(px(-(t.shape.border_width * 2.0).max(2.0) * k));
            }
            if !self.disabled {
                seg = seg.cursor_pointer();
                if let Some(f) = self.on_select.clone() {
                    seg = seg.on_click(move |_, w, cx| f(i, w, cx));
                }
            }
            row = row.child(seg);
        }
        // Hug the segments, like the Material strip.
        div().flex().child(row.flex_none())
    }
}

crate::entry_render!(Segmented);

struct AccState {
    open: Vec<bool>,
    /// Each body's natural height, measured as it draws.
    heights: Rc<RefCell<Vec<f32>>>,
}

/// Collapsible sections `(title, body)`. Each toggles independently; sections
/// listed in `expanded` start expanded.
#[derive(IntoElement)]
pub struct Accordion {
    id: ElementId,
    items: Vec<(SharedString, SharedString)>,
    open: Vec<usize>,
    corners: Option<dodai_core::Corners>,
    motion: Option<crate::motion::Motion>,
}

crate::corners_setter!(Accordion);

impl Accordion {
    /// Create a new [`Accordion`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            items: Vec::new(),
            open: Vec::new(),
            motion: None,
        }
    }
    /// Pace, easing and travel of opening and closing (a spring overshoots a little). Defaults
    /// to the design's motion.
    pub fn motion(mut self, m: crate::motion::Motion) -> Self {
        self.motion = Some(m);
        self
    }
    /// Add an item.
    pub fn item(mut self, title: impl Into<SharedString>, body: impl Into<SharedString>) -> Self {
        self.items.push((title.into(), body.into()));
        self
    }
    /// Indices expanded initially.
    pub fn expanded(mut self, open: impl IntoIterator<Item = usize>) -> Self {
        self.open = open.into_iter().collect();
        self
    }
}

impl Accordion {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let n = self.items.len();
        let initial = self.open.clone();
        let st = window.use_keyed_state(self.id.clone(), cx, move |_, _| AccState {
            open: (0..n).map(|i| initial.contains(&i)).collect(),
            heights: Rc::default(),
        });
        let open = st.read(cx).open.clone();
        let heights = st.read(cx).heights.clone();
        let motion = {
            use crate::motion::MotionExt;
            self.motion.unwrap_or_else(|| crate::motion::Motion::of(cx))
        };
        let outer = ShapeStyle::from_tokens(&t, None, Some(t.palette.border), false, false);
        let mut col = shaped(&t.shape, outer)
            .w_full()
            .flex()
            .flex_col()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md));
        for (i, (title, body)) in self.items.iter().enumerate() {
            let id: ElementId = (self.id.clone(), format!("h{i}")).into();
            let hv = use_interact(&id, window, cx);
            let hovered = hv.read(cx).hovered;
            let is_open = open.get(i).copied().unwrap_or(false);
            let stc = st.clone();
            let head = track(div().id(id), &hv)
                .min_h(px(t.shape.control_height))
                .px(px(t.spacing.md))
                .flex()
                .items_center()
                .gap(px(t.spacing.sm))
                .cursor_pointer()
                .bg(if hovered {
                    hsla(t.palette.surface_raised)
                } else {
                    gpui::transparent_black()
                })
                .text_color(hsla(if is_open || hovered {
                    t.palette.primary
                } else {
                    t.palette.text
                }))
                .on_click(move |_, _, cx| {
                    stc.update(cx, |s, cx| {
                        if let Some(o) = s.open.get_mut(i) {
                            *o = !*o;
                        }
                        cx.notify();
                    })
                })
                .child(
                    div()
                        .w(px(t.spacing.sm * 1.5))
                        .child(if is_open { "▾" } else { "▸" }),
                )
                .child(format_label(&t.typography, title));
            if i > 0 {
                col = col.child(div().h(px(1.0)).w_full().bg(hsla(t.palette.border)));
            }
            col = col.child(head);
            let p = crate::motion::smooth_with(
                window,
                cx,
                (self.id.clone(), format!("b{i}")).into(),
                if is_open { 1.0 } else { 0.0 },
                motion,
            );
            if p > 0.001 {
                let hs = heights.clone();
                let measure = gpui::canvas(
                    move |b, w, _| {
                        let now: f32 = b.size.height.into();
                        let mut hs = hs.borrow_mut();
                        if hs.len() <= i {
                            hs.resize(i + 1, 0.0);
                        }
                        if (now - hs[i]).abs() > 0.5 {
                            hs[i] = now;
                            w.refresh();
                        }
                    },
                    |_, _, _, _| (),
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full();
                let b = div()
                    .relative()
                    .px(px(t.spacing.lg + 4.0))
                    .pb(px(t.spacing.md))
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(body.clone())
                    .child(measure);
                // Open and settled: natural height. Moving: the measured height, scaled.
                col = col.child(if p < 0.999 {
                    let full = heights.borrow().get(i).copied().unwrap_or(0.0);
                    div()
                        .overflow_hidden()
                        .h(px(full * p))
                        .opacity(p.min(1.0))
                        .child(b)
                } else {
                    div().child(b)
                });
            }
        }
        col
    }
}

crate::entry_render!(Accordion);

/// Used by tests and docs: states a nav button resolves to.
pub fn nav_state(disabled: bool) -> ControlState {
    if disabled {
        ControlState::Disabled
    } else {
        ControlState::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_items_resolve_to_the_disabled_state() {
        assert_eq!(nav_state(true), ControlState::Disabled);
        assert_eq!(nav_state(false), ControlState::Idle);
    }
}
