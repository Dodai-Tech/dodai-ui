//! Gradient instruments: a dial, a ring and a cell bar whose lit part runs through a colour
//! [`Ramp`]. Every ramp is built from the active theme's palette, so one instrument gives a family
//! of variations per theme and changes with it.

use crate::gauge::chamfer_rect;
use crate::gpui::draw::{polar, turn_arc};
use crate::gpui::gauges::SWEEP;
use crate::instrument::{readout, Scale, Severity};
use dodai_core::{Color, Theme};
use dodai_gpui::follow::{follow, Follow};
use dodai_gpui::follow_api;
use dodai_gpui::paint::{fill, fill_circle, scale_alpha, stroke};
use dodai_gpui::plot::{glow_stroke, ChartGlow};
use dodai_gpui::text::{first_family, format_label};
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window,
};

/// Which palette colours a gradient instrument blends through.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ramp {
    /// Primary into secondary: the design's own accent pair.
    #[default]
    Accent,
    /// Info into primary.
    Cool,
    /// Success, warning, danger: calm to critical.
    Heat,
    /// Info, success, warning, danger: the whole status range.
    Spectrum,
    /// Primary from faint to full.
    Fade,
}

impl Ramp {
    /// Every ramp, in showcase order.
    pub const ALL: [Ramp; 5] = [
        Ramp::Accent,
        Ramp::Cool,
        Ramp::Heat,
        Ramp::Spectrum,
        Ramp::Fade,
    ];

    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            Ramp::Accent => "Accent",
            Ramp::Cool => "Cool",
            Ramp::Heat => "Heat",
            Ramp::Spectrum => "Spectrum",
            Ramp::Fade => "Fade",
        }
    }

    /// The colour stops of this ramp in theme `t`, first to last.
    pub fn stops(self, t: &Theme) -> Vec<Color> {
        let p = &t.palette;
        match self {
            Ramp::Accent => vec![p.primary, p.secondary],
            Ramp::Cool => vec![p.info, p.primary],
            Ramp::Heat => vec![p.success, p.warning, p.danger],
            Ramp::Spectrum => vec![p.info, p.success, p.warning, p.danger],
            Ramp::Fade => vec![p.primary.with_alpha(0.2), p.primary],
        }
    }

    /// The colour at `f` (`0..=1`) along this ramp in theme `t`.
    pub fn at(self, t: &Theme, f: f32) -> Hsla {
        hsla(blend(&self.stops(t), f))
    }
}

/// The colour at `f` (`0..=1`) along evenly spaced `stops`. Pure.
pub fn blend(stops: &[Color], f: f32) -> Color {
    match stops {
        [] => Color::rgba(0, 0, 0, 0),
        [only] => *only,
        _ => {
            let x = f.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
            let i = (x.floor() as usize).min(stops.len() - 2);
            stops[i].mix(stops[i + 1], x - i as f32)
        }
    }
}

/// Resolved ramp colours sampled `n + 1` times, so painting does not need the theme.
fn sample(ramp: Ramp, t: &Theme, n: usize) -> Vec<Hsla> {
    let stops = ramp.stops(t);
    (0..=n)
        .map(|i| hsla(blend(&stops, i as f32 / n as f32)))
        .collect()
}

/// The lit colour at `f` from a sampled ramp.
fn pick(cols: &[Hsla], f: f32) -> Hsla {
    let n = cols.len().saturating_sub(1);
    cols[((f.clamp(0.0, 1.0) * n as f32).round() as usize).min(n)]
}

/// Strokes the arc `f0..f1` of a sweep in short segments coloured along `cols`.
fn ramp_arc(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    (c, r, width): (f32, f32, f32),
    (start, span): (f32, f32),
    (f0, f1): (f32, f32),
    cols: &[Hsla],
    glow: Option<f32>,
) {
    if f1 <= f0 {
        return;
    }
    let n = ((f1 - f0) * 96.0).ceil().max(1.0) as usize;
    for i in 0..n {
        let a = f0 + (f1 - f0) * i as f32 / n as f32;
        // Overlap a hair so the joins do not show.
        let b = (f0 + (f1 - f0) * (i + 1) as f32 / n as f32 + 0.002).min(f1);
        let col = pick(cols, (a + b) / 2.0);
        let pts = turn_arc(c, c, r, start + span * a, start + span * b);
        glow_stroke(w, o, &pts, width, col, glow);
    }
}

