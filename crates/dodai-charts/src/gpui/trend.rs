//! Trend charts over categories/time: stacked area and candlestick.

use crate::gpui::common::{
    chart_cfg, down_color, fill, fmt_num, fmt_with, framed, glow_of, glow_stroke, labels_of,
    nice_ticks, norm, rect, series_color, stroke, titled, up_color, Legend, NumFmt, Series,
};
use dodai_gpui::paint::scale_alpha;
use dodai_gpui::panel::DodaiTooltip;
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, px, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

pub use crate::layers::StackOffset;

// ─────────────────────────────── StackedAreaChart ───────────────────────────────

/// Stacked area chart: layers add up to the total.
#[derive(IntoElement)]
pub struct StackedAreaChart {
    follow: dodai_gpui::follow::Follow,
    glow: crate::gpui::common::ChartGlow,
    annotations: Vec<crate::gpui::annotate::Annotation>,
    series: Vec<Series>,
    labels: Vec<SharedString>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
    legend: bool,
    offset: StackOffset,
}
chart_cfg!(StackedAreaChart);

impl StackedAreaChart {
    dodai_gpui::follow_api!();
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: crate::gpui::common::ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// Threshold lines and highlighted bands drawn over the plot.
    pub fn annotations(
        mut self,
        a: impl IntoIterator<Item = crate::gpui::annotate::Annotation>,
    ) -> Self {
        self.annotations = a.into_iter().collect();
        self
    }
    #[track_caller]
    /// Create a new [`StackedAreaChart`].
    pub fn new(series: impl IntoIterator<Item = Series>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            glow: crate::gpui::common::ChartGlow::Theme,
            annotations: Vec::new(),
            series: series.into_iter().collect(),
            labels: Vec::new(),
            height: 200.0,
            fmt: None,
            title: None,
            legend: true,
            offset: StackOffset::Zero,
        }
    }
    /// Where the stack sits: on zero (default), stretched to 100%, or floating as a stream
    /// graph ([`StackOffset::Silhouette`], [`StackOffset::Wiggle`]; no value axis then).
    pub fn offset(mut self, o: StackOffset) -> Self {
        self.offset = o;
        self
    }
    /// Labels, one per item.
    pub fn labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels_of(l);
        self
    }
    /// Show or hide the legend.
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

