//! Instrument-style charts: [`Oscilloscope`], [`Spectrum`], [`RadarScope`],
//! [`HexMap`] and [`RingStack`]. They sit in a themed panel frame, glow when
//! the theme does and take every colour from the palette. The `live` ones
//! animate from a shared clock (and only then ask for frames).

use crate::charts::{fmt_num, glow_of, glow_stroke, norm, polar, turn_arc};
use crate::gauges::{chamfer_rect, disc, zone, SWEEP};
use crate::paint::{fill, scale_alpha, shaped, stroke};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::{Color, Theme};
use gpui::{
    canvas, div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window,
};
use std::sync::OnceLock;
use std::time::Instant;

// ───────────────────────────── pure helpers ─────────────────────────────

/// Seconds since the first call (shared clock for live charts).
pub(crate) fn clock() -> f32 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f32()
}

/// One heartbeat-shaped period (`-1..=1`): P wave, QRS spike, T wave. Pure.
pub fn ecg_beat(x: f32) -> f32 {
    let g = |c: f32, w: f32, a: f32| a * (-((x - c) / w).powi(2)).exp();
    g(0.18, 0.04, 0.18)
        + g(0.36, 0.012, -0.22)
        + g(0.39, 0.012, 1.0)
        + g(0.42, 0.012, -0.35)
        + g(0.62, 0.06, 0.28)
}

/// Brightness `0..=1` of a radar blip at turn-fraction `blip` when the sweep is
/// at `sweep`: full as the beam passes, fading over one revolution.
pub fn blip_alpha(sweep: f32, blip: f32) -> f32 {
    let since = (sweep - blip).rem_euclid(1.0);
    (1.0 - since).powi(2)
}

/// Centre of hex `(row, col)` for a pointy-top grid with circumradius `s`.
pub fn hex_center(row: usize, col: usize, s: f32) -> (f32, f32) {
    let w = 3f32.sqrt() * s;
    let off = if row % 2 == 1 { w / 2.0 } else { 0.0 };
    (w / 2.0 + col as f32 * w + off, s + row as f32 * 1.5 * s)
}

/// Six corners of a pointy-top hex of circumradius `r` around `(cx, cy)`.
pub fn hex_points(cx: f32, cy: f32, r: f32) -> [(f32, f32); 6] {
    let mut p = [(0.0, 0.0); 6];
    for (i, q) in p.iter_mut().enumerate() {
        *q = polar(cx, cy, r, i as f32 / 6.0);
    }
    p
}

/// Grid size `(width, height)` of `rows × cols` hexes of circumradius `s`.
pub fn hex_extent(rows: usize, cols: usize, s: f32) -> (f32, f32) {
    let w = 3f32.sqrt() * s;
    (w * (cols as f32 + 0.5), s * (1.5 * rows as f32 + 0.5))
}

/// Radius of ring `i` of `n` concentric rings, outermost first. Pure.
pub fn ring_radius(i: usize, outer: f32, thickness: f32, gap: f32) -> f32 {
    outer - thickness / 2.0 - i as f32 * (thickness + gap)
}

/// Animated bar value: `v` modulated by a per-bar sine at time `t`.
pub fn jitter(v: f32, i: usize, t: f32) -> f32 {
    let f = 1.7 + i as f32 * 0.37;
    (v * (0.65 + 0.35 * (t * f + i as f32).sin())).clamp(0.0, 1.0)
}

fn frame(
    t: &Theme,
    w: f32,
    h: f32,
    caption: Option<SharedString>,
    body: impl IntoElement,
) -> gpui::Div {
    let mut f = shaped(&t.shape, crate::panel::panel_style(t, false))
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
    f.child(div().relative().w(px(w)).h(px(h)).child(body))
}

fn grid_color(t: &Theme) -> Hsla {
    scale_alpha(hsla(t.palette.border), 0.55)
}

// ────────────────────────────── Oscilloscope ──────────────────────────────

/// Phosphor-style trace over a divided graticule. Samples are `-1..=1`.
#[derive(IntoElement)]
pub struct Oscilloscope {
    samples: Vec<f32>,
    w: f32,
    h: f32,
    live: bool,
    color: Option<Color>,
    caption: Option<SharedString>,
}

impl Oscilloscope {
    pub fn new(samples: impl IntoIterator<Item = f32>) -> Self {
        Self {
            samples: samples.into_iter().collect(),
            w: 320.0,
            h: 160.0,
            live: false,
            color: None,
            caption: None,
        }
    }
    /// Heartbeat trace: `beats` periods over `n` samples.
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
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.w = w;
        self.h = h;
        self
    }
    /// Scroll the (periodic) trace and leave a fading phosphor trail.
    pub fn live(mut self, on: bool) -> Self {
        self.live = on;
        self
    }
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
}

