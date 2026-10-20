//! [`ListItem`] (selectable row) and [`Divider`].

use crate::interact::{track, use_interact};
use crate::paint::{shaped_id, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    div, px, App, ClickEvent, ElementId, Hsla, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window,
};

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
    icon: Option<crate::icons::IconName>,
    on_click: Option<crate::interact::ClickHandler>,
    corners: Option<dodai_core::Corners>,
    depth: u8,
}

impl ListItem {
    /// Nest the row `levels` deep: indented, smaller, muted until selected.
    /// Use it for the children of a menu entry (sub-groups, jump targets).
    pub fn depth(mut self, levels: u8) -> Self {
        self.depth = levels;
        self
    }
    /// Cut these corners instead of the theme's.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.corners = Some(c);
        self
    }
}

impl ListItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            corners: None,
            depth: 0,
            id: id.into(),
            label: label.into(),
            selected: false,
            icon: None,
            on_click: None,
        }
    }
    pub fn selected(mut self, s: bool) -> Self {
        self.selected = s;
        self
    }
    /// Vector icon shown at the row start, in the row's text colour.
    pub fn icon(mut self, g: crate::icons::IconName) -> Self {
        self.icon = Some(g);
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl ListItem {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let st = use_interact(&self.id, window, cx);
        let hovered = st.read(cx).hovered;
        let (mut style, mut fg, marker) = list_style(&t, self.selected, hovered);
        if self.depth > 0 && !self.selected && !hovered {
            fg = hsla(t.palette.text_muted);
        }
        // Nested rows indent the whole row (fill and marker included), so a
        // selected child reads as belonging to its parent instead of repeating
        // the parent's full-width highlight.
        let (indent, size, hgt) = if self.depth > 0 {
            (self.depth as f32 * t.spacing.md, t.typography.size_sm, 0.65)
        } else {
            (0.0, t.typography.size_md, 0.8)
        };
        if self.depth > 0 {
            style.fill = style.fill.map(|c| c.opacity(0.55));
        }
        let pad = t.spacing.md;
        let mut el = track(shaped_id(self.id, &t.shape, style), &st)
            .min_h(px(t.shape.control_height * hgt))
            .py(px(t.spacing.xs * 0.5))
            .ml(px(indent))
            .pl(px(pad))
            .pr(px(t.spacing.md))
            .overflow_hidden()
            .flex()
            .items_center()
            .cursor_pointer()
            .font_family(first_family(t.typography.body))
            .text_size(px(size))
            .text_color(fg)
            .gap(px(t.spacing.sm));
        if let Some(g) = self.icon {
            el = el.child(
                crate::icons::Icon::new(g)
                    .size(t.typography.size_md * 1.2)
                    .color(fg),
            );
        }
        el = el.child(
            div()
                .flex_1()
                .min_w_0()
                .child(format_label(&t.typography, &self.label)),
        );
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

crate::entry_render!(ListItem);

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
    use crate::testing::{light_rounded, square};

    #[test]
    fn selected_row_has_marker_on_square_themes() {
        let t = square();
        let (s, fg, m) = list_style(&t, true, false);
        assert_eq!(s.fill, Some(hsla(t.palette.selection)));
        assert_eq!(fg, hsla(t.palette.text));
        assert_eq!(m, Some(hsla(t.palette.primary)));
        let (s, fg, m) = list_style(&t, false, false);
        assert!(s.fill.is_none() && m.is_none());
        assert_eq!(fg, hsla(t.palette.text_muted));
    }

    #[test]
    fn rounded_themes_have_no_accent_bar() {
        let t = light_rounded();
        let (s, _, m) = list_style(&t, true, false);
        assert_eq!(
            s.fill,
            Some(hsla(t.palette.roles.secondary_container)),
            "tonal pill"
        );
        assert!(m.is_none(), "no vertical bar on Material");
    }
}
