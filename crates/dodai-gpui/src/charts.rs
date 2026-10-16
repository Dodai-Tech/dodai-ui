//! Charts drawn on gpui's canvas / flex layout: [`LineChart`], [`BarChart`],
//! [`PieChart`] (+ donut), [`Sparkline`], [`Gauge`], [`RadarChart`],
//! [`ScatterPlot`], [`Heatmap`] and the shared [`Legend`].
//!
//! Colours come from `Palette::series`; lines glow when the theme has glow
//! effects; bars use the theme's shape (chamfer on cyberpunk).

use crate::paint::{glow_layers, scale_alpha, shaped, ShapeStyle};
use crate::panel::DodaiTooltip;
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_core::{Color, Theme, Tone};
use gpui::{
    canvas, div, point, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement,
    ParentElement, PathBuilder, Pixels, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window,
};
use std::rc::Rc;

// ───────────────────────────── pure helpers ─────────────────────────────

/// "Nice" axis ticks covering `min..=max` with about `count` steps. Pure.
pub fn nice_ticks(min: f32, max: f32, count: usize) -> Vec<f32> {
    let count = count.max(1) as f32;
    let (lo, hi) = if max > min {
        (min, max)
    } else {
        (min, min + 1.0)
    };
    let raw = (hi - lo) / count;
    let mag = 10f32.powf(raw.log10().floor());
    let norm = raw / mag;
    let step = mag
        * if norm <= 1.0 {
            1.0
        } else if norm <= 2.0 {
            2.0
        } else if norm <= 5.0 {
            5.0
        } else {
            10.0
        };
    let start = (lo / step).floor() * step;
    let mut out = Vec::new();
    let mut v = start;
    while v < hi + step * 0.999 {
        out.push((v / step).round() * step);
        v += step;
        if out.len() > 64 {
            break;
        }
    }
    out
}

/// Compact number label: `1200` → `1.2k`, `3.0` → `3`.
pub fn fmt_num(v: f32) -> String {
    let a = v.abs();
    if a >= 1_000_000.0 {
        format!("{}M", trim(v / 1_000_000.0))
    } else if a >= 1000.0 {
        format!("{}k", trim(v / 1000.0))
    } else {
        trim(v)
    }
}

pub(crate) fn trim(v: f32) -> String {
    let s = format!("{v:.1}");
    s.strip_suffix(".0").map(str::to_string).unwrap_or(s)
}

/// Catmull-Rom smoothing: `seg` interpolated points per input segment. Pure.
pub fn smooth_points(p: &[(f32, f32)], seg: usize) -> Vec<(f32, f32)> {
    if p.len() < 3 || seg < 2 {
        return p.to_vec();
    }
    let mut out = Vec::with_capacity(p.len() * seg);
    for i in 0..p.len() - 1 {
        let p0 = p[i.saturating_sub(1)];
        let (p1, p2) = (p[i], p[i + 1]);
        let p3 = p[(i + 2).min(p.len() - 1)];
        for s in 0..seg {
            let t = s as f32 / seg as f32;
            let (t2, t3) = (t * t, t * t * t);
            let f = |a: f32, b: f32, c: f32, d: f32| {
                0.5 * (2.0 * b
                    + (c - a) * t
                    + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
                    + (3.0 * b - a - 3.0 * c + d) * t3)
            };
            out.push((f(p0.0, p1.0, p2.0, p3.0), f(p0.1, p1.1, p2.1, p3.1)));
        }
    }
    out.push(*p.last().unwrap());
    out
}

/// Slice `(start, end)` fractions of a full turn for `values`. Pure.
pub fn pie_fractions(values: &[f32]) -> Vec<(f32, f32)> {
    let total: f32 = values.iter().map(|v| v.max(0.0)).sum();
    let mut acc = 0.0;
    values
        .iter()
        .map(|v| {
            let s = acc;
            if total > 0.0 {
                acc += v.max(0.0) / total;
            }
            (s, acc)
        })
        .collect()
}

/// Point on a circle; fraction 0 is 12 o'clock, clockwise. Pure.
pub fn polar(cx: f32, cy: f32, r: f32, frac: f32) -> (f32, f32) {
    let th = frac * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    (cx + r * th.cos(), cy + r * th.sin())
}

/// Staircase version of a polyline: horizontal then vertical. Pure.
pub fn step_points(p: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut out = Vec::with_capacity(p.len() * 2);
    for (i, &(x, y)) in p.iter().enumerate() {
        if let Some(&(_, py)) = i.checked_sub(1).and_then(|j| p.get(j)) {
            out.push((x, py));
        }
        out.push((x, y));
    }
    out
}

/// Arc points between two turn fractions. Pure.
pub fn turn_arc(cx: f32, cy: f32, r: f32, a: f32, b: f32) -> Vec<(f32, f32)> {
    let n = (((b - a).abs() * 360.0 / 4.0).ceil() as usize).max(2);
    (0..=n)
        .map(|i| polar(cx, cy, r, a + (b - a) * i as f32 / n as f32))
        .collect()
}

