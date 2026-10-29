//! [`LineChart`].

use crate::charts::*;
use crate::paint::scale_alpha;
use crate::panel::DodaiTooltip;
use crate::{hsla, theme_of};
use gpui::{
    canvas, div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ─────────────────────────────── LineChart ───────────────────────────────

/// Line / area chart with gridlines, axes, optional smoothing, point markers
/// (with hover tooltips) and glow on glowing themes.
#[derive(IntoElement)]
pub struct LineChart {
    follow: crate::instruments_motion::Follow,
    glow: crate::charts::ChartGlow,
    annotations: Vec<crate::charts_annotate::Annotation>,
    id: SharedString,
    series: Vec<Series>,
    secondary: Vec<Series>,
    labels: Vec<SharedString>,
    height: f32,
    area: bool,
    smooth: bool,
    points: bool,
    crosshair: bool,
    legend: bool,
    toggleable: bool,
    patterns: bool,
    time: Option<(Vec<f64>, i32)>,
    y_bounds: Option<(f32, f32)>,
    fmt: Option<NumFmt>,
    grid: bool,
    stroke_w: Option<f32>,
    stepped: bool,
    zero: bool,
    title: Option<SharedString>,
    axes: crate::charts::AxisTitles,
}

impl LineChart {
    crate::instruments_motion::follow_api!();
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: crate::charts::ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// Threshold lines and highlighted bands drawn over the plot.
    pub fn annotations(
        mut self,
        a: impl IntoIterator<Item = crate::charts_annotate::Annotation>,
    ) -> Self {
        self.annotations = a.into_iter().collect();
        self
    }
    /// Create a new [`LineChart`].
    pub fn new(id: impl Into<SharedString>, series: impl IntoIterator<Item = Series>) -> Self {
        let id: SharedString = id.into();
        Self {
            follow: crate::instruments_motion::Follow::named(&id),
            glow: crate::charts::ChartGlow::Theme,
            annotations: Vec::new(),
            id,
            secondary: Vec::new(),
            series: series.into_iter().collect(),
            labels: Vec::new(),
            height: 200.0,
            area: false,
            smooth: false,
            points: true,
            crosshair: false,
            legend: true,
            toggleable: false,
            patterns: false,
            time: None,
            y_bounds: None,
            fmt: None,
            grid: true,
            stroke_w: None,
            stepped: false,
            zero: true,
            title: None,
            axes: crate::charts::AxisTitles::default(),
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
    /// Line thickness in px (default: the design's chart style).
    pub fn stroke_width(mut self, w: f32) -> Self {
        self.stroke_w = Some(w);
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
    /// Name of the horizontal axis, centred under the plot (e.g. "Seconds").
    pub fn x_title(mut self, t: impl Into<SharedString>) -> Self {
        self.axes.x = Some(t.into());
        self
    }
    /// Name of the vertical axis, over its value labels (e.g. a unit, "Mbps").
    pub fn y_title(mut self, t: impl Into<SharedString>) -> Self {
        self.axes.y = Some(t.into());
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
    /// Vertical hairline that snaps to the nearest sample under the pointer,
    /// with a value readout for every series.
    pub fn crosshair(mut self, on: bool) -> Self {
        self.crosshair = on;
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
    /// Place the points by timestamp (unix seconds, UTC) instead of equal slots, and label the
    /// x axis with calendar-aligned ticks (seconds … years, chosen from the span). Sample `i`
    /// of every series sits at `stamps[i]`. Use [`LineChart::utc_offset`] for a local zone.
    pub fn time_axis(mut self, stamps: impl IntoIterator<Item = f64>) -> Self {
        let off = self.time.as_ref().map_or(0, |t| t.1);
        self.time = Some((stamps.into_iter().collect(), off));
        self
    }
    /// Fixed UTC offset in seconds for the time axis labels (default 0 = UTC).
    pub fn utc_offset(mut self, secs: i32) -> Self {
        if let Some(t) = self.time.as_mut() {
            t.1 = secs;
        } else {
            self.time = Some((Vec::new(), secs));
        }
        self
    }
    /// Draw each series with its own dash pattern (solid, dashed, dotted, dash-dot), so the
    /// lines stay apart without relying on colour (colour-blind and print safe).
    pub fn patterns(mut self, on: bool) -> Self {
        self.patterns = on;
        self
    }
    /// Clicking a legend entry hides or shows that series; the axis refits to what is left.
    pub fn toggleable(mut self, on: bool) -> Self {
        self.toggleable = on;
        self
    }
}

impl LineChart {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow_for(window, cx, &self.follow, "LineChart");
        let mut legend_all: Option<(Vec<Series>, crate::charts::Hidden)> = None;
        {
            let np = self.series.len();
            let all: Vec<Series> = self
                .series
                .iter()
                .chain(self.secondary.iter())
                .cloned()
                .collect();
            let mut shown =
                crate::instruments_motion::follow_series(window, cx, &self.follow, &all);
            if self.toggleable {
                let (visible, h, pinned) = crate::charts::toggled(window, cx, &self.id, shown);
                let off = h.read(cx).clone();
                let hid = |i: usize| off.get(i).copied().unwrap_or(false);
                let ns = (0..np).filter(|i| !hid(*i)).count();
                self.secondary = visible[ns..].to_vec();
                self.series = visible[..ns].to_vec();
                legend_all = Some((pinned, h));
            } else {
                self.secondary = shown.split_off(np);
                self.series = shown;
            }
        }
        // A live window slides left between samples.
        let scroll = self
            .series
            .iter()
            .chain(self.secondary.iter())
            .find_map(Series::scroll);
        if scroll.is_some() {
            crate::instruments_motion::next_frame(window);
        }
        let (lo, hi, ticks) = y_range(&self.series, self.zero, self.y_bounds);
        let (lo, hi) =
            crate::instruments_motion::follow_range(window, cx, &self.follow, "y", (lo, hi));
        let np = self.series.len();
        let (lo2, hi2, ticks2) = if self.secondary.is_empty() {
            (lo, hi, Vec::new())
        } else {
            let (a, b, _) = y_range(&self.secondary, self.zero, None);
            let (_, t2) = dodai_charts_core::axis::aligned_ticks((lo, hi), (a, b), 4);
            let (a, b) = (*t2.first().unwrap_or(&a), *t2.last().unwrap_or(&b));
            let (a, b) =
                crate::instruments_motion::follow_range(window, cx, &self.follow, "y2", (a, b));
            (a, b, t2)
        };
        let all: Vec<Series> = self
            .series
            .iter()
            .chain(self.secondary.iter())
            .cloned()
            .collect();
        let range_of = move |i: usize| if i < np { (lo, hi) } else { (lo2, hi2) };
        let n = all.iter().map(|s| s.values.len()).max().unwrap_or(0);
        let style = t.chart_style();
        let grid = hsla(style.grid_color);
        let glow = self.glow.resolve(&t);
        let patterns = self.patterns;
        let xs: Option<Vec<f32>> = self.time.as_ref().and_then(|(s, _)| time_fracs(s));
        let (area, smooth, stepped, sw, show_grid) = (
            self.area,
            self.smooth,
            self.stepped,
            self.stroke_w.unwrap_or(style.line_width),
            self.grid,
        );
        let lines: Vec<(Hsla, Vec<f32>, (f32, f32))> = all
            .iter()
            .enumerate()
            .map(|(i, s)| (hsla(series_color(&t, s, i)), s.values.clone(), range_of(i)))
            .collect();
        let bands: Vec<Option<(Vec<f32>, Vec<f32>)>> = all.iter().map(|s| s.band.clone()).collect();
        let xh = self.crosshair.then(|| {
            window.use_keyed_state(SharedString::from(format!("{}-xh", self.id)), cx, |_, _| {
                None::<usize>
            })
        });
        let cur: Option<usize> = xh.as_ref().and_then(|s| *s.read(cx)).filter(|i| *i < n);
        let hair = hsla(t.palette.text_muted);
        let xs_hover = xs.clone();
        let tick_fracs: Vec<f32> = ticks
            .iter()
            .map(|v| norm(*v, lo, hi))
            .filter(|f| (0.0..=1.0).contains(f))
            .collect();

        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                for f in tick_fracs.iter().filter(|_| show_grid) {
                    let y = bh - f * bh;
                    crate::charts::grid_line(w, b.origin, (0.0, y), (bw, y), &style);
                }
                let mut shown = b;
                if scroll.is_none() {
                    shown.size.width = b.size.width * g.clamp(0.0, 1.0);
                }
                let mask = gpui::ContentMask { bounds: shown };
                w.with_content_mask(Some(mask), |w| {
                    for (li, (c, vals, (lo, hi))) in lines.iter().enumerate() {
                        let (lo, hi) = (*lo, *hi);
                        let m = vals.len();
                        let x_of = |i: usize| match scroll {
                            // The newest sample sits at the right edge, sliding left.
                            Some(s) => (i as f32 - s) / (m.max(2) - 1) as f32 * bw,
                            // Centre of category slot `i`, matching the x labels.
                            None => slot_frac(&xs, i, m) * bw,
                        };
                        // The same path rule as the line, so a band hugs it.
                        let path = |vs: &[f32]| {
                            let raw: Vec<(f32, f32)> = vs
                                .iter()
                                .enumerate()
                                .map(|(i, v)| (x_of(i), bh - norm(*v, lo, hi) * bh))
                                .collect();
                            let raw = if stepped { step_points(&raw) } else { raw };
                            if smooth && !stepped {
                                smooth_points(&raw, 8)
                            } else {
                                raw
                            }
                        };
                        if let Some((blo, bhi)) = &bands[li] {
                            let (top, bottom) = (path(bhi), path(blo));
                            if top.len() > 1 && bottom.len() > 1 {
                                let mut poly = top;
                                poly.extend(bottom.into_iter().rev());
                                fill(w, b.origin, &poly, scale_alpha(*c, style.area_alpha));
                            }
                        }
                        let p = path(vals);
                        if p.len() < 2 {
                            continue;
                        }
                        if area {
                            let mut poly = p.clone();
                            poly.push((p.last().unwrap().0, bh));
                            poly.push((p[0].0, bh));
                            fill(w, b.origin, &poly, scale_alpha(*c, style.area_alpha));
                        }
                        match dash_pattern(li).filter(|_| patterns) {
                            Some((on, off)) => {
                                for seg in dashed(&p, on * sw.max(1.0), off * sw.max(1.0)) {
                                    glow_stroke(w, b.origin, &seg, sw, *c, glow);
                                }
                            }
                            None => glow_stroke(w, b.origin, &p, sw, *c, glow),
                        }
                    }
                });
                if let Some(st) = xh.clone() {
                    if let Some(i) = cur {
                        let x = slot_frac(&xs, i, n) * bw;
                        stroke(w, b.origin, &[(x, 0.0), (x, bh)], 1.0, hair, false);
                        for (c, vals, (lo, hi)) in &lines {
                            if let Some(v) = vals.get(i) {
                                let y = bh - norm(*v, *lo, *hi) * bh;
                                fill(w, b.origin, &marker_points(Marker::Dot, x, y, 4.0), *c);
                            }
                        }
                    }
                    w.on_mouse_event(move |e: &gpui::MouseMoveEvent, _, w, cx| {
                        let next = b.contains(&e.position).then(|| {
                            let fx = f32::from(e.position.x - b.origin.x) / f32::from(b.size.width);
                            nearest_slot(&xs, fx, n)
                        });
                        if *st.read(cx) != next {
                            st.update(cx, |v, _| *v = next);
                            w.refresh();
                        }
                    });
                }
            },
        )
        .absolute()
        .size_full();

        let mut plot = div().relative().w_full().h(px(self.height)).child(cv);
        plot = plot.child(crate::charts_annotate::layer(
            &t,
            &self.annotations,
            (lo, hi),
            &crate::charts_annotate::category_x(n),
        ));

        if let Some(i) = cur {
            let label = match self
                .time
                .as_ref()
                .and_then(|(s, o)| s.get(i).map(|t| (*t, *o)))
            {
                Some((t, o)) => dodai_charts_core::timeaxis::format_datetime(t, o),
                None => self
                    .labels
                    .get(i)
                    .map(|l| l.to_string())
                    .unwrap_or_default(),
            };
            let mut tip = label_style(&t, div())
                .absolute()
                .top(px(t.spacing.xs))
                .px(px(t.spacing.sm))
                .py(px(t.spacing.xs))
                .rounded(px(t.spacing.xs))
                .bg(hsla(t.palette.surface_raised))
                .border_1()
                .border_color(grid)
                .flex()
                .flex_col()
                .child(div().child(label));
            for (si, s) in all.iter().enumerate() {
                if let Some(v) = s.values.get(i) {
                    tip = tip.child(
                        div()
                            .text_color(hsla(series_color(&t, s, si)))
                            .child(format!("{}: {}", s.name, fmt_with(&self.fmt, *v))),
                    );
                }
            }
            // Park the readout on the side away from the pointer.
            tip = if i * 2 < n {
                tip.right(px(t.spacing.xs))
            } else {
                tip.left(px(t.spacing.xs))
            };
            plot = plot.child(tip);
        }
        if self.points && scroll.is_none() {
            for (si, s) in all.iter().enumerate() {
                let (lo, hi) = range_of(si);
                let c = hsla(series_color(&t, s, si));
                let m = s.values.len();
                for (i, v) in s.values.iter().enumerate() {
                    let fx = slot_frac(&xs_hover, i, m);
                    let fy = 1.0 - norm(*v, lo, hi);
                    let label = self.labels.get(i).cloned().unwrap_or_default();
                    let label = match self
                        .time
                        .as_ref()
                        .and_then(|(s, o)| s.get(i).map(|t| (*t, *o)))
                    {
                        Some((t, o)) => dodai_charts_core::timeaxis::format_datetime(t, o).into(),
                        None => label,
                    };
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
        let time_row = self.time.as_ref().and_then(|(s, o)| time_row(&t, s, *o));
        let x_labels: Vec<SharedString> = if time_row.is_some() {
            Vec::new()
        } else if self.labels.len() == n {
            self.labels.clone()
        } else {
            Vec::new()
        };
        let mut root = div().flex().flex_col().gap(px(t.spacing.sm)).w_full();
        let ylab: Vec<String> = ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect();
        let left = match time_row {
            Some(row) => crate::charts::framed_row(&t, ylab, Some(row), plot),
            None => framed(&t, ylab, x_labels, plot),
        };
        let left = self.axes.around(&t, t.sizes().axis_gutter, left);
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
        if let (true, Some((pinned, h))) = (self.legend, legend_all) {
            if pinned.len() > 1 {
                root =
                    root.child(Legend::of_series(&t, &pinned).toggle(format!("{}-lg", self.id), h));
            }
        } else if self.legend && all.len() > 1 {
            root = root.child(Legend::of_series(&t, &all));
        }
        titled(&t, self.title.clone(), root)
    }
}

crate::entry_render!(LineChart);

/// Dash pattern `(on, off)` in stroke widths for series `i`; `None` = solid.
pub(crate) fn dash_pattern(i: usize) -> Option<(f32, f32)> {
    match i % 4 {
        0 => None,
        1 => Some((4.0, 3.0)),
        2 => Some((1.0, 2.5)),
        _ => Some((5.0, 2.0)),
    }
}

/// Split a polyline into dashes `on` long separated by `off` gaps (px). Pure.
pub(crate) fn dashed(p: &[(f32, f32)], on: f32, off: f32) -> Vec<Vec<(f32, f32)>> {
    let (on, off) = (on.max(0.5), off.max(0.5));
    let mut out = Vec::new();
    let mut cur: Vec<(f32, f32)> = Vec::new();
    let (mut drawing, mut left) = (true, on);
    for pair in p.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        if len <= f32::EPSILON {
            continue;
        }
        let mut at = 0.0;
        if drawing && cur.is_empty() {
            cur.push(a);
        }
        while at < len {
            let step = (len - at).min(left);
            at += step;
            left -= step;
            let q = (a.0 + (b.0 - a.0) * at / len, a.1 + (b.1 - a.1) * at / len);
            if drawing {
                cur.push(q);
            }
            if left <= f32::EPSILON {
                if drawing {
                    out.push(std::mem::take(&mut cur));
                } else {
                    cur.push(q);
                }
                drawing = !drawing;
                left = if drawing { on } else { off };
            }
        }
    }
    if cur.len() > 1 {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod dash_tests {
    use super::*;

    #[test]
    fn dashes_cover_the_on_fraction_of_a_straight_line() {
        let d = dashed(&[(0.0, 0.0), (100.0, 0.0)], 4.0, 6.0);
        assert_eq!(d.len(), 10);
        let covered: f32 = d.iter().map(|s| s.last().unwrap().0 - s[0].0).sum();
        assert!((covered - 40.0).abs() < 0.5, "{covered}");
    }

    #[test]
    fn dashes_follow_corners_and_survive_degenerate_input() {
        let d = dashed(&[(0.0, 0.0), (5.0, 0.0), (5.0, 50.0)], 20.0, 5.0);
        assert!(
            d.iter().any(|s| s.len() > 2),
            "a dash bends round the corner"
        );
        assert!(dashed(&[], 3.0, 3.0).is_empty());
        assert!(dashed(&[(1.0, 1.0), (1.0, 1.0)], 3.0, 3.0).is_empty());
    }

    #[test]
    fn first_series_stays_solid() {
        assert!(dash_pattern(0).is_none() && dash_pattern(1).is_some());
    }
}

/// Inset so end markers are not clipped by the plot edge.
const TIME_PAD: f32 = 0.03;

/// Horizontal fractions (0..1) of each timestamp, or `None` if fewer than two distinct finite
/// times (the chart then falls back to equal slots). Pure.
pub(crate) fn time_fracs(stamps: &[f64]) -> Option<Vec<f32>> {
    let fin = || stamps.iter().copied().filter(|t| t.is_finite());
    let lo = fin().fold(f64::INFINITY, f64::min);
    let hi = fin().fold(f64::NEG_INFINITY, f64::max);
    if hi.partial_cmp(&lo) != Some(std::cmp::Ordering::Greater) {
        return None;
    }
    Some(
        stamps
            .iter()
            .map(|t| {
                let f = dodai_charts_core::timeaxis::frac(*t, lo, hi);
                TIME_PAD + f * (1.0 - 2.0 * TIME_PAD)
            })
            .collect(),
    )
}

/// Fraction of sample `i` of `m`: by timestamp when set, else the centre of equal slot `i`.
pub(crate) fn slot_frac(xs: &Option<Vec<f32>>, i: usize, m: usize) -> f32 {
    xs.as_ref()
        .and_then(|x| x.get(i).copied())
        .unwrap_or((i as f32 + 0.5) / m.max(1) as f32)
}

/// Index of the sample nearest to pointer fraction `fx`.
pub(crate) fn nearest_slot(xs: &Option<Vec<f32>>, fx: f32, n: usize) -> usize {
    match xs {
        Some(x) if !x.is_empty() => x
            .iter()
            .enumerate()
            .min_by(|a, b| (a.1 - fx).abs().total_cmp(&(b.1 - fx).abs()))
            .map_or(0, |(i, _)| i),
        _ => ((fx * n as f32) as usize).min(n.saturating_sub(1)),
    }
}

/// The x-axis label row for a time axis: calendar ticks placed by position.
fn time_row(t: &dodai_core::Theme, stamps: &[f64], offset: i32) -> Option<gpui::Div> {
    time_fracs(stamps)?;
    let fin = || stamps.iter().copied().filter(|v| v.is_finite());
    let lo = fin().fold(f64::INFINITY, f64::min);
    let hi = fin().fold(f64::NEG_INFINITY, f64::max);
    let cell = 72.0;
    let ticks = dodai_charts_core::timeaxis::ticks(lo, hi, 6, offset);
    let line = (t.typography.size_xs * 1.5).ceil();
    let mut row = label_style(t, div()).relative().h(px(line)).w_full();
    for k in ticks {
        let f = TIME_PAD + dodai_charts_core::timeaxis::frac(k.t, lo, hi) * (1.0 - 2.0 * TIME_PAD);
        // Skip a label whose box would spill over the y-axis gutter or the right edge.
        if !(0.07..=0.93).contains(&f) {
            continue;
        }
        row = row.child(
            div()
                .absolute()
                .left(relative(f))
                .ml(px(-cell / 2.0))
                .w(px(cell))
                .flex()
                .justify_center()
                .child(k.label),
        );
    }
    Some(row)
}

#[cfg(test)]
mod time_tests {
    use super::*;

    #[test]
    fn fractions_follow_the_clock_not_the_index() {
        let f = time_fracs(&[0.0, 10.0, 100.0]).unwrap();
        assert!((f[0] - TIME_PAD).abs() < 1e-6 && (f[2] - (1.0 - TIME_PAD)).abs() < 1e-6);
        assert!(
            f[1] - f[0] < f[2] - f[1],
            "a 10 s gap is narrower than a 90 s gap"
        );
    }

    #[test]
    fn degenerate_stamps_fall_back_to_slots() {
        assert!(time_fracs(&[]).is_none());
        assert!(time_fracs(&[5.0, 5.0]).is_none());
        assert!(time_fracs(&[f64::NAN, 1.0]).is_none());
        assert_eq!(slot_frac(&None, 1, 4), 0.375);
    }

    #[test]
    fn pointer_snaps_to_the_nearest_timestamp() {
        let xs = time_fracs(&[0.0, 10.0, 100.0]);
        assert_eq!(nearest_slot(&xs, 0.2, 3), 1);
        assert_eq!(nearest_slot(&xs, 0.9, 3), 2);
        assert_eq!(nearest_slot(&None, 0.9, 4), 3);
    }
}
