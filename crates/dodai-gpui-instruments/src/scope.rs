//! Instrument-style charts: [`Oscilloscope`], [`Spectrum`], [`RadarScope`],
//! [`HexMap`] and [`RingStack`]. They sit in a themed panel frame, glow when
//! the theme does and take every colour from the palette. The `live` ones
//! animate from a shared clock (and only then ask for frames).

use crate::draw::{fmt_num, glow_stroke, norm, turn_arc, ChartGlow};
use crate::gauges::{chamfer_rect, disc, zone, SWEEP};
use dodai_core::{Color, Theme};
use dodai_gpui::paint::{fill, fill_batch, scale_alpha, shaped, stroke, stroke_batch};
use dodai_gpui::text::{first_family, format_label};
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window,
};

// ───────────────────────────── pure helpers ─────────────────────────────

#[cfg(feature = "medical")]
pub use dodai_instruments_core::gauge::ecg_beat;
/// Seconds since the first call (shared clock for live charts).
pub use dodai_instruments_core::gauge::{
    blip_alpha, hex_center, hex_extent, hex_points, jitter, ring_radius,
};

pub(crate) fn clock() -> f32 {
    dodai_gpui::follow::clock()
}

pub(crate) fn frame(
    t: &Theme,
    w: f32,
    h: f32,
    caption: Option<SharedString>,
    body: impl IntoElement,
) -> gpui::Div {
    let mut f = shaped(&t.shape, dodai_gpui::panel::panel_style(t, false))
        .p(px(t.spacing.sm))
        .flex()
        .flex_col()
        .gap(px(t.spacing.xs));
    if let Some(c) = caption {
        f = f.child(
            div()
                .text_size(px(t.typography.size_xs))
                .text_color(hsla(t.palette.text_muted))
                .font_family(first_family(t.typography.mono))
                .child(format_label(&t.typography, &c)),
        );
    }
    f.max_w_full()
        .child(div().relative().w(px(w)).max_w_full().h(px(h)).child(body))
}

/// Per-design drawing parameters of the scope family.
pub(super) struct Look {
    line: f32,
    grid: f32,
    minor: bool,
    fill: f32,
    trail: f32,
    solid: bool,
    ring: f32,
}

pub(super) fn look(t: &Theme) -> Look {
    use dodai_core::ScopeLook;
    let (line, grid, minor, fill, trail, solid, ring) = match t.chart_style().scope {
        ScopeLook::Neon => (2.0, 0.55, true, 0.0, 1.0, false, 1.0),
        ScopeLook::Frosted => (2.5, 0.3, false, 0.16, 0.6, false, 1.0),
        ScopeLook::Clean => (2.5, 0.35, true, 0.1, 0.0, true, 1.0),
        ScopeLook::Ink => (4.0, 1.0, false, 0.0, 0.0, true, 2.0),
    };
    Look {
        line,
        grid,
        minor,
        fill,
        trail,
        solid,
        ring,
    }
}

pub(super) fn grid_color(t: &Theme) -> Hsla {
    scale_alpha(hsla(t.palette.border), look(t).grid)
}

// ────────────────────────────── Oscilloscope ──────────────────────────────

/// Phosphor-style trace over a divided graticule. Samples are `-1..=1`.
#[derive(IntoElement)]
pub struct Oscilloscope {
    stream: Option<dodai_gpui::live::Live>,
    trail: f32,
    glow: ChartGlow,
    samples: Vec<f32>,
    w: f32,
    h: f32,
    live: bool,
    color: Option<Color>,
    caption: Option<SharedString>,
}

