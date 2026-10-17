//! [`ResponsiveShell`]: sidebar + content that collapses into a slide-over drawer
//! (opened from a ☰ button) when the window is narrower than a breakpoint.

use crate::{hsla, theme_of};
use gpui::{
    deferred, div, px, AnyElement, App, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, StatefulInteractiveElement, Styled, Window,
};

/// Whether the shell shows the sidebar inline for a viewport `width` (pure).
pub fn is_wide(width: f32, breakpoint: f32) -> bool {
    width >= breakpoint
}

/// Responsive app frame. Wide: `[sidebar | content]`. Narrow: content only, with a
/// menu button that opens the sidebar as an overlay drawer.
#[derive(IntoElement)]
pub struct ResponsiveShell {
    id: ElementId,
    sidebar: Option<AnyElement>,
    content: Option<AnyElement>,
    breakpoint: f32,
    sidebar_width: f32,
    menu_label: String,
}

impl ResponsiveShell {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            sidebar: None,
            content: None,
            breakpoint: 640.0,
            sidebar_width: 220.0,
            menu_label: "☰ Menu".into(),
        }
    }
    pub fn sidebar(mut self, el: impl IntoElement) -> Self {
        self.sidebar = Some(el.into_any_element());
        self
    }
    pub fn content(mut self, el: impl IntoElement) -> Self {
        self.content = Some(el.into_any_element());
        self
    }
    /// Viewport width (px) below which the sidebar collapses into a drawer.
    pub fn breakpoint(mut self, px: f32) -> Self {
        self.breakpoint = px;
        self
    }
    pub fn sidebar_width(mut self, px: f32) -> Self {
        self.sidebar_width = px;
        self
    }
    pub fn menu_label(mut self, s: impl Into<String>) -> Self {
        self.menu_label = s.into();
        self
    }
}

impl RenderOnce for ResponsiveShell {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let wide = is_wide(window.viewport_size().width.into(), self.breakpoint);
        let open = window.use_keyed_state(self.id.clone(), cx, |_, _| false);
        let w = px(self.sidebar_width);
        let side = self.sidebar.map(|s| {
            div()
                .id((self.id.clone(), "side"))
                .flex_none()
                .w(w)
                .h_full()
                .overflow_y_scroll()
                .child(s)
        });
        let content = div()
            .id((self.id.clone(), "content"))
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .children(self.content);
        let mut root = div().relative().size_full().flex();
        if wide {
            return root.children(side).child(content);
        }
        // Narrow: menu button on top, sidebar as overlay drawer.
        let is_open = *open.read(cx);
        let toggle = open.clone();
        let bar = div()
            .id((self.id.clone(), "menu"))
            .flex_none()
            .px(px(t.spacing.md))
            .py(px(t.spacing.sm))
            .cursor_pointer()
            .border_b_1()
            .border_color(hsla(t.palette.border))
            .text_color(hsla(t.palette.text))
            .on_click(move |_, _, cx| {
                toggle.update(cx, |o, cx| {
                    *o = !*o;
                    cx.notify()
                })
            })
            .child(self.menu_label);
        root = root.child(
            div()
                .flex_1()
                .min_w_0()
                .h_full()
                .flex()
                .flex_col()
                .child(bar)
                .child(content),
        );
        if is_open {
            let close = open.clone();
            let drawer = div()
                .id((self.id.clone(), "drawer"))
                .absolute()
                .inset_0()
                .flex()
                .child(
                    div()
                        .h_full()
                        .bg(hsla(t.palette.background))
                        .border_r_1()
                        .border_color(hsla(t.palette.border_strong))
                        .occlude()
                        .children(side),
                )
                .child(
                    div()
                        .id((self.id.clone(), "backdrop"))
                        .flex_1()
                        .h_full()
                        .bg(gpui::black().opacity(0.45))
                        .on_click(move |_, _, cx| {
                            close.update(cx, |o, cx| {
                                *o = false;
                                cx.notify()
                            })
                        }),
                );
            root = root.child(deferred(drawer).with_priority(80));
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoint_is_inclusive() {
        assert!(is_wide(640.0, 640.0));
        assert!(!is_wide(639.0, 640.0));
    }
}