impl StackedAreaChart {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "StackedAreaChart");
        self.series = crate::gpui::common::follow_series(window, cx, &self.follow, &self.series);
        let t = theme_of(cx);
        let rows: Vec<Vec<f32>> = self.series.iter().map(|s| s.values.clone()).collect();
        let layers = crate::layers::stack_layers(&rows, self.offset);
        let n = layers.first().map_or(0, Vec::len);
        let (lo_v, hi_v) = crate::layers::layers_extent(&layers);
        // A stream floats: no ticks, a little air above and below.
        let (bottom, ticks) = match self.offset {
            StackOffset::Expand => (0.0, vec![0.0, 0.25, 0.5, 0.75, 1.0]),
            o if o.floats() => {
                let pad = (hi_v - lo_v).max(f32::MIN_POSITIVE) * 0.05;
                (lo_v - pad, vec![lo_v - pad, hi_v + pad])
            }
            _ => (0.0, nice_ticks(0.0, hi_v, 4)),
        };
        let top = *ticks.last().unwrap_or(&1.0);
        let floats = self.offset.floats();
        let expand = self.offset == StackOffset::Expand;
        let grid = hsla(t.palette.border);
        let glow = self.glow.resolve(&t);
        let cols: Vec<Hsla> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| hsla(series_color(&t, s, i)))
            .collect();
        let tf: Vec<f32> = if floats {
            Vec::new()
        } else {
            ticks.iter().map(|v| norm(*v, bottom, top)).collect()
        };
        let (patterns, hatch_ink) = (
            crate::gpui::common::patterns(),
            crate::gpui::common::hatch_ink(&t),
        );
        let focus = crate::gpui::focus::Focus::new(window, cx, &self.follow, layers.len());
        let looks: Vec<_> = (0..layers.len()).map(|k| focus.item(k)).collect();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let hit_layers = layers.clone();
        let cv = crate::gpui::common::tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            for f in &tf {
                let y = bh - f * bh;
                stroke(w, b.origin, &[(0.0, y), (bw, y)], 1.0, grid, false);
            }
            let x = |i: usize| (i as f32 + 0.5) / n.max(1) as f32 * bw;
            // Grow from the stack's own floor: zero, or the stream's centre line.
            let floor = if floats { norm(0.0, bottom, top) } else { 0.0 };
            let y = |v: f32| bh - (floor + (norm(v, bottom, top) - floor) * g) * bh;
            for (k, layer) in layers.iter().enumerate() {
                let upper: Vec<(f32, f32)> = (0..n).map(|i| (x(i), y(layer[i].1))).collect();
                let lower: Vec<(f32, f32)> = (0..n).rev().map(|i| (x(i), y(layer[i].0))).collect();
                if upper.len() < 2 {
                    continue;
                }
                // Quads per segment stay convex.
                for i in 0..n - 1 {
                    let quad = [upper[i], upper[i + 1], lower[n - 2 - i], lower[n - 1 - i]];
                    let a = (0.45 + 0.3 * looks[k].lift) * looks[k].alpha;
                    fill(w, b.origin, &quad, scale_alpha(cols[k], a));
                }
                if let Some(kind) = crate::gpui::common::hatch_of(k).filter(|_| patterns) {
                    let mut poly = upper.clone();
                    poly.extend(lower.iter().copied());
                    crate::gpui::common::hatch_poly(w, b.origin, &poly, hatch_ink, kind);
                }
                glow_stroke(
                    w,
                    b.origin,
                    &upper,
                    2.0,
                    scale_alpha(cols[k], looks[k].alpha),
                    glow,
                );
            }
        })
        .absolute()
        .size_full();
        let hits = div()
            .id(self.follow.sub("layers").element_id(window, cx))
            .absolute()
            .size_full()
            .child(cv);
        let hits = focus.pointer(hits, &bounds, move |x, y, w, h| {
            if w <= 0.0 || h <= 0.0 || hit_layers.is_empty() {
                return None;
            }
            let i = ((x / w * n as f32) as usize).min(n.saturating_sub(1));
            let v = bottom + (1.0 - y / h) * (top - bottom);
            hit_layers
                .iter()
                .position(|l| l.get(i).is_some_and(|(lo, hi)| (*lo..=*hi).contains(&v)))
        });
        let mut plot = div().relative().w_full().h(px(self.height)).child(hits);
        plot = plot.child(crate::gpui::annotate::layer(
            &t,
            &self.annotations,
            (bottom, top),
            &crate::gpui::annotate::category_x(n),
        ));

        let x_labels = if self.labels.len() == n {
            self.labels.clone()
        } else {
            Vec::new()
        };
        let sums: Vec<f32> = rows.iter().map(|r| r.iter().sum()).collect();
        let whole: f32 = sums.iter().sum();
        let text = focus.shown(cx).and_then(|k| {
            let s = self.series.get(k)?;
            let peak = crate::peak(s.values.iter().copied());
            let share = if whole > 0.0 {
                sums[k] / whole * 100.0
            } else {
                0.0
            };
            Some(format!(
                "{} · peak {} · {share:.0}% of the stack",
                s.name,
                fmt_with(&self.fmt, peak)
            ))
        });
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(framed(
                &t,
                ticks
                    .iter()
                    .map(|v| match (floats, expand, &self.fmt) {
                        (true, _, _) => String::new(),
                        (_, true, None) => format!("{:.0}%", v * 100.0),
                        _ => fmt_with(&self.fmt, *v),
                    })
                    .collect(),
                x_labels,
                plot,
            ))
            .child(crate::gpui::focus::readout(&t, text, "Hover a layer"));
        if self.legend && self.series.len() > 1 {
            root = root.child(Legend::of_series(&t, &self.series));
        }
        titled(&t, self.title, root)
    }
}

dodai_gpui::entry_render!(StackedAreaChart);

// ─────────────────────────────── Candlestick ───────────────────────────────

pub use crate::Candle;

/// How candles are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CandleStyle {
    /// Plain candlesticks.
    #[default]
    Candles,
    /// Smoothed Heikin-Ashi candles.
    HeikinAshi,
    /// OHLC bars: open tick left, close tick right.
    Ohlc,
}