impl RenderOnce for Oscilloscope {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let col = hsla(self.color.unwrap_or(t.palette.primary));
        let (gw, gh) = (self.w, self.h);
        let samples = self.samples;
        let live = self.live;
        let glow = glow_of(&t);
        let grid = grid_color(&t);
        let bright = scale_alpha(hsla(t.palette.text_muted), 0.6);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                for i in 0..=10 {
                    let x = gw * i as f32 / 10.0;
                    let c = if i == 5 { bright } else { grid };
                    stroke(w, o, &[(x, 0.0), (x, gh)], 1.0, c, false);
                }
                for i in 0..=8 {
                    let y = gh * i as f32 / 8.0;
                    let c = if i == 4 { bright } else { grid };
                    stroke(w, o, &[(0.0, y), (gw, y)], 1.0, c, false);
                }
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
                                gw * i as f32 / (n - 1) as f32,
                                gh / 2.0 - samples[j].clamp(-1.0, 1.0) * gh * 0.45,
                            )
                        })
                        .collect()
                };
                if live {
                    for g in (1..=3).rev() {
                        stroke(
                            w,
                            o,
                            &trace(g as f32 * 0.06),
                            2.0,
                            scale_alpha(col, 0.16 / g as f32 * 2.0),
                            false,
                        );
                    }
                    w.request_animation_frame();
                }
                glow_stroke(w, o, &trace(0.0), 2.0, col, glow);
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
    values: Vec<f32>,
    w: f32,
    h: f32,
    cells: usize,
    live: bool,
    zones: bool,
    caption: Option<SharedString>,
}

impl Spectrum {
    /// Values are `0..=1`.
    pub fn new(values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            values: values.into_iter().collect(),
            w: 320.0,
            h: 160.0,
            cells: 14,
            live: false,
            zones: true,
            caption: None,
        }
    }
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
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
}

impl RenderOnce for Spectrum {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (gw, gh, cells, live) = (self.w, self.h, self.cells, self.live);
        let values = self.values;
        let zc = self.zones.then(|| {
            [
                hsla(t.palette.success),
                hsla(t.palette.warning),
                hsla(t.palette.danger),
            ]
        });
        let primary = hsla(t.palette.primary);
        let dim = scale_alpha(hsla(t.palette.border), 0.7);
        let glow = glow_of(&t);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let n = values.len();
                if n == 0 {
                    return;
                }
                let gap = 3.0;
                let bw = ((gw - gap * (n as f32 - 1.0)) / n as f32).max(2.0);
                let cg = 2.0;
                let ch = (gh - cg * (cells as f32 - 1.0)) / cells as f32;
                let now = clock();
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
                        let f = (c as f32 + 0.5) / cells as f32;
                        let col = zc.map_or(primary, |z| z[zone(f)]);
                        let cov = crate::gauges::cell_lit(c, cells, v);
                        let poly = chamfer_rect(x, y, bw, ch, (ch * 0.5).min(bw * 0.4));
                        if cov > 0.0 {
                            if let Some(r) = glow {
                                for l in crate::paint::glow_layers(r * 0.4, 0.0) {
                                    let g = l.width;
                                    let big = chamfer_rect(
                                        x - g / 2.0,
                                        y - g / 2.0,
                                        bw + g,
                                        ch + g,
                                        ch * 0.5,
                                    );
                                    fill(w, b.origin, &big, scale_alpha(col, l.alpha * 0.35));
                                }
                            }
                            fill(w, b.origin, &poly, scale_alpha(col, 0.5 + 0.5 * cov));
                        } else if c == peak_cell && peak > 0.0 {
                            fill(w, b.origin, &poly, scale_alpha(col, 0.95));
                        } else {
                            fill(w, b.origin, &poly, scale_alpha(dim, 0.5));
                        }
                    }
                }
                if live {
                    w.request_animation_frame();
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
    values: Vec<f32>,
    rows: usize,
    w: f32,
    caption: Option<SharedString>,
}

impl DotMatrix {
    pub fn new(values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            values: values.into_iter().collect(),
            rows: 8,
            w: 320.0,
            caption: None,
        }
    }
    /// Dots per column (default 8).
    pub fn rows(mut self, r: usize) -> Self {
        self.rows = r.max(2);
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.w = w;
        self
    }
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
}

