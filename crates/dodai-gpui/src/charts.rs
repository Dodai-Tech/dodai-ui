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
    ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ───────────────────────────── pure helpers ─────────────────────────────

pub(crate) use dodai_charts_core::trim;
pub use dodai_charts_core::{
    fmt_num, marker_points, nice_ticks, norm, pie_fractions, polar, smooth_points, step_points,
    turn_arc, Marker,
};

pub(crate) use crate::paint::{fill, fill_batch, stroke, stroke_batch};

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

/// NaN / ±inf become 0: hostile data must never reach the tessellator.
pub(crate) fn finite_or_zero(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

impl Series {
    pub fn new(name: impl Into<SharedString>, values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            name: name.into(),
            values: values.into_iter().map(finite_or_zero).collect(),
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
        let motion = crate::motion::current(cx);
        crate::motion::entry(self.build(cx), "Legend", motion)
    }
}

impl Legend {
    fn build(self, cx: &mut App) -> impl IntoElement {
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
                    .child(
                        div()
                            .w(px(t.sizes().legend_swatch))
                            .h(px(t.sizes().legend_swatch))
                            .bg(hsla(c)),
                    )
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
        .w(px(t.sizes().axis_gutter))
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
    secondary: Vec<Series>,
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
            secondary: Vec::new(),
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
    /// Series drawn against a second y axis on the right; its gridlines align
    /// with the primary axis.
    pub fn secondary(mut self, s: impl IntoIterator<Item = Series>) -> Self {
        self.secondary = s.into_iter().collect();
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

impl LineChart {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "LineChart");
        let (lo, hi, ticks) = y_range(&self.series, self.zero, self.y_bounds);
        let np = self.series.len();
        let (lo2, hi2, ticks2) = if self.secondary.is_empty() {
            (lo, hi, Vec::new())
        } else {
            let (a, b, _) = y_range(&self.secondary, self.zero, None);
            let (_, t2) = dodai_charts_core::axis::aligned_ticks((lo, hi), (a, b), 4);
            (*t2.first().unwrap_or(&a), *t2.last().unwrap_or(&b), t2)
        };
        let all: Vec<Series> = self
            .series
            .iter()
            .chain(self.secondary.iter())
            .cloned()
            .collect();
        let range_of = move |i: usize| if i < np { (lo, hi) } else { (lo2, hi2) };
        let n = all.iter().map(|s| s.values.len()).max().unwrap_or(0);
        let grid = hsla(t.palette.border);
        let glow = glow_of(&t);
        let (area, smooth, stepped, sw, show_grid) = (
            self.area,
            self.smooth,
            self.stepped,
            self.stroke_w,
            self.grid,
        );
        let lines: Vec<(Hsla, Vec<f32>, (f32, f32))> = all
            .iter()
            .enumerate()
            .map(|(i, s)| (hsla(series_color(&t, s, i)), s.values.clone(), range_of(i)))
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
                for (c, vals, (lo, hi)) in &lines {
                    let (lo, hi) = (*lo, *hi);
                    let m = vals.len();
                    let raw: Vec<(f32, f32)> = vals
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            // Centre of category slot `i`, matching the x labels.
                            let x = (i as f32 + 0.5) / m as f32 * bw;
                            (x, bh - norm(*v, lo, hi) * bh * g)
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
            for (si, s) in all.iter().enumerate() {
                let (lo, hi) = range_of(si);
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
                            .ml(px(-t.sizes().legend_swatch / 2.0))
                            .mt(px(-t.sizes().legend_swatch / 2.0))
                            .w(px(t.sizes().legend_swatch))
                            .h(px(t.sizes().legend_swatch))
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
        let left = framed(
            &t,
            ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
            x_labels,
            plot,
        );
        if ticks2.is_empty() {
            root = root.child(left);
        } else {
            let right = label_style(&t, div())
                .w(px(t.sizes().axis_gutter))
                .h(px(self.height))
                .flex()
                .flex_col()
                .justify_between()
                .pl(px(t.spacing.xs))
                .children(ticks2.iter().rev().map(|v| div().child(fmt_num(*v))));
            root = root.child(div().flex().w_full().child(left.flex_1()).child(right));
        }
        if self.legend && all.len() > 1 {
            root = root.child(Legend::of_series(&t, &all));
        }
        titled(&t, self.title.clone(), root)
    }
}

crate::entry_render!(LineChart);

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
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(BarChart);

impl BarChart {
    pub fn new(id: impl Into<SharedString>, series: impl IntoIterator<Item = Series>) -> Self {
        Self {
            corners: None,
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

impl BarChart {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let g = crate::motion::grow(window, cx, "BarChart");
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

        let bar_top = top / g.max(1e-3);
        let mut cats = div().flex().size_full();
        cats = if h { cats.flex_col() } else { cats };
        for i in 0..n {
            let mut cell = div()
                .flex_1()
                .size_full()
                .flex()
                .gap(px(t.spacing.xs / 2.0))
                .p(px(t.spacing.xs));
            if h {
                // Bars grow right; stacked bars lay segments left→right.
                cell = cell.items_center();
                if self.stacked {
                    let mut stack = div().h(px(t.sizes().chart_bar)).flex();
                    for (si, s) in self.series.iter().enumerate() {
                        let v = s.values.get(i).copied().unwrap_or(0.0).max(0.0);
                        stack = stack.child(bar(si, i, v, v / bar_top.max(f32::EPSILON)));
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
                        cell = cell.child(div().w_full().h(px(t.sizes().chart_cell)).child(bar(
                            si,
                            i,
                            v,
                            norm(v, 0.0, bar_top),
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
                        .h(relative(norm(total, 0.0, bar_top)))
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
                                        .h(relative(norm(v, 0.0, bar_top)))
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
                line.left(relative(*f))
                    .top_0()
                    .h_full()
                    .w(px(t.sizes().hairline))
            } else {
                line.bottom(relative(*f))
                    .left_0()
                    .w_full()
                    .h(px(t.sizes().hairline))
            });
        }
        plot = plot.child(cats);

        let labels: Vec<SharedString> = (0..n)
            .map(|i| self.labels.get(i).cloned().unwrap_or_default())
            .collect();
        let body = if h {
            let ycol = label_style(&t, div())
                .w(px(t.sizes().axis_gutter_lg))
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
                .ml(px(t.sizes().axis_gutter_lg))
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

crate::entry_render!(BarChart);

// ─────────────────────────────── Sparkline ───────────────────────────────

/// Tiny inline line, no axes.
#[derive(IntoElement)]
pub struct Sparkline {
    style: SparkStyle,
    baseline: bool,
    values: Vec<f32>,
    w: f32,
    fluid: bool,
    h: f32,
    color: Option<Color>,
    area: bool,
    marker: Marker,
    all_points: bool,
}

/// How a [`Sparkline`] draws its values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SparkStyle {
    /// Polyline (default).
    #[default]
    Line,
    /// Staircase: the value holds until the next sample.
    Step,
    /// One column per value.
    Bars,
    /// Points only, no connecting line.
    Dots,
    /// Up / down ticks around the mean: above it wins, below it loses.
    WinLoss,
}

/// Column rectangles `(x0, y0, x1, y1)` for [`SparkStyle::Bars`] /
/// [`SparkStyle::WinLoss`] in a `w`×`h` box (y down). Pure.
pub fn spark_columns(vals: &[f32], w: f32, h: f32, win_loss: bool) -> Vec<(f32, f32, f32, f32)> {
    let n = vals.len();
    if n == 0 {
        return Vec::new();
    }
    let gap = if n > 1 {
        (w / n as f32 * 0.25).min(3.0)
    } else {
        0.0
    };
    let bw = ((w - gap * (n - 1) as f32) / n as f32).max(1.0);
    let (lo, hi) = vals
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, hh), v| (l.min(*v), hh.max(*v)));
    let mean = vals.iter().sum::<f32>() / n as f32;
    vals.iter()
        .enumerate()
        .map(|(i, v)| {
            let x0 = i as f32 * (bw + gap);
            if win_loss {
                let half = h / 2.0;
                if *v >= mean {
                    (x0, 1.0, x0 + bw, half - 0.5)
                } else {
                    (x0, half + 0.5, x0 + bw, h - 1.0)
                }
            } else {
                let frac = 0.12 + 0.88 * norm(*v, lo, hi);
                (x0, h - frac * h, x0 + bw, h)
            }
        })
        .collect()
}

impl Sparkline {
    /// Drawing style (default [`SparkStyle::Line`]).
    pub fn style(mut self, s: SparkStyle) -> Self {
        self.style = s;
        self
    }
    /// Faint dashed reference line at the mean.
    pub fn baseline(mut self, b: bool) -> Self {
        self.baseline = b;
        self
    }
    /// Stretch to the available width instead of the fixed `w`.
    pub fn fluid(mut self, f: bool) -> Self {
        self.fluid = f;
        self
    }
}

impl Sparkline {
    pub fn new(values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            style: SparkStyle::Line,
            baseline: false,
            values: values.into_iter().collect(),
            w: 96.0,
            fluid: false,
            h: 28.0,
            color: None,
            area: true,
            marker: Marker::Dot,
            all_points: false,
        }
    }
    /// Shape at the line's tip (default `Dot`; `Marker::None` to hide).
    pub fn marker(mut self, m: Marker) -> Self {
        self.marker = m;
        self
    }
    /// Draw the marker on every point instead of only the last.
    pub fn all_points(mut self, a: bool) -> Self {
        self.all_points = a;
        self
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
        let motion = crate::motion::current(cx);
        crate::motion::entry(self.build(cx), "Sparkline", motion)
    }
}

impl Sparkline {
    fn build(self, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let c = hsla(self.color.unwrap_or(t.palette.primary));
        let glow = glow_of(&t).map(|r| r.min(4.0));
        let (vals, area, marker, all_points) =
            (self.values, self.area, self.marker, self.all_points);
        let (style, baseline) = (self.style, self.baseline);
        let up = hsla(t.palette.success);
        let down = hsla(t.palette.danger);
        let (lo, hi) = vals
            .iter()
            .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        let fluid = self.fluid;
        let base = div();
        let base = if fluid {
            base.w_full()
        } else {
            base.w(px(self.w))
        };
        base.h(px(self.h)).child(
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
                    if baseline {
                        let mean = vals.iter().sum::<f32>() / m as f32;
                        let y = pad + (1.0 - norm(mean, lo, hi)) * (bh - 2.0 * pad);
                        let mut x = 0.0;
                        while x < bw {
                            stroke(
                                w,
                                b.origin,
                                &[(x, y), ((x + 3.0).min(bw), y)],
                                1.0,
                                scale_alpha(c, 0.4),
                                false,
                            );
                            x += 6.0;
                        }
                    }
                    match style {
                        SparkStyle::Bars | SparkStyle::WinLoss => {
                            let win = style == SparkStyle::WinLoss;
                            let mean = vals.iter().sum::<f32>() / m as f32;
                            for (i, (x0, y0, x1, y1)) in
                                spark_columns(&vals, bw, bh, win).into_iter().enumerate()
                            {
                                let col = if win && vals[i] < mean {
                                    down
                                } else if win {
                                    up
                                } else {
                                    c
                                };
                                fill(w, b.origin, &[(x0, y0), (x1, y0), (x1, y1), (x0, y1)], col);
                            }
                            return;
                        }
                        SparkStyle::Dots => {
                            for &(x, y) in &p {
                                let x = x.clamp(3.0, bw - 3.0);
                                fill(w, b.origin, &marker_points(marker, x, y, 2.6), c);
                            }
                            return;
                        }
                        SparkStyle::Step => {
                            let sp = step_points(&p);
                            if area {
                                let mut poly = sp.clone();
                                poly.push((bw, bh));
                                poly.push((0.0, bh));
                                fill(w, b.origin, &poly, scale_alpha(c, 0.16));
                            }
                            glow_stroke(w, b.origin, &sp, 1.5, c, glow);
                            return;
                        }
                        SparkStyle::Line => {}
                    }
                    if area {
                        let mut poly = p.clone();
                        poly.push((bw, bh));
                        poly.push((0.0, bh));
                        fill(w, b.origin, &poly, scale_alpha(c, 0.16));
                    }
                    glow_stroke(w, b.origin, &p, 1.5, c, glow);
                    let tips = if all_points { &p[..] } else { &p[m - 1..] };
                    for &(x, y) in tips {
                        let x = x.clamp(3.0, bw - 3.0);
                        fill(w, b.origin, &marker_points(marker, x, y, 2.6), c);
                    }
                },
            )
            .size_full(),
        )
    }
}

// ───────────────────────────── PieChart / donut ─────────────────────────────

type SelectFn = Rc<dyn Fn(Option<usize>, &mut Window, &mut App)>;

/// How far (px) a hovered pie slice pops out.
const PIE_POP: f32 = 6.0;

#[derive(Default)]
struct PieState {
    bounds: Bounds<Pixels>,
    hover: Option<usize>,
    sel: Option<usize>,
    pops: Vec<crate::motion::Tween>,
}

/// Slice index under a pointer `(dx, dy)` from the pie centre (y down), or
/// `None` outside the ring / disc. `fr` are `(start, end)` turn fractions,
/// 0 at the top, clockwise. Pure.
pub fn pie_hit(dx: f32, dy: f32, r: f32, hole: f32, fr: &[(f32, f32)]) -> Option<usize> {
    let d = (dx * dx + dy * dy).sqrt();
    if d > r + PIE_POP || d < r * hole {
        return None;
    }
    let turn = (dx.atan2(-dy) / std::f32::consts::TAU).rem_euclid(1.0);
    // Fractions may run past 1 (a partial donut starting at 0.75).
    fr.iter()
        .position(|(a, z)| (turn >= *a && turn < *z) || (turn + 1.0 >= *a && turn + 1.0 < *z))
}

/// How a [`PieChart`] draws its slices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PieStyle {
    /// Solid wedges / rings (default).
    #[default]
    Solid,
    /// Ring built from separate cells, like a segmented readout.
    Segmented,
    /// Thin glowing band between an outer tick scale and a dashed inner ring.
    Hud,
    /// Radius follows the value (a rose / polar-area chart) over guide rings.
    Rose,
}

/// Pie or donut chart; `center` is shown in the donut hole.
#[derive(IntoElement)]
pub struct PieChart {
    style: PieStyle,
    slices: Vec<(SharedString, f32)>,
    size: f32,
    hole: f32,
    center: Option<SharedString>,
    legend: bool,
    selected: Option<Option<usize>>,
    select_color: Option<Color>,
    on_select: Option<SelectFn>,
    gap: f32,
    explode: f32,
    colors: Option<Vec<Color>>,
    start: f32,
    sweep: f32,
}

impl PieChart {
    pub fn new<S: Into<SharedString>>(slices: impl IntoIterator<Item = (S, f32)>) -> Self {
        Self {
            style: PieStyle::Solid,
            slices: slices.into_iter().map(|(n, v)| (n.into(), v)).collect(),
            size: 160.0,
            hole: 0.0,
            center: None,
            legend: true,
            selected: None,
            select_color: None,
            on_select: None,
            gap: 2.0,
            explode: 0.0,
            colors: None,
            start: 0.0,
            sweep: 1.0,
        }
    }
    /// Look of the slices (default [`PieStyle::Solid`]). `Segmented` and `Hud`
    /// are rings: they use at least a 0.55 / 0.76 hole.
    pub fn style(mut self, s: PieStyle) -> Self {
        self.style = s;
        self
    }
    /// Straight gap between slices in px (default 2, `0` = flush).
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g.max(0.0);
        self
    }
    /// Push every slice outward by `px` (an exploded pie), up to 10.
    pub fn explode(mut self, px: f32) -> Self {
        self.explode = px.clamp(0.0, 10.0);
        self
    }
    /// Slice colours (cycled) instead of the theme's series colours, e.g. a
    /// monochrome ramp.
    pub fn colors(mut self, c: impl IntoIterator<Item = Color>) -> Self {
        self.colors = Some(c.into_iter().collect());
        self
    }
    /// Draw only `sweep` of a full turn (`0.5` = half donut), beginning at
    /// `start` turns clockwise from the top (`0.75` = from the left).
    pub fn arc(mut self, start: f32, sweep: f32) -> Self {
        self.start = start;
        self.sweep = sweep.clamp(0.05, 1.0);
        self
    }
    /// Controlled selection (`None` = nothing selected). Without this the
    /// chart keeps its own selection, toggled by clicking a slice.
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = Some(i);
        self
    }
    /// Outline colour of the selected slice (default: the theme accent).
    pub fn selected_color(mut self, c: Color) -> Self {
        self.select_color = Some(c);
        self
    }
    crate::callback!(
        /// Called with the new selection (`None` when deselected) on click.
        on_select, Option<usize>
    );
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

impl PieChart {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let vals: Vec<f32> = self.slices.iter().map(|s| s.1).collect();
        let (start, sweep) = (self.start, self.sweep);
        let fr: Vec<(f32, f32)> = pie_fractions(&vals)
            .iter()
            .map(|(a, z)| (start + a * sweep, start + z * sweep))
            .collect();
        // Pies that differ in look must not share state: their pop targets
        // would fight and the tween would never settle (a redraw every frame).
        let key: SharedString = format!(
            "pie-{}-{:?}-{}-{}-{}-{}-{}",
            self.slices
                .iter()
                .map(|s| s.0.as_ref())
                .collect::<Vec<_>>()
                .join("|"),
            self.style,
            self.explode,
            self.hole,
            self.start,
            self.sweep,
            self.size,
        )
        .into();
        let st = window.use_keyed_state(key.clone(), cx, |_, _| PieState::default());
        let hover = st.read(cx).hover;
        let sel = self.selected.unwrap_or(st.read(cx).sel);
        let ring = hsla(self.select_color.unwrap_or(t.palette.primary));
        let motion = crate::motion::current(cx);
        let (st_paint, st_move, st_leave, st_click) =
            (st.clone(), st.clone(), st.clone(), st.clone());
        let (controlled, on_select) = (self.selected.is_some(), self.on_select.clone());
        let fr_click = fr.clone();
        let fr_hit = fr.clone();
        let g = crate::motion::grow(window, cx, "PieChart");
        let fr: Vec<(f32, f32)> = fr
            .iter()
            .map(|(a, z)| (start + (a - start) * g, start + (z - start) * g))
            .collect();
        let slice_col = {
            let (t2, custom) = (t.clone(), self.colors.clone());
            move |i: usize| match &custom {
                Some(c) if !c.is_empty() => c[i % c.len()],
                _ => t2.palette.series_at(i),
            }
        };
        let cols: Vec<Hsla> = (0..vals.len()).map(|i| hsla(slice_col(i))).collect();
        let style = self.style;
        let hole = match style {
            PieStyle::Segmented => self.hole.max(0.55),
            PieStyle::Hud => self.hole.max(0.76),
            _ => self.hole,
        };
        let size = self.size;
        let explode = self.explode;
        // Rounded (Material-like) themes get the calm version: no tick scale
        // or dashed ring, fewer and wider-spaced segments.
        let soft = t.look().soft_elevation;
        let hud = style == PieStyle::Hud && !soft;
        let reserve = PIE_POP + explode + if hud { 12.0 } else { 0.0 };
        let vmax = vals
            .iter()
            .cloned()
            .fold(0.0_f32, f32::max)
            .max(f32::EPSILON);
        let rfac: Vec<f32> = vals
            .iter()
            .map(|v| {
                if style == PieStyle::Rose {
                    0.38 + 0.62 * (v / vmax).clamp(0.0, 1.0)
                } else {
                    1.0
                }
            })
            .collect();
        let scale_col = hsla(t.palette.text_muted);
        let guide_col = hsla(t.palette.border);
        let glow = glow_of(&t);
        let gap = if vals.len() > 1 { self.gap } else { 0.0 }; // px
        let cvs = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, cx| {
                st_paint.update(cx, |s, _| s.bounds = b);
                let r = size / 2.0 - 1.0 - reserve;
                let c = size / 2.0;
                if hud {
                    // Outer tick scale + dashed inner reference ring.
                    let (mut minor, mut major) = (Vec::new(), Vec::new());
                    // Only decorate the part of the turn the arc covers.
                    let within = |u: f32| ((u - start).rem_euclid(1.0)) <= sweep + 0.001;
                    for k in 0..72 {
                        let u = k as f32 / 72.0;
                        if !within(u) {
                            continue;
                        }
                        let long = k % 6 == 0;
                        let seg = vec![
                            polar(c, c, r + 4.0, u),
                            polar(c, c, r + if long { 11.0 } else { 7.0 }, u),
                        ];
                        if long {
                            major.push(seg);
                        } else {
                            minor.push(seg);
                        }
                    }
                    stroke_batch(w, b.origin, &minor, 1.0, guide_col, false);
                    stroke_batch(w, b.origin, &major, 1.5, scale_col, false);
                    let dashes: Vec<Vec<(f32, f32)>> = (0..48)
                        .filter(|k| within(*k as f32 / 48.0))
                        .map(|k| {
                            let u = k as f32 / 48.0;
                            turn_arc(c, c, r * hole - 6.0, u, u + 0.01)
                        })
                        .collect();
                    stroke_batch(w, b.origin, &dashes, 1.0, scale_col, false);
                }
                if style == PieStyle::Rose {
                    for f in [0.38, 0.69, 1.0] {
                        stroke(
                            w,
                            b.origin,
                            &turn_arc(c, c, r * f, 0.0, 1.0),
                            1.0,
                            guide_col,
                            true,
                        );
                    }
                }
                for (i, ((a, z), col)) in fr.iter().zip(&cols).enumerate() {
                    let r = r * rfac[i];
                    // Hovered slice pops out along its bisector.
                    let target = explode
                        + if hover == Some(i) {
                            PIE_POP
                        } else if sel == Some(i) {
                            PIE_POP * 0.5
                        } else {
                            0.0
                        };
                    let (pop, busy) = st_paint.update(cx, |s, _| {
                        if s.pops.len() <= i {
                            s.pops.resize(i + 1, crate::motion::Tween::new(0.0, motion));
                        }
                        s.pops[i].to(target, motion, crate::motion::now());
                        let now = crate::motion::now();
                        (s.pops[i].value(now), !s.pops[i].is_done(now))
                    });
                    if busy {
                        crate::motion::next_frame(w);
                    }
                    let (dx, dy) = polar(0.0, 0.0, pop, (a + z) / 2.0);
                    let origin = point(b.origin.x + px(dx), b.origin.y + px(dy));
                    // Constant-width straight gap: each edge is offset by
                    // `gap / 2` from the true radial line, so the gap has
                    // parallel sides instead of widening towards the rim.
                    let half = gap / 2.0;
                    let (a, z) = (*a, *z);
                    let delta =
                        |rad: f32| (half / rad).clamp(0.0, 1.0).asin() / std::f32::consts::TAU;
                    let (ao, zo) = (a + delta(r), z - delta(r));
                    if zo <= ao {
                        continue;
                    }
                    if style == PieStyle::Segmented {
                        let r_in = r * hole;
                        let per_turn = if soft { 16.0 } else { 36.0 };
                        let n = (((zo - ao) * per_turn).ceil() as usize).max(1);
                        let g = if soft { 0.012 } else { 0.004 };
                        let mut cells = Vec::new();
                        for k in 0..n {
                            let c0 = ao + (zo - ao) * k as f32 / n as f32 + g / 2.0;
                            let c1 = ao + (zo - ao) * (k + 1) as f32 / n as f32 - g / 2.0;
                            if c1 <= c0 {
                                continue;
                            }
                            let mut p = turn_arc(c, c, r, c0, c1);
                            let mut inner = turn_arc(c, c, r_in, c0, c1);
                            inner.reverse();
                            p.extend(inner);
                            cells.push(p);
                        }
                        fill_batch(w, origin, &cells, *col);
                        if sel == Some(i) {
                            stroke(
                                w,
                                origin,
                                &turn_arc(c, c, r + 2.0, ao, zo),
                                2.0,
                                ring,
                                false,
                            );
                        }
                        continue;
                    }
                    if hole <= 0.0 {
                        let theta = (z - a) * std::f32::consts::TAU;
                        let d = if half > 0.0 {
                            half / (theta / 2.0).sin().max(0.05)
                        } else {
                            0.0
                        };
                        let (ax, ay) = polar(c, c, d, (a + z) / 2.0);
                        // Convex wedges (<= half a turn) tessellate without
                        // holes; fewer pieces means fewer AA seams.
                        let n = (((zo - ao) * 2.0).ceil() as usize).max(1);
                        for k in 0..n {
                            let w0 = ao + (zo - ao) * k as f32 / n as f32;
                            let w1 = ao + (zo - ao) * (k + 1) as f32 / n as f32;
                            let mut p =
                                turn_arc(c, c, r, w0, if k + 1 == n { w1 } else { w1 + 0.0015 });
                            p.push((ax, ay));
                            fill(w, origin, &p, *col);
                        }
                        if sel == Some(i) {
                            let mut o = turn_arc(c, c, r, ao, zo);
                            o.push((ax, ay));
                            stroke(w, origin, &o, 1.5, ring, true);
                        }
                        continue;
                    }
                    // Annular sectors as small filled quads: thick strokes
                    // leave mitre artefacts and seams, fills do not.
                    let r_in = r * hole;
                    let (ai, zi) = (a + delta(r_in), z - delta(r_in));
                    let n = (((zo - ao) * 24.0).ceil() as usize).max(1);
                    for k in 0..n {
                        let f0 = k as f32 / n as f32;
                        let f1 = (k + 1) as f32 / n as f32;
                        let e = if k + 1 == n { 0.0 } else { 0.001 };
                        let mut p = turn_arc(c, c, r, ao + (zo - ao) * f0, ao + (zo - ao) * f1 + e);
                        let mut inner =
                            turn_arc(c, c, r_in, ai + (zi - ai) * f0, ai + (zi - ai) * f1 + e);
                        inner.reverse();
                        p.extend(inner);
                        fill(w, origin, &p, *col);
                    }
                    if hud {
                        glow_stroke(w, origin, &turn_arc(c, c, r + 2.0, ao, zo), 1.5, *col, glow);
                    }
                    if sel == Some(i) {
                        let mut o = turn_arc(c, c, r, ao, zo);
                        let mut inner = turn_arc(c, c, r_in, ai, zi);
                        inner.reverse();
                        o.extend(inner);
                        stroke(w, origin, &o, 1.5, ring, true);
                    }
                }
            },
        )
        .absolute()
        .size_full();
        // A top half-ring (arc 0.75 + 0.5) only needs the upper half of the box.
        let top_half = self.sweep <= 0.5 && (self.start - 0.75).abs() < 0.01;
        let disc_h = if top_half { size / 2.0 + 2.0 } else { size };
        let mut disc = div()
            .id(key)
            .relative()
            .w(px(size))
            .h(px(disc_h))
            .overflow_hidden()
            .child(cvs)
            .on_mouse_move(move |e, w, cx| {
                let b = st_move.read(cx).bounds;
                let (ox, oy): (f32, f32) = (b.origin.x.into(), b.origin.y.into());
                let (mx, my): (f32, f32) = (e.position.x.into(), e.position.y.into());
                let h = pie_hit(
                    mx - ox - size / 2.0,
                    my - oy - size / 2.0,
                    size / 2.0 - 1.0 - reserve,
                    hole,
                    &fr_hit,
                );
                if st_move.read(cx).hover != h {
                    st_move.update(cx, |s, _| s.hover = h);
                    w.refresh();
                }
            })
            .on_mouse_down(gpui::MouseButton::Left, move |e, w, cx| {
                let b = st_click.read(cx).bounds;
                let (ox, oy): (f32, f32) = (b.origin.x.into(), b.origin.y.into());
                let (mx, my): (f32, f32) = (e.position.x.into(), e.position.y.into());
                let r = size / 2.0 - 1.0 - reserve;
                let h = pie_hit(
                    mx - ox - size / 2.0,
                    my - oy - size / 2.0,
                    r,
                    hole,
                    &fr_click,
                );
                let cur = if controlled {
                    sel
                } else {
                    st_click.read(cx).sel
                };
                let next = match h {
                    Some(i) if cur == Some(i) => None,
                    Some(i) => Some(i),
                    None => None,
                };
                if !controlled {
                    st_click.update(cx, |s, _| s.sel = next);
                    w.refresh();
                }
                if let Some(f) = &on_select {
                    f(next, w, cx);
                }
            })
            .on_hover(move |over, w, cx| {
                if !*over && st_leave.read(cx).hover.is_some() {
                    st_leave.update(cx, |s, _| s.hover = None);
                    w.refresh();
                }
            });
        if let (Some(txt), true) = (self.center.clone(), hole > 0.0) {
            let overlay = div().absolute().left_0().w(px(size));
            // Top half ring: centre the value on the baseline, not the full circle.
            let overlay = if top_half {
                overlay.bottom_0().h(px(size * 0.3))
            } else {
                overlay.top_0().h(px(size))
            };
            disc = disc.child(
                overlay
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
                                .w(px(t.sizes().legend_swatch))
                                .h(px(t.sizes().legend_swatch))
                                .bg(hsla(slice_col(i))),
                        )
                        .child(format!("{n} · {}%", trim(pct)))
                }),
            ));
        }
        root
    }
}