/// The centre readout every round gradient instrument uses.
fn centre(
    t: &Theme,
    size: f32,
    big: f32,
    value: SharedString,
    color: Hsla,
    unit: &str,
    label: &str,
) -> gpui::Div {
    let muted = hsla(t.palette.text_muted);
    div()
        .absolute()
        .top_0()
        .left_0()
        .size(px(size))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .child(
            div()
                .font_family(first_family(t.typography.display))
                .text_size(px(big))
                .line_height(px(big * 1.05))
                .font_weight(gpui::FontWeight::LIGHT)
                .text_color(color)
                .child(value),
        )
        .child(
            div()
                .text_size(px(t.typography.size_xs))
                .text_color(muted)
                .child(format_label(&t.typography, unit)),
        )
        .child(
            div()
                .mt(px(size * 0.03))
                .text_size(px(t.typography.size_xs))
                .text_color(hsla(t.palette.text))
                .child(label.to_uppercase()),
        )
}

// ───────────────────────────────── GradientDial ─────────────────────────────────

/// An open-ring dial whose value band runs through a [`Ramp`]: the band shows the ramp up to the
/// value, ticks light in the colour they sit under, and a knob in the tip colour marks the value.
#[derive(IntoElement)]
pub struct GradientDial {
    follow: Follow,
    scale: Scale,
    value: f32,
    ramp: Ramp,
    label: SharedString,
    unit: SharedString,
    decimals: usize,
    size: f32,
    glow: ChartGlow,
}

impl GradientDial {
    follow_api!();
    /// A gradient dial for `value` on `scale`.
    #[track_caller]
    pub fn new(scale: Scale, value: f32) -> Self {
        Self {
            follow: Follow::here(),
            scale,
            value,
            ramp: Ramp::default(),
            label: SharedString::default(),
            unit: SharedString::default(),
            decimals: 0,
            size: 200.0,
            glow: ChartGlow::Theme,
        }
    }
    /// The colour ramp of the band.
    pub fn ramp(mut self, r: Ramp) -> Self {
        self.ramp = r;
        self
    }
    /// Caption under the value.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = l.into();
        self
    }
    /// Unit under the value.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// Decimals of the readout.
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s.max(96.0);
        self
    }
    /// Glow under the band; follows the app-wide chart glow by default.
    pub fn glow(mut self, g: ChartGlow) -> Self {
        self.glow = g;
        self
    }
}

impl RenderOnce for GradientDial {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.value = follow(w, cx, &self.follow, 0, self.value);
        let t = theme_of(cx);
        let s = self.size;
        let c = s / 2.0;
        let band = (s * 0.04).max(3.0);
        let r = c - band / 2.0 - 2.0;
        let frac = self.scale.fraction(self.value);
        let sev = self.scale.severity_at(self.value);
        let cols = sample(self.ramp, &t, 48);
        let track = hsla(t.progress(None).track);
        let muted = hsla(t.palette.text_muted);
        let face = hsla(t.palette.surface);
        let glow = self.glow.resolve(&t);
        let ticks: Vec<(f32, bool)> = self
            .scale
            .ticks()
            .into_iter()
            .map(|k| (k.frac, k.major))
            .collect();
        let tip = pick(&cols, frac);
        let sweep = SWEEP;
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let (start, span) = sweep;
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r, start, start + span),
                    band,
                    track,
                    false,
                );
                ramp_arc(w, o, (c, r, band), sweep, (0.0, frac), &cols, glow);
                for &(f, major) in &ticks {
                    let (r0, len) = (r - band * 1.6, if major { s * 0.05 } else { s * 0.025 });
                    let col = if f <= frac { pick(&cols, f) } else { muted };
                    let seg = [
                        polar(c, c, r0, start + span * f),
                        polar(c, c, r0 - len, start + span * f),
                    ];
                    stroke(w, o, &seg, if major { 1.2 } else { 0.6 }, col, false);
                }
                let (kx, ky) = polar(c, c, r, start + span * frac);
                fill_circle(w, o, (kx, ky), band * 1.3, tip);
                fill_circle(w, o, (kx, ky), band * 0.7, face);
            },
        )
        .absolute()
        .size_full();
        let ink = match sev {
            Severity::Normal => hsla(t.palette.text),
            _ => crate::gpui::dial::severity_color(&t, sev),
        };
        div().relative().size(px(s)).child(cv).child(centre(
            &t,
            s,
            s * 0.22,
            readout(self.value, self.decimals, "").into(),
            ink,
            &self.unit,
            &self.label,
        ))
    }
}

