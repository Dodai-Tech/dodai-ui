//! Gauge styles for [`crate::Gauge`]: HUD-like dials, segmented rings, a
//! reticle and a slanted cell bar. Geometry is pure and unit-tested; colours,
//! glow and fonts all come from the theme.

use crate::charts::{glow_stroke, turn_arc};
use crate::paint::{fill, scale_alpha, stroke};
use gpui::{Bounds, Hsla, Pixels, Window};

/// How a [`crate::Gauge`] draws its value.
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
}

impl GaugeStyle {
    pub const ALL: [GaugeStyle; 8] = [
        GaugeStyle::Arc,
        GaugeStyle::Segmented,
        GaugeStyle::Dial,
        GaugeStyle::Meter,
        GaugeStyle::Reticle,
        GaugeStyle::Bar,
        GaugeStyle::Chevron,
        GaugeStyle::Level,
    ];
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
        }
    }
    /// `(width, height)` for a gauge of nominal `size`.
    pub fn dims(self, size: f32) -> (f32, f32) {
        match self {
            GaugeStyle::Meter => (size, size * 0.62),
            GaugeStyle::Bar | GaugeStyle::Chevron => (size * 1.6, size * 0.2),
            GaugeStyle::Level => (size * 0.34, size),
            _ => (size, size),
        }
    }
}

/// 270° sweep with the gap at the bottom: starts at 7:30 going clockwise.
pub(crate) const SWEEP: (f32, f32) = (0.625, 0.75);

// ───────────────────────────── pure helpers ─────────────────────────────

/// Fill coverage `0..=1` of cell `i` of `n` when `frac` of the range is lit.
pub fn cell_lit(i: usize, n: usize, frac: f32) -> f32 {
    if n == 0 {
        return 0.0;
    }
    (frac.clamp(0.0, 1.0) * n as f32 - i as f32).clamp(0.0, 1.0)
}

/// Tick positions along a sweep: `(frac, is_major)`. `major` spans, each split
/// into `minor` sub-spans.
pub fn tick_fracs(major: usize, minor: usize) -> Vec<(f32, bool)> {
    let major = major.max(1);
    let minor = minor.max(1);
    let total = major * minor;
    (0..=total)
        .map(|i| (i as f32 / total as f32, i % minor == 0))
        .collect()
}

/// Which colour zone `frac` falls in: 0 = ok, 1 = warn, 2 = danger.
pub fn zone(frac: f32) -> usize {
    if frac < 0.6 {
        0
    } else if frac < 0.85 {
        1
    } else {
        2
    }
}

/// Tapered needle (triangle) from the hub at `(cx, cy)` towards turn-fraction
/// `at`, `len` long with a `base`-wide root.
pub fn needle(cx: f32, cy: f32, len: f32, base: f32, at: f32) -> [(f32, f32); 3] {
    let tip = crate::charts::polar(cx, cy, len, at);
    let l = crate::charts::polar(cx, cy, base / 2.0, at - 0.25);
    let r = crate::charts::polar(cx, cy, base / 2.0, at + 0.25);
    [l, tip, r]
}

/// Parallelogram slanted by `skew` px: `(x, y)` is the bottom-left corner.
pub fn slanted_cell(x: f32, y: f32, w: f32, h: f32, skew: f32) -> [(f32, f32); 4] {
    [(x, y + h), (x + skew, y), (x + skew + w, y), (x + w, y + h)]
}

/// Arrow-head cell pointing right: a `>` with `pt` px point depth.
pub fn chevron_cell(x: f32, y: f32, w: f32, h: f32, pt: f32) -> [(f32, f32); 6] {
    [
        (x, y),
        (x + w, y),
        (x + w + pt, y + h / 2.0),
        (x + w, y + h),
        (x, y + h),
        (x + pt, y + h / 2.0),
    ]
}

/// Rectangle with the top-right corner cut by `c` px.
pub fn chamfer_rect(x: f32, y: f32, w: f32, h: f32, c: f32) -> [(f32, f32); 5] {
    [
        (x, y),
        (x + w - c, y),
        (x + w, y + c),
        (x + w, y + h),
        (x, y + h),
    ]
}

/// Regular polygon approximating a circle. Pure.
pub fn disc(cx: f32, cy: f32, r: f32, n: usize) -> Vec<(f32, f32)> {
    let n = n.max(3);
    (0..n)
        .map(|i| crate::charts::polar(cx, cy, r, i as f32 / n as f32))
        .collect()
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
}

impl Look {
    fn at(&self, frac: f32) -> Hsla {
        self.zones.map_or(self.tone, |z| z[zone(frac)])
    }
}

pub(crate) fn paint(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    match k.style {
        GaugeStyle::Arc => {}
        GaugeStyle::Segmented => segmented(w, b, k),
        GaugeStyle::Dial => dial(w, b, k, SWEEP.0, SWEEP.1, true),
        GaugeStyle::Meter => dial(w, b, k, 0.75, 0.5, false),
        GaugeStyle::Reticle => reticle(w, b, k),
        GaugeStyle::Bar | GaugeStyle::Chevron => bar(w, b, k),
        GaugeStyle::Level => level(w, b, k),
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
            crate::charts::polar(c, c, r_tick, a),
            crate::charts::polar(c, c, r_tick - len, a),
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
        fill(w, o, &disc(c, c, k.size * 0.055, 20), k.tone);
        stroke(
            w,
            o,
            &disc(c, c, k.size * 0.075, 20),
            1.0,
            scale_alpha(k.text, 0.6),
            true,
        );
    }
}

