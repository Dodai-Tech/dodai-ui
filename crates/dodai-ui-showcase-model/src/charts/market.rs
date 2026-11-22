//! Time series and market charts: Renko, depth, Gantt, indicators, stacks, control charts, pulse.

// The charts index parallel series and slots by position; iterators would hide the geometry.
#![allow(clippy::needless_range_loop)]

use dodai_ui_charts::candles::{heikin_ashi, Candle};
use dodai_ui_charts::{control, depth, gantt, indicators, pulse, renko};
use dodai_ui_core::Theme;

use super::{dot, fade, grid, plot, poly, rect, series, stroke, wander};
use crate::plot::Plot;

/// A drifting price walk of `n` closes in `0..1`.
fn walk(now: f32, n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let x = i as f32;
            (0.5 + 0.2 * (x * 0.22 + now * 0.3).sin()
                + 0.12 * (x * 0.6 + 1.0 + now * 0.2).sin()
                + 0.05 * (x * 1.7).sin())
            .clamp(0.05, 0.95)
        })
        .collect()
}

fn candle_set(now: f32, n: usize) -> Vec<Candle> {
    let c = walk(now, n + 1);
    (0..n)
        .map(|i| {
            let (o, cl) = (c[i], c[i + 1]);
            Candle::new(o, o.max(cl) + 0.04, o.min(cl) - 0.04, cl)
                .volume(0.3 + 0.7 * wander(now, 0.5, i as f32 * 1.7))
        })
        .collect()
}

/// Candles of a plot box `(w, h)` inside `pad`, their values in `0..1`.
fn draw_candles(
    t: &Theme,
    cs: &[Candle],
    w: f32,
    h: f32,
    pad: f32,
    fills: &mut Vec<crate::plot::Poly>,
    lines: &mut Vec<crate::plot::Line>,
) {
    let s = series(t);
    let slot = (w - 2.0 * pad) / cs.len() as f32;
    let y = |v: f32| pad + (h - 2.0 * pad) * (1.0 - v);
    for (i, c) in cs.iter().enumerate() {
        let col = if c.bullish() { s[1] } else { s[3] };
        let x = pad + slot * (i as f32 + 0.5);
        lines.push(stroke(col, 1.5, vec![(x, y(c.high)), (x, y(c.low))]));
        let (a, b) = c.body();
        fills.push(rect(
            col,
            x - slot * 0.3,
            y(b),
            slot * 0.6,
            (y(a) - y(b)).max(2.0),
        ));
    }
}

/// Heikin-Ashi smoothed candles.
pub fn heikin(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let pad = 10.0;
    let cs = heikin_ashi(&candle_set(now, 18));
    let (mut fills, mut lines) = (Vec::new(), grid(t, w, h, pad));
    draw_candles(t, &cs, w, h, pad, &mut fills, &mut lines);
    plot(w, h, fills, lines)
}

/// Renko bricks of a drifting price.
pub fn renko_bricks(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let closes: Vec<f32> = walk(now, 80).iter().map(|v| v * 100.0).collect();
    let bricks = renko::renko(&closes, 9.0);
    let n = bricks.len().max(1) as f32;
    let (lo, hi) = bricks
        .iter()
        .fold((f32::MAX, f32::MIN), |a, b| (a.0.min(b.lo), a.1.max(b.hi)));
    let slot = ((w - 2.0 * pad) / n).min(26.0);
    let y = |v: f32| h - pad - (v - lo) / (hi - lo).max(1.0) * (h - 2.0 * pad);
    let fills = bricks
        .iter()
        .enumerate()
        .map(|(i, b)| {
            rect(
                if b.up { s[1] } else { s[3] },
                pad + slot * i as f32 + 1.0,
                y(b.hi),
                slot - 2.0,
                y(b.lo) - y(b.hi) - 1.0,
            )
        })
        .collect();
    plot(w, h, fills, grid(t, w, h, pad))
}

