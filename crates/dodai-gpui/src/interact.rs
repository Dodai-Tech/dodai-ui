//! Hover / pressed tracking shared by interactive components.

use dodai_core::ControlState;
use gpui::{
    App, ClickEvent, ElementId, Entity, FocusHandle, Hsla, InteractiveElement, MouseButton,
    Stateful, StatefulInteractiveElement, Styled, Window,
};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

/// Shape plus the edge gap (a cropped control) the focus ring has to follow.
type RingShape = (
    dodai_core::Shape,
    Option<(crate::paint::ShapeEdge, f32, f32)>,
);

thread_local! {
    /// Shape of the most recently built shaped element (for the focus ring overlay).
    static RING_SHAPE: RefCell<Option<RingShape>> = const { RefCell::new(None) };
}

/// Remember the shape being built, so [`track`] can draw its focus ring along the same outline.
pub(crate) fn set_ring_shape(
    s: &dodai_core::Shape,
    edge_gap: Option<(crate::paint::ShapeEdge, f32, f32)>,
) {
    RING_SHAPE.with(|c| *c.borrow_mut() = Some((s.clone(), edge_gap)));
}

thread_local! {
    /// The last few focus handles handed out by [`use_interact`], so [`track`] (which only gets
    /// the state entity) can attach the right one. Bounded: tracking follows creation closely.
    static HANDLES: RefCell<VecDeque<(gpui::EntityId, FocusHandle)>> = const { RefCell::new(VecDeque::new()) };
    /// Focus ring of the theme in use, refreshed by every [`use_interact`] call (the ring is
    /// theme-wide, so any recent value is right) and read by [`track`].
    static RING: Cell<(Hsla, f32)> = const { Cell::new((Hsla { h: 0.0, s: 0.0, l: 0.0, a: 0.0 }, 0.0)) };
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
    let t = crate::theme_of(cx);
    let (c, w) = t.focus_ring();
    RING.with(|r| r.set((crate::hsla(c), w)));
    let st = window.use_keyed_state(id.clone(), cx, |_, cx| Interact {
        focus: Some(cx.focus_handle().tab_stop(true)),
        ..Interact::default()
    });
    if let Some(h) = st.read(cx).focus.clone() {
        HANDLES.with(|q| {
            let mut q = q.borrow_mut();
            q.retain(|(e, _)| *e != st.entity_id());
            if q.len() >= 64 {
                q.pop_front();
            }
            q.push_back((st.entity_id(), h));
        });
    }
    st
}

/// Wire hover/press tracking onto an id'd element.
pub fn track<E>(el: Stateful<E>, state: &Entity<Interact>) -> Stateful<E>
where
    Stateful<E>: StatefulInteractiveElement + InteractiveElement + gpui::ParentElement,
{
    let s1 = state.clone();
    let s2 = state.clone();
    let s3 = state.clone();
    let focus = HANDLES.with(|q| {
        q.borrow()
            .iter()
            .find(|(e, _)| *e == state.entity_id())
            .map(|(_, h)| h.clone())
    });
    let (ring, thick) = RING.with(Cell::get);
    let shape = RING_SHAPE.with(|c| c.borrow().clone());
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
                Some((sh, gap)) if sh.kind() == dodai_core::ShapeKind::Chamfered => {
                    let style = crate::paint::ShapeStyle {
                        fill: None,
                        stroke: Some(ring),
                        stroke_width: thick,
                        glow: None,
                        shadow: None,
                        shadow_dir: (1.0, 1.0),
                        scanlines: false,
                        scanline: ring,
                        surface: None,
                        shimmer: None,
                        elevation: None,
                        lift_layers: None,
                        edge_gap: gap,
                    };
                    gpui::ParentElement::child(
                        base.invisible().focus(|s| s.visible()),
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
                        .filter(|s| s.kind() == dodai_core::ShapeKind::Rounded)
                        .map_or(0.0, |s| s.radius);
                    base.rounded(gpui::px(radius)).focus(move |s| {
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
    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
        s2.update(cx, |s, cx| {
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
        };
        assert_eq!(i(false, false).control_state(false), ControlState::Idle);
        assert_eq!(i(true, false).control_state(false), ControlState::Hovered);
        assert_eq!(i(true, true).control_state(false), ControlState::Pressed);
        assert_eq!(i(true, true).control_state(true), ControlState::Disabled);
    }
}
