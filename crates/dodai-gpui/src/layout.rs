//! [`TreeView`], [`SplitPane`] and [`PageGrid`].

use crate::interact::{track, use_interact};
use crate::list::list_style;
use crate::paint::{shaped, shaped_id};
use crate::text::first_family;
use crate::{hsla, theme_of};
use gpui::{
    canvas, div, px, relative, AnyElement, App, Bounds, ElementId, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ─────────────────────────────── TreeView ───────────────────────────────

/// Node of a [`TreeView`]. `expanded` is owned by the caller (controlled).
#[derive(Clone, Debug)]
pub struct TreeItem {
    pub label: SharedString,
    pub expanded: bool,
    pub children: Vec<TreeItem>,
}

impl TreeItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            expanded: false,
            children: Vec::new(),
        }
    }
    pub fn expanded(mut self, e: bool) -> Self {
        self.expanded = e;
        self
    }
    pub fn child(mut self, c: TreeItem) -> Self {
        self.children.push(c);
        self
    }
}

/// One visible row of a flattened tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeRow {
    pub path: Vec<usize>,
    pub depth: usize,
    pub label: SharedString,
    pub expandable: bool,
    pub expanded: bool,
}

/// Flatten the expanded parts of a tree depth-first.
pub fn visible_rows(items: &[TreeItem]) -> Vec<TreeRow> {
    fn walk(items: &[TreeItem], path: &mut Vec<usize>, out: &mut Vec<TreeRow>) {
        for (i, it) in items.iter().enumerate() {
            path.push(i);
            out.push(TreeRow {
                path: path.clone(),
                depth: path.len() - 1,
                label: it.label.clone(),
                expandable: !it.children.is_empty(),
                expanded: it.expanded,
            });
            if it.expanded {
                walk(&it.children, path, out);
            }
            path.pop();
        }
    }
    let mut out = Vec::new();
    walk(items, &mut Vec::new(), &mut out);
    out
}

type PathFn = Rc<dyn Fn(&[usize], &mut Window, &mut App)>;

/// Hierarchical list with expand/collapse.
#[derive(IntoElement)]
pub struct TreeView {
    id: ElementId,
    items: Vec<TreeItem>,
    selected: Option<Vec<usize>>,
    on_toggle: Option<PathFn>,
    on_select: Option<PathFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(TreeView);

impl TreeView {
    pub fn new(id: impl Into<ElementId>, items: Vec<TreeItem>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            items,
            selected: None,
            on_toggle: None,
            on_select: None,
        }
    }
    pub fn selected(mut self, path: Option<Vec<usize>>) -> Self {
        self.selected = path;
        self
    }
    crate::callback!(
        /// Called with the node path when the chevron is clicked.
        on_toggle, &[usize]
    );
    crate::callback!(on_select, &[usize]);
}

impl TreeView {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let mut col = div()
            .flex()
            .flex_col()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md));
        for row in visible_rows(&self.items) {
            let key = row
                .path
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join("-");
            let id: ElementId = (self.id.clone(), key).into();
            let st = use_interact(&id, window, cx);
            let hovered = st.read(cx).hovered;
            let sel = self.selected.as_deref() == Some(row.path.as_slice());
            let (style, fg, marker) = list_style(&t, sel, hovered);
            let _ = fg;
            let chev = if !row.expandable {
                "  "
            } else if row.expanded {
                "▾"
            } else {
                "▸"
            };
            let toggle_path = row.path.clone();
            let toggle = self.on_toggle.clone();
            let mut chevron = div()
                .id((id.clone(), "chev"))
                .w(px(16.0))
                .text_color(hsla(t.palette.text_muted))
                .child(chev);
            if let (Some(f), true) = (toggle, row.expandable) {
                chevron = chevron.cursor_pointer().on_click(move |_, w, cx| {
                    cx.stop_propagation();
                    f(&toggle_path, w, cx)
                });
            }
            let select_path = row.path.clone();
            let mut r = track(shaped_id(id, &t.shape, style), &st)
                .relative()
                .flex()
                .items_center()
                .gap(px(t.spacing.xs))
                .h(px(t.shape.control_height * 0.8))
                .pl(px(t.spacing.sm + row.depth as f32 * 16.0))
                .text_color(hsla(t.palette.text))
                .child(chevron)
                .child(row.label.clone());
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
            if let Some(f) = self.on_select.clone() {
                r = r
                    .cursor_pointer()
                    .on_click(move |_, w, cx| f(&select_path, w, cx));
            }
            col = col.child(r);
        }
        col
    }
}

crate::entry_render!(TreeView);

// ─────────────────────────────── SplitPane ───────────────────────────────

/// Clamp a split ratio so both panes keep at least `min` of the space.
pub fn clamp_ratio(r: f32, min: f32) -> f32 {
    r.clamp(min, 1.0 - min)
}

#[derive(Default)]
struct SplitState {
    bounds: Bounds<Pixels>,
    dragging: bool,
}

type RatioFn = Rc<dyn Fn(f32, &mut Window, &mut App)>;

