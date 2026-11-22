//! Hover and select focus shared by the item-based charts: the hovered or selected item pops
//! forward while, once one is selected, the others dim. Both ease with the component's
//! response, like the pie's slices.

use gpui::{App, Bounds, Entity, Pixels, Window};
use std::cell::Cell;
use std::rc::Rc;

use dodai_ui_gpui::follow::{follow_all, FocusFn, Follow};

/// Opacity of the other items while one is selected.
const DIM: f32 = 0.35;

/// Opacity of the other items while the pointer rests on one.
const HOVER_DIM: f32 = 0.6;

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

/// The handlers' side of a chart's focus: changes the shared state and tells the chart's
/// `on_focus` callback when the focused item (hovered, else selected) changes.
#[derive(Clone)]
pub(crate) struct Gate {
    state: Entity<FocusState>,
    on_focus: Option<FocusFn>,
}

impl Gate {
    fn change(&self, w: &mut Window, cx: &mut App, f: impl FnOnce(&mut FocusState)) {
        let (before, after) = self.state.update(cx, |s, cx| {
            let before = s.hover.or(s.sel);
            let old = *s;
            f(s);
            if s.hover != old.hover || s.sel != old.sel {
                cx.notify();
            }
            (before, s.hover.or(s.sel))
        });
        w.refresh();
        if before != after {
            if let Some(cb) = &self.on_focus {
                cb(after, w, cx);
            }
        }
    }

    /// Sets the hovered item.
    pub fn hover(&self, i: Option<usize>, w: &mut Window, cx: &mut App) {
        self.change(w, cx, |s| s.hover = i);
    }

    /// Selects item `i`, or clears the selection when it already is.
    pub fn toggle(&self, i: Option<usize>, w: &mut Window, cx: &mut App) {
        self.change(w, cx, |s| s.sel = i.filter(|i| s.sel != Some(*i)));
    }

    /// The pointer left item `i`: clears the hover only if it is still on `i`.
    fn leave(&self, i: usize, w: &mut Window, cx: &mut App) {
        self.change(w, cx, |s| {
            if s.hover == Some(i) {
                s.hover = None;
            }
        });
    }
}

/// The state shared by the items of one chart.
pub(crate) struct Focus {
    state: Entity<FocusState>,
    items: Vec<ItemFocus>,
    /// `None` when the chart is not interactive: nothing is hit-tested or selected.
    gate: Option<Gate>,
}

impl Focus {
    /// The eased focus of `n` items. `id` must be unique to the chart.
    pub fn new(w: &mut Window, cx: &mut App, follow: &Follow, n: usize) -> Self {
        // Hover and selection answer the pointer: they ease even while Animate is off.
        let follow = &follow.sub("focus").answering_input();
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
        let gate = follow.interactive().then(|| Gate {
            state: state.clone(),
            on_focus: follow.on_focus(),
        });
        Self { state, items, gate }
    }

    pub fn item(&self, i: usize) -> ItemFocus {
        self.items.get(i).copied().unwrap_or(ItemFocus {
            alpha: 1.0,
            lift: 0.0,
        })
    }

    /// Opacity for item `i` of a chart made of elements: its eased dimming, and fainter still
    /// while the pointer rests on another item.
    pub fn opacity(&self, i: usize, cx: &App) -> f32 {
        let other = self.shown(cx).is_some_and(|k| k != i);
        self.item(i).alpha * if other { HOVER_DIM } else { 1.0 }
    }

    /// The handlers' side of the focus, or `None` when the chart is not interactive.
    pub fn gate(&self) -> Option<Gate> {
        self.gate.clone()
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
        let Some(gate) = self.gate() else {
            return el;
        };
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
        let (g_move, g_click, g_leave) = (gate.clone(), gate.clone(), gate);
        el.on_mouse_move(move |e, w, cx| g_move.hover(at_move(e.position), w, cx))
            .on_hover(move |over, w, cx| {
                if !*over {
                    g_leave.hover(None, w, cx);
                }
            })
            .on_click(move |e, w, cx| g_click.toggle(at_click(e.position()), w, cx))
    }

    /// Hover and click handlers for item `i`'s element.
    pub fn bind<E: gpui::InteractiveElement + gpui::StatefulInteractiveElement>(
        &self,
        el: E,
        i: usize,
    ) -> E {
        let Some(gate) = self.gate() else {
            return el;
        };
        let (a, b) = (gate.clone(), gate);
        el.on_hover(move |on, w, cx| {
            if *on {
                a.hover(Some(i), w, cx);
            } else {
                a.leave(i, w, cx);
            }
        })
        .on_click(move |_, w, cx| b.toggle(Some(i), w, cx))
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
pub(crate) fn readout(t: &dodai_ui_core::Theme, text: Option<String>, hint: &str) -> gpui::Div {
    use gpui::{div, px, ParentElement as _, Styled as _};
    let line = (t.typography.size_xs * 1.5).ceil();
    let (text, col) = match text {
        Some(s) => (s, t.palette.text),
        None => (hint.to_string(), t.palette.text_muted),
    };
    crate::gpui::common::label_style(t, div())
        .h(px(line))
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .text_color(dodai_ui_gpui::hsla(col))
        .child(text)
}
