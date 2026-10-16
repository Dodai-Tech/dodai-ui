//! [`ListItem`] (selectable row) and [`Divider`].

use crate::interact::{track, use_interact};
use crate::paint::{shaped_id, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use gpui::{
    div, px, App, ClickEvent, ElementId, Hsla, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use legendary_core::Theme;

/// Row paint: shape style, text colour, marker colour (pure).
pub fn list_style(
    theme: &Theme,
    selected: bool,
    hovered: bool,
) -> (ShapeStyle, Hsla, Option<Hsla>) {
    let k = theme.list_item(selected, hovered);
    (
        ShapeStyle::from_tokens(theme, k.background, None, false, false),
        hsla(k.foreground),
        k.marker.map(hsla),
    )
}

/// Selectable list/menu row with a left accent marker when selected.
#[derive(IntoElement)]
pub struct ListItem {
    id: ElementId,
    label: SharedString,
    selected: bool,
    on_click: Option<crate::interact::ClickHandler>,
}

impl ListItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected: false,
            on_click: None,
        }
    }
    pub fn selected(mut self, s: bool) -> Self {
        self.selected = s;
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl RenderOnce for ListItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = use_interact(&self.id, window, cx);
        let hovered = st.read(cx).hovered;
        let (style, fg, marker) = list_style(&t, self.selected, hovered);
        let mut el = track(shaped_id(self.id, &t.shape, style), &st)
            .h(px(t.shape.control_height * 0.8))
            .px(px(t.spacing.md))
            .flex()
            .items_center()
            .cursor_pointer()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(fg)
            .child(format_label(&t.typography, &self.label));
        if let Some(c) = marker {
            el = el.child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .bottom_0()
                    .w(px(t.shape.border_width.max(2.0)))
                    .bg(c),
            );
        }
        if let Some(h) = self.on_click {
            el = el.on_click(move |e, w, cx| h(e, w, cx));
        }
        el
    }
}

/// Thin separator line in the `divider` colour.
#[derive(IntoElement)]
pub struct Divider {
    vertical: bool,
}

impl Divider {
    /// Horizontal divider.
    pub fn new() -> Self {
        Self { vertical: false }
    }
    /// Vertical divider (needs a parent with a height).
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }
}

impl Default for Divider {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Divider {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let bg = hsla(t.divider());
        if self.vertical {
            div().w(px(1.0)).h_full().bg(bg)
        } else {
            div().h(px(1.0)).w_full().bg(bg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::light_rounded;

    #[test]
    fn selected_row_has_marker() {
        let t = light_rounded();
        let (s, fg, m) = list_style(&t, true, false);
        assert_eq!(s.fill, Some(hsla(t.palette.selection)));
        assert_eq!(fg, hsla(t.palette.text));
        assert_eq!(m, Some(hsla(t.palette.primary)));
        let (s, fg, m) = list_style(&t, false, false);
        assert!(s.fill.is_none() && m.is_none());
        assert_eq!(fg, hsla(t.palette.text_muted));
    }
}