/// Value → `0..=1` within `lo..=hi` (0 when degenerate). Pure.
pub fn norm(v: f32, lo: f32, hi: f32) -> f32 {
    if hi > lo {
        ((v - lo) / (hi - lo)).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub(crate) fn pts(o: gpui::Point<Pixels>, p: &[(f32, f32)]) -> Vec<gpui::Point<Pixels>> {
    p.iter()
        .map(|&(x, y)| point(o.x + px(x), o.y + px(y)))
        .collect()
}

pub(crate) fn stroke(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    p: &[(f32, f32)],
    width: f32,
    c: Hsla,
    closed: bool,
) {
    let mut pb = PathBuilder::stroke(px(width));
    pb.add_polygon(&pts(o, p), closed);
    if let Ok(path) = pb.build() {
        w.paint_path(path, c);
    }
}

pub(crate) fn fill(w: &mut Window, o: gpui::Point<Pixels>, p: &[(f32, f32)], c: Hsla) {
    let mut pb = PathBuilder::fill();
    pb.add_polygon(&pts(o, p), true);
    if let Ok(path) = pb.build() {
        w.paint_path(path, c);
    }
}

/// Glow `(radius)` when the theme glows. Pure.
pub(crate) fn glow_of(t: &Theme) -> Option<f32> {
    t.effects.glow_enabled().then(|| t.effects.glow_radius())
}

/// Stroke with glow layers underneath.
pub(crate) fn glow_stroke(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    p: &[(f32, f32)],
    width: f32,
    c: Hsla,
    glow: Option<f32>,
) {
    if let Some(r) = glow {
        for l in glow_layers(r, 0.0) {
            stroke(w, o, p, width + l.width, scale_alpha(c, l.alpha), false);
        }
    }
    stroke(w, o, p, width, c, false);
}

// ─────────────────────────────── data types ───────────────────────────────

/// Named numeric series; colour defaults to `palette.series[index]`.
#[derive(Clone)]
pub struct Series {
    pub name: SharedString,
    pub values: Vec<f32>,
    pub color: Option<Color>,
}

impl Series {
    pub fn new(name: impl Into<SharedString>, values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            name: name.into(),
            values: values.into_iter().collect(),
            color: None,
        }
    }
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

pub(crate) fn series_color(t: &Theme, s: &Series, i: usize) -> Color {
    s.color.unwrap_or_else(|| t.palette.series_at(i))
}

pub(crate) fn y_range(
    series: &[Series],
    zero: bool,
    fixed: Option<(f32, f32)>,
) -> (f32, f32, Vec<f32>) {
    if let Some((lo, hi)) = fixed {
        let ticks = nice_ticks(lo, hi, 4);
        return (
            lo,
            hi,
            ticks.into_iter().filter(|v| *v >= lo && *v <= hi).collect(),
        );
    }
    let vals = series.iter().flat_map(|s| s.values.iter().copied());
    let (mut lo, mut hi) = vals.fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(v), h.max(v)));
    if lo > hi {
        (lo, hi) = (0.0, 1.0);
    }
    if zero {
        lo = lo.min(0.0);
    }
    let ticks = nice_ticks(lo, hi, 4);
    (
        *ticks.first().unwrap_or(&lo),
        *ticks.last().unwrap_or(&hi),
        ticks,
    )
}

// ─────────────────────────────── Legend ───────────────────────────────

/// Colour swatch + name row, shared by all multi-series charts.
#[derive(IntoElement)]
pub struct Legend {
    items: Vec<(SharedString, Color)>,
}

impl Legend {
    pub fn new(items: impl IntoIterator<Item = (SharedString, Color)>) -> Self {
        Self {
            items: items.into_iter().collect(),
        }
    }
    /// Legend for `series` using palette colours.
    pub fn of_series(t: &Theme, series: &[Series]) -> Self {
        Self::new(
            series
                .iter()
                .enumerate()
                .map(|(i, s)| (s.name.clone(), series_color(t, s, i))),
        )
    }
}

impl RenderOnce for Legend {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        div()
            .flex()
            .flex_wrap()
            .gap(px(t.spacing.md))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_xs))
            .text_color(hsla(t.palette.text_muted))
            .children(self.items.into_iter().map(|(n, c)| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(t.spacing.xs))
                    .child(div().w(px(10.0)).h(px(10.0)).bg(hsla(c)))
                    .child(n)
            }))
    }
}

// ─────────────────────────── axes frame (shared) ───────────────────────────

pub(crate) fn label_style(t: &Theme, d: gpui::Div) -> gpui::Div {
    d.font_family(first_family(t.typography.mono))
        .text_size(px(t.typography.size_xs))
        .text_color(hsla(t.palette.text_muted))
}

/// Y-axis labels (top = max) + plot + optional X labels underneath.
pub(crate) fn framed(
    t: &Theme,
    y_labels: Vec<String>,
    x_labels: Vec<SharedString>,
    plot: gpui::Div,
) -> gpui::Div {
    let ycol = label_style(t, div())
        .w(px(36.0))
        .flex()
        .flex_col()
        .justify_between()
        .items_end()
        .pr(px(t.spacing.xs))
        .children(y_labels.into_iter().rev().map(|l| div().child(l)));
    let mut right = div()
        .flex_1()
        .flex()
        .flex_col()
        .gap(px(t.spacing.xs))
        .child(plot);
    if !x_labels.is_empty() {
        right = right.child(
            label_style(t, div()).flex().children(
                x_labels
                    .into_iter()
                    .map(|l| div().flex_1().flex().justify_center().child(l)),
            ),
        );
    }
    div().flex().w_full().child(ycol).child(right)
}

/// Number formatter shared by chart configuration.
pub type NumFmt = Rc<dyn Fn(f32) -> String>;

pub(crate) fn fmt_with(f: &Option<NumFmt>, v: f32) -> String {
    f.as_ref().map_or_else(|| fmt_num(v), |f| f(v))
}

/// Optional chart title above `body`.
pub(crate) fn titled(t: &Theme, title: Option<SharedString>, body: gpui::Div) -> gpui::Div {
    match title {
        None => body,
        Some(s) => div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(
                div()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(t.typography.size_md))
                    .text_color(hsla(t.palette.text))
                    .child(s),
            )
            .child(body),
    }
}