impl Oscilloscope {
    /// Strength of the faded afterimage trailing the live trace: `0` none,
    /// `1` the default, up to `2` for a longer, stronger smear.
    pub fn trail(mut self, amount: f32) -> Self {
        self.trail = amount.clamp(0.0, 2.0);
        self
    }
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// Create a new [`Oscilloscope`].
    pub fn new(samples: impl IntoIterator<Item = f32>) -> Self {
        Self {
            stream: None,
            trail: 1.0,
            glow: ChartGlow::Theme,
            samples: samples.into_iter().collect(),
            w: 320.0,
            h: 160.0,
            live: false,
            color: None,
            caption: None,
        }
    }
    /// Heartbeat trace: `beats` periods over `n` samples.
    #[cfg(feature = "medical")]
    pub fn ecg(beats: usize, n: usize) -> Self {
        let beats = beats.max(1) as f32;
        Self::new((0..n.max(2)).map(|i| ecg_beat((i as f32 / n.max(2) as f32 * beats).fract())))
    }
    /// A sum of sines (`(cycles, amplitude)` partials) sampled `n` times.
    pub fn wave(partials: &[(f32, f32)], n: usize) -> Self {
        let norm_amp: f32 = partials.iter().map(|p| p.1.abs()).sum::<f32>().max(1e-3);
        Self::new((0..n.max(2)).map(|i| {
            let x = i as f32 / n.max(2) as f32;
            partials
                .iter()
                .map(|&(f, a)| a * (x * f * std::f32::consts::TAU).sin())
                .sum::<f32>()
                / norm_amp
        }))
    }
    /// Width and height in px.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.w = w;
        self.h = h;
        self
    }
    /// Draws a [`dodai_gpui::Live`] window (samples `-1..=1`) that slides left between samples,
    /// newest at the right edge.
    pub fn stream(mut self, window: &dodai_gpui::live::Live) -> Self {
        self.samples = window.values();
        self.stream = Some(window.clone());
        self
    }
    /// Scroll the (periodic) trace and leave a fading phosphor trail.
    pub fn live(mut self, on: bool) -> Self {
        self.live = on;
        self
    }
    /// Colour (default: the theme's).
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    /// Caption text.
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
}

impl RenderOnce for Oscilloscope {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let scroll = self.stream.as_ref().map(dodai_gpui::live::Live::scroll);
        if scroll.is_some() {
            dodai_gpui::follow::next_frame(window);
        }
        let col = hsla(self.color.unwrap_or(t.palette.primary));
        let (gw, gh) = (self.w, self.h);
        let samples = self.samples;
        let live = self.live;
        let lk = look(&t);
        let trail = self.trail * lk.trail;
        let glow = self.glow.resolve(&t);
        let grid = grid_color(&t);
        let bright = scale_alpha(hsla(t.palette.text_muted), 0.6);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let (mut minor, mut major) = (Vec::new(), Vec::new());
                for i in 0..=10 {
                    let x = gw * i as f32 / 10.0;
                    let l = vec![(x, 0.0), (x, gh)];
                    if i == 5 {
                        major.push(l)
                    } else if lk.minor {
                        minor.push(l)
                    }
                }
                for i in 0..=8 {
                    let y = gh * i as f32 / 8.0;
                    let l = vec![(0.0, y), (gw, y)];
                    if i == 4 {
                        major.push(l)
                    } else if lk.minor {
                        minor.push(l)
                    }
                }
                stroke_batch(w, o, &minor, 1.0, grid, false);
                stroke_batch(w, o, &major, lk.ring, bright, false);
                let n = samples.len();
                if n < 2 {
                    return;
                }
                let trace = |lag: f32| -> Vec<(f32, f32)> {
                    let shift = if live {
                        (clock() - lag) * 0.35 * n as f32
                    } else {
                        0.0
                    };
                    (0..n)
                        .map(|i| {
                            let j = ((i as f32 + shift).rem_euclid(n as f32)) as usize % n;
                            (
                                gw * (i as f32 - scroll.unwrap_or(0.0)) / (n - 1) as f32,
                                gh / 2.0 - samples[j].clamp(-1.0, 1.0) * gh * 0.45,
                            )
                        })
                        .collect()
                };
                if live {
                    for g in (1..=3).rev().filter(|_| trail > 0.0) {
                        stroke(
                            w,
                            o,
                            &trace(g as f32 * 0.06 * trail),
                            2.0,
                            scale_alpha(col, (0.16 / g as f32 * 2.0 * trail).min(0.9)),
                            false,
                        );
                    }
                    dodai_gpui::follow::next_frame(w);
                }
                let main = trace(0.0);
                w.with_content_mask(Some(gpui::ContentMask { bounds: b }), |w| {
                    if lk.fill > 0.0 {
                        let mut area = main.clone();
                        area.push((main.last().map_or(gw, |q| q.0), gh));
                        area.push((main.first().map_or(0.0, |q| q.0), gh));
                        fill(w, o, &area, scale_alpha(col, lk.fill));
                    }
                    glow_stroke(w, o, &main, lk.line, col, glow);
                });
            },
        )
        .absolute()
        .size_full();
        frame(&t, gw, gh, self.caption, cv)
    }
}

