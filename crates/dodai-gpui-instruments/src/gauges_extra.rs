//! The newer [`crate::gauges::GaugeStyle`] painters: ring, comet arc, dots,
//! LEDs, battery, thermometer and signal bars.

use crate::draw::{glow_stroke, polar, turn_arc};
use crate::gauges::{cell_lit, pill, round_arc, Look, SWEEP};
use dodai_gpui::paint::{fill, fill_circle, scale_alpha, stroke, stroke_circle};
pub use dodai_instruments_core::gauge::lit_bars;
use gpui::{Bounds, Pixels, Window};

/// Full ring from 12 o'clock.
pub(crate) fn ring(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let c = k.size / 2.0;
    let r = c - k.ring / 2.0 - 2.0;
    stroke_circle(w, b.origin, (c, c), r, k.ring, scale_alpha(k.track, 0.8));
    if k.frac > 0.0 {
        let f = k.frac.min(0.9999);
        round_arc(w, b.origin, c, r, 0.0, f, k.ring, k.at(k.frac));
    }
}

/// 270° arc whose alpha climbs towards a bright head.
pub(crate) fn smooth(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let c = k.size / 2.0;
    let r = c - k.ring / 2.0 - 2.0;
    let (s0, span) = SWEEP;
    let o = b.origin;
    stroke(
        w,
        o,
        &turn_arc(c, c, r, s0, s0 + span),
        k.ring,
        scale_alpha(k.track, 0.7),
        false,
    );
    if k.frac <= 0.0 {
        return;
    }
    let steps = ((k.frac * 60.0).ceil() as usize).max(2);
    for i in 0..steps {
        let (f0, f1) = (
            k.frac * i as f32 / steps as f32,
            k.frac * (i + 1) as f32 / steps as f32,
        );
        let p = turn_arc(c, c, r, s0 + span * f0, s0 + span * f1);
        let fade = 0.18 + 0.82 * (f1 / k.frac);
        let col = scale_alpha(k.at((f0 + f1) / 2.0), fade);
        glow_stroke(w, o, &p, k.ring, col, k.glow.filter(|_| i + 6 >= steps));
    }
    let head = polar(c, c, r, s0 + span * k.frac);
    fill_circle(w, o, (head.0, head.1), k.ring * 0.7, k.text);
}

/// 270° sweep of dots.
pub(crate) fn dots(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let c = k.size / 2.0;
    let r = c - k.ring - 1.0;
    let (s0, span) = SWEEP;
    let n = 30;
    for i in 0..n {
        let (x, y) = polar(c, c, r, s0 + span * i as f32 / (n - 1) as f32);
        let cov = cell_lit(i, n, k.frac);
        let (col, rad) = if cov > 0.0 {
            (
                scale_alpha(k.at(i as f32 / (n - 1) as f32), 0.5 + 0.5 * cov),
                k.ring * 0.5,
            )
        } else {
            (scale_alpha(k.track, 0.7), k.ring * 0.34)
        };
        if cov > 0.0 {
            if let Some(g) = k.glow {
                for l in dodai_gpui::paint::glow_layers(g * 0.5, 0.0) {
                    fill_circle(
                        w,
                        b.origin,
                        (x, y),
                        rad + l.width / 2.0,
                        scale_alpha(col, l.alpha * 0.5),
                    );
                }
            }
        }
        fill_circle(w, b.origin, (x, y), rad, col);
    }
}

/// Row of round LEDs.
pub(crate) fn led(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let (width, h) = k.style.dims(k.size);
    let n = 12;
    let d = (width / n as f32).min(h);
    let step = (width - d) / (n as f32 - 1.0);
    for i in 0..n {
        let (x, y) = (d / 2.0 + i as f32 * step, h / 2.0);
        let cov = cell_lit(i, n, k.frac);
        let lit = cov > 0.0;
        let col = if lit {
            scale_alpha(k.at((i as f32 + 0.5) / n as f32), 0.55 + 0.45 * cov)
        } else {
            scale_alpha(k.track, 0.6)
        };
        if lit {
            if let Some(g) = k.glow {
                for l in dodai_gpui::paint::glow_layers(g * 0.6, 0.0) {
                    fill_circle(
                        w,
                        b.origin,
                        (x, y),
                        d * 0.4 + l.width / 2.0,
                        scale_alpha(col, l.alpha * 0.5),
                    );
                }
            }
        }
        fill_circle(w, b.origin, (x, y), d * 0.4, col);
    }
}

