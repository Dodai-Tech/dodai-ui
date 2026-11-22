//! Flows, hierarchies, networks and layered charts, laid out by `dodai-ui-charts`.

// The charts index parallel series and slots by position; iterators would hide the geometry.
#![allow(clippy::needless_range_loop)]

use std::f32::consts::TAU;

use dodai_ui_charts::hierarchy::{partition, Node};
use dodai_ui_charts::layers::{stack_layers, StackOffset};
use dodai_ui_charts::{
    cascade, chord, fan, force, hexbin, horizon, milestones, pack, peaks, rank, sankey, shards,
    share, swarm,
};
use dodai_ui_core::Theme;

use super::{arc, dot, fade, gauss, grid, plot, poly, rect, stroke, wander};
use crate::plot::Plot;

/// The colour of branch `i`.
fn pick(t: &Theme, i: usize) -> dodai_ui_core::Color {
    t.palette.series[i % 8]
}

/// A tree of languages, the sample of every hierarchy chart.
fn tree(now: f32) -> Node {
    let v = |k: f32, base: f32| base * (0.9 + 0.1 * (now * 0.6 + k).sin());
    Node::branch(
        "All",
        [
            Node::branch(
                "Rust",
                [
                    Node::leaf("core", v(0.0, 30.0)),
                    Node::leaf("ui", v(1.0, 22.0)),
                ],
            ),
            Node::branch(
                "Kotlin",
                [
                    Node::leaf("app", v(2.0, 24.0)),
                    Node::leaf("wear", v(3.0, 12.0)),
                    Node::leaf("tv", v(4.0, 6.0)),
                ],
            ),
            Node::branch(
                "Web",
                [
                    Node::leaf("css", v(5.0, 14.0)),
                    Node::leaf("ts", v(6.0, 10.0)),
                ],
            ),
            Node::leaf("Monkey C", v(7.0, 9.0)),
        ],
    )
}

/// A Sankey: three columns of nodes, ribbons between them.
pub fn sankey_flow(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let links: Vec<sankey::Link> = [
        (0, 3, 30.0),
        (0, 4, 20.0),
        (1, 3, 15.0),
        (1, 4, 25.0),
        (2, 4, 10.0),
        (3, 5, 40.0),
        (3, 6, 20.0),
        (4, 5, 15.0),
        (4, 6, 38.0),
    ]
    .iter()
    .map(|&(from, to, v)| sankey::Link {
        from,
        to,
        value: v * (0.9 + 0.1 * (now * 0.7 + v).sin()),
    })
    .collect();
    let lay = sankey::layout(7, &links, 0.06);
    let pad = 8.0;
    let cols = lay.columns.max(2) as f32;
    let nw = 12.0;
    let x = |c: usize| pad + (w - 2.0 * pad - nw) * c as f32 / (cols - 1.0);
    let y = |f: f32| pad + f * (h - 2.0 * pad);
    let mut fills = Vec::new();
    for r in &lay.ribbons {
        let (a, b) = (&lay.nodes[r.from], &lay.nodes[r.to]);
        let (x0, x1) = (x(a.col) + nw, x(b.col));
        let mid = (x0 + x1) / 2.0;
        let (ya, yb) = (y(r.y_from), y(r.y_to));
        let th = r.h * (h - 2.0 * pad);
        let top = [(x0, ya), (mid, ya), (mid, yb), (x1, yb)];
        let curve = |pts: [(f32, f32); 4], dy: f32| -> Vec<(f32, f32)> {
            (0..=14)
                .map(|k| {
                    let u = k as f32 / 14.0;
                    let s = u * u * (3.0 - 2.0 * u);
                    (
                        pts[0].0 + (pts[3].0 - pts[0].0) * u,
                        pts[0].1 + (pts[3].1 - pts[0].1) * s + dy,
                    )
                })
                .collect()
        };
        let mut p = curve(top, 0.0);
        let mut bot = curve(top, th);
        bot.reverse();
        p.extend(bot);
        fills.push(poly(fade(pick(t, r.from), 0.45), p));
    }
    for (i, n) in lay.nodes.iter().enumerate() {
        fills.push(rect(
            pick(t, i),
            x(n.col),
            y(n.y),
            nw,
            n.h * (h - 2.0 * pad),
        ));
    }
    plot(w, h, fills, Vec::new())
}

