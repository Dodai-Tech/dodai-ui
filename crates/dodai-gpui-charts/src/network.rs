//! Relationship and flow charts: [`ParallelCoordinates`], [`Hexbin`], [`NetworkGraph`],
//! [`ChordDiagram`] and [`ArcDiagram`].

use crate::common::{
    chart_cfg, fill, fmt_with, label_column, label_style, marker_points, nice_ticks, norm,
    plot_box, polar, stroke, tick_row, titled, tracked_canvas, turn_arc, CanvasText, Marker,
    NumFmt,
};
use crate::focus::{readout, Focus};
use dodai_charts_core::{
    chord::chord,
    force::force_layout,
    hexbin::{hex_corners, hexbin, Hex},
};
use dodai_gpui::paint::scale_alpha;
use dodai_gpui::{hsla, theme_of};
use gpui::{
    div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, Styled, Window,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Points of a quadratic Bézier from `a` to `b` bending towards `c`. Pure.
fn quad(a: (f32, f32), c: (f32, f32), b: (f32, f32), n: usize) -> Vec<(f32, f32)> {
    (0..=n)
        .map(|i| {
            let t = i as f32 / n.max(1) as f32;
            let u = 1.0 - t;
            (
                u * u * a.0 + 2.0 * u * t * c.0 + t * t * b.0,
                u * u * a.1 + 2.0 * u * t * c.1 + t * t * b.1,
            )
        })
        .collect()
}

/// Distance from `p` to the segment `a..b`. Pure.
fn seg_dist(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    ((p.0 - a.0 - t * dx).powi(2) + (p.1 - a.1 - t * dy).powi(2)).sqrt()
}

/// Weighted degree of every node.
fn degrees(n: usize, links: &[(usize, usize, f32)]) -> Vec<f32> {
    let mut d = vec![0.0; n];
    for &(a, b, w) in links {
        let w = if w.is_finite() { w.max(0.0) } else { 0.0 };
        if a < n && b < n {
            d[a] += w;
            d[b] += w;
        }
    }
    d
}

/// "name · degree · linked to …" for node `i`.
fn node_text(names: &[SharedString], links: &[(usize, usize, f32)], i: usize) -> Option<String> {
    let name = names.get(i)?;
    let mut peers: Vec<&str> = links
        .iter()
        .filter_map(|&(a, b, _)| match (a == i, b == i) {
            (true, false) => names.get(b),
            (false, true) => names.get(a),
            _ => None,
        })
        .map(|s| s.as_ref())
        .collect();
    peers.sort_unstable();
    peers.dedup();
    Some(if peers.is_empty() {
        format!("{name} · no links")
    } else {
        format!("{name} · {} links: {}", peers.len(), peers.join(", "))
    })
}

// ─────────────────────────────── ParallelCoordinates ───────────────────────────────

/// Parallel coordinates: one vertical axis per measure, each item a line across them, so
/// items with similar profiles bundle and trade-offs show as crossings. Every axis has its
/// own scale.
#[derive(IntoElement)]
pub struct ParallelCoordinates {
    follow: dodai_gpui::follow::Follow,
    axes: Vec<SharedString>,
    items: Vec<(SharedString, Vec<f32>)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(ParallelCoordinates);

impl ParallelCoordinates {
    dodai_gpui::follow_api!();
    /// Create new [`ParallelCoordinates`]: axis names, then `(item, one value per axis)`.
    #[track_caller]
    pub fn new<A: Into<SharedString>, S: Into<SharedString>>(
        axes: impl IntoIterator<Item = A>,
        items: impl IntoIterator<Item = (S, Vec<f32>)>,
    ) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            axes: axes.into_iter().map(Into::into).collect(),
            items: items.into_iter().map(|(l, v)| (l.into(), v)).collect(),
            height: 220.0,
            fmt: None,
            title: None,
        }
    }
}

