//! Hover and select focus shared by the item-based charts: the hovered or selected item pops
//! forward while, once one is selected, the others dim. Both ease with the component's
//! response, like the pie's slices.

use gpui::{App, Entity, Window};

use crate::instruments_motion::{follow_all, Follow};

/// Opacity of the other items while one is selected.
const DIM: f32 = 0.35;

#[derive(Default, Clone, Copy)]
pub(crate) struct FocusState {
    pub hover: Option<usize>,
    pub sel: Option<usize>,
}

/// One item's eased focus: `alpha` to paint with, `lift` in 0..=1 for its pop.
#[derive(Clone, Copy)]
pub(crate) struct ItemFocus {
    pub alpha: f32,
    pub lift: f32,
}

/// The state shared by the items of one chart.
pub(crate) struct Focus {
    state: Entity<FocusState>,
    items: Vec<ItemFocus>,
}

impl Focus {
    /// The eased focus of `n` items. `id` must be unique to the chart.
    pub fn new(w: &mut Window, cx: &mut App, follow: &Follow, n: usize) -> Self {
        let follow = &follow.sub("focus");
        let key = follow.element_id(w, cx);
        let state = w.with_id(key, |w| {
            w.use_keyed_state("focus", cx, |_, _| FocusState::default())
        });
        let s = *state.read(cx);
        let mut target = Vec::with_capacity(n * 2);
        for i in 0..n {
            let picked = s.sel == Some(i);
            let dim = s.sel.is_some() && !picked;
            target.push(if dim { DIM } else { 1.0 });
            target.push(if picked {
                1.0
            } else if s.hover == Some(i) {
                0.65
            } else {
                0.0
            });
        }
        let shown = follow_all(w, cx, follow, &target);
        let items = (0..n)
            .map(|i| ItemFocus {
                alpha: shown.get(2 * i).copied().unwrap_or(1.0).clamp(0.0, 1.0),
                lift: shown.get(2 * i + 1).copied().unwrap_or(0.0).max(0.0),
            })
            .collect();
        Self { state, items }
    }

    pub fn item(&self, i: usize) -> ItemFocus {
        self.items.get(i).copied().unwrap_or(ItemFocus {
            alpha: 1.0,
            lift: 0.0,
        })
    }

    /// Sets the hovered item (for charts that hit-test the pointer themselves).
    pub fn set_hover(state: &Entity<FocusState>, i: Option<usize>, w: &mut Window, cx: &mut App) {
        let moved = state.update(cx, |s, cx| {
            let moved = s.hover != i;
            s.hover = i;
            if moved {
                cx.notify();
            }
            moved
        });
        if moved {
            w.refresh();
        }
    }

    /// Selects item `i`, or clears the selection when it already is.
    pub fn toggle(state: &Entity<FocusState>, i: Option<usize>, w: &mut Window, cx: &mut App) {
        state.update(cx, |s, cx| {
            s.sel = i.filter(|i| s.sel != Some(*i));
            cx.notify();
        });
        w.refresh();
    }

    /// The shared state, for handlers that outlive the build.
    pub fn state(&self) -> Entity<FocusState> {
        self.state.clone()
    }

    /// Hover and click handlers for item `i`'s element.
    pub fn bind<E: gpui::InteractiveElement + gpui::StatefulInteractiveElement>(
        &self,
        el: E,
        i: usize,
    ) -> E {
        let (a, b) = (self.state.clone(), self.state.clone());
        el.on_hover(move |on, w, cx| {
            a.update(cx, |s, cx| {
                let next = if *on {
                    Some(i)
                } else if s.hover == Some(i) {
                    None
                } else {
                    s.hover
                };
                if s.hover != next {
                    s.hover = next;
                    cx.notify();
                }
            });
            w.refresh();
        })
        .on_click(move |_, w, cx| {
            b.update(cx, |s, cx| {
                s.sel = if s.sel == Some(i) { None } else { Some(i) };
                cx.notify();
            });
            w.refresh();
        })
    }
}
