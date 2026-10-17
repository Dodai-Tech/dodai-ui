//! [`Tabs`]: a row of selectable tabs driven by `theme.tab` tokens.

use crate::interact::{track, use_interact};
use crate::paint::{bottom_band_canvas, shaped_id, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    div, px, App, ElementId, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

/// Resolved tab paint: shape style, text colour, optional indicator colour.
pub fn tab_style(theme: &Theme, active: bool, hovered: bool) -> (ShapeStyle, Hsla, Option<Hsla>) {
    let k = theme.tab(active, hovered);
    (
        ShapeStyle::from_tokens(theme, k.background, None, active, false),
        hsla(k.foreground),
        k.indicator.map(hsla),
    )
}

type SelectHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Tab strip. Controlled: pass `active` and update it in `on_select`.
#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    labels: Vec<SharedString>,
    active: usize,
    on_select: Option<SelectHandler>,
    corners: Option<dodai_core::Corners>,
}

impl Tabs {
    /// Cut these corners instead of the theme's.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.corners = Some(c);
        self
    }
}

impl Tabs {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            labels: Vec::new(),
            active: 0,
            on_select: None,
        }
    }
    /// Tab labels in order.
    pub fn labels<S: Into<SharedString>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }
    /// Index of the active tab.
    pub fn active(mut self, i: usize) -> Self {
        self.active = i;
        self
    }
    /// Called with the clicked tab index.
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl Tabs {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let mut row = div().flex().gap(px(t.spacing.xs));
        for (i, label) in self.labels.iter().enumerate() {
            let id: ElementId = (self.id.clone(), i.to_string()).into();
            let st = use_interact(&id, window, cx);
            let hovered = st.read(cx).hovered;
            let active = i == self.active;
            let p = crate::motion::transition(window, cx, id.clone(), active);
            let (mut style, fg, indicator) = tab_style(&t, active, hovered);
            let (on_s, on_fg, _) = tab_style(&t, true, hovered);
            let (off_s, off_fg, _) = tab_style(&t, false, hovered);
            style.fill = crate::motion::mix_opt(off_s.fill, on_s.fill, p);
            let fg = if p > 0.0 && p < 1.0 {
                crate::motion::mix(off_fg, on_fg, p)
            } else {
                fg
            };
            let indicator = indicator
                .or_else(|| tab_style(&t, true, hovered).2.filter(|_| p > 0.01))
                .map(|c| c.opacity(p));
            let mut tab = track(shaped_id(id, &t.shape, style), &st)
                .h(px(t.shape.control_height))
                .px(px(t.spacing.md))
                .flex()
                .items_center()
                .cursor_pointer()
                .font_family(first_family(t.typography.body))
                .text_size(px(t.typography.size_md))
                .text_color(fg)
                .child(format_label(&t.typography, label));
            if let Some(c) = indicator {
                tab = tab.child(bottom_band_canvas(&t.shape, t.shape.border_width, c));
            }
            if let Some(f) = self.on_select.clone() {
                tab = tab.on_click(move |_, w, cx| f(i, w, cx));
            }
            row = row.child(tab);
        }
        row
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "Tabs", motion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::light_rounded;

    #[test]
    fn active_tab_uses_indicator_tokens() {
        let t = light_rounded();
        let (s, fg, ind) = tab_style(&t, true, false);
        assert_eq!(s.fill, Some(hsla(t.palette.primary)));
        assert_eq!(fg, hsla(t.palette.on_primary));
        assert!(ind.is_some());
        let (s, _, ind) = tab_style(&t, false, false);
        assert!(s.fill.is_none() && ind.is_none());
        let (s, _, _) = tab_style(&t, false, true);
        assert_eq!(s.fill, Some(hsla(t.palette.surface_raised)));
    }
}