impl ParallelCoordinates {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut rows: Vec<Vec<f32>> = self.items.iter().map(|i| i.1.clone()).collect();
        dodai_gpui::follow::follow_rows(window, cx, &self.follow, &mut rows);
        for (it, r) in self.items.iter_mut().zip(rows) {
            it.1 = r;
        }
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "ParallelCoordinates");
        let na = self.axes.len().max(2);
        // Each axis' own range.
        let ranges: Vec<(f32, f32)> = (0..na)
            .map(|k| {
                let (lo, hi) = self
                    .items
                    .iter()
                    .filter_map(|it| it.1.get(k).copied().filter(|v| v.is_finite()))
                    .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), v| {
                        (a.min(v), b.max(v))
                    });
                if lo.is_finite() && hi > lo {
                    (lo, hi)
                } else if lo.is_finite() {
                    (lo - 1.0, lo + 1.0)
                } else {
                    (0.0, 1.0)
                }
            })
            .collect();
        let n = self.items.len();
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|i| focus.item(i)).collect();
        let cols: Vec<Hsla> = (0..n).map(|i| hsla(t.palette.series_at(i))).collect();
        let axis = hsla(t.palette.border_strong);
        // (item, fraction from the bottom) per axis.
        let lines: Vec<Vec<Option<f32>>> = self
            .items
            .iter()
            .map(|it| {
                (0..na)
                    .map(|k| {
                        let v = *it.1.get(k)?;
                        v.is_finite().then(|| norm(v, ranges[k].0, ranges[k].1))
                    })
                    .collect()
            })
            .collect();
        let hit_lines = lines.clone();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let x_of = move |k: usize, bw: f32| k as f32 / (na - 1) as f32 * bw;
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            for k in 0..na {
                let x = x_of(k, bw);
                stroke(w, b.origin, &[(x, 0.0), (x, bh)], 1.5, axis, false);
            }
            // Lines draw left to right as they grow; the focused one on top.
            let mut order: Vec<usize> = (0..lines.len()).collect();
            order.sort_by(|a, b| looks[*a].lift.total_cmp(&looks[*b].lift));
            for i in order {
                let lk = looks[i];
                let p: Vec<(f32, f32)> = lines[i]
                    .iter()
                    .enumerate()
                    .filter_map(|(k, f)| f.map(|f| (x_of(k, bw), bh * (1.0 - f))))
                    .collect();
                let reach = g * bw;
                let mut shown: Vec<(f32, f32)> = Vec::new();
                for pair in p.windows(2) {
                    let (a, z) = (pair[0], pair[1]);
                    if a.0 > reach {
                        break;
                    }
                    if shown.is_empty() {
                        shown.push(a);
                    }
                    if z.0 <= reach {
                        shown.push(z);
                    } else {
                        let f = (reach - a.0) / (z.0 - a.0).max(1e-3);
                        shown.push((reach, a.1 + (z.1 - a.1) * f));
                        break;
                    }
                }
                let alpha = lk.alpha * (0.55 + 0.45 * lk.lift.min(1.0));
                stroke(
                    w,
                    b.origin,
                    &shown,
                    1.5 + lk.lift * 1.5,
                    scale_alpha(cols[i], alpha),
                    false,
                );
            }
        })
        .absolute()
        .size_full();
        let plot = plot_box(
            self.follow.sub("lines").element_id(window, cx),
            self.height,
            cv,
        );
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            hit_lines
                .iter()
                .enumerate()
                .filter_map(|(i, l)| {
                    let p: Vec<(f32, f32)> = l
                        .iter()
                        .enumerate()
                        .filter_map(|(k, f)| f.map(|f| (x_of(k, w), h * (1.0 - f))))
                        .collect();
                    let d = p
                        .windows(2)
                        .map(|s| seg_dist((x, y), s[0], s[1]))
                        .fold(f32::INFINITY, f32::min);
                    (d < 6.0).then_some((i, d))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| i)
        });
        let edge_row = |top: bool| {
            let mut row = label_style(&t, div())
                .relative()
                .w_full()
                .h(px(t.typography.size_xs * 1.6));
            for (k, (lo, hi)) in ranges.iter().enumerate() {
                let f = k as f32 / (na - 1) as f32;
                let text = if top {
                    self.axes.get(k).map(|a| a.to_string()).unwrap_or_default()
                } else {
                    format!(
                        "{} – {}",
                        fmt_with(&self.fmt, *lo),
                        fmt_with(&self.fmt, *hi)
                    )
                };
                row = row.child(
                    div()
                        .absolute()
                        .left(relative(f))
                        .w(px(120.0))
                        .ml(px(if k == 0 {
                            0.0
                        } else if k == na - 1 {
                            -120.0
                        } else {
                            -60.0
                        }))
                        .flex()
                        .when_else(
                            k == 0,
                            |d| d.justify_start(),
                            |d| {
                                if k == na - 1 {
                                    d.justify_end()
                                } else {
                                    d.justify_center()
                                }
                            },
                        )
                        .when(top, |d| d.text_color(hsla(t.palette.text)))
                        .child(text),
                );
            }
            row
        };
        let text = focus.shown(cx).and_then(|i| {
            let (name, v) = self.items.get(i)?;
            let parts: Vec<String> = self
                .axes
                .iter()
                .zip(v)
                .map(|(a, v)| format!("{a} {}", fmt_with(&self.fmt, *v)))
                .collect();
            Some(format!("{name} · {}", parts.join(" · ")))
        });
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(edge_row(true))
            .child(plot)
            .child(edge_row(false))
            .child(readout(&t, text, "Hover a line"));
        titled(&t, self.title, body)
    }
}