/// A chord diagram: groups on a ring, ribbons between them.
pub fn chord_ring(t: &Theme, size: f32, now: f32) -> Plot {
    let m: Vec<Vec<f32>> = (0..4)
        .map(|i| {
            (0..4)
                .map(|j| {
                    if i == j {
                        4.0
                    } else {
                        6.0 + 4.0 * ((i * 3 + j * 5) % 4) as f32
                            + 2.0 * (now * 0.6 + (i + j) as f32).sin()
                    }
                })
                .collect()
        })
        .collect();
    let ch = chord::chord(&m, 0.02);
    let c = (size / 2.0, size / 2.0);
    let r = size / 2.0 - 12.0;
    let pt = |turn: f32, rr: f32| {
        (
            c.0 + rr * (turn * TAU - TAU / 4.0).cos(),
            c.1 + rr * (turn * TAU - TAU / 4.0).sin(),
        )
    };
    let (mut fills, mut lines) = (Vec::new(), Vec::new());
    for rb in &ch.ribbons {
        let ring = |a: f32, b: f32| -> Vec<(f32, f32)> {
            (0..=8)
                .map(|k| pt(a + (b - a) * k as f32 / 8.0, r - 8.0))
                .collect()
        };
        let mut p = ring(rb.source.0, rb.source.1);
        // Bend through the centre to the target arc.
        p.extend((0..=8).map(|k| {
            let u = k as f32 / 8.0;
            let (a, b) = (pt(rb.source.1, r - 8.0), pt(rb.target.0, r - 8.0));
            let q = (c.0, c.1);
            (
                (1.0 - u).powi(2) * a.0 + 2.0 * u * (1.0 - u) * q.0 + u * u * b.0,
                (1.0 - u).powi(2) * a.1 + 2.0 * u * (1.0 - u) * q.1 + u * u * b.1,
            )
        }));
        p.extend(ring(rb.target.0, rb.target.1));
        p.extend((0..=8).map(|k| {
            let u = k as f32 / 8.0;
            let (a, b) = (pt(rb.target.1, r - 8.0), pt(rb.source.0, r - 8.0));
            (
                (1.0 - u).powi(2) * a.0 + 2.0 * u * (1.0 - u) * c.0 + u * u * b.0,
                (1.0 - u).powi(2) * a.1 + 2.0 * u * (1.0 - u) * c.1 + u * u * b.1,
            )
        }));
        fills.push(poly(fade(pick(t, rb.from), 0.5), p));
    }
    for (i, (a, b)) in ch.groups.iter().enumerate() {
        let pts: Vec<(f32, f32)> = (0..=16)
            .map(|k| pt(a + (b - a) * k as f32 / 16.0, r))
            .collect();
        lines.push(stroke(pick(t, i), 8.0, pts));
    }
    plot(size, size, fills, lines)
}

/// Split flow: one quantity splitting down through levels.
pub fn split_flow(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let k = 0.9 + 0.1 * (now * 0.7).sin();
    let nodes = [
        cascade::Node {
            parent: None,
            value: 100.0,
        },
        cascade::Node {
            parent: Some(0),
            value: 62.0 * k,
        },
        cascade::Node {
            parent: Some(0),
            value: 38.0,
        },
        cascade::Node {
            parent: Some(1),
            value: 40.0,
        },
        cascade::Node {
            parent: Some(1),
            value: 22.0 * k,
        },
        cascade::Node {
            parent: Some(2),
            value: 25.0,
        },
        cascade::Node {
            parent: Some(2),
            value: 13.0,
        },
    ];
    let pad = 8.0;
    let lay = cascade::layout(&nodes, w - 2.0 * pad, 6.0);
    let levels = lay.iter().map(|b| b.level).max().unwrap_or(0) + 1;
    let row = (h - 2.0 * pad) / levels as f32;
    let bh = row * 0.4;
    let ry = |l: usize| pad + row * l as f32;
    let mut fills = Vec::new();
    for (i, b) in lay.iter().enumerate() {
        if let Some(src) = b.source {
            let p = cascade::ribbon(
                (src.0 + pad, src.1 + pad),
                (b.x0 + pad, b.x1 + pad),
                ry(b.level - 1) + bh,
                ry(b.level),
                12,
            );
            fills.push(poly(fade(pick(t, i), 0.35), p));
        }
        fills.push(rect(
            pick(t, i),
            b.x0 + pad,
            ry(b.level),
            (b.x1 - b.x0).max(1.0),
            bh,
        ));
    }
    plot(w, h, fills, Vec::new())
}