crate::entry_render!(PieChart);

// ─────────────────────────────────── Gauge ───────────────────────────────────

/// Radial gauge with a centre label. [`crate::gauges::GaugeStyle`] picks the look: the
/// default 270° arc, segmented ring, dial, meter, reticle or slanted-cell bar.
#[derive(IntoElement)]
pub struct Gauge {
    value: f32,
    min: f32,
    max: f32,
    size: f32,
    label: Option<SharedString>,
    caption: Option<SharedString>,
    tone: Option<dodai_core::Tone>,
    style: crate::gauges::GaugeStyle,
    zones: bool,
}

impl Gauge {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            min: 0.0,
            max: 100.0,
            size: 120.0,
            label: None,
            caption: None,
            tone: None,
            style: crate::gauges::GaugeStyle::Arc,
            zones: false,
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
    /// Small caption under the label (uppercased when the theme does).
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
    pub fn tone(mut self, t: dodai_core::Tone) -> Self {
        self.tone = Some(t);
        self
    }
    pub fn style(mut self, s: crate::gauges::GaugeStyle) -> Self {
        self.style = s;
        self
    }
    /// Colour the value green → amber → red as it climbs (overrides `tone`).
    pub fn zones(mut self, on: bool) -> Self {
        self.zones = on;
        self
    }
}

