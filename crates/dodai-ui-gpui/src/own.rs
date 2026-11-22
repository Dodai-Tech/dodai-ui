//! A value a component can run on its own: controlled or not.
//!
//! A control remembers what the user last chose and follows the value its caller passes in as
//! soon as that changes. So `Tabs::selected(2)` with no state in the parent still switches on
//! click, and a parent that does keep state stays the source of truth.

use gpui::{App, ElementId, Entity, Window};

/// The caller's value and the one currently shown.
pub(crate) struct Own<T: 'static> {
    state: Entity<(T, T)>,
    shown: T,
}

impl<T: PartialEq + Clone + 'static> Own<T> {
    /// Follow `given`, keeping the user's choice until `given` changes.
    pub(crate) fn new(
        window: &mut Window,
        cx: &mut App,
        id: impl Into<ElementId>,
        given: T,
    ) -> Self {
        let state = window.use_keyed_state((id.into(), "own"), cx, |_, _| {
            (given.clone(), given.clone())
        });
        let (last, shown) = state.read(cx).clone();
        let shown = if given == last {
            shown
        } else {
            state.update(cx, |s, _| *s = (given.clone(), given.clone()));
            given
        };
        Self { state, shown }
    }

    /// The value to draw.
    pub(crate) fn get(&self) -> T {
        self.shown.clone()
    }

    /// A setter to move into click handlers.
    pub(crate) fn setter(&self) -> Entity<(T, T)> {
        self.state.clone()
    }
}

/// Record a user's choice.
pub(crate) fn choose<T: 'static>(state: &Entity<(T, T)>, cx: &mut App, v: T) {
    state.update(cx, |s, _| s.1 = v);
}
