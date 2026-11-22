//! Distribution charts that show every sample or its spread: [`Beeswarm`], [`Ecdf`] and
//! [`ErrorBars`] (a forest plot).

use crate::gpui::common::{
    chart_cfg, fill, fmt_with, label_column, label_style, marker_points, nice_ticks, norm,
    plot_box, series_color, step_points, stroke, tick_row, titled, tracked_canvas, Legend, Marker,
    NumFmt, Series,
};
use crate::gpui::focus::{readout, Focus};
use crate::{density::ecdf, swarm::swarm};
use dodai_ui_gpui::paint::scale_alpha;
use dodai_ui_gpui::{hsla, theme_of};
use gpui::{
    div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, Styled, Window,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Nice ticks covering every finite value (0..1 when there are none).
fn ticks_of<'a>(values: impl Iterator<Item = &'a f32>) -> Vec<f32> {
    let (lo, hi) = crate::padded_extent(values.copied());
    nice_ticks(lo, hi, 5)
}

/// A row-label column beside a plot of `rows` equal rows.
fn lane_labels(t: &dodai_ui_core::Theme, names: &[SharedString], w: f32, h: f32) -> gpui::Div {
    let n = names.len().max(1);
    let mut col = label_style(t, div())
        .relative()
        .w(px(w))
        .flex_none()
        .h(px(h));
    for (i, name) in names.iter().enumerate() {
        col = col.child(
            div()
                .absolute()
                .top(relative(i as f32 / n as f32))
                .h(relative(1.0 / n as f32))
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
    col
}

// ─────────────────────────────── Beeswarm ───────────────────────────────

/// Beeswarm: every sample as a dot on a shared value axis, one lane per group, nudged
/// sideways so none overlap. Shows the shape of a distribution and each outlier.
#[derive(IntoElement)]
pub struct Beeswarm {
    follow: dodai_ui_gpui::follow::Follow,
    groups: Vec<(SharedString, Vec<f32>)>,
    radius: f32,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(Beeswarm);

impl Beeswarm {
    dodai_ui_gpui::follow_api!();
    /// Create a new [`Beeswarm`] from `(label, samples)` groups, one lane each.
    #[track_caller]
    pub fn new<S: Into<SharedString>>(groups: impl IntoIterator<Item = (S, Vec<f32>)>) -> Self {
        let groups: Vec<(SharedString, Vec<f32>)> =
            groups.into_iter().map(|(l, v)| (l.into(), v)).collect();
        Self {
            follow: dodai_ui_gpui::follow::Follow::here(),
            height: (groups.len().max(1) as f32 * 70.0).max(120.0),
            groups,
            radius: 3.5,
            fmt: None,
            title: None,
        }
    }
    /// Dot radius in px (default 3.5, `1..=10`).
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = if r.is_finite() {
            r.clamp(1.0, 10.0)
        } else {
            3.5
        };
        self
    }
}

impl Beeswarm {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_ui_gpui::motion::grow_for(window, cx, &self.follow, "Beeswarm");
        let ticks = ticks_of(self.groups.iter().flat_map(|(_, v)| v.iter()));
        let (lo, hi) = (ticks[0], ticks[ticks.len() - 1]);
        // Flat sample list: (group, value).
        let samples: Vec<(usize, f32)> = self
            .groups
            .iter()
            .enumerate()
            .flat_map(|(k, (_, v))| v.iter().filter(|x| x.is_finite()).map(move |x| (k, *x)))
            .collect();
        let lanes = self.groups.len().max(1);
        let focus = Focus::new(window, cx, &self.follow, samples.len());
        let looks: Vec<_> = (0..samples.len()).map(|i| focus.item(i)).collect();
        let cols: Vec<Hsla> = (0..lanes).map(|i| hsla(t.palette.series_at(i))).collect();
        let r = self.radius;
        let style = t.chart_style();
        // Dot centres, kept from the last paint for hit testing.
        let placed: Rc<RefCell<Vec<(f32, f32)>>> = Default::default();
        let painted = placed.clone();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let paint_samples = samples.clone();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let lane = bh / lanes as f32;
            for k in 1..lanes {
                let y = k as f32 * lane;
                crate::gpui::common::grid_line(w, b.origin, (0.0, y), (bw, y), &style);
            }
            let mut at = vec![(0.0, 0.0); paint_samples.len()];
            for k in 0..lanes {
                let idx: Vec<usize> = (0..paint_samples.len())
                    .filter(|i| paint_samples[*i].0 == k)
                    .collect();
                let xs: Vec<f32> = idx
                    .iter()
                    .map(|i| norm(paint_samples[*i].1, lo, hi) * bw)
                    .collect();
                let half = lane / 2.0 - r;
                for (j, dy) in idx.iter().zip(swarm(&xs, r + 0.5)) {
                    let x = norm(paint_samples[*j].1, lo, hi) * bw;
                    // A swarm too tall for its lane is squeezed, never spilled.
                    at[*j] = (x, (k as f32 + 0.5) * lane + dy.clamp(-half, half));
                }
            }
            for (i, &(x, y)) in at.iter().enumerate() {
                let gi = dodai_ui_core::motion::reveal(g, i, at.len());
                if gi <= 0.0 {
                    continue;
                }
                let lk = looks[i];
                let dot = marker_points(Marker::Dot, x, y, (r + lk.lift * 2.0) * gi);
                fill(
                    w,
                    b.origin,
                    &dot,
                    scale_alpha(cols[paint_samples[i].0], lk.alpha),
                );
            }
            *painted.borrow_mut() = at;
        })
        .absolute()
        .size_full();
        let plot = div()
            .id(self.follow.sub("swarm").element_id(window, cx))
            .relative()
            .flex_1()
            .h(px(self.height))
            .child(cv);
        let plot = focus.pointer(plot, &bounds, move |x, y, _, _| {
            placed
                .borrow()
                .iter()
                .enumerate()
                .map(|(i, (px_, py))| (i, (px_ - x).powi(2) + (py - y).powi(2)))
                .filter(|(_, d)| *d <= (r + 3.0).powi(2))
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| i)
        });
        let names: Vec<SharedString> = self.groups.iter().map(|g| g.0.clone()).collect();
        let gutter = label_column(&t, names.iter().map(|n| n.as_ref()));
        let text = focus.shown(cx).and_then(|i| {
            let (k, v) = samples.get(i)?;
            Some(format!("{} · {}", names.get(*k)?, fmt_with(&self.fmt, *v)))
        });
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(
                div()
                    .flex()
                    .gap(px(t.spacing.sm))
                    .child(lane_labels(&t, &names, gutter, self.height))
                    .child(plot),
            )
            .child(
                div()
                    .ml(px(gutter + t.spacing.sm))
                    .child(tick_row(&t, &ticks, &self.fmt)),
            )
            .child(readout(&t, text, "Hover a dot"));
        titled(&t, self.title, body)
    }
}