pub(crate) fn rect(x: f32, y: f32, w: f32, h: f32) -> [(f32, f32); 4] {
    [(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
}

pub(crate) fn up_color(t: &Theme) -> Hsla {
    hsla(t.palette.tone(Tone::Success))
}
pub(crate) fn down_color(t: &Theme) -> Hsla {
    hsla(t.palette.tone(Tone::Danger))
}

macro_rules! chart_cfg {
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

pub(crate) fn labels_of<S: Into<SharedString>>(
    l: impl IntoIterator<Item = S>,
) -> Vec<SharedString> {
    l.into_iter().map(Into::into).collect()
}

pub(crate) use chart_cfg;

// ─────────────────────────────── LineChart ───────────────────────────────

/// Line / area chart with gridlines, axes, optional smoothing, point markers
/// (with hover tooltips) and glow on glowing themes.
#[derive(IntoElement)]
pub struct LineChart {
    id: SharedString,
    series: Vec<Series>,
    labels: Vec<SharedString>,
    height: f32,
    area: bool,
    smooth: bool,
    points: bool,
    legend: bool,
    y_bounds: Option<(f32, f32)>,
    fmt: Option<NumFmt>,
    grid: bool,
    stroke_w: f32,
    stepped: bool,
    zero: bool,
    title: Option<SharedString>,
}

impl LineChart {
    pub fn new(id: impl Into<SharedString>, series: impl IntoIterator<Item = Series>) -> Self {
        Self {
            id: id.into(),
            series: series.into_iter().collect(),
            labels: Vec::new(),
            height: 200.0,
            area: false,
            smooth: false,
            points: true,
            legend: true,
            y_bounds: None,
            fmt: None,
            grid: true,
            stroke_w: 2.0,
            stepped: false,
            zero: true,
            title: None,
        }
    }
    /// Fix the Y axis to `lo..=hi` (default: fit data, including zero).
    pub fn y_range(mut self, lo: f32, hi: f32) -> Self {
        self.y_bounds = Some((lo, hi));
        self
    }
    /// Custom number formatter for axis labels and tooltips.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.fmt = Some(Rc::new(f));
        self
    }
    /// Show horizontal gridlines (default on).
    pub fn grid(mut self, g: bool) -> Self {
        self.grid = g;
        self
    }
    /// Line thickness in px (default 2).
    pub fn stroke_width(mut self, w: f32) -> Self {
        self.stroke_w = w;
        self
    }
    /// Step (staircase) interpolation instead of straight segments.
    pub fn stepped(mut self, s: bool) -> Self {
        self.stepped = s;
        self
    }
    /// Force the Y axis to include zero (default on).
    pub fn include_zero(mut self, z: bool) -> Self {
        self.zero = z;
        self
    }
    /// Title above the chart.
    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = Some(t.into());
        self
    }
    /// X-axis category labels.
    pub fn labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.labels = l.into_iter().map(Into::into).collect();
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// Fill under each line.
    pub fn area(mut self, a: bool) -> Self {
        self.area = a;
        self
    }
    pub fn smooth(mut self, s: bool) -> Self {
        self.smooth = s;
        self
    }
    /// Hoverable point markers (default on).
    pub fn points(mut self, p: bool) -> Self {
        self.points = p;
        self
    }
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

impl RenderOnce for LineChart {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (lo, hi, ticks) = y_range(&self.series, self.zero, self.y_bounds);
        let n = self
            .series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(0);
        let grid = hsla(t.palette.border);
        let glow = glow_of(&t);
        let (area, smooth, stepped, sw, show_grid) = (
            self.area,
            self.smooth,
            self.stepped,
            self.stroke_w,
            self.grid,
        );
        let lines: Vec<(Hsla, Vec<f32>)> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| (hsla(series_color(&t, s, i)), s.values.clone()))
            .collect();
        let tick_fracs: Vec<f32> = ticks.iter().map(|v| norm(*v, lo, hi)).collect();

        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                for f in tick_fracs.iter().filter(|_| show_grid) {
                    let y = bh - f * bh;
                    stroke(w, b.origin, &[(0.0, y), (bw, y)], 1.0, grid, false);
                }
                for (c, vals) in &lines {
                    let m = vals.len();
                    let raw: Vec<(f32, f32)> = vals
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            // Centre of category slot `i`, matching the x labels.
                            let x = (i as f32 + 0.5) / m as f32 * bw;
                            (x, bh - norm(*v, lo, hi) * bh)
                        })
                        .collect();
                    let raw = if stepped { step_points(&raw) } else { raw };
                    let p = if smooth && !stepped {
                        smooth_points(&raw, 8)
                    } else {
                        raw
                    };
                    if p.len() < 2 {
                        continue;
                    }
                    if area {
                        let mut poly = p.clone();
                        poly.push((p.last().unwrap().0, bh));
                        poly.push((p[0].0, bh));
                        fill(w, b.origin, &poly, scale_alpha(*c, 0.18));
                    }
                    glow_stroke(w, b.origin, &p, sw, *c, glow);
                }
            },
        )
        .absolute()
        .size_full();

        let mut plot = div().relative().w_full().h(px(self.height)).child(cv);
        if self.points {
            for (si, s) in self.series.iter().enumerate() {
                let c = hsla(series_color(&t, s, si));
                let m = s.values.len();
                for (i, v) in s.values.iter().enumerate() {
                    let fx = (i as f32 + 0.5) / m as f32;
                    let fy = 1.0 - norm(*v, lo, hi);
                    let label = self.labels.get(i).cloned().unwrap_or_default();
                    plot = plot.child(
                        div()
                            .id((SharedString::from(format!("{}-{si}-{i}", self.id)), i))
                            .absolute()
                            .left(relative(fx))
                            .top(relative(fy))
                            .ml(px(-5.0))
                            .mt(px(-5.0))
                            .w(px(10.0))
                            .h(px(10.0))
                            .rounded_full()
                            .hover(move |d| d.bg(c))
                            .tooltip(DodaiTooltip::text(format!(
                                "{} {label}: {}",
                                s.name,
                                fmt_with(&self.fmt, *v)
                            ))),
                    );
                }
            }
        }
        let x_labels: Vec<SharedString> = if self.labels.len() == n {
            self.labels.clone()
        } else {
            Vec::new()
        };
        let mut root = div().flex().flex_col().gap(px(t.spacing.sm)).w_full();
        root = root.child(framed(
            &t,
            ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
            x_labels,
            plot,
        ));
        if self.legend && self.series.len() > 1 {
            root = root.child(Legend::of_series(&t, &self.series));
        }
        titled(&t, self.title.clone(), root)
    }
}