// ─────────────────────────────── Spectrum ───────────────────────────────

/// Equaliser-style bars built from stacked LED cells with peak caps.
#[derive(IntoElement)]
pub struct Spectrum {
    follow: dodai_gpui::follow::Follow,
    glow: ChartGlow,
    values: Vec<f32>,
    w: f32,
    h: f32,
    cells: usize,
    live: bool,
    zones: bool,
    caption: Option<SharedString>,
}

impl Spectrum {
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: ChartGlow) -> Self {
        self.glow = g;
        self
    }
    dodai_gpui::follow_api!();
    /// Values are `0..=1`; a changed value glides when `live` is off.
    #[track_caller]
    pub fn new(values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            glow: ChartGlow::Theme,
            values: values.into_iter().collect(),
            w: 320.0,
            h: 160.0,
            cells: 14,
            live: false,
            zones: true,
            caption: None,
        }
    }
    /// Width and height in px.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.w = w;
        self.h = h;
        self
    }
    /// Cells per bar (default 14).
    pub fn cells(mut self, n: usize) -> Self {
        self.cells = n.max(2);
        self
    }
    /// Bounce the bars (and hold a decaying peak cap).
    pub fn live(mut self, on: bool) -> Self {
        self.live = on;
        self
    }
    /// Green → amber → red by height (default on); off uses the primary colour.
    pub fn zones(mut self, on: bool) -> Self {
        self.zones = on;
        self
    }
    /// Caption text.
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
}

impl RenderOnce for Spectrum {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let chamfered = t.shape.kind() == dodai_core::ShapeKind::Chamfered;
        let (gw, gh, cells, live) = (self.w, self.h, self.cells, self.live);
        let values = if self.live {
            self.values
        } else {
            dodai_gpui::follow::follow_all(window, cx, &self.follow, &self.values)
        };
        let zc = self.zones.then(|| {
            [
                hsla(t.palette.success),
                hsla(t.palette.warning),
                hsla(t.palette.danger),
            ]
        });
        let primary = hsla(t.palette.primary);
        let dim = scale_alpha(hsla(t.palette.border), 0.7);
        let glow = self.glow.resolve(&t);
        let solid = look(&t).solid;
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let n = values.len();
                if n == 0 {
                    return;
                }
                let gap = 3.0;
                let bw = ((gw - gap * (n as f32 - 1.0)) / n as f32).max(2.0);
                let cg = if solid { 0.0 } else { 2.0 };
                let ch = (gh - cg * (cells as f32 - 1.0)) / cells as f32;
                let cut = if chamfered {
                    (ch * 0.5).min(bw * 0.4)
                } else {
                    0.0
                };
                let now = clock();
                // one bucket per colour: [zone 0..3] lit, [zone 0..3] peak caps, dim
                let mut lit: [Vec<Vec<(f32, f32)>>; 3] = Default::default();
                let mut caps: [Vec<Vec<(f32, f32)>>; 3] = Default::default();
                let mut off: Vec<Vec<(f32, f32)>> = Vec::new();
                for (i, &v0) in values.iter().enumerate() {
                    let (v, peak) = if live {
                        let v = jitter(v0, i, now);
                        let p = v
                            .max(jitter(v0, i, now - 0.15))
                            .max(jitter(v0, i, now - 0.3))
                            .max(jitter(v0, i, now - 0.45));
                        (v, p)
                    } else {
                        (v0.clamp(0.0, 1.0), v0.clamp(0.0, 1.0))
                    };
                    let x = i as f32 * (bw + gap);
                    let peak_cell = ((peak * cells as f32).ceil() as usize).clamp(1, cells) - 1;
                    for c in 0..cells {
                        let y = gh - (c as f32 + 1.0) * ch - c as f32 * cg;
                        let z = if zc.is_some() {
                            zone((c as f32 + 0.5) / cells as f32)
                        } else {
                            0
                        };
                        let poly = chamfer_rect(x, y, bw, ch, cut).to_vec();
                        if crate::gauges::cell_lit(c, cells, v) >= 0.5 {
                            lit[z].push(poly);
                        } else if c == peak_cell && peak > 0.0 {
                            caps[z].push(poly);
                        } else {
                            off.push(poly);
                        }
                    }
                }
                if !solid {
                    fill_batch(w, b.origin, &off, scale_alpha(dim, 0.5));
                }
                for z in 0..3 {
                    let col = zc.map_or(primary, |c| c[z]);
                    if glow.is_some() && !lit[z].is_empty() {
                        // one soft halo pass for the whole colour, not per cell
                        let halo: Vec<_> = lit[z]
                            .iter()
                            .map(|p| {
                                chamfer_rect(p[0].0 - 2.0, p[0].1 - 2.0, bw + 4.0, ch + 4.0, cut)
                                    .to_vec()
                            })
                            .collect();
                        fill_batch(w, b.origin, &halo, scale_alpha(col, 0.16));
                    }
                    fill_batch(w, b.origin, &lit[z], scale_alpha(col, 0.95));
                    fill_batch(w, b.origin, &caps[z], col);
                }
                if live {
                    dodai_gpui::follow::next_frame(w);
                }
            },
        )
        .absolute()
        .size_full();
        frame(&t, gw, gh, self.caption, cv)
    }
}