use gpui::prelude::FluentBuilder as _;

dodai_gpui::entry_render!(ParallelCoordinates);

// ─────────────────────────────── Hexbin ───────────────────────────────

/// Hexbin: a scatter plot too dense for dots, binned into hexagons shaded by how many
/// points each holds.
#[derive(IntoElement)]
pub struct Hexbin {
    follow: dodai_gpui::follow::Follow,
    points: Vec<(f32, f32)>,
    radius: f32,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(Hexbin);

impl Hexbin {
    dodai_gpui::follow_api!();
    /// Create a new [`Hexbin`] from `(x, y)` points.
    #[track_caller]
    pub fn new(points: impl IntoIterator<Item = (f32, f32)>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            points: points.into_iter().collect(),
            radius: 10.0,
            height: 220.0,
            fmt: None,
            title: None,
        }
    }
    /// Hexagon radius in px (default 10, `4..=40`).
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = if r.is_finite() {
            r.clamp(4.0, 40.0)
        } else {
            10.0
        };
        self
    }
}

impl Hexbin {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "Hexbin");
        let fin: Vec<(f32, f32)> = self
            .points
            .iter()
            .copied()
            .filter(|(x, y)| x.is_finite() && y.is_finite())
            .collect();
        let span = |v: Vec<f32>| {
            let (lo, hi) = v
                .iter()
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), x| {
                    (a.min(*x), b.max(*x))
                });
            if lo.is_finite() && hi > lo {
                nice_ticks(lo, hi, 5)
            } else if lo.is_finite() {
                vec![lo - 1.0, lo + 1.0]
            } else {
                vec![0.0, 1.0]
            }
        };
        let xt = span(fin.iter().map(|p| p.0).collect());
        let yt = span(fin.iter().map(|p| p.1).collect());
        let (x0, x1, y0, y1) = (xt[0], xt[xt.len() - 1], yt[0], yt[yt.len() - 1]);
        let r = self.radius;
        let lo_c = t.palette.surface.mix(t.palette.primary, 0.15);
        let hi_c = t.palette.primary;
        let (hot, cold) = (hsla(hi_c), hsla(lo_c));
        let shade = move |f: f32| hsla(lo_c.mix(hi_c, f));
        let style = t.chart_style();
        let y_labels: Vec<String> = yt.iter().map(|v| fmt_with(&self.fmt, *v)).collect();
        let cells: Rc<RefCell<Vec<Hex>>> = Default::default();
        let painted = cells.clone();
        // Hexagons exist only once painted at a size, so focus is just the hovered index.
        let focus = Focus::new(window, cx, &self.follow, 0);
        let hover = focus.shown(cx);
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            for k in 1..yt.len().saturating_sub(1) {
                let y = bh * (1.0 - k as f32 / (yt.len() - 1) as f32);
                crate::common::grid_line(w, b.origin, (0.0, y), (bw, y), &style);
            }
            let pts: Vec<(f32, f32)> = fin
                .iter()
                .map(|(x, y)| (norm(*x, x0, x1) * bw, bh * (1.0 - norm(*y, y0, y1))))
                .collect();
            let hexes = hexbin(&pts, r);
            let top = hexes.iter().map(|h| h.count).max().unwrap_or(1).max(1);
            for (i, h) in hexes.iter().enumerate() {
                let f = h.count as f32 / top as f32;
                let size = (r - 0.75) * g * if hover == Some(i) { 1.15 } else { 1.0 };
                fill(w, b.origin, &hex_corners(h.x, h.y, size), shade(f.sqrt()));
            }
            *painted.borrow_mut() = hexes;
        })
        .absolute()
        .size_full();
        let plot = div()
            .id(self.follow.sub("hex").element_id(window, cx))
            .relative()
            .w_full()
            .h(px(self.height))
            .overflow_hidden()
            .child(cv);
        let hit_cells = cells.clone();
        let plot = focus.pointer(plot, &bounds, move |x, y, _, _| {
            hit_cells
                .borrow()
                .iter()
                .position(|h| (h.x - x).powi(2) + (h.y - y).powi(2) <= r * r)
        });
        let text = hover.and_then(|i| {
            let h = *cells.borrow().get(i)?;
            let b = bounds.get();
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let (vx, vy) = (
                x0 + (x1 - x0) * h.x / bw.max(1.0),
                y1 - (y1 - y0) * h.y / bh.max(1.0),
            );
            Some(format!(
                "{} points around ({}, {})",
                h.count,
                fmt_with(&self.fmt, vx),
                fmt_with(&self.fmt, vy)
            ))
        });
        let scale = label_style(&t, div())
            .flex()
            .items_center()
            .gap(px(t.spacing.sm))
            .child("fewer")
            .child(div().w(px(80.0)).h(px(8.0)).bg(gpui::linear_gradient(
                90.0,
                gpui::linear_color_stop(cold, 0.0),
                gpui::linear_color_stop(hot, 1.0),
            )))
            .child("more");
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(crate::common::framed_row(
                &t,
                y_labels,
                Some(tick_row(&t, &xt, &self.fmt)),
                div().w_full().child(plot),
            ))
            .child(scale)
            .child(readout(&t, text, "Hover a hexagon"));
        titled(&t, self.title, body)
    }
}

