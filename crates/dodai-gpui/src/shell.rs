//! [`ResponsiveShell`]: sidebar + content that collapses into a slide-over drawer
//! (opened from a ☰ button) when the window is narrower than a breakpoint.

use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    deferred, div, px, AnyElement, App, ElementId, Entity, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

type CollapseFn = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// Spring stiffness / damping for the sidebar width (slightly under-damped).
const STIFFNESS: f32 = 380.0;
const DAMPING: f32 = 24.0;
/// How far ahead (s) a release velocity is projected when choosing a side.
const FLING: f32 = 0.12;

/// Which side a released sidebar of width `w` and velocity `v` (px/s) snaps to:
/// `true` = collapsed. Flicking carries it past the midpoint (pure).
pub fn snap_collapsed(w: f32, v: f32, rail: f32, full: f32) -> bool {
    w + v * FLING < (rail + full) * 0.5
}

/// One physics step of the width spring towards `target` (pure).
pub fn spring_step(w: f32, v: f32, target: f32, dt: f32) -> (f32, f32) {
    let mut w = w;
    let mut v = v;
    let n = ((dt / (1.0 / 240.0)).ceil() as u32).clamp(1, 24);
    let h = dt / n as f32;
    for _ in 0..n {
        let a = STIFFNESS * (target - w) - DAMPING * v;
        v += a * h;
        w += v * h;
    }
    (w, v)
}

#[derive(Clone, Copy)]
struct Drag {
    start_x: f32,
    start_w: f32,
    last_x: f32,
    last_at: std::time::Instant,
    moved: bool,
}

struct SideState {
    w: f32,
    v: f32,
    last: std::time::Instant,
    drag: Option<Drag>,
}

impl SideState {
    /// Advance the spring (unless held by the mouse); returns (width, animating).
    fn advance(&mut self, target: f32) -> (f32, bool) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last).as_secs_f32().min(0.05);
        self.last = now;
        if self.drag.is_some() {
            return (self.w, true);
        }
        let (w, v) = spring_step(self.w, self.v, target, dt);
        if (w - target).abs() < 0.3 && v.abs() < 5.0 {
            self.w = target;
            self.v = 0.0;
            return (target, false);
        }
        self.w = w;
        self.v = v;
        (w, true)
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
    /// Create a new [`ResponsiveShell`].
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
    /// The sidebar content.
    pub fn sidebar(mut self, el: impl IntoElement) -> Self {
        self.sidebar = Some(el.into_any_element());
        self
    }
    /// The popup content.
    pub fn content(mut self, el: impl IntoElement) -> Self {
        self.content = Some(el.into_any_element());
        self
    }
    /// Viewport width (px) below which the sidebar collapses into a drawer.
    pub fn breakpoint(mut self, px: f32) -> Self {
        self.breakpoint = px;
        self
    }
    /// Sidebar width in px.
    pub fn sidebar_width(mut self, px: f32) -> Self {
        self.sidebar_width = px;
        self
    }
    /// Icon-only sidebar shown while collapsed (wide layout only).
    pub fn rail(mut self, el: impl IntoElement) -> Self {
        self.rail = Some(el.into_any_element());
        self
    }
    /// Collapsed rail width in px.
    pub fn rail_width(mut self, px: f32) -> Self {
        self.rail_width = px;
        self
    }
    /// Controlled collapsed state; pair with [`Self::on_collapse`].
    pub fn collapsed(mut self, c: bool) -> Self {
        self.collapsed = c;
        self
    }
    crate::callback!(
        /// Enables the drag/click handle on the sidebar edge. Called with the new
        /// collapsed state when the user drags past the threshold or clicks it.
        on_collapse, bool
    );
    /// Label of the menu button when collapsed.
    pub fn menu_label(mut self, s: impl Into<String>) -> Self {
        self.menu_label = s.into();
        self
    }
}

/// Drag the sidebar edge to resize it; on release it snaps to the rail or the full width and the
/// owner hears about each change of the collapsed state.
fn drag_handlers(
    root: gpui::Div,
    side_state: &Entity<SideState>,
    f: CollapseFn,
    collapsed: bool,
    (rail_w, full_w): (f32, f32),
) -> gpui::Div {
    let (mv, up, f2) = (side_state.clone(), side_state.clone(), f.clone());
    root.on_mouse_move(move |e, w, cx| {
        let x = f32::from(e.position.x);
        let mut live = None;
        mv.update(cx, |s, cx| {
            let Some(mut d) = s.drag else { return };
            let dx = x - d.start_x;
            d.moved |= dx.abs() > 3.0;
            let dt = d.last_at.elapsed().as_secs_f32();
            if dt > 0.004 {
                s.v = 0.6 * s.v + 0.4 * (x - d.last_x) / dt;
                d.last_x = x;
                d.last_at = std::time::Instant::now();
            }
            if d.moved {
                s.w = (d.start_w + dx).clamp(rail_w, full_w);
                live = Some(s.w < (rail_w + full_w) * 0.5);
            }
            s.drag = Some(d);
            cx.notify();
        });
        if let Some(c) = live {
            if c != collapsed {
                f(c, w, cx);
            }
        }
    })
    .on_mouse_up(MouseButton::Left, move |_, w, cx| {
        let mut next = None;
        up.update(cx, |s, cx| {
            let Some(d) = s.drag.take() else { return };
            if d.last_at.elapsed().as_secs_f32() > 0.08 {
                s.v = 0.0;
            }
            next = Some(if d.moved {
                snap_collapsed(s.w, s.v, rail_w, full_w)
            } else {
                !collapsed
            });
            cx.notify();
        });
        if let Some(c) = next {
            if c != collapsed {
                f2(c, w, cx);
            }
        }
    })
}