impl RenderOnce for DotMatrix {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "DotMatrix");
        let n = self.values.len().max(1);
        let pitch = self.w / n as f32;
        let h = pitch * self.rows as f32;
        let (rows, values) = (self.rows, self.values);
        let on = hsla(t.palette.primary);
        let dim = scale_alpha(hsla(t.palette.border), 0.7);
        let glow = glow_of(&t);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let r = pitch * 0.3;
                for (i, v) in values.iter().enumerate() {
                    let lit = crate::charts::norm(*v * g, 0.0, 1.0);
                    for row in 0..rows {
                        let cov = crate::gauges::cell_lit(row, rows, lit);
                        let (x, y) = (pitch * (i as f32 + 0.5), h - pitch * (row as f32 + 0.5));
                        if cov > 0.0 {
                            if let Some(gr) = glow {
                                fill(
                                    w,
                                    b.origin,
                                    &disc(x, y, r + gr * 0.25, 10),
                                    scale_alpha(on, 0.18),
                                );
                            }
                            fill(
                                w,
                                b.origin,
                                &disc(x, y, r, 10),
                                scale_alpha(on, 0.55 + 0.45 * cov),
                            );
                        } else {
                            fill(w, b.origin, &disc(x, y, r * 0.7, 8), dim);
                        }
                    }
                }
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
    level: usize,
    bars: usize,
    height: f32,
}

impl SignalBars {
    pub fn new(level: usize) -> Self {
        Self {
            level,
            bars: 5,
            height: 28.0,
        }
    }
    pub fn bars(mut self, n: usize) -> Self {
        self.bars = n.max(1);
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
}

impl RenderOnce for SignalBars {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (n, level, h) = (self.bars, self.level, self.height);
        let bw = h * 0.22;
        let gap = bw * 0.5;
        let w_total = n as f32 * bw + (n as f32 - 1.0) * gap;
        let on = hsla(t.palette.primary);
        let dim = scale_alpha(hsla(t.palette.border), 0.8);
        let glow = glow_of(&t);
        canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                for i in 0..n {
                    let bh = h * (i as f32 + 1.0) / n as f32;
                    let x = i as f32 * (bw + gap);
                    let poly = chamfer_rect(x, h - bh, bw, bh, bw * 0.45);
                    if i < level {
                        if let Some(r) = glow {
                            let big =
                                chamfer_rect(x - 2.0, h - bh - 2.0, bw + 4.0, bh + 4.0, bw * 0.45);
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

// ────────────────────────────── RadarScope ──────────────────────────────

/// A contact on a [`RadarScope`]: `bearing` in turns (0 = up, clockwise) and
/// `range` as `0..=1` of the radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blip {
    pub bearing: f32,
    pub range: f32,
}

impl Blip {
    pub fn new(bearing: f32, range: f32) -> Self {
        Self { bearing, range }
    }
}

/// Sweeping radar scope with range rings and fading contacts.
#[derive(IntoElement)]
pub struct RadarScope {
    blips: Vec<Blip>,
    size: f32,
    live: bool,
    period: f32,
    caption: Option<SharedString>,
}

impl RadarScope {
    pub fn new(blips: impl IntoIterator<Item = Blip>) -> Self {
        Self {
            blips: blips.into_iter().collect(),
            size: 220.0,
            live: true,
            period: 4.0,
            caption: None,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Rotate the sweep (default on); off freezes it and shows all contacts.
    pub fn live(mut self, on: bool) -> Self {
        self.live = on;
        self
    }
    /// Seconds per revolution (default 4).
    pub fn period(mut self, s: f32) -> Self {
        self.period = s.max(0.5);
        self
    }
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
}

impl RenderOnce for RadarScope {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (size, live, period) = (self.size, self.live, self.period);
        let blips = self.blips;
        let col = hsla(t.palette.primary);
        let warn = hsla(t.palette.danger);
        let ring = grid_color(&t);
        let glow = glow_of(&t);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let c = size / 2.0;
                let r = c - 3.0;
                for i in 1..=4 {
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r * i as f32 / 4.0, 0.0, 1.0),
                        1.0,
                        ring,
                        true,
                    );
                }
                for q in 0..4 {
                    let a = q as f32 / 4.0;
                    stroke(
                        w,
                        o,
                        &[polar(c, c, 0.0, a), polar(c, c, r, a)],
                        1.0,
                        ring,
                        false,
                    );
                }
                for i in 0..36 {
                    let a = i as f32 / 36.0;
                    let len = if i % 3 == 0 { 7.0 } else { 3.5 };
                    stroke(
                        w,
                        o,
                        &[polar(c, c, r, a), polar(c, c, r - len, a)],
                        1.0,
                        scale_alpha(col, 0.55),
                        false,
                    );
                }
                let sweep = if live {
                    (clock() / period).fract()
                } else {
                    0.12
                };
                // fading fan behind the beam
                let fan = 28;
                for i in 0..fan {
                    let a0 = sweep - 0.22 * (i + 1) as f32 / fan as f32;
                    let a1 = sweep - 0.22 * i as f32 / fan as f32;
                    let tri = [(c, c), polar(c, c, r, a0), polar(c, c, r, a1)];
                    let k = 1.0 - i as f32 / fan as f32;
                    fill(w, o, &tri, scale_alpha(col, 0.28 * k * k));
                }
                glow_stroke(w, o, &[(c, c), polar(c, c, r, sweep)], 2.0, col, glow);
                for bl in &blips {
                    let a = if live {
                        blip_alpha(sweep, bl.bearing)
                    } else {
                        1.0
                    };
                    if a < 0.04 {
                        continue;
                    }
                    let p = polar(c, c, r * bl.range.clamp(0.0, 1.0), bl.bearing);
                    fill(w, o, &disc(p.0, p.1, 4.0, 10), scale_alpha(warn, a));
                    stroke(
                        w,
                        o,
                        &disc(p.0, p.1, 7.0 + 5.0 * (1.0 - a), 14),
                        1.0,
                        scale_alpha(warn, a * 0.7),
                        true,
                    );
                }
                fill(w, o, &disc(c, c, 2.5, 8), col);
                if live {
                    w.request_animation_frame();
                }
            },
        )
        .absolute()
        .size_full();
        frame(&t, size, size, self.caption, cv)
    }
}

// ──────────────────────────────── HexMap ────────────────────────────────

/// Honeycomb heat map: values `0..=1`, row-major, `cols` per row.
#[derive(IntoElement)]
pub struct HexMap {
    values: Vec<f32>,
    cols: usize,
    w: f32,
    live: bool,
    caption: Option<SharedString>,
}

impl HexMap {
    pub fn new(cols: usize, values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            values: values.into_iter().collect(),
            cols: cols.max(1),
            w: 320.0,
            live: false,
            caption: None,
        }
    }
    /// Total width in px; the hex size follows.
    pub fn width(mut self, w: f32) -> Self {
        self.w = w;
        self
    }
    /// Pulse the cells.
    pub fn live(mut self, on: bool) -> Self {
        self.live = on;
        self
    }
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
}

