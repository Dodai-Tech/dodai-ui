//! Histogram, box plot and bubble charts (distribution of values).

use crate::charts::{
    chart_cfg, fill, fmt_with, framed, nice_ticks, norm, rect, stroke, titled, Legend, NumFmt,
};
use crate::paint::{scale_alpha, shaped, ShapeStyle};
use crate::panel::DodaiTooltip;
use crate::{hsla, theme_of};
use dodai_core::Color;
use gpui::{
    canvas, div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, SharedString, StatefulInteractiveElement, Styled, Window,
};
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
type Pts3 = Vec<(f32, f32, f32)>;

// ─────────────────────────────── Histogram ───────────────────────────────

/// Histogram of raw values with equal-width bins.
#[derive(IntoElement)]
pub struct Histogram {
    id: SharedString,
    values: Vec<f32>,
    bins: usize,
    height: f32,
    color: Option<Color>,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Histogram);
chart_cfg!(Histogram);

impl Histogram {
    pub fn new(id: impl Into<SharedString>, values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            values: values.into_iter().collect(),
            bins: 10,
            height: 180.0,
            color: None,
            fmt: None,
            title: None,
        }
    }
    pub fn bins(mut self, n: usize) -> Self {
        self.bins = n.max(1);
        self
    }
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

impl Histogram {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let g = crate::motion::grow(window, cx, "Histogram");
        let bins = histogram_bins(&self.values, self.bins);
        let maxc = bins.iter().map(|b| b.2).max().unwrap_or(1) as f32;
        let ticks = nice_ticks(0.0, maxc, 4);
        let top = *ticks.last().unwrap_or(&1.0);
        let col = self.color.unwrap_or(t.palette.series_at(0));
        let grid = hsla(t.palette.border);
        let mut plot = div().relative().w_full().h(px(self.height));
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
                            .h(relative(norm(*c as f32, 0.0, top) * g))
                            .child(shaped(&t.shape, style).size_full()),
                    ),
            );
        }
        plot = plot.child(row);
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

crate::entry_render!(Histogram);

// ─────────────────────────────── BoxPlot ───────────────────────────────

