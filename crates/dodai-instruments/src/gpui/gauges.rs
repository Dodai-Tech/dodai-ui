//! Gauge styles for [`crate::gpui::Gauge`]: HUD-like dials, segmented rings, a
//! reticle and a slanted cell bar. Geometry is pure and unit-tested; colours,
//! glow and fonts all come from the theme.

use crate::gpui::draw::{glow_stroke, turn_arc};
use dodai_gpui::paint::{fill, fill_circle, scale_alpha, stroke, stroke_circle};
use gpui::{Bounds, Hsla, Pixels, Window};

/// How a [`crate::gpui::Gauge`] draws its value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GaugeStyle {
    /// Continuous 270° arc (default).
    #[default]
    Arc,
    /// 270° ring of separate blocks; the last lit one may be partial.
    Segmented,
    /// Speedometer: tick ring, value arc, tapered needle and hub.
    Dial,
    /// Top half-dial: tick ring and a thick value arc.
    Meter,
    /// Full ring with cardinal ticks, a bright head and a dashed inner ring.
    Reticle,
    /// Horizontal row of slanted cells.
    Bar,
    /// Horizontal row of arrow-head cells (›››).
    Chevron,
    /// Vertical stack of chamfered cells, filling upwards.
    Level,
    /// Full 360° ring from 12 o'clock with round caps.
    Ring,
    /// 270° arc that fades in towards a bright head (comet).
    Smooth,
    /// 270° sweep of round dots.
    Dots,
    /// Row of round LEDs, coloured by zone.
    Led,
    /// Battery outline with a charge fill; turns red when low.
    Battery,
    /// Thermometer: tube, bulb and tick marks.
    Thermometer,
    /// Five ascending signal-strength bars.
    Signal,
}

impl GaugeStyle {
    /// Every gauge style, in showcase order.
    pub const ALL: [GaugeStyle; 15] = [
        GaugeStyle::Arc,
        GaugeStyle::Segmented,
        GaugeStyle::Dial,
        GaugeStyle::Meter,
        GaugeStyle::Reticle,
        GaugeStyle::Bar,
        GaugeStyle::Chevron,
        GaugeStyle::Level,
        GaugeStyle::Ring,
        GaugeStyle::Smooth,
        GaugeStyle::Dots,
        GaugeStyle::Led,
        GaugeStyle::Battery,
        GaugeStyle::Thermometer,
        GaugeStyle::Signal,
    ];
    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            GaugeStyle::Arc => "Arc",
            GaugeStyle::Segmented => "Segmented",
            GaugeStyle::Dial => "Dial",
            GaugeStyle::Meter => "Meter",
            GaugeStyle::Reticle => "Reticle",
            GaugeStyle::Bar => "Bar",
            GaugeStyle::Chevron => "Chevron",
            GaugeStyle::Level => "Level",
            GaugeStyle::Ring => "Ring",
            GaugeStyle::Smooth => "Smooth",
            GaugeStyle::Dots => "Dots",
            GaugeStyle::Led => "LED",
            GaugeStyle::Battery => "Battery",
            GaugeStyle::Thermometer => "Thermometer",
            GaugeStyle::Signal => "Signal",
        }
    }
    /// `(width, height)` for a gauge of nominal `size`.
    pub fn dims(self, size: f32) -> (f32, f32) {
        match self {
            GaugeStyle::Meter => (size, size * 0.62),
            GaugeStyle::Bar | GaugeStyle::Chevron => (size * 1.6, size * 0.2),
            GaugeStyle::Level => (size * 0.34, size),
            GaugeStyle::Led => (size * 1.6, size * 0.14),
            GaugeStyle::Battery => (size * 1.1, size * 0.5),
            GaugeStyle::Thermometer => (size * 0.3, size),
            GaugeStyle::Signal => (size * 0.6, size * 0.5),
            _ => (size, size),
        }
    }
}

pub(crate) use crate::gauge::SWEEP;

// ───────────────────────────── pure helpers ─────────────────────────────