/// Battery outline, charge fill and terminal nub.
pub(crate) fn battery(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let (width, h) = k.style.dims(k.size);
    let nub = width * 0.07;
    let body = width - nub;
    let o = b.origin;
    let edge = scale_alpha(k.text, 0.8);
    stroke(w, o, &pill(1.0, 1.0, body - 2.0, h - 2.0), 2.0, edge, true);
    fill(w, o, &crate::draw::rect(body, h * 0.3, nub, h * 0.4), edge);
    if k.frac > 0.0 {
        let pad = 5.0;
        // At least as wide as it is tall: a narrower pill flips to a vertical capsule.
        let fw = ((body - pad * 2.0) * k.frac).max(h - pad * 2.0);
        let col = if k.frac < 0.2 { k.alert } else { k.at(0.0) };
        let col = if k.zones.is_some() && k.frac >= 0.2 {
            k.zones.map_or(col, |z| z[0])
        } else {
            col
        };
        fill(w, o, &pill(pad, pad, fw, h - pad * 2.0), col);
    }
}

/// Thermometer: tube, bulb, ticks and a mercury column.
pub(crate) fn thermometer(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let (width, height) = k.style.dims(k.size);
    let o = b.origin;
    let bulb = width * 0.5;
    let tube_w = width * 0.36;
    let tx = (width - tube_w) / 2.0 - width * 0.12;
    let tube_h = height - bulb * 1.6;
    let track = scale_alpha(k.track, 0.8);
    fill(w, o, &pill(tx, 0.0, tube_w, tube_h + tube_w), track);
    let (cx, cy) = (tx + tube_w / 2.0, height - bulb);
    fill_circle(w, o, (cx, cy), bulb, track);
    let col = k.at(k.frac);
    fill_circle(w, o, (cx, cy), bulb * 0.78, col);
    if k.frac > 0.0 {
        let top = tube_w * 0.5 + (tube_h - tube_w) * (1.0 - k.frac);
        let inner = tube_w * 0.5;
        fill(
            w,
            o,
            &crate::draw::rect(cx - inner / 2.0, top, inner, cy - top),
            col,
        );
        fill_circle(w, o, (cx, top), inner / 2.0, col);
    }
    for i in 0..=5 {
        let y = tube_w * 0.5 + (tube_h - tube_w) * i as f32 / 5.0;
        let len = if i % 5 == 0 {
            width * 0.2
        } else {
            width * 0.12
        };
        stroke(
            w,
            o,
            &[(tx + tube_w + 3.0, y), (tx + tube_w + 3.0 + len, y)],
            1.0,
            scale_alpha(k.text, 0.7),
            false,
        );
    }
}

/// Five ascending bars.
pub(crate) fn signal(w: &mut Window, b: Bounds<Pixels>, k: Look) {
    let (width, h) = k.style.dims(k.size);
    let n = 5;
    let gap = width * 0.07;
    let bw = (width - gap * (n as f32 - 1.0)) / n as f32;
    let lit = lit_bars(k.frac, n);
    for i in 0..n {
        let bh = h * (0.28 + 0.72 * (i as f32 + 1.0) / n as f32);
        let x = i as f32 * (bw + gap);
        let col = if i < lit {
            k.at((i as f32 + 0.5) / n as f32)
        } else {
            scale_alpha(k.track, 0.6)
        };
        fill(w, b.origin, &pill(x, h - bh, bw, bh), col);
    }
}