// ─────────────────────────────── BarChart ───────────────────────────────

/// Bar chart: vertical or horizontal, grouped or stacked.
#[derive(IntoElement)]
pub struct BarChart {
    id: SharedString,
    series: Vec<Series>,
    labels: Vec<SharedString>,
    height: f32,
    horizontal: bool,
    stacked: bool,
    legend: bool,
    y_max: Option<f32>,
    fmt: Option<NumFmt>,
    value_labels: bool,
    bar_width: f32,
    title: Option<SharedString>,
}

impl BarChart {
    pub fn new(id: impl Into<SharedString>, series: impl IntoIterator<Item = Series>) -> Self {
        Self {
            id: id.into(),
            series: series.into_iter().collect(),
            labels: Vec::new(),
            height: 200.0,
            horizontal: false,
            stacked: false,
            legend: true,
            y_max: None,
            fmt: None,
            value_labels: false,
            bar_width: 32.0,
            title: None,
        }
    }
    /// Fix the value-axis maximum (default: fit data).
    pub fn max(mut self, m: f32) -> Self {
        self.y_max = Some(m);
        self
    }
    /// Custom number formatter for axis labels, tooltips and value labels.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.fmt = Some(Rc::new(f));
        self
    }
    /// Print the value above each bar (vertical, grouped only).
    pub fn value_labels(mut self, v: bool) -> Self {
        self.value_labels = v;
        self
    }
    /// Maximum bar thickness in px (default 32).
    pub fn bar_width(mut self, w: f32) -> Self {
        self.bar_width = w;
        self
    }
    /// Title above the chart.
    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.labels = l.into_iter().map(Into::into).collect();
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    pub fn horizontal(mut self, h: bool) -> Self {
        self.horizontal = h;
        self
    }
    pub fn stacked(mut self, s: bool) -> Self {
        self.stacked = s;
        self
    }
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

/// Max of per-category totals (stacked) or single values (grouped). Pure.
pub fn bar_max(series: &[Series], stacked: bool) -> f32 {
    let n = series.iter().map(|s| s.values.len()).max().unwrap_or(0);
    (0..n)
        .map(|i| {
            let it = series
                .iter()
                .map(|s| s.values.get(i).copied().unwrap_or(0.0).max(0.0));
            if stacked {
                it.sum::<f32>()
            } else {
                it.fold(0.0, f32::max)
            }
        })
        .fold(0.0, f32::max)
}

impl RenderOnce for BarChart {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let ticks = nice_ticks(
            0.0,
            self.y_max
                .unwrap_or_else(|| bar_max(&self.series, self.stacked)),
            4,
        );
        let top = *ticks.last().unwrap_or(&1.0);
        let n = self
            .series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(0);
        let h = self.horizontal;
        let grid = hsla(t.palette.border);

        let bar = |si: usize, i: usize, v: f32, frac: f32| {
            let s = &self.series[si];
            let style =
                ShapeStyle::from_tokens(&t, Some(series_color(&t, s, si)), None, false, false);
            let label = self.labels.get(i).cloned().unwrap_or_default();
            let el = div()
                .id((SharedString::from(format!("{}-{si}-{i}", self.id)), i))
                .tooltip(DodaiTooltip::text(format!(
                    "{} {label}: {}",
                    s.name,
                    fmt_with(&self.fmt, v)
                )))
                .child(shaped(&t.shape, style).size_full());
            if h {
                el.h_full().w(relative(frac))
            } else {
                el.w_full().h(relative(frac))
            }
        };

        let mut cats = div().flex().size_full();
        cats = if h { cats.flex_col() } else { cats };
        for i in 0..n {
            let mut cell = div()
                .flex_1()
                .size_full()
                .flex()
                .gap(px(2.0))
                .p(px(t.spacing.xs));
            if h {
                // Bars grow right; stacked bars lay segments left→right.
                cell = cell.items_center();
                if self.stacked {
                    let mut stack = div().h(px(18.0)).flex();
                    for (si, s) in self.series.iter().enumerate() {
                        let v = s.values.get(i).copied().unwrap_or(0.0).max(0.0);
                        stack = stack.child(bar(si, i, v, v / top.max(f32::EPSILON)));
                    }
                    cell = cell.child(
                        div()
                            .h_full()
                            .w_full()
                            .flex()
                            .items_center()
                            .child(stack.w_full()),
                    );
                    cell = cell.flex_col();
                } else {
                    cell = cell.flex_col().justify_center();
                    for (si, s) in self.series.iter().enumerate() {
                        let v = s.values.get(i).copied().unwrap_or(0.0).max(0.0);
                        cell = cell.child(div().w_full().h(px(14.0)).child(bar(
                            si,
                            i,
                            v,
                            norm(v, 0.0, top),
                        )));
                    }
                }
            } else {
                cell = cell.items_end().justify_center();
                if self.stacked {
                    let total: f32 = self
                        .series
                        .iter()
                        .map(|s| s.values.get(i).copied().unwrap_or(0.0).max(0.0))
                        .sum();
                    let mut stack = div()
                        .h(relative(norm(total, 0.0, top)))
                        .w_full()
                        .max_w(px(self.bar_width * 1.5))
                        .flex()
                        .flex_col_reverse();
                    for (si, s) in self.series.iter().enumerate() {
                        let v = s.values.get(i).copied().unwrap_or(0.0).max(0.0);
                        stack = stack.child(
                            bar(si, i, v, v / total.max(f32::EPSILON))
                                .h(relative(v / total.max(f32::EPSILON))),
                        );
                    }
                    cell = cell.child(stack);
                } else {
                    for (si, s) in self.series.iter().enumerate() {
                        let v = s.values.get(i).copied().unwrap_or(0.0).max(0.0);
                        cell = cell.child(
                            div()
                                .h_full()
                                .flex_1()
                                .max_w(px(self.bar_width))
                                .flex()
                                .flex_col()
                                .justify_end()
                                .children(self.value_labels.then(|| {
                                    label_style(&t, div())
                                        .flex()
                                        .justify_center()
                                        .child(fmt_with(&self.fmt, v))
                                }))
                                .child(
                                    div()
                                        .w_full()
                                        .h(relative(norm(v, 0.0, top)))
                                        .child(bar(si, i, v, 1.0)),
                                ),
                        );
                    }
                }
            }
            cats = cats.child(cell);
        }

