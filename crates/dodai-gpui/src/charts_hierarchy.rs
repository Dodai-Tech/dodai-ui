//! Part-of-a-whole charts beyond pie and treemap: [`Sunburst`] and [`Icicle`] (a hierarchy as
//! nested rings or rows), [`Waffle`] (a grid of cells) and [`Marimekko`] (a mosaic).

use crate::charts::{
    chart_cfg, fill, fmt_with, label_style, stroke, titled, turn_arc, Legend, NumFmt,
};
use crate::charts_focus::{readout, Focus};
use crate::paint::scale_alpha;
use crate::{hsla, theme_of};
use dodai_charts_core::hierarchy::{partition, Span};
use dodai_charts_core::share::{marimekko, waffle_cells};
use gpui::{
    canvas, div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, SharedString, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// A node of a chart hierarchy ([`Sunburst`], [`Icicle`]): `ChartNode::leaf(label, value)` or
/// `ChartNode::branch(label, children)`.
pub use dodai_charts_core::hierarchy::Node as ChartNode;

/// The colour of a span: its top-level branch's series colour, paler with each level down.
fn span_color(t: &dodai_core::Theme, s: &Span) -> Hsla {
    let base = t.palette.series_at(s.branch);
    hsla(base.mix(t.palette.surface, (0.18 * s.depth as f32).min(0.6)))
}

/// "a › b › c · value (share)" for span `i`.
fn span_text(spans: &[Span], i: usize, total: f32, fmt: &Option<NumFmt>) -> Option<String> {
    let s = spans.get(i)?;
    let mut path = vec![s.label.clone()];
    let mut at = s.parent;
    while let Some(p) = at {
        path.push(spans[p].label.clone());
        at = spans[p].parent;
    }
    path.reverse();
    let share = if total > 0.0 {
        s.value / total * 100.0
    } else {
        0.0
    };
    Some(format!(
        "{} · {} ({share:.1}%)",
        path.join(" › "),
        fmt_with(fmt, s.value)
    ))
}

/// The spans with their start/end eased towards the data (new data glides into place).
fn eased_spans(
    window: &mut Window,
    cx: &mut App,
    f: &crate::instruments_motion::Follow,
    root: &ChartNode,
) -> Vec<Span> {
    let mut spans = partition(root);
    let flat: Vec<f32> = spans.iter().flat_map(|s| [s.start, s.end]).collect();
    let shown = crate::instruments_motion::follow_all(window, cx, &f.sub("spans"), &flat);
    for (s, p) in spans.iter_mut().zip(shown.chunks(2)) {
        if let [a, b] = p {
            (s.start, s.end) = (*a, *b);
        }
    }
    spans
}

// ─────────────────────────────── Sunburst ───────────────────────────────

/// Sunburst: a hierarchy as nested rings. Each ring is one level; every node takes its share
/// of its parent's arc. Hover a segment for its path and share.
#[derive(IntoElement)]
pub struct Sunburst {
    follow: crate::instruments_motion::Follow,
    root: ChartNode,
    hole: f32,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(Sunburst);

impl Sunburst {
    crate::instruments_motion::follow_api!();
    /// Create a new [`Sunburst`] of everything below `root`.
    #[track_caller]
    pub fn new(root: ChartNode) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            root,
            hole: 0.25,
            height: 260.0,
            fmt: None,
            title: None,
        }
    }
    /// The empty middle, as a share of the radius (default 0.25, `0..=0.8`).
    pub fn hole(mut self, h: f32) -> Self {
        self.hole = if h.is_finite() {
            h.clamp(0.0, 0.8)
        } else {
            0.25
        };
        self
    }
}

/// Ring geometry for a disc of side `size`: (inner radius, ring width).
fn rings(size: f32, hole: f32, levels: usize) -> (f32, f32) {
    let outer = (size / 2.0 - 4.0).max(1.0);
    let r0 = outer * hole;
    (r0, (outer - r0) / levels.max(1) as f32)
}

