//! Hover / pressed tracking shared by interactive components.

use gpui::{
    App, ClickEvent, ElementId, Entity, InteractiveElement, MouseButton, Stateful,
    StatefulInteractiveElement, Window,
};
use legendary_core::ControlState;

/// Pointer state of one control (kept in gpui keyed element state).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Interact {
    /// Pointer is over the control.
    pub hovered: bool,
    /// Primary button is held down on the control.
    pub pressed: bool,
}

impl Interact {
    /// Map pointer state (and `disabled`) to a token [`ControlState`].
    pub fn control_state(self, disabled: bool) -> ControlState {
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
    window.use_keyed_state(id.clone(), cx, |_, _| Interact::default())
}

/// Wire hover/press tracking onto an id'd element.
pub fn track<E>(el: Stateful<E>, state: &Entity<Interact>) -> Stateful<E>
where
    Stateful<E>: StatefulInteractiveElement + InteractiveElement,
{
    let s1 = state.clone();
    let s2 = state.clone();
    let s3 = state.clone();
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
    .on_mouse_up(MouseButton::Left, move |_, _, cx| {
        s3.update(cx, |s, cx| {
            if s.pressed {
                s.pressed = false;
                cx.notify();
            }
        });
    })
}

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
        };
        assert_eq!(i(false, false).control_state(false), ControlState::Idle);
        assert_eq!(i(true, false).control_state(false), ControlState::Hovered);
        assert_eq!(i(true, true).control_state(false), ControlState::Pressed);
        assert_eq!(i(true, true).control_state(true), ControlState::Disabled);
    }
}
