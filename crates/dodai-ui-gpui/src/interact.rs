//! Hover / pressed tracking shared by interactive components.

use dodai_ui_core::ControlState;
use gpui::{
    App, ClickEvent, ElementId, Entity, FocusHandle, InteractiveElement, MouseButton, Stateful,
    StatefulInteractiveElement, Styled, Window,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Shape plus the edge gap (a cropped control) the focus ring has to follow.
type RingShape = (
    dodai_ui_core::Shape,
    Option<(crate::paint::ShapeEdge, f32, f32)>,
);

thread_local! {
    /// Shape of the most recently built shaped element (for the focus ring overlay).
    static RING_SHAPE: RefCell<Option<RingShape>> = const { RefCell::new(None) };
}

/// Remember the shape being built, so [`track`] can draw its focus ring along the same outline.
pub(crate) fn set_ring_shape(
    s: &dodai_ui_core::Shape,
    edge_gap: Option<(crate::paint::ShapeEdge, f32, f32)>,
) {
    RING_SHAPE.with(|c| *c.borrow_mut() = Some((s.clone(), edge_gap)));
}

/// Pointer state of one control (kept in gpui keyed element state).
#[derive(Debug, Clone, Default)]
pub struct Interact {
    /// Pointer is over the control.
    pub hovered: bool,
    /// Primary button is held down on the control.
    pub pressed: bool,
    /// Keyboard focus target: makes the control a tab stop (Enter / Space click it) and shows
    /// the theme's focus ring while focused.
    pub focus: Option<FocusHandle>,
    /// Set while focus came from a click rather than the keyboard: the focus ring stays hidden
    /// (like CSS `:focus-visible`). Cleared on blur and on the first key press.
    pointer: Rc<Cell<bool>>,
}

impl Interact {
    /// Map pointer state (and `disabled`) to a token [`ControlState`].
    pub fn control_state(&self, disabled: bool) -> ControlState {
        if disabled {
            ControlState::Disabled
        } else if self.pressed {
            ControlState::Pressed
        } else if self.hovered {
            ControlState::Hovered
        } else {
            ControlState::Idle
        }
    }
}

/// Get (or create) the pointer state entity for `id`.
pub fn use_interact(id: &ElementId, window: &mut Window, cx: &mut App) -> Entity<Interact> {
    window.use_keyed_state(id.clone(), cx, |window, cx| {
        let focus = cx.focus_handle().tab_stop(true);
        let me = Interact {
            focus: Some(focus.clone()),
            ..Interact::default()
        };
        let flag = me.pointer.clone();
        cx.on_blur(&focus, window, move |_, _, _| flag.set(false))
            .detach();
        me
    })
}

/// Wire hover/press tracking, keyboard focus and the theme's focus ring onto an id'd element.
pub fn track<E>(el: Stateful<E>, state: &Entity<Interact>, cx: &App) -> Stateful<E>
where
    Stateful<E>: StatefulInteractiveElement + InteractiveElement + gpui::ParentElement,
{
    let s1 = state.clone();
    let s2 = state.clone();
    let s3 = state.clone();
    let (focus, pointer) = {
        let s = state.read(cx);
        (s.focus.clone(), s.pointer.clone())
    };
    let (ring, thick) = {
        let (c, w) = crate::theme_of(cx).focus_ring();
        (crate::hsla(c), w)
    };
    let shape = RING_SHAPE.with(|c| c.borrow().clone());
    let (p1, p2, p3) = (pointer.clone(), pointer.clone(), pointer);
    let el = match focus {
        Some(h) => {
            // The ring is an absolutely positioned overlay tracking the same focus handle, so it
            // takes no layout space and shows only while the control has keyboard focus.
            let base = gpui::div()
                .id(("dodai-focus-ring", state.entity_id()))
                .track_focus(&h)
                .absolute()
                .top_0()
                .left_0()
                .size_full();
            let overlay = match shape {
                // Cut shapes: the ring follows the polygon, hidden until the control has focus.
                Some((sh, gap)) if sh.kind() == dodai_ui_core::ShapeKind::Chamfered => {
                    let style = crate::paint::ShapeStyle {
                        fill: None,
                        stroke: Some(ring),
                        stroke_width: thick,
                        glow: None,
                        shadow: None,
                        shadow_dir: (1.0, 1.0),
                        scanlines: false,
                        scanline: ring,
                        scanline_pitch: dodai_ui_core::Effects::SCANLINE_PITCH,
                        surface: None,
                        shimmer: None,
                        elevation: None,
                        lift_layers: None,
                        edge_gap: gap,
                        frost: None,
                    };
                    gpui::ParentElement::child(
                        base.invisible()
                            .focus(move |s| if p1.get() { s } else { s.visible() }),
                        gpui::canvas(
                            |_, _, _| (),
                            move |b, _, window, _| {
                                crate::paint::paint_polygon(window, b, &sh, &style)
                            },
                        )
                        .size_full(),
                    )
                }
                other => {
                    let other = other.map(|(s, _)| s);
                    let radius = other
                        .filter(|s| s.kind() == dodai_ui_core::ShapeKind::Rounded)
                        .map_or(0.0, |s| s.radius);
                    base.rounded(gpui::px(radius)).focus(move |s| {
                        if p2.get() {
                            return s;
                        }
                        let mut s = s.border_color(ring);
                        s.border_widths = gpui::EdgesRefinement {
                            top: Some(gpui::px(thick).into()),
                            right: Some(gpui::px(thick).into()),
                            bottom: Some(gpui::px(thick).into()),
                            left: Some(gpui::px(thick).into()),
                        };
                        s
                    })
                }
            };
            gpui::ParentElement::child(el.track_focus(&h), overlay)
        }
        None => el,
    };
    el.on_hover(move |h, _, cx| {
        let h = *h;
        s1.update(cx, |s, cx| {
            if s.hovered != h || (!h && s.pressed) {
                s.hovered = h;
                if !h {
                    s.pressed = false;
                }
                cx.notify();
            }
        });
    })
    .on_key_down(move |_, _, _| p3.set(false))
    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
        s2.update(cx, |s, cx| {
            s.pointer.set(true);
            if !s.pressed {
                s.pressed = true;
                cx.notify();
            }
        });
    })
    .on_mouse_up(MouseButton::Left, move |_, window, cx| {
        // Pointer users don't get the keyboard focus ring: drop the focus a click gave us.
        let mine = s3.read(cx).focus.clone();
        s3.update(cx, |s, cx| {
            if s.pressed {
                s.pressed = false;
                cx.notify();
            }
        });
        if mine.is_some_and(|h| h.is_focused(window)) {
            window.blur();
        }
    })
}

/// How much of a control shows when it is disabled and has no disabled tokens of its own.
pub const DISABLED_OPACITY: f32 = 0.38;

/// Shared index-selection callback type (tabs, select, pagination…).
pub type IndexHandler = std::rc::Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Shared click callback type.
pub type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_mapping() {
        let i = |h, p| Interact {
            hovered: h,
            pressed: p,
            focus: None,
            pointer: Default::default(),
        };
        assert_eq!(i(false, false).control_state(false), ControlState::Idle);
        assert_eq!(i(true, false).control_state(false), ControlState::Hovered);
        assert_eq!(i(true, true).control_state(false), ControlState::Pressed);
        assert_eq!(i(true, true).control_state(true), ControlState::Disabled);
    }
}
