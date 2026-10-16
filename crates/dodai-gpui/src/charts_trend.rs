//! Trend charts over categories/time: stacked area and candlestick.

use crate::charts::{
    chart_cfg, down_color, fill, fmt_with, framed, glow_of, glow_stroke, labels_of, nice_ticks,
    norm, rect, series_color, stroke, titled, up_color, Legend, NumFmt, Series,
};
use crate::paint::scale_alpha;
use crate::panel::DodaiTooltip;
use crate::{hsla, theme_of};
use gpui::{
    canvas, div, px, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

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
chart_cfg!(StackedAreaChart);

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
chart_cfg!(Candlestick);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_is_cumulative() {
        let s = [Series::new("a", [1.0, 2.0]), Series::new("b", [3.0, -1.0])];
        assert_eq!(stack_cumulative(&s), vec![vec![1.0, 2.0], vec![4.0, 2.0]]);
    }
}