/// A force-directed network.
pub fn network(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let edges = [
        (0, 1, 1.0),
        (0, 2, 1.0),
        (0, 3, 1.0),
        (1, 4, 1.0),
        (1, 5, 1.0),
        (2, 6, 1.0),
        (3, 7, 1.0),
        (4, 5, 1.0),
        (6, 7, 1.0),
        (2, 3, 1.0),
        (7, 8, 1.0),
        (8, 9, 1.0),
        (5, 9, 1.0),
    ];
    let pos = force::force_layout(10, &edges, 120);
    let pad = 16.0;
    let at = |i: usize| {
        let (x, y) = pos[i];
        (
            pad + x * (w - 2.0 * pad) + 3.0 * (now * 0.8 + i as f32).sin(),
            pad + y * (h - 2.0 * pad) + 3.0 * (now * 0.7 + i as f32 * 1.3).cos(),
        )
    };
    let mut lines: Vec<_> = edges
        .iter()
        .map(|&(a, b, _)| stroke(fade(t.palette.text_muted, 0.55), 1.5, vec![at(a), at(b)]))
        .collect();
    for i in 0..10 {
        lines.push(dot(pick(t, i / 3), at(i), 13.0));
    }
    plot(w, h, Vec::new(), lines)
}

/// An arc diagram: nodes on a line, arcs above.
pub fn arc_diagram(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let pad = 14.0;
    let n = 10;
    let x = |i: usize| pad + (w - 2.0 * pad) * i as f32 / (n - 1) as f32;
    let base = h - pad;
    let mut lines = Vec::new();
    for (k, &(a, b)) in [
        (0, 3),
        (1, 6),
        (2, 4),
        (3, 9),
        (5, 8),
        (6, 9),
        (0, 7),
        (4, 5),
    ]
    .iter()
    .enumerate()
    {
        let (x0, x1) = (x(a), x(b));
        let r = (x1 - x0) / 2.0;
        let r_y = (r * (0.9 + 0.1 * (now * 0.8 + k as f32).sin())).min(base - pad);
        let pts = (0..=24).map(|s| {
            let u = s as f32 / 24.0 * std::f32::consts::PI;
            (x0 + r - r * u.cos(), base - r_y * u.sin())
        });
        lines.push(stroke(fade(pick(t, k), 0.7), 2.0, pts.collect()));
    }
    for i in 0..n {
        lines.push(dot(t.palette.text, (x(i), base), 8.0));
    }
    plot(w, h, Vec::new(), lines)
}

/// Parallel coordinates: one polyline per item across the axes.
pub fn parallel(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let pad = 14.0;
    let axes = 5;
    let x = |a: usize| pad + (w - 2.0 * pad) * a as f32 / (axes - 1) as f32;
    let mut lines: Vec<_> = (0..axes)
        .map(|a| {
            stroke(
                fade(t.palette.border_strong, 0.9),
                1.5,
                vec![(x(a), pad), (x(a), h - pad)],
            )
        })
        .collect();
    for k in 0..8 {
        let pts = (0..axes)
            .map(|a| {
                let v = wander(
                    now * 0.4,
                    0.5 + 0.35 * ((k * 5 + a * 3) as f32 * 1.3).sin(),
                    (k * 7 + a) as f32,
                );
                (x(a), h - pad - v * (h - 2.0 * pad))
            })
            .collect();
        lines.push(stroke(fade(pick(t, k / 3), 0.8), 2.0, pts));
    }
    plot(w, h, Vec::new(), lines)
}

