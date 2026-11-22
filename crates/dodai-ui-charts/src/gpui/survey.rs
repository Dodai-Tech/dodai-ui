//! Survey and quality charts: [`ParetoChart`] (the vital few) and [`DivergingBars`] (Likert
//! scales, population pyramids).

use crate::gpui::common::{
    chart_cfg, fill, fmt_with, label_column, label_style, marker_points, nice_ticks, stroke,
    titled, tracked_canvas, Legend, Marker, NumFmt,
};
use crate::gpui::focus::{readout, Focus};
use crate::{layers::diverging, rank::pareto};
use dodai_ui_gpui::paint::scale_alpha;
use dodai_ui_gpui::{hsla, theme_of};
use gpui::{
    div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// A value column of `labels` (top = last) beside a plot.
fn value_col(t: &dodai_ui_core::Theme, labels: Vec<String>, start: bool) -> gpui::Div {
    label_style(t, div())
        .min_w(px(t.sizes().axis_gutter))
        .flex_none()
        .whitespace_nowrap()
        .flex()
        .flex_col()
        .justify_between()
        .when_else(
            start,
            |d| d.items_start().pl(px(t.spacing.xs)),
            |d| d.items_end().pr(px(t.spacing.xs)),
        )
        .children(labels.into_iter().rev().map(|l| div().child(l)))
}

use gpui::prelude::FluentBuilder as _;

// ─────────────────────────────── ParetoChart ───────────────────────────────

/// Pareto chart: causes as bars, largest first, with the running share of the total as a
/// line on a 0–100 % axis. Bars up to the cut-off (80 % by default) are the "vital few";
/// the rest are drawn fainter.
#[derive(IntoElement)]
pub struct ParetoChart {
    follow: dodai_ui_gpui::follow::Follow,
    items: Vec<(SharedString, f32)>,
    cutoff: f32,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(ParetoChart);

impl ParetoChart {
    dodai_ui_gpui::follow_api!();
    /// Create a new [`ParetoChart`] from `(cause, count)` items in any order.
    #[track_caller]
    pub fn new<S: Into<SharedString>>(items: impl IntoIterator<Item = (S, f32)>) -> Self {
        Self {
            follow: dodai_ui_gpui::follow::Follow::here(),
            items: items.into_iter().map(|(l, v)| (l.into(), v)).collect(),
            cutoff: 0.8,
            height: 220.0,
            fmt: None,
            title: None,
        }
    }
    /// The cumulative share that ends the vital few (default 0.8, `0..=1`).
    pub fn cutoff(mut self, c: f32) -> Self {
        self.cutoff = if c.is_finite() {
            c.clamp(0.0, 1.0)
        } else {
            0.8
        };
        self
    }
}

/// How many of the sorted items it takes to reach `cutoff` of the total (at least one).
fn vital_few(cum: &[f32], cutoff: f32) -> usize {
    cum.iter()
        .position(|c| *c >= cutoff - 1e-6)
        .map_or(cum.len(), |i| i + 1)
}

impl ParetoChart {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        dodai_ui_gpui::follow::follow_pairs(window, cx, &self.follow, &mut self.items);
        let t = theme_of(cx);
        let g = dodai_ui_gpui::motion::grow_for(window, cx, &self.follow, "ParetoChart");
        let values: Vec<f32> = self.items.iter().map(|i| i.1).collect();
        let (order, cum) = pareto(&values);
        let n = order.len();
        let few = vital_few(&cum, self.cutoff);
        let clean = |v: f32| if v.is_finite() { v.max(0.0) } else { 0.0 };
        let top = order.first().map_or(0.0, |i| clean(values[*i]));
        let ticks = nice_ticks(0.0, top.max(1e-6), 4);
        let ymax = *ticks.last().unwrap_or(&1.0);
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|i| focus.item(i)).collect();
        let (bar, line, guide) = (
            hsla(t.palette.series_at(0)),
            hsla(t.palette.series_at(1)),
            hsla(t.palette.text_muted),
        );
        let heights: Vec<f32> = order.iter().map(|i| clean(values[*i]) / ymax).collect();
        let paint_cum = cum.clone();
        let cutoff = self.cutoff;
        let style = t.chart_style();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            for k in 1..4 {
                let y = bh * k as f32 / 4.0;
                crate::gpui::common::grid_line(w, b.origin, (0.0, y), (bw, y), &style);
            }
            let col = bw / n.max(1) as f32;
            for (i, h) in heights.iter().enumerate() {
                let lk = looks[i];
                let gi = dodai_ui_core::motion::reveal(g, i, n);
                let x0 = i as f32 * col + col * 0.15;
                let x1 = (i + 1) as f32 * col - col * 0.15;
                let y0 = bh * (1.0 - h * gi);
                let alpha = if i < few { 1.0 } else { 0.4 };
                fill(
                    w,
                    b.origin,
                    &[(x0, y0), (x1, y0), (x1, bh), (x0, bh)],
                    scale_alpha(bar, alpha * lk.alpha * (0.85 + 0.15 * lk.lift)),
                );
            }
            let yc = bh * (1.0 - cutoff);
            stroke(
                w,
                b.origin,
                &[(0.0, yc), (bw, yc)],
                1.0,
                scale_alpha(guide, 0.7),
                false,
            );
            let p: Vec<(f32, f32)> = paint_cum
                .iter()
                .enumerate()
                .take(((n as f32 * g).ceil() as usize).max(1))
                .map(|(i, c)| ((i as f32 + 0.5) * col, bh * (1.0 - c)))
                .collect();
            stroke(w, b.origin, &p, 2.0, line, false);
            for &(x, y) in &p {
                fill(w, b.origin, &marker_points(Marker::Dot, x, y, 3.0), line);
            }
        })
        .absolute()
        .size_full();
        let plot = div()
            .id(self.follow.sub("bars").element_id(window, cx))
            .relative()
            .flex_1()
            .h(px(self.height))
            .child(cv);
        let plot = focus.pointer(plot, &bounds, move |x, _, w, _| {
            let i = (x / w.max(1.0) * n as f32) as usize;
            (i < n).then_some(i)
        });
        let total: f32 = values.iter().map(|v| clean(*v)).sum();
        let text = focus.shown(cx).and_then(|i| {
            let (name, v) = self.items.get(*order.get(i)?)?;
            Some(format!(
                "{name} · {} · {:.0}% of all · {:.0}% so far{}",
                fmt_with(&self.fmt, *v),
                if total > 0.0 {
                    clean(*v) / total * 100.0
                } else {
                    0.0
                },
                cum[i] * 100.0,
                if i < few { " · vital few" } else { "" }
            ))
        });
        let left = value_col(
            &t,
            ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
            false,
        );
        let right = value_col(
            &t,
            ["0%", "25%", "50%", "75%", "100%"]
                .map(String::from)
                .to_vec(),
            true,
        );
        let names = label_style(&t, div())
            .flex()
            .mx(px(t.sizes().axis_gutter))
            .children(order.iter().map(|i| {
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .justify_center()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(self.items[*i].0.clone())
            }));
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(div().flex().w_full().child(left).child(plot).child(right))
            .child(names)
            .child(readout(&t, text, "Hover a bar"));
        titled(&t, self.title, body)
    }
}

