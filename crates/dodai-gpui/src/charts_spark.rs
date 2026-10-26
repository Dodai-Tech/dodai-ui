//! [`Sparkline`].

use crate::charts::*;
use crate::paint::scale_alpha;
use crate::{hsla, theme_of};
use dodai_core::Color;
use gpui::{
    canvas, div, px, App, Bounds, IntoElement, ParentElement, Pixels, RenderOnce, Styled, Window,
};

// ─────────────────────────────── Sparkline ───────────────────────────────

/// Tiny inline line, no axes.
#[derive(IntoElement)]
pub struct Sparkline {
    live: Option<crate::live::Live>,
    follow: crate::instruments_motion::Follow,
    glow: ChartGlow,
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
    smooth: usize,
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
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: ChartGlow) -> Self {
        self.glow = g;
        self
    }
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
    crate::instruments_motion::follow_api!();
    /// Draws a [`crate::Live`] window instead of fixed values, sliding left between samples.
    pub fn live(mut self, window: &crate::live::Live) -> Self {
        self.values = window.values();
        self.live = Some(window.clone());
        self
    }
    /// Create a new [`Sparkline`].
    #[track_caller]
    pub fn new(values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            live: None,
            follow: crate::instruments_motion::Follow::here(),
            glow: ChartGlow::Theme,
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
            smooth: 0,
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
    /// Round the line with `n` interpolated points per segment (0 = straight).
    pub fn smooth(mut self, n: usize) -> Self {
        self.smooth = n;
        self
    }
    /// Fill the area under the line.
    pub fn area(mut self, a: bool) -> Self {
        self.area = a;
        self
    }
}

impl RenderOnce for Sparkline {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if let Some(l) = &self.live {
            self.values = l.values();
            crate::instruments_motion::next_frame(window);
        } else {
            self.values =
                crate::instruments_motion::follow_all(window, cx, &self.follow, &self.values);
        }
        let motion = crate::motion::current(cx);
        let scroll = self.live.as_ref().map(crate::live::Live::scroll);
        crate::motion::entry(self.build(cx, scroll), "Sparkline", motion)
    }
}

impl Sparkline {
    fn build(self, cx: &mut App, scroll: Option<f32>) -> impl IntoElement {
        let t = theme_of(cx);
        let c = hsla(self.color.unwrap_or(t.palette.primary));
        let glow = self.glow.resolve(&t).map(|r| r.min(4.0));
        let (vals, area, marker, all_points) =
            (self.values, self.area, self.marker, self.all_points);
        let (style, baseline, smooth) = (self.style, self.baseline, self.smooth);
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
                                (i as f32 - scroll.unwrap_or(0.0)) / (m - 1) as f32 * bw,
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
                    let curve = if smooth > 0 {
                        smooth_points(&p, smooth)
                    } else {
                        p.clone()
                    };
                    w.with_content_mask(Some(gpui::ContentMask { bounds: b }), |w| {
                        if area {
                            let mut poly = curve.clone();
                            poly.push((curve.last().map_or(bw, |q| q.0), bh));
                            poly.push((curve.first().map_or(0.0, |q| q.0), bh));
                            fill(w, b.origin, &poly, scale_alpha(c, 0.16));
                        }
                        glow_stroke(w, b.origin, &curve, 1.5, c, glow);
                    });
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