/// Hexagonal bins of a point cloud.
pub fn hex_bins(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let r = 14.0;
    let pts: Vec<(f32, f32)> = (0..400)
        .map(|i| {
            (
                w / 2.0 + gauss(i, 3.0) * w * 0.17 + 6.0 * (now * 0.3).sin(),
                h / 2.0 + gauss(i, 9.0) * h * 0.17,
            )
        })
        .collect();
    let bins = hexbin::hexbin(&pts, r);
    let top = bins.iter().map(|b| b.count).max().unwrap_or(1) as f32;
    let fills = bins
        .iter()
        .map(|b| {
            poly(
                fade(t.palette.primary, 0.15 + 0.85 * b.count as f32 / top),
                hexbin::hex_corners(b.x, b.y, r - 1.0).to_vec(),
            )
        })
        .collect();
    plot(w, h, fills, Vec::new())
}

/// A bump chart: each series' rank over time.
pub fn bump(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let pad = 16.0;
    let (series_n, steps) = (5, 6);
    let rows: Vec<Vec<f32>> = (0..series_n)
        .map(|s| {
            (0..steps)
                .map(|i| {
                    10.0 + 6.0 * ((i as f32 * 0.9 + s as f32 * 1.7) + now * 0.2).sin() + s as f32
                })
                .collect()
        })
        .collect();
    let r = rank::ranks(&rows);
    let x = |i: usize| pad + (w - 2.0 * pad) * i as f32 / (steps - 1) as f32;
    let y = |k: usize| pad + (h - 2.0 * pad) * k as f32 / (series_n - 1) as f32;
    let mut lines = Vec::new();
    for s in 0..series_n {
        lines.push(stroke(
            pick(t, s),
            3.0,
            (0..steps).map(|i| (x(i), y(r[s][i]))).collect(),
        ));
        for i in 0..steps {
            lines.push(dot(pick(t, s), (x(i), y(r[s][i])), 10.0));
        }
    }
    plot(w, h, Vec::new(), lines)
}

/// Layered areas: stacked, a stream graph or a 100% stack.
pub fn streams(t: &Theme, w: f32, h: f32, now: f32, offset: StackOffset) -> Plot {
    let n = 24;
    let vals: Vec<Vec<f32>> = (0..4)
        .map(|k| {
            (0..n)
                .map(|i| 2.0 + 1.6 * ((i as f32 * 0.4 + k as f32 * 1.5 + now * 0.4).sin() + 1.0))
                .collect()
        })
        .collect();
    let layers = stack_layers(&vals, offset);
    let (lo, hi) = dodai_ui_charts::layers::layers_extent(&layers);
    let pad = 8.0;
    let x = |i: usize| pad + (w - 2.0 * pad) * i as f32 / (n - 1) as f32;
    let y = |v: f32| h - pad - (v - lo) / (hi - lo).max(f32::EPSILON) * (h - 2.0 * pad);
    let fills = layers
        .iter()
        .enumerate()
        .map(|(k, l)| {
            let mut p: Vec<(f32, f32)> = l
                .iter()
                .enumerate()
                .map(|(i, (_, up))| (x(i), y(*up)))
                .collect();
            p.extend(
                l.iter()
                    .enumerate()
                    .rev()
                    .map(|(i, (low, _))| (x(i), y(*low))),
            );
            poly(fade(pick(t, k), 0.85), p)
        })
        .collect();
    plot(w, h, fills, Vec::new())
}

/// Ridgeline: offset, overlapping density curves.
pub fn ridgeline(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let pad = 10.0;
    let rows = 5;
    let row = (h - 2.0 * pad) / rows as f32;
    let n = 40;
    let mut fills = Vec::new();
    let mut lines = Vec::new();
    for k in 0..rows {
        let base = pad + row * (k as f32 + 1.0);
        let c = 0.25 + 0.12 * k as f32 + 0.04 * (now * 0.5 + k as f32).sin();
        let pts: Vec<(f32, f32)> = (0..n)
            .map(|i| {
                let f = i as f32 / (n - 1) as f32;
                (
                    pad + f * (w - 2.0 * pad),
                    base - row * 1.6 * (-((f - c) * 5.0).powi(2)).exp(),
                )
            })
            .collect();
        let mut p = pts.clone();
        p.push((w - pad, base));
        p.push((pad, base));
        fills.push(poly(fade(pick(t, k), 0.65), p));
        lines.push(stroke(pick(t, k), 1.5, pts));
    }
    plot(w, h, fills, lines)
}

