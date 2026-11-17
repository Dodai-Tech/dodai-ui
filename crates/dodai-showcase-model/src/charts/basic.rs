//! Charts from the gpui showcase's first tabs: pie and bar variants, comparisons, distributions.

// The charts index parallel series and slots by position; iterators would hide the geometry.
#![allow(clippy::needless_range_loop)]

use std::f32::consts::TAU;

use dodai_charts::{bins, density};
use dodai_core::Theme;

use super::{arc, dot, fade, gauss, grid, plot, poly, rect, series, stroke, wander};
use crate::plot::Plot;

/// Which radial shape [`radial`] draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Radial {
    /// Slices of equal radius, pulled out of the centre a little.
    Exploded,
    /// Slices whose radius follows the value.
    Rose,
    /// A half circle.
    Half,
    /// A three-quarter ring.
    ThreeQuarter,
    /// A thin ring of segments.
    Segmented,
}

/// Pie variants.
pub fn radial(t: &Theme, size: f32, now: f32, kind: Radial) -> Plot {
    let s = series(t);
    let raw = [0.34, 0.26, 0.22, 0.18].map(|v| wander(now, v, v * 11.0));
    let total: f32 = raw.iter().sum();
    let (start, sweep) = match kind {
        Radial::Half => (-TAU / 2.0, TAU / 2.0),
        Radial::ThreeQuarter => (-TAU * 0.625, TAU * 0.75),
        _ => (-TAU / 4.0, TAU),
    };
    let h = if kind == Radial::Half {
        size / 2.0 + 6.0
    } else {
        size
    };
    let c = (size / 2.0, size / 2.0);
    let r = size / 2.0 - 6.0;
    let (mut fills, mut lines) = (Vec::new(), Vec::new());
    let mut a = start;
    for (i, v) in raw.iter().enumerate() {
        let e = a + v / total * sweep;
        let mid = (a + e) / 2.0;
        match kind {
            Radial::Exploded => {
                let o = (c.0 + 7.0 * mid.cos(), c.1 + 7.0 * mid.sin());
                let mut p = vec![o];
                p.extend(arc(o, r - 8.0, a + 0.02, e - 0.02));
                fills.push(poly(s[i], p));
            }
            Radial::Rose => {
                let rr = r * (0.45 + 0.55 * v / raw.iter().cloned().fold(0.0, f32::max));
                let mut p = vec![c];
                p.extend(arc(c, rr, a + 0.02, e - 0.02));
                fills.push(poly(s[i], p));
            }
            Radial::Segmented => {
                let steps = 6;
                for k in 0..steps {
                    let a0 = a + (e - a) * k as f32 / steps as f32 + 0.015;
                    let a1 = a + (e - a) * (k + 1) as f32 / steps as f32 - 0.015;
                    lines.push(stroke(s[i], r * 0.16, arc(c, r * 0.84, a0, a1)));
                }
            }
            Radial::Half | Radial::ThreeQuarter => {
                lines.push(stroke(s[i], r * 0.26, arc(c, r * 0.8, a + 0.02, e - 0.02)));
            }
        }
        a = e;
    }
    plot(size, h, fills, lines)
}

/// Bars laid on their side, two series grouped.
pub fn hbars(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let n = 6;
    let slot = (h - 2.0 * pad) / n as f32;
    let mut fills = Vec::new();
    for i in 0..n {
        for k in 0..2 {
            let v = wander(
                now,
                0.3 + 0.12 * ((i + k * 2) as f32).sin().abs(),
                i as f32 * 1.7 + k as f32 * 3.0,
            );
            let y = pad + slot * i as f32 + slot * (0.12 + 0.39 * k as f32);
            fills.push(rect(s[k], pad, y, v * (w - 2.0 * pad), slot * 0.36));
        }
    }
    let lines = (0..=4)
        .map(|i| {
            let x = pad + (w - 2.0 * pad) * i as f32 / 4.0;
            stroke(
                fade(t.palette.border, 0.7),
                1.0,
                vec![(x, pad), (x, h - pad)],
            )
        })
        .collect();
    plot(w, h, fills, lines)
}