pub use crate::gauge::{
    cell_lit, chamfer_rect, chevron_cell, needle, pill, slanted_cell, tick_fracs, zone,
};

#[allow(clippy::too_many_arguments)]
/// Stroke an arc with round caps (butt-capped paths plus a disc at each end).
pub(crate) fn round_arc(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    c: f32,
    r: f32,
    a0: f32,
    a1: f32,
    width: f32,
    col: Hsla,
) {
    stroke(w, o, &turn_arc(c, c, r, a0, a1), width, col, false);
    for a in [a0, a1] {
        let (x, y) = crate::gpui::draw::polar(c, c, r, a);
        fill_circle(w, o, (x, y), width / 2.0, col);
    }
}

// ─────────────────────────────── painting ───────────────────────────────

/// Resolved colours and options for one paint.
#[derive(Clone, Copy)]
pub(crate) struct Look {
    pub style: GaugeStyle,
    pub frac: f32,
    pub size: f32,
    pub ring: f32,
    pub tone: Hsla,
    pub track: Hsla,
    pub text: Hsla,
    /// `Some([ok, warn, danger])` colours the value by zone.
    pub zones: Option<[Hsla; 3]>,
    pub glow: Option<f32>,
    /// Colour for "bad" states (low battery).
    pub alert: Hsla,
    /// Target position `0..=1`, drawn as a marker.
    pub target: Option<f32>,
    /// Rounded (Material-like) look: round caps, pills, no ticks or glow.
    pub soft: bool,
}

impl Look {
    pub(crate) fn at(&self, frac: f32) -> Hsla {
        self.zones.map_or(self.tone, |z| z[zone(frac)])
    }
}

pub(crate) fn paint(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    paint_style(w, b, k);
    target_mark(w, b, k);
}

fn paint_style(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    use crate::gpui::gauges_extra as x;
    match k.style {
        GaugeStyle::Ring => return x::ring(w, b, k),
        GaugeStyle::Smooth => return x::smooth(w, b, k),
        GaugeStyle::Dots => return x::dots(w, b, k),
        GaugeStyle::Led => return x::led(w, b, k),
        GaugeStyle::Battery => return x::battery(w, b, k),
        GaugeStyle::Thermometer => return x::thermometer(w, b, k),
        GaugeStyle::Signal => return x::signal(w, b, k),
        _ => {}
    }
    if k.soft && k.style != GaugeStyle::Arc {
        return soft(w, b, k);
    }
    match k.style {
        GaugeStyle::Arc => {}
        GaugeStyle::Segmented => segmented(w, b, k),
        GaugeStyle::Dial => dial(w, b, k, SWEEP.0, SWEEP.1, true),
        GaugeStyle::Meter => dial(w, b, k, 0.75, 0.5, false),
        GaugeStyle::Reticle => reticle(w, b, k),
        GaugeStyle::Bar | GaugeStyle::Chevron => bar(w, b, k),
        GaugeStyle::Level => level(w, b, k),
        _ => {}
    }
}