dodai_gpui::entry_render!(Hexbin);

// ─────────────────────────────── NetworkGraph ───────────────────────────────

/// Network graph: nodes and links laid out by a force simulation (linked nodes pull
/// together, all push apart). Node size follows its weighted links; hover one to light up
/// its neighbourhood.
#[derive(IntoElement)]
pub struct NetworkGraph {
    follow: dodai_gpui::follow::Follow,
    nodes: Vec<SharedString>,
    groups: Vec<usize>,
    links: Vec<(usize, usize, f32)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(NetworkGraph);

impl NetworkGraph {
    dodai_gpui::follow_api!();
    /// Create a new [`NetworkGraph`]: node names and `(from, to, weight)` links by index.
    #[track_caller]
    pub fn new<S: Into<SharedString>>(
        nodes: impl IntoIterator<Item = S>,
        links: impl IntoIterator<Item = (usize, usize, f32)>,
    ) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            nodes: nodes.into_iter().map(Into::into).collect(),
            groups: Vec::new(),
            links: links.into_iter().collect(),
            height: 280.0,
            fmt: None,
            title: None,
        }
    }
    /// A group per node (by index), coloured alike; nodes without one share the first colour.
    pub fn groups(mut self, g: impl IntoIterator<Item = usize>) -> Self {
        self.groups = g.into_iter().collect();
        self
    }
}

