//! Charts as data: filled polygons, strokes and dots in a plot's pixel space, so a toolkit only
//! fills and strokes them. Values wander on the animation clock, as the gpui showcase's do.

use std::f32::consts::TAU;

use dodai_core::{Color, Theme};

use crate::plot::{Line, Plot, Poly};

pub mod basic;
pub mod flow;
pub mod market;
pub mod status;

/// The colours of the series: the theme's, in order.
fn series(t: &Theme) -> [Color; 4] {
    let p = &t.palette;
    [p.primary, p.accent, p.info, p.danger]
}

/// A value of 0..1 drifting around `base` (a sum of two slow sines per `key`).
fn wander(now: f32, base: f32, key: f32) -> f32 {
    let v = base + 0.16 * (now * 0.7 + key * 1.9).sin() + 0.09 * (now * 1.3 + key * 3.1).sin();
    v.clamp(0.04, 0.98)
}

fn fade(c: Color, a: f32) -> Color {
    c.with_alpha(a)
}

fn stroke(color: Color, width: f32, points: Vec<(f32, f32)>) -> Line {
    Line {
        color,
        width,
        points,
    }
}

fn poly(color: Color, points: Vec<(f32, f32)>) -> Poly {
    Poly { color, points }
}

fn rect(color: Color, x: f32, y: f32, w: f32, h: f32) -> Poly {
    poly(color, vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)])
}

/// A dot: a short fat stroke.
fn dot(color: Color, (x, y): (f32, f32), size: f32) -> Line {
    stroke(color, size, vec![(x - 0.3, y), (x + 0.3, y)])
}

/// Points of an arc around `c`.
fn arc(c: (f32, f32), r: f32, a0: f32, a1: f32) -> Vec<(f32, f32)> {
    let n = ((a1 - a0).abs() / TAU * 64.0).ceil().max(2.0) as usize;
    (0..=n)
        .map(|i| {
            let a = a0 + (a1 - a0) * i as f32 / n as f32;
            (c.0 + r * a.cos(), c.1 + r * a.sin())
        })
        .collect()
}

/// The frame: horizontal grid lines.
fn grid(t: &Theme, w: f32, h: f32, pad: f32) -> Vec<Line> {
    (0..=4)
        .map(|i| {
            let y = pad + (h - 2.0 * pad) * i as f32 / 4.0;
            stroke(
                fade(t.palette.border, 0.7),
                1.0,
                vec![(pad, y), (w - pad, y)],
            )
        })
        .collect()
}

fn plot(w: f32, h: f32, fills: Vec<Poly>, lines: Vec<Line>) -> Plot {
    Plot { w, h, fills, lines }
}

/// Two lines over a grid; `area` fills under the first.
pub fn line(t: &Theme, w: f32, h: f32, now: f32, area: bool) -> Plot {
    let pad = 10.0;
    let s = series(t);
    let n = 14;
    let pts = |key: f32, base: f32| -> Vec<(f32, f32)> {
        (0..n)
            .map(|i| {
                let x = pad + (w - 2.0 * pad) * i as f32 / (n - 1) as f32;
                let v = wander(
                    now,
                    base + 0.25 * (i as f32 * 0.55 + key).sin(),
                    key + i as f32 * 0.37,
                );
                (x, h - pad - v * (h - 2.0 * pad))
            })
            .collect()
    };
    let a = pts(0.0, 0.5);
    let b = pts(2.0, 0.4);
    let mut fills = Vec::new();
    if area {
        let mut p = a.clone();
        p.push((w - pad, h - pad));
        p.push((pad, h - pad));
        fills.push(poly(fade(s[0], 0.25), p));
    }
    let mut lines = grid(t, w, h, pad);
    lines.push(stroke(s[0], 2.5, a));
    lines.push(stroke(s[1], 2.5, b));
    plot(w, h, fills, lines)
}