        // Gridlines behind the bars (vertical orientation: horizontal lines).
        let grid_fracs: Vec<f32> = ticks.iter().map(|v| norm(*v, 0.0, top)).collect();
        let mut plot = div().relative().w_full().h(px(self.height));
        for f in &grid_fracs {
            let line = div().absolute().bg(grid);
            plot = plot.child(if h {
                line.left(relative(*f)).top_0().h_full().w(px(1.0))
            } else {
                line.bottom(relative(*f)).left_0().w_full().h(px(1.0))
            });
        }
        plot = plot.child(cats);

        let labels: Vec<SharedString> = (0..n)
            .map(|i| self.labels.get(i).cloned().unwrap_or_default())
            .collect();
        let body = if h {
            let ycol = label_style(&t, div())
                .w(px(56.0))
                .h(px(self.height))
                .flex()
                .flex_col()
                .children(labels.into_iter().map(|l| {
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_end()
                        .pr(px(t.spacing.xs))
                        .child(l)
                }));
            let xrow = label_style(&t, div())
                .ml(px(56.0))
                .flex()
                .justify_between()
                .children(ticks.iter().map(|v| div().child(fmt_with(&self.fmt, *v))));
            div()
                .flex()
                .flex_col()
                .w_full()
                .child(div().flex().child(ycol).child(div().flex_1().child(plot)))
                .child(xrow)
        } else {
            framed(
                &t,
                ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
                labels,
                plot,
            )
        };
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(body);
        if self.legend && self.series.len() > 1 {
            root = root.child(Legend::of_series(&t, &self.series));
        }
        titled(&t, self.title.clone(), root)
    }
}

// ─────────────────────────────── Sparkline ───────────────────────────────

/// Tiny inline line, no axes.
#[derive(IntoElement)]
pub struct Sparkline {
    values: Vec<f32>,
    w: f32,
    h: f32,
    color: Option<Color>,
    area: bool,
}

impl Sparkline {
    pub fn new(values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            values: values.into_iter().collect(),
            w: 96.0,
            h: 28.0,
            color: None,
            area: true,
        }
    }
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.w = w;
        self.h = h;
        self
    }
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    pub fn area(mut self, a: bool) -> Self {
        self.area = a;
        self
    }
}

impl RenderOnce for Sparkline {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let c = hsla(self.color.unwrap_or(t.palette.primary));
        let glow = glow_of(&t).map(|r| r.min(4.0));
        let (vals, area) = (self.values, self.area);
        let (lo, hi) = vals
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        div().w(px(self.w)).h(px(self.h)).child(
            canvas(
                |_, _, _| (),
                move |b: Bounds<Pixels>, _, w, _| {
                    let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                    let m = vals.len();
                    if m < 2 {
                        return;
                    }
                    let pad = 2.0;
                    let p: Vec<(f32, f32)> = vals
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            (
                                i as f32 / (m - 1) as f32 * bw,
                                pad + (1.0 - norm(*v, lo, hi)) * (bh - 2.0 * pad),
                            )
                        })
                        .collect();
                    if area {
                        let mut poly = p.clone();
                        poly.push((bw, bh));
                        poly.push((0.0, bh));
                        fill(w, b.origin, &poly, scale_alpha(c, 0.16));
                    }
                    glow_stroke(w, b.origin, &p, 1.5, c, glow);
                },
            )
            .size_full(),
        )
    }
}

// ───────────────────────────── PieChart / donut ─────────────────────────────

/// Pie or donut chart; `center` is shown in the donut hole.
#[derive(IntoElement)]
pub struct PieChart {
    slices: Vec<(SharedString, f32)>,
    size: f32,
    hole: f32,
    center: Option<SharedString>,
    legend: bool,
}