/// Material-flavoured variants: the same value, drawn as rounded progress
/// indicators (round-capped arcs, pill bars) with no ticks, glow or chamfers.
fn soft(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let o = b.origin;
    let c = k.size / 2.0;
    let track = scale_alpha(k.track, 0.8);
    let cap = k.ring.min(10.0);
    match k.style {
        GaugeStyle::Arc => {}
        GaugeStyle::Segmented | GaugeStyle::Dial | GaugeStyle::Meter | GaugeStyle::Reticle => {
            let (s0, span, r) = match k.style {
                GaugeStyle::Meter => (0.75, 0.5, c - cap / 2.0 - 2.0),
                GaugeStyle::Reticle => (0.0, 1.0, c - cap / 2.0 - 2.0),
                _ => (SWEEP.0, SWEEP.1, c - cap / 2.0 - 2.0),
            };
            if k.style == GaugeStyle::Segmented {
                // a handful of rounded segments with visible gaps
                let n = 12;
                for i in 0..n {
                    let a0 = s0 + span * (i as f32 + 0.12) / n as f32;
                    let a1 = s0 + span * (i as f32 + 0.88) / n as f32;
                    let cov = cell_lit(i, n, k.frac);
                    let col = if cov > 0.0 {
                        scale_alpha(k.at((i as f32 + 0.5) / n as f32), 0.45 + 0.55 * cov)
                    } else {
                        track
                    };
                    round_arc(w, o, c, r, a0, a1, cap * 0.8, col);
                }
            } else {
                round_arc(w, o, c, r, s0, s0 + span, cap, track);
                if k.frac > 0.0 {
                    round_arc(w, o, c, r, s0, s0 + span * k.frac, cap, k.at(k.frac));
                }
                if k.style == GaugeStyle::Dial {
                    // slim needle with a round hub
                    let tip = crate::gpui::draw::polar(c, c, r - cap, s0 + span * k.frac);
                    stroke(w, o, &[(c, c), tip], 2.5, k.text, false);
                    fill_circle(w, o, (c, c), k.size * 0.045, k.text);
                }
            }
        }
        GaugeStyle::Bar | GaugeStyle::Chevron => {
            let (width, h) = k.style.dims(k.size);
            let h = h * 0.55;
            let y = (k.style.dims(k.size).1 - h) / 2.0;
            fill(w, o, &pill(0.0, y, width, h), track);
            if k.frac > 0.0 {
                let fw = (width * k.frac).max(h);
                fill(w, o, &pill(0.0, y, fw, h), k.at(k.frac));
            }
        }
        GaugeStyle::Level => {
            let (width, height) = k.style.dims(k.size);
            fill(w, o, &pill(0.0, 0.0, width, height), track);
            if k.frac > 0.0 {
                let fh = (height * k.frac).max(width);
                fill(w, o, &pill(0.0, height - fh, width, fh), k.at(k.frac));
            }
        }
        _ => {}
    }
}

/// The target marker for `k.target`, placed per style family.
fn target_mark(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let Some(t) = k.target.map(|t| t.clamp(0.0, 1.0)) else {
        return;
    };
    let (width, height) = k.style.dims(k.size);
    let o = b.origin;
    let col = k.text;
    let radial = |s0: f32, span: f32, w: &mut Window| {
        let c = k.size / 2.0;
        let a = s0 + span * t;
        let line = [
            crate::gpui::draw::polar(c, c, c - 1.0, a),
            crate::gpui::draw::polar(c, c, c - k.size * 0.16, a),
        ];
        stroke(w, o, &line, 2.5, col, false);
    };
    match k.style {
        GaugeStyle::Arc
        | GaugeStyle::Segmented
        | GaugeStyle::Dial
        | GaugeStyle::Smooth
        | GaugeStyle::Dots => radial(SWEEP.0, SWEEP.1, w),
        GaugeStyle::Meter => radial(0.75, 0.5, w),
        GaugeStyle::Reticle | GaugeStyle::Ring => radial(0.0, 1.0, w),
        GaugeStyle::Bar | GaugeStyle::Chevron | GaugeStyle::Led => {
            let x = width * t;
            stroke(w, o, &[(x, -2.0), (x, height + 2.0)], 2.5, col, false);
        }
        GaugeStyle::Level | GaugeStyle::Thermometer => {
            let y = height * (1.0 - t);
            stroke(w, o, &[(-2.0, y), (width + 2.0, y)], 2.5, col, false);
        }
        GaugeStyle::Battery | GaugeStyle::Signal => {}
    }
}

fn segmented(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let c = k.size / 2.0;
    let r = c - k.ring / 2.0 - 2.0;
    let n = 28;
    let (s0, span) = SWEEP;
    for i in 0..n {
        let a0 = s0 + span * i as f32 / n as f32;
        let a1 = s0 + span * (i as f32 + 0.8) / n as f32;
        let p = turn_arc(c, c, r, a0, a1);
        let cov = cell_lit(i, n, k.frac);
        if cov > 0.0 {
            let col = k.at((i as f32 + 0.5) / n as f32);
            glow_stroke(
                w,
                b.origin,
                &p,
                k.ring,
                scale_alpha(col, 0.45 + 0.55 * cov),
                k.glow,
            );
        } else {
            stroke(w, b.origin, &p, k.ring, scale_alpha(k.track, 0.7), false);
        }
    }
}