impl NetworkGraph {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "NetworkGraph");
        let n = self.nodes.len();
        let pos = force_layout(n, &self.links, 300);
        let deg = degrees(n, &self.links);
        let top = deg.iter().copied().fold(0.0f32, f32::max).max(1e-6);
        let size: Vec<f32> = deg.iter().map(|d| 4.0 + 8.0 * (d / top).sqrt()).collect();
        let focus = Focus::new(window, cx, &self.follow, n);
        let shown = focus.shown(cx);
        let cols: Vec<Hsla> = (0..n)
            .map(|i| {
                hsla(
                    t.palette
                        .series_at(self.groups.get(i).copied().unwrap_or(0)),
                )
            })
            .collect();
        // Hovering a node keeps it and its neighbours bright and dims the rest.
        let near: Vec<bool> = (0..n)
            .map(|i| {
                shown.is_none_or(|s| {
                    s == i
                        || self
                            .links
                            .iter()
                            .any(|&(a, b, _)| (a == s && b == i) || (b == s && a == i))
                })
            })
            .collect();
        let link_col = hsla(t.palette.text_muted);
        let paint_near = near.clone();
        let links = self.links.clone();
        let wmax = links
            .iter()
            .map(|l| l.2)
            .filter(|w| w.is_finite())
            .fold(0.0f32, f32::max)
            .max(1e-6);
        let paint_pos = pos.clone();
        let paint_size = size.clone();
        let hit_pos = pos.clone();
        let hit_size = size.clone();
        let edge = hsla(t.palette.surface);
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let place = move |p: (f32, f32), bw: f32, bh: f32| {
            // Grows out of the middle.
            let (x, y) = (0.5 + (p.0 - 0.5) * g, 0.5 + (p.1 - 0.5) * g);
            (x * bw, y * bh)
        };
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            for &(a, z, wt) in &links {
                let (Some(pa), Some(pz)) = (paint_pos.get(a), paint_pos.get(z)) else {
                    continue;
                };
                if !wt.is_finite() || a == z {
                    continue;
                }
                let lit = shown.is_none_or(|s| s == a || s == z);
                let alpha = if lit { 0.35 + 0.4 * (wt / wmax) } else { 0.08 };
                stroke(
                    w,
                    b.origin,
                    &[place(*pa, bw, bh), place(*pz, bw, bh)],
                    1.0 + 2.0 * (wt / wmax).clamp(0.0, 1.0),
                    scale_alpha(link_col, alpha),
                    false,
                );
            }
            for (i, p) in paint_pos.iter().enumerate() {
                let (x, y) = place(*p, bw, bh);
                let r = paint_size[i] * if shown == Some(i) { 1.3 } else { 1.0 };
                let c = scale_alpha(cols[i], if paint_near[i] { 1.0 } else { 0.25 });
                fill(
                    w,
                    b.origin,
                    &marker_points(Marker::Dot, x, y, r + 1.5),
                    edge,
                );
                fill(w, b.origin, &marker_points(Marker::Dot, x, y, r), c);
            }
        })
        .absolute()
        .size_full();
        let mut plot = plot_box(
            self.follow.sub("graph").element_id(window, cx),
            self.height,
            cv,
        );
        // Names beside the bigger nodes (and any focused one).
        let label_at = top * 0.35;
        for (i, name) in self.nodes.iter().enumerate() {
            if deg[i] < label_at && shown != Some(i) {
                continue;
            }
            let (x, y) = (0.5 + (pos[i].0 - 0.5) * g, 0.5 + (pos[i].1 - 0.5) * g);
            plot = plot.child(
                label_style(&t, div())
                    .absolute()
                    .left(relative(x))
                    .top(relative(y))
                    .ml(px(size[i] + 3.0))
                    .mt(px(-t.typography.size_xs * 0.75))
                    .whitespace_nowrap()
                    .text_color(hsla(if near[i] {
                        t.palette.text
                    } else {
                        t.palette.text_muted
                    }))
                    .child(name.clone()),
            );
        }
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            hit_pos
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let (px_, py) = place(*p, w, h);
                    (
                        i,
                        ((px_ - x).powi(2) + (py - y).powi(2)).sqrt() - hit_size[i],
                    )
                })
                .filter(|(_, d)| *d < 4.0)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| i)
        });
        let text = shown.and_then(|i| node_text(&self.nodes, &self.links, i));
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

dodai_gpui::entry_render!(NetworkGraph);

// ─────────────────────────────── ChordDiagram ───────────────────────────────