impl PieChart {
    pub fn new<S: Into<SharedString>>(slices: impl IntoIterator<Item = (S, f32)>) -> Self {
        Self {
            slices: slices.into_iter().map(|(n, v)| (n.into(), v)).collect(),
            size: 160.0,
            hole: 0.0,
            center: None,
            legend: true,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Donut hole as a fraction of the radius (`0` = pie).
    pub fn donut(mut self, hole: f32) -> Self {
        self.hole = hole.clamp(0.0, 0.9);
        self
    }
    pub fn center(mut self, c: impl Into<SharedString>) -> Self {
        self.center = Some(c.into());
        self
    }
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

impl RenderOnce for PieChart {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let vals: Vec<f32> = self.slices.iter().map(|s| s.1).collect();
        let fr = pie_fractions(&vals);
        let cols: Vec<Hsla> = (0..vals.len())
            .map(|i| hsla(t.palette.series_at(i)))
            .collect();
        let (size, hole) = (self.size, self.hole);
        let gap = if vals.len() > 1 { 0.004 } else { 0.0 };
        let cvs = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let r = size / 2.0 - 1.0;
                let c = size / 2.0;
                for ((a, z), col) in fr.iter().zip(&cols) {
                    if hole <= 0.0 {
                        // Convex wedges (<= 1/8 turn) tessellate without
                        // seams or holes, even for slices past 180 degrees.
                        let n = (((z - a) * 8.0).ceil() as usize).max(1);
                        for k in 0..n {
                            let w0 = a + (z - a) * k as f32 / n as f32;
                            let w1 = a + (z - a) * (k + 1) as f32 / n as f32;
                            let mut p = turn_arc(
                                c,
                                c,
                                r,
                                w0,
                                if *z >= 0.999 && k + 1 == n {
                                    w1
                                } else {
                                    w1 + 0.0015
                                },
                            );
                            p.push((c, c));
                            fill(w, b.origin, &p, *col);
                        }
                        continue;
                    }
                    let (a, z) = (a + gap, z - gap);
                    if z <= a {
                        continue;
                    }
                    // Annular sectors as small filled quads: thick strokes
                    // leave mitre artefacts and seams, fills do not.
                    let r_in = r * hole;
                    let n = (((z - a) * 24.0).ceil() as usize).max(1);
                    for k in 0..n {
                        let w0 = a + (z - a) * k as f32 / n as f32;
                        let w1 = a + (z - a) * (k + 1) as f32 / n as f32;
                        let w1 = if k + 1 == n { w1 } else { w1 + 0.001 };
                        let mut p = turn_arc(c, c, r, w0, w1);
                        let mut inner = turn_arc(c, c, r_in, w0, w1);
                        inner.reverse();
                        p.extend(inner);
                        fill(w, b.origin, &p, *col);
                    }
                }
            },
        )
        .absolute()
        .size_full();
        let mut disc = div().relative().w(px(size)).h(px(size)).child(cvs);
        if let (Some(txt), true) = (self.center.clone(), self.hole > 0.0) {
            disc = disc.child(
                div()
                    .absolute()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(t.typography.size_xl))
                    .text_color(hsla(t.palette.text))
                    .child(txt),
            );
        }
        let total: f32 = vals.iter().sum();
        let mut root = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.lg))
            .child(disc);
        if self.legend {
            root = root.child(div().flex().flex_col().gap(px(t.spacing.xs)).children(
                self.slices.iter().enumerate().map(|(i, (n, v))| {
                    let pct = if total > 0.0 { v / total * 100.0 } else { 0.0 };
                    label_style(&t, div())
                        .flex()
                        .items_center()
                        .gap(px(t.spacing.xs))
                        .child(
                            div()
                                .w(px(10.0))
                                .h(px(10.0))
                                .bg(hsla(t.palette.series_at(i))),
                        )
                        .child(format!("{n} · {}%", trim(pct)))
                }),
            ));
        }
        root
    }
}

// ─────────────────────────────────── Gauge ───────────────────────────────────

/// Radial gauge (270° sweep) with a centre label.
#[derive(IntoElement)]
pub struct Gauge {
    value: f32,
    min: f32,
    max: f32,
    size: f32,
    label: Option<SharedString>,
    tone: Option<dodai_core::Tone>,
}

impl Gauge {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            min: 0.0,
            max: 100.0,
            size: 120.0,
            label: None,
            tone: None,
        }
    }
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Centre label (default: the rounded value).
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn tone(mut self, t: dodai_core::Tone) -> Self {
        self.tone = Some(t);
        self
    }
}

impl RenderOnce for Gauge {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let frac = norm(self.value, self.min, self.max);
        let col = hsla(self.tone.map_or(t.palette.primary, |k| t.palette.tone(k)));
        let track = hsla(t.palette.border);
        let glow = glow_of(&t);
        let size = self.size;
        let ring = (size * 0.1).clamp(4.0, 12.0);
        let label = self
            .label
            .unwrap_or_else(|| fmt_num(self.value.round()).into());
        div()
            .relative()
            .w(px(size))
            .h(px(size))
            .child(
                canvas(
                    |_, _, _| (),
                    move |b: Bounds<Pixels>, _, w, _| {
                        let c = size / 2.0;
                        let r = c - ring / 2.0 - 2.0;
                        // 270° sweep from bottom-left (135° → 405°) in turn fractions.
                        let (s0, span) = (0.375, 0.75);
                        stroke(
                            w,
                            b.origin,
                            &turn_arc(c, c, r, s0, s0 + span),
                            ring,
                            track,
                            false,
                        );
                        if frac > 0.0 {
                            let p = turn_arc(c, c, r, s0, s0 + span * frac);
                            glow_stroke(w, b.origin, &p, ring, col, glow);
                        }
                    },
                )
                .absolute()
                .size_full(),
            )
            .child(
                div()
                    .absolute()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(size * 0.22))
                    .text_color(hsla(t.palette.text))
                    .child(label),
            )
    }
}

// ─────────────────────────────── RadarChart ───────────────────────────────

/// Radar (spider) chart: one polygon per series over shared axes.
#[derive(IntoElement)]
pub struct RadarChart {
    axes: Vec<SharedString>,
    series: Vec<Series>,
    size: f32,
    max: Option<f32>,
    legend: bool,
}