/// Bars; `stacked` stacks three series, otherwise one series of two colours.
pub fn bars(t: &Theme, w: f32, h: f32, now: f32, stacked: bool) -> Plot {
    let pad = 10.0;
    let s = series(t);
    let n = if stacked { 8 } else { 12 };
    let slot = (w - 2.0 * pad) / n as f32;
    let mut fills = Vec::new();
    for i in 0..n {
        let x = pad + slot * i as f32 + slot * 0.18;
        let bw = slot * 0.64;
        if stacked {
            let mut y = h - pad;
            for (k, &color) in s.iter().enumerate().take(3) {
                let v = wander(now, 0.2 + 0.05 * k as f32, i as f32 * 1.3 + k as f32 * 4.0)
                    * (h - 2.0 * pad)
                    * 0.5;
                y -= v;
                fills.push(rect(color, x, y, bw, v - 1.0));
            }
        } else {
            let v = wander(
                now,
                0.3 + 0.35 * ((i as f32 * 0.6).sin() * 0.5 + 0.5),
                i as f32 * 2.3,
            ) * (h - 2.0 * pad);
            fills.push(rect(s[i % 2], x, h - pad - v, bw, v));
        }
    }
    plot(w, h, fills, grid(t, w, h, pad))
}

/// A pie or a donut.
pub fn pie(t: &Theme, size: f32, now: f32, donut: bool) -> Plot {
    let s = series(t);
    let c = (size / 2.0, size / 2.0);
    let r = size / 2.0 - 4.0;
    let raw = [0.38, 0.27, 0.21, 0.14].map(|v| wander(now, v, v * 9.0));
    let total: f32 = raw.iter().sum();
    let (mut fills, mut lines) = (Vec::new(), Vec::new());
    let mut a = -TAU / 4.0;
    for (i, v) in raw.iter().enumerate() {
        let e = a + v / total * TAU;
        if donut {
            lines.push(stroke(s[i], r * 0.34, arc(c, r * 0.72, a + 0.03, e - 0.03)));
        } else {
            let mut p = vec![c];
            p.extend(arc(c, r, a, e));
            fills.push(poly(s[i], p));
        }
        a = e;
    }
    plot(size, size, fills, lines)
}

/// A radar of six axes.
pub fn radar(t: &Theme, size: f32, now: f32) -> Plot {
    let s = series(t);
    let c = size / 2.0;
    let r = size / 2.0 - 10.0;
    let at = |i: usize, k: f32| {
        let a = -TAU / 4.0 + i as f32 * TAU / 6.0;
        (c + r * k * a.cos(), c + r * k * a.sin())
    };
    let mut lines = Vec::new();
    for ring in [1.0, 0.66, 0.33] {
        let mut p: Vec<_> = (0..6).map(|i| at(i, ring)).collect();
        p.push(p[0]);
        lines.push(stroke(fade(t.palette.border, 0.8), 1.0, p));
    }
    for i in 0..6 {
        lines.push(stroke(
            fade(t.palette.border, 0.5),
            1.0,
            vec![(c, c), at(i, 1.0)],
        ));
    }
    let shape = |key: f32, base: f32| -> Vec<(f32, f32)> {
        (0..6)
            .map(|i| at(i, wander(now, base, key + i as f32 * 1.7)))
            .collect()
    };
    let a = shape(0.0, 0.62);
    let b = shape(3.0, 0.42);
    let mut closed = a.clone();
    closed.push(a[0]);
    lines.push(stroke(s[0], 2.5, closed));
    let mut closed = b.clone();
    closed.push(b[0]);
    lines.push(stroke(s[1], 2.0, closed));
    plot(
        size,
        size,
        vec![poly(fade(s[0], 0.28), a), poly(fade(s[1], 0.2), b)],
        lines,
    )
}

/// A half-circle gauge showing `value` (0..1).
pub fn gauge(t: &Theme, w: f32, now: f32) -> Plot {
    let s = series(t);
    let h = w * 0.62;
    let c = (w / 2.0, h - 8.0);
    let r = w / 2.0 - 12.0;
    let v = wander(now, 0.62, 5.0);
    let lines = vec![
        stroke(
            fade(t.palette.border, 0.8),
            14.0,
            arc(c, r, -TAU / 2.0, 0.0),
        ),
        stroke(
            s[0],
            14.0,
            arc(c, r, -TAU / 2.0, -TAU / 2.0 + v * TAU / 2.0),
        ),
    ];
    plot(w, h, Vec::new(), lines)
}

/// Scattered, drifting bubbles.
pub fn scatter(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let pad = 10.0;
    let s = series(t);
    let mut lines = grid(t, w, h, pad);
    for i in 0..30 {
        let fx = (i * 37 % 100) as f32 / 100.0;
        let fy = wander(now * 0.5, 0.25 + fx * 0.5, i as f32 * 1.1);
        let (x, y) = (pad + fx * (w - 2.0 * pad), h - pad - fy * (h - 2.0 * pad));
        lines.push(dot(
            fade(s[i % 3], 0.75),
            (x, y),
            5.0 + (i % 4) as f32 * 3.0,
        ));
    }
    plot(w, h, Vec::new(), lines)
}

