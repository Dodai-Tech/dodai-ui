//! [`ResponsiveShell`]: sidebar + content that collapses into a slide-over drawer
//! (opened from a ☰ button) when the window is narrower than a breakpoint.

use crate::{hsla, theme_of};
use gpui::{
    deferred, div, px, AnyElement, App, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

type CollapseFn = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// Horizontal drag distance (px) that collapses or expands the sidebar.
const DRAG_THRESHOLD: f32 = 36.0;

/// What a horizontal drag of `dx` px should do to a sidebar that is currently
/// `collapsed`: `Some(new_state)` once past the threshold (pure).
pub fn drag_collapse(collapsed: bool, dx: f32) -> Option<bool> {
    if !collapsed && dx <= -DRAG_THRESHOLD {
        Some(true)
    } else if collapsed && dx >= DRAG_THRESHOLD {
        Some(false)
    } else {
        None
    }
}

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
    rail: Option<AnyElement>,
    rail_width: f32,
    collapsed: bool,
    on_collapse: Option<CollapseFn>,
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
            rail: None,
            rail_width: 64.0,
            collapsed: false,
            on_collapse: None,
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
    /// Icon-only sidebar shown while collapsed (wide layout only).
    pub fn rail(mut self, el: impl IntoElement) -> Self {
        self.rail = Some(el.into_any_element());
        self
    }
    pub fn rail_width(mut self, px: f32) -> Self {
        self.rail_width = px;
        self
    }
    /// Controlled collapsed state; pair with [`Self::on_collapse`].
    pub fn collapsed(mut self, c: bool) -> Self {
        self.collapsed = c;
        self
    }
    /// Enables the drag/click handle on the sidebar edge. Called with the new
    /// collapsed state when the user drags past the threshold or clicks it.
    pub fn on_collapse(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_collapse = Some(Rc::new(f));
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
        let rail_mode = wide && self.collapsed && self.rail.is_some();
        let w = px(if rail_mode {
            self.rail_width
        } else {
            self.sidebar_width
        });
        let id = self.id.clone();
        let (wide_body, drawer_body) = if wide {
            (if rail_mode { self.rail } else { self.sidebar }, None)
        } else {
            (None, self.sidebar)
        };
        let drawer_side = drawer_body.map(|s| {
            div()
                .id((id.clone(), "side"))
                .flex_none()
                .w(px(self.sidebar_width))
                .h_full()
                .overflow_y_scroll()
                .child(s)
        });
        // (press x, moved) while the edge handle is held.
        let drag = window.use_keyed_state((id.clone(), "drag"), cx, |_, _| None::<(f32, bool)>);
        let collapsed = self.collapsed;
        let grip = self.on_collapse.clone().map(|f| {
            let (down, up) = (drag.clone(), drag.clone());
            let up_f = f.clone();
            div()
                .id((id.clone(), "grip"))
                .absolute()
                .top_0()
                .bottom_0()
                .right_0()
                .w(px(6.0))
                .cursor_col_resize()
                .bg(hsla(t.palette.border))
                .hover(|s| s.bg(hsla(t.palette.primary)))
                .on_mouse_down(MouseButton::Left, move |e, _, cx| {
                    let x = f32::from(e.position.x);
                    down.update(cx, |d, _| *d = Some((x, false)));
                })
                .on_mouse_up(MouseButton::Left, move |_, w, cx| {
                    let pressed = up.update(cx, |d, _| d.take());
                    if matches!(pressed, Some((_, false))) {
                        up_f(!collapsed, w, cx);
                    }
                })
        });
        let wide_side = wide_body.map(|b| {
            div()
                .id((id.clone(), "wide-side"))
                .relative()
                .flex_none()
                .w(w)
                .h_full()
                .child(
                    div()
                        .id((id.clone(), "side"))
                        .size_full()
                        .overflow_y_scroll()
                        .child(b),
                )
                .children(grip)
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
            if let Some(f) = self.on_collapse.clone() {
                let (mv, up) = (drag.clone(), drag.clone());
                root = root
                    .on_mouse_move(move |e, w, cx| {
                        let Some((sx, _)) = *mv.read(cx) else { return };
                        let dx = f32::from(e.position.x) - sx;
                        if let Some(next) = drag_collapse(collapsed, dx) {
                            mv.update(cx, |d, _| *d = None);
                            f(next, w, cx);
                        } else if dx.abs() > 3.0 {
                            mv.update(cx, |d, _| *d = Some((sx, true)));
                        }
                    })
                    .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                        up.update(cx, |d, _| *d = None);
                    });
            }
            return root.children(wide_side).child(content);
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
                        .children(drawer_side),
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
    fn drag_collapses_and_expands_past_threshold() {
        assert_eq!(drag_collapse(false, -10.0), None);
        assert_eq!(drag_collapse(false, -40.0), Some(true));
        assert_eq!(drag_collapse(false, 80.0), None);
        assert_eq!(drag_collapse(true, 20.0), None);
        assert_eq!(drag_collapse(true, 40.0), Some(false));
    }

    #[test]
    fn breakpoint_is_inclusive() {
        assert!(is_wide(640.0, 640.0));
        assert!(!is_wide(639.0, 640.0));
    }
}