dodai_ui_gpui::entry_render!(Beeswarm);

// ─────────────────────────────── Ecdf ───────────────────────────────

/// Empirical cumulative distribution: for each value, the share of samples at or below it.
/// Compares whole distributions without binning; medians and tails read straight off the
/// 50 % and 90 % lines. Each [`Series`]' values are its samples.
#[derive(IntoElement)]
pub struct Ecdf {
    follow: dodai_ui_gpui::follow::Follow,
    series: Vec<Series>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(Ecdf);

impl Ecdf {
    dodai_ui_gpui::follow_api!();
    /// Create a new [`Ecdf`], one curve per series of samples.
    #[track_caller]
    pub fn new(series: impl IntoIterator<Item = Series>) -> Self {
        Self {
            follow: dodai_ui_gpui::follow::Follow::here(),
            series: series.into_iter().collect(),
            height: 200.0,
            fmt: None,
            title: None,
        }
    }
}

/// The value below which `q` of the steps lie (the first step reaching `q`).
fn quantile(steps: &[(f32, f32)], q: f32) -> Option<f32> {
    steps.iter().find(|(_, f)| *f >= q - 1e-6).map(|(v, _)| *v)
}

impl Ecdf {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_ui_gpui::motion::grow_for(window, cx, &self.follow, "Ecdf");
        let curves: Vec<Vec<(f32, f32)>> = self.series.iter().map(|s| ecdf(&s.values)).collect();
        let ticks = ticks_of(self.series.iter().flat_map(|s| s.values.iter()));
        let (lo, hi) = (ticks[0], ticks[ticks.len() - 1]);
        let n = self.series.len();
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|i| focus.item(i)).collect();
        let cols: Vec<Hsla> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| hsla(series_color(&t, s, i)))
            .collect();
        let style = t.chart_style();
        let guide = hsla(t.palette.text_muted);
        let paint_curves = curves.clone();
        let hit_curves = curves.clone();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            for f in [0.25, 0.5, 0.75] {
                let y = bh * (1.0 - f);
                crate::gpui::common::grid_line(w, b.origin, (0.0, y), (bw, y), &style);
            }
            for q in [0.5, 0.9] {
                let y = bh * (1.0 - q);
                stroke(
                    w,
                    b.origin,
                    &[(0.0, y), (bw, y)],
                    1.0,
                    scale_alpha(guide, 0.5),
                    false,
                );
            }
            for (i, c) in paint_curves.iter().enumerate() {
                if c.is_empty() {
                    continue;
                }
                let lk = looks[i];
                // From 0 at the left edge, up the steps, flat to the right edge.
                let mut p = vec![(0.0, bh)];
                p.extend(
                    c.iter()
                        .map(|(v, f)| (norm(*v, lo, hi) * bw, bh * (1.0 - f * g))),
                );
                p.push((bw, bh * (1.0 - g)));
                let p = step_points(&p);
                stroke(
                    w,
                    b.origin,
                    &p,
                    2.0 + lk.lift * 1.5,
                    scale_alpha(cols[i], lk.alpha),
                    false,
                );
            }
        })
        .absolute()
        .size_full();
        let plot = plot_box(
            self.follow.sub("ecdf").element_id(window, cx),
            self.height,
            cv,
        );
        // The curve nearest the pointer, by its height at the pointer's value.
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            let v = lo + (hi - lo) * x / w.max(1.0);
            let f = 1.0 - y / h.max(1.0);
            hit_curves
                .iter()
                .enumerate()
                .filter(|(_, c)| !c.is_empty())
                .map(|(i, c)| {
                    let at = c.iter().rev().find(|(cv, _)| *cv <= v).map_or(0.0, |s| s.1);
                    (i, (at - f).abs())
                })
                .filter(|(_, d)| *d < 0.12)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| i)
        });
        let text = focus.shown(cx).and_then(|i| {
            let c = curves.get(i)?;
            let s = self.series.get(i)?;
            Some(format!(
                "{} · median {} · 90% below {} · n = {}",
                s.name,
                fmt_with(&self.fmt, quantile(c, 0.5)?),
                fmt_with(&self.fmt, quantile(c, 0.9)?),
                s.values.iter().filter(|v| v.is_finite()).count()
            ))
        });
        let body = crate::gpui::common::framed_row(
            &t,
            ["0%", "25%", "50%", "75%", "100%"]
                .map(String::from)
                .to_vec(),
            Some(tick_row(&t, &ticks, &self.fmt)),
            div().w_full().child(plot),
        );
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(body)
            .child(Legend::of_series(&t, &self.series))
            .child(readout(&t, text, "Hover a curve"));
        titled(&t, self.title, body)
    }
}