dodai_ui_gpui::entry_render!(ParetoChart);

// ─────────────────────────────── DivergingBars ───────────────────────────────

/// Diverging stacked bars: each row's parts stacked out from a centre line, the first ones
/// to the left. A Likert scale (disagree ← neutral → agree) splits at its middle answer; a
/// population pyramid is two parts split at 1.
#[derive(IntoElement)]
pub struct DivergingBars {
    follow: dodai_ui_gpui::follow::Follow,
    parts: Vec<SharedString>,
    rows: Vec<(SharedString, Vec<f32>)>,
    split: f32,
    percent: bool,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(DivergingBars);

impl DivergingBars {
    dodai_ui_gpui::follow_api!();
    /// Create new [`DivergingBars`]: part names (left to right) and `(row, one value per
    /// part)`. Splits at the middle part (2.5 for 5 parts).
    #[track_caller]
    pub fn new<P: Into<SharedString>, S: Into<SharedString>>(
        parts: impl IntoIterator<Item = P>,
        rows: impl IntoIterator<Item = (S, Vec<f32>)>,
    ) -> Self {
        let parts: Vec<SharedString> = parts.into_iter().map(Into::into).collect();
        let rows: Vec<(SharedString, Vec<f32>)> =
            rows.into_iter().map(|(l, v)| (l.into(), v)).collect();
        Self {
            follow: dodai_ui_gpui::follow::Follow::here(),
            split: parts.len() as f32 / 2.0,
            parts,
            height: (rows.len().max(1) as f32 * 30.0).max(100.0),
            rows,
            percent: true,
            fmt: None,
            title: None,
        }
    }
    /// How many parts sit left of the centre; a fraction puts that part across it.
    pub fn split(mut self, s: f32) -> Self {
        if s.is_finite() {
            self.split = s.clamp(0.0, self.parts.len() as f32);
        }
        self
    }
    /// Scale every row to 100 % (default on; off keeps the raw values, as for a pyramid).
    pub fn percent(mut self, p: bool) -> Self {
        self.percent = p;
        self
    }
}

/// The part colours: each side a ramp of its own series colour, deeper away from the centre;
/// a part across the centre is neutral.
fn diverging_colors(
    t: &dodai_ui_core::Theme,
    parts: usize,
    split: f32,
) -> Vec<dodai_ui_core::Color> {
    let left = split.floor() as usize;
    let neutral = split.fract() > 1e-6;
    let right_from = left + usize::from(neutral);
    (0..parts)
        .map(|i| {
            let (base, depth) = if i < left {
                (
                    t.palette.series_at(0),
                    (left - i) as f32 / left.max(1) as f32,
                )
            } else if neutral && i == left {
                return t.palette.border_strong;
            } else {
                let k = parts.saturating_sub(right_from).max(1);
                (
                    t.palette.series_at(1),
                    (i + 1 - right_from) as f32 / k as f32,
                )
            };
            t.palette.surface.mix(base, 0.35 + 0.65 * depth)
        })
        .collect()
}

impl DivergingBars {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut rows: Vec<Vec<f32>> = self.rows.iter().map(|r| r.1.clone()).collect();
        dodai_ui_gpui::follow::follow_rows(window, cx, &self.follow, &mut rows);
        for (r, v) in self.rows.iter_mut().zip(rows) {
            r.1 = v;
        }
        let t = theme_of(cx);
        let g = dodai_ui_gpui::motion::grow_for(window, cx, &self.follow, "DivergingBars");
        let np = self.parts.len();
        // Rows scaled to 100 % when asked.
        let scaled: Vec<Vec<f32>> = self
            .rows
            .iter()
            .map(|(_, v)| {
                // One value per part: extras dropped, missing ones empty.
                let mut clean: Vec<f32> = v
                    .iter()
                    .map(|x| if x.is_finite() { x.abs() } else { 0.0 })
                    .collect();
                clean.resize(np, 0.0);
                let sum: f32 = clean.iter().sum();
                if self.percent && sum > 0.0 {
                    clean.iter().map(|x| x / sum * 100.0).collect()
                } else {
                    clean
                }
            })
            .collect();
        let spans: Vec<Vec<(f32, f32)>> = scaled.iter().map(|r| diverging(r, self.split)).collect();
        let reach = crate::peak_from(
            spans.iter().flatten().flat_map(|(a, b)| [a.abs(), b.abs()]),
            1e-6,
        );
        let ticks = nice_ticks(0.0, reach, 3);
        let m = *ticks.last().unwrap_or(&1.0);
        let sym: Vec<f32> = ticks
            .iter()
            .rev()
            .map(|v| -v)
            .chain(ticks.iter().skip(1).copied())
            .collect();
        let nr = self.rows.len().max(1);
        let n = self.rows.len() * np;
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|i| focus.item(i)).collect();
        let core_cols = diverging_colors(&t, np, self.split);
        let cols: Vec<Hsla> = core_cols.iter().map(|c| hsla(*c)).collect();
        let axis = hsla(t.palette.text_muted);
        let style = t.chart_style();
        let paint_spans = spans.clone();
        let hit_spans = spans.clone();
        let nt = sym.len();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            for k in 0..nt {
                let x = k as f32 / (nt - 1).max(1) as f32 * bw;
                crate::gpui::common::grid_line(w, b.origin, (x, 0.0), (x, bh), &style);
            }
            let row = bh / nr as f32;
            let x_of = |v: f32| bw / 2.0 + v / m * bw / 2.0 * g;
            for (r, segs) in paint_spans.iter().enumerate() {
                let (y0, y1) = (
                    r as f32 * row + row * 0.18,
                    (r + 1) as f32 * row - row * 0.18,
                );
                for (s, &(a, z)) in segs.iter().enumerate() {
                    let k = r * np + s;
                    let lk = looks[k];
                    let (x0, x1) = (x_of(a), x_of(z));
                    if x1 - x0 < 0.3 {
                        continue;
                    }
                    let grow = lk.lift * 2.0;
                    fill(
                        w,
                        b.origin,
                        &[
                            (x0, y0 - grow),
                            (x1, y0 - grow),
                            (x1, y1 + grow),
                            (x0, y1 + grow),
                        ],
                        scale_alpha(cols[s], lk.alpha),
                    );
                }
            }
            stroke(
                w,
                b.origin,
                &[(bw / 2.0, 0.0), (bw / 2.0, bh)],
                1.5,
                axis,
                false,
            );
        })
        .absolute()
        .size_full();
        let plot = div()
            .id(self.follow.sub("rows").element_id(window, cx))
            .relative()
            .flex_1()
            .h(px(self.height))
            .child(cv);
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            let r = (y / (h / nr as f32)) as usize;
            let v = (x - w / 2.0) / (w / 2.0) * m;
            let s = hit_spans
                .get(r)?
                .iter()
                .position(|(a, z)| (*a..*z).contains(&v))?;
            Some(r * np + s)
        });
        let unit = if self.percent { "%" } else { "" };
        let text = focus.shown(cx).and_then(|k| {
            let (r, s) = (k / np.max(1), k % np.max(1));
            let (name, raw) = self.rows.get(r)?;
            let part = self.parts.get(s)?;
            let shown = scaled.get(r)?.get(s)?;
            Some(if self.percent {
                format!(
                    "{name} · {part}: {shown:.0}% ({})",
                    fmt_with(&self.fmt, *raw.get(s)?)
                )
            } else {
                format!("{name} · {part}: {}", fmt_with(&self.fmt, *raw.get(s)?))
            })
        });
        let names: Vec<SharedString> = self.rows.iter().map(|r| r.0.clone()).collect();
        let gutter = label_column(&t, names.iter().map(|n| n.as_ref()));
        let mut lanes = label_style(&t, div())
            .relative()
            .w(px(gutter))
            .flex_none()
            .h(px(self.height));
        for (i, name) in names.iter().enumerate() {
            lanes = lanes.child(
                div()
                    .absolute()
                    .top(relative(i as f32 / nr as f32))
                    .h(relative(1.0 / nr as f32))
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_end()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(hsla(t.palette.text))
                    .child(name.clone()),
            );
        }
        let abs_fmt = self.fmt.clone();
        let tick_labels: Vec<f32> = sym.iter().map(|v| v.abs()).collect();
        let ticks_row = label_style(&t, div()).flex().justify_between().children(
            tick_labels
                .iter()
                .map(|v| div().child(format!("{}{unit}", fmt_with(&abs_fmt, *v)))),
        );
        let legend = Legend::new(
            self.parts
                .iter()
                .enumerate()
                .map(|(i, p)| (p.clone(), core_cols[i])),
        );
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(div().flex().gap(px(t.spacing.sm)).child(lanes).child(plot))
            .child(div().ml(px(gutter + t.spacing.sm)).child(ticks_row))
            .child(legend)
            .child(readout(&t, text, "Hover a segment"));
        titled(&t, self.title, body)
    }
}

dodai_ui_gpui::entry_render!(DivergingBars);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_vital_few_reach_the_cutoff() {
        assert_eq!(vital_few(&[0.5, 0.8, 0.95, 1.0], 0.8), 2);
        assert_eq!(vital_few(&[0.5, 0.7], 0.8), 2);
        assert_eq!(vital_few(&[1.0], 0.0), 1);
    }

    #[test]
    fn a_neutral_middle_part_gets_the_neutral_colour() {
        let t = dodai_ui_core::switcher().current().clone();
        let c = diverging_colors(&t, 5, 2.5);
        assert_eq!(c.len(), 5);
        assert_eq!(c[2], t.palette.border_strong);
        assert_ne!(c[0], c[1]);
        assert_eq!(diverging_colors(&t, 2, 1.0).len(), 2);
    }
}
