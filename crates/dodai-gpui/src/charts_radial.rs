//! Gauge and radar chart.

use crate::charts::*;
use crate::paint::scale_alpha;
use crate::text::first_family;
use crate::{hsla, theme_of};
use gpui::{
    canvas, div, px, relative, App, Bounds, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, Pixels, SharedString, Styled, Window,
};

// ─────────────────────────────────── Gauge ───────────────────────────────────

/// Radial gauge with a centre label. [`crate::gauges::GaugeStyle`] picks the look: the
/// default 270° arc, segmented ring, dial, meter, reticle or slanted-cell bar.
#[derive(IntoElement)]
pub struct Gauge {
    glow: crate::charts::ChartGlow,
    value: f32,
    min: f32,
    max: f32,
    size: f32,
    label: Option<SharedString>,
    unit: &'static str,
    caption: Option<SharedString>,
    tone: Option<dodai_core::Tone>,
    style: crate::gauges::GaugeStyle,
    zones: bool,
    target: Option<f32>,
}

impl Gauge {
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: crate::charts::ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// Create a new [`Gauge`].
    pub fn new(value: f32) -> Self {
        Self {
            glow: crate::charts::ChartGlow::Theme,
            value,
            min: 0.0,
            max: 100.0,
            size: 120.0,
            label: None,
            unit: "",
            caption: None,
            tone: None,
            style: crate::gauges::GaugeStyle::Arc,
            zones: false,
            target: None,
        }
    }
    /// Value range `min..max`.
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Centre label (default: the rounded value).
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Unit appended to the default label (the rounded value), so it follows a changing value.
    pub fn unit(mut self, u: &'static str) -> Self {
        self.unit = u;
        self
    }
    /// Small caption under the label (uppercased when the theme does).
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
    /// Semantic tone (colour).
    pub fn tone(mut self, t: dodai_core::Tone) -> Self {
        self.tone = Some(t);
        self
    }
    /// Visual style.
    pub fn style(mut self, s: crate::gauges::GaugeStyle) -> Self {
        self.style = s;
        self
    }
    /// Target marker at `value` (same range as the gauge).
    pub fn target(mut self, v: f32) -> Self {
        self.target = Some(v);
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
        let glow = self.glow.resolve(&t);
        let size = self.size;
        let style = self.style;
        let (bw, bh) = style.dims(size);
        let ring = (size * 0.1).clamp(4.0, 12.0);
        let label = self.label.unwrap_or_else(|| {
            format!("{}{}", fmt_num((self.value * g).round()), self.unit).into()
        });
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
            alert: hsla(t.palette.danger),
            target: self.target.map(|v| norm(v, self.min, self.max)),
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
                    crate::gauges::paint(w, b, look);
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
                crate::gauges::paint(w, b, look);
            },
        )
        .absolute()
        .size_full();
        let caption = self
            .caption
            .map(|c| crate::text::format_label(&t.typography, &c));
        let value_size = match style {
            GaugeStyle::Bar | GaugeStyle::Chevron | GaugeStyle::Led => bh * 0.9,
            GaugeStyle::Battery | GaugeStyle::Signal | GaugeStyle::Thermometer => size * 0.16,
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
            GaugeStyle::Bar
            | GaugeStyle::Chevron
            | GaugeStyle::Level
            | GaugeStyle::Led
            | GaugeStyle::Battery
            | GaugeStyle::Signal
            | GaugeStyle::Thermometer => div()
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
    follow: crate::instruments_motion::Follow,
    glow: crate::charts::ChartGlow,
    axes: Vec<SharedString>,
    series: Vec<Series>,
    size: f32,
    max: Option<f32>,
    legend: bool,
}

impl RadarChart {
    crate::instruments_motion::follow_api!();
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: crate::charts::ChartGlow) -> Self {
        self.glow = g;
        self
    }
    #[track_caller]
    /// Create a new [`RadarChart`].
    pub fn new<S: Into<SharedString>>(
        axes: impl IntoIterator<Item = S>,
        series: impl IntoIterator<Item = Series>,
    ) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            glow: crate::charts::ChartGlow::Theme,
            axes: axes.into_iter().map(Into::into).collect(),
            series: series.into_iter().collect(),
            size: 220.0,
            max: None,
            legend: true,
        }
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Maximum value.
    pub fn max(mut self, m: f32) -> Self {
        self.max = Some(m);
        self
    }
    /// Show or hide the legend.
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

impl RadarChart {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let g = crate::motion::grow(window, cx, "RadarChart");
        self.series =
            crate::instruments_motion::follow_series(window, cx, &self.follow, &self.series);
        let t = theme_of(cx);
        let n = self.axes.len().max(3);
        let max = self.max.unwrap_or_else(|| {
            *nice_ticks(0.0, y_range(&self.series, true, None).1, 4)
                .last()
                .unwrap_or(&1.0)
        });
        let grid = hsla(t.palette.border);
        let glow = self.glow.resolve(&t);
        let size = self.size;
        let focus = crate::charts_focus::Focus::new(window, cx, &self.follow, self.axes.len());
        let foc: Vec<crate::charts_focus::ItemFocus> =
            (0..self.axes.len()).map(|i| focus.item(i)).collect();
        let foc_c = foc.clone();
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
                    let lf = foc_c
                        .get(i)
                        .copied()
                        .unwrap_or(crate::charts_focus::ItemFocus {
                            alpha: 1.0,
                            lift: 0.0,
                        });
                    stroke(
                        w,
                        b.origin,
                        &[(c, c), polar(c, c, r, i as f32 / n as f32)],
                        1.0 + 1.5 * lf.lift,
                        scale_alpha(grid, lf.alpha),
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
                    for (i, q) in p.iter().enumerate() {
                        let lf = foc_c.get(i).map_or(0.0, |f| f.lift);
                        if lf > 0.02 {
                            fill(
                                w,
                                b.origin,
                                &crate::charts::marker_points(
                                    crate::charts::Marker::Dot,
                                    q.0,
                                    q.1,
                                    2.0 + 2.5 * lf,
                                ),
                                *col,
                            );
                        }
                    }
                }
            },
        )
        .absolute()
        .size_full();
        // Axis labels positioned just outside the web.
        let mut web = div().relative().w(px(size)).h(px(size)).child(cvs);
        for (i, a) in self.axes.iter().enumerate() {
            let (x, y) = polar(0.5, 0.5, 0.5, i as f32 / n as f32);
            let lf = foc[i];
            web = web.child(
                focus
                    .bind(label_style(&t, div()).id(("radar-axis", i)), i)
                    .opacity(lf.alpha)
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