/// Areas stacked on each other.
pub fn stacked_area(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let n = 12;
    let x = |i: usize| pad + (w - 2.0 * pad) * i as f32 / (n - 1) as f32;
    let mut tops: Vec<Vec<f32>> = Vec::new();
    let mut acc = vec![0.0_f32; n];
    for k in 0..3 {
        for (i, a) in acc.iter_mut().enumerate() {
            *a += 0.1
                + 0.16
                    * wander(
                        now,
                        0.5 + 0.25 * ((i as f32 * 0.5) + k as f32 * 2.0).sin(),
                        i as f32 * 0.8 + k as f32 * 5.0,
                    );
        }
        tops.push(acc.clone());
    }
    let top = tops[2].iter().cloned().fold(0.0, f32::max).max(0.01);
    let y = |v: f32| h - pad - v / top * (h - 2.0 * pad);
    let mut fills = Vec::new();
    for k in (0..3).rev() {
        let mut p: Vec<(f32, f32)> = (0..n).map(|i| (x(i), y(tops[k][i]))).collect();
        for i in (0..n).rev() {
            let base = if k == 0 { 0.0 } else { tops[k - 1][i] };
            p.push((x(i), y(base)));
        }
        fills.push(poly(fade(s[k], 0.78), p));
    }
    plot(w, h, fills, grid(t, w, h, pad))
}

/// A histogram of a bell-shaped sample.
pub fn histogram(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let data: Vec<f32> = (0..400)
        .map(|i| gauss(i, 1.0) + 0.3 * (now * 0.4).sin())
        .collect();
    let bins = bins::histogram(&data, Some(16));
    let top = bins.iter().map(|b| b.count).max().unwrap_or(1) as f32;
    let slot = (w - 2.0 * pad) / bins.len().max(1) as f32;
    let fills = bins
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let bh = b.count as f32 / top * (h - 2.0 * pad);
            rect(
                s[0],
                pad + slot * i as f32 + 1.0,
                h - pad - bh,
                slot - 2.0,
                bh,
            )
        })
        .collect();
    plot(w, h, fills, grid(t, w, h, pad))
}

/// A waterfall: a start, gains and losses, a total.
pub fn waterfall(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let p = &t.palette;
    let pad = 10.0;
    let steps = [40.0, 25.0, -12.0, -8.0, 15.0 + 4.0 * (now * 0.8).sin(), 0.0];
    let mut level = 0.0_f32;
    let mut spans = Vec::new();
    for (i, v) in steps.iter().enumerate() {
        if i == 0 {
            spans.push((0.0, *v, p.primary));
            level = *v;
        } else if i == steps.len() - 1 {
            spans.push((0.0, level, p.primary));
        } else {
            spans.push((
                level,
                level + v,
                if *v >= 0.0 { p.success } else { p.danger },
            ));
            level += v;
        }
    }
    let top = 80.0;
    let slot = (w - 2.0 * pad) / spans.len() as f32;
    let y = |v: f32| h - pad - v / top * (h - 2.0 * pad);
    let fills = spans
        .iter()
        .enumerate()
        .map(|(i, (a, b, c))| {
            let (lo, hi) = (a.min(*b), a.max(*b));
            rect(
                *c,
                pad + slot * i as f32 + slot * 0.15,
                y(hi),
                slot * 0.7,
                y(lo) - y(hi),
            )
        })
        .collect();
    plot(w, h, fills, grid(t, w, h, pad))
}

/// A funnel of narrowing, centred bars.
pub fn funnel(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let vals =
        [1000.0, 620.0, 340.0, 120.0].map(|v: f32| v * (0.95 + 0.05 * (now * 0.9 + v).sin()));
    let n = vals.len();
    let slot = h / n as f32;
    let mut fills = Vec::new();
    for i in 0..n {
        let top = vals[i] / vals[0] * w;
        let bot = vals.get(i + 1).map_or(top * 0.7, |v| v / vals[0] * w);
        let (y0, y1) = (slot * i as f32 + 1.0, slot * (i + 1) as f32 - 1.0);
        fills.push(poly(
            s[i % 4],
            vec![
                ((w - top) / 2.0, y0),
                ((w + top) / 2.0, y0),
                ((w + bot) / 2.0, y1),
                ((w - bot) / 2.0, y1),
            ],
        ));
    }
    plot(w, h, fills, Vec::new())
}

