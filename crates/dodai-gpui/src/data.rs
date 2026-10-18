//! Data display: [`Table`], [`Badge`], [`Avatar`], [`Chip`], [`Kbd`].

use crate::interact::{track, use_interact, IndexHandler};
use crate::list::list_style;
use crate::paint::{round_shape, scaled_shape, shaped, shaped_id, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::{Theme, Tone};
use gpui::{
    div, px, App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

/// Data table: header + rows, optional selected row. Cells are text.
/// Column `i` gets flex weight `weights[i]` (default 1).
#[derive(IntoElement)]
pub struct Table {
    id: ElementId,
    columns: Vec<SharedString>,
    weights: Vec<f32>,
    rows: Vec<Vec<SharedString>>,
    selected: Option<usize>,
    on_select: Option<IndexHandler>,
    sort: Option<(usize, bool)>,
    on_sort: Option<IndexHandler>,
    corners: Option<dodai_core::Corners>,
}

impl Table {
    /// Cut these corners instead of the theme's.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.corners = Some(c);
        self
    }
}

impl Table {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            columns: Vec::new(),
            weights: Vec::new(),
            rows: Vec::new(),
            selected: None,
            on_select: None,
            sort: None,
            on_sort: None,
        }
    }
    pub fn columns<S: Into<SharedString>>(mut self, c: impl IntoIterator<Item = S>) -> Self {
        self.columns = c.into_iter().map(Into::into).collect();
        self
    }
    pub fn weights(mut self, w: impl IntoIterator<Item = f32>) -> Self {
        self.weights = w.into_iter().collect();
        self
    }
    pub fn rows<S: Into<SharedString>, R: IntoIterator<Item = S>>(
        mut self,
        rows: impl IntoIterator<Item = R>,
    ) -> Self {
        self.rows = rows
            .into_iter()
            .map(|r| r.into_iter().map(Into::into).collect())
            .collect();
        self
    }
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self
    }
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
    /// Show a sort arrow on `column` (`ascending` → ▲).
    pub fn sorted(mut self, column: usize, ascending: bool) -> Self {
        self.sort = Some((column, ascending));
        self
    }
    /// Make headers clickable; called with the column index.
    pub fn on_sort(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_sort = Some(Rc::new(f));
        self
    }
}

impl Table {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let p = &t.palette;
        let w = |i: usize| self.weights.get(i).copied().unwrap_or(1.0);
        let cell = |i: usize, text: SharedString| {
            div()
                .flex_basis(px(0.0))
                .flex_grow()
                .min_w(px(0.0))
                .overflow_hidden()
                .whitespace_nowrap()
                .px(px(t.spacing.xs))
                .child(text)
                .map_weight(w(i))
        };
        let outer = ShapeStyle::from_tokens(&t, Some(p.surface), Some(p.border), false, false);
        let mut col = shaped(&t.shape, outer)
            .w_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md));
        let mut head = div()
            .flex()
            .items_center()
            .h(px(t.shape.control_height * 0.85))
            .px(px(t.spacing.md))
            .bg(hsla(p.surface_raised))
            .text_size(px(t.typography.size_sm))
            .text_color(hsla(p.text_muted));
        for (i, c) in self.columns.iter().enumerate() {
            let mut label = format_label(&t.typography, c).to_string();
            if let Some((sc, asc)) = self.sort {
                if sc == i {
                    label.push_str(if asc { " ▲" } else { " ▼" });
                }
            }
            let mut h = cell(i, label.into()).id((self.id.clone(), format!("h{i}")));
            if let Some(f) = self.on_sort.clone() {
                h = h.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx));
            }
            head = head.child(h);
        }
        col = col.child(head);
        for (ri, row) in self.rows.iter().enumerate() {
            let id: ElementId = (self.id.clone(), ri.to_string()).into();
            let st = use_interact(&id, window, cx);
            let hovered = st.read(cx).hovered;
            let (style, fg, marker) = list_style(&t, self.selected == Some(ri), hovered);
            let fg = if self.selected == Some(ri) || hovered {
                hsla(p.text)
            } else {
                let _ = fg;
                hsla(p.text)
            };
            let mut r = track(
                shaped_id(
                    id,
                    &t.shape,
                    ShapeStyle {
                        stroke: None,
                        ..style
                    },
                ),
                &st,
            )
            .flex()
            .items_center()
            .h(px(t.shape.control_height))
            .px(px(t.spacing.md))
            .border_t_1()
            .border_color(hsla(p.border))
            .text_color(fg);
            for (i, c) in row.iter().enumerate() {
                r = r.child(cell(i, c.clone()));
            }
            if let Some(m) = marker {
                r = r.child(
                    div()
                        .absolute()
                        .left_0()
                        .top_0()
                        .bottom_0()
                        .w(px(t.shape.border_width.max(2.0)))
                        .bg(m),
                );
            }
            if let Some(f) = self.on_select.clone() {
                r = r.cursor_pointer().on_click(move |_, w, cx| f(ri, w, cx));
            }
            col = col.child(r);
        }
        col
    }
}