impl Sunburst {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow_for(window, cx, &self.follow, "Sunburst");
        let spans = eased_spans(window, cx, &self.follow, &self.root);
        let total = self.root.total();
        let levels = self.root.depth();
        let focus = Focus::new(window, cx, &self.follow, spans.len());
        let looks: Vec<_> = (0..spans.len()).map(|i| focus.item(i)).collect();
        let cols: Vec<Hsla> = spans.iter().map(|s| span_color(&t, s)).collect();
        let geo: Vec<(usize, f32, f32)> = spans.iter().map(|s| (s.depth, s.start, s.end)).collect();
        let hit_geo = geo.clone();
        let (hole, size) = (self.hole, self.height);
        let edge = hsla(t.palette.surface);
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let pb = bounds.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                pb.set(b);
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let (cx_, cy_) = (bw / 2.0, bh / 2.0);
                let (r0, ring) = rings(bw.min(bh), hole, levels);
                for (i, &(d, a, e)) in geo.iter().enumerate() {
                    let (a, e) = (a * g, e * g);
                    if e - a < 1e-4 {
                        continue;
                    }
                    let lk = looks[i];
                    let r1 = r0 + ring * d as f32;
                    let r2 = r1 + ring + lk.lift * 4.0;
                    let mut p = turn_arc(cx_, cy_, r2, a, e);
                    p.extend(turn_arc(cx_, cy_, r1, e, a));
                    fill(w, b.origin, &p, scale_alpha(cols[i], lk.alpha));
                    stroke(w, b.origin, &p, 1.0, edge, true);
                }
            },
        )
        .absolute()
        .size_full();
        let disc = div()
            .id(self.follow.sub("disc").element_id(window, cx))
            .relative()
            .w(px(size))
            .h(px(size))
            .child(cv)
            .child(
                label_style(&t, div())
                    .absolute()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(hsla(t.palette.text_muted))
                    .child(fmt_with(&self.fmt, total)),
            );
        let disc = focus.pointer(disc, &bounds, move |x, y, w, h| {
            let (dx, dy) = (x - w / 2.0, y - h / 2.0);
            let (r0, ring) = rings(w.min(h), hole, levels);
            let d = (dx * dx + dy * dy).sqrt();
            if d < r0 || ring <= 0.0 {
                return None;
            }
            let level = ((d - r0) / ring) as usize;
            let turn = (dy.atan2(dx) / std::f32::consts::TAU + 0.25).rem_euclid(1.0);
            hit_geo
                .iter()
                .position(|&(dep, a, e)| dep == level && (a..e).contains(&turn))
        });
        let text = focus
            .shown(cx)
            .and_then(|i| span_text(&spans, i, total, &self.fmt));
        let body = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(disc)
            .child(readout(&t, text, "Hover a segment"));
        titled(&t, self.title, body)
    }
}

crate::entry_render!(Sunburst);

// ─────────────────────────────── Icicle ───────────────────────────────