impl Gauge {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use crate::gauges::{GaugeStyle, Look};
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "Gauge");
        let frac = norm(self.value, self.min, self.max) * g;
        let col = hsla(self.tone.map_or(t.palette.primary, |k| t.palette.tone(k)));
        let track = hsla(t.palette.border);
        let glow = glow_of(&t);
        let size = self.size;
        let style = self.style;
        let (bw, bh) = style.dims(size);
        let ring = (size * 0.1).clamp(4.0, 12.0);
        let label = self
            .label
            .unwrap_or_else(|| fmt_num((self.value * g).round()).into());
        let look = Look {
            style,
            frac,
            size,
            ring,
            tone: col,
            track,
            text: hsla(t.palette.text),
            zones: self.zones.then(|| {
                [
                    hsla(t.palette.success),
                    hsla(t.palette.warning),
                    hsla(t.palette.danger),
                ]
            }),
            glow,
            // Cut-corner gauges are Cyberpunk's; every other design gets the round ones.
            soft: t.look().soft_elevation || t.shape.kind() != dodai_core::ShapeKind::Chamfered,
        };
        let soft = look.soft;
        let canvas_el = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                if style != GaugeStyle::Arc {
                    crate::gauges::paint(w, b, look);
                    return;
                }
                let c = size / 2.0;
                let r = c - ring / 2.0 - 2.0;
                let (s0, span) = crate::gauges::SWEEP;
                if soft {
                    crate::gauges::round_arc(w, b.origin, c, r, s0, s0 + span, ring, track);
                    if frac > 0.0 {
                        crate::gauges::round_arc(
                            w,
                            b.origin,
                            c,
                            r,
                            s0,
                            s0 + span * frac,
                            ring,
                            col,
                        );
                    }
                    return;
                }
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
        .size_full();
        let caption = self
            .caption
            .map(|c| crate::text::format_label(&t.typography, &c));
        let value_size = match style {
            GaugeStyle::Bar | GaugeStyle::Chevron => bh * 0.9,
            GaugeStyle::Dial => size * 0.17,
            GaugeStyle::Meter => size * 0.15,
            GaugeStyle::Level => size * 0.13,
            _ => size * 0.22,
        };
        let readout = div()
            .flex()
            .flex_col()
            .items_center()
            .font_family(first_family(t.typography.display))
            .text_size(px(value_size))
            .text_color(hsla(t.palette.text))
            .child(label)
            .children(caption.map(|c| {
                div()
                    .text_size(px((size * 0.085).max(9.0)))
                    .text_color(hsla(t.palette.text_muted))
                    .font_family(first_family(t.typography.mono))
                    .child(c)
            }));
        let frame = div().relative().w(px(bw)).h(px(bh)).child(canvas_el);
        match style {
            GaugeStyle::Bar | GaugeStyle::Chevron | GaugeStyle::Level => div()
                .flex()
                .items_center()
                .gap(px(t.spacing.md))
                .child(frame)
                .child(readout)
                .into_any_element(),
            GaugeStyle::Dial => frame
                .child(
                    div()
                        .absolute()
                        .w_full()
                        .top(px(size * 0.62))
                        .flex()
                        .justify_center()
                        .child(readout),
                )
                .into_any_element(),
            GaugeStyle::Meter => frame
                .child(
                    div()
                        .absolute()
                        .w_full()
                        .bottom_0()
                        .flex()
                        .justify_center()
                        .child(readout),
                )
                .into_any_element(),
            _ => frame
                .child(
                    div()
                        .absolute()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(readout),
                )
                .into_any_element(),
        }
    }
}

