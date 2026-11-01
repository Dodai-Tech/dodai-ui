//! Histogram, box plot and bubble charts (distribution of values).

use crate::common::{
    chart_cfg, fill, fmt_with, framed, nice_ticks, norm, stroke, titled, tracked_canvas, NumFmt,
};
use crate::focus::{columns, readout, Focus};
use dodai_core::Color;
use dodai_gpui::paint::{scale_alpha, shaped, ShapeStyle};
use dodai_gpui::panel::DodaiTooltip;
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// Equal-width histogram: `(lo, hi, count)` per bin. Pure.
pub fn histogram_bins(values: &[f32], bins: usize) -> Vec<(f32, f32, usize)> {
    let vals: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
    let bins = bins.max(1);
    if vals.is_empty() {
        return Vec::new();
    }
    let lo = vals.iter().copied().fold(f32::MAX, f32::min);
    let hi = vals.iter().copied().fold(f32::MIN, f32::max);
    let span = if hi > lo { hi - lo } else { 1.0 };
    let w = span / bins as f32;
    let mut out: Vec<(f32, f32, usize)> = (0..bins)
        .map(|i| (lo + w * i as f32, lo + w * (i + 1) as f32, 0))
        .collect();
    for v in vals {
        let i = (((v - lo) / w) as usize).min(bins - 1);
        out[i].2 += 1;
    }
    out
}

/// `(min, q1, median, q3, max)` of `values` (linear interpolation). Pure.
pub fn quartiles(values: &[f32]) -> Option<(f32, f32, f32, f32, f32)> {
    let mut v: Vec<f32> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    let q = |p: f32| {
        let pos = p * (v.len() - 1) as f32;
        let (i, f) = (pos.floor() as usize, pos.fract());
        let j = (i + 1).min(v.len() - 1);
        v[i] + (v[j] - v[i]) * f
    };
    Some((v[0], q(0.25), q(0.5), q(0.75), v[v.len() - 1]))
}

type Stats = (f32, f32, f32, f32, f32);

// ─────────────────────────────── Histogram ───────────────────────────────

/// Histogram of raw values with equal-width bins.
#[derive(IntoElement)]
pub struct Histogram {
    follow: dodai_gpui::follow::Follow,
    annotations: Vec<crate::annotate::Annotation>,
    kde: bool,
    stats: bool,
    id: SharedString,
    values: Vec<f32>,
    bins: usize,
    height: f32,
    color: Option<Color>,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
    corners: Option<dodai_core::Corners>,
}

dodai_gpui::corners_setter!(Histogram);
chart_cfg!(Histogram);

impl Histogram {
    /// Threshold lines and highlighted bands drawn over the plot.
    pub fn annotations(mut self, a: impl IntoIterator<Item = crate::annotate::Annotation>) -> Self {
        self.annotations = a.into_iter().collect();
        self
    }
    dodai_gpui::follow_api!();
    /// Create a new [`Histogram`].
    #[track_caller]
    pub fn new(values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            annotations: Vec::new(),
            kde: false,
            stats: false,
            corners: None,
            id: SharedString::default(),
            values: values.into_iter().collect(),
            bins: 10,
            height: 180.0,
            color: None,
            fmt: None,
            title: None,
        }
    }
    /// Overlay a kernel-density curve (scaled to the tallest bin).
    pub fn kde(mut self, on: bool) -> Self {
        self.kde = on;
        self
    }
    /// Mark the mean (solid) and median (dashed) with labelled lines.
    pub fn stats(mut self, on: bool) -> Self {
        self.stats = on;
        self
    }
    /// Number of bins.
    pub fn bins(mut self, n: usize) -> Self {
        self.bins = n.max(1);
        self
    }
    /// Colour (default: the theme's).
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

