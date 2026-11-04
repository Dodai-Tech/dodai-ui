//! The aviation group: flight and airfield symbols.

use super::*;

/// A top-view airliner nose-up, centred at `(cx, cy)`, `s` tall.
fn jet(cx: f32, cy: f32, s: f32) -> Vec<Shape> {
    let k = s / 20.0;
    let p = |x: f32, y: f32| (cx + x * k, cy + y * k);
    let mut v = vec![(
        vec![
            p(0.0, -10.0),
            p(1.5, -6.0),
            p(1.5, -2.0),
            p(10.0, 3.0),
            p(10.0, 5.0),
            p(1.5, 3.0),
            p(1.2, 7.5),
            p(4.5, 9.5),
            p(4.5, 10.5),
            p(0.0, 9.5),
            p(-4.5, 10.5),
            p(-4.5, 9.5),
            p(-1.2, 7.5),
            p(-1.5, 3.0),
            p(-10.0, 5.0),
            p(-10.0, 3.0),
            p(-1.5, -2.0),
            p(-1.5, -6.0),
        ],
        true,
    )];
    // Engine pods only where there is room for them.
    if s >= 16.0 {
        v.push((
            vec![p(5.6, -0.6), p(5.6, 2.6), p(6.8, 2.6), p(6.8, -0.1)],
            false,
        ));
        v.push((
            vec![p(-5.6, -0.6), p(-5.6, 2.6), p(-6.8, 2.6), p(-6.8, -0.1)],
            false,
        ));
    }
    v
}

/// Rotates every shape by `deg` degrees (clockwise on screen) about `(cx, cy)`.
fn turn(shapes: Vec<Shape>, cx: f32, cy: f32, deg: f32) -> Vec<Shape> {
    let (s, c) = rad(deg).sin_cos();
    shapes
        .into_iter()
        .map(|(pts, closed)| {
            let pts = pts
                .into_iter()
                .map(|(x, y)| {
                    let (dx, dy) = (x - cx, y - cy);
                    (cx + dx * c - dy * s, cy + dx * s + dy * c)
                })
                .collect();
            (pts, closed)
        })
        .collect()
}

/// `n` points of a wave of amplitude `amp` and `cycles` periods from `x0` to `x1` about `y`.
fn wave(x0: f32, x1: f32, y: f32, amp: f32, cycles: f32, phase: f32) -> Shape {
    let n = 28;
    let pts = (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            (
                x0 + (x1 - x0) * t,
                y + amp * (phase + t * cycles * 2.0 * PI).sin(),
            )
        })
        .collect();
    (pts, false)
}

