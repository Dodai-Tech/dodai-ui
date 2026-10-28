//! [`TreeView`], [`TreeNavigation`] and [`SplitPane`].

use crate::interact::{track, use_interact};
use crate::list::list_style;
use crate::paint::{shaped, shaped_id};
use crate::text::first_family;
use crate::{hsla, theme_of};
use gpui::{
    canvas, div, px, relative, AnyElement, App, Bounds, ElementId, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::collections::HashMap;
use std::rc::Rc;

// ─────────────────────────────── TreeView ───────────────────────────────

/// Node of a [`TreeView`]. `expanded` is owned by the caller (controlled).
#[derive(Clone, Debug)]
pub struct TreeItem {
    /// Text shown for the node.
    pub label: SharedString,
    /// Whether its children are shown.
    pub expanded: bool,
    /// Child nodes.
    pub children: Vec<TreeItem>,
}

impl TreeItem {
    /// Create a new [`TreeItem`].
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            expanded: false,
            children: Vec::new(),
        }
    }
    /// Whether the item starts expanded.
    pub fn expanded(mut self, e: bool) -> Self {
        self.expanded = e;
        self
    }
    /// Add a child.
    pub fn child(mut self, c: TreeItem) -> Self {
        self.children.push(c);
        self
    }
}

/// One visible row of a flattened tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeRow {
    /// Index path from the root.
    pub path: Vec<usize>,
    /// Nesting depth, 0 for roots.
    pub depth: usize,
    /// Text shown for the node.
    pub label: SharedString,
    /// Whether it has children.
    pub expandable: bool,
    /// Whether its children are shown.
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
    /// Create a new [`TreeView`].
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
    /// The selected item (controlled).
    pub fn selected(mut self, path: Option<Vec<usize>>) -> Self {
        self.selected = path;
        self
    }
    crate::callback!(
        /// Called with the node path when the chevron is clicked.
        on_toggle, &[usize]
    );
    crate::callback!(
        /// Called when an item is chosen, with the selected indices.
        on_select, &[usize]
    );
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
            let (style, _, marker) = list_style(&t, sel, hovered);
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
                .w(px(t.spacing.md))
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
                .min_h(px(t.scale(dodai_core::Scale::Compact)))
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

// ───────────────────────────── TreeNavigation ─────────────────────────────

/// The node at `path`, if any.
pub fn item_at<'a>(items: &'a [TreeItem], path: &[usize]) -> Option<&'a TreeItem> {
    let (first, rest) = path.split_first()?;
    let it = items.get(*first)?;
    if rest.is_empty() {
        Some(it)
    } else {
        item_at(&it.children, rest)
    }
}

/// `items` with the `expanded` overrides applied by path.
fn with_expanded(items: &[TreeItem], over: &HashMap<Vec<usize>, bool>) -> Vec<TreeItem> {
    fn walk(
        items: &[TreeItem],
        over: &HashMap<Vec<usize>, bool>,
        path: &mut Vec<usize>,
    ) -> Vec<TreeItem> {
        items
            .iter()
            .enumerate()
            .map(|(i, it)| {
                path.push(i);
                let mut n = it.clone();
                if let Some(e) = over.get(path.as_slice()) {
                    n.expanded = *e;
                }
                n.children = walk(&it.children, over, path);
                path.pop();
                n
            })
            .collect()
    }
    walk(items, over, &mut Vec::new())
}

#[derive(Default)]
struct NavState {
    expanded: HashMap<Vec<usize>, bool>,
    selected: Option<Vec<usize>>,
}

/// Navigation tree that keeps its own expansion and selection: a click on a branch folds or
/// unfolds it, a click on a leaf selects it and calls `on_select`. Built on [`TreeView`].
/// It never touches global state, so it can show any hierarchy (design systems and their
/// themes, files, settings) without side effects.
#[derive(IntoElement)]
pub struct TreeNavigation {
    id: ElementId,
    items: Vec<TreeItem>,
    selected: Option<Vec<usize>>,
    on_select: Option<PathFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(TreeNavigation);

impl TreeNavigation {
    /// Create a new [`TreeNavigation`]; each item's `expanded` is its initial state.
    pub fn new(id: impl Into<ElementId>, items: Vec<TreeItem>) -> Self {
        Self {
            id: id.into(),
            items,
            selected: None,
            on_select: None,
            corners: None,
        }
    }
    /// The leaf selected at first.
    pub fn selected(mut self, path: Vec<usize>) -> Self {
        self.selected = Some(path);
        self
    }
    crate::callback!(
        /// Called with the path of a leaf when it is chosen.
        on_select, &[usize]
    );
}

impl TreeNavigation {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let initial = self.selected.clone();
        let st = window.use_keyed_state((self.id.clone(), "nav"), cx, |_, _| NavState {
            expanded: HashMap::new(),
            selected: initial,
        });
        let (over, selected) = {
            let s = st.read(cx);
            (s.expanded.clone(), s.selected.clone())
        };
        let items = with_expanded(&self.items, &over);
        let toggle = {
            let st = st.clone();
            let items = items.clone();
            move |path: &[usize], _: &mut Window, cx: &mut App| {
                let now = item_at(&items, path).is_some_and(|i| i.expanded);
                st.update(cx, |s, cx| {
                    s.expanded.insert(path.to_vec(), !now);
                    cx.notify();
                });
            }
        };
        let choose = {
            let st = st.clone();
            let items = items.clone();
            let on_select = self.on_select.clone();
            let toggle = toggle.clone();
            move |path: &[usize], w: &mut Window, cx: &mut App| {
                if item_at(&items, path).is_some_and(|i| !i.children.is_empty()) {
                    toggle(path, w, cx);
                    return;
                }
                st.update(cx, |s, cx| {
                    s.selected = Some(path.to_vec());
                    cx.notify();
                });
                if let Some(f) = &on_select {
                    f(path, w, cx);
                }
            }
        };
        let mut tree = TreeView::new(self.id.clone(), items)
            .selected(selected)
            .on_toggle(toggle)
            .on_select(choose);
        if let Some(c) = self.corners {
            tree = tree.corners(c);
        }
        tree
    }
}

crate::entry_render!(TreeNavigation);

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
    /// Create a new [`SplitPane`].
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
    /// Height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// The first pane.
    pub fn first(mut self, e: impl IntoElement) -> Self {
        self.first = Some(e.into_any_element());
        self
    }
    /// The second pane.
    pub fn second(mut self, e: impl IntoElement) -> Self {
        self.second = Some(e.into_any_element());
        self
    }
    crate::callback!(
        /// Called when the value changes, with the new value.
        on_change, f32
    );
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
        .w(px(t.sizes().bar_thickness))
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

#[cfg(test)]
mod nav_tests {
    use super::*;

    #[test]
    fn finds_items_by_path_and_applies_overrides() {
        let items = vec![
            TreeItem::new("a").child(TreeItem::new("a0")),
            TreeItem::new("b"),
        ];
        assert_eq!(item_at(&items, &[0, 0]).unwrap().label, "a0");
        assert_eq!(item_at(&items, &[1]).unwrap().label, "b");
        assert!(item_at(&items, &[2]).is_none());
        assert!(item_at(&items, &[]).is_none());
        let mut over = HashMap::new();
        over.insert(vec![0], true);
        let open = with_expanded(&items, &over);
        assert!(open[0].expanded && !open[1].expanded);
        assert_eq!(visible_rows(&open).len(), 3);
        assert_eq!(
            visible_rows(&with_expanded(&items, &HashMap::new())).len(),
            2
        );
    }
}