/// Chord diagram: groups around a circle, ribbons for the flow between each pair (both
/// directions; each end as wide as the flow leaving that side). Hover a group to isolate it.
#[derive(IntoElement)]
pub struct ChordDiagram {
    follow: dodai_gpui::follow::Follow,
    names: Vec<SharedString>,
    matrix: Vec<Vec<f32>>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(ChordDiagram);

impl ChordDiagram {
    dodai_gpui::follow_api!();
    /// Create a new [`ChordDiagram`]: group names and the flow matrix (`m[i][j]` = from `i` to
    /// `j`).
    #[track_caller]
    pub fn new<S: Into<SharedString>>(
        names: impl IntoIterator<Item = S>,
        matrix: Vec<Vec<f32>>,
    ) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            names: names.into_iter().map(Into::into).collect(),
            matrix,
            height: 300.0,
            fmt: None,
            title: None,
        }
    }
}

/// The group whose arc covers the pointer's angle (anywhere inside the ring and its names).
fn chord_hit(groups: &[(f32, f32)], x: f32, y: f32, w: f32, h: f32) -> Option<usize> {
    let (dx, dy) = (x - w / 2.0, y - h / 2.0);
    if (dx * dx + dy * dy).sqrt() > w.max(h) / 2.0 {
        return None;
    }
    let turn = (dy.atan2(dx) / std::f32::consts::TAU + 0.25).rem_euclid(1.0);
    groups.iter().position(|(a, b)| (*a..*b).contains(&turn))
}

impl ChordDiagram {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        dodai_gpui::follow::follow_rows(window, cx, &self.follow, &mut self.matrix);
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "ChordDiagram");
        let lay = chord(&self.matrix, 0.015);
        let n = lay.groups.len();
        let focus = Focus::new(window, cx, &self.follow, n);
        let shown = focus.shown(cx);
        let cols: Vec<Hsla> = (0..n).map(|i| hsla(t.palette.series_at(i))).collect();
        let paint = lay.clone();
        let hit_groups = lay.groups.clone();
        let size = self.height;
        // Names sit around the ring: room for the longest beside it, a line above and below.
        let text = CanvasText::of(&t);
        let side = label_column(&t, &self.names) + t.spacing.sm;
        let above = text.size_line() + t.spacing.sm;
        let names = self.names.clone();
        let (ink, muted) = (hsla(t.palette.text), hsla(t.palette.text_muted));
        let gap = t.spacing.xs;
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, cx| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let (cx_, cy_) = (bw / 2.0, bh / 2.0);
            let outer = ((bw / 2.0 - side).min(bh / 2.0 - above) - gap).max(8.0);
            let band = (outer * 0.07).clamp(6.0, 14.0);
            let inner = outer - band;
            // Ribbons stop short of the arcs, so each arc reads as a clean band.
            let reach = inner - gap;
            let c = (cx_, cy_);
            for r in &paint.ribbons {
                let lit = shown.is_none_or(|s| s == r.from || s == r.to);
                let (s0, s1) = (r.source.0 * g, r.source.1 * g);
                let (t0, t1) = (r.target.0 * g, r.target.1 * g);
                let mut p = turn_arc(cx_, cy_, reach, s0, s1);
                if r.from == r.to {
                    // Flow that stays: a short loop just inside its own arc.
                    let tip = polar(cx_, cy_, reach * 0.55, (s0 + s1) / 2.0);
                    p.extend(quad(
                        polar(cx_, cy_, reach, s1),
                        tip,
                        polar(cx_, cy_, reach, s0),
                        24,
                    ));
                } else {
                    p.extend(quad(
                        polar(cx_, cy_, reach, s1),
                        c,
                        polar(cx_, cy_, reach, t0),
                        32,
                    ));
                    p.extend(turn_arc(cx_, cy_, reach, t0, t1));
                    p.extend(quad(
                        polar(cx_, cy_, reach, t1),
                        c,
                        polar(cx_, cy_, reach, s0),
                        32,
                    ));
                }
                // A ribbon takes the colour of the side that sends more.
                let col = cols[if s1 - s0 >= t1 - t0 { r.from } else { r.to }];
                // Flow that stays is quieter than flow that moves.
                let calm = if r.from == r.to { 0.45 } else { 1.0 };
                let (body, edge) = if lit {
                    (0.5 * calm, 0.85 * calm)
                } else {
                    (0.06, 0.12)
                };
                fill(w, b.origin, &p, scale_alpha(col, body));
                stroke(w, b.origin, &p, 1.0, scale_alpha(col, edge), true);
            }
            for (i, (a, e)) in paint.groups.iter().enumerate() {
                let (a, e) = (a * g, e * g);
                if e - a < 1e-4 {
                    continue;
                }
                let lift = if shown == Some(i) { gap } else { 0.0 };
                let mut p = turn_arc(cx_, cy_, outer + lift, a, e);
                p.extend(turn_arc(cx_, cy_, inner, e, a));
                fill(w, b.origin, &p, cols[i]);
                if g < 1.0 {
                    continue;
                }
                let mid = (a + e) / 2.0;
                let (px_, py_) = polar(0.0, 0.0, 1.0, mid);
                let at = polar(cx_, cy_, outer + lift + gap, mid);
                let col = if shown.is_none_or(|s| s == i) {
                    ink
                } else {
                    muted
                };
                if let Some(name) = names.get(i) {
                    text.beside(w, cx, name, (b.origin, at), (px_, py_), col);
                }
            }
        })
        .absolute()
        .size_full();
        let disc = div()
            .id(self.follow.sub("disc").element_id(window, cx))
            .relative()
            .flex_none()
            .w(px(size + 2.0 * side))
            .h(px(size + 2.0 * above))
            .child(cv);
        let disc = focus.pointer(disc, &bounds, move |x, y, w, h| {
            chord_hit(&hit_groups, x, y, w, h)
        });
        let at = |i: usize, j: usize| {
            self.matrix
                .get(i)
                .and_then(|r| r.get(j))
                .copied()
                .filter(|v| v.is_finite())
                .unwrap_or(0.0)
                .max(0.0)
        };
        let text = shown.and_then(|i| {
            let name = self.names.get(i)?;
            let out: f32 = (0..n).map(|j| at(i, j)).sum();
            let inn: f32 = (0..n).map(|j| at(j, i)).sum();
            let (j, big) = (0..n)
                .filter(|j| *j != i)
                .map(|j| (j, at(i, j)))
                .max_by(|a, b| a.1.total_cmp(&b.1))?;
            Some(format!(
                "{name} · out {} · in {} · most to {}: {}",
                fmt_with(&self.fmt, out),
                fmt_with(&self.fmt, inn),
                self.names.get(j).map_or("", |s| s.as_ref()),
                fmt_with(&self.fmt, big)
            ))
        });
        let body = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(disc)
            .child(readout(&t, text, "Hover a group"));
        titled(&t, self.title, body)
    }
}