/// Price overlay drawn on the main pane.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CandleOverlay {
    /// Simple moving average over this many candles.
    Sma(usize),
    /// Exponential moving average over this many candles.
    Ema(usize),
    /// `(period, deviations)`
    Bollinger(usize, f32),
}

/// Indicator pane drawn under the candles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CandlePane {
    /// Relative strength index over this many candles.
    Rsi(usize),
    /// `(fast, slow, signal)`
    Macd(usize, usize, usize),
}

/// A buy or sell marker on candle `index`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TradeMarker {
    /// Candle index the trade happened on.
    pub index: usize,
    /// Trade price.
    pub price: f32,
    /// `true` for a buy, `false` for a sell.
    pub buy: bool,
}

impl TradeMarker {
    /// A buy marker at candle `index` and `price`.
    pub fn buy(index: usize, price: f32) -> Self {
        Self {
            index,
            price,
            buy: true,
        }
    }
    /// A sell marker at candle `index` and `price`.
    pub fn sell(index: usize, price: f32) -> Self {
        Self {
            index,
            price,
            buy: false,
        }
    }
}

type Line = Vec<Option<f32>>;

/// Overlay lines `(values, alpha)` for `o` over `closes`. Pure.
pub fn overlay_lines(o: CandleOverlay, closes: &[f32]) -> Vec<(Line, f32)> {
    use crate::indicators::{bollinger, ema, sma};
    match o {
        CandleOverlay::Sma(p) => vec![(sma(closes, p), 1.0)],
        CandleOverlay::Ema(p) => vec![(ema(closes, p), 1.0)],
        CandleOverlay::Bollinger(p, k) => {
            let (m, u, l) = bollinger(closes, p, k);
            vec![(m, 1.0), (u, 0.55), (l, 0.55)]
        }
    }
}

/// Runs of consecutive `Some` values as pixel polylines.
fn runs(v: &[Option<f32>], pt: impl Fn(usize, f32) -> (f32, f32)) -> Vec<Vec<(f32, f32)>> {
    let mut out: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut cur: Vec<(f32, f32)> = Vec::new();
    for (i, x) in v.iter().enumerate() {
        match x {
            Some(x) => cur.push(pt(i, *x)),
            None => {
                if cur.len() > 1 {
                    out.push(std::mem::take(&mut cur));
                }
                cur.clear();
            }
        }
    }
    if cur.len() > 1 {
        out.push(cur);
    }
    out
}

/// Candlestick chart: candles, Heikin-Ashi or OHLC bars, optional volume,
/// SMA/EMA/Bollinger overlays, RSI/MACD panes and trade markers.
#[derive(IntoElement)]
pub struct Candlestick {
    id: SharedString,
    follow: dodai_gpui::follow::Follow,
    candles: Vec<Candle>,
    labels: Vec<SharedString>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
    style: CandleStyle,
    volume: bool,
    overlays: Vec<CandleOverlay>,
    panes: Vec<CandlePane>,
    markers: Vec<TradeMarker>,
}
chart_cfg!(Candlestick);

/// Everything the candlestick canvas needs to paint one frame.
struct CandlePaint {
    /// Gridline heights as fractions of the plot.
    tf: Vec<f32>,
    cs: Vec<Candle>,
    lines: Vec<(Line, f32, usize)>,
    line_cols: Vec<Hsla>,
    markers: Vec<TradeMarker>,
    lo: f32,
    hi: f32,
    vmax: f32,
    up: Hsla,
    down: Hsla,
    grid: Hsla,
    /// Grow-in progress.
    g: f32,
    show_vol: bool,
    style: CandleStyle,
}

impl CandlePaint {
    fn paint(&self, b: Bounds<Pixels>, w: &mut Window) {
        let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
        for f in &self.tf {
            let y = bh - f * bh;
            stroke(w, b.origin, &[(0.0, y), (bw, y)], 1.0, self.grid, false);
        }
        let slot = bw / self.cs.len().max(1) as f32;
        let yr = |v: f32| bh - norm(v, self.lo, self.hi) * bh;
        if self.show_vol && self.vmax > 0.0 {
            self.volume(w, b.origin, slot, bh);
        }
        self.candles(w, b.origin, slot, &yr);
        self.overlays(w, b.origin, slot, &yr);
        self.trades(w, b.origin, slot, &yr);
    }