/// A heatmap of `rows` by `cols` cells.
pub fn heat(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let (rows, cols) = (7, 20);
    let (cw, ch) = (w / cols as f32, h / rows as f32);
    let mut fills = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let v = wander(
                now * 0.6,
                0.5 + 0.4 * ((r * 3 + c) as f32 * 0.45).sin(),
                (r * cols + c) as f32 * 0.21,
            );
            fills.push(rect(
                fade(s[0], 0.1 + 0.9 * v),
                c as f32 * cw + 1.0,
                r as f32 * ch + 1.0,
                cw - 2.0,
                ch - 2.0,
            ));
        }
    }
    plot(w, h, fills, Vec::new())
}

/// Candlesticks.
pub fn candles(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let pad = 10.0;
    let s = series(t);
    let n = 18;
    let slot = (w - 2.0 * pad) / n as f32;
    let (mut fills, mut lines) = (Vec::new(), grid(t, w, h, pad));
    let y = |v: f32| h - pad - v * (h - 2.0 * pad);
    let mut prev = 0.5;
    for i in 0..n {
        let close = wander(
            now * 0.4,
            0.5 + 0.2 * (i as f32 * 0.5).sin(),
            i as f32 * 0.9,
        );
        let (open, close) = (prev, close);
        prev = close;
        let col = if close >= open { s[1] } else { s[3] };
        let x = pad + slot * (i as f32 + 0.5);
        let (hi, lo) = (open.max(close) + 0.06, open.min(close) - 0.06);
        lines.push(stroke(col, 1.5, vec![(x, y(hi)), (x, y(lo))]));
        let (top, bot) = (y(open.max(close)), y(open.min(close)));
        fills.push(rect(
            col,
            x - slot * 0.3,
            top,
            slot * 0.6,
            (bot - top).max(2.0),
        ));
    }
    plot(w, h, fills, lines)
}

/// A treemap of six cells that trade area.
pub fn treemap(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let split = wander(now * 0.5, 0.55, 1.0) * w;
    let vsplit = wander(now * 0.5, 0.5, 2.0) * h;
    let cells = [
        (0.0, 0.0, split, h),
        (split, 0.0, w - split, vsplit),
        (split, vsplit, (w - split) * 0.5, h - vsplit),
        (
            split + (w - split) * 0.5,
            vsplit,
            (w - split) * 0.5,
            h - vsplit,
        ),
    ];
    let fills = cells
        .iter()
        .enumerate()
        .map(|(i, (x, y, cw, ch))| {
            rect(
                fade(s[i % 3], 0.9 - 0.12 * i as f32),
                x + 1.5,
                y + 1.5,
                cw - 3.0,
                ch - 3.0,
            )
        })
        .collect();
    plot(w, h, fills, Vec::new())
}

/// A sparkline with its last point marked.
pub fn spark(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let n = 24;
    let pts: Vec<_> = (0..n)
        .map(|i| {
            let v = wander(
                now,
                0.5 + 0.2 * (i as f32 * 0.4 - now).sin(),
                i as f32 * 0.5,
            );
            (
                6.0 + (w - 12.0) * i as f32 / (n - 1) as f32,
                h - 6.0 - v * (h - 12.0),
            )
        })
        .collect();
    let last = pts[n - 1];
    plot(
        w,
        h,
        Vec::new(),
        vec![stroke(s[1], 2.0, pts), dot(s[1], last, 8.0)],
    )
}

/// A repeatable pseudo-random number in `0..1` for sample `i` of stream `k`.
fn rnd(i: usize, k: f32) -> f32 {
    let mut x =
        (i as u32).wrapping_mul(0x9E37_79B1) ^ ((k * 1000.0) as u32).wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    x as f32 / u32::MAX as f32
}

/// A repeatable bell-shaped sample (Box–Muller) for sample `i` of stream `k`.
fn gauss(i: usize, k: f32) -> f32 {
    (-2.0 * rnd(i, k).max(1e-4).ln()).sqrt() * (TAU * rnd(i, k + 17.0)).cos()
}