dodai_gpui::entry_render!(ChordDiagram);

// ─────────────────────────────── ArcDiagram ───────────────────────────────

/// Arc diagram: nodes in a row, each link an arc above them (thicker = heavier). Keeps the
/// node order readable where a network graph would tangle.
#[derive(IntoElement)]
pub struct ArcDiagram {
    follow: dodai_gpui::follow::Follow,
    nodes: Vec<SharedString>,
    links: Vec<(usize, usize, f32)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(ArcDiagram);

impl ArcDiagram {
    dodai_gpui::follow_api!();
    /// Create a new [`ArcDiagram`]: node names (in order) and `(from, to, weight)` links.
    #[track_caller]
    pub fn new<S: Into<SharedString>>(
        nodes: impl IntoIterator<Item = S>,
        links: impl IntoIterator<Item = (usize, usize, f32)>,
    ) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            nodes: nodes.into_iter().map(Into::into).collect(),
            links: links.into_iter().collect(),
            height: 180.0,
            fmt: None,
            title: None,
        }
    }
}

impl ArcDiagram {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "ArcDiagram");
        let n = self.nodes.len();
        let focus = Focus::new(window, cx, &self.follow, n);
        let shown = focus.shown(cx);
        let deg = degrees(n, &self.links);
        let top = deg.iter().copied().fold(0.0f32, f32::max).max(1e-6);
        let cols: Vec<Hsla> = (0..n).map(|i| hsla(t.palette.series_at(i))).collect();
        let links = self.links.clone();
        let wmax = links
            .iter()
            .map(|l| l.2)
            .filter(|w| w.is_finite())
            .fold(0.0f32, f32::max)
            .max(1e-6);
        let x_of = move |i: usize, bw: f32| (i as f32 + 0.5) / n.max(1) as f32 * bw;
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let paint_deg = deg.clone();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let base = bh - 10.0;
            // The widest arc fits the height; the rest keep its proportions.
            let widest = links
                .iter()
                .map(|l| (x_of(l.0, bw) - x_of(l.1, bw)).abs() / 2.0)
                .fold(1.0f32, f32::max);
            let squash = ((base - 4.0) / widest).min(1.0);
            for &(a, z, wt) in &links {
                if a >= n || z >= n || a == z || !wt.is_finite() {
                    continue;
                }
                let (xa, xz) = (x_of(a, bw), x_of(z, bw));
                let (mid, r) = ((xa + xz) / 2.0, (xz - xa).abs() / 2.0);
                let steps = 32;
                let p: Vec<(f32, f32)> = (0..=((steps as f32 * g) as usize))
                    .map(|k| {
                        let th = std::f32::consts::PI * k as f32 / steps as f32;
                        let from = if xa < xz { -1.0 } else { 1.0 };
                        (mid + from * r * th.cos(), base - r * th.sin() * squash)
                    })
                    .collect();
                let lit = shown.is_none_or(|s| s == a || s == z);
                stroke(
                    w,
                    b.origin,
                    &p,
                    1.0 + 3.0 * (wt / wmax).clamp(0.0, 1.0),
                    scale_alpha(cols[a.min(z)], if lit { 0.7 } else { 0.1 }),
                    false,
                );
            }
            for i in 0..n {
                let r = 3.0
                    + 5.0 * (paint_deg[i] / top).sqrt()
                    + if shown == Some(i) { 2.0 } else { 0.0 };
                fill(
                    w,
                    b.origin,
                    &marker_points(Marker::Dot, x_of(i, bw), base, r),
                    cols[i],
                );
            }
        })
        .absolute()
        .size_full();
        let plot = plot_box(
            self.follow.sub("arcs").element_id(window, cx),
            self.height,
            cv,
        );
        let plot = focus.pointer(plot, &bounds, move |x, _, w, _| {
            let i = (x / w.max(1.0) * n as f32) as usize;
            (i < n).then_some(i)
        });
        let names = label_style(&t, div())
            .flex()
            .w_full()
            .children(self.nodes.iter().map(|s| {
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .justify_center()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(s.clone())
            }));
        let text = shown.and_then(|i| node_text(&self.nodes, &self.links, i));
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(plot)
            .child(names)
            .child(readout(&t, text, "Hover a node"));
        titled(&t, self.title, body)
    }
}