// ────────────────────────────── DotMatrix ──────────────────────────────

/// Bar chart drawn as columns of LED dots (`0..=1` values).
#[derive(IntoElement)]
pub struct DotMatrix {
    follow: dodai_gpui::follow::Follow,
    stream: Option<dodai_gpui::live::Live>,
    glow: ChartGlow,
    values: Vec<f32>,
    rows: usize,
    w: f32,
    caption: Option<SharedString>,
}

impl DotMatrix {
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// Create a new [`DotMatrix`]; changed values glide.
    #[track_caller]
    pub fn new(values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            stream: None,
            glow: ChartGlow::Theme,
            values: values.into_iter().collect(),
            rows: 8,
            w: 320.0,
            caption: None,
        }
    }
    dodai_gpui::follow_api!();
    /// Shows the newest samples of a [`Live`](dodai_gpui::live::Live) window, one column each, with
    /// `max` as the full-height sample value.
    pub fn stream(mut self, window: &dodai_gpui::live::Live, max: f32) -> Self {
        let max = if max > 0.0 { max } else { 1.0 };
        self.values = window.values().into_iter().map(|v| v / max).collect();
        self.stream = Some(window.clone());
        self
    }
    /// Dots per column (default 8).
    pub fn rows(mut self, r: usize) -> Self {
        self.rows = r.max(2);
        self
    }
    /// Width in px.
    pub fn width(mut self, w: f32) -> Self {
        self.w = w;
        self
    }
    /// Caption text.
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
}

