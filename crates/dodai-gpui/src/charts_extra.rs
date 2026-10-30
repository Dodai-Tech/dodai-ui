//! [`Sankey`], [`CalendarHeatmap`], [`ViolinChart`] and [`RoseChart`].

use crate::charts::{
    chart_cfg, fmt_with, framed, heat_color, label_style, nice_ticks, norm, titled, turn_arc,
    Legend, NumFmt,
};
use crate::charts_distribution::quartiles;
use crate::charts_focus::{readout, Focus};
use crate::paint::{fill, fill_batch, scale_alpha, stroke};
use crate::{hsla, theme_of};
use dodai_charts_core::{calendar, density, sankey};
use gpui::{
    canvas, div, px, relative, App, Bounds, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

// ─────────────────────────────── Sankey ───────────────────────────────

/// Flow diagram: nodes in columns joined by ribbons whose thickness is the
/// flow. Links are `(from, to, value)` by node name; nodes are created in
/// order of first appearance.
#[derive(IntoElement)]
pub struct Sankey {
    follow: crate::instruments_motion::Follow,
    names: Vec<SharedString>,
    links: Vec<sankey::Link>,
    height: f32,
    node_w: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}

impl Sankey {
    crate::instruments_motion::follow_api!();
    /// Create a new [`Sankey`].
    #[track_caller]
    pub fn new<S: Into<SharedString>>(links: impl IntoIterator<Item = (S, S, f32)>) -> Self {
        let mut names: Vec<SharedString> = Vec::new();
        let mut index = |n: SharedString| {
            names.iter().position(|x| *x == n).unwrap_or_else(|| {
                names.push(n);
                names.len() - 1
            })
        };
        let links = links
            .into_iter()
            .map(|(a, b, v)| {
                let (from, to) = (index(a.into()), index(b.into()));
                sankey::Link { from, to, value: v }
            })
            .collect();
        Self {
            follow: crate::instruments_motion::Follow::here(),
            names,
            links,
            height: 280.0,
            node_w: 12.0,
            fmt: None,
            title: None,
        }
    }
    /// Node bar width in px.
    pub fn node_width(mut self, w: f32) -> Self {
        self.node_w = w.max(2.0);
        self
    }
}
chart_cfg!(Sankey);

impl Sankey {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let flat: Vec<f32> = self.links.iter().map(|l| l.value).collect();
        for (l, v) in self
            .links
            .iter_mut()
            .zip(crate::instruments_motion::follow_all(
                window,
                cx,
                &self.follow,
                &flat,
            ))
        {
            l.value = v;
        }
        let g = crate::motion::grow_for(window, cx, &self.follow, "Sankey");
        let lay = sankey::layout(self.names.len(), &self.links, 0.04);
        let cols = lay.columns.max(1);
        let nw = self.node_w;
        let cols_f = (cols - 1).max(1) as f32;
        let colors: Vec<gpui::Hsla> = (0..self.names.len())
            .map(|i| hsla(t.palette.series_at(i)))
            .collect();
        let (nodes, ribbons) = (lay.nodes.clone(), lay.ribbons.clone());
        let cc = colors.clone();
        // Items: the nodes, then the ribbons. A node lights its ribbons, a ribbon its nodes.
        let nn = nodes.len();
        let focus = Focus::new(window, cx, &self.follow, nn + ribbons.len());
        let shown = focus.shown(cx);
        let lit_ribbon = |k: usize, r: &sankey::Ribbon| match shown {
            None => true,
            Some(s) if s < nn => r.from == s || r.to == s,
            Some(s) => s == nn + k,
        };
        let lit_node = |i: usize| match shown {
            None => true,
            Some(s) if s < nn => s == i,
            Some(s) => ribbons
                .get(s - nn)
                .is_some_and(|r| r.from == i || r.to == i),
        };
        let ribbon_alpha: Vec<f32> = ribbons
            .iter()
            .enumerate()
            .map(|(k, r)| {
                if lit_ribbon(k, r) {
                    0.4 + 0.2 * focus.item(nn + k).lift
                } else {
                    0.08
                }
            })
            .collect();
        let node_alpha: Vec<f32> = (0..nn)
            .map(|i| if lit_node(i) { 1.0 } else { 0.35 })
            .collect();
        let readout_text = shown.and_then(|s| {
            let name = |i: usize| self.names.get(i).map_or("", |n| n.as_ref());
            if s < nn {
                Some(format!(
                    "{} · {}",
                    name(s),
                    fmt_with(&self.fmt, nodes[s].total)
                ))
            } else {
                let r = ribbons.get(s - nn)?;
                let v = self.links.get(r.link).map_or(0.0, |l| l.value);
                Some(format!(
                    "{} → {} · {}",
                    name(r.from),
                    name(r.to),
                    fmt_with(&self.fmt, v)
                ))
            }
        });
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let pb = bounds.clone();
        let (hit_nodes, hit_ribbons) = (nodes.clone(), ribbons.clone());
        let hit = move |x: f32, y: f32, bw: f32, bh: f32| {
            let x_of = |c: usize| c as f32 / cols_f * (bw - nw);
            let node = hit_nodes.iter().position(|n| {
                let x0 = x_of(n.col);
                (x0 - 4.0..x0 + nw + 4.0).contains(&x) && (n.y * bh..(n.y + n.h) * bh).contains(&y)
            });
            node.or_else(|| {
                hit_ribbons
                    .iter()
                    .position(|r| {
                        let (x0, x1) =
                            (x_of(hit_nodes[r.from].col) + nw, x_of(hit_nodes[r.to].col));
                        if !(x0..x1).contains(&x) {
                            return false;
                        }
                        let t = (x - x0) / (x1 - x0);
                        let top = (r.y_from + (r.y_to - r.y_from) * t * t * (3.0 - 2.0 * t)) * bh;
                        (top..top + r.h * bh).contains(&y)
                    })
                    .map(|k| nn + k)
            })
        };
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                pb.set(b);
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let x_of = |c: usize| c as f32 / cols_f * (bw - nw);
                for (k, r) in ribbons.iter().enumerate() {
                    let (x0, x1) = (x_of(nodes[r.from].col) + nw, x_of(nodes[r.to].col));
                    let steps = 24;
                    let polys: Vec<Vec<(f32, f32)>> = (0..steps)
                        .map(|i| {
                            let (a, c) = (i as f32 / steps as f32, (i + 1) as f32 / steps as f32);
                            let s = |t: f32| t * t * (3.0 - 2.0 * t);
                            let top = |t: f32| (r.y_from + (r.y_to - r.y_from) * s(t)) * bh;
                            let h = r.h * bh * g;
                            let (xa, xc) = (x0 + (x1 - x0) * a, x0 + (x1 - x0) * c);
                            vec![
                                (xa, top(a)),
                                (xc, top(c)),
                                (xc, top(c) + h),
                                (xa, top(a) + h),
                            ]
                        })
                        .collect();
                    crate::paint::fill_batch(
                        w,
                        b.origin,
                        &polys,
                        scale_alpha(cc[r.from], ribbon_alpha[k]),
                    );
                }
                for (i, n) in nodes.iter().enumerate() {
                    let x = x_of(n.col);
                    fill(
                        w,
                        b.origin,
                        &crate::charts::rect(x, n.y * bh, nw, (n.h * bh * g).max(1.0)),
                        scale_alpha(cc[i], node_alpha[i]),
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let hits = div()
            .id(self.follow.sub("flows").element_id(window, cx))
            .absolute()
            .size_full()
            .child(cv);
        let mut plot = div()
            .relative()
            .w_full()
            .h(px(self.height))
            .child(focus.pointer(hits, &bounds, hit));
        for (i, n) in lay.nodes.iter().enumerate() {
            if n.h <= 0.0 {
                continue;
            }
            let f = n.col as f32 / cols_f;
            let text = format!("{} · {}", self.names[i], fmt_with(&self.fmt, n.total));
            let last = n.col + 1 == cols && cols > 1;
            let l = label_style(&t, div())
                .absolute()
                .top(relative(n.y + n.h / 2.0))
                .mt(px(-7.0))
                .text_color(hsla(t.palette.text))
                .child(text);
            plot = plot.child(if last {
                l.right_0().mr(px(nw + 4.0))
            } else {
                l.left(relative(f)).ml(px(nw + 4.0 - f * nw))
            });
        }
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(plot)
            .child(readout(&t, readout_text, "Hover a node or a flow"));
        titled(&t, self.title.clone(), body)
    }
}
crate::entry_render!(Sankey);

// ──────────────────────────── CalendarHeatmap ────────────────────────────

/// GitHub-style grid: one cell per day, weeks as columns, intensity in
/// discrete levels, with a legend. Give a start date for month labels.
#[derive(IntoElement)]
pub struct CalendarHeatmap {
    follow: crate::instruments_motion::Follow,
    values: Vec<f32>,
    start: Option<(i32, u32, u32)>,
    start_weekday: u32,
    cell: f32,
    levels: usize,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
    height: f32,
}

impl CalendarHeatmap {
    crate::instruments_motion::follow_api!();
    /// Create a new [`CalendarHeatmap`] from one value per consecutive day.
    #[track_caller]
    pub fn new(values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            values: values
                .into_iter()
                .map(crate::charts::finite_or_zero)
                .collect(),
            start: None,
            start_weekday: 0,
            cell: 14.0,
            levels: 5,
            fmt: None,
            title: None,
            height: 0.0,
        }
    }
    /// First day as `(year, month, day)`: sets the weekday and labels months.
    pub fn from_date(mut self, year: i32, month: u32, day: u32) -> Self {
        self.start = Some((year, month, day));
        self.start_weekday = calendar::weekday(year, month, day);
        self
    }
    /// Weekday of the first value when no date is given (`0` Monday).
    pub fn start_weekday(mut self, d: u32) -> Self {
        self.start_weekday = d % 7;
        self
    }
    /// Cell size in px (default 14).
    pub fn cell(mut self, px: f32) -> Self {
        self.cell = px.max(6.0);
        self
    }
    /// Number of intensity levels including "none" (default 5).
    pub fn levels(mut self, n: usize) -> Self {
        self.levels = n.clamp(2, 9);
        self
    }
}
chart_cfg!(CalendarHeatmap);

/// Intensity level `0..levels` of `v` against `max`. Zero stays level 0.
pub(crate) fn calendar_level(v: f32, max: f32, levels: usize) -> usize {
    if v <= 0.0 || max <= 0.0 {
        return 0;
    }
    let top = levels.saturating_sub(1).max(1);
    (((v / max).clamp(0.0, 1.0) * top as f32).ceil() as usize).clamp(1, top)
}

impl CalendarHeatmap {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        const MONTHS: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let t = theme_of(cx);
        // New data glides: each day's shade steps through the levels to its new one.
        self.values = crate::instruments_motion::follow_all(window, cx, &self.follow, &self.values);
        let focus = Focus::new(window, cx, &self.follow, self.values.len());
        let readout_text = focus.shown(cx).and_then(|i| {
            let v = self.values.get(i)?;
            let day = match self.start {
                Some((y, m, d)) => {
                    let (yy, mm, dd) = calendar::add_days(y, m, d, i as u32);
                    format!("{dd} {} {yy}", MONTHS[(mm - 1) as usize])
                }
                None => format!("Day {}", i + 1),
            };
            Some(format!("{day} · {}", fmt_with(&self.fmt, *v)))
        });
        let max = self.values.iter().copied().fold(0.0f32, f32::max);
        let off = self.start_weekday as usize;
        let weeks = (off + self.values.len()).div_ceil(7).max(1);
        let gap = 3.0;
        let level_color = |l: usize| {
            hsla(heat_color(
                &t,
                l as f32 / self.levels.saturating_sub(1).max(1) as f32,
            ))
        };
        let mut month_at: Vec<(usize, &str)> = Vec::new();
        if let Some((y, m, d)) = self.start {
            let mut last = None;
            for i in 0..self.values.len() {
                let (_, mm, dd) = calendar::add_days(y, m, d, i as u32);
                if (dd == 1 || i == 0) && last != Some(mm) {
                    month_at.push(((off + i) / 7, MONTHS[(mm - 1) as usize]));
                    last = Some(mm);
                }
            }
        }
        // The grid's id scopes the per-day ids, so two calendars never share hover state.
        let mut grid = div()
            .id(self.follow.sub("days").element_id(window, cx))
            .flex()
            .gap(px(gap));
        for wk in 0..weeks {
            let mut col = div().flex().flex_col().gap(px(gap));
            for d in 0..7 {
                let idx = (wk * 7 + d).checked_sub(off);
                let cell = div().w(px(self.cell)).h(px(self.cell)).rounded(px(2.0));
                col = col.child(
                    match idx.and_then(|i| self.values.get(i).map(|v| (i, *v))) {
                        Some((i, v)) => {
                            let c = level_color(calendar_level(v, max, self.levels));
                            let f = focus.item(i);
                            let cell = cell.id(("cal", i)).bg(scale_alpha(c, f.alpha));
                            let cell = if f.lift > 0.01 {
                                cell.border_1().border_color(hsla(t.palette.text))
                            } else {
                                cell
                            };
                            focus.bind(cell, i).into_any_element()
                        }
                        None => cell.into_any_element(),
                    },
                );
            }
            grid = grid.child(col);
        }
        let mut months = div()
            .relative()
            .h(px(14.0))
            .w(px(weeks as f32 * (self.cell + gap)));
        for (wk, name) in month_at {
            months = months.child(
                label_style(&t, div())
                    .absolute()
                    .left(px(wk as f32 * (self.cell + gap)))
                    .child(name),
            );
        }
        let days = label_style(&t, div())
            .flex()
            .flex_col()
            .gap(px(gap))
            .mr(px(t.spacing.xs))
            .children(["Mon", "", "Wed", "", "Fri", "", ""].map(|d| {
                div()
                    .h(px(self.cell))
                    .flex()
                    .items_center()
                    .justify_end()
                    .child(d)
            }));
        let legend = label_style(&t, div())
            .flex()
            .items_center()
            .gap(px(2.0))
            .child("Less")
            .children((0..self.levels).map(|l| {
                div()
                    .w(px(self.cell * 0.8))
                    .h(px(self.cell * 0.8))
                    .rounded(px(2.0))
                    .bg(level_color(l))
            }))
            .child("More");
        titled(
            &t,
            self.title.clone(),
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .child(
                    div().flex().child(days.mt(px(14.0 + t.spacing.xs))).child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(t.spacing.xs))
                            .child(months)
                            .child(grid),
                    ),
                )
                .child(legend)
                .child(readout(&t, readout_text, "Hover a day")),
        )
    }
}
crate::entry_render!(CalendarHeatmap);