/// Horizon charts: signed values folded into stacked bands.
pub fn horizon_strips(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let p = &t.palette;
    let (rows, bands, n) = (3, 3, 60);
    let strip = h / rows as f32;
    let mut fills = Vec::new();
    for r in 0..rows {
        let vals: Vec<f32> = (0..n)
            .map(|i| {
                (i as f32 * 0.25 + r as f32 * 2.0 + now * 0.5).sin() * 3.0 + (i as f32 * 0.07).cos()
            })
            .collect();
        let band = horizon::band_for(&vals, bands);
        let cw = w / n as f32;
        let base = strip * (r as f32 + 1.0) - 2.0;
        // One stepped polygon per band and sign, not a rectangle per cell: the page redraws often.
        for b in 0..bands {
            for (up, colour) in [(true, p.info), (false, p.danger)] {
                let mut top = Vec::new();
                for (i, v) in vals.iter().enumerate() {
                    let f = if (*v >= 0.0) == up {
                        horizon::fold(*v, band, bands)[b]
                    } else {
                        0.0
                    };
                    let y = base - f * (strip - 4.0);
                    top.push((i as f32 * cw, y));
                    top.push(((i + 1) as f32 * cw, y));
                }
                if top.iter().any(|q| q.1 < base) {
                    let mut poly_pts = vec![(0.0, base)];
                    poly_pts.extend(top);
                    poly_pts.push((w, base));
                    fills.push(poly(fade(colour, 0.35 + 0.3 * b as f32), poly_pts));
                }
            }
        }
    }
    plot(w, h, fills, Vec::new())
}

/// A marimekko: variable-width columns split into segments.
pub fn marimekko(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let cols: Vec<Vec<f32>> = (0..4)
        .map(|c| {
            (0..3)
                .map(|s| {
                    2.0 + (c + s) as f32 % 3.0 + 1.5 * (now * 0.5 + (c * 3 + s) as f32).sin().abs()
                })
                .collect()
        })
        .collect();
    let tiles = share::marimekko(&cols);
    let mut fills = Vec::new();
    for col in &tiles {
        for (s, tl) in col.iter().enumerate() {
            fills.push(rect(
                pick(t, s),
                tl.x0 * w + 1.0,
                tl.y0 * h + 1.0,
                (tl.x1 - tl.x0) * w - 2.0,
                (tl.y1 - tl.y0) * h - 2.0,
            ));
        }
    }
    plot(w, h, fills, Vec::new())
}

/// A waffle: 100 cells coloured by share.
pub fn waffle(t: &Theme, size: f32, now: f32) -> Plot {
    let v = [0.46 + 0.04 * (now * 0.7).sin(), 0.28, 0.16, 0.10];
    let counts = share::waffle_cells(&v, 100);
    let cells: Vec<usize> = counts
        .iter()
        .enumerate()
        .flat_map(|(k, n)| std::iter::repeat_n(k, *n))
        .collect();
    let step = size / 10.0;
    let fills = cells
        .iter()
        .enumerate()
        .map(|(i, c)| {
            rect(
                pick(t, *c),
                (i % 10) as f32 * step + 1.0,
                (9 - i / 10) as f32 * step + 1.0,
                step - 2.0,
                step - 2.0,
            )
        })
        .collect();
    plot(size, size, fills, Vec::new())
}

/// A sunburst: the hierarchy as concentric rings.
pub fn sunburst(t: &Theme, size: f32, now: f32) -> Plot {
    let spans = partition(&tree(now));
    let depth = spans.iter().map(|s| s.depth).max().unwrap_or(0) + 1;
    let c = (size / 2.0, size / 2.0);
    let ring = (size / 2.0 - 4.0) / (depth as f32 + 0.6);
    let mut fills = Vec::new();
    for s in &spans {
        let (r0, r1) = (
            ring * (0.6 + s.depth as f32),
            ring * (1.6 + s.depth as f32) - 2.0,
        );
        let (a0, a1) = (
            s.start * TAU - TAU / 4.0 + 0.01,
            s.end * TAU - TAU / 4.0 - 0.01,
        );
        let mut p = arc(c, r1, a0, a1);
        let mut inner = arc(c, r0, a0, a1);
        inner.reverse();
        p.extend(inner);
        fills.push(poly(
            fade(pick(t, s.branch), 1.0 - 0.18 * s.depth as f32),
            p,
        ));
    }
    plot(size, size, fills, Vec::new())
}