/// Order-book depth: two stepped cumulative curves meeting at the mid price.
pub fn depth_book(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let lv = |k: f32, dir: f32| -> Vec<(f32, f32)> {
        (1..=14)
            .map(|i| {
                (
                    100.0 + dir * i as f32,
                    3.0 + 6.0 * wander(now, 0.5, i as f32 * 1.3 + k),
                )
            })
            .collect()
    };
    let (bids, asks) = depth::depth_curves(&lv(0.0, -1.0), &lv(5.0, 1.0));
    let top = bids
        .last()
        .map_or(1.0, |b| b.1)
        .max(asks.last().map_or(1.0, |a| a.1))
        .max(1.0);
    let x = |p: f32| pad + (p - 85.0) / 30.0 * (w - 2.0 * pad);
    let y = |c: f32| h - pad - c / top * (h - 2.0 * pad);
    let mut fills = Vec::new();
    let mut lines = grid(t, w, h, pad);
    for (curve, col) in [(&bids, s[1]), (&asks, s[3])] {
        let mut path = Vec::new();
        let mut prev = 0.0;
        for (p, c) in curve.iter() {
            path.push((x(*p), y(prev)));
            path.push((x(*p), y(*c)));
            prev = *c;
        }
        let mut area = path.clone();
        if let (Some(a), Some(b)) = (area.first().copied(), area.last().copied()) {
            area.push((b.0, h - pad));
            area.push((a.0, h - pad));
        }
        fills.push(poly(fade(col, 0.22), area));
        lines.push(stroke(col, 2.2, path));
    }
    plot(w, h, fills, lines)
}

/// A Gantt: tasks as bars with progress and dependency links.
pub fn gantt_chart(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let p = 0.5 + 0.5 * (now * 0.5).sin();
    let tasks = [
        gantt::Task::new("Design", 0.0, 10.0).progress(1.0),
        gantt::Task::new("Build", 8.0, 24.0).progress(p),
        gantt::Task::new("Test", 20.0, 30.0)
            .after(1)
            .progress(p * 0.5),
        gantt::Task::new("Ship", 30.0, 34.0).after(2),
    ];
    let lay = gantt::layout(&tasks);
    let rows = tasks.len() as f32;
    let slot = (h - 2.0 * pad) / rows;
    let xs = |f: f32| pad + f * (w - 2.0 * pad);
    let (mut fills, mut lines) = (Vec::new(), Vec::new());
    for b in &lay.bars {
        let y = pad + slot * b.row as f32 + slot * 0.2;
        fills.push(rect(
            fade(s[b.row % 4], 0.4),
            xs(b.x),
            y,
            b.w * (w - 2.0 * pad),
            slot * 0.6,
        ));
        fills.push(rect(
            s[b.row % 4],
            xs(b.x),
            y,
            b.done * (w - 2.0 * pad),
            slot * 0.6,
        ));
    }
    for (a, b) in &lay.links {
        let (ba, bb) = (&lay.bars[*a], &lay.bars[*b]);
        let ya = pad + slot * (ba.row as f32 + 0.5);
        let yb = pad + slot * (bb.row as f32 + 0.5);
        let ex = xs(ba.x + ba.w) + 4.0;
        lines.push(stroke(
            t.palette.text_muted,
            1.5,
            vec![(ex - 4.0, ya), (ex, ya), (ex, yb)],
        ));
    }
    plot(w, h, fills, lines)
}

/// Bollinger bands around a price line.
pub fn bollinger(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let n = 40;
    let v = walk(now, n);
    let (mid, up, lo) = indicators::bollinger(&v, 10, 2.0);
    let x = |i: usize| pad + (w - 2.0 * pad) * i as f32 / (n - 1) as f32;
    let y = |q: f32| h - pad - q.clamp(0.0, 1.0) * (h - 2.0 * pad);
    let path = |o: &Vec<Option<f32>>| -> Vec<(f32, f32)> {
        o.iter()
            .enumerate()
            .filter_map(|(i, q)| q.map(|q| (x(i), y(q))))
            .collect()
    };
    let (u, l) = (path(&up), path(&lo));
    let mut band = u.clone();
    band.extend(l.iter().rev());
    let mut lines = grid(t, w, h, pad);
    lines.push(stroke(fade(s[2], 0.9), 1.5, u));
    lines.push(stroke(fade(s[2], 0.9), 1.5, l));
    lines.push(stroke(fade(s[0], 0.8), 1.5, path(&mid)));
    lines.push(stroke(
        s[1],
        2.4,
        v.iter().enumerate().map(|(i, q)| (x(i), y(*q))).collect(),
    ));
    plot(w, h, vec![poly(fade(s[2], 0.12), band)], lines)
}