impl RenderOnce for Table {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "Table", motion)
    }
}

/// Small helper trait: apply a flex weight to a cell.
trait Weight: Styled + Sized {
    fn map_weight(mut self, w: f32) -> Self {
        self.style().flex_grow = Some(w.max(0.0));
        self
    }
}
impl<T: Styled + Sized> Weight for T {}

/// Paint for a [`Badge`]: solid tone fill. Pure.
pub fn badge_style(theme: &Theme, tone: Tone) -> (ShapeStyle, gpui::Hsla) {
    let c = theme.palette.tone(tone);
    (
        ShapeStyle::from_tokens(theme, Some(c), None, false, false),
        hsla(theme.on(c)),
    )
}

/// Count / dot indicator. Round on rounded themes, shaped otherwise.
#[derive(IntoElement)]
pub struct Badge {
    text: Option<SharedString>,
    tone: Tone,
    corners: Option<dodai_core::Corners>,
}

impl Badge {
    /// Cut these corners instead of the theme's.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.corners = Some(c);
        self
    }
}

impl Badge {
    /// Badge with a short label (e.g. a count).
    pub fn new(text: impl Into<SharedString>, tone: Tone) -> Self {
        Self {
            corners: None,
            text: Some(text.into()),
            tone,
        }
    }
    /// Label-less status dot.
    pub fn dot(tone: Tone) -> Self {
        Self {
            text: None,
            tone,
            corners: None,
        }
    }
}

impl Badge {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let (style, fg) = badge_style(&t, self.tone);
        let h = if self.text.is_some() { 18.0 } else { 10.0 };
        let shape = round_shape(&t.shape, h);
        let d = shaped(&shape, style)
            .h(px(h))
            .flex()
            .items_center()
            .justify_center()
            .text_color(fg)
            .text_size(px(t.typography.size_xs))
            .font_family(first_family(t.typography.body));
        match self.text {
            Some(s) => d.min_w(px(18.0)).px(px(5.0)).child(s),
            None => d.w(px(h)),
        }
    }
}

impl RenderOnce for Badge {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "Badge", motion)
    }
}

/// Initials in a shaped tile, optional status dot.
#[derive(IntoElement)]
pub struct Avatar {
    initials: SharedString,
    size: f32,
    status: Option<Tone>,
    tone: Option<Tone>,
    corners: Option<dodai_core::Corners>,
}

impl Avatar {
    /// Cut these corners instead of the theme's.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.corners = Some(c);
        self
    }
}

impl Avatar {
    pub fn new(initials: impl Into<SharedString>) -> Self {
        Self {
            corners: None,
            initials: initials.into(),
            size: 36.0,
            status: None,
            tone: None,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Small presence dot at the bottom-right.
    pub fn status(mut self, t: Tone) -> Self {
        self.status = Some(t);
        self
    }
    /// Tint the tile with a tone instead of the brand colour.
    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = Some(t);
        self
    }
}

impl Avatar {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let c = self.tone.map_or(t.palette.primary, |x| t.palette.tone(x));
        let shape = round_shape(&t.shape, self.size);
        let style = ShapeStyle::from_tokens(&t, Some(c.with_alpha(0.2)), Some(c), false, false);
        let mut d = shaped(&shape, style)
            .w(px(self.size))
            .h(px(self.size))
            .flex_none()
            .flex()
            .items_center()
            .justify_center()
            .text_color(hsla(c))
            .text_size(px(self.size * 0.38))
            .font_family(first_family(t.typography.display))
            .child(self.initials.to_uppercase());
        if let Some(s) = self.status {
            let dot = round_shape(&t.shape, 10.0);
            d = d.child(
                shaped(
                    &dot,
                    ShapeStyle::from_tokens(
                        &t,
                        Some(t.palette.tone(s)),
                        Some(t.palette.surface),
                        false,
                        false,
                    ),
                )
                .absolute()
                .right(px(-2.0))
                .bottom(px(-2.0))
                .w(px(11.0))
                .h(px(11.0)),
            );
        }
        d
    }
}

impl RenderOnce for Avatar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "Avatar", motion)
    }
}