impl RenderOnce for DotMatrix {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow(window, cx, "DotMatrix");
        let values = if self.stream.is_some() {
            dodai_gpui::follow::next_frame(window);
            self.values
        } else {
            dodai_gpui::follow::follow_all(window, cx, &self.follow, &self.values)
        };
        let n = values.len().max(1);
        let pitch = self.w / n as f32;
        let h = pitch * self.rows as f32;
        let rows = self.rows;
        let on = hsla(t.palette.primary);
        let dim = scale_alpha(hsla(t.palette.border), 0.7);
        let glow = self.glow.resolve(&t);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let r = pitch * 0.3;
                let (mut lit, mut soft, mut off, mut halo) =
                    (Vec::new(), Vec::new(), Vec::new(), Vec::new());
                for (i, v) in values.iter().enumerate() {
                    let f = crate::draw::norm(*v * g, 0.0, 1.0);
                    for row in 0..rows {
                        let cov = crate::gauges::cell_lit(row, rows, f);
                        let (x, y) = (pitch * (i as f32 + 0.5), h - pitch * (row as f32 + 0.5));
                        if cov >= 0.99 {
                            if let Some(gr) = glow {
                                halo.push(disc(x, y, r + gr * 0.25, 10));
                            }
                            lit.push(disc(x, y, r, 10));
                        } else if cov > 0.0 {
                            soft.push(disc(x, y, r, 10));
                        } else {
                            off.push(disc(x, y, r * 0.7, 8));
                        }
                    }
                }
                fill_batch(w, b.origin, &off, dim);
                fill_batch(w, b.origin, &halo, scale_alpha(on, 0.18));
                fill_batch(w, b.origin, &soft, scale_alpha(on, 0.6));
                fill_batch(w, b.origin, &lit, on);
            },
        )
        .absolute()
        .size_full();
        frame(&t, self.w, h, self.caption, cv)
    }
}

// ─────────────────────────────── SignalBars ───────────────────────────────

/// Ascending signal-strength bars; `level` of `bars` are lit.
#[derive(IntoElement)]
pub struct SignalBars {
    glow: ChartGlow,
    level: usize,
    bars: usize,
    height: f32,
}

impl SignalBars {
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// Create a new [`SignalBars`].
    pub fn new(level: usize) -> Self {
        Self {
            glow: ChartGlow::Theme,
            level,
            bars: 5,
            height: 28.0,
        }
    }
    /// Number of bars.
    pub fn bars(mut self, n: usize) -> Self {
        self.bars = n.max(1);
        self
    }
    /// Height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
}

impl RenderOnce for SignalBars {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let chamfered = t.shape.kind() == dodai_core::ShapeKind::Chamfered;
        let (n, level, h) = (self.bars, self.level, self.height);
        let bw = h * 0.22;
        let gap = bw * 0.5;
        let w_total = n as f32 * bw + (n as f32 - 1.0) * gap;
        let on = hsla(t.palette.primary);
        let dim = scale_alpha(hsla(t.palette.border), 0.8);
        let glow = self.glow.resolve(&t);
        canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                for i in 0..n {
                    let bh = h * (i as f32 + 1.0) / n as f32;
                    let x = i as f32 * (bw + gap);
                    let poly =
                        chamfer_rect(x, h - bh, bw, bh, if chamfered { bw * 0.45 } else { 0.0 });
                    if i < level {
                        if let Some(r) = glow {
                            let big = chamfer_rect(
                                x - 2.0,
                                h - bh - 2.0,
                                bw + 4.0,
                                bh + 4.0,
                                if chamfered { bw * 0.45 } else { 0.0 },
                            );
                            fill(w, b.origin, &big, scale_alpha(on, 0.12 + r * 0.004));
                        }
                        fill(w, b.origin, &poly, on);
                    } else {
                        fill(w, b.origin, &poly, dim);
                    }
                }
            },
        )
        .w(px(w_total))
        .h(px(h))
    }
}

mod hex;
#[cfg(feature = "defense")]
mod radar;
pub use hex::{HexMap, RingStack};
#[cfg(feature = "defense")]
pub use radar::{Blip, RadarScope};

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "medical")]
    #[test]
    fn ecg_beat_has_one_dominant_spike_per_period() {
        let peak = (0..200)
            .map(|i| ecg_beat(i as f32 / 200.0))
            .fold(f32::MIN, f32::max);
        assert!(peak > 0.9 && peak <= 1.1);
        assert!(ecg_beat(0.0).abs() < 0.05 && ecg_beat(0.95).abs() < 0.05);
        assert_eq!(Oscilloscope::ecg(2, 100).samples.len(), 100);
    }

    #[test]
    fn wave_is_normalised() {
        let o = Oscilloscope::wave(&[(2.0, 1.0), (5.0, 0.5)], 64);
        assert_eq!(o.samples.len(), 64);
        assert!(o.samples.iter().all(|s| s.abs() <= 1.0 + 1e-5));
    }
}