fn dial(w: &mut Window, b: Bounds<Pixels>, k: Look, s0: f32, span: f32, with_needle: bool) {
    let c = k.size / 2.0;
    let o = b.origin;
    let r_tick = c - 2.0;
    let r_arc = r_tick - k.size * 0.17;
    // tick ring
    for (f, major) in tick_fracs(10, 4) {
        let len = if major { k.size * 0.1 } else { k.size * 0.05 };
        let a = s0 + span * f;
        let line = [
            crate::gpui::draw::polar(c, c, r_tick, a),
            crate::gpui::draw::polar(c, c, r_tick - len, a),
        ];
        let col = if f <= k.frac + 1e-4 {
            k.at(f)
        } else {
            scale_alpha(k.track, 0.9)
        };
        stroke(w, o, &line, if major { 2.0 } else { 1.0 }, col, false);
    }
    // value arc: thin track, thicker lit part, zone-coloured when asked
    let wide = if with_needle { 3.0 } else { k.size * 0.075 };
    stroke(
        w,
        o,
        &turn_arc(c, c, r_arc, s0, s0 + span),
        wide,
        scale_alpha(k.track, 0.8),
        false,
    );
    if k.frac > 0.0 {
        let steps = ((k.frac * 24.0).ceil() as usize).max(1);
        for i in 0..steps {
            let f0 = k.frac * i as f32 / steps as f32;
            let f1 = k.frac * (i + 1) as f32 / steps as f32;
            let p = turn_arc(c, c, r_arc, s0 + span * f0, s0 + span * f1);
            glow_stroke(w, o, &p, wide, k.at((f0 + f1) / 2.0), k.glow);
        }
    }
    if with_needle {
        let tip = s0 + span * k.frac;
        let tri = needle(c, c, r_arc - 6.0, k.size * 0.07, tip);
        fill(w, o, &tri, k.text);
        fill_circle(w, o, (c, c), k.size * 0.055, k.tone);
        stroke_circle(w, o, (c, c), k.size * 0.075, 1.0, scale_alpha(k.text, 0.6));
    }
}

fn reticle(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let c = k.size / 2.0;
    let o = b.origin;
    let r_out = c - 2.0;
    stroke_circle(w, o, (c, c), r_out, 1.0, scale_alpha(k.track, 0.9));
    for q in 0..4 {
        let a = q as f32 / 4.0;
        let line = [
            crate::gpui::draw::polar(c, c, r_out + 1.0, a),
            crate::gpui::draw::polar(c, c, r_out - k.size * 0.09, a),
        ];
        stroke(w, o, &line, 2.0, k.text, false);
    }
    // dashed inner ring
    let r_in = c * 0.5;
    for i in 0..24 {
        let a0 = i as f32 / 24.0;
        let p = turn_arc(c, c, r_in, a0, a0 + 0.5 / 24.0);
        stroke(w, o, &p, 1.5, scale_alpha(k.track, 0.9), false);
    }
    // value ring from the top
    let r_val = c * 0.76;
    let ring = (k.ring * 0.7).max(3.0);
    stroke_circle(w, o, (c, c), r_val, ring, scale_alpha(k.track, 0.45));
    if k.frac > 0.0 {
        let steps = ((k.frac * 36.0).ceil() as usize).max(1);
        for i in 0..steps {
            let f0 = k.frac * i as f32 / steps as f32;
            let f1 = k.frac * (i + 1) as f32 / steps as f32;
            let p = turn_arc(c, c, r_val, f0, f1);
            glow_stroke(w, o, &p, ring, k.at((f0 + f1) / 2.0), k.glow);
        }
        let head = crate::gpui::draw::polar(c, c, r_val, k.frac);
        fill_circle(w, o, (head.0, head.1), ring * 0.9, k.text);
    }
}

