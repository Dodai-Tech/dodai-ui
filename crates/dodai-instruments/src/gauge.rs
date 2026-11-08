//! Geometry shared by gauges and signal instruments: lit cells, ticks, zones, needle and cell
//! polygons, hex grids, ring radii and the demo signals (ECG beat, radar blip fade, bar jitter).
//! Pure, so every toolkit draws the same shapes.

use dodai_charts::math::polar;

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
    let tip = dodai_charts::math::polar(cx, cy, len, at);
    let l = dodai_charts::math::polar(cx, cy, base / 2.0, at - 0.25);
    let r = dodai_charts::math::polar(cx, cy, base / 2.0, at + 0.25);
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

/// Stadium (pill) outline: a `w`×`h` rectangle with fully round short ends.
pub fn pill(x: f32, y: f32, w: f32, h: f32) -> Vec<(f32, f32)> {
    let r = (w.min(h) / 2.0).max(0.0);
    let n = 8;
    let mut out = Vec::with_capacity(n * 2 + 2);
    if w >= h {
        // right cap (top → bottom), then left cap (bottom → top)
        for i in 0..=n {
            let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / n as f32;
            out.push((x + w - r + r * a.cos(), y + r + r * a.sin()));
        }
        for i in 0..=n {
            let a = std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / n as f32;
            out.push((x + r + r * a.cos(), y + r + r * a.sin()));
        }
    } else {
        // bottom cap (right → left), then top cap (left → right)
        for i in 0..=n {
            let a = std::f32::consts::PI * i as f32 / n as f32;
            out.push((x + r + r * a.cos(), y + h - r + r * a.sin()));
        }
        for i in 0..=n {
            let a = std::f32::consts::PI + std::f32::consts::PI * i as f32 / n as f32;
            out.push((x + r + r * a.cos(), y + r + r * a.sin()));
        }
    }
    out
}

/// Lit bars out of `n` for `frac` (at least one when any value is set). Pure.
pub fn lit_bars(frac: f32, n: usize) -> usize {
    if frac.is_nan() || frac <= 0.0 || n == 0 {
        0
    } else {
        ((frac.clamp(0.0, 1.0) * n as f32).ceil() as usize).clamp(1, n)
    }
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

/// Roll position of every digit of a mechanical counter showing `value`, least significant
/// digit first. Position `p` means the strip sits at digit `p`, so `3.5` is halfway between 3
/// and 4. The ones digit follows the value; each higher digit waits until the digit below
/// wraps (the last tenth of its turn) and then rolls one step, like an odometer.
pub fn odometer_positions(value: f32, digits: usize) -> Vec<f32> {
    let v = if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    };
    (0..digits)
        .map(|k| {
            let scaled = v / 10f32.powi(k as i32);
            let whole = scaled.floor();
            let roll = if k == 0 {
                scaled.fract()
            } else {
                ((scaled.fract() - 0.9) / 0.1).clamp(0.0, 1.0)
            };
            (whole + roll).rem_euclid(10.0)
        })
        .collect()
}

/// 270° dial sweep with the gap at the bottom, as `(start, span)` in turns: starts at 7:30 going
/// clockwise.
pub const SWEEP: (f32, f32) = (0.625, 0.75);

/// A closed circle of radius `r` around `(c, c)` as 96 segments.
pub fn ring_points(c: f32, r: f32) -> Vec<(f32, f32)> {
    (0..=96).map(|i| polar(c, c, r, i as f32 / 96.0)).collect()
}

/// `(fraction, value)` of every major tick of a scale, for numbering a dial.
pub fn major_labels(scale: &crate::instrument::Scale) -> Vec<(f32, f32)> {
    scale
        .ticks()
        .into_iter()
        .filter(|k| k.major)
        .map(|k| (k.frac, k.value))
        .collect()
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
    fn chevron_points_right_and_chamfer_cuts_one_corner() {
        let c = chevron_cell(0.0, 0.0, 10.0, 10.0, 4.0);
        assert!(c[2].0 > c[1].0 && c[5].0 > c[0].0);
        let r = chamfer_rect(0.0, 0.0, 10.0, 10.0, 3.0);
        assert_eq!((r[1], r[2]), ((7.0, 0.0), (10.0, 3.0)));
    }

    #[test]
    fn pill_stays_inside_its_box_and_is_round_ended() {
        for (w, h) in [(40.0, 10.0), (10.0, 40.0), (12.0, 12.0)] {
            let p = pill(2.0, 3.0, w, h);
            assert!(p.len() >= 16);
            for (x, y) in p {
                assert!(x >= 2.0 - 1e-3 && x <= 2.0 + w + 1e-3, "x {x}");
                assert!(y >= 3.0 - 1e-3 && y <= 3.0 + h + 1e-3, "y {y}");
            }
        }
    }

    #[test]
    fn signal_bars_light_up_in_steps() {
        assert_eq!(lit_bars(0.0, 5), 0);
        assert_eq!(lit_bars(0.01, 5), 1);
        assert_eq!(lit_bars(0.5, 5), 3);
        assert_eq!(lit_bars(1.0, 5), 5);
        assert_eq!(lit_bars(9.0, 5), 5);
        assert_eq!(lit_bars(1.0, 0), 0);
        assert_eq!(lit_bars(f32::NAN, 5), 0);
    }

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
    fn odometer_higher_digits_wait_for_the_wrap() {
        let at = |v: f32| odometer_positions(v, 3);
        assert_eq!(at(0.0), vec![0.0, 0.0, 0.0]);
        assert!((at(7.5)[0] - 7.5).abs() < 1e-4 && at(7.5)[1] == 0.0);
        // 19.95: tens has started to roll from 1 towards 2
        let p = at(19.95);
        assert!(p[1] > 1.0 && p[1] < 2.0, "{p:?}");
        // 20.0: settled
        assert!((at(20.0)[1] - 2.0).abs() < 1e-4);
        assert!(at(f32::NAN).iter().all(|x| *x == 0.0));
        assert!(at(1e6).iter().all(|x| (0.0..10.0).contains(x)));
    }

    #[test]
    fn ring_closes_and_labels_are_the_majors() {
        let r = ring_points(50.0, 40.0);
        assert_eq!(r.len(), 97);
        assert!((r[0].0 - r[96].0).abs() < 1e-3 && (r[0].1 - r[96].1).abs() < 1e-3);
        let s = crate::instrument::Scale::new(0.0, 100.0);
        let l = major_labels(&s);
        assert!(l.first().is_some_and(|m| m.0 == 0.0));
        assert!(l.iter().all(|m| (0.0..=1.0).contains(&m.0)));
        assert!((SWEEP.0 + SWEEP.1 - 1.375).abs() < 1e-6);
    }
}