/// Two panes with a draggable divider (controlled ratio of the first pane).
#[derive(IntoElement)]
pub struct SplitPane {
    id: ElementId,
    ratio: f32,
    height: f32,
    first: Option<AnyElement>,
    second: Option<AnyElement>,
    on_change: Option<RatioFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(SplitPane);

impl SplitPane {
    pub fn new(id: impl Into<ElementId>, ratio: f32) -> Self {
        Self {
            corners: None,
            id: id.into(),
            ratio,
            height: 200.0,
            first: None,
            second: None,
            on_change: None,
        }
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    pub fn first(mut self, e: impl IntoElement) -> Self {
        self.first = Some(e.into_any_element());
        self
    }
    pub fn second(mut self, e: impl IntoElement) -> Self {
        self.second = Some(e.into_any_element());
        self
    }
    crate::callback!(on_change, f32);
}

impl SplitPane {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let st = window.use_keyed_state(self.id.clone(), cx, |_, _| SplitState::default());
        let ratio = clamp_ratio(self.ratio, 0.1);
        let rec = st.clone();
        let (down, mv, up, up2) = (st.clone(), st.clone(), st.clone(), st.clone());
        let f = self.on_change.clone();
        let apply = move |x: Pixels, w: &mut Window, cx: &mut App| {
            let b = st.read(cx).bounds;
            let width: f32 = b.size.width.into();
            if let (Some(f), true) = (&f, width > 0.0) {
                let dx: f32 = (x - b.origin.x).into();
                let r = dx / width;
                f(clamp_ratio(r, 0.1), w, cx);
            }
        };
        let apply2 = apply.clone();
        let handle = shaped(
            &t.shape,
            crate::paint::ShapeStyle::from_tokens(
                &t,
                Some(t.palette.border_strong),
                None,
                false,
                false,
            ),
        )
        .w(px(6.0))
        .h_full()
        .flex_none()
        .cursor_col_resize();
        let pane = |e: Option<AnyElement>| {
            div()
                .h_full()
                .overflow_hidden()
                .p(px(t.spacing.sm))
                .children(e)
        };
        div()
            .id(self.id)
            .relative()
            .flex()
            .w_full()
            .h(px(self.height))
            .child(
                canvas(
                    |_, _, _| (),
                    move |b, _, _, cx| rec.update(cx, |s, _| s.bounds = b),
                )
                .absolute()
                .size_full(),
            )
            .child(pane(self.first).w(relative(ratio)))
            .child(handle.on_mouse_down(MouseButton::Left, move |e, w, cx| {
                down.update(cx, |s, _| s.dragging = true);
                apply(e.position.x, w, cx);
            }))
            .child(pane(self.second).flex_1())
            .on_mouse_move(move |e, w, cx| {
                if mv.read(cx).dragging {
                    apply2(e.position.x, w, cx);
                }
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                up.update(cx, |s, _| s.dragging = false)
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                up2.update(cx, |s, _| s.dragging = false)
            })
    }
}

crate::entry_render!(SplitPane);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_respects_expansion() {
        let tree = vec![
            TreeItem::new("a")
                .expanded(true)
                .child(TreeItem::new("a1"))
                .child(TreeItem::new("a2").child(TreeItem::new("deep"))),
            TreeItem::new("b").child(TreeItem::new("hidden")),
        ];
        let rows = visible_rows(&tree);
        let labels: Vec<_> = rows.iter().map(|r| r.label.as_ref()).collect();
        assert_eq!(labels, ["a", "a1", "a2", "b"]);
        assert_eq!(rows[1].depth, 1);
        assert_eq!(rows[2].path, vec![0, 1]);
        assert!(rows[2].expandable && !rows[1].expandable);
    }

    #[test]
    fn ratio_clamps() {
        assert_eq!(clamp_ratio(0.0, 0.1), 0.1);
        assert_eq!(clamp_ratio(2.0, 0.1), 0.9);
        assert_eq!(clamp_ratio(0.4, 0.1), 0.4);
    }
}

// ───────────────────────────────── PageGrid ─────────────────────────────────

/// Width in px of a cell spanning
/// `span` of `columns` columns inside `inner` px with `gap` between columns.
fn grid_span_width(inner: f32, columns: u32, gap: f32, span: u32) -> f32 {
    let columns = columns.max(1);
    let span = span.clamp(1, columns);
    let col = (inner - gap * (columns - 1) as f32) / columns as f32;
    col * span as f32 + gap * (span - 1) as f32
}

/// The design's page grid (columns, gap and outer margins at the viewport width) laid out as
/// wrapping rows of cells that each span a number of columns.
///
/// The width is the window's unless set with [`PageGrid::width`] (use it when the page sits
/// beside a sidebar).
#[derive(IntoElement)]
pub struct PageGrid {
    width: Option<f32>,
    cells: Vec<(u32, AnyElement)>,
}

impl Default for PageGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl PageGrid {
    pub fn new() -> Self {
        Self {
            width: None,
            cells: Vec::new(),
        }
    }
    /// The width the grid lays out in, instead of the window's.
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    /// A cell spanning `span` columns (clamped to the grid's column count).
    pub fn cell(mut self, span: u32, el: impl IntoElement) -> Self {
        self.cells.push((span, el.into_any_element()));
        self
    }
}

impl gpui::RenderOnce for PageGrid {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let viewport = self
            .width
            .unwrap_or_else(|| window.viewport_size().width.into());
        let g = t.page_grid(viewport);
        let page = g.max_width.map_or(viewport, |m| viewport.min(m));
        let inner = (page - 2.0 * g.margin).max(0.0);
        let mut row = div()
            .flex()
            .flex_wrap()
            .gap(px(g.gap))
            .w(px(inner))
            .mx_auto();
        for (span, el) in self.cells {
            row = row.child(
                div()
                    .w(px(grid_span_width(inner, g.columns, g.gap, span)))
                    .child(el),
            );
        }
        row
    }
}

#[cfg(test)]
mod grid_tests {
    use super::grid_span_width;

    #[test]
    fn spans_add_up_to_the_row() {
        let (inner, cols, gap) = (1000.0, 12, 20.0);
        let half = grid_span_width(inner, cols, gap, 6);
        assert!((half * 2.0 + gap - inner).abs() < 0.01);
        assert!((grid_span_width(inner, cols, gap, 12) - inner).abs() < 0.01);
        assert_eq!(
            grid_span_width(inner, cols, gap, 99),
            grid_span_width(inner, cols, gap, 12)
        );
    }
}