fn bar(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let (width, h) = k.style.dims(k.size);
    let chevron = k.style == GaugeStyle::Chevron;
    let n = if chevron { 16 } else { 22 };
    let lean = h * if chevron { 0.5 } else { 0.35 };
    let gap = 3.0;
    let cw = (width - lean - gap * (n as f32 - 1.0)) / n as f32;
    let shape = |x: f32, grow: f32| -> Vec<(f32, f32)> {
        if chevron {
            chevron_cell(
                x - grow / 2.0,
                -grow / 2.0,
                cw - lean * 0.2 + grow,
                h + grow,
                lean,
            )
            .to_vec()
        } else {
            slanted_cell(x - grow / 2.0, -grow / 2.0, cw + grow, h + grow, lean).to_vec()
        }
    };
    for i in 0..n {
        let x = i as f32 * (cw + gap);
        let cov = cell_lit(i, n, k.frac);
        if cov > 0.0 {
            let col = scale_alpha(k.at((i as f32 + 0.5) / n as f32), 0.5 + 0.5 * cov);
            if let Some(r) = k.glow {
                for l in dodai_gpui::paint::glow_layers(r * 0.6, 0.0) {
                    fill(
                        w,
                        b.origin,
                        &shape(x, l.width),
                        scale_alpha(col, l.alpha * 0.5),
                    );
                }
            }
            fill(w, b.origin, &shape(x, 0.0), col);
        } else {
            fill(w, b.origin, &shape(x, 0.0), scale_alpha(k.track, 0.6));
        }
    }
}

fn level(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let (width, height) = GaugeStyle::Level.dims(k.size);
    let n = 14;
    let gap = 3.0;
    let ch = (height - gap * (n as f32 - 1.0)) / n as f32;
    let cut = ch * 0.8;
    for i in 0..n {
        // i = 0 is the bottom cell
        let y = height - (i as f32 + 1.0) * ch - i as f32 * gap;
        let cov = cell_lit(i, n, k.frac);
        if cov > 0.0 {
            let col = scale_alpha(k.at((i as f32 + 0.5) / n as f32), 0.5 + 0.5 * cov);
            if let Some(r) = k.glow {
                for l in dodai_gpui::paint::glow_layers(r * 0.6, 0.0) {
                    let g = l.width;
                    let big = chamfer_rect(-g / 2.0, y - g / 2.0, width + g, ch + g, cut);
                    fill(w, b.origin, &big, scale_alpha(col, l.alpha * 0.5));
                }
            }
            fill(w, b.origin, &chamfer_rect(0.0, y, width, ch, cut), col);
        } else {
            fill(
                w,
                b.origin,
                &chamfer_rect(0.0, y, width, ch, cut),
                scale_alpha(k.track, 0.6),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dims_per_style() {
        assert_eq!(GaugeStyle::Arc.dims(100.0), (100.0, 100.0));
        assert!(GaugeStyle::Bar.dims(100.0).0 > GaugeStyle::Bar.dims(100.0).1);
        assert!(GaugeStyle::Meter.dims(100.0).1 < 100.0);
        assert_eq!(GaugeStyle::ALL.len(), 15);
        assert!(GaugeStyle::Thermometer.dims(100.0).0 < 50.0);
        assert!(GaugeStyle::Battery.dims(100.0).0 > GaugeStyle::Battery.dims(100.0).1);
        assert!(GaugeStyle::Level.dims(100.0).0 < GaugeStyle::Level.dims(100.0).1);
    }

    #[test]
    fn sweep_leaves_the_gap_at_the_bottom() {
        // the gap is centred on the bottom (0.5 turn)
        let (s0, span) = SWEEP;
        let gap_mid = (s0 + span + s0 + 1.0) / 2.0 - 1.0;
        assert!((gap_mid - 0.5).abs() < 1e-5);
    }
}