/// RSI over a price panel, and a MACD histogram under it.
pub fn indicator_panels(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let n = 50;
    let v = walk(now, n);
    let x = |i: usize| pad + (w - 2.0 * pad) * i as f32 / (n - 1) as f32;
    let half = (h - 3.0 * pad) / 2.0;
    let rsi = indicators::rsi(&v, 14);
    let (_, _, hist) = indicators::macd(&v, 12, 26, 9);
    let mut lines = Vec::new();
    for k in 0..2 {
        let y0 = pad + (half + pad) * k as f32;
        for g in [0.0, 0.5, 1.0] {
            lines.push(stroke(
                fade(t.palette.border, 0.7),
                1.0,
                vec![(pad, y0 + half * g), (w - pad, y0 + half * g)],
            ));
        }
    }
    lines.push(stroke(
        s[0],
        2.2,
        rsi.iter()
            .enumerate()
            .filter_map(|(i, r)| r.map(|r| (x(i), pad + half * (1.0 - r / 100.0))))
            .collect(),
    ));
    let mut fills = Vec::new();
    let y0 = pad + half + pad;
    let amp = hist.iter().flatten().fold(0.001_f32, |a, b| a.max(b.abs()));
    for (i, hv) in hist.iter().enumerate() {
        if let Some(hv) = hv {
            let h2 = hv / amp * half * 0.5;
            let (a, b) = if h2 >= 0.0 {
                (y0 + half / 2.0 - h2, y0 + half / 2.0)
            } else {
                (y0 + half / 2.0, y0 + half / 2.0 - h2)
            };
            fills.push(rect(
                if h2 >= 0.0 { s[1] } else { s[3] },
                x(i) - 2.0,
                a,
                4.0,
                (b - a).max(1.0),
            ));
        }
    }
    plot(w, h, fills, lines)
}

/// Price candles over a shared-axis volume panel.
pub fn price_volume(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let cs = candle_set(now, 20);
    let vol_h = h * 0.22;
    let (mut fills, mut lines) = (Vec::new(), Vec::new());
    for g in 0..=3 {
        let y = pad + (h - vol_h - 2.0 * pad) * g as f32 / 3.0;
        lines.push(stroke(
            fade(t.palette.border, 0.7),
            1.0,
            vec![(pad, y), (w - pad, y)],
        ));
    }
    let price_box = h - vol_h - pad;
    draw_candles(t, &cs, w, price_box, pad, &mut fills, &mut lines);
    let slot = (w - 2.0 * pad) / cs.len() as f32;
    for (i, c) in cs.iter().enumerate() {
        let col = if c.bullish() { s[1] } else { s[3] };
        let vh = c.volume * (vol_h - 4.0);
        fills.push(rect(
            fade(col, 0.7),
            pad + slot * i as f32 + slot * 0.2,
            h - pad - vh,
            slot * 0.6,
            vh,
        ));
    }
    plot(w, h, fills, lines)
}

/// Bars on the left axis and a line on a second, independent axis.
pub fn secondary_axis(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let n = 8;
    let slot = (w - 2.0 * pad) / n as f32;
    let y = |v: f32| h - pad - v * (h - 2.0 * pad);
    let mut fills = Vec::new();
    let mut pts = Vec::new();
    for i in 0..n {
        let b = wander(now, 0.4 + 0.2 * (i as f32 * 0.7).sin(), i as f32 * 1.4);
        fills.push(rect(
            fade(s[0], 0.8),
            pad + slot * i as f32 + slot * 0.2,
            y(b),
            slot * 0.6,
            h - pad - y(b),
        ));
        let l = wander(now, 0.3 + 0.05 * i as f32, i as f32 * 2.9 + 3.0);
        pts.push((pad + slot * (i as f32 + 0.5), y(l)));
    }
    let mut lines = grid(t, w, h, pad);
    lines.push(stroke(s[1], 2.5, pts.clone()));
    lines.extend(pts.into_iter().map(|p| dot(s[1], p, 8.0)));
    plot(w, h, fills, lines)
}

/// A control chart: a series with a centre line, control limits and signalling points.
pub fn control_chart(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 10.0;
    let n = 36;
    let v: Vec<f32> = (0..n)
        .map(|i| {
            50.0 + 6.0 * (i as f32 * 0.9 + now * 0.3).sin()
                + 3.0 * (i as f32 * 2.3).cos()
                + if i == 24 { 26.0 } else { 0.0 }
        })
        .collect();
    let l = control::limits(&v, 3.0);
    let sig = control::signals(&v, &l);
    let (lo, hi) = (l.lower.min(40.0) - 4.0, l.upper.max(80.0) + 4.0);
    let x = |i: usize| pad + (w - 2.0 * pad) * i as f32 / (n - 1) as f32;
    let y = |q: f32| h - pad - (q - lo) / (hi - lo) * (h - 2.0 * pad);
    let mut lines = Vec::new();
    for (q, c) in [
        (l.mean, t.palette.text_muted),
        (l.upper, s[3]),
        (l.lower, s[3]),
    ] {
        lines.push(stroke(
            fade(c, 0.8),
            1.5,
            vec![(pad, y(q)), (w - pad, y(q))],
        ));
    }
    lines.push(stroke(
        s[0],
        2.0,
        v.iter().enumerate().map(|(i, q)| (x(i), y(*q))).collect(),
    ));
    for (i, q) in v.iter().enumerate() {
        lines.push(dot(
            if sig[i] { s[3] } else { s[0] },
            (x(i), y(*q)),
            if sig[i] { 9.0 } else { 5.0 },
        ));
    }
    plot(w, h, Vec::new(), lines)
}