impl Histogram {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.id = self.follow.element_id(window, cx).to_string().into();
        let t = theme_of(cx);
        let t = dodai_gpui::apply_corners(t, self.corners);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "Histogram");
        let bins = histogram_bins(&self.values, self.bins);
        let counts: Vec<f32> = bins.iter().map(|b| b.2 as f32).collect();
        let shown = dodai_gpui::follow::follow_all(window, cx, &self.follow, &counts);
        let maxc = bins.iter().map(|b| b.2).max().unwrap_or(1) as f32;
        let ticks = nice_ticks(0.0, maxc, 4);
        let top = *ticks.last().unwrap_or(&1.0);
        let col = self.color.unwrap_or(t.palette.series_at(0));
        let grid = hsla(t.palette.border);
        let mut plot = div().relative().w_full().h(px(self.height));
        let mut anns = self.annotations.clone();
        if self.stats && !bins.is_empty() && !self.values.is_empty() {
            let (lo, hi) = (bins[0].0, bins[bins.len() - 1].1);
            let at = |v: f32| norm(v, lo, hi) * bins.len() as f32 - 0.5;
            let mean = self.values.iter().sum::<f32>() / self.values.len() as f32;
            let mut sorted = self.values.clone();
            sorted.sort_by(|a, b| a.total_cmp(b));
            let med = sorted[sorted.len() / 2];
            anns.push(crate::annotate::Annotation::vline(at(mean)).label(format!(
                "mean {} · median {}",
                fmt_with(&self.fmt, mean),
                fmt_with(&self.fmt, med)
            )));
            anns.push(crate::annotate::Annotation::vline(at(med)).color(t.palette.warning));
        }
        plot = plot.child(crate::annotate::layer(
            &t,
            &anns,
            (0.0, top),
            &crate::annotate::category_x(bins.len()),
        ));

        for v in &ticks {
            plot = plot.child(
                div()
                    .absolute()
                    .left_0()
                    .w_full()
                    .h(px(1.0))
                    .bottom(relative(norm(*v, 0.0, top)))
                    .bg(grid),
            );
        }
        let mut row = div().flex().size_full().items_end();
        for (i, (lo, hi, c)) in bins.iter().enumerate() {
            let style = ShapeStyle::from_tokens(&t, Some(col), None, false, false);
            row = row.child(
                div()
                    .id((self.id.clone(), i))
                    .h_full()
                    .flex_1()
                    .flex()
                    .items_end()
                    .px(px(1.0))
                    .tooltip(DodaiTooltip::text(format!(
                        "{} – {}: {c}",
                        fmt_with(&self.fmt, *lo),
                        fmt_with(&self.fmt, *hi)
                    )))
                    .child(
                        div()
                            .w_full()
                            .h(relative(
                                norm(shown.get(i).copied().unwrap_or(*c as f32), 0.0, top) * g,
                            ))
                            .child(shaped(&t.shape, style).size_full()),
                    ),
            );
        }
        plot = plot.child(row);
        if self.kde && !bins.is_empty() {
            let d = dodai_charts_core::density::kde(&self.values, 96);
            let (blo, bhi) = (bins[0].0, bins[bins.len() - 1].1);
            let line = hsla(t.palette.series_at(1));
            let k = maxc / top;
            plot = plot.child(
                div().absolute().top_0().left_0().size_full().child(
                    canvas(
                        |_, _, _| (),
                        move |b: Bounds<Pixels>, _, w, _| {
                            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                            let n = d.y.len();
                            let pts: Vec<(f32, f32)> = d
                                .y
                                .iter()
                                .enumerate()
                                .map(|(i, y)| {
                                    let v = d.lo + (d.hi - d.lo) * i as f32 / (n.max(2) - 1) as f32;
                                    (norm(v, blo, bhi) * bw, bh - y * k * g * bh)
                                })
                                .filter(|p| p.0 >= 0.0 && p.0 <= bw)
                                .collect();
                            stroke(w, b.origin, &pts, 2.0, line, false);
                        },
                    )
                    .size_full(),
                ),
            );
        }
        let x_labels: Vec<SharedString> = bins
            .iter()
            .map(|b| fmt_with(&self.fmt, b.0).into())
            .collect();
        let root = div().flex().flex_col().w_full().child(framed(
            &t,
            ticks.iter().map(|v| format!("{v}")).collect(),
            x_labels,
            plot,
        ));
        titled(&t, self.title, root)
    }
}

dodai_gpui::entry_render!(Histogram);

/// Outline of a notched box (screen coords, y down): the notch narrows to
/// 60% width at the median and spans its ~95% interval, clamped to the box.
fn notched_box(
    x: f32,
    half: f32,
    (q1, md, q3): (f32, f32, f32),
    n: usize,
    y: &dyn Fn(f32) -> f32,
) -> Vec<(f32, f32)> {
    let ci = 1.57 * (q3 - q1) / (n.max(1) as f32).sqrt();
    let (nl, nh) = ((md - ci).max(q1), (md + ci).min(q3));
    let inner = half * 0.6;
    vec![
        (x - half, y(q3)),
        (x + half, y(q3)),
        (x + half, y(nh)),
        (x + inner, y(md)),
        (x + half, y(nl)),
        (x + half, y(q1)),
        (x - half, y(q1)),
        (x - half, y(nl)),
        (x - inner, y(md)),
        (x - half, y(nh)),
    ]
}

