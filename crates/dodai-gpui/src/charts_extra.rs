//! More chart types: [`StackedAreaChart`], [`Histogram`], [`Waterfall`],
//! [`Funnel`], [`Candlestick`], [`BoxPlot`], [`RadialBar`], [`BubbleChart`]
//! and [`Treemap`]. Same theming rules as [`crate::charts`].

use crate::charts::{
    fill, fmt_with, framed, glow_of, glow_stroke, label_style, nice_ticks, norm, series_color,
    stroke, titled, turn_arc, Legend, NumFmt, Series,
};
use crate::paint::{scale_alpha, shaped, ShapeStyle};
use crate::panel::DodaiTooltip;
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_core::{Color, Theme, Tone};
use gpui::{
    canvas, div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ───────────────────────────── pure helpers ─────────────────────────────

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

/// Running `(start, end)` for each delta of a waterfall. Pure.
pub fn waterfall_steps(deltas: &[f32]) -> Vec<(f32, f32)> {
    let mut acc = 0.0;
    deltas
        .iter()
        .map(|d| {
            let s = acc;
            acc += d;
            (s, acc)
        })
        .collect()
}

/// Cumulative stack boundaries: `out[k][i]` = sum of series `0..=k` at `i`
/// (negatives clamp to 0). Pure.
pub fn stack_cumulative(series: &[Series]) -> Vec<Vec<f32>> {
    let n = series.iter().map(|s| s.values.len()).max().unwrap_or(0);
    let mut acc = vec![0.0f32; n];
    series
        .iter()
        .map(|s| {
            for (i, a) in acc.iter_mut().enumerate() {
                *a += s.values.get(i).copied().unwrap_or(0.0).max(0.0);
            }
            acc.clone()
        })
        .collect()
}

/// Treemap rectangles `(x, y, w, h)` for `values` inside `w × h`, in input
/// order; each area is proportional to its value. Pure.
pub fn treemap_layout(values: &[f32], w: f32, h: f32) -> Vec<(f32, f32, f32, f32)> {
    let mut items: Vec<(usize, f32)> = values
        .iter()
        .enumerate()
        .map(|(i, v)| (i, v.max(0.0)))
        .collect();
    items.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut out = vec![(0.0, 0.0, 0.0, 0.0); values.len()];
    split(&items, 0.0, 0.0, w, h, &mut out);
    out
}

fn split(items: &[(usize, f32)], x: f32, y: f32, w: f32, h: f32, out: &mut [(f32, f32, f32, f32)]) {
    match items {
        [] => {}
        [(i, _)] => out[*i] = (x, y, w, h),
        _ => {
            let total: f32 = items.iter().map(|i| i.1).sum();
            if total <= 0.0 {
                for (i, _) in items {
                    out[*i] = (x, y, 0.0, 0.0);
                }
                return;
            }
            let (mut acc, mut k) = (0.0, 0);
            while k < items.len() - 1 && acc + items[k].1 / 2.0 < total / 2.0 {
                acc += items[k].1;
                k += 1;
            }
            let k = k.clamp(1, items.len() - 1);
            let first: f32 = items[..k].iter().map(|i| i.1).sum();
            let f = first / total;
            if w >= h {
                split(&items[..k], x, y, w * f, h, out);
                split(&items[k..], x + w * f, y, w * (1.0 - f), h, out);
            } else {
                split(&items[..k], x, y, w, h * f, out);
                split(&items[k..], x, y + h * f, w, h * (1.0 - f), out);
            }
        }
    }
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> [(f32, f32); 4] {
    [(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
}

type Stats = (f32, f32, f32, f32, f32);
type Pts3 = Vec<(f32, f32, f32)>;

fn up_color(t: &Theme) -> Hsla {
    hsla(t.palette.tone(Tone::Success))
}
fn down_color(t: &Theme) -> Hsla {
    hsla(t.palette.tone(Tone::Danger))
}

macro_rules! cfg {
    ($ty:ident) => {
        impl $ty {
            /// Chart height in px.
            pub fn height(mut self, h: f32) -> Self {
                self.height = h;
                self
            }
            /// Title above the chart.
            pub fn title(mut self, t: impl Into<SharedString>) -> Self {
                self.title = Some(t.into());
                self
            }
            /// Custom number formatter for labels and tooltips.
            pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
                self.fmt = Some(Rc::new(f));
                self
            }
        }
    };
}

fn labels_of<S: Into<SharedString>>(l: impl IntoIterator<Item = S>) -> Vec<SharedString> {
    l.into_iter().map(Into::into).collect()
}

// ─────────────────────────────── StackedAreaChart ───────────────────────────────

/// Stacked area chart: layers add up to the total.
#[derive(IntoElement)]
pub struct StackedAreaChart {
    series: Vec<Series>,
    labels: Vec<SharedString>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
    legend: bool,
}
cfg!(StackedAreaChart);

impl StackedAreaChart {
    pub fn new(series: impl IntoIterator<Item = Series>) -> Self {
        Self {
            series: series.into_iter().collect(),
            labels: Vec::new(),
            height: 200.0,
            fmt: None,
            title: None,
            legend: true,
        }
    }
    pub fn labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels_of(l);
        self
    }
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

impl RenderOnce for StackedAreaChart {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let cum = stack_cumulative(&self.series);
        let n = cum.first().map_or(0, Vec::len);
        let top_v = cum
            .last()
            .map_or(1.0, |l| l.iter().copied().fold(0.0, f32::max));
        let ticks = nice_ticks(0.0, top_v, 4);
        let top = *ticks.last().unwrap_or(&1.0);
        let grid = hsla(t.palette.border);
        let glow = glow_of(&t);
        let cols: Vec<Hsla> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| hsla(series_color(&t, s, i)))
            .collect();
        let tf: Vec<f32> = ticks.iter().map(|v| norm(*v, 0.0, top)).collect();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                for f in &tf {
                    let y = bh - f * bh;
                    stroke(w, b.origin, &[(0.0, y), (bw, y)], 1.0, grid, false);
                }
                let x = |i: usize| (i as f32 + 0.5) / n.max(1) as f32 * bw;
                for (k, layer) in cum.iter().enumerate() {
                    let upper: Vec<(f32, f32)> = (0..n)
                        .map(|i| (x(i), bh - norm(layer[i], 0.0, top) * bh))
                        .collect();
                    let lower: Vec<(f32, f32)> = (0..n)
                        .rev()
                        .map(|i| {
                            let v = if k == 0 { 0.0 } else { cum[k - 1][i] };
                            (x(i), bh - norm(v, 0.0, top) * bh)
                        })
                        .collect();
                    if upper.len() < 2 {
                        continue;
                    }
                    // Quads per segment stay convex.
                    for i in 0..n - 1 {
                        let quad = [upper[i], upper[i + 1], lower[n - 2 - i], lower[n - 1 - i]];
                        fill(w, b.origin, &quad, scale_alpha(cols[k], 0.45));
                    }
                    glow_stroke(w, b.origin, &upper, 2.0, cols[k], glow);
                }
            },
        )
        .absolute()
        .size_full();
        let plot = div().relative().w_full().h(px(self.height)).child(cv);
        let x_labels = if self.labels.len() == n {
            self.labels.clone()
        } else {
            Vec::new()
        };
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(framed(
                &t,
                ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
                x_labels,
                plot,
            ));
        if self.legend && self.series.len() > 1 {
            root = root.child(Legend::of_series(&t, &self.series));
        }
        titled(&t, self.title, root)
    }
}

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
}
cfg!(Histogram);

