//! Hover and select focus shared by the item-based charts: the hovered or selected item pops
//! forward while, once one is selected, the others dim. Both ease with the component's
//! response, like the pie's slices.

use gpui::{App, Bounds, Entity, Pixels, Window};
use std::cell::Cell;
use std::rc::Rc;

use dodai_gpui::follow::{follow_all, Follow};

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

    /// The item under the pointer, else the selected one.
    pub fn shown(&self, cx: &App) -> Option<usize> {
        let s = self.state.read(cx);
        s.hover.or(s.sel)
    }

    /// Makes `el` (which holds the chart's canvas) hit-test the pointer: `hit(x, y, w, h)`
    /// gets the pointer in the canvas's own pixels and names the item there. Hover follows the
    /// pointer, a click selects (again to clear), leaving clears the hover. The canvas must
    /// store its bounds in `bounds` while painting.
    pub fn pointer(
        &self,
        el: gpui::Stateful<gpui::Div>,
        bounds: &Rc<Cell<Bounds<Pixels>>>,
        hit: impl Fn(f32, f32, f32, f32) -> Option<usize> + 'static,
    ) -> gpui::Stateful<gpui::Div> {
        use gpui::{InteractiveElement as _, StatefulInteractiveElement as _};
        let hit = Rc::new(hit);
        let at = {
            let b = bounds.clone();
            move |p: gpui::Point<Pixels>| {
                let b = b.get();
                hit(
                    (p.x - b.origin.x).into(),
                    (p.y - b.origin.y).into(),
                    b.size.width.into(),
                    b.size.height.into(),
                )
            }
        };
        let (at_move, at_click) = (at.clone(), at);
        let (st_move, st_click, st_leave) = (self.state(), self.state(), self.state());
        el.on_mouse_move(move |e, w, cx| Focus::set_hover(&st_move, at_move(e.position), w, cx))
            .on_hover(move |over, w, cx| {
                if !*over {
                    Focus::set_hover(&st_leave, None, w, cx);
                }
            })
            .on_click(move |e, w, cx| Focus::toggle(&st_click, at_click(e.position()), w, cx))
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

/// Hit test for `n` equal columns across a canvas: the column under the pointer.
pub(crate) fn columns(n: usize) -> impl Fn(f32, f32, f32, f32) -> Option<usize> + 'static {
    move |x, _, w, _| slot(x, w, n)
}

/// Which of `n` equal slots along `0..len` holds `at`.
fn slot(at: f32, len: f32, n: usize) -> Option<usize> {
    (n > 0 && len > 0.0 && (0.0..len).contains(&at))
        .then(|| ((at / len * n as f32) as usize).min(n - 1))
}

/// A one-line readout under a chart: the focused item's text, else `hint`. Always one line
/// tall, so hovering never moves the layout.
pub(crate) fn readout(t: &dodai_core::Theme, text: Option<String>, hint: &str) -> gpui::Div {
    use gpui::{div, px, ParentElement as _, Styled as _};
    let line = (t.typography.size_xs * 1.5).ceil();
    let (text, col) = match text {
        Some(s) => (s, t.palette.text),
        None => (hint.to_string(), t.palette.text_muted),
    };
    crate::common::label_style(t, div())
        .h(px(line))
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .text_color(dodai_gpui::hsla(col))
        .child(text)
}