dodai_ui_gpui::entry_render!(Ecdf);

// ─────────────────────────────── ErrorBars ───────────────────────────────

/// Error bars (a forest plot): one row per estimate, a mark at the value and a whisker over
/// its interval, with an optional reference line (no effect, target, last year).
#[derive(IntoElement)]
pub struct ErrorBars {
    follow: dodai_ui_gpui::follow::Follow,
    rows: Vec<(SharedString, f32, f32, f32)>,
    reference: Option<f32>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(ErrorBars);

impl ErrorBars {
    dodai_ui_gpui::follow_api!();
    /// Create new [`ErrorBars`] from `(label, estimate, low, high)` rows, top to bottom.
    #[track_caller]
    pub fn new<S: Into<SharedString>>(rows: impl IntoIterator<Item = (S, f32, f32, f32)>) -> Self {
        let rows: Vec<(SharedString, f32, f32, f32)> = rows
            .into_iter()
            .map(|(l, e, a, b)| (l.into(), e, a.min(b), a.max(b)))
            .collect();
        Self {
            follow: dodai_ui_gpui::follow::Follow::here(),
            height: (rows.len().max(1) as f32 * 28.0).max(100.0),
            rows,
            reference: None,
            fmt: None,
            title: None,
        }
    }
    /// A vertical reference line at `v`.
    pub fn reference(mut self, v: f32) -> Self {
        self.reference = v.is_finite().then_some(v);
        self
    }
}

impl ErrorBars {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut flat: Vec<f32> = self.rows.iter().flat_map(|r| [r.1, r.2, r.3]).collect();
        flat = dodai_ui_gpui::follow::follow_all(window, cx, &self.follow.sub("rows"), &flat);
        for (r, p) in self.rows.iter_mut().zip(flat.chunks(3)) {
            if let [e, a, b] = p {
                (r.1, r.2, r.3) = (*e, *a, *b);
            }
        }
        let t = theme_of(cx);
        let g = dodai_ui_gpui::motion::grow_for(window, cx, &self.follow, "ErrorBars");
        let ticks = ticks_of(
            self.rows
                .iter()
                .flat_map(|r| [&r.1, &r.2, &r.3])
                .chain(self.reference.as_ref()),
        );
        let (lo, hi) = (ticks[0], ticks[ticks.len() - 1]);
        let n = self.rows.len().max(1);
        let focus = Focus::new(window, cx, &self.follow, self.rows.len());
        let looks: Vec<_> = (0..self.rows.len()).map(|i| focus.item(i)).collect();
        let ink = hsla(t.palette.primary);
        let refc = hsla(t.palette.text_muted);
        let style = t.chart_style();
        let geo: Vec<(f32, f32, f32)> = self.rows.iter().map(|r| (r.1, r.2, r.3)).collect();
        let reference = self.reference;
        let grid_n = ticks.len();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let row = bh / n as f32;
            for k in 0..grid_n {
                let x = k as f32 / (grid_n - 1).max(1) as f32 * bw;
                crate::gpui::common::grid_line(w, b.origin, (x, 0.0), (x, bh), &style);
            }
            if let Some(r) = reference {
                let x = norm(r, lo, hi) * bw;
                stroke(w, b.origin, &[(x, 0.0), (x, bh)], 1.5, refc, false);
            }
            for (i, &(e, a, z)) in geo.iter().enumerate() {
                let lk = looks[i];
                let c = scale_alpha(ink, lk.alpha);
                let y = (i as f32 + 0.5) * row;
                let xe = norm(e, lo, hi) * bw;
                // Whiskers open out from the estimate.
                let gi = dodai_ui_core::motion::reveal(g, i, n);
                let xa = xe + (norm(a, lo, hi) * bw - xe) * gi;
                let xz = xe + (norm(z, lo, hi) * bw - xe) * gi;
                let cap = (row * 0.25).min(6.0);
                stroke(w, b.origin, &[(xa, y), (xz, y)], 1.5 + lk.lift, c, false);
                for x in [xa, xz] {
                    stroke(w, b.origin, &[(x, y - cap), (x, y + cap)], 1.5, c, false);
                }
                let mark = marker_points(Marker::Square, xe, y, 4.0 + lk.lift * 2.0);
                fill(w, b.origin, &mark, c);
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
        let rows = self.rows.len();
        let plot = focus.pointer(plot, &bounds, move |_, y, _, h| {
            let i = (y / (h / n as f32)) as usize;
            (i < rows).then_some(i)
        });
        let names: Vec<SharedString> = self.rows.iter().map(|r| r.0.clone()).collect();
        let gutter = label_column(&t, names.iter().map(|n| n.as_ref()));
        let text = focus.shown(cx).and_then(|i| {
            let (l, e, a, b) = self.rows.get(i)?;
            Some(format!(
                "{l} · {} [{} – {}]",
                fmt_with(&self.fmt, *e),
                fmt_with(&self.fmt, *a),
                fmt_with(&self.fmt, *b)
            ))
        });
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(
                div()
                    .flex()
                    .gap(px(t.spacing.sm))
                    .child(lane_labels(&t, &names, gutter, self.height))
                    .child(plot),
            )
            .child(
                div()
                    .ml(px(gutter + t.spacing.sm))
                    .child(tick_row(&t, &ticks, &self.fmt)),
            )
            .child(readout(&t, text, "Hover a row"));
        titled(&t, self.title, body)
    }
}

dodai_ui_gpui::entry_render!(ErrorBars);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantiles_read_off_the_steps() {
        let s = ecdf(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(quantile(&s, 0.5), Some(2.0));
        assert_eq!(quantile(&s, 0.9), Some(4.0));
        assert_eq!(quantile(&[], 0.5), None);
    }

    #[test]
    fn ticks_cover_the_data_and_survive_hostile_input() {
        let t = ticks_of([1.0, 9.0, f32::NAN].iter());
        assert!(t[0] <= 1.0 && *t.last().unwrap() >= 9.0);
        let empty = ticks_of([].iter());
        assert!(empty[0] <= 0.0 && *empty.last().unwrap() >= 1.0);
        let one = ticks_of([5.0].iter());
        assert!(one[0] <= 4.0 && *one.last().unwrap() >= 6.0);
        let e = ErrorBars::new([("a", 1.0, 3.0, 0.0)]).reference(f32::NAN);
        assert_eq!((e.rows[0].2, e.rows[0].3, e.reference), (0.0, 3.0, None));
    }
}