/// Box-and-whisker plot from raw samples per category.
#[derive(IntoElement)]
pub struct BoxPlot {
    groups: Vec<(SharedString, Vec<f32>)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(BoxPlot);

impl BoxPlot {
    pub fn new<S: Into<SharedString>>(groups: impl IntoIterator<Item = (S, Vec<f32>)>) -> Self {
        Self {
            groups: groups.into_iter().map(|(a, b)| (a.into(), b)).collect(),
            height: 220.0,
            fmt: None,
            title: None,
        }
    }
}

impl BoxPlot {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let g = crate::motion::grow(window, cx, "BoxPlot");
        let t = theme_of(cx);
        let stats: Vec<Option<Stats>> = self.groups.iter().map(|g| quartiles(&g.1)).collect();
        let lo = stats.iter().flatten().map(|s| s.0).fold(f32::MAX, f32::min);
        let hi = stats.iter().flatten().map(|s| s.4).fold(f32::MIN, f32::max);
        let (lo, hi) = if lo <= hi { (lo, hi) } else { (0.0, 1.0) };
        let ticks = nice_ticks(lo, hi, 4);
        let (lo, hi) = (*ticks.first().unwrap_or(&lo), *ticks.last().unwrap_or(&hi));
        let grid = hsla(t.palette.border);
        let text = hsla(t.palette.text);
        let cols: Vec<Hsla> = (0..stats.len())
            .map(|i| hsla(t.palette.series_at(i)))
            .collect();
        let n = stats.len();
        let tf: Vec<f32> = ticks.iter().map(|v| norm(*v, lo, hi)).collect();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
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
                    let gi = crate::motion::reveal(g, i, n);
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
                    fill(
                        w,
                        b.origin,
                        &rect(x - half, y(*q3), half * 2.0, (y(*q1) - y(*q3)).max(1.0)),
                        scale_alpha(cols[i], 0.35),
                    );
                    stroke(
                        w,
                        b.origin,
                        &rect(x - half, y(*q3), half * 2.0, (y(*q1) - y(*q3)).max(1.0)),
                        1.5,
                        cols[i],
                        true,
                    );
                    stroke(
                        w,
                        b.origin,
                        &[(x - half, y(*md)), (x + half, y(*md))],
                        2.5,
                        text,
                        false,
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let plot = div().relative().w_full().h(px(self.height)).child(cv);
        let names: Vec<SharedString> = self.groups.iter().map(|g| g.0.clone()).collect();
        let root = div().flex().flex_col().w_full().child(framed(
            &t,
            ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
            names,
            plot,
        ));
        titled(&t, self.title, root)
    }
}

crate::entry_render!(BoxPlot);

// ─────────────────────────────── BubbleChart ───────────────────────────────

/// Scatter plot where the third value controls the bubble area.
#[derive(IntoElement)]
pub struct BubbleChart {
    id: SharedString,
    groups: Vec<(SharedString, Pts3)>,
    height: f32,
    max_radius: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(BubbleChart);

impl BubbleChart {
    pub fn new<S: Into<SharedString>>(
        id: impl Into<SharedString>,
        groups: impl IntoIterator<Item = (S, Vec<(f32, f32, f32)>)>,
    ) -> Self {
        Self {
            id: id.into(),
            groups: groups.into_iter().map(|(a, b)| (a.into(), b)).collect(),
            height: 240.0,
            max_radius: 22.0,
            fmt: None,
            title: None,
        }
    }
    /// Radius in px of the largest bubble (default 22).
    pub fn max_radius(mut self, r: f32) -> Self {
        self.max_radius = r;
        self
    }
}

impl BubbleChart {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let g = crate::motion::grow(window, cx, "BubbleChart");
        let t = theme_of(cx);
        let all = || self.groups.iter().flat_map(|g| g.1.iter());
        let rng = |f: fn(&(f32, f32, f32)) -> f32| {
            let lo = all().map(f).fold(f32::MAX, f32::min);
            let hi = all().map(f).fold(f32::MIN, f32::max);
            let tk = nice_ticks(
                if lo <= hi { lo } else { 0.0 },
                if lo <= hi { hi } else { 1.0 },
                4,
            );
            (*tk.first().unwrap(), *tk.last().unwrap(), tk)
        };
        let (xl, xh, xt) = rng(|p| p.0);
        let (yl, yh, yt) = rng(|p| p.1);
        let smax = all().map(|p| p.2).fold(f32::MIN_POSITIVE, f32::max);
        let grid = hsla(t.palette.border);
        let mut plot = div().relative().w_full().h(px(self.height));
        for v in &yt {
            plot = plot.child(
                div()
                    .absolute()
                    .left_0()
                    .w_full()
                    .h(px(1.0))
                    .bottom(relative(norm(*v, yl, yh)))
                    .bg(grid),
            );
        }
        for v in &xt {
            plot = plot.child(
                div()
                    .absolute()
                    .top_0()
                    .h_full()
                    .w(px(1.0))
                    .left(relative(norm(*v, xl, xh)))
                    .bg(grid),
            );
        }
        for (gi, (name, pts)) in self.groups.iter().enumerate() {
            let c = hsla(t.palette.series_at(gi));
            for (i, (x, y, s)) in pts.iter().enumerate() {
                let r =
                    (s / smax).sqrt() * self.max_radius * crate::motion::reveal(g, i, pts.len());
                plot = plot.child(
                    div()
                        .id((SharedString::from(format!("{}-{gi}", self.id)), i))
                        .absolute()
                        .left(relative(norm(*x, xl, xh)))
                        .top(relative(1.0 - norm(*y, yl, yh)))
                        .ml(px(-r))
                        .mt(px(-r))
                        .w(px(r * 2.0))
                        .h(px(r * 2.0))
                        .rounded_full()
                        .bg(scale_alpha(c, 0.45))
                        .border_1()
                        .border_color(c)
                        .tooltip(DodaiTooltip::text(format!(
                            "{name}: x {} · y {} · size {}",
                            fmt_with(&self.fmt, *x),
                            fmt_with(&self.fmt, *y),
                            fmt_with(&self.fmt, *s)
                        ))),
                );
            }
        }
        let xl_labels: Vec<SharedString> =
            xt.iter().map(|v| fmt_with(&self.fmt, *v).into()).collect();
        let framed_plot = div().flex().flex_col().w_full().child(framed(
            &t,
            yt.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
            xl_labels,
            plot,
        ));
        let legend = Legend::new(
            self.groups
                .iter()
                .enumerate()
                .map(|(i, g)| (g.0.clone(), t.palette.series_at(i))),
        );
        titled(
            &t,
            self.title,
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .w_full()
                .child(framed_plot)
                .child(legend),
        )
    }
}

crate::entry_render!(BubbleChart);

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
