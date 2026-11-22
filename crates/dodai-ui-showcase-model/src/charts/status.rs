//! Axes, bands, survey bars and status displays: the charts that read a state at a glance.

// The charts index parallel series and slots by position; iterators would hide the geometry.
#![allow(clippy::needless_range_loop)]

use std::f32::consts::TAU;

use dodai_ui_charts::{layers, pack, rank, timeaxis};
use dodai_ui_core::Theme;

use super::{dot, fade, gauss, grid, plot, poly, rect, series, stroke, wander};
use crate::plot::Plot;

/// A line over a time axis: ticks come from the calendar maths (days, then hours).
pub fn time_axis(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let (start, end) = (1_700_000_000.0_f64, 1_700_000_000.0 + 5.0 * 86_400.0);
    let mut lines = Vec::new();
    for tick in timeaxis::ticks(start, end, 6, 0) {
        let x = pad + timeaxis::frac(tick.t, start, end) * (w - 2.0 * pad);
        lines.push(stroke(
            fade(t.palette.border, 0.8),
            1.0,
            vec![(x, pad), (x, h - pad)],
        ));
        lines.push(stroke(
            t.palette.text_muted,
            1.5,
            vec![(x, h - pad), (x, h - pad + 5.0)],
        ));
    }
    let pts = (0..60)
        .map(|i| {
            let f = i as f32 / 59.0;
            (
                pad + f * (w - 2.0 * pad),
                h - pad
                    - (0.5 + 0.3 * (f * 12.0 + now * 0.4).sin() * (0.6 + 0.4 * f))
                        * (h - 2.0 * pad),
            )
        })
        .collect();
    lines.push(stroke(s[0], 2.5, pts));
    plot(w, h, Vec::new(), lines)
}

/// A line with a confidence band around it.
pub fn band(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let n = 30;
    let x = |i: usize| pad + (w - 2.0 * pad) * i as f32 / (n - 1) as f32;
    let y = |v: f32| h - pad - v * (h - 2.0 * pad);
    let mid: Vec<f32> = (0..n)
        .map(|i| 0.5 + 0.22 * (i as f32 * 0.25 + now * 0.4).sin())
        .collect();
    let spread = |i: usize| 0.05 + 0.1 * i as f32 / n as f32;
    let mut p: Vec<(f32, f32)> = (0..n).map(|i| (x(i), y(mid[i] + spread(i)))).collect();
    p.extend((0..n).rev().map(|i| (x(i), y(mid[i] - spread(i)))));
    let mut lines = grid(t, w, h, pad);
    lines.push(stroke(
        s[0],
        2.5,
        (0..n).map(|i| (x(i), y(mid[i]))).collect(),
    ));
    plot(w, h, vec![poly(fade(s[0], 0.2), p)], lines)
}

/// A line with a threshold, the part above it in a warning colour, and a marked peak.
pub fn annotated(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let p = &t.palette;
    let pad = 10.0;
    let n = 40;
    let x = |i: usize| pad + (w - 2.0 * pad) * i as f32 / (n - 1) as f32;
    let y = |v: f32| h - pad - v * (h - 2.0 * pad);
    let v: Vec<f32> = (0..n)
        .map(|i| 0.5 + 0.3 * (i as f32 * 0.3 + now * 0.4).sin() + 0.1 * (i as f32 * 1.1).sin())
        .collect();
    let limit = 0.7;
    let mut lines = grid(t, w, h, pad);
    let mut fills = vec![rect(
        fade(p.danger, 0.1),
        pad,
        y(1.0),
        w - 2.0 * pad,
        y(limit) - y(1.0),
    )];
    lines.push(stroke(
        fade(p.danger, 0.9),
        1.5,
        vec![(pad, y(limit)), (w - pad, y(limit))],
    ));
    lines.push(stroke(
        p.primary,
        2.4,
        v.iter().enumerate().map(|(i, q)| (x(i), y(*q))).collect(),
    ));
    for (i, q) in v.iter().enumerate().filter(|(_, q)| **q > limit) {
        lines.push(dot(p.danger, (x(i), y(*q)), 6.0));
    }
    if let Some((i, q)) = v.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)) {
        lines.push(stroke(
            p.text_muted,
            1.0,
            vec![(x(i), y(*q) - 4.0), (x(i), y(*q) - 16.0)],
        ));
        fills.push(rect(p.text_muted, x(i) - 3.0, y(*q) - 22.0, 6.0, 6.0));
    }
    plot(w, h, fills, lines)
}