/// Concentric rings, each filled to its value.
pub fn radial_bar(t: &Theme, size: f32, now: f32) -> Plot {
    let s = series(t);
    let c = (size / 2.0, size / 2.0);
    let mut lines = Vec::new();
    for (i, v) in [0.72, 0.48, 0.31].iter().enumerate() {
        let r = size / 2.0 - 8.0 - i as f32 * 18.0;
        lines.push(stroke(
            fade(t.palette.border, 0.8),
            12.0,
            arc(c, r, 0.0, TAU),
        ));
        let f = wander(now, *v, i as f32 * 2.0);
        lines.push(stroke(
            s[i],
            12.0,
            arc(c, r, -TAU / 4.0, -TAU / 4.0 + TAU * 0.75 * f),
        ));
    }
    plot(size, size, Vec::new(), lines)
}

/// Bullet charts: a value bar over range bands with a target tick.
pub fn bullet(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let p = &t.palette;
    let rows = 3;
    let slot = h / rows as f32;
    let mut fills = Vec::new();
    let mut lines = Vec::new();
    for i in 0..rows {
        let y = slot * i as f32 + slot * 0.18;
        let bh = slot * 0.64;
        for (k, f) in [1.0, 0.7, 0.45].iter().enumerate() {
            fills.push(rect(
                fade(p.border, 0.35 + 0.25 * k as f32),
                0.0,
                y,
                w * f,
                bh,
            ));
        }
        let v = wander(now, 0.4 + 0.15 * i as f32, i as f32 * 2.5);
        fills.push(rect(p.primary, 0.0, y + bh * 0.33, w * v, bh * 0.34));
        let tx = w * (0.62 + 0.08 * i as f32);
        lines.push(stroke(
            p.text,
            2.5,
            vec![(tx, y + bh * 0.12), (tx, y + bh * 0.88)],
        ));
    }
    plot(w, h, fills, lines)
}

/// Pairs of bars side by side, before and after.
pub fn paired_bars(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let n = 4;
    let slot = (w - 2.0 * pad) / n as f32;
    let mut fills = Vec::new();
    for i in 0..n {
        for k in 0..2 {
            let v = wander(
                now,
                0.3 + 0.1 * ((i * 2 + k) as f32).cos().abs() + 0.2 * k as f32,
                i as f32 * 1.9 + k as f32,
            );
            let bh = v * (h - 2.0 * pad);
            fills.push(rect(
                s[k],
                pad + slot * i as f32 + slot * (0.1 + 0.42 * k as f32),
                h - pad - bh,
                slot * 0.38,
                bh,
            ));
        }
    }
    plot(w, h, fills, grid(t, w, h, pad))
}

/// Lollipops (a stem and a dot), and dumbbells (two dots joined).
pub fn lollipop(t: &Theme, w: f32, h: f32, now: f32, dumbbell: bool) -> Plot {
    let s = series(t);
    let pad = 14.0;
    let n = 6;
    let slot = (w - 2.0 * pad) / n as f32;
    let mut lines = grid(t, w, h, pad);
    for i in 0..n {
        let x = pad + slot * (i as f32 + 0.5);
        let a = wander(
            now,
            0.35 + 0.3 * (i as f32 * 1.1).sin().abs(),
            i as f32 * 2.0,
        );
        let ya = h - pad - a * (h - 2.0 * pad);
        if dumbbell {
            let b = wander(now, 0.5 + 0.2 * (i as f32).cos(), i as f32 * 3.1);
            let yb = h - pad - b * (h - 2.0 * pad);
            lines.push(stroke(
                fade(t.palette.text_muted, 0.8),
                2.0,
                vec![(x, ya), (x, yb)],
            ));
            lines.push(dot(s[1], (x, yb), 11.0));
        } else {
            lines.push(stroke(fade(s[0], 0.7), 2.0, vec![(x, h - pad), (x, ya)]));
        }
        lines.push(dot(s[0], (x, ya), 11.0));
    }
    plot(w, h, Vec::new(), lines)
}