/// ECG, pulse-oximeter and respiration traces, one under the other.
pub fn pulse_traces(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 8.0;
    let kinds = [pulse::Kind::Ecg, pulse::Kind::Pleth, pulse::Kind::Resp];
    let band = (h - 2.0 * pad) / 3.0;
    let mut lines = Vec::new();
    for (k, kind) in kinds.iter().enumerate() {
        let sig = pulse::Signal {
            kind: *kind,
            rhythm: pulse::Rhythm::Normal,
            rate: if *kind == pulse::Kind::Resp {
                16.0
            } else {
                72.0
            },
            variability: 0.1,
            noise: 0.01,
            wander: 0.02,
            seed: k as f32,
        };
        let mid = pad + band * (k as f32 + 0.5);
        let secs = 5.0;
        let pts = (0..160)
            .map(|i| {
                let f = i as f32 / 159.0;
                (
                    pad + f * (w - 2.0 * pad),
                    mid - sig.at(now + f * secs) * band * 0.42,
                )
            })
            .collect();
        lines.push(stroke(s[[1, 2, 0][k]], 2.0, pts));
    }
    plot(w, h, Vec::new(), lines)
}

/// Calendar heatmap: weeks as columns, weekdays as rows.
pub fn calendar_heat(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let p = &t.palette;
    let weeks = 26;
    let (cw, ch) = (w / weeks as f32, h / 7.0);
    let mut fills = Vec::new();
    for wk in 0..weeks {
        for d in 0..7 {
            let v = wander(
                now * 0.2,
                0.5 + 0.4 * ((wk * 7 + d) as f32 * 0.13).sin(),
                (wk * 7 + d) as f32 * 0.9,
            );
            let lvl = (v * 4.0).floor() / 4.0;
            fills.push(rect(
                if lvl < 0.25 {
                    fade(p.border, 0.5)
                } else {
                    fade(p.success, 0.25 + lvl * 0.75)
                },
                wk as f32 * cw + 1.0,
                d as f32 * ch + 1.0,
                cw - 2.0,
                ch - 2.0,
            ));
        }
    }
    plot(w, h, fills, Vec::new())
}

/// An event timeline: lanes of dots and spans over a time line.
pub fn event_timeline(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let s = series(t);
    let pad = 12.0;
    let lanes = 3;
    let lane = (h - 2.0 * pad) / lanes as f32;
    let (mut fills, mut lines) = (Vec::new(), Vec::new());
    for l in 0..lanes {
        let y = pad + lane * (l as f32 + 0.5);
        lines.push(stroke(
            fade(t.palette.border, 0.8),
            1.0,
            vec![(pad, y), (w - pad, y)],
        ));
        for k in 0..5 {
            let f = ((k * 29 + l * 13) % 100) as f32 / 100.0;
            let x = pad + f * (w - 2.0 * pad - 40.0);
            if (k + l) % 2 == 0 {
                fills.push(rect(
                    fade(s[l], 0.75),
                    x,
                    y - lane * 0.22,
                    14.0 + 22.0 * wander(now * 0.2, 0.5, k as f32),
                    lane * 0.44,
                ));
            } else {
                lines.push(dot(s[l], (x, y), 10.0));
            }
        }
    }
    plot(w, h, fills, lines)
}

/// State spans: a lane per item, coloured runs of state.
pub fn state_spans(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let p = &t.palette;
    let pad = 8.0;
    let lanes = 4;
    let lane = (h - 2.0 * pad) / lanes as f32;
    let mut fills = Vec::new();
    for l in 0..lanes {
        let mut x = pad;
        let mut k = 0;
        while x < w - pad {
            let run =
                (16.0 + 60.0 * wander(now * 0.1, 0.4, (l * 7 + k) as f32 * 1.3)).min(w - pad - x);
            let col = [p.success, p.warning, p.danger, p.info][(l + k * 3 + (k / 2)) % 4];
            fills.push(rect(
                fade(col, 0.85),
                x,
                pad + lane * l as f32 + lane * 0.15,
                run - 2.0,
                lane * 0.7,
            ));
            x += run;
            k += 1;
        }
    }
    plot(w, h, fills, Vec::new())
}