/// Bubbles over time, their size a magnitude.
pub fn bubble_timeline(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let pad = 14.0;
    let mut lines = vec![stroke(
        fade(t.palette.border_strong, 0.9),
        1.5,
        vec![(pad, h / 2.0), (w - pad, h / 2.0)],
    )];
    for i in 0..9 {
        let f = (i as f32 + 0.5) / 9.0;
        let m = 6.0 + 22.0 * wander(now * 0.3, 0.5, i as f32 * 1.9);
        let lane = [-1.0, 0.0, 1.0][i % 3];
        lines.push(dot(
            fade(t.palette.series[i % 4], 0.7),
            (pad + f * (w - 2.0 * pad), h / 2.0 + lane * h * 0.2),
            m,
        ));
    }
    plot(w, h, Vec::new(), lines)
}

/// A Pareto chart: bars sorted by size and the running share as a line.
pub fn pareto(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let vals: Vec<f32> = [42.0, 27.0, 14.0, 9.0, 5.0, 3.0]
        .iter()
        .enumerate()
        .map(|(i, v)| v * (0.92 + 0.08 * (now * 0.6 + i as f32).sin()))
        .collect();
    let (order, share) = rank::pareto(&vals);
    let top = vals.iter().cloned().fold(0.0, f32::max).max(1.0);
    let slot = (w - 2.0 * pad) / vals.len() as f32;
    let y = |f: f32| h - pad - f * (h - 2.0 * pad);
    let mut fills = Vec::new();
    let mut pts = Vec::new();
    for (k, &i) in order.iter().enumerate() {
        let x = pad + slot * k as f32;
        fills.push(rect(
            s[0],
            x + slot * 0.15,
            y(vals[i] / top * 0.8),
            slot * 0.7,
            h - pad - y(vals[i] / top * 0.8),
        ));
        pts.push((x + slot * 0.5, y(share[k])));
    }
    let mut lines = grid(t, w, h, pad);
    lines.push(stroke(s[3], 2.4, pts.clone()));
    lines.extend(pts.into_iter().map(|p| dot(s[3], p, 7.0)));
    plot(w, h, fills, lines)
}

/// Diverging bars: survey answers split round a centre line.
pub fn diverging(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let p = &t.palette;
    let pad = 10.0;
    let rows = 5;
    let slot = (h - 2.0 * pad) / rows as f32;
    let cols = [
        p.danger,
        fade(p.danger, 0.5),
        p.border_strong,
        fade(p.success, 0.5),
        p.success,
    ];
    let mut fills = Vec::new();
    for r in 0..rows {
        let row: Vec<f32> = (0..5)
            .map(|k| 4.0 + 8.0 * wander(now * 0.3, 0.5, (r * 5 + k) as f32 * 1.7))
            .collect();
        let total: f32 = row.iter().sum();
        for (k, (a, b)) in layers::diverging(&row, 2.5).iter().enumerate() {
            let c = w / 2.0;
            let sc = (w - 2.0 * pad) / 2.0 / (total * 0.55);
            fills.push(rect(
                cols[k],
                c + a * sc,
                pad + slot * r as f32 + slot * 0.18,
                (b - a) * sc,
                slot * 0.64,
            ));
        }
    }
    let lines = vec![stroke(
        p.text_muted,
        1.5,
        vec![(w / 2.0, pad), (w / 2.0, h - pad)],
    )];
    plot(w, h, fills, lines)
}

/// A beeswarm whose dots have different sizes.
pub fn bubble_swarm(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let pad = 16.0;
    let n = 36;
    let xs: Vec<f32> = (0..n)
        .map(|i| {
            (0.5 + 0.22 * gauss(i, 11.0) + 0.02 * (now * 0.3 + i as f32).sin()).clamp(0.0, 1.0)
                * (w - 2.0 * pad)
        })
        .collect();
    let rs: Vec<f32> = (0..n).map(|i| 3.0 + 9.0 * super::rnd(i, 4.0)).collect();
    let off = pack::swarm_sized(&xs, &rs);
    let mut lines = vec![stroke(
        fade(t.palette.border_strong, 0.9),
        1.5,
        vec![(pad, h / 2.0), (w - pad, h / 2.0)],
    )];
    for i in 0..n {
        lines.push(dot(
            fade(t.palette.series[i % 3], 0.75),
            (pad + xs[i], h / 2.0 + off[i]),
            rs[i] * 2.0,
        ));
    }
    plot(w, h, Vec::new(), lines)
}

/// A grid of slots, each a state: free, busy, faulty.
pub fn slot_grid(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let p = &t.palette;
    let (cols, rows) = (12, 5);
    let (cw, ch) = (w / cols as f32, h / rows as f32);
    let mut fills = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let v = super::rnd(r * cols + c, 2.0) + 0.1 * (now * 0.4 + (r + c) as f32).sin();
            let col = if v > 0.92 {
                p.danger
            } else if v > 0.55 {
                p.primary
            } else {
                fade(p.border, 0.8)
            };
            fills.push(rect(
                col,
                c as f32 * cw + 2.0,
                r as f32 * ch + 2.0,
                cw - 4.0,
                ch - 4.0,
            ));
        }
    }
    plot(w, h, fills, Vec::new())
}