// ───────────────────────────────── GradientRing ─────────────────────────────────

/// A closed ring for a share (charge, load, progress) with a [`Ramp`] band, a faint echo of the
/// ramp on the track and the percentage in the middle.
#[derive(IntoElement)]
pub struct GradientRing {
    follow: Follow,
    percent: f32,
    ramp: Ramp,
    label: SharedString,
    caption: SharedString,
    size: f32,
    glow: ChartGlow,
}

impl GradientRing {
    follow_api!();
    /// A ring filled to `percent` (`0..=100`).
    #[track_caller]
    pub fn new(percent: f32) -> Self {
        Self {
            follow: Follow::here(),
            percent,
            ramp: Ramp::default(),
            label: SharedString::default(),
            caption: SharedString::default(),
            size: 160.0,
            glow: ChartGlow::Theme,
        }
    }
    /// The colour ramp of the band.
    pub fn ramp(mut self, r: Ramp) -> Self {
        self.ramp = r;
        self
    }
    /// Caption under the percentage.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = l.into();
        self
    }
    /// Small print between the percentage and the label, such as a range.
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = c.into();
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s.max(80.0);
        self
    }
    /// Glow under the band; follows the app-wide chart glow by default.
    pub fn glow(mut self, g: ChartGlow) -> Self {
        self.glow = g;
        self
    }
}

impl RenderOnce for GradientRing {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.percent = follow(w, cx, &self.follow, 0, self.percent.clamp(0.0, 100.0));
        let t = theme_of(cx);
        let s = self.size;
        let c = s / 2.0;
        let band = (s * 0.055).max(3.0);
        let r = c - band / 2.0 - 2.0;
        let frac = self.percent / 100.0;
        let cols = sample(self.ramp, &t, 48);
        let echo: Vec<Hsla> = cols.iter().map(|col| scale_alpha(*col, 0.16)).collect();
        let glow = self.glow.resolve(&t);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let full = (0.0, 1.0);
                ramp_arc(w, o, (c, r, band), full, (0.0, 1.0), &echo, None);
                ramp_arc(w, o, (c, r, band), full, (0.0, frac), &cols, glow);
                // Round the ends with discs in the end colours.
                for f in [0.0, frac] {
                    let (x, y) = polar(c, c, r, f);
                    fill_circle(w, o, (x, y), band / 2.0, pick(&cols, f));
                }
            },
        )
        .absolute()
        .size_full();
        let value: SharedString = format!("{}%", self.percent.round() as i32).into();
        div().relative().size(px(s)).child(cv).child(centre(
            &t,
            s,
            s * 0.2,
            value,
            hsla(t.palette.text),
            &self.caption,
            &self.label,
        ))
    }
}

// ───────────────────────────────── GradientBar ─────────────────────────────────

/// A row of cells lit through a [`Ramp`] from the scale's zero (or its start) to the value, so a
/// two-sided scale such as power and regeneration lights either way.
#[derive(IntoElement)]
pub struct GradientBar {
    follow: Follow,
    scale: Scale,
    value: f32,
    ramp: Ramp,
    label: SharedString,
    unit: SharedString,
    cells: usize,
    width: f32,
}

