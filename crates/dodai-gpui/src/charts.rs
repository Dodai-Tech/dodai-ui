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
    canvas, div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

pub use crate::charts_grid::*;
pub use crate::charts_pie::*;
pub use crate::charts_radial::*;

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
    /// Series name (legend entry).
    pub name: SharedString,
    /// One value per category or x position.
    pub values: Vec<f32>,
    /// Explicit colour; `None` takes the next palette colour.
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
    /// Create a new [`Series`].
    pub fn new(name: impl Into<SharedString>, values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            name: name.into(),
            values: values.into_iter().map(finite_or_zero).collect(),
            color: None,
        }
    }
    /// Colour (default: the theme's).
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
    /// Create a new [`Legend`].
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
    /// Create a new [`LineChart`].
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
    /// Height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// Fill under each line.
    pub fn area(mut self, a: bool) -> Self {
        self.area = a;
        self
    }
    /// Draw smooth curves instead of straight segments.
    pub fn smooth(mut self, s: bool) -> Self {
        self.smooth = s;
        self
    }
    /// Hoverable point markers (default on).
    pub fn points(mut self, p: bool) -> Self {
        self.points = p;
        self
    }
    /// Show or hide the legend.
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
    /// Create a new [`BarChart`].
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
    /// Labels, one per item.
    pub fn labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.labels = l.into_iter().map(Into::into).collect();
        self
    }
    /// Height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// Lay out horizontally.
    pub fn horizontal(mut self, h: bool) -> Self {
        self.horizontal = h;
        self
    }
    /// Stack the series instead of grouping them.
    pub fn stacked(mut self, s: bool) -> Self {
        self.stacked = s;
        self
    }
    /// Show or hide the legend.
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
    /// Create a new [`Sparkline`].
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
    /// Width and height in px.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.w = w;
        self.h = h;
        self
    }
    /// Colour (default: the theme's).
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    /// Fill the area under the line.
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