// ───────────────────────────── ViolinChart ─────────────────────────────

/// Mirrored density per group with an inner quartile box and median dot.
#[derive(IntoElement)]
pub struct ViolinChart {
    groups: Vec<(SharedString, Vec<f32>)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}

impl ViolinChart {
    /// Create a new [`ViolinChart`] from `(name, samples)` groups.
    pub fn new<S: Into<SharedString>>(groups: impl IntoIterator<Item = (S, Vec<f32>)>) -> Self {
        Self {
            groups: groups.into_iter().map(|(n, v)| (n.into(), v)).collect(),
            height: 220.0,
            fmt: None,
            title: None,
        }
    }
}
chart_cfg!(ViolinChart);

impl ViolinChart {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "ViolinChart");
        let all = self
            .groups
            .iter()
            .flat_map(|g| g.1.iter().copied())
            .filter(|v| v.is_finite());
        let lo = all.clone().fold(f32::INFINITY, f32::min);
        let hi = all.fold(f32::NEG_INFINITY, f32::max);
        let (lo, hi) = if lo.is_finite() && hi > lo {
            (lo, hi)
        } else {
            (0.0, 1.0)
        };
        let ticks = nice_ticks(lo, hi, 4);
        let (lo, hi) = (*ticks.first().unwrap_or(&lo), *ticks.last().unwrap_or(&hi));
        let n = self.groups.len().max(1);
        struct Shape {
            curve: Vec<(f32, f32)>,
            q: Option<(f32, f32, f32, f32, f32)>,
            col: gpui::Hsla,
        }
        let shapes: Vec<Shape> = self
            .groups
            .iter()
            .enumerate()
            .map(|(i, (_, v))| {
                let d = density::kde(v, 96);
                let q = quartiles(v);
                let curve = q
                    .map(|q| {
                        d.y.iter()
                            .enumerate()
                            .map(|(k, y)| {
                                (d.lo + (d.hi - d.lo) * k as f32 / (d.y.len() - 1) as f32, *y)
                            })
                            .filter(|(x, _)| *x >= q.0 && *x <= q.4)
                            .collect()
                    })
                    .unwrap_or_default();
                Shape {
                    curve,
                    q,
                    col: hsla(t.palette.series_at(i)),
                }
            })
            .collect();
        let (text, grid, bg) = (
            hsla(t.palette.text),
            hsla(t.palette.border),
            hsla(t.palette.background),
        );
        let tick_fracs: Vec<f32> = ticks.iter().map(|v| norm(*v, lo, hi)).collect();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                for f in &tick_fracs {
                    stroke(
                        w,
                        b.origin,
                        &[(0.0, bh - f * bh), (bw, bh - f * bh)],
                        1.0,
                        grid,
                        false,
                    );
                }
                let slot = bw / n as f32;
                for (i, s) in shapes.iter().enumerate() {
                    let cx = (i as f32 + 0.5) * slot;
                    let half = slot * 0.42 * g;
                    let pts: Vec<(f32, f32)> = s
                        .curve
                        .iter()
                        .map(|(v, d)| (bh - norm(*v, lo, hi) * bh, d * half))
                        .collect();
                    let quads: Vec<Vec<(f32, f32)>> = pts
                        .windows(2)
                        .map(|p| {
                            vec![
                                (cx - p[0].1, p[0].0),
                                (cx + p[0].1, p[0].0),
                                (cx + p[1].1, p[1].0),
                                (cx - p[1].1, p[1].0),
                            ]
                        })
                        .collect();
                    fill_batch(w, b.origin, &quads, scale_alpha(s.col, 0.5));
                    let right: Vec<(f32, f32)> = pts.iter().map(|p| (cx + p.1, p.0)).collect();
                    let left: Vec<(f32, f32)> = pts.iter().map(|p| (cx - p.1, p.0)).collect();
                    stroke(w, b.origin, &right, 1.5, s.col, false);
                    stroke(w, b.origin, &left, 1.5, s.col, false);
                    if let Some((mn, q1, med, q3, mx)) = s.q {
                        let y = |v: f32| bh - norm(v, lo, hi) * bh;
                        stroke(w, b.origin, &[(cx, y(mn)), (cx, y(mx))], 1.0, text, false);
                        let bx = (slot * 0.04).max(2.0);
                        fill(
                            w,
                            b.origin,
                            &crate::charts::rect(
                                cx - bx,
                                y(q3),
                                bx * 2.0,
                                (y(q1) - y(q3)).max(1.0),
                            ),
                            text,
                        );
                        let r = bx * 1.4;
                        fill(
                            w,
                            b.origin,
                            &[
                                (cx - r, y(med)),
                                (cx, y(med) - r),
                                (cx + r, y(med)),
                                (cx, y(med) + r),
                            ],
                            bg,
                        );
                    }
                }
            },
        )
        .absolute()
        .size_full();
        let plot = div().relative().w_full().h(px(self.height)).child(cv);
        let labels: Vec<SharedString> = self.groups.iter().map(|g| g.0.clone()).collect();
        titled(
            &t,
            self.title.clone(),
            div().w_full().child(framed(
                &t,
                ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
                labels,
                plot,
            )),
        )
    }
}
crate::entry_render!(ViolinChart);