impl GradientBar {
    follow_api!();
    /// A cell bar for `value` on `scale`.
    #[track_caller]
    pub fn new(scale: Scale, value: f32) -> Self {
        Self {
            follow: Follow::here(),
            scale,
            value,
            ramp: Ramp::default(),
            label: SharedString::default(),
            unit: SharedString::default(),
            cells: 32,
            width: 320.0,
        }
    }
    /// The colour ramp of the cells.
    pub fn ramp(mut self, r: Ramp) -> Self {
        self.ramp = r;
        self
    }
    /// Caption on the left.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = l.into();
        self
    }
    /// Unit after the readout.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// Number of cells.
    pub fn cells(mut self, n: usize) -> Self {
        self.cells = n.clamp(4, 120);
        self
    }
    /// Width of the cell row in px.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w.max(80.0);
        self
    }
}

/// The lit span `(from, to)` of a bar: from the scale's zero when the scale spans it, else from
/// its start, to `frac`.
/// Pure.
pub fn lit_span(scale: &Scale, frac: f32) -> (f32, f32) {
    let zero = if scale.min < 0.0 && scale.max > 0.0 {
        scale.fraction(0.0)
    } else {
        0.0
    };
    (zero.min(frac), zero.max(frac))
}

impl RenderOnce for GradientBar {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.value = follow(w, cx, &self.follow, 0, self.value);
        let t = theme_of(cx);
        let n = self.cells;
        let (width, h) = (self.width, (t.sizes().slider_track * 2.4).max(12.0));
        let frac = self.scale.fraction(self.value);
        let (lo, hi) = lit_span(&self.scale, frac);
        let cols = sample(self.ramp, &t, n);
        let track = hsla(t.progress(None).track);
        // The design's chamfer cuts each cell, so a cut-corner design gets cut cells.
        let cut = t.shape.chamfer;
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let gap = 1.5;
                let cw = (width - gap * (n - 1) as f32) / n as f32;
                for i in 0..n {
                    let (f0, f1) = (i as f32 / n as f32, (i + 1) as f32 / n as f32);
                    let mid = (f0 + f1) / 2.0;
                    let on = mid >= lo && mid <= hi && hi > lo;
                    let x = i as f32 * (cw + gap);
                    // Cells grow a little towards the top end, like a level meter.
                    let ch = h * (0.55 + 0.45 * f1);
                    let col = if on { pick(&cols, mid) } else { track };
                    let rect = chamfer_rect(x, h - ch, cw, ch, cut.min(cw * 0.6));
                    fill(w, o, &rect, col);
                }
            },
        )
        .w(px(width))
        .h(px(h));
        let text = hsla(t.palette.text);
        let muted = hsla(t.palette.text_muted);
        div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w(px(width))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_size(px(t.typography.size_sm))
                    .child(
                        div()
                            .text_color(muted)
                            .child(format_label(&t.typography, &self.label)),
                    )
                    .child(
                        div()
                            .text_color(text)
                            .child(readout(self.value, 0, &self.unit)),
                    ),
            )
            .child(cv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blend_hits_every_stop_and_clamps() {
        let (a, b, c) = (
            Color::rgba(0, 0, 0, 255),
            Color::rgba(100, 100, 100, 255),
            Color::rgba(200, 0, 0, 255),
        );
        let s = [a, b, c];
        assert_eq!(blend(&s, 0.0), a);
        assert_eq!(blend(&s, 0.5), b);
        assert_eq!(blend(&s, 1.0), c);
        assert_eq!(blend(&s, 7.0), c);
        assert_eq!(blend(&s, -1.0), a);
        assert_eq!(blend(&s, 0.25), Color::rgba(50, 50, 50, 255));
        assert_eq!(blend(&[b], 0.3), b);
    }

    #[test]
    fn every_ramp_has_at_least_two_stops() {
        let t = Theme::default();
        for r in Ramp::ALL {
            assert!(r.stops(&t).len() >= 2, "{}", r.name());
        }
    }

    #[test]
    fn two_sided_bar_lights_from_zero() {
        let s = Scale::new(-60.0, 120.0);
        let zero = s.fraction(0.0);
        assert_eq!(lit_span(&s, s.fraction(60.0)), (zero, s.fraction(60.0)));
        assert_eq!(lit_span(&s, s.fraction(-30.0)), (s.fraction(-30.0), zero));
        let p = Scale::new(10.0, 50.0);
        assert_eq!(lit_span(&p, 0.5), (0.0, 0.5));
    }
}