/// A slope chart: each item travels from one column to another.
pub fn slope(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 16.0;
    let mut lines = Vec::new();
    for (i, (a, b)) in [(0.3, 0.6), (0.5, 0.42), (0.7, 0.85), (0.2, 0.25)]
        .iter()
        .enumerate()
    {
        let a = wander(now, *a, i as f32);
        let b = wander(now, *b, i as f32 + 7.0);
        let (ya, yb) = (h - pad - a * (h - 2.0 * pad), h - pad - b * (h - 2.0 * pad));
        lines.push(stroke(s[i], 2.5, vec![(pad, ya), (w - pad, yb)]));
        lines.push(dot(s[i], (pad, ya), 9.0));
        lines.push(dot(s[i], (w - pad, yb), 9.0));
    }
    plot(w, h, Vec::new(), lines)
}

/// Box plots (min, quartiles, median, max) for four groups.
pub fn box_plot(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 12.0;
    let slot = (w - 2.0 * pad) / 4.0;
    let y = |v: f32| h - pad - v * (h - 2.0 * pad);
    let (mut fills, mut lines) = (Vec::new(), grid(t, w, h, pad));
    for i in 0..4 {
        let m = wander(now * 0.5, 0.35 + 0.1 * i as f32, i as f32 * 2.0);
        let (q1, q3) = (m - 0.1, m + 0.12);
        let (lo, hi) = (q1 - 0.12, q3 + 0.1);
        let x = pad + slot * (i as f32 + 0.5);
        lines.push(stroke(
            t.palette.text_muted,
            1.5,
            vec![(x, y(lo)), (x, y(hi))],
        ));
        lines.push(stroke(
            t.palette.text_muted,
            1.5,
            vec![(x - 7.0, y(lo)), (x + 7.0, y(lo))],
        ));
        lines.push(stroke(
            t.palette.text_muted,
            1.5,
            vec![(x - 7.0, y(hi)), (x + 7.0, y(hi))],
        ));
        fills.push(rect(
            fade(s[i], 0.8),
            x - slot * 0.24,
            y(q3),
            slot * 0.48,
            y(q1) - y(q3),
        ));
        lines.push(stroke(
            t.palette.text,
            2.5,
            vec![(x - slot * 0.24, y(m)), (x + slot * 0.24, y(m))],
        ));
    }
    plot(w, h, fills, lines)
}

/// Violins: a mirrored density per group.
pub fn violin(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let slot = (w - 2.0 * pad) / 3.0;
    let mut fills = Vec::new();
    for g in 0..3 {
        let data: Vec<f32> = (0..120)
            .map(|i| {
                gauss(i, g as f32) * (0.8 + 0.2 * g as f32)
                    + g as f32
                    + 0.2 * (now * 0.5 + g as f32).sin()
            })
            .collect();
        let d = density::kde(&data, 40);
        let top = d.y.iter().cloned().fold(f32::EPSILON, f32::max);
        let cx = pad + slot * (g as f32 + 0.5);
        let n = d.y.len();
        let ypx = |k: usize| h - pad - k as f32 / (n - 1) as f32 * (h - 2.0 * pad);
        let mut p: Vec<(f32, f32)> = (0..n)
            .map(|k| (cx + d.y[k] / top * slot * 0.42, ypx(k)))
            .collect();
        p.extend(
            (0..n)
                .rev()
                .map(|k| (cx - d.y[k] / top * slot * 0.42, ypx(k))),
        );
        fills.push(poly(fade(s[g], 0.8), p));
    }
    plot(w, h, fills, Vec::new())
}