/// An icicle: the same hierarchy as stacked bars.
pub fn icicle(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let spans = partition(&tree(now));
    let depth = spans.iter().map(|s| s.depth).max().unwrap_or(0) + 1;
    let row = h / depth as f32;
    let fills = spans
        .iter()
        .map(|s| {
            rect(
                fade(pick(t, s.branch), 1.0 - 0.18 * s.depth as f32),
                s.start * w + 1.0,
                s.depth as f32 * row + 1.0,
                (s.end - s.start) * w - 2.0,
                row - 2.0,
            )
        })
        .collect();
    plot(w, h, fills, Vec::new())
}

/// Circle packing: nested discs.
pub fn circle_pack(t: &Theme, size: f32, now: f32) -> Plot {
    let discs = pack::circle_pack(&tree(now));
    let mut fills = Vec::new();
    let mut lines = Vec::new();
    for d in &discs {
        let c = (d.x * size, d.y * size);
        let r = d.r * size;
        if d.leaf {
            fills.push(poly(fade(pick(t, d.branch), 0.9), arc(c, r, 0.0, TAU)));
        } else {
            lines.push(stroke(
                fade(pick(t, d.branch), 0.7),
                1.5,
                arc(c, r, 0.0, TAU),
            ));
        }
    }
    plot(size, size, fills, lines)
}

/// Proportional squares: area is the value.
pub fn squares(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let vals: Vec<f32> = [52.0, 28.0, 18.0, 12.0, 5.0]
        .iter()
        .enumerate()
        .map(|(i, v)| v * (0.9 + 0.1 * (now * 0.6 + i as f32).sin()))
        .collect();
    let sq = pack::square_pack(&vals, h / w);
    let fills = sq
        .iter()
        .enumerate()
        .map(|(i, s)| {
            rect(
                pick(t, i),
                s.x * w + 1.0,
                s.y * w + 1.0,
                s.side * w - 2.0,
                s.side * w - 2.0,
            )
        })
        .collect();
    plot(w, h, fills, Vec::new())
}

/// A shard map: an outline cut into cells sized by value.
pub fn shard_map(t: &Theme, size: f32, now: f32) -> Plot {
    let groups: Vec<Vec<f32>> = (0..3)
        .map(|g| {
            (0..4)
                .map(|i| 3.0 + ((g * 4 + i) % 5) as f32 + (now * 0.5 + (g + i) as f32).sin())
                .collect()
        })
        .collect();
    let outline = shards::Outline::Circle.polygon();
    let (cells, _) = shards::layout(&groups, &outline);
    let k = size / 2.0 - 4.0;
    let fills = cells
        .iter()
        .filter(|c| !c.poly.is_empty())
        .map(|c| {
            poly(
                fade(pick(t, c.group), 0.55 + 0.1 * c.item as f32),
                c.poly
                    .iter()
                    .map(|(x, y)| (size / 2.0 + x * k, size / 2.0 + y * k))
                    .collect(),
            )
        })
        .collect();
    plot(size, size, fills, Vec::new())
}