// ─────────────────────────────── BoxPlot ───────────────────────────────

/// Box-and-whisker plot from raw samples per category.
#[derive(IntoElement)]
pub struct BoxPlot {
    follow: dodai_gpui::follow::Follow,
    notch: bool,
    mean: bool,
    points: bool,
    groups: Vec<(SharedString, Vec<f32>)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(BoxPlot);

impl BoxPlot {
    dodai_gpui::follow_api!();
    /// Create a new [`BoxPlot`].
    #[track_caller]
    pub fn new<S: Into<SharedString>>(groups: impl IntoIterator<Item = (S, Vec<f32>)>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            notch: false,
            mean: false,
            points: false,
            groups: groups.into_iter().map(|(a, b)| (a.into(), b)).collect(),
            height: 220.0,
            fmt: None,
            title: None,
        }
    }
}

impl BoxPlot {
    /// Notched boxes: the notch is the ~95% confidence interval of the median.
    pub fn notch(mut self, on: bool) -> Self {
        self.notch = on;
        self
    }
    /// Mark each group's mean with a diamond.
    pub fn mean(mut self, on: bool) -> Self {
        self.mean = on;
        self
    }
    /// Draw every sample as a jittered dot behind the box.
    pub fn points(mut self, on: bool) -> Self {
        self.points = on;
        self
    }
}