fn reticle(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let c = k.size / 2.0;
    let o = b.origin;
    let r_out = c - 2.0;
    stroke(
        w,
        o,
        &turn_arc(c, c, r_out, 0.0, 1.0),
        1.0,
        scale_alpha(k.track, 0.9),
        true,
    );
    for q in 0..4 {
        let a = q as f32 / 4.0;
        let line = [
            crate::charts::polar(c, c, r_out + 1.0, a),
            crate::charts::polar(c, c, r_out - k.size * 0.09, a),
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
    stroke(
        w,
        o,
        &turn_arc(c, c, r_val, 0.0, 1.0),
        ring,
        scale_alpha(k.track, 0.45),
        true,
    );
    if k.frac > 0.0 {
        let steps = ((k.frac * 36.0).ceil() as usize).max(1);
        for i in 0..steps {
            let f0 = k.frac * i as f32 / steps as f32;
            let f1 = k.frac * (i + 1) as f32 / steps as f32;
            let p = turn_arc(c, c, r_val, f0, f1);
            glow_stroke(w, o, &p, ring, k.at((f0 + f1) / 2.0), k.glow);
        }
        let head = crate::charts::polar(c, c, r_val, k.frac);
        fill(w, o, &disc(head.0, head.1, ring * 0.9, 14), k.text);
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
                for l in crate::paint::glow_layers(r * 0.6, 0.0) {
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
                for l in crate::paint::glow_layers(r * 0.6, 0.0) {
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
    fn cells_fill_in_order_with_a_partial_tip() {
        assert_eq!(cell_lit(0, 10, 0.0), 0.0);
        assert_eq!(cell_lit(9, 10, 1.0), 1.0);
        assert_eq!(cell_lit(2, 10, 0.25), 0.5);
        assert_eq!(cell_lit(3, 10, 0.25), 0.0);
        assert_eq!(cell_lit(0, 0, 0.5), 0.0);
    }

    #[test]
    fn ticks_span_the_sweep_with_majors_every_minor_step() {
        let t = tick_fracs(10, 4);
        assert_eq!(t.len(), 41);
        assert_eq!((t[0].0, t[40].0), (0.0, 1.0));
        assert_eq!(t.iter().filter(|x| x.1).count(), 11);
        assert!(t[4].1 && !t[5].1);
    }

    #[test]
    fn zones_escalate() {
        assert_eq!((zone(0.1), zone(0.7), zone(0.95)), (0, 1, 2));
    }

    #[test]
    fn needle_tip_is_on_the_radius() {
        let [l, tip, r] = needle(50.0, 50.0, 40.0, 6.0, 0.25);
        // 0.25 turn = pointing right
        assert!((tip.0 - 90.0).abs() < 1e-3 && (tip.1 - 50.0).abs() < 1e-3);
        assert!(l.1 != r.1, "base is perpendicular to the tip");
    }

    #[test]
    fn slanted_cell_leans_right_at_the_top() {
        let c = slanted_cell(0.0, 0.0, 10.0, 20.0, 5.0);
        assert!(c[1].0 > c[0].0 && c[1].1 < c[0].1);
    }

    #[test]
    fn dims_per_style() {
        assert_eq!(GaugeStyle::Arc.dims(100.0), (100.0, 100.0));
        assert!(GaugeStyle::Bar.dims(100.0).0 > GaugeStyle::Bar.dims(100.0).1);
        assert!(GaugeStyle::Meter.dims(100.0).1 < 100.0);
        assert_eq!(GaugeStyle::ALL.len(), 8);
        assert!(GaugeStyle::Level.dims(100.0).0 < GaugeStyle::Level.dims(100.0).1);
    }

    #[test]
    fn chevron_points_right_and_chamfer_cuts_one_corner() {
        let c = chevron_cell(0.0, 0.0, 10.0, 10.0, 4.0);
        assert!(c[2].0 > c[1].0 && c[5].0 > c[0].0);
        let r = chamfer_rect(0.0, 0.0, 10.0, 10.0, 3.0);
        assert_eq!((r[1], r[2]), ((7.0, 0.0), (10.0, 3.0)));
    }

    #[test]
    fn disc_is_a_closed_ring_of_equal_radius() {
        let d = disc(5.0, 5.0, 2.0, 12);
        assert_eq!(d.len(), 12);
        for (x, y) in d {
            assert!((((x - 5.0).powi(2) + (y - 5.0).powi(2)).sqrt() - 2.0).abs() < 1e-4);
        }
    }

    #[test]
    fn sweep_leaves_the_gap_at_the_bottom() {
        // the gap is centred on the bottom (0.5 turn)
        let (s0, span) = SWEEP;
        let gap_mid = (s0 + span + s0 + 1.0) / 2.0 - 1.0;
        assert!((gap_mid - 0.5).abs() < 1e-5);
    }
}