/// Paint of a [`Chip`] (pure): `(style, text colour)`.
pub fn chip_style(theme: &Theme, selected: bool, hovered: bool) -> (ShapeStyle, gpui::Hsla) {
    let p = &theme.palette;
    if selected {
        (
            ShapeStyle::from_tokens(theme, Some(p.selection), Some(p.primary), false, false),
            hsla(p.text),
        )
    } else {
        (
            ShapeStyle::from_tokens(
                theme,
                hovered.then_some(p.surface_raised),
                Some(if hovered { p.border_strong } else { p.border }),
                false,
                false,
            ),
            hsla(if hovered { p.text } else { p.text_muted }),
        )
    }
}

type ClickFn = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// Selectable filter chip with optional ✕ remove affordance.
#[derive(IntoElement)]
pub struct Chip {
    id: ElementId,
    label: SharedString,
    selected: bool,
    removable: bool,
    disabled: bool,
    on_click: Option<ClickFn>,
    on_remove: Option<ClickFn>,
    corners: Option<dodai_core::Corners>,
}

impl Chip {
    /// Cut these corners instead of the theme's.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.corners = Some(c);
        self
    }
}

impl Chip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            label: label.into(),
            selected: false,
            removable: false,
            disabled: false,
            on_click: None,
            on_remove: None,
        }
    }
    pub fn selected(mut self, s: bool) -> Self {
        self.selected = s;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
    /// Show ✕ and call `f` when it is clicked.
    pub fn on_remove(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.removable = true;
        self.on_remove = Some(Rc::new(f));
        self
    }
}

impl Chip {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let st = use_interact(&self.id, window, cx);
        let hovered = st.read(cx).hovered && !self.disabled;
        let (mut style, mut fg) = chip_style(&t, self.selected, hovered);
        if self.disabled {
            style = ShapeStyle::from_tokens(&t, None, Some(t.palette.border), false, false);
            fg = hsla(t.palette.text_disabled);
        }
        let h = t.shape.control_height * 0.7;
        let shape = round_shape(&t.shape, h);
        let rid: ElementId = (self.id.clone(), "x").into();
        let mut el = track(shaped_id(self.id, &shape, style), &st)
            .h(px(h))
            .px(px(t.spacing.md))
            .flex()
            .items_center()
            .gap(px(t.spacing.sm))
            .text_color(fg)
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_sm))
            .child(format_label(&t.typography, &self.label));
        if self.removable {
            let mut x = div()
                .id(rid)
                .text_color(hsla(t.palette.text_muted))
                .hover(|s| s.text_color(hsla(t.palette.danger)))
                .child("✕");
            if let (false, Some(f)) = (self.disabled, self.on_remove) {
                x = x.cursor_pointer().on_click(move |e, w, cx| {
                    cx.stop_propagation();
                    f(e, w, cx)
                });
            }
            el = el.child(x);
        }
        if let (false, Some(f)) = (self.disabled, self.on_click) {
            el = el.cursor_pointer().on_click(move |e, w, cx| f(e, w, cx));
        }
        el
    }
}

impl RenderOnce for Chip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "Chip", motion)
    }
}

/// Keyboard key hint, e.g. `Kbd::new("⌘K")`.
#[derive(IntoElement)]
pub struct Kbd(SharedString, Option<dodai_core::Corners>);

impl Kbd {
    pub fn new(key: impl Into<SharedString>) -> Self {
        Self(key.into(), None)
    }
    /// Cut these corners instead of the theme's.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.1 = Some(c);
        self
    }
}

impl Kbd {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = crate::apply_corners(theme_of(cx), self.1);
        let p = &t.palette;
        let style = ShapeStyle::from_tokens(
            &t,
            Some(p.surface_raised),
            Some(p.border_strong),
            false,
            false,
        );
        shaped(&scaled_shape(&t.shape, 20.0), style)
            .h(px(22.0))
            .min_w(px(22.0))
            .px(px(6.0))
            .flex()
            .items_center()
            .justify_center()
            .font_family(first_family(t.typography.mono))
            .text_size(px(t.typography.size_xs))
            .text_color(hsla(p.text_muted))
            .child(self.0)
    }
}

impl RenderOnce for Kbd {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "Kbd", motion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{light_rounded, square};

    #[test]
    fn badge_and_chip_tokens() {
        let t = square();
        let (s, fg) = badge_style(&t, Tone::Warning);
        assert_eq!(s.fill, Some(hsla(t.palette.warning)));
        assert_eq!(fg, hsla(t.on(t.palette.warning)));
        let l = light_rounded();
        let (s, _) = chip_style(&l, true, false);
        assert_eq!(s.stroke, Some(hsla(l.palette.primary)));
        let (s, _) = chip_style(&l, false, false);
        assert!(s.fill.is_none());
    }
}