impl RadarChart {
    pub fn new<S: Into<SharedString>>(
        axes: impl IntoIterator<Item = S>,
        series: impl IntoIterator<Item = Series>,
    ) -> Self {
        Self {
            axes: axes.into_iter().map(Into::into).collect(),
            series: series.into_iter().collect(),
            size: 220.0,
            max: None,
            legend: true,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    pub fn max(mut self, m: f32) -> Self {
        self.max = Some(m);
        self
    }
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

impl RenderOnce for RadarChart {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let n = self.axes.len().max(3);
        let max = self.max.unwrap_or_else(|| {
            *nice_ticks(0.0, y_range(&self.series, true, None).1, 4)
                .last()
                .unwrap_or(&1.0)
        });
        let grid = hsla(t.palette.border);
        let glow = glow_of(&t);
        let size = self.size;
        let polys: Vec<(Hsla, Vec<f32>)> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| (hsla(series_color(&t, s, i)), s.values.clone()))
            .collect();
        let cvs = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let c = size / 2.0;
                let r = c - 4.0;
                for ring in 1..=4 {
                    let rr = r * ring as f32 / 4.0;
                    let p: Vec<_> = (0..n)
                        .map(|i| polar(c, c, rr, i as f32 / n as f32))
                        .collect();
                    stroke(w, b.origin, &p, 1.0, grid, true);
                }
                for i in 0..n {
                    stroke(
                        w,
                        b.origin,
                        &[(c, c), polar(c, c, r, i as f32 / n as f32)],
                        1.0,
                        grid,
                        false,
                    );
                }
                for (col, vals) in &polys {
                    let p: Vec<_> = (0..n)
                        .map(|i| {
                            polar(
                                c,
                                c,
                                r * norm(vals.get(i).copied().unwrap_or(0.0), 0.0, max),
                                i as f32 / n as f32,
                            )
                        })
                        .collect();
                    fill(w, b.origin, &p, scale_alpha(*col, 0.2));
                    let mut closed = p.clone();
                    closed.push(p[0]);
                    glow_stroke(w, b.origin, &closed, 2.0, *col, glow);
                }
            },
        )
        .absolute()
        .size_full();
        // Axis labels positioned just outside the web.
        let mut web = div().relative().w(px(size)).h(px(size)).child(cvs);
        for (i, a) in self.axes.iter().enumerate() {
            let (x, y) = polar(0.5, 0.5, 0.5, i as f32 / n as f32);
            web = web.child(
                label_style(&t, div())
                    .absolute()
                    .left(relative(x))
                    .top(relative(y))
                    .child(a.clone()),
            );
        }
        let mut root = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.sm))
            .p(px(24.0))
            .child(web);
        if self.legend && self.series.len() > 1 {
            root = root.child(Legend::of_series(&t, &self.series));
        }
        root
    }
}

// ─────────────────────────────── ScatterPlot ───────────────────────────────

/// Scatter plot of `(x, y)` points per series, with hover tooltips.
#[derive(IntoElement)]
pub struct ScatterPlot {
    id: SharedString,
    series: Vec<(SharedString, Vec<(f32, f32)>)>,
    height: f32,
    legend: bool,
}

impl ScatterPlot {
    pub fn new<S: Into<SharedString>>(
        id: impl Into<SharedString>,
        series: impl IntoIterator<Item = (S, Vec<(f32, f32)>)>,
    ) -> Self {
        Self {
            id: id.into(),
            series: series.into_iter().map(|(n, p)| (n.into(), p)).collect(),
            height: 200.0,
            legend: true,
        }
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

impl RenderOnce for ScatterPlot {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let all = || self.series.iter().flat_map(|s| s.1.iter());
        let fold = |f: fn(&(f32, f32)) -> f32| {
            all()
                .map(f)
                .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(v), h.max(v)))
        };
        let (xl, xh) = fold(|p| p.0);
        let (yl, yh) = fold(|p| p.1);
        let xt = nice_ticks(xl.min(xh), xh.max(xl), 4);
        let yt = nice_ticks(yl.min(yh), yh.max(yl), 4);
        let (x0, x1) = (*xt.first().unwrap(), *xt.last().unwrap());
        let (y0, y1) = (*yt.first().unwrap(), *yt.last().unwrap());
        let grid = hsla(t.palette.border);
        let yf: Vec<f32> = yt.iter().map(|v| norm(*v, y0, y1)).collect();
        let xf: Vec<f32> = xt.iter().map(|v| norm(*v, x0, x1)).collect();
        let cvs = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                for f in &yf {
                    stroke(
                        w,
                        b.origin,
                        &[(0.0, bh - f * bh), (bw, bh - f * bh)],
                        1.0,
                        grid,
                        false,
                    );
                }
                for f in &xf {
                    stroke(
                        w,
                        b.origin,
                        &[(f * bw, 0.0), (f * bw, bh)],
                        1.0,
                        grid,
                        false,
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let mut plot = div().relative().w_full().h(px(self.height)).child(cvs);
        for (si, (name, pts)) in self.series.iter().enumerate() {
            let c = hsla(t.palette.series_at(si));
            for (i, (x, y)) in pts.iter().enumerate() {
                plot = plot.child(
                    div()
                        .id((SharedString::from(format!("{}-{si}-{i}", self.id)), i))
                        .absolute()
                        .left(relative(norm(*x, x0, x1)))
                        .top(relative(1.0 - norm(*y, y0, y1)))
                        .ml(px(-4.0))
                        .mt(px(-4.0))
                        .w(px(8.0))
                        .h(px(8.0))
                        .rounded_full()
                        .bg(scale_alpha(c, 0.85))
                        .hover(move |d| d.bg(c))
                        .tooltip(DodaiTooltip::text(format!(
                            "{name}: ({}, {})",
                            fmt_num(*x),
                            fmt_num(*y)
                        ))),
                );
            }
        }
        let xl: Vec<SharedString> = xt.iter().map(|v| fmt_num(*v).into()).collect();
        let mut root = div().flex().flex_col().gap(px(t.spacing.sm)).w_full();
        root = root.child(framed(
            &t,
            yt.iter().map(|v| fmt_num(*v)).collect(),
            xl,
            plot,
        ));
        if self.legend && self.series.len() > 1 {
            root = root.child(Legend::new(
                self.series
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (s.0.clone(), t.palette.series_at(i))),
            ));
        }
        root
    }
}