/// The drag handle on the sidebar edge: a 1 px line in a wider hit area that thickens on hover.
fn grip(id: &ElementId, t: &Theme, side_state: &Entity<SideState>) -> gpui::Stateful<gpui::Div> {
    let down = side_state.clone();
    div()
        .id((id.clone(), "grip"))
        .absolute()
        .top_0()
        .bottom_0()
        .right_0()
        // A 1 px line inside a wider invisible hit area; it thickens on hover.
        .group("shell-grip")
        .w(px(t.spacing.sm))
        .flex()
        .justify_end()
        .cursor_col_resize()
        .child(
            div()
                .h_full()
                .w(px(1.0))
                .bg(hsla(t.palette.border))
                .group_hover("shell-grip", |s| s.w(px(3.0)).bg(hsla(t.palette.primary))),
        )
        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
            let x = f32::from(e.position.x);
            down.update(cx, |s, _| {
                s.v = 0.0;
                s.drag = Some(Drag {
                    start_x: x,
                    start_w: s.w,
                    last_x: x,
                    last_at: std::time::Instant::now(),
                    moved: false,
                });
            });
        })
}

impl RenderOnce for ResponsiveShell {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let wide = is_wide(window.viewport_size().width.into(), self.breakpoint);
        let open = window.use_keyed_state(self.id.clone(), cx, |_, _| false);
        let rail_mode = wide && self.collapsed && self.rail.is_some();
        let (rail_w, full_w) = (self.rail_width, self.sidebar_width);
        let target = if self.collapsed { rail_w } else { full_w };
        let side_state =
            window.use_keyed_state((self.id.clone(), "sw"), cx, move |_, _| SideState {
                w: target,
                v: 0.0,
                last: std::time::Instant::now(),
                drag: None,
            });
        let (cur_w, animating) = side_state.update(cx, |s, _| s.advance(target));
        if animating {
            crate::motion::next_frame(window);
        }
        let w = px(cur_w.clamp(rail_w.min(full_w) * 0.5, full_w.max(rail_w) * 1.1));
        let id = self.id.clone();
        let (wide_body, drawer_body) = if wide {
            (if rail_mode { self.rail } else { self.sidebar }, None)
        } else {
            (None, self.sidebar)
        };
        // The drawer carries its own close icon, so it can be folded away from inside as well
        // as by tapping the backdrop or the menu bar.
        let close_icon = {
            let o = open.clone();
            crate::IconButton::new((id.clone(), "close"), "«").on_click(move |_, _, cx| {
                o.update(cx, |o, cx| {
                    *o = false;
                    cx.notify()
                })
            })
        };
        let drawer_side = drawer_body.map(|s| {
            div()
                .id((id.clone(), "side"))
                .relative()
                .flex_none()
                .w(px(self.sidebar_width))
                .h_full()
                .overflow_y_scroll()
                .child(s)
                .child(
                    div()
                        .debug_selector(|| "shell-close".into())
                        .absolute()
                        .top(px(t.spacing.md))
                        .right(px(t.spacing.md))
                        .child(close_icon),
                )
        });
        let collapsed = self.collapsed;
        let grip = self
            .on_collapse
            .as_ref()
            .map(|_| grip(&id, &t, &side_state));
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
                        .overflow_x_hidden()
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
        let mut root = div()
            .relative()
            .size_full()
            .flex()
            .child(crate::motion::input_tracker());
        if wide {
            if let Some(f) = self.on_collapse.clone() {
                root = drag_handlers(root, &side_state, f, collapsed, (rail_w, full_w));
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
                        .bg(crate::overlay::scrim_color(&t))
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
    fn release_snaps_by_position_and_fling() {
        assert!(snap_collapsed(80.0, 0.0, 64.0, 240.0));
        assert!(!snap_collapsed(200.0, 0.0, 64.0, 240.0));
        // flicked left from past the middle: collapses
        assert!(snap_collapsed(170.0, -900.0, 64.0, 240.0));
        // flicked right from near the rail: expands
        assert!(!snap_collapsed(100.0, 900.0, 64.0, 240.0));
    }

    #[test]
    fn spring_settles_on_target() {
        let (mut w, mut v) = (240.0, 0.0);
        for _ in 0..240 {
            (w, v) = spring_step(w, v, 64.0, 1.0 / 60.0);
        }
        assert!((w - 64.0).abs() < 0.5 && v.abs() < 5.0);
    }

    #[test]
    fn breakpoint_is_inclusive() {
        assert!(is_wide(640.0, 640.0));
        assert!(!is_wide(639.0, 640.0));
    }
}