crate::entry_render!(Gauge);

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

impl RadarChart {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let g = crate::motion::grow(window, cx, "RadarChart");
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
                                r * norm(vals.get(i).copied().unwrap_or(0.0), 0.0, max) * g,
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
            .p(px(t.sizes().chart_pad))
            .child(web);
        if self.legend && self.series.len() > 1 {
            root = root.child(Legend::of_series(&t, &self.series));
        }
        root
    }
}

crate::entry_render!(RadarChart);

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

impl ScatterPlot {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let g = crate::motion::grow(window, cx, "ScatterPlot");
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
                let gi = crate::motion::reveal(g, i, pts.len());
                plot = plot.child(
                    div()
                        .id((SharedString::from(format!("{}-{si}-{i}", self.id)), i))
                        .absolute()
                        .left(relative(norm(*x, x0, x1)))
                        .top(relative(1.0 - norm(*y, y0, y1)))
                        .ml(px(-t.sizes().scatter_dot / 2.0 * gi))
                        .mt(px(-t.sizes().scatter_dot / 2.0 * gi))
                        .w(px(t.sizes().scatter_dot * gi))
                        .h(px(t.sizes().scatter_dot * gi))
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

crate::entry_render!(ScatterPlot);

// ─────────────────────────────── Heatmap ───────────────────────────────

/// Colour of heat `v` (`0..=1`): surface → primary. Pure.
pub fn heat_color(t: &Theme, v: f32) -> Color {
    t.palette
        .surface_raised
        .mix(t.palette.primary, v.clamp(0.0, 1.0))
}

/// Like [`heat_color`] but with a selectable gradient. Pure.
pub fn heat_color_with(t: &Theme, g: dodai_charts_core::Gradient, v: f32) -> Color {
    match g.sample(v) {
        Some([r, gr, b]) => Color::rgba(r, gr, b, 255),
        None => heat_color(t, v),
    }
}

/// Grid of cells coloured by value, with hover tooltips.
#[derive(IntoElement)]
pub struct Heatmap {
    id: SharedString,
    rows: Vec<Vec<f32>>,
    row_labels: Vec<SharedString>,
    col_labels: Vec<SharedString>,
    cell: f32,
    gradient: dodai_charts_core::Gradient,
}

impl Heatmap {
    pub fn new(id: impl Into<SharedString>, rows: Vec<Vec<f32>>) -> Self {
        Self {
            id: id.into(),
            rows,
            row_labels: Vec::new(),
            col_labels: Vec::new(),
            cell: 28.0,
            gradient: dodai_charts_core::Gradient::Theme,
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
    /// Colour scale; `Gradient::Theme` (default) mixes surface → primary.
    pub fn gradient(mut self, g: dodai_charts_core::Gradient) -> Self {
        self.gradient = g;
        self
    }
}

impl Heatmap {
    /// Dense grid (many cells): painted on one canvas with a hover readout
    /// instead of one element per cell.
    fn build_dense(self, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
        let g = crate::motion::grow(window, cx, "Heatmap");
        let t = theme_of(cx);
        let (lo, hi) = self
            .rows
            .iter()
            .flatten()
            .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        let cell = self.cell;
        let (nr, nc) = (
            self.rows.len(),
            self.rows.iter().map(Vec::len).max().unwrap_or(0),
        );
        let hov = window.use_keyed_state(
            SharedString::from(format!("{}-hov", self.id)),
            cx,
            |_, _| None::<(usize, usize)>,
        );
        let cur = *hov.read(cx);
        let colors: Rc<Vec<gpui::Hsla>> = Rc::new(
            self.rows
                .iter()
                .flat_map(|r| {
                    r.iter()
                        .map(|v| hsla(heat_color_with(&t, self.gradient, norm(*v, lo, hi))))
                })
                .collect(),
        );
        let lens: Rc<Vec<usize>> = Rc::new(self.rows.iter().map(Vec::len).collect());
        let outline = hsla(t.palette.text);
        let hov_w = hov.clone();
        let canvas = gpui::canvas(
            |_, _, _| (),
            move |b: gpui::Bounds<gpui::Pixels>, _, w, _| {
                let cw = f32::from(b.size.width) / nc.max(1) as f32;
                let mut i = 0;
                for r in 0..nr {
                    for c in 0..lens[r] {
                        let o = b.origin + gpui::point(px(c as f32 * cw), px(r as f32 * cell));
                        w.paint_quad(gpui::fill(
                            gpui::Bounds::new(o, gpui::size(px(cw), px(cell))),
                            colors[i].opacity(crate::motion::reveal(g, c, nc)),
                        ));
                        i += 1;
                    }
                }
                if let Some((r, c)) = cur {
                    let o = b.origin + gpui::point(px(c as f32 * cw), px(r as f32 * cell));
                    w.paint_quad(gpui::outline(
                        gpui::Bounds::new(o, gpui::size(px(cw), px(cell))),
                        outline,
                        gpui::BorderStyle::Solid,
                    ));
                }
                let st = hov_w.clone();
                w.on_mouse_event(move |e: &gpui::MouseMoveEvent, _, w, cx| {
                    let cw = f32::from(b.size.width) / nc.max(1) as f32;
                    let next = b.contains(&e.position).then(|| {
                        let d = e.position - b.origin;
                        (
                            ((f32::from(d.y) / cell) as usize).min(nr.saturating_sub(1)),
                            ((f32::from(d.x) / cw) as usize).min(nc.saturating_sub(1)),
                        )
                    });
                    if *st.read(cx) != next {
                        st.update(cx, |v, _| *v = next);
                        w.refresh();
                    }
                });
            },
        )
        .flex_1()
        .min_w_0()
        .h(px(nr as f32 * cell));
        let readout = match cur {
            Some((r, c)) => format!(
                "{} · {} = {}",
                self.row_labels
                    .get(r)
                    .filter(|l| !l.is_empty())
                    .map_or(format!("row {r}"), |l| l.to_string()),
                self.col_labels
                    .get(c)
                    .filter(|l| !l.is_empty())
                    .map_or(format!("col {c}"), |l| l.to_string()),
                fmt_num(self.rows[r][c])
            ),
            None => "hover a cell".to_string(),
        };
        let mut cols = div().flex().flex_col();
        if !self.col_labels.is_empty() {
            let mut hdr = label_style(&t, div()).flex().h(px(t.sizes().chart_cell));
            if !self.row_labels.is_empty() {
                hdr = hdr.child(div().w(px(t.sizes().axis_label)).flex_none());
            }
            cols = cols.child(hdr.children(self.col_labels.iter().map(|l| {
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .child(l.clone())
            })));
        }
        let mut body = div().flex();
        if !self.row_labels.is_empty() {
            body = body.child(
                label_style(&t, div())
                    .w(px(t.sizes().axis_label))
                    .flex_none()
                    .children(
                        self.row_labels
                            .iter()
                            .map(|l| div().h(px(cell)).whitespace_nowrap().child(l.clone())),
                    ),
            );
        }
        cols.child(body.child(canvas))
            .child(
                label_style(&t, div())
                    .pt(px(t.spacing.sm - t.spacing.xs / 2.0))
                    .child(readout),
            )
            .into_any_element()
    }

    fn build(self, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
        let cells: usize = self.rows.iter().map(Vec::len).sum();
        if cells > 200 {
            return self.build_dense(window, cx);
        }
        self.build_grid(cx).into_any_element()
    }

    fn build_grid(self, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (lo, hi) = self
            .rows
            .iter()
            .flatten()
            .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        let cell = self.cell;
        let mut root = div().flex().flex_col().gap(px(t.spacing.xs / 2.0));
        if !self.col_labels.is_empty() {
            let mut hdr = label_style(&t, div()).flex().gap(px(t.spacing.xs / 2.0));
            if !self.row_labels.is_empty() {
                hdr = hdr.child(div().w(px(t.sizes().axis_label)));
            }
            root = root.child(hdr.children(self.col_labels.iter().map(|l| {
                div()
                    .flex_1()
                    .min_w_0()
                    .max_w(px(cell * 4.0))
                    .flex()
                    .justify_center()
                    .child(l.clone())
            })));
        }
        for (r, row) in self.rows.iter().enumerate() {
            let mut line = div().flex().gap(px(t.spacing.xs / 2.0)).items_center();
            if let Some(l) = self.row_labels.get(r) {
                line = line.child(
                    label_style(&t, div())
                        .w(px(t.sizes().axis_label))
                        .child(l.clone()),
                );
            } else if !self.row_labels.is_empty() {
                line = line.child(div().w(px(t.sizes().axis_label)));
            }
            for (c, v) in row.iter().enumerate() {
                let color = heat_color_with(&t, self.gradient, norm(*v, lo, hi));
                let style = ShapeStyle::from_tokens(&t, Some(color), None, false, false);
                line = line.child(
                    div()
                        .id((SharedString::from(format!("{}-{r}", self.id)), c))
                        .flex_1()
                        .min_w_0()
                        .max_w(px(cell * 4.0))
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

crate::entry_render!(Heatmap);

#[cfg(test)]
mod tests {
    #[test]
    fn spark_columns_shapes() {
        let v = [1.0, 5.0, 3.0, 9.0];
        let bars = spark_columns(&v, 40.0, 20.0, false);
        assert_eq!(bars.len(), 4);
        // Tallest value reaches the top; the lowest keeps a visible stub.
        assert!(bars[3].1 < 0.5 && bars[0].3 - bars[0].1 > 2.0);
        assert!(
            bars.windows(2).all(|w| w[0].2 <= w[1].0 + 0.01),
            "no overlap"
        );
        let wl = spark_columns(&v, 40.0, 20.0, true);
        // mean 4.5: values 1,3 lose (below the midline), 5,9 win (above).
        assert!(wl[0].1 > 10.0 && wl[2].1 > 10.0);
        assert!(wl[1].3 < 10.0 && wl[3].3 < 10.0);
        assert!(spark_columns(&[], 10.0, 10.0, false).is_empty());
    }

    #[test]
    fn pie_hit_wraps_past_one_turn() {
        // Half donut starting at the left (0.75) running over the top to 1.25.
        let fr = [(0.75, 1.0), (1.0, 1.25)];
        assert_eq!(pie_hit(0.0, -30.0, 50.0, 0.4, &fr), Some(1)); // just right of top
        assert_eq!(pie_hit(-30.0, -1.0, 50.0, 0.4, &fr), Some(0)); // upper left
        assert_eq!(pie_hit(0.0, 30.0, 50.0, 0.4, &fr), None); // empty lower half
    }

    #[test]
    fn pie_hit_finds_slice() {
        let fr = [(0.0, 0.5), (0.5, 1.0)];
        assert_eq!(pie_hit(10.0, 0.0, 50.0, 0.0, &fr), Some(0)); // right = 25%
        assert_eq!(pie_hit(-10.0, 0.0, 50.0, 0.0, &fr), Some(1));
        assert_eq!(pie_hit(0.0, 0.0, 50.0, 0.4, &fr), None); // donut hole
        assert_eq!(pie_hit(200.0, 0.0, 50.0, 0.0, &fr), None);
    }

    use super::*;

    #[test]
    fn bar_max_stacked_vs_grouped() {
        let s = [Series::new("a", [1.0, 4.0]), Series::new("b", [3.0, 1.0])];
        assert_eq!(bar_max(&s, true), 5.0);
        assert_eq!(bar_max(&s, false), 4.0);
    }
}