impl Histogram {
    pub fn new(id: impl Into<SharedString>, values: impl IntoIterator<Item = f32>) -> Self {
        Self {
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

impl RenderOnce for Histogram {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
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
                            .h(relative(norm(*c as f32, 0.0, top)))
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

// ─────────────────────────────── Waterfall ───────────────────────────────

/// Waterfall of deltas; increases green, decreases red, optional total bar.
#[derive(IntoElement)]
pub struct Waterfall {
    id: SharedString,
    steps: Vec<(SharedString, f32)>,
    total: bool,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
cfg!(Waterfall);

impl Waterfall {
    pub fn new<S: Into<SharedString>>(
        id: impl Into<SharedString>,
        steps: impl IntoIterator<Item = (S, f32)>,
    ) -> Self {
        Self {
            id: id.into(),
            steps: steps.into_iter().map(|(a, b)| (a.into(), b)).collect(),
            total: true,
            height: 200.0,
            fmt: None,
            title: None,
        }
    }
    /// Append a "Total" bar (default on).
    pub fn total(mut self, on: bool) -> Self {
        self.total = on;
        self
    }
}

impl RenderOnce for Waterfall {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let deltas: Vec<f32> = self.steps.iter().map(|s| s.1).collect();
        let st = waterfall_steps(&deltas);
        let end = st.last().map_or(0.0, |s| s.1);
        let lo = st.iter().flat_map(|s| [s.0, s.1]).fold(0.0f32, f32::min);
        let hi = st.iter().flat_map(|s| [s.0, s.1]).fold(0.0f32, f32::max);
        let ticks = nice_ticks(lo, hi, 4);
        let (lo, hi) = (*ticks.first().unwrap_or(&lo), *ticks.last().unwrap_or(&hi));
        let grid = hsla(t.palette.border);
        let mut plot = div().relative().w_full().h(px(self.height));
        for v in &ticks {
            plot = plot.child(
                div()
                    .absolute()
                    .left_0()
                    .w_full()
                    .h(px(1.0))
                    .bottom(relative(norm(*v, lo, hi)))
                    .bg(grid),
            );
        }
        let mut bars: Vec<(SharedString, f32, f32, Color, f32)> = self
            .steps
            .iter()
            .zip(&st)
            .map(|((n, d), (a, b))| {
                let c = if *d >= 0.0 {
                    t.palette.tone(Tone::Success)
                } else {
                    t.palette.tone(Tone::Danger)
                };
                (n.clone(), a.min(*b), a.max(*b), c, *d)
            })
            .collect();
        if self.total {
            bars.push((
                "Total".into(),
                end.min(0.0),
                end.max(0.0),
                t.palette.primary,
                end,
            ));
        }
        let mut row = div().flex().size_full();
        let names: Vec<SharedString> = bars.iter().map(|b| b.0.clone()).collect();
        for (i, (n, a, b, c, d)) in bars.into_iter().enumerate() {
            let style = ShapeStyle::from_tokens(&t, Some(c), None, false, false);
            let (fa, fb) = (norm(a, lo, hi), norm(b, lo, hi));
            row = row.child(
                div().relative().flex_1().h_full().child(
                    div()
                        .id((self.id.clone(), i))
                        .absolute()
                        .left(relative(0.15))
                        .w(relative(0.7))
                        .bottom(relative(fa))
                        .h(relative((fb - fa).max(0.004)))
                        .tooltip(DodaiTooltip::text(format!(
                            "{n}: {}{}",
                            if d > 0.0 && i < self.steps.len() {
                                "+"
                            } else {
                                ""
                            },
                            fmt_with(&self.fmt, d)
                        )))
                        .child(shaped(&t.shape, style).size_full()),
                ),
            );
        }
        plot = plot.child(row);
        let root = div().flex().flex_col().w_full().child(framed(
            &t,
            ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
            names,
            plot,
        ));
        titled(&t, self.title, root)
    }
}

// ─────────────────────────────── Funnel ───────────────────────────────

/// Funnel of decreasing stages, drawn as joined trapezoids.
#[derive(IntoElement)]
pub struct Funnel {
    stages: Vec<(SharedString, f32)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
cfg!(Funnel);

impl Funnel {
    pub fn new<S: Into<SharedString>>(stages: impl IntoIterator<Item = (S, f32)>) -> Self {
        Self {
            stages: stages.into_iter().map(|(a, b)| (a.into(), b)).collect(),
            height: 220.0,
            fmt: None,
            title: None,
        }
    }
}

impl RenderOnce for Funnel {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let n = self.stages.len().max(1);
        let maxv = self
            .stages
            .iter()
            .map(|s| s.1)
            .fold(f32::MIN_POSITIVE, f32::max);
        let fr: Vec<f32> = self
            .stages
            .iter()
            .map(|s| (s.1 / maxv).clamp(0.0, 1.0))
            .collect();
        let cols: Vec<Hsla> = (0..n).map(|i| hsla(t.palette.series_at(i))).collect();
        let fr2 = fr.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let sh = bh / n as f32;
                for i in 0..n {
                    let next = fr2.get(i + 1).copied().unwrap_or(fr2[i] * 0.8);
                    let (w0, w1) = (fr2[i] * bw, next * bw);
                    let (y0, y1) = (i as f32 * sh + 1.0, (i + 1) as f32 * sh - 1.0);
                    let c = bw / 2.0;
                    fill(
                        w,
                        b.origin,
                        &[
                            (c - w0 / 2.0, y0),
                            (c + w0 / 2.0, y0),
                            (c + w1 / 2.0, y1),
                            (c - w1 / 2.0, y1),
                        ],
                        cols[i],
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let mut labels = div().absolute().size_full().flex().flex_col();
        for (i, (name, v)) in self.stages.iter().enumerate() {
            let pct = fr[i] * 100.0;
            labels = labels.child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_family(first_family(t.typography.mono))
                    .text_size(px(t.typography.size_xs))
                    .text_color(hsla(t.palette.on_primary))
                    .child(format!("{name} · {} ({pct:.0}%)", fmt_with(&self.fmt, *v))),
            );
        }
        let root = div()
            .relative()
            .w_full()
            .h(px(self.height))
            .child(cv)
            .child(labels);
        titled(&t, self.title, div().w_full().child(root))
    }
}

// ─────────────────────────────── Candlestick ───────────────────────────────

/// One OHLC candle.
#[derive(Clone, Copy, Debug)]
pub struct Candle {
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
}

impl Candle {
    pub fn new(open: f32, high: f32, low: f32, close: f32) -> Self {
        Self {
            open,
            high,
            low,
            close,
        }
    }
}

/// Candlestick (OHLC) chart; up candles green, down red.
#[derive(IntoElement)]
pub struct Candlestick {
    id: SharedString,
    candles: Vec<Candle>,
    labels: Vec<SharedString>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
cfg!(Candlestick);

impl Candlestick {
    pub fn new(id: impl Into<SharedString>, candles: impl IntoIterator<Item = Candle>) -> Self {
        Self {
            id: id.into(),
            candles: candles.into_iter().collect(),
            labels: Vec::new(),
            height: 220.0,
            fmt: None,
            title: None,
        }
    }
    pub fn labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels_of(l);
        self
    }
}

impl RenderOnce for Candlestick {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let lo = self.candles.iter().map(|c| c.low).fold(f32::MAX, f32::min);
        let hi = self.candles.iter().map(|c| c.high).fold(f32::MIN, f32::max);
        let (lo, hi) = if lo <= hi { (lo, hi) } else { (0.0, 1.0) };
        let ticks = nice_ticks(lo, hi, 4);
        let (lo, hi) = (*ticks.first().unwrap_or(&lo), *ticks.last().unwrap_or(&hi));
        let (up, down, grid) = (up_color(&t), down_color(&t), hsla(t.palette.border));
        let n = self.candles.len();
        let cs = self.candles.clone();
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
                let y = |v: f32| bh - norm(v, lo, hi) * bh;
                for (i, c) in cs.iter().enumerate() {
                    let col = if c.close >= c.open { up } else { down };
                    let x = (i as f32 + 0.5) * slot;
                    stroke(
                        w,
                        b.origin,
                        &[(x, y(c.high)), (x, y(c.low))],
                        1.5,
                        col,
                        false,
                    );
                    let (top, bot) = (y(c.open.max(c.close)), y(c.open.min(c.close)));
                    let bwid = (slot * 0.6).min(24.0);
                    fill(
                        w,
                        b.origin,
                        &rect(x - bwid / 2.0, top, bwid, (bot - top).max(1.5)),
                        col,
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let mut plot = div().relative().w_full().h(px(self.height)).child(cv);
        let mut hit = div().absolute().size_full().flex();
        for (i, c) in self.candles.iter().enumerate() {
            let label = self.labels.get(i).cloned().unwrap_or_default();
            hit = hit.child(div().id((self.id.clone(), i)).flex_1().h_full().tooltip(
                DodaiTooltip::text(format!(
                    "{label} O {} H {} L {} C {}",
                    fmt_with(&self.fmt, c.open),
                    fmt_with(&self.fmt, c.high),
                    fmt_with(&self.fmt, c.low),
                    fmt_with(&self.fmt, c.close)
                )),
            ));
        }
        plot = plot.child(hit);
        let x_labels = if self.labels.len() == n {
            self.labels.clone()
        } else {
            Vec::new()
        };
        let root = div().flex().flex_col().w_full().child(framed(
            &t,
            ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
            x_labels,
            plot,
        ));
        titled(&t, self.title, root)
    }
}

// ─────────────────────────────── BoxPlot ───────────────────────────────

/// Box-and-whisker plot from raw samples per category.
#[derive(IntoElement)]
pub struct BoxPlot {
    groups: Vec<(SharedString, Vec<f32>)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
cfg!(BoxPlot);

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

impl RenderOnce for BoxPlot {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
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
                let y = |v: f32| bh - norm(v, lo, hi) * bh;
                for (i, s) in stats.iter().enumerate() {
                    let Some((mn, q1, md, q3, mx)) = s else {
                        continue;
                    };
                    let x = (i as f32 + 0.5) * slot;
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

// ─────────────────────────────── RadialBar ───────────────────────────────

/// Concentric progress rings, one per `(name, value, max)`.
#[derive(IntoElement)]
pub struct RadialBar {
    rings: Vec<(SharedString, f32, f32)>,
    size: f32,
    legend: bool,
}

impl RadialBar {
    pub fn new<S: Into<SharedString>>(rings: impl IntoIterator<Item = (S, f32, f32)>) -> Self {
        Self {
            rings: rings
                .into_iter()
                .map(|(a, b, c)| (a.into(), b, c))
                .collect(),
            size: 160.0,
            legend: true,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

impl RenderOnce for RadialBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let n = self.rings.len().max(1);
        let size = self.size;
        let fr: Vec<f32> = self.rings.iter().map(|r| norm(r.1, 0.0, r.2)).collect();
        let cols: Vec<Hsla> = (0..n).map(|i| hsla(t.palette.series_at(i))).collect();
        let track = hsla(t.palette.border);
        let glow = glow_of(&t);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let c = size / 2.0;
                let ring = (size / 2.0 / (n as f32 + 1.0)).min(14.0);
                for i in 0..n {
                    let r = c - ring * (i as f32 + 0.5) - 2.0 * i as f32 - 1.0;
                    if r <= 1.0 {
                        break;
                    }
                    stroke(
                        w,
                        b.origin,
                        &turn_arc(c, c, r, 0.0, 0.999),
                        ring - 2.0,
                        scale_alpha(track, 0.6),
                        false,
                    );
                    if fr[i] > 0.001 {
                        glow_stroke(
                            w,
                            b.origin,
                            &turn_arc(c, c, r, 0.0, fr[i].min(0.999)),
                            ring - 2.0,
                            cols[i],
                            glow,
                        );
                    }
                }
            },
        )
        .absolute()
        .size_full();
        let mut root = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.lg))
            .child(div().relative().w(px(size)).h(px(size)).child(cv));
        if self.legend {
            root = root.child(div().flex().flex_col().gap(px(t.spacing.xs)).children(
                self.rings.iter().enumerate().map(|(i, (n, v, m))| {
                    label_style(&t, div())
                        .flex()
                        .items_center()
                        .gap(px(t.spacing.xs))
                        .child(div().w(px(10.0)).h(px(10.0)).bg(cols_at(&t, i)))
                        .child(format!("{n} · {:.0}%", norm(*v, 0.0, *m) * 100.0))
                }),
            ));
        }
        root
    }
}

fn cols_at(t: &Theme, i: usize) -> Hsla {
    hsla(t.palette.series_at(i))
}

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
cfg!(BubbleChart);

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

impl RenderOnce for BubbleChart {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
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
                let r = (s / smax).sqrt() * self.max_radius;
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

// ─────────────────────────────── Treemap ───────────────────────────────

/// Treemap: rectangle area ∝ value.
#[derive(IntoElement)]
pub struct Treemap {
    id: SharedString,
    items: Vec<(SharedString, f32)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
cfg!(Treemap);

impl Treemap {
    pub fn new<S: Into<SharedString>>(
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = (S, f32)>,
    ) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().map(|(a, b)| (a.into(), b)).collect(),
            height: 220.0,
            fmt: None,
            title: None,
        }
    }
}

impl RenderOnce for Treemap {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let vals: Vec<f32> = self.items.iter().map(|i| i.1).collect();
        // Layout in unit space, rendered with relative sizes.
        let rects = treemap_layout(&vals, 1.0, 1.0);
        let mut plot = div().relative().w_full().h(px(self.height));
        for (i, ((x, y, w, h), (name, v))) in rects.iter().zip(&self.items).enumerate() {
            let style =
                ShapeStyle::from_tokens(&t, Some(t.palette.series_at(i)), None, false, false);
            plot = plot.child(
                div()
                    .id((self.id.clone(), i))
                    .absolute()
                    .left(relative(*x))
                    .top(relative(*y))
                    .w(relative(*w))
                    .h(relative(*h))
                    .p(px(1.5))
                    .tooltip(DodaiTooltip::text(format!(
                        "{name}: {}",
                        fmt_with(&self.fmt, *v)
                    )))
                    .child(
                        shaped(&t.shape, style)
                            .size_full()
                            .overflow_hidden()
                            .p(px(t.spacing.xs))
                            .font_family(first_family(t.typography.mono))
                            .text_size(px(t.typography.size_xs))
                            .text_color(hsla(t.palette.on_primary))
                            .child(format!("{name}\n{}", fmt_with(&self.fmt, *v))),
                    ),
            );
        }
        titled(&t, self.title, div().w_full().child(plot))
    }
}

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

    #[test]
    fn waterfall_accumulates() {
        assert_eq!(
            waterfall_steps(&[5.0, -2.0, 4.0]),
            vec![(0.0, 5.0), (5.0, 3.0), (3.0, 7.0)]
        );
    }

    #[test]
    fn stack_is_cumulative() {
        let s = [Series::new("a", [1.0, 2.0]), Series::new("b", [3.0, -1.0])];
        assert_eq!(stack_cumulative(&s), vec![vec![1.0, 2.0], vec![4.0, 2.0]]);
    }

    #[test]
    fn treemap_areas_proportional_and_tile() {
        let v = [6.0, 3.0, 2.0, 1.0];
        let r = treemap_layout(&v, 12.0, 6.0);
        let total: f32 = r.iter().map(|(_, _, w, h)| w * h).sum();
        assert!((total - 72.0).abs() < 1e-3);
        for ((_, _, w, h), val) in r.iter().zip(v) {
            assert!((w * h - val / 12.0 * 72.0).abs() < 1e-3);
        }
    }

    #[test]
    fn step_points_stairs() {
        use crate::charts::step_points;
        assert_eq!(
            step_points(&[(0.0, 1.0), (2.0, 3.0)]),
            vec![(0.0, 1.0), (2.0, 1.0), (2.0, 3.0)]
        );
    }
}