// ─────────────────────────────── Heatmap ───────────────────────────────

/// Colour of heat `v` (`0..=1`): surface → primary. Pure.
pub fn heat_color(t: &Theme, v: f32) -> Color {
    t.palette
        .surface_raised
        .mix(t.palette.primary, v.clamp(0.0, 1.0))
}

/// Grid of cells coloured by value, with hover tooltips.
#[derive(IntoElement)]
pub struct Heatmap {
    id: SharedString,
    rows: Vec<Vec<f32>>,
    row_labels: Vec<SharedString>,
    col_labels: Vec<SharedString>,
    cell: f32,
}

impl Heatmap {
    pub fn new(id: impl Into<SharedString>, rows: Vec<Vec<f32>>) -> Self {
        Self {
            id: id.into(),
            rows,
            row_labels: Vec::new(),
            col_labels: Vec::new(),
            cell: 28.0,
        }
    }
    pub fn row_labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.row_labels = l.into_iter().map(Into::into).collect();
        self
    }
    pub fn col_labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.col_labels = l.into_iter().map(Into::into).collect();
        self
    }
    pub fn cell(mut self, s: f32) -> Self {
        self.cell = s;
        self
    }
}

impl RenderOnce for Heatmap {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (lo, hi) = self
            .rows
            .iter()
            .flatten()
            .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        let cell = self.cell;
        let mut root = div().flex().flex_col().gap(px(2.0));
        if !self.col_labels.is_empty() {
            let mut hdr = label_style(&t, div()).flex().gap(px(2.0));
            if !self.row_labels.is_empty() {
                hdr = hdr.child(div().w(px(48.0)));
            }
            root = root.child(
                hdr.children(
                    self.col_labels
                        .iter()
                        .map(|l| div().w(px(cell)).flex().justify_center().child(l.clone())),
                ),
            );
        }
        for (r, row) in self.rows.iter().enumerate() {
            let mut line = div().flex().gap(px(2.0)).items_center();
            if let Some(l) = self.row_labels.get(r) {
                line = line.child(label_style(&t, div()).w(px(48.0)).child(l.clone()));
            } else if !self.row_labels.is_empty() {
                line = line.child(div().w(px(48.0)));
            }
            for (c, v) in row.iter().enumerate() {
                let color = heat_color(&t, norm(*v, lo, hi));
                let style = ShapeStyle::from_tokens(&t, Some(color), None, false, false);
                line = line.child(
                    div()
                        .id((SharedString::from(format!("{}-{r}", self.id)), c))
                        .w(px(cell))
                        .h(px(cell))
                        .tooltip(DodaiTooltip::text(fmt_num(*v)))
                        .child(shaped(&t.shape, style).size_full()),
                );
            }
            root = root.child(line);
        }
        root
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn step_points_stairs() {
        assert_eq!(
            step_points(&[(0.0, 1.0), (2.0, 3.0)]),
            vec![(0.0, 1.0), (2.0, 1.0), (2.0, 3.0)]
        );
    }

    use super::*;

    #[test]
    fn ticks_cover_range_and_are_nice() {
        let t = nice_ticks(0.0, 87.0, 4);
        assert_eq!(t, vec![0.0, 50.0, 100.0]);
        let t = nice_ticks(0.0, 1.0, 4);
        assert_eq!(*t.last().unwrap(), 1.0);
        assert!(nice_ticks(5.0, 5.0, 4).len() >= 2);
    }

    #[test]
    fn number_format() {
        assert_eq!(fmt_num(3.0), "3");
        assert_eq!(fmt_num(2.5), "2.5");
        assert_eq!(fmt_num(1200.0), "1.2k");
        assert_eq!(fmt_num(2_000_000.0), "2M");
    }

    #[test]
    fn smoothing_hits_endpoints() {
        let p = [(0.0, 0.0), (1.0, 2.0), (2.0, 0.0)];
        let s = smooth_points(&p, 4);
        assert_eq!(s[0], p[0]);
        assert_eq!(*s.last().unwrap(), p[2]);
        assert_eq!(s.len(), 2 * 4 + 1);
        assert_eq!(smooth_points(&p[..2], 4).len(), 2);
    }

    #[test]
    fn pie_fractions_sum_to_one() {
        let f = pie_fractions(&[1.0, 1.0, 2.0]);
        assert_eq!(f[0], (0.0, 0.25));
        assert_eq!(f[2], (0.5, 1.0));
        assert!(pie_fractions(&[0.0, 0.0]).iter().all(|(a, b)| a == b));
        assert_eq!(pie_fractions(&[-1.0, 1.0])[1].1, 1.0);
    }

    #[test]
    fn polar_starts_at_top_clockwise() {
        let (x, y) = polar(10.0, 10.0, 5.0, 0.0);
        assert!((x - 10.0).abs() < 1e-4 && (y - 5.0).abs() < 1e-4);
        let (x, _) = polar(10.0, 10.0, 5.0, 0.25);
        assert!((x - 15.0).abs() < 1e-4);
    }

    #[test]
    fn bar_max_stacked_vs_grouped() {
        let s = [Series::new("a", [1.0, 4.0]), Series::new("b", [3.0, 1.0])];
        assert_eq!(bar_max(&s, true), 5.0);
        assert_eq!(bar_max(&s, false), 4.0);
    }

    #[test]
    fn norm_clamps() {
        assert_eq!(norm(5.0, 0.0, 10.0), 0.5);
        assert_eq!(norm(20.0, 0.0, 10.0), 1.0);
        assert_eq!(norm(1.0, 3.0, 3.0), 0.0);
    }
}