    fn colour(&self, c: &Candle) -> Hsla {
        if c.bullish() {
            self.up
        } else {
            self.down
        }
    }

    /// Volume bars along the bottom.
    fn volume(&self, w: &mut Window, origin: gpui::Point<Pixels>, slot: f32, bh: f32) {
        let n = self.cs.len();
        for (i, c) in self.cs.iter().enumerate() {
            let h = c.volume / self.vmax * bh * 0.18 * dodai_gpui::motion::reveal(self.g, i, n);
            let bwid = (slot * 0.6).min(24.0);
            fill(
                w,
                origin,
                &rect((i as f32 + 0.5) * slot - bwid / 2.0, bh - h, bwid, h),
                scale_alpha(self.colour(c), 0.3),
            );
        }
    }

    /// Wicks plus bodies (or open/close ticks for OHLC).
    fn candles(
        &self,
        w: &mut Window,
        origin: gpui::Point<Pixels>,
        slot: f32,
        yr: &dyn Fn(f32) -> f32,
    ) {
        let n = self.cs.len();
        for (i, c) in self.cs.iter().enumerate() {
            let ym = yr((c.high + c.low) / 2.0);
            let gi = dodai_gpui::motion::reveal(self.g, i, n);
            let y = |v: f32| ym + (yr(v) - ym) * gi;
            let col = self.colour(c);
            let x = (i as f32 + 0.5) * slot;
            stroke(w, origin, &[(x, y(c.high)), (x, y(c.low))], 1.5, col, false);
            if self.style == CandleStyle::Ohlc {
                let half = (slot * 0.3).min(8.0);
                let open = [(x - half, y(c.open)), (x, y(c.open))];
                let close = [(x, y(c.close)), (x + half, y(c.close))];
                stroke(w, origin, &open, 1.5, col, false);
                stroke(w, origin, &close, 1.5, col, false);
            } else {
                let (top, bot) = (y(c.open.max(c.close)), y(c.open.min(c.close)));
                let bwid = (slot * 0.6).min(24.0);
                fill(
                    w,
                    origin,
                    &rect(x - bwid / 2.0, top, bwid, (bot - top).max(1.5)),
                    col,
                );
            }
        }
    }

    /// Indicator overlays (moving averages and the like) over the candles.
    fn overlays(
        &self,
        w: &mut Window,
        origin: gpui::Point<Pixels>,
        slot: f32,
        yr: &dyn Fn(f32) -> f32,
    ) {
        for (l, a, k) in &self.lines {
            for run in runs(l, |i, v| ((i as f32 + 0.5) * slot, yr(v))) {
                stroke(
                    w,
                    origin,
                    &run,
                    1.5,
                    scale_alpha(self.line_cols[*k], *a),
                    false,
                );
            }
        }
    }

    /// Buy / sell triangles.
    fn trades(
        &self,
        w: &mut Window,
        origin: gpui::Point<Pixels>,
        slot: f32,
        yr: &dyn Fn(f32) -> f32,
    ) {
        for m in &self.markers {
            let x = (m.index as f32 + 0.5) * slot;
            let y0 = yr(m.price);
            let (tri, col) = if m.buy {
                (
                    [(x, y0 + 2.0), (x - 5.0, y0 + 11.0), (x + 5.0, y0 + 11.0)],
                    self.up,
                )
            } else {
                (
                    [(x, y0 - 2.0), (x - 5.0, y0 - 11.0), (x + 5.0, y0 - 11.0)],
                    self.down,
                )
            };
            fill(w, origin, &tri, col);
        }
    }
}

impl Candlestick {
    dodai_gpui::follow_api!();
    /// Create a new [`Candlestick`].
    #[track_caller]
    pub fn new(candles: impl IntoIterator<Item = Candle>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            id: SharedString::default(),
            candles: candles.into_iter().collect(),
            labels: Vec::new(),
            height: 220.0,
            fmt: None,
            title: None,
            style: CandleStyle::Candles,
            volume: false,
            overlays: Vec::new(),
            panes: Vec::new(),
            markers: Vec::new(),
        }
    }
    /// Labels, one per item.
    pub fn labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels_of(l);
        self
    }
    /// Candles (default), Heikin-Ashi or OHLC bars.
    pub fn style(mut self, s: CandleStyle) -> Self {
        self.style = s;
        self
    }
    /// Volume bars along the bottom of the price pane.
    pub fn volume(mut self, on: bool) -> Self {
        self.volume = on;
        self
    }
    /// Add a price overlay (call repeatedly).
    pub fn overlay(mut self, o: CandleOverlay) -> Self {
        self.overlays.push(o);
        self
    }
    /// Add an indicator pane under the chart (call repeatedly).
    pub fn pane(mut self, p: CandlePane) -> Self {
        self.panes.push(p);
        self
    }
    /// Buy/sell markers.
    pub fn markers(mut self, m: impl IntoIterator<Item = TradeMarker>) -> Self {
        self.markers = m.into_iter().collect();
        self
    }
}