/// A strip of status over time: green, amber, red segments.
pub fn status_strip(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let p = &t.palette;
    let n = 48;
    let cw = w / n as f32;
    let fills = (0..n)
        .map(|i| {
            let v = super::rnd(i, 7.0) + 0.1 * (now * 0.3 + i as f32 * 0.4).sin();
            let c = if v > 0.93 {
                p.danger
            } else if v > 0.8 {
                p.warning
            } else {
                p.success
            };
            rect(c, i as f32 * cw + 1.0, 0.0, cw - 2.0, h)
        })
        .collect();
    plot(w, h, fills, Vec::new())
}

/// Bucket bars: counts per latency bucket, the tail in a warning colour.
pub fn bucket_bar(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let p = &t.palette;
    let pad = 8.0;
    let rows = 7;
    let slot = (h - 2.0 * pad) / rows as f32;
    let fills = (0..rows)
        .map(|i| {
            let v = (-(i as f32 - 1.5).powi(2) / 4.0).exp()
                * (0.9 + 0.1 * (now * 0.6 + i as f32).sin());
            rect(
                if i >= 5 { p.warning } else { p.primary },
                pad,
                pad + slot * i as f32 + 2.0,
                v * (w - 2.0 * pad),
                slot - 4.0,
            )
        })
        .collect();
    plot(w, h, fills, Vec::new())
}

/// Pipeline stage bars: how far each stage has got.
pub fn stage_bars(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let p = &t.palette;
    let pad = 8.0;
    let stages = 4;
    let slot = (h - 2.0 * pad) / stages as f32;
    let mut fills = Vec::new();
    for i in 0..stages {
        let y = pad + slot * i as f32 + slot * 0.2;
        fills.push(rect(fade(p.border, 0.7), pad, y, w - 2.0 * pad, slot * 0.6));
        let f = (1.0 - i as f32 * 0.25 + 0.1 * (now * 0.5 + i as f32).sin()).clamp(0.05, 1.0);
        fills.push(rect(
            if f >= 1.0 { p.success } else { p.primary },
            pad,
            y,
            f * (w - 2.0 * pad),
            slot * 0.6,
        ));
    }
    plot(w, h, fills, Vec::new())
}

/// A colour scale with a marker.
pub fn gradient_scale(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let p = &t.palette;
    let n = 40;
    let cw = w / n as f32;
    let fills = (0..n)
        .map(|i| {
            let f = i as f32 / (n - 1) as f32;
            let c = if f < 0.5 {
                p.info.mix(p.warning, f * 2.0)
            } else {
                p.warning.mix(p.danger, f * 2.0 - 1.0)
            };
            rect(c, i as f32 * cw, h * 0.3, cw + 0.5, h * 0.4)
        })
        .collect();
    let x = w * (0.5 + 0.35 * (now * 0.6).sin());
    let marker = vec![poly(
        p.text,
        vec![(x - 5.0, h * 0.05), (x + 5.0, h * 0.05), (x, h * 0.26)],
    )];
    let mut all: Vec<_> = fills;
    all.extend(marker);
    plot(w, h, all, Vec::new())
}

/// Small multiples: six tiny trends side by side.
pub fn small_multiples(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let (cols, rows) = (3, 2);
    let (cw, ch) = (w / cols as f32, h / rows as f32);
    let mut lines = Vec::new();
    for k in 0..6 {
        let (ox, oy) = ((k % cols) as f32 * cw, (k / cols) as f32 * ch);
        let pts = (0..16)
            .map(|i| {
                let f = i as f32 / 15.0;
                (
                    ox + 6.0 + f * (cw - 12.0),
                    oy + ch
                        - 8.0
                        - (0.5 + 0.35 * (f * (3.0 + k as f32) + now * 0.4 + k as f32).sin())
                            * (ch - 16.0),
                )
            })
            .collect();
        lines.push(stroke(
            fade(t.palette.border, 0.8),
            1.0,
            vec![(ox + 6.0, oy + ch - 8.0), (ox + cw - 6.0, oy + ch - 8.0)],
        ));
        lines.push(stroke(s[k % 4], 2.0, pts));
    }
    plot(w, h, Vec::new(), lines)
}

/// A KPI trend: a line with a filled area, as under a big number.
pub fn kpi_trend(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let n = 20;
    let pts: Vec<(f32, f32)> = (0..n)
        .map(|i| {
            let f = i as f32 / (n - 1) as f32;
            (
                f * w,
                h - 4.0 - (0.25 + 0.5 * f + 0.1 * (f * TAU * 2.0 + now * 0.5).sin()) * (h - 8.0),
            )
        })
        .collect();
    let mut area = pts.clone();
    area.push((w, h));
    area.push((0.0, h));
    plot(
        w,
        h,
        vec![poly(fade(s[1], 0.22), area)],
        vec![stroke(s[1], 2.2, pts)],
    )
}
