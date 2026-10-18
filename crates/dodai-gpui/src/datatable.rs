//! [`DataTable`]: sortable, filterable, selectable table with resizable columns
//! and virtual scrolling (only the visible rows are built). Keeps its own state.

use crate::controls::Checkbox;
use crate::display::thumb_geometry;
use crate::interact::{track, use_interact};
use crate::list::list_style;
use crate::paint::{shaped, shaped_id, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use gpui::{
    div, px, App, ElementId, InteractiveElement, IntoElement, MouseButton, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::ops::Range;
use std::rc::Rc;

// ───────────────────────────── pure logic ─────────────────────────────

/// Numbers compare as numbers, everything else case-insensitively.
pub fn compare_cells(a: &str, b: &str) -> Ordering {
    match (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
        (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

/// Click on a header: none → ascending → descending → none.
pub fn next_sort(cur: Option<(usize, bool)>, col: usize) -> Option<(usize, bool)> {
    match cur {
        Some((c, true)) if c == col => Some((col, false)),
        Some((c, false)) if c == col => None,
        _ => Some((col, true)),
    }
}

/// Row indices to show: rows with any cell containing `filter` (case-insensitive),
/// ordered by `sort`. Stable, so equal cells keep source order.
pub fn view_indices(
    rows: &[Vec<SharedString>],
    filter: &str,
    sort: Option<(usize, bool)>,
) -> Vec<usize> {
    let q = filter.trim().to_lowercase();
    let mut idx: Vec<usize> = (0..rows.len())
        .filter(|&i| q.is_empty() || rows[i].iter().any(|c| c.to_lowercase().contains(&q)))
        .collect();
    if let Some((col, asc)) = sort {
        let cell = |i: usize| rows[i].get(col).map_or("", |s| s.as_ref());
        idx.sort_by(|&a, &b| {
            let o = compare_cells(cell(a), cell(b));
            if asc {
                o
            } else {
                o.reverse()
            }
        });
    }
    idx
}

/// Largest valid scroll offset (px) for `n` rows of `row_h` in a `view_h` viewport.
pub fn max_scroll(n: usize, row_h: f32, view_h: f32) -> f32 {
    (n as f32 * row_h - view_h).max(0.0)
}

/// Rows to build for a scroll offset: the visible ones plus `overscan` each side.
pub fn visible_range(
    n: usize,
    row_h: f32,
    view_h: f32,
    scroll: f32,
    overscan: usize,
) -> Range<usize> {
    if n == 0 || row_h <= 0.0 {
        return 0..0;
    }
    let first = (scroll.max(0.0) / row_h).floor() as usize;
    let last = ((scroll.max(0.0) + view_h) / row_h).ceil() as usize;
    first.saturating_sub(overscan).min(n)..(last + overscan).min(n)
}

/// Column width after dragging its edge by `dx`.
pub fn resized(w0: f32, dx: f32, min: f32) -> f32 {
    (w0 + dx).max(min)
}

// ───────────────────────────── component ─────────────────────────────

/// One column of a [`DataTable`].
#[derive(Clone)]
pub struct Column {
    title: SharedString,
    width: f32,
    sortable: bool,
    right: bool,
}

impl Column {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            width: 140.0,
            sortable: true,
            right: false,
        }
    }
    /// Initial width in px (the last column also stretches to fill).
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    pub fn sortable(mut self, on: bool) -> Self {
        self.sortable = on;
        self
    }
    /// Right-align (numbers).
    pub fn numeric(mut self, on: bool) -> Self {
        self.right = on;
        self
    }
}

struct DtState {
    sort: Option<(usize, bool)>,
    selected: BTreeSet<usize>,
    widths: Vec<f32>,
    scroll: f32,
    /// Column being resized: `(column, pointer x at start, width at start)`.
    resize: Option<(usize, f32, f32)>,
}

type SelectionFn = Rc<dyn Fn(&[usize], &mut Window, &mut App)>;

const MIN_COL: f32 = 48.0;

/// Table with sorting (click a header), filtering, row selection, draggable column
/// edges and windowed rendering for long lists.
#[derive(IntoElement)]
pub struct DataTable {
    id: ElementId,
    columns: Vec<Column>,
    rows: Rc<Vec<Vec<SharedString>>>,
    filter: SharedString,
    selectable: bool,
    height: f32,
    on_selection: Option<SelectionFn>,
}

impl DataTable {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            columns: Vec::new(),
            rows: Rc::new(Vec::new()),
            filter: "".into(),
            selectable: true,
            height: 280.0,
            on_selection: None,
        }
    }
    pub fn columns(mut self, c: impl IntoIterator<Item = Column>) -> Self {
        self.columns = c.into_iter().collect();
        self
    }
    pub fn rows<S: Into<SharedString>, R: IntoIterator<Item = S>>(
        mut self,
        rows: impl IntoIterator<Item = R>,
    ) -> Self {
        self.rows = Rc::new(
            rows.into_iter()
                .map(|r| r.into_iter().map(Into::into).collect())
                .collect(),
        );
        self
    }
    /// Share an existing row buffer (cheap for big tables).
    pub fn rows_shared(mut self, rows: Rc<Vec<Vec<SharedString>>>) -> Self {
        self.rows = rows;
        self
    }
    /// Show only rows containing this text.
    pub fn filter(mut self, f: impl Into<SharedString>) -> Self {
        self.filter = f.into();
        self
    }
    /// Checkbox column with select-all (default on).
    pub fn selectable(mut self, on: bool) -> Self {
        self.selectable = on;
        self
    }
    /// Body viewport height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// Called with the selected source-row indices.
    pub fn on_selection(mut self, f: impl Fn(&[usize], &mut Window, &mut App) + 'static) -> Self {
        self.on_selection = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for DataTable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let p = &t.palette;
        let init: Vec<f32> = self.columns.iter().map(|c| c.width).collect();
        let st = window.use_keyed_state(self.id.clone(), cx, move |_, _| DtState {
            sort: None,
            selected: BTreeSet::new(),
            widths: init,
            scroll: 0.0,
            resize: None,
        });
        let (sort, selected, widths, resize) = {
            let s = st.read(cx);
            (s.sort, s.selected.clone(), s.widths.clone(), s.resize)
        };
        let view = view_indices(&self.rows, &self.filter, sort);
        let n = view.len();
        let row_h = t.shape.control_height;
        let head_h = t.shape.control_height * 0.85;
        let max = max_scroll(n, row_h, self.height);
        let scroll = st.read(cx).scroll.min(max);
        let range = visible_range(n, row_h, self.height, scroll, 2);
        let w_of = |i: usize| widths.get(i).copied().unwrap_or(140.0);
        let last = self.columns.len().saturating_sub(1);
        let check_w = if self.selectable { 40.0 } else { 0.0 };

        let notify_selection = {
            let f = self.on_selection.clone();
            move |sel: &BTreeSet<usize>, w: &mut Window, cx: &mut App| {
                if let Some(f) = &f {
                    let v: Vec<usize> = sel.iter().copied().collect();
                    f(&v, w, cx);
                }
            }
        };

        // header
        let all_on = n > 0 && view.iter().all(|i| selected.contains(i));
        let mut head = div()
            .flex()
            .items_center()
            .h(px(head_h))
            .px(px(t.spacing.md))
            .bg(hsla(p.surface_raised))
            .text_size(px(t.typography.size_sm))
            .text_color(hsla(p.text_muted));
        if self.selectable {
            let (st2, view2, notify) = (st.clone(), view.clone(), notify_selection.clone());
            head = head.child(div().w(px(check_w)).flex_none().child(
                Checkbox::new((self.id.clone(), "all"), all_on).on_change(move |on, w, cx| {
                    let mut sel = st2.read(cx).selected.clone();
                    for i in &view2 {
                        if on {
                            sel.insert(*i);
                        } else {
                            sel.remove(i);
                        }
                    }
                    st2.update(cx, |s, cx| {
                        s.selected = sel.clone();
                        cx.notify();
                    });
                    notify(&sel, w, cx);
                }),
            ));
        }
        for (i, c) in self.columns.iter().enumerate() {
            let mut label = format_label(&t.typography, &c.title).to_string();
            if let Some((sc, asc)) = sort {
                if sc == i {
                    label.push_str(if asc { " ▲" } else { " ▼" });
                }
            }
            let mut h = div()
                .id((self.id.clone(), format!("h{i}")))
                .relative()
                .h_full()
                .flex()
                .items_center()
                .overflow_hidden()
                .whitespace_nowrap()
                .px(px(t.spacing.xs));
            h = if i == last {
                h.flex_1().min_w(px(w_of(i)))
            } else {
                h.w(px(w_of(i))).flex_none()
            };
            if c.right {
                h = h.justify_end();
            }
            if c.sortable {
                let st2 = st.clone();
                h = h.cursor_pointer().on_click(move |_, _, cx| {
                    st2.update(cx, |s, cx| {
                        s.sort = next_sort(s.sort, i);
                        s.scroll = 0.0;
                        cx.notify();
                    })
                });
            }
            h = h.child(label);
            let st2 = st.clone();
            let w0 = w_of(i);
            h = h.child(
                div()
                    .id((self.id.clone(), format!("grip{i}")))
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .right_0()
                    .w(px(6.0))
                    .cursor_col_resize()
                    .on_mouse_down(MouseButton::Left, move |e, _, cx| {
                        let x = f32::from(e.position.x);
                        st2.update(cx, |s, cx| {
                            s.resize = Some((i, x, w0));
                            cx.notify();
                        });
                        cx.stop_propagation();
                    })
                    .child(
                        div()
                            .absolute()
                            .right(px(2.0))
                            .top(px(5.0))
                            .bottom(px(5.0))
                            .w(px(1.0))
                            .bg(hsla(if resize.is_some_and(|r| r.0 == i) {
                                p.primary
                            } else {
                                p.border_strong
                            })),
                    ),
            );
            head = head.child(h);
        }

        // body (windowed)
        let mut body = div()
            .id((self.id.clone(), "body"))
            .relative()
            .h(px(self.height))
            .overflow_hidden()
            .on_scroll_wheel({
                let st2 = st.clone();
                move |e, window, cx| {
                    let dy = f32::from(e.delta.pixel_delta(px(row_h)).y);
                    st2.update(cx, |s, cx| {
                        s.scroll = (s.scroll - dy).clamp(0.0, max);
                        cx.notify();
                    });
                    window.refresh();
                }
            });
        let mut rows = div()
            .flex()
            .flex_col()
            .mt(px(range.start as f32 * row_h - scroll));
        for vi in range.clone() {
            let ri = view[vi];
            let row = &self.rows[ri];
            let id: ElementId = (self.id.clone(), format!("r{ri}")).into();
            let hs = use_interact(&id, window, cx);
            let hovered = hs.read(cx).hovered;
            let is_sel = selected.contains(&ri);
            let (style, _, marker) = list_style(&t, is_sel, hovered);
            let mut r = track(
                shaped_id(
                    id,
                    &t.shape,
                    ShapeStyle {
                        stroke: None,
                        ..style
                    },
                ),
                &hs,
            )
            .flex()
            .items_center()
            .h(px(row_h))
            .flex_none()
            .px(px(t.spacing.md))
            .border_t_1()
            .border_color(hsla(p.border))
            .text_color(hsla(p.text))
            .cursor_pointer();
            if self.selectable {
                let (st2, notify) = (st.clone(), notify_selection.clone());
                r = r
                    .on_click(move |_, w, cx| {
                        let mut sel = st2.read(cx).selected.clone();
                        if !sel.remove(&ri) {
                            sel.insert(ri);
                        }
                        st2.update(cx, |s, cx| {
                            s.selected = sel.clone();
                            cx.notify();
                        });
                        notify(&sel, w, cx);
                    })
                    .child(
                        div()
                            .w(px(check_w))
                            .flex_none()
                            .text_color(hsla(p.primary))
                            .child(if is_sel { "✓" } else { "" }),
                    );
            }
            for (i, c) in self.columns.iter().enumerate() {
                let mut cell = div()
                    .h_full()
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .px(px(t.spacing.xs))
                    .child(row.get(i).cloned().unwrap_or_default());
                cell = if i == last {
                    cell.flex_1().min_w(px(w_of(i)))
                } else {
                    cell.w(px(w_of(i))).flex_none()
                };
                if c.right {
                    cell = cell.justify_end();
                }
                r = r.child(cell);
            }
            if let Some(m) = marker {
                r = r.child(
                    div()
                        .absolute()
                        .left_0()
                        .top_0()
                        .bottom_0()
                        .w(px(t.shape.border_width.max(2.0)))
                        .bg(m),
                );
            }
            rows = rows.child(r);
        }
        body = body.child(rows);
        if n == 0 {
            body = body.child(
                div()
                    .p(px(t.spacing.lg))
                    .text_color(hsla(p.text_muted))
                    .child("No matching rows"),
            );
        }
        let (len, pos) = thumb_geometry(self.height - 4.0, self.height, max, scroll);
        if len > 0.0 {
            body = body.child(
                div()
                    .absolute()
                    .top(px(2.0 + pos))
                    .right(px(2.0))
                    .w(px(5.0))
                    .h(px(len))
                    .bg(hsla(p.border_strong)),
            );
        }

        let footer = div()
            .flex()
            .justify_between()
            .px(px(t.spacing.md))
            .h(px(head_h))
            .items_center()
            .bg(hsla(p.surface_raised))
            .text_size(px(t.typography.size_xs))
            .text_color(hsla(p.text_muted))
            .child(format!("{} of {} rows", n, self.rows.len()))
            .child(if self.selectable {
                format!("{} selected", selected.len())
            } else {
                String::new()
            });

        let outer = ShapeStyle::from_tokens(&t, Some(p.surface), Some(p.border), false, false);
        let (s1, s2, s3) = (st.clone(), st.clone(), st);
        shaped(&t.shape, outer)
            .w_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .on_mouse_move(move |e, _, cx| {
                let Some((col, x0, w0)) = s1.read(cx).resize else {
                    return;
                };
                let w = resized(w0, f32::from(e.position.x) - x0, MIN_COL);
                s1.update(cx, |s, cx| {
                    if let Some(slot) = s.widths.get_mut(col) {
                        *slot = w;
                    }
                    cx.notify();
                });
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                s2.update(cx, |s, cx| {
                    if s.resize.take().is_some() {
                        cx.notify();
                    }
                })
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                s3.update(cx, |s, cx| {
                    if s.resize.take().is_some() {
                        cx.notify();
                    }
                })
            })
            .child(head)
            .child(body)
            .child(footer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<Vec<SharedString>> {
        [["b", "10"], ["a", "9"], ["C", "100"], ["a", "1"]]
            .iter()
            .map(|r| r.iter().map(|s| SharedString::from(*s)).collect())
            .collect()
    }

    #[test]
    fn numbers_sort_numerically_text_case_insensitively() {
        assert_eq!(compare_cells("9", "10"), Ordering::Less);
        assert_eq!(compare_cells("a", "B"), Ordering::Less);
        assert_eq!(compare_cells("x", "x"), Ordering::Equal);
    }

    #[test]
    fn sort_cycle() {
        let s = next_sort(None, 2);
        assert_eq!(s, Some((2, true)));
        let s = next_sort(s, 2);
        assert_eq!(s, Some((2, false)));
        assert_eq!(next_sort(s, 2), None);
        assert_eq!(next_sort(Some((2, false)), 1), Some((1, true)));
    }

    #[test]
    fn view_filters_then_sorts_stably() {
        let r = rows();
        assert_eq!(view_indices(&r, "", None), vec![0, 1, 2, 3]);
        assert_eq!(view_indices(&r, " A ", None), vec![1, 3]);
        assert_eq!(view_indices(&r, "", Some((1, true))), vec![3, 1, 0, 2]);
        assert_eq!(view_indices(&r, "", Some((1, false))), vec![2, 0, 1, 3]);
        assert_eq!(view_indices(&r, "", Some((0, true))), vec![1, 3, 0, 2]); // a,a keep order
        assert!(view_indices(&r, "zzz", None).is_empty());
    }

    #[test]
    fn window_covers_viewport_and_clamps() {
        assert_eq!(visible_range(0, 30.0, 300.0, 0.0, 2), 0..0);
        assert_eq!(visible_range(1000, 30.0, 300.0, 0.0, 2), 0..12);
        assert_eq!(visible_range(1000, 30.0, 300.0, 3000.0, 2), 98..112);
        assert_eq!(visible_range(5, 30.0, 300.0, 0.0, 2), 0..5);
        assert_eq!(max_scroll(5, 30.0, 300.0), 0.0);
        assert_eq!(max_scroll(100, 30.0, 300.0), 2700.0);
    }

    #[test]
    fn resize_has_a_floor() {
        assert_eq!(resized(100.0, 30.0, 48.0), 130.0);
        assert_eq!(resized(100.0, -90.0, 48.0), 48.0);
    }
}