impl RenderOnce for HexMap {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let cols = self.cols;
        let rows = self.values.len().div_ceil(cols);
        let s = self.w / (3f32.sqrt() * (cols as f32 + 0.5));
        let (gw, gh) = hex_extent(rows.max(1), cols, s);
        let (values, live) = (self.values, self.live);
        let base = t.palette.surface_raised;
        let hot = t.palette.primary;
        let hi = t.palette.danger;
        let edge = scale_alpha(hsla(t.palette.border), 0.8);
        let glow = glow_of(&t);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let now = clock();
                for (i, &v0) in values.iter().enumerate() {
                    let (row, col) = (i / cols, i % cols);
                    let v = if live {
                        (v0 * (0.75 + 0.25 * (now * 1.3 + i as f32 * 0.9).sin())).clamp(0.0, 1.0)
                    } else {
                        v0.clamp(0.0, 1.0)
                    };
                    let (cx_, cy_) = hex_center(row, col, s);
                    let fill_c = hsla(base.mix(if v > 0.8 { hi } else { hot }, v));
                    let inner = hex_points(cx_, cy_, s * 0.9);
                    if v > 0.05 {
                        if v > 0.7 {
                            if let Some(r) = glow {
                                for l in crate::paint::glow_layers(r * 0.5, 0.0) {
                                    let big = hex_points(cx_, cy_, s * 0.9 + l.width / 2.0);
                                    fill(w, b.origin, &big, scale_alpha(fill_c, l.alpha * 0.4));
                                }
                            }
                        }
                        fill(w, b.origin, &inner, fill_c);
                    }
                    stroke(w, b.origin, &inner, 1.0, edge, true);
                }
                if live {
                    w.request_animation_frame();
                }
            },
        )
        .absolute()
        .size_full();
        frame(&t, gw, gh, self.caption, cv)
    }
}

// ─────────────────────────────── RingStack ───────────────────────────────

/// Concentric 270° rings, one per `(label, percent)`, with a legend.
#[derive(IntoElement)]
pub struct RingStack {
    items: Vec<(SharedString, f32)>,
    size: f32,
}