/// Icicle: a hierarchy as stacked rows, the top row the first level, each node as wide as
/// its share. Easier to label than a sunburst; reads like a flame graph.
#[derive(IntoElement)]
pub struct Icicle {
    follow: crate::instruments_motion::Follow,
    root: ChartNode,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(Icicle);

impl Icicle {
    crate::instruments_motion::follow_api!();
    /// Create a new [`Icicle`] of everything below `root`.
    #[track_caller]
    pub fn new(root: ChartNode) -> Self {
        let levels = root.depth().max(1);
        Self {
            follow: crate::instruments_motion::Follow::here(),
            root,
            height: (levels as f32 * 40.0).max(120.0),
            fmt: None,
            title: None,
        }
    }
}

impl Icicle {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow_for(window, cx, &self.follow, "Icicle");
        let spans = eased_spans(window, cx, &self.follow, &self.root);
        let total = self.root.total();
        let levels = self.root.depth().max(1);
        let focus = Focus::new(window, cx, &self.follow, spans.len());
        let looks: Vec<_> = (0..spans.len()).map(|i| focus.item(i)).collect();
        let cols: Vec<Hsla> = spans.iter().map(|s| span_color(&t, s)).collect();
        let geo: Vec<(usize, f32, f32)> = spans.iter().map(|s| (s.depth, s.start, s.end)).collect();
        let hit_geo = geo.clone();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let pb = bounds.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                pb.set(b);
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let row = bh / levels as f32;
                for (i, &(d, a, e)) in geo.iter().enumerate() {
                    // Rows drop in one after another.
                    let gi = dodai_core::motion::reveal(g, d, levels);
                    let (x0, x1) = (a * bw + 1.0, e * bw - 1.0);
                    if x1 - x0 < 0.5 || gi <= 0.0 {
                        continue;
                    }
                    let lk = looks[i];
                    let y0 = d as f32 * row + 1.0;
                    let y1 = y0 + (row - 2.0) * gi;
                    fill(
                        w,
                        b.origin,
                        &[(x0, y0), (x1, y0), (x1, y1), (x0, y1)],
                        scale_alpha(cols[i], lk.alpha * (0.85 + 0.15 * lk.lift)),
                    );
                }
            },
        )
        .absolute()
        .size_full();
        // Labels on the cells wide enough to hold one.
        let mut plot = div()
            .id(self.follow.sub("rows").element_id(window, cx))
            .relative()
            .w_full()
            .h(px(self.height))
            .child(cv);
        for s in spans.iter().filter(|s| s.end - s.start > 0.08) {
            plot = plot.child(
                label_style(&t, div())
                    .absolute()
                    .left(relative(s.start))
                    .w(relative(s.end - s.start))
                    .top(relative(s.depth as f32 / levels as f32))
                    .h(relative(1.0 / levels as f32))
                    .px(px(t.spacing.xs))
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .text_color(hsla(t.palette.on_primary))
                    .child(s.label.clone()),
            );
        }
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            let level = (y / (h / levels as f32)) as usize;
            let at = x / w.max(1.0);
            hit_geo
                .iter()
                .position(|&(d, a, e)| d == level && (a..e).contains(&at))
        });
        let text = focus
            .shown(cx)
            .and_then(|i| span_text(&spans, i, total, &self.fmt));
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(plot)
            .child(readout(&t, text, "Hover a node"));
        titled(&t, self.title, body)
    }
}

crate::entry_render!(Icicle);

// ─────────────────────────────── Waffle ───────────────────────────────

/// Waffle: a grid of cells (10 × 10 by default, so one cell = 1 %), each coloured by the
/// part it counts towards. Shares read as countable cells rather than angles.
#[derive(IntoElement)]
pub struct Waffle {
    follow: crate::instruments_motion::Follow,
    items: Vec<(SharedString, f32)>,
    grid: (usize, usize),
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(Waffle);

impl Waffle {
    crate::instruments_motion::follow_api!();
    /// Create a new [`Waffle`] from `(label, value)` parts.
    #[track_caller]
    pub fn new<S: Into<SharedString>>(items: impl IntoIterator<Item = (S, f32)>) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            items: items.into_iter().map(|(l, v)| (l.into(), v)).collect(),
            grid: (10, 10),
            height: 200.0,
            fmt: None,
            title: None,
        }
    }
    /// Columns and rows of the grid (each `1..=50`).
    pub fn grid(mut self, cols: usize, rows: usize) -> Self {
        self.grid = (cols.clamp(1, 50), rows.clamp(1, 50));
        self
    }
}