// ─────────────────────────────── RoseChart ───────────────────────────────

/// How a wedge's value maps to its radius.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RoseScale {
    /// Radius ∝ √value, so wedge *area* is proportional (the honest default).
    #[default]
    Area,
    /// Radius ∝ value.
    Linear,
}

/// Polar-area ("Nightingale rose") chart: equal-angle wedges whose reach shows
/// the value, with an optional hole.
#[derive(IntoElement)]
pub struct RoseChart {
    items: Vec<(SharedString, f32)>,
    size: f32,
    hole: f32,
    scale: RoseScale,
    legend: bool,
    title: Option<SharedString>,
}

impl RoseChart {
    /// Create a new [`RoseChart`] from `(label, value)` items.
    pub fn new<S: Into<SharedString>>(items: impl IntoIterator<Item = (S, f32)>) -> Self {
        Self {
            items: items
                .into_iter()
                .map(|(l, v)| (l.into(), crate::charts::finite_or_zero(v).max(0.0)))
                .collect(),
            size: 260.0,
            hole: 0.0,
            scale: RoseScale::Area,
            legend: true,
            title: None,
        }
    }
    /// Diameter in px.
    pub fn size(mut self, px: f32) -> Self {
        self.size = px.max(80.0);
        self
    }
    /// Hole radius as a fraction `0..0.8` of the outer radius.
    pub fn hole(mut self, h: f32) -> Self {
        self.hole = h.clamp(0.0, 0.8);
        self
    }
    /// Value → radius mapping.
    pub fn scale(mut self, s: RoseScale) -> Self {
        self.scale = s;
        self
    }
    /// Show the legend (default on).
    pub fn legend(mut self, on: bool) -> Self {
        self.legend = on;
        self
    }
    /// Title above the chart.
    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = Some(t.into());
        self
    }
}