dodai_gpui::entry_render!(ArcDiagram);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_curve_and_distance_helpers_hold() {
        let q = quad((0.0, 0.0), (5.0, 10.0), (10.0, 0.0), 10);
        assert_eq!(q[0], (0.0, 0.0));
        assert_eq!(q[10], (10.0, 0.0));
        assert_eq!(q[5], (5.0, 5.0));
        assert_eq!(seg_dist((5.0, 3.0), (0.0, 0.0), (10.0, 0.0)), 3.0);
        assert_eq!(seg_dist((1.0, 1.0), (0.0, 0.0), (0.0, 0.0)), 2f32.sqrt());
    }

    #[test]
    fn nodes_read_as_their_links() {
        let names: Vec<SharedString> = ["a", "b", "c"].map(SharedString::from).to_vec();
        let links = [(0, 1, 1.0), (2, 0, 2.0), (9, 0, 1.0)];
        assert_eq!(
            node_text(&names, &links, 0).as_deref(),
            Some("a · 2 links: b, c")
        );
        assert_eq!(node_text(&names, &[], 1).as_deref(), Some("b · no links"));
        assert_eq!(degrees(3, &links), vec![3.0, 1.0, 2.0]);
    }

    #[test]
    fn the_chord_hit_follows_the_angle() {
        let groups = [(0.0, 0.5), (0.5, 1.0)];
        // Right of centre is a quarter turn: the first group; left is the second.
        assert_eq!(chord_hit(&groups, 90.0, 50.0, 100.0, 100.0), Some(0));
        assert_eq!(chord_hit(&groups, 10.0, 50.0, 100.0, 100.0), Some(1));
        assert_eq!(chord_hit(&groups, 0.0, 0.0, 100.0, 100.0), None);
    }
}