const PANE_H: f32 = 72.0;

impl Candlestick {
    fn indicator_pane(
        &self,
        t: &dodai_core::Theme,
        pane: CandlePane,
        closes: &[f32],
        g: f32,
    ) -> gpui::Div {
        use crate::indicators::{macd, rsi};
        let (up, down, grid) = (up_color(t), down_color(t), hsla(t.palette.border));
        let c1 = hsla(t.palette.series[0]);
        let c2 = hsla(t.palette.series[1]);
        let glow = glow_of(t);
        let n = closes.len();
        let (labels, cv) = match pane {
            CandlePane::Rsi(p) => {
                let r = rsi(closes, p);
                let cv = canvas(
                    |_, _, _| (),
                    move |b: Bounds<Pixels>, _, w, _| {
                        let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                        for lvl in [30.0f32, 70.0] {
                            let y = bh - lvl / 100.0 * bh;
                            stroke(w, b.origin, &[(0.0, y), (bw, y)], 1.0, grid, false);
                        }
                        let slot = bw / n.max(1) as f32;
                        for run in runs(&r, |i, v| {
                            ((i as f32 + 0.5) * slot, bh - v / 100.0 * bh * g)
                        }) {
                            glow_stroke(w, b.origin, &run, 1.5, c1, glow);
                        }
                    },
                );
                (vec!["0".to_string(), "50".into(), "100".into()], cv)
            }
            CandlePane::Macd(f, s, sg) => {
                let (m, sig, h) = macd(closes, f, s, sg);
                let top = h
                    .iter()
                    .chain(&m)
                    .chain(&sig)
                    .flatten()
                    .fold(0.0f32, |a, v| a.max(v.abs()))
                    .max(1e-6);
                let cv = canvas(
                    |_, _, _| (),
                    move |b: Bounds<Pixels>, _, w, _| {
                        let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                        let slot = bw / n.max(1) as f32;
                        let ymid = bh / 2.0;
                        stroke(w, b.origin, &[(0.0, ymid), (bw, ymid)], 1.0, grid, false);
                        let y = |v: f32| ymid - v / top * ymid * g;
                        for (i, v) in h.iter().enumerate() {
                            if let Some(v) = v {
                                let (y0, y1) = (ymid, y(*v));
                                let col = if *v >= 0.0 { up } else { down };
                                let bwid = slot * 0.6;
                                fill(
                                    w,
                                    b.origin,
                                    &rect(
                                        (i as f32 + 0.5) * slot - bwid / 2.0,
                                        y0.min(y1),
                                        bwid,
                                        (y0 - y1).abs().max(0.5),
                                    ),
                                    scale_alpha(col, 0.6),
                                );
                            }
                        }
                        for (line, col) in [(&m, c1), (&sig, c2)] {
                            for run in runs(line, |i, v| ((i as f32 + 0.5) * slot, y(v))) {
                                glow_stroke(w, b.origin, &run, 1.5, col, glow);
                            }
                        }
                    },
                );
                (
                    vec![
                        fmt_with(&self.fmt, -top),
                        "0".into(),
                        fmt_with(&self.fmt, top),
                    ],
                    cv,
                )
            }
        };
        let plot = div()
            .relative()
            .w_full()
            .h(px(PANE_H))
            .child(cv.absolute().size_full());
        framed(t, labels, Vec::new(), plot)
    }

    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.id = self.follow.element_id(window, cx).to_string().into();
        let flat: Vec<f32> = self
            .candles
            .iter()
            .flat_map(|c| [c.open, c.high, c.low, c.close])
            .collect();
        let mut shown = dodai_gpui::follow::follow_all(window, cx, &self.follow, &flat).into_iter();
        for c in &mut self.candles {
            c.open = shown.next().unwrap_or(c.open);
            c.high = shown.next().unwrap_or(c.high);
            c.low = shown.next().unwrap_or(c.low);
            c.close = shown.next().unwrap_or(c.close);
        }
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "Candlestick");
        let t = theme_of(cx);
        let shown: Vec<Candle> = match self.style {
            CandleStyle::HeikinAshi => crate::heikin_ashi(&self.candles),
            _ => self.candles.clone(),
        };
        let closes: Vec<f32> = shown.iter().map(|c| c.close).collect();
        let lines: Vec<(Line, f32, usize)> = self
            .overlays
            .iter()
            .enumerate()
            .flat_map(|(k, o)| {
                overlay_lines(*o, &closes)
                    .into_iter()
                    .map(move |(l, a)| (l, a, k))
            })
            .collect();
        let (lo, hi, ticks) = crate::tick_span(
            shown
                .iter()
                .flat_map(|c| [c.low, c.high])
                .chain(lines.iter().flat_map(|l| l.0.iter().flatten().copied())),
            4,
        );
        let (up, down, grid) = (up_color(&t), down_color(&t), hsla(t.palette.border));
        let n = shown.len();
        let line_cols: Vec<Hsla> = (0..self.overlays.len())
            .map(|k| hsla(t.palette.series[(k + 2) % t.palette.series.len()]))
            .collect();
        let paint = CandlePaint {
            tf: ticks.iter().map(|v| norm(*v, lo, hi)).collect(),
            vmax: shown.iter().map(|c| c.volume).fold(0.0f32, f32::max),
            cs: shown.clone(),
            lines,
            line_cols,
            markers: self.markers.clone(),
            lo,
            hi,
            up,
            down,
            grid,
            g,
            show_vol: self.volume,
            style: self.style,
        };
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| paint.paint(b, w),
        )
        .absolute()
        .size_full();
        let mut plot = div().relative().w_full().h(px(self.height)).child(cv);
        let mut hit = div().absolute().size_full().flex();
        for (i, c) in shown.iter().enumerate() {
            let label = self.labels.get(i).cloned().unwrap_or_default();
            let vol = if c.volume > 0.0 {
                format!(" V {}", fmt_num(c.volume))
            } else {
                String::new()
            };
            hit = hit.child(div().id((self.id.clone(), i)).flex_1().h_full().tooltip(
                DodaiTooltip::text(format!(
                    "{label} O {} H {} L {} C {}{vol}",
                    fmt_with(&self.fmt, c.open),
                    fmt_with(&self.fmt, c.high),
                    fmt_with(&self.fmt, c.low),
                    fmt_with(&self.fmt, c.close)
                )),
            ));
        }
        plot = plot.child(hit);
        let x_labels = if self.labels.len() == n && self.panes.is_empty() {
            self.labels.clone()
        } else {
            Vec::new()
        };
        let mut root = div().flex().flex_col().gap(px(4.0)).w_full().child(framed(
            &t,
            ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
            x_labels,
            plot,
        ));
        for p in self.panes.iter() {
            root = root.child(self.indicator_pane(&t, *p, &closes, g));
        }
        titled(&t, self.title, root)
    }
}

dodai_gpui::entry_render!(Candlestick);

#[cfg(test)]
mod candle_tests {
    use super::*;

    #[test]
    fn overlays_have_expected_lines() {
        let c: Vec<f32> = (0..10).map(|i| i as f32).collect();
        assert_eq!(overlay_lines(CandleOverlay::Sma(3), &c).len(), 1);
        assert_eq!(overlay_lines(CandleOverlay::Bollinger(3, 2.0), &c).len(), 3);
    }

    #[test]
    fn runs_split_on_gaps() {
        let v = [Some(1.0), Some(2.0), None, Some(3.0), Some(4.0), Some(5.0)];
        let r = runs(&v, |i, y| (i as f32, y));
        assert_eq!(r.len(), 2);
        assert_eq!(r[1].len(), 3);
    }

    #[test]
    fn markers_build() {
        assert!(TradeMarker::buy(1, 2.0).buy);
        assert!(!TradeMarker::sell(1, 2.0).buy);
    }
}