impl BoxPlot {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let flat: Vec<f32> = self
            .groups
            .iter()
            .flat_map(|g| g.1.iter().copied())
            .collect();
        let mut shown = dodai_gpui::follow::follow_all(window, cx, &self.follow, &flat).into_iter();
        for v in self.groups.iter_mut().flat_map(|g| g.1.iter_mut()) {
            *v = shown.next().unwrap_or(*v);
        }
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "BoxPlot");
        let t = theme_of(cx);
        let stats: Vec<Option<Stats>> = self.groups.iter().map(|g| quartiles(&g.1)).collect();
        let (lo, hi, ticks) =
            dodai_charts_core::tick_span(stats.iter().flatten().flat_map(|s| [s.0, s.4]), 4);
        let grid = hsla(t.palette.border);
        let text = hsla(t.palette.text);
        let cols: Vec<Hsla> = (0..stats.len())
            .map(|i| hsla(t.palette.series_at(i)))
            .collect();
        let n = stats.len();
        let focus = Focus::new(window, cx, &self.follow, n);
        let cols: Vec<Hsla> = cols
            .iter()
            .enumerate()
            .map(|(i, c)| scale_alpha(*c, focus.item(i).alpha))
            .collect();
        let shown = focus.shown(cx);
        let readout_text = shown.and_then(|i| {
            let (mn, q1, md, q3, mx) = stats.get(i).copied().flatten()?;
            let f = |v: f32| fmt_with(&self.fmt, v);
            Some(format!(
                "{} · median {} · middle half {}–{} · range {}–{} · {} samples",
                self.groups[i].0,
                f(md),
                f(q1),
                f(q3),
                f(mn),
                f(mx),
                self.groups[i].1.len()
            ))
        });
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let (notch, show_mean, show_pts) = (self.notch, self.mean, self.points);
        let raw: Vec<Vec<f32>> = self.groups.iter().map(|g| g.1.clone()).collect();
        let tf: Vec<f32> = ticks.iter().map(|v| norm(*v, lo, hi)).collect();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            for f in &tf {
                let y = bh - f * bh;
                stroke(w, b.origin, &[(0.0, y), (bw, y)], 1.0, grid, false);
            }
            let slot = bw / n.max(1) as f32;
            let yr = |v: f32| bh - norm(v, lo, hi) * bh;
            for (i, s) in stats.iter().enumerate() {
                let Some((mn, q1, md, q3, mx)) = s else {
                    continue;
                };
                let x = (i as f32 + 0.5) * slot;
                let ym = yr(*md);
                let gi = dodai_gpui::motion::reveal(g, i, n);
                let y = |v: f32| ym + (yr(v) - ym) * gi;
                let half = (slot * 0.5).min(40.0) / 2.0;
                stroke(
                    w,
                    b.origin,
                    &[(x, y(*mx)), (x, y(*q3))],
                    1.5,
                    cols[i],
                    false,
                );
                stroke(
                    w,
                    b.origin,
                    &[(x, y(*q1)), (x, y(*mn))],
                    1.5,
                    cols[i],
                    false,
                );
                stroke(
                    w,
                    b.origin,
                    &[(x - half / 2.0, y(*mx)), (x + half / 2.0, y(*mx))],
                    1.5,
                    cols[i],
                    false,
                );
                stroke(
                    w,
                    b.origin,
                    &[(x - half / 2.0, y(*mn)), (x + half / 2.0, y(*mn))],
                    1.5,
                    cols[i],
                    false,
                );
                if show_pts {
                    for (k, v) in raw[i].iter().enumerate() {
                        let j = dodai_charts_core::pulse::hash((k + 977 * i) as f32);
                        let dx = (j - 0.5) * half * 1.6;
                        fill(
                            w,
                            b.origin,
                            &crate::common::marker_points(
                                crate::common::Marker::Dot,
                                x + dx,
                                y(*v),
                                2.0,
                            ),
                            scale_alpha(cols[i], 0.55),
                        );
                    }
                }
                let outline = if notch {
                    notched_box(x, half, (*q1, *md, *q3), raw[i].len(), &y)
                } else {
                    let (t, bt) = (y(*q3), y(*q1));
                    vec![(x - half, t), (x + half, t), (x + half, bt), (x - half, bt)]
                };
                fill(w, b.origin, &outline, scale_alpha(cols[i], 0.35));
                stroke(w, b.origin, &outline, 1.5, cols[i], true);
                stroke(
                    w,
                    b.origin,
                    &[(x - half, y(*md)), (x + half, y(*md))],
                    2.5,
                    text,
                    false,
                );
                if show_mean && !raw[i].is_empty() {
                    let m = raw[i].iter().sum::<f32>() / raw[i].len() as f32;
                    let (cx, cy) = (x, y(m));
                    let d = 4.5;
                    let dia = [(cx, cy - d), (cx + d, cy), (cx, cy + d), (cx - d, cy)];
                    fill(w, b.origin, &dia, text);
                }
            }
        })
        .absolute()
        .size_full();
        let hits = div()
            .id(self.follow.sub("boxes").element_id(window, cx))
            .absolute()
            .size_full()
            .child(cv);
        let plot = div()
            .relative()
            .w_full()
            .h(px(self.height))
            .child(focus.pointer(hits, &bounds, columns(n)));
        let names: Vec<SharedString> = self.groups.iter().map(|g| g.0.clone()).collect();
        let root = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(framed(
                &t,
                ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
                names,
                plot,
            ))
            .child(readout(&t, readout_text, "Hover a box"));
        titled(&t, self.title, root)
    }
}

dodai_gpui::entry_render!(BoxPlot);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_counts_all_values() {
        let b = histogram_bins(&[1.0, 2.0, 2.0, 3.0, 10.0], 3);
        assert_eq!(b.iter().map(|x| x.2).sum::<usize>(), 5);
        assert_eq!(b.len(), 3);
        assert_eq!(b[2].2, 1);
        assert!(histogram_bins(&[], 4).is_empty());
    }

    #[test]
    fn quartiles_basic() {
        assert_eq!(
            quartiles(&[1.0, 2.0, 3.0, 4.0, 5.0]),
            Some((1.0, 2.0, 3.0, 4.0, 5.0))
        );
        assert_eq!(quartiles(&[]), None);
        assert_eq!(quartiles(&[7.0]), Some((7.0, 7.0, 7.0, 7.0, 7.0)));
    }
}

#[cfg(test)]
mod notch_tests {
    use super::*;

    #[test]
    fn notch_stays_inside_the_box_and_pinches_at_the_median() {
        let y = |v: f32| 100.0 - v;
        let p = notched_box(50.0, 10.0, (20.0, 30.0, 40.0), 9, &y);
        assert_eq!(p.len(), 10);
        let (top, bottom) = (y(40.0), y(20.0));
        assert!(p.iter().all(|q| q.1 >= top && q.1 <= bottom));
        assert!(p.iter().any(|q| q.0 == 56.0 && q.1 == y(30.0)));
    }
}