impl RingStack {
    pub fn new<S: Into<SharedString>>(items: impl IntoIterator<Item = (S, f32)>) -> Self {
        Self {
            items: items.into_iter().map(|(l, v)| (l.into(), v)).collect(),
            size: 150.0,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
}

impl RenderOnce for RingStack {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "RingStack");
        let size = self.size;
        let n = self.items.len().max(1);
        let thick = (size / 2.0 / (n as f32 + 1.5)).clamp(4.0, 12.0);
        let gap = thick * 0.45;
        let fr: Vec<f32> = self
            .items
            .iter()
            .map(|i| norm(i.1, 0.0, 100.0) * g)
            .collect();
        let cols: Vec<Hsla> = (0..n).map(|i| hsla(t.palette.series_at(i))).collect();
        let (track, glow) = (scale_alpha(hsla(t.palette.border), 0.8), glow_of(&t));
        let (fr2, cols2) = (fr.clone(), cols.clone());
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let c = size / 2.0;
                let (s0, span) = SWEEP;
                for (i, f) in fr2.iter().enumerate() {
                    let r = ring_radius(i, c - 2.0, thick, gap);
                    stroke(
                        w,
                        b.origin,
                        &turn_arc(c, c, r, s0, s0 + span),
                        thick,
                        track,
                        false,
                    );
                    if *f > 0.0 {
                        glow_stroke(
                            w,
                            b.origin,
                            &turn_arc(c, c, r, s0, s0 + span * f),
                            thick,
                            cols2[i],
                            glow,
                        );
                    }
                }
            },
        )
        .absolute()
        .size_full();
        let legend = self.items.iter().enumerate().map(|(i, (l, v))| {
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(div().w(px(10.0)).h(px(10.0)).bg(cols[i]))
                .child(
                    div()
                        .text_size(px(t.typography.size_sm))
                        .text_color(hsla(t.palette.text_muted))
                        .font_family(first_family(t.typography.mono))
                        .child(format_label(&t.typography, l)),
                )
                .child(
                    div()
                        .text_size(px(t.typography.size_sm))
                        .text_color(hsla(t.palette.text))
                        .font_family(first_family(t.typography.display))
                        .child(format!("{}%", fmt_num((*v * g).round()))),
                )
        });
        div()
            .flex()
            .items_center()
            .gap(px(t.spacing.lg))
            .child(div().relative().w(px(size)).h(px(size)).child(cv))
            .child(div().flex().flex_col().gap(px(6.0)).children(legend))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blip_is_brightest_at_the_beam_and_fades_after() {
        assert!((blip_alpha(0.3, 0.3) - 1.0).abs() < 1e-6);
        assert!(blip_alpha(0.5, 0.3) < blip_alpha(0.4, 0.3));
        // a blip just *ahead* of the beam has had a whole revolution to fade
        assert!(blip_alpha(0.3, 0.31) < 0.01);
    }

    #[test]
    fn hex_rows_interlock() {
        let s = 10.0;
        let (x0, y0) = hex_center(0, 0, s);
        let (x1, y1) = hex_center(1, 0, s);
        assert!((x1 - x0 - 3f32.sqrt() * s / 2.0).abs() < 1e-4);
        assert!((y1 - y0 - 1.5 * s).abs() < 1e-4);
    }

    #[test]
    fn hex_has_six_equal_corners() {
        let p = hex_points(0.0, 0.0, 5.0);
        for (x, y) in p {
            assert!(((x * x + y * y).sqrt() - 5.0).abs() < 1e-4);
        }
    }

    #[test]
    fn hex_extent_covers_every_centre() {
        let (w, h) = hex_extent(3, 4, 10.0);
        for r in 0..3 {
            for c in 0..4 {
                let (x, y) = hex_center(r, c, 10.0);
                assert!(x + 3f32.sqrt() * 5.0 <= w + 1e-3 && y + 10.0 <= h + 1e-3);
            }
        }
    }

    #[test]
    fn rings_shrink_inward_without_overlap() {
        let a = ring_radius(0, 50.0, 8.0, 3.0);
        let b = ring_radius(1, 50.0, 8.0, 3.0);
        assert!((a - b - 11.0).abs() < 1e-4);
        assert!(a + 4.0 <= 50.0);
    }

    #[test]
    fn jitter_stays_in_range_and_below_the_value() {
        for i in 0..8 {
            for k in 0..50 {
                let v = jitter(0.8, i, k as f32 * 0.1);
                assert!((0.0..=0.8 + 1e-6).contains(&v));
            }
        }
    }

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
