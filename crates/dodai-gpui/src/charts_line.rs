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
    y_bounds: Option<(f32, f32)>,
    fmt: Option<NumFmt>,
    grid: bool,
    stroke_w: f32,
    stepped: bool,
    zero: bool,
    title: Option<SharedString>,
}

impl LineChart {
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
        Self {
            glow: crate::charts::ChartGlow::Theme,
            annotations: Vec::new(),
            id: id.into(),
            secondary: Vec::new(),
            series: series.into_iter().collect(),
            labels: Vec::new(),
            height: 200.0,
            area: false,
            smooth: false,
            points: true,
            crosshair: false,
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
        let glow = self.glow.resolve(&t);
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
        let xh = self.crosshair.then(|| {
            window.use_keyed_state(SharedString::from(format!("{}-xh", self.id)), cx, |_, _| {
                None::<usize>
            })
        });
        let cur: Option<usize> = xh.as_ref().and_then(|s| *s.read(cx)).filter(|i| *i < n);
        let hair = hsla(t.palette.text_muted);
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
                if let Some(st) = xh.clone() {
                    if let Some(i) = cur {
                        let x = (i as f32 + 0.5) / n as f32 * bw;
                        stroke(w, b.origin, &[(x, 0.0), (x, bh)], 1.0, hair, false);
                        for (c, vals, (lo, hi)) in &lines {
                            if let Some(v) = vals.get(i) {
                                let y = bh - norm(*v, *lo, *hi) * bh * g;
                                fill(w, b.origin, &marker_points(Marker::Dot, x, y, 4.0), *c);
                            }
                        }
                    }
                    w.on_mouse_event(move |e: &gpui::MouseMoveEvent, _, w, cx| {
                        let next = b.contains(&e.position).then(|| {
                            let fx = f32::from(e.position.x - b.origin.x) / f32::from(b.size.width);
                            ((fx * n as f32) as usize).min(n.saturating_sub(1))
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
            let label = self
                .labels
                .get(i)
                .map(|l| l.to_string())
                .unwrap_or_default();
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