/// An empirical cumulative distribution as a step line.
pub fn ecdf(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let mut lines = grid(t, w, h, pad);
    for (k, off) in [0.0, 0.8].iter().enumerate() {
        let data: Vec<f32> = (0..60)
            .map(|i| gauss(i, k as f32) * 0.8 + 1.8 + off + 0.3 * (now * 0.5).sin())
            .collect();
        let pts = density::ecdf(&data);
        let (lo, hi) = (0.0, 4.2);
        let mut path = Vec::new();
        let mut prev = 0.0;
        for (x, y) in pts {
            let px = pad + ((x - lo) / (hi - lo)).clamp(0.0, 1.0) * (w - 2.0 * pad);
            path.push((px, h - pad - prev * (h - 2.0 * pad)));
            path.push((px, h - pad - y * (h - 2.0 * pad)));
            prev = y;
        }
        lines.push(stroke(s[k], 2.5, path));
    }
    plot(w, h, Vec::new(), lines)
}

/// Points with error bars.
pub fn error_bars(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 14.0;
    let n = 6;
    let slot = (w - 2.0 * pad) / n as f32;
    let mut lines = grid(t, w, h, pad);
    for i in 0..n {
        let x = pad + slot * (i as f32 + 0.5);
        let v = wander(now, 0.35 + 0.08 * i as f32, i as f32 * 2.2);
        let e = 0.06 + 0.05 * (i as f32).sin().abs();
        let y = |q: f32| h - pad - q * (h - 2.0 * pad);
        lines.push(stroke(s[1], 1.8, vec![(x, y(v - e)), (x, y(v + e))]));
        lines.push(stroke(
            s[1],
            1.8,
            vec![(x - 5.0, y(v - e)), (x + 5.0, y(v - e))],
        ));
        lines.push(stroke(
            s[1],
            1.8,
            vec![(x - 5.0, y(v + e)), (x + 5.0, y(v + e))],
        ));
        lines.push(dot(s[0], (x, y(v)), 9.0));
    }
    plot(w, h, Vec::new(), lines)
}

/// Sparkline variants beyond the line: bars, steps, dots, win/loss.
pub fn spark_kind(t: &Theme, w: f32, h: f32, now: f32, kind: u8) -> Plot {
    let s = series(t);
    let n = 24;
    let vals: Vec<f32> = (0..n)
        .map(|i| wander(now, 0.5 + 0.3 * (i as f32 * 0.5).sin(), i as f32 * 0.9))
        .collect();
    let x = |i: usize| 2.0 + (w - 4.0) * i as f32 / (n - 1) as f32;
    let y = |v: f32| h - 3.0 - v * (h - 6.0);
    let slot = (w - 4.0) / n as f32;
    let (mut fills, mut lines) = (Vec::new(), Vec::new());
    match kind {
        0 => {
            for (i, v) in vals.iter().enumerate() {
                fills.push(rect(
                    s[0],
                    2.0 + slot * i as f32 + 1.0,
                    y(*v),
                    slot - 2.0,
                    h - 3.0 - y(*v),
                ));
            }
        }
        1 => {
            let mut p = Vec::new();
            for (i, v) in vals.iter().enumerate() {
                p.push((x(i), y(*v)));
                if let Some(nx) = vals.get(i + 1) {
                    let _ = nx;
                    p.push((x(i + 1), y(*v)));
                }
            }
            lines.push(stroke(s[0], 2.0, p));
        }
        2 => {
            for (i, v) in vals.iter().enumerate() {
                lines.push(dot(s[0], (x(i), y(*v)), 4.0));
            }
        }
        _ => {
            for (i, v) in vals.iter().enumerate() {
                let up = *v >= 0.5;
                let (c, y0, y1) = if up {
                    (t.palette.success, h / 2.0 - h * 0.38, h / 2.0 - 1.0)
                } else {
                    (t.palette.danger, h / 2.0 + 1.0, h / 2.0 + h * 0.38)
                };
                fills.push(rect(
                    c,
                    2.0 + slot * i as f32 + 1.0,
                    y0,
                    slot - 2.0,
                    y1 - y0,
                ));
            }
        }
    }
    plot(w, h, fills, lines)
}