/// Radius fraction `0..=1` of `v` against `max` for `scale`. Pure.
pub(crate) fn rose_radius(v: f32, max: f32, scale: RoseScale) -> f32 {
    if max <= 0.0 {
        return 0.0;
    }
    let f = (v / max).clamp(0.0, 1.0);
    match scale {
        RoseScale::Area => f.sqrt(),
        RoseScale::Linear => f,
    }
}

impl RoseChart {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "RoseChart");
        let n = self.items.len().max(1);
        let max = self.items.iter().map(|i| i.1).fold(0.0f32, f32::max);
        let (hole, scale) = (self.hole, self.scale);
        let vals: Vec<f32> = self.items.iter().map(|i| i.1).collect();
        let cols: Vec<gpui::Hsla> = (0..n).map(|i| hsla(t.palette.series_at(i))).collect();
        let ring = hsla(t.palette.border);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let (cx, cy) = (bw / 2.0, bh / 2.0);
                let r_out = bw.min(bh) / 2.0 - 2.0;
                let r_in = r_out * hole;
                for k in 1..=3 {
                    let r = r_in + (r_out - r_in) * k as f32 / 3.0;
                    stroke(
                        w,
                        b.origin,
                        &turn_arc(cx, cy, r, 0.0, 1.0),
                        1.0,
                        ring,
                        false,
                    );
                }
                let gap = 0.004;
                for (i, v) in vals.iter().enumerate() {
                    let r = r_in + (r_out - r_in) * rose_radius(*v, max, scale) * g;
                    if r <= r_in + 0.5 {
                        continue;
                    }
                    let (a, c) = (i as f32 / n as f32 + gap, (i + 1) as f32 / n as f32 - gap);
                    let outer = turn_arc(cx, cy, r, a, c);
                    let inner = turn_arc(cx, cy, r_in.max(0.5), a, c);
                    let quads: Vec<Vec<(f32, f32)>> = (0..outer.len() - 1)
                        .map(|k| vec![inner[k], outer[k], outer[k + 1], inner[k + 1]])
                        .collect();
                    fill_batch(w, b.origin, &quads, scale_alpha(cols[i], 0.85));
                    let mut edge = outer.clone();
                    edge.push(*inner.last().unwrap_or(&(cx, cy)));
                    edge.extend(inner.iter().rev().copied());
                    stroke(w, b.origin, &edge, 1.0, cols[i], true);
                }
            },
        )
        .absolute()
        .size_full();
        let mut root = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(div().relative().w(px(self.size)).h(px(self.size)).child(cv));
        if self.legend {
            root = root.child(Legend::new(
                self.items
                    .iter()
                    .enumerate()
                    .map(|(i, (l, _))| (l.clone(), t.palette.series_at(i))),
            ));
        }
        titled(&t, self.title.clone(), root)
    }
}
crate::entry_render!(RoseChart);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_levels() {
        assert_eq!(calendar_level(0.0, 10.0, 5), 0);
        assert_eq!(calendar_level(0.1, 10.0, 5), 1);
        assert_eq!(calendar_level(10.0, 10.0, 5), 4);
        assert_eq!(calendar_level(5.0, 0.0, 5), 0);
        assert_eq!(calendar_level(-3.0, 10.0, 5), 0);
    }

    #[test]
    fn rose_radius_modes() {
        assert!((rose_radius(25.0, 100.0, RoseScale::Area) - 0.5).abs() < 1e-6);
        assert!((rose_radius(25.0, 100.0, RoseScale::Linear) - 0.25).abs() < 1e-6);
        assert_eq!(rose_radius(5.0, 0.0, RoseScale::Area), 0.0);
        assert_eq!(rose_radius(500.0, 100.0, RoseScale::Linear), 1.0);
    }
}