impl Waffle {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        crate::instruments_motion::follow_pairs(window, cx, &self.follow, &mut self.items);
        let t = theme_of(cx);
        let g = crate::motion::grow_for(window, cx, &self.follow, "Waffle");
        let (gc, gr) = self.grid;
        let values: Vec<f32> = self.items.iter().map(|i| i.1).collect();
        let counts = waffle_cells(&values, gc * gr);
        // Cell k belongs to the part whose run covers it.
        let owner: Vec<Option<usize>> = counts
            .iter()
            .enumerate()
            .flat_map(|(i, n)| std::iter::repeat_n(Some(i), *n))
            .chain(std::iter::repeat(None))
            .take(gc * gr)
            .collect();
        let n = self.items.len();
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|i| focus.item(i)).collect();
        let cols: Vec<Hsla> = (0..n).map(|i| hsla(t.palette.series_at(i))).collect();
        let empty = hsla(t.palette.border);
        let hit_owner = owner.clone();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let pb = bounds.clone();
        let cell = move |bw: f32, bh: f32| (bw / gc as f32).min(bh / gr as f32);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                pb.set(b);
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let s = cell(bw, bh);
                let gap = (s * 0.12).max(1.0);
                let shown = (g * (gc * gr) as f32).ceil() as usize;
                // Filled column by column from the bottom left, like a rising tally.
                for (k, o) in owner.iter().enumerate() {
                    let (c, r) = (k / gr, gr - 1 - k % gr);
                    let (x, y) = (c as f32 * s, r as f32 * s);
                    let col = match o {
                        Some(i) if k < shown => scale_alpha(cols[*i], looks[*i].alpha),
                        _ => scale_alpha(empty, 0.5),
                    };
                    let inset = gap / 2.0 - o.map_or(0.0, |i| looks[i].lift * gap * 0.4);
                    fill(
                        w,
                        b.origin,
                        &[
                            (x + inset, y + inset),
                            (x + s - inset, y + inset),
                            (x + s - inset, y + s - inset),
                            (x + inset, y + s - inset),
                        ],
                        col,
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let side = self.height / gr as f32;
        let grid = div()
            .id(self.follow.sub("grid").element_id(window, cx))
            .relative()
            .flex_none()
            .w(px(side * gc as f32))
            .h(px(self.height))
            .child(cv);
        let grid = focus.pointer(grid, &bounds, move |x, y, w, h| {
            let s = cell(w, h);
            let (c, r) = ((x / s) as usize, (y / s) as usize);
            if c >= gc || r >= gr {
                return None;
            }
            hit_owner.get(c * gr + (gr - 1 - r)).copied().flatten()
        });
        let total: f32 = values
            .iter()
            .filter(|v| v.is_finite())
            .map(|v| v.max(0.0))
            .sum();
        let text = focus.shown(cx).and_then(|i| {
            let (l, v) = self.items.get(i)?;
            Some(format!(
                "{l} · {} · {} of {} cells",
                fmt_with(&self.fmt, *v),
                counts[i],
                gc * gr
            ))
        });
        let legend = Legend::new(self.items.iter().enumerate().map(|(i, (l, v))| {
            let pct = if total > 0.0 {
                v.max(0.0) / total * 100.0
            } else {
                0.0
            };
            (
                SharedString::from(format!("{l} {pct:.0}%")),
                t.palette.series_at(i),
            )
        }));
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .child(grid)
            .child(legend)
            .child(readout(&t, text, "Hover a cell"));
        titled(&t, self.title, body)
    }
}

crate::entry_render!(Waffle);

// ─────────────────────────────── Marimekko ───────────────────────────────

/// Marimekko (mosaic): a 100 % stacked bar chart whose bars are also as wide as their share
/// of the total, so one picture shows both the size of each column and its mix.
#[derive(IntoElement)]
pub struct Marimekko {
    follow: crate::instruments_motion::Follow,
    segments: Vec<SharedString>,
    columns: Vec<(SharedString, Vec<f32>)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(Marimekko);

impl Marimekko {
    crate::instruments_motion::follow_api!();
    /// Create a new [`Marimekko`]: segment names (top to bottom) and `(column, values)` with
    /// one value per segment.
    #[track_caller]
    pub fn new<S: Into<SharedString>, C: Into<SharedString>>(
        segments: impl IntoIterator<Item = S>,
        columns: impl IntoIterator<Item = (C, Vec<f32>)>,
    ) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            segments: segments.into_iter().map(Into::into).collect(),
            columns: columns.into_iter().map(|(c, v)| (c.into(), v)).collect(),
            height: 220.0,
            fmt: None,
            title: None,
        }
    }
}