/// A span fan: lifespans as spokes round an arc.
pub fn span_fan(t: &Theme, size: f32, now: f32) -> Plot {
    let items: Vec<fan::Span> = (0..14)
        .map(|i| {
            let s = (i * 7 % 10) as f32;
            fan::Span {
                start: s,
                end: s + 4.0 + (i * 3 % 7) as f32 + (now * 0.5 + i as f32).sin(),
                peak: Some((s + 1.0, s + 2.5)),
                magnitude: 1.0 + (i % 5) as f32,
            }
        })
        .collect();
    let spokes = fan::layout(&items, 0.8, fan::FanOrder::Start);
    let c = (size / 2.0, size * 0.55);
    let (hub, rim) = (size * 0.1, size * 0.46);
    let pt = |turn: f32, f: f32| {
        let a = turn * TAU - TAU / 4.0;
        (
            c.0 + (hub + (rim - hub) * f) * a.cos(),
            c.1 + (hub + (rim - hub) * f) * a.sin(),
        )
    };
    let mut lines = Vec::new();
    for (i, sp) in spokes.iter().enumerate() {
        lines.push(stroke(
            fade(pick(t, i), 0.55),
            2.0,
            vec![pt(sp.turn, 0.0), pt(sp.turn, sp.len)],
        ));
        if let Some((a, b)) = sp.peak {
            lines.push(stroke(
                pick(t, i),
                5.0,
                vec![pt(sp.turn, a), pt(sp.turn, b)],
            ));
        }
        lines.push(dot(pick(t, i), pt(sp.turn, sp.len), 4.0 + 10.0 * sp.mark));
    }
    plot(size, size, Vec::new(), lines)
}

/// A peak range: overlapping series as mountains.
pub fn peak_range(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let pad = 8.0;
    let series_v: Vec<Vec<f32>> = (0..4)
        .map(|k| {
            (0..40)
                .map(|i| {
                    let f = i as f32 / 39.0;
                    (-((f - 0.2 - 0.2 * k as f32 - 0.03 * (now * 0.5 + k as f32).sin()) * 6.0)
                        .powi(2))
                    .exp()
                        * (0.5 + 0.1 * k as f32)
                })
                .collect()
        })
        .collect();
    let order = peaks::back_to_front(&series_v);
    let mut fills = Vec::new();
    let mut lines = Vec::new();
    for &k in &order {
        let pts: Vec<(f32, f32)> = series_v[k]
            .iter()
            .enumerate()
            .map(|(i, v)| {
                (
                    pad + (w - 2.0 * pad) * i as f32 / 39.0,
                    h - pad - v * (h - 2.0 * pad),
                )
            })
            .collect();
        let mut p = pts.clone();
        p.push((w - pad, h - pad));
        p.push((pad, h - pad));
        fills.push(poly(fade(pick(t, k), 0.6), p));
        lines.push(stroke(pick(t, k), 2.0, pts));
    }
    plot(w, h, fills, lines)
}

/// A time scatter with its record frontier.
pub fn time_scatter(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let pad = 12.0;
    let pts: Vec<(f32, f32)> = (0..26)
        .map(|i| {
            (
                i as f32 / 25.0,
                0.15 + 0.7 * ((i * 13 % 17) as f32 / 17.0) * (0.4 + i as f32 / 40.0)
                    + 0.04 * (now * 0.5 + i as f32).sin(),
            )
        })
        .collect();
    let px = |(x, y): (f32, f32)| (pad + x * (w - 2.0 * pad), h - pad - y * (h - 2.0 * pad));
    let mut lines = grid(t, w, h, pad);
    lines.push(stroke(
        t.palette.accent,
        2.5,
        milestones::frontier(&pts).into_iter().map(px).collect(),
    ));
    lines.extend(
        pts.iter()
            .map(|p| dot(fade(t.palette.primary, 0.8), px(*p), 8.0)),
    );
    plot(w, h, Vec::new(), lines)
}

/// A beeswarm: dots on an axis, nudged apart.
pub fn beeswarm(t: &Theme, w: f32, h: f32, now: f32) -> Plot {
    let pad = 14.0;
    let xs: Vec<f32> = (0..60)
        .map(|i| {
            ((0.5 + 0.17 * gauss(i, 5.0) + 0.02 * (now * 0.3 + i as f32).sin()).clamp(0.0, 1.0))
                * (w - 2.0 * pad)
        })
        .collect();
    let off = swarm::swarm(&xs, 5.0);
    let mut lines = vec![stroke(
        fade(t.palette.border_strong, 0.9),
        1.5,
        vec![(pad, h / 2.0), (w - pad, h / 2.0)],
    )];
    for (i, (x, o)) in xs.iter().zip(&off).enumerate() {
        lines.push(dot(fade(pick(t, i % 3), 0.85), (pad + x, h / 2.0 + o), 9.0));
    }
    plot(w, h, Vec::new(), lines)
}