pub(super) fn shapes(i: Symbol) -> Vec<Shape> {
    use Symbol::*;
    match i {
        Airliner => jet(12.0, 12.0, 20.0),
        Runway => {
            let mut v = vec![
                line(&[(7.5, 22.0), (10.0, 2.0)]),
                line(&[(16.5, 22.0), (14.0, 2.0)]),
            ];
            for k in 0..4 {
                let y = 20.0 - k as f32 * 4.6;
                v.push(line(&[(12.0, y), (12.0, y - 2.4)]));
            }
            // Threshold bars at the near end.
            for k in 0..3 {
                let x = 9.3 + k as f32 * 2.7;
                v.push(line(&[(x, 21.4), (x, 18.8)]));
            }
            v
        }
        Horizon => {
            let (s, c) = rad(-10.0).sin_cos();
            let at = |along: f32| (12.0 + along * c, 12.0 + along * s);
            vec![
                circ(12.0, 12.0, 10.0),
                line(&[at(-9.6), at(9.6)]),
                poly(&[(12.0, 4.2), (10.6, 6.8), (13.4, 6.8)]),
                line(&[(5.5, 15.5), (9.5, 15.5), (11.0, 17.5)]),
                line(&[(18.5, 15.5), (14.5, 15.5), (13.0, 17.5)]),
            ]
        }
        Altimeter => {
            let mut v = vec![circ(12.0, 12.0, 10.0), circ(12.0, 12.0, 1.0)];
            for k in 0..12 {
                let a = rad(k as f32 * 30.0 - 90.0);
                let r0 = if k % 3 == 0 { 7.0 } else { 8.0 };
                v.push(line(&[
                    (12.0 + r0 * a.cos(), 12.0 + r0 * a.sin()),
                    (12.0 + 8.8 * a.cos(), 12.0 + 8.8 * a.sin()),
                ]));
            }
            v.push(line(&[(12.0, 12.0), (9.0, 8.0)]));
            v.push(line(&[(12.0, 12.0), (16.0, 6.8)]));
            v
        }
        Flaps => vec![
            poly(&[
                (2.5, 12.5),
                (4.5, 9.0),
                (8.0, 7.5),
                (15.0, 8.0),
                (15.0, 12.5),
            ]),
            poly(&[(15.0, 8.0), (21.5, 13.5), (19.8, 15.5), (15.0, 12.5)]),
            line(&[(5.0, 17.0), (12.0, 17.0)]),
            line(&[(5.0, 20.5), (9.5, 20.5)]),
            line(&[(18.0, 18.0), (18.0, 22.0)]),
            line(&[(16.4, 20.4), (18.0, 22.0), (19.6, 20.4)]),
        ],
        LandingGear => vec![
            line(&[(5.0, 3.5), (19.0, 3.5)]),
            line(&[(12.0, 3.5), (12.0, 14.0)]),
            line(&[(12.0, 7.0), (6.5, 12.0)]),
            circ(12.0, 18.4, 4.2),
            circ(12.0, 18.4, 1.4),
            line(&[(5.0, 22.8), (19.0, 22.8)]),
        ],
        Tower => vec![
            poly(&[(6.0, 3.5), (18.0, 3.5), (20.5, 8.5), (3.5, 8.5)]),
            line(&[(9.0, 3.5), (8.2, 8.5)]),
            line(&[(12.0, 3.5), (12.0, 8.5)]),
            line(&[(15.0, 3.5), (15.8, 8.5)]),
            line(&[(12.0, 3.5), (12.0, 0.8)]),
            line(&[(10.4, 2.0), (13.6, 2.0)]),
            line(&[(3.5, 10.4), (20.5, 10.4)]),
            line(&[(10.0, 8.5), (9.2, 22.0)]),
            line(&[(14.0, 8.5), (14.8, 22.0)]),
            line(&[(9.6, 14.0), (14.4, 14.0)]),
            line(&[(9.4, 18.0), (14.6, 18.0)]),
            line(&[(6.5, 22.0), (17.5, 22.0)]),
        ],
        Windsock => {
            let mut v = vec![
                line(&[(5.0, 2.0), (5.0, 22.0)]),
                line(&[(3.0, 22.0), (7.0, 22.0)]),
                // The sock: wide mouth at the pole, narrow tail sagging downwind.
                line(&[(5.0, 4.0), (11.0, 4.8), (16.0, 6.6), (21.0, 9.4)]),
                line(&[(5.0, 12.5), (11.0, 12.6), (16.0, 12.0), (21.0, 11.6)]),
                line(&[(21.0, 9.4), (21.0, 11.6)]),
                line(&[(8.0, 4.4), (8.0, 12.6)]),
            ];
            for &(x, t0, t1) in &[(11.6, 4.8, 12.6), (16.2, 6.7, 12.0)] {
                v.push(line(&[(x, t0), (x, t1)]));
            }
            v
        }
        Heading => {
            let mut v = vec![circ(12.0, 12.5, 9.5)];
            for k in 0..12 {
                let a = rad(k as f32 * 30.0 - 90.0);
                let r0 = if k % 3 == 0 { 6.6 } else { 7.8 };
                v.push(line(&[
                    (12.0 + r0 * a.cos(), 12.5 + r0 * a.sin()),
                    (12.0 + 8.5 * a.cos(), 12.5 + 8.5 * a.sin()),
                ]));
            }
            v.extend(jet(12.0, 12.5, 8.0));
            v.push(poly(&[(12.0, 0.8), (10.4, 3.0), (13.6, 3.0)]));
            v
        }
        Turbine => {
            let mut v = vec![circ(12.0, 12.0, 10.0), circ(12.0, 12.0, 2.2)];
            // Swept fan blades.
            for k in 0..7 {
                let a0 = k as f32 * (360.0 / 7.0);
                let pts = (0..=6)
                    .map(|j| {
                        let t = j as f32 / 6.0;
                        let (r, a) = (2.8 + 5.4 * t, rad(a0 + 38.0 * t));
                        (12.0 + r * a.cos(), 12.0 + r * a.sin())
                    })
                    .collect::<Vec<_>>();
                v.push((pts, false));
            }
            v
        }
        Propeller => {
            let blade = [
                (0.0, -2.4),
                (-2.3, -5.6),
                (-1.8, -9.4),
                (0.4, -10.4),
                (2.0, -8.0),
                (1.6, -4.6),
            ];
            let mut v = Vec::new();
            for k in 0..3 {
                let (s, c) = rad(k as f32 * 120.0).sin_cos();
                let pts: Vec<(f32, f32)> = blade
                    .iter()
                    .map(|&(x, y)| (12.0 + x * c - y * s, 12.5 + x * s + y * c))
                    .collect();
                v.push(curve(&pts, true, 5));
            }
            v.push(circ(12.0, 12.5, 1.6));
            v
        }
        GlideSlope => vec![
            line(&[(2.0, 21.0), (22.0, 21.0)]),
            line(&[(20.0, 20.0), (3.0, 7.0)]),
            line(&[(20.0, 20.0), (3.0, 14.5)]),
            line(&[(16.0, 18.4), (12.5, 16.4)]),
            line(&[(9.5, 14.8), (6.5, 13.0)]),
        ],
        Transponder => vec![
            rrect(3.0, 10.0, 21.0, 21.0, 2.0),
            rrect(5.0, 12.0, 14.0, 16.0, 0.8),
            line(&[(7.0, 13.4), (7.0, 14.6)]),
            line(&[(9.0, 13.4), (9.0, 14.6)]),
            line(&[(11.0, 13.4), (11.0, 14.6)]),
            circ(17.6, 14.0, 1.8),
            line(&[(5.5, 18.6), (8.0, 18.6)]),
            line(&[(10.0, 18.6), (12.5, 18.6)]),
            line(&[(12.0, 10.0), (12.0, 7.6)]),
            circ(12.0, 7.0, 0.6),
            arc(12.0, 7.0, 3.6, 215.0, 325.0),
            arc(12.0, 7.0, 6.4, 215.0, 325.0),
        ],
        Takeoff => {
            let mut v = vec![line(&[(2.0, 21.0), (22.0, 21.0)])];
            v.extend(turn(jet(14.0, 10.0, 13.0), 14.0, 10.0, 50.0));
            for &(x, y) in &[(3.0, 19.0), (5.0, 16.4), (7.0, 13.8)] {
                v.push(line(&[(x, y), (x + 1.2, y - 1.4)]));
            }
            v.push(line(&[(18.0, 21.0), (19.5, 21.0)]));
            v
        }
        Landing => {
            let mut v = vec![line(&[(2.0, 21.0), (22.0, 21.0)])];
            v.extend(turn(jet(10.0, 11.0, 13.0), 10.0, 11.0, 125.0));
            for &(x, y) in &[(14.6, 13.6), (17.0, 15.8), (19.4, 18.0)] {
                v.push(line(&[(x, y), (x + 1.3, y + 1.3)]));
            }
            v
        }
        Turbulence => vec![
            wave(2.0, 22.0, 6.0, 2.0, 2.0, 0.0),
            wave(2.0, 22.0, 12.0, 2.5, 2.0, 1.6),
            wave(2.0, 14.0, 18.0, 2.0, 1.2, 0.0),
            line(&[(17.0, 18.0), (18.0, 18.0)]),
            line(&[(20.0, 18.0), (21.0, 18.0)]),
        ],
        Autopilot => {
            let mut v = jet(12.0, 12.0, 12.0);
            v.push(arc(12.0, 12.0, 10.0, 205.0, 335.0));
            v.push(arc(12.0, 12.0, 10.0, 25.0, 155.0));
            v.push(poly(&[(20.2, 5.6), (22.4, 7.8), (19.0, 8.6)]));
            v.push(poly(&[(3.8, 18.4), (1.6, 16.2), (5.0, 15.4)]));
            v
        }
        Parachute => {
            let mut v = vec![arc(12.0, 11.0, 9.0, 180.0, 360.0)];
            v.push(line(&[(3.0, 11.0), (21.0, 11.0)]));
            v.push(line(&[(7.0, 2.8), (9.0, 11.0)]));
            v.push(line(&[(17.0, 2.8), (15.0, 11.0)]));
            for &x in &[3.0, 9.0, 15.0, 21.0] {
                v.push(line(&[(x, 11.0), (12.0, 17.4)]));
            }
            v.push(circ(12.0, 18.6, 1.2));
            v.push(line(&[(12.0, 19.8), (12.0, 22.4)]));
            v
        }
        #[cfg(any(
            feature = "symbols-vehicle",
            feature = "symbols-medical",
            feature = "symbols-military",
            feature = "symbols-industrial"
        ))]
        _ => Vec::new(),
    }
}