impl Marimekko {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut rows: Vec<Vec<f32>> = self.columns.iter().map(|c| c.1.clone()).collect();
        crate::instruments_motion::follow_rows(window, cx, &self.follow, &mut rows);
        for (c, r) in self.columns.iter_mut().zip(rows) {
            c.1 = r;
        }
        let t = theme_of(cx);
        let g = crate::motion::grow_for(window, cx, &self.follow, "Marimekko");
        let tiles = marimekko(&rows_of(&self.columns));
        let ns = tiles.iter().map(Vec::len).max().unwrap_or(0);
        let n = tiles.len() * ns;
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|i| focus.item(i)).collect();
        let cols: Vec<Hsla> = (0..ns).map(|i| hsla(t.palette.series_at(i))).collect();
        let flat: Vec<(usize, dodai_charts_core::share::Tile)> = tiles
            .iter()
            .enumerate()
            .flat_map(|(c, col)| col.iter().enumerate().map(move |(s, tl)| (c * ns + s, *tl)))
            .collect();
        let hit_flat = flat.clone();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let pb = bounds.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                pb.set(b);
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                for &(k, tl) in &flat {
                    let lk = looks[k];
                    let (x0, x1) = (tl.x0 * bw + 1.0, tl.x1 * bw - 1.0);
                    // Grows up from the floor.
                    let (y0, y1) = (
                        bh - (1.0 - tl.y0) * bh * g + 1.0,
                        bh - (1.0 - tl.y1) * bh * g - 1.0,
                    );
                    if x1 - x0 < 0.5 || y1 - y0 < 0.5 {
                        continue;
                    }
                    fill(
                        w,
                        b.origin,
                        &[(x0, y0), (x1, y0), (x1, y1), (x0, y1)],
                        scale_alpha(cols[k % ns.max(1)], lk.alpha * (0.85 + 0.15 * lk.lift)),
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let plot = div()
            .id(self.follow.sub("tiles").element_id(window, cx))
            .relative()
            .w_full()
            .h(px(self.height))
            .child(cv);
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            let (fx, fy) = (x / w.max(1.0), y / h.max(1.0));
            hit_flat
                .iter()
                .find(|(_, tl)| (tl.x0..tl.x1).contains(&fx) && (tl.y0..tl.y1).contains(&fy))
                .map(|(k, _)| *k)
        });
        // Column names under their columns, as wide as the columns.
        let mut names = label_style(&t, div())
            .relative()
            .w_full()
            .h(px(t.typography.size_xs * 1.6));
        for (col, (name, _)) in tiles.iter().zip(&self.columns) {
            if let Some(tl) = col.first() {
                names = names.child(
                    div()
                        .absolute()
                        .left(relative(tl.x0))
                        .w(relative(tl.x1 - tl.x0))
                        .flex()
                        .justify_center()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(name.clone()),
                );
            }
        }
        let grand: f32 = rows_of(&self.columns)
            .iter()
            .flatten()
            .filter(|v| v.is_finite())
            .map(|v| v.max(0.0))
            .sum();
        let text = focus.shown(cx).and_then(|k| {
            let (c, s) = (k / ns.max(1), k % ns.max(1));
            let (cname, vals) = self.columns.get(c)?;
            let v = *vals.get(s)?;
            let seg = self.segments.get(s).cloned().unwrap_or_default();
            let tl = tiles.get(c)?.get(s)?;
            Some(format!(
                "{cname} · {seg}: {} · {:.0}% of {cname}, {:.0}% of all",
                fmt_with(&self.fmt, v),
                (tl.y1 - tl.y0) * 100.0,
                if grand > 0.0 {
                    v.max(0.0) / grand * 100.0
                } else {
                    0.0
                }
            ))
        });
        let legend = Legend::new(
            self.segments
                .iter()
                .enumerate()
                .map(|(i, s)| (s.clone(), t.palette.series_at(i))),
        );
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(plot)
            .child(names)
            .child(legend)
            .child(readout(&t, text, "Hover a tile"));
        titled(&t, self.title, body)
    }
}

fn rows_of(columns: &[(SharedString, Vec<f32>)]) -> Vec<Vec<f32>> {
    columns.iter().map(|c| c.1.clone()).collect()
}

crate::entry_render!(Marimekko);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_span_reads_as_its_path_value_and_share() {
        let root = ChartNode::branch(
            "all",
            [
                ChartNode::branch("a", [ChartNode::leaf("a1", 1.0)]),
                ChartNode::leaf("b", 3.0),
            ],
        );
        let spans = partition(&root);
        assert_eq!(
            span_text(&spans, 1, root.total(), &None).as_deref(),
            Some("a › a1 · 1 (25.0%)")
        );
        assert!(span_text(&spans, 9, 4.0, &None).is_none());
    }

    #[test]
    fn rings_fit_inside_the_disc() {
        let (r0, ring) = rings(200.0, 0.25, 3);
        assert!(r0 + 3.0 * ring <= 100.0);
        assert_eq!(Waffle::new([("a", 1.0)]).grid(0, 99).grid, (1, 50));
    }
}
