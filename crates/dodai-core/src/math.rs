//! Pure numeric and geometry helpers shared by every chart renderer.

/// `v` rounded up to a clean scale top (1, 1.5, 2, 2.5, 3, 4, 5, 6 or 8 times a power of
/// ten; 420 → 500, 96 → 100). Not positive or not finite: `v` unchanged.
pub fn nice_ceiling(v: f32) -> f32 {
    if !(v > 0.0 && v.is_finite()) {
        return v;
    }
    let m = 10f32.powf(v.log10().floor());
    [1.0, 1.5, 2.0, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0]
        .into_iter()
        .map(|f| f * m)
        .find(|top| *top >= v * (1.0 - 1e-6))
        .unwrap_or(10.0 * m)
}

/// "Nice" axis ticks covering `min..=max` with about `count` steps. Pure.
///
/// Always at least two strictly increasing ticks. Worked out in `f64`, so a narrow range at a
/// large magnitude (timestamps, `1e9..1e9 + 60`) gives a few distinct ticks instead of
/// duplicates. `max <= min` covers `min..min + 1`; a non-finite end is replaced by the other
/// one (both: `0..1`).
pub fn nice_ticks(min: f32, max: f32, count: usize) -> Vec<f32> {
    let (min, max) = match (min.is_finite(), max.is_finite()) {
        (true, true) => (min, max),
        (true, false) => (min, min),
        (false, true) => (max, max),
        (false, false) => (0.0, 1.0),
    };
    let (lo, hi) = if max > min {
        (f64::from(min), f64::from(max))
    } else {
        (f64::from(min), f64::from(min) + 1.0)
    };
    let raw = (hi - lo) / count.max(1) as f64;
    let mag = 10f64.powf(raw.log10().floor());
    let norm = raw / mag;
    let step = mag
        * if norm <= 1.0 {
            1.0
        } else if norm <= 2.0 {
            2.0
        } else if norm <= 5.0 {
            5.0
        } else {
            10.0
        };
    let start = (lo / step).floor();
    let mut out: Vec<f32> = Vec::new();
    for i in 0..=64 {
        let v = (start + i as f64) * step;
        if v >= hi + step * 0.999 {
            break;
        }
        let v = v.clamp(f64::from(f32::MIN), f64::from(f32::MAX)) as f32;
        if out.last().is_none_or(|l| v > *l) {
            out.push(v);
        }
    }
    match out.as_slice() {
        [] => vec![min, next_up(min)],
        [only] => vec![*only, next_up(*only)],
        _ => out,
    }
}

/// The lowest and highest finite value of `v`, or `None` when it has none.
pub fn extent(v: impl IntoIterator<Item = f32>) -> Option<(f32, f32)> {
    v.into_iter()
        .filter(|v| v.is_finite())
        .fold(None, |e, v| match e {
            None => Some((v, v)),
            Some((l, h)) => Some((v.min(l), v.max(h))),
        })
}

/// The highest finite value of `v`, but at least `floor` (so `0.0` for a non-negative scale
/// and an empty or all-negative `v`).
pub fn peak_from(v: impl IntoIterator<Item = f32>, floor: f32) -> f32 {
    extent(v).map_or(floor, |(_, hi)| hi.max(floor))
}

/// The highest finite value of `v`, never below zero: the top of a scale that starts at zero.
pub fn peak(v: impl IntoIterator<Item = f32>) -> f32 {
    peak_from(v, 0.0)
}

/// The lowest and highest finite value of `v` with zero always inside the range: the extent of
/// a bar or waterfall scale that is anchored at zero.
pub fn extent_with_zero(v: impl IntoIterator<Item = f32>) -> (f32, f32) {
    extent(v).map_or((0.0, 0.0), |(lo, hi)| (lo.min(0.0), hi.max(0.0)))
}

/// [`extent`] of `v` as a drawable range: `0..1` when there is no finite value, and one unit
/// either side of a single repeated value.
pub fn padded_extent(v: impl IntoIterator<Item = f32>) -> (f32, f32) {
    match extent(v) {
        None => (0.0, 1.0),
        Some((lo, hi)) if hi > lo => (lo, hi),
        Some((lo, _)) => (lo - 1.0, lo + 1.0),
    }
}

/// About `count` [`nice_ticks`] covering the finite values of `v` (`0..1` when there are
/// none), with the first and last tick: the axis range most charts plot against.
pub fn tick_span(v: impl IntoIterator<Item = f32>, count: usize) -> (f32, f32, Vec<f32>) {
    let (lo, hi) = extent(v).unwrap_or((0.0, 1.0));
    let ticks = nice_ticks(lo, hi, count);
    (ticks[0], ticks[ticks.len() - 1], ticks)
}

/// The smallest `f32` above a finite `v`.
fn next_up(v: f32) -> f32 {
    if v == 0.0 {
        f32::from_bits(1)
    } else if v > 0.0 {
        f32::from_bits(v.to_bits() + 1)
    } else {
        f32::from_bits(v.to_bits() - 1)
    }
}

/// Compact number label: `1200` → `1.2k`, `3.0` → `3`.
pub fn fmt_num(v: f32) -> String {
    let a = v.abs();
    if a >= 1_000_000.0 {
        format!("{}M", trim(v / 1_000_000.0))
    } else if a >= 1000.0 {
        format!("{}k", trim(v / 1000.0))
    } else {
        trim(v)
    }
}

pub fn trim(v: f32) -> String {
    let s = format!("{v:.1}");
    s.strip_suffix(".0").map(str::to_string).unwrap_or(s)
}

/// Catmull-Rom smoothing: `seg` interpolated points per input segment. Pure.
pub fn smooth_points(p: &[(f32, f32)], seg: usize) -> Vec<(f32, f32)> {
    if p.len() < 3 || seg < 2 {
        return p.to_vec();
    }
    let mut out = Vec::with_capacity(p.len() * seg);
    for i in 0..p.len() - 1 {
        let p0 = p[i.saturating_sub(1)];
        let (p1, p2) = (p[i], p[i + 1]);
        let p3 = p[(i + 2).min(p.len() - 1)];
        for s in 0..seg {
            let t = s as f32 / seg as f32;
            let (t2, t3) = (t * t, t * t * t);
            let f = |a: f32, b: f32, c: f32, d: f32| {
                0.5 * (2.0 * b
                    + (c - a) * t
                    + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
                    + (3.0 * b - a - 3.0 * c + d) * t3)
            };
            out.push((f(p0.0, p1.0, p2.0, p3.0), f(p0.1, p1.1, p2.1, p3.1)));
        }
    }
    out.push(*p.last().unwrap());
    out
}

/// Slice `(start, end)` fractions of a full turn for `values`. Pure.
pub fn pie_fractions(values: &[f32]) -> Vec<(f32, f32)> {
    let total: f32 = values.iter().map(|v| v.max(0.0)).sum();
    let mut acc = 0.0;
    values
        .iter()
        .map(|v| {
            let s = acc;
            if total > 0.0 {
                acc += v.max(0.0) / total;
            }
            (s, acc)
        })
        .collect()
}

/// Point on a circle; fraction 0 is 12 o'clock, clockwise. Pure.
pub fn polar(cx: f32, cy: f32, r: f32, frac: f32) -> (f32, f32) {
    let th = frac * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    (cx + r * th.cos(), cy + r * th.sin())
}

/// Staircase version of a polyline: horizontal then vertical. Pure.
pub fn step_points(p: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut out = Vec::with_capacity(p.len() * 2);
    for (i, &(x, y)) in p.iter().enumerate() {
        if let Some(&(_, py)) = i.checked_sub(1).and_then(|j| p.get(j)) {
            out.push((x, py));
        }
        out.push((x, y));
    }
    out
}

/// Arc points between two turn fractions. Pure.
pub fn turn_arc(cx: f32, cy: f32, r: f32, a: f32, b: f32) -> Vec<(f32, f32)> {
    let n = (((b - a).abs() * 360.0 / 4.0).ceil() as usize).max(2);
    (0..=n)
        .map(|i| polar(cx, cy, r, a + (b - a) * i as f32 / n as f32))
        .collect()
}

/// Value → `0..=1` within `lo..=hi` (0 when degenerate). Pure.
pub fn norm(v: f32, lo: f32, hi: f32) -> f32 {
    if hi > lo {
        ((v - lo) / (hi - lo)).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Radius of a mark whose *area* follows `v` (`0` at `rmin`, `vmax` at `rmax`), so a value twice
/// as big looks twice as big rather than four times. Not positive or `vmax <= 0`: `rmin`.
pub fn area_radius(v: f32, vmax: f32, rmin: f32, rmax: f32) -> f32 {
    if v > 0.0 && vmax > 0.0 {
        rmin + (v / vmax).min(1.0).sqrt() * (rmax - rmin)
    } else {
        rmin
    }
}

/// Shape drawn at sparkline points.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Marker {
    None,
    #[default]
    Dot,
    Square,
    Diamond,
    Triangle,
}

/// A circle of radius `r` around `(x, y)` as a polygon that reads as round at any size: just
/// enough sides that no side bows in more than [`CIRCLE_TOLERANCE`] px from the true circle.
pub fn circle_points(x: f32, y: f32, r: f32) -> Vec<(f32, f32)> {
    let r = if r.is_finite() { r.max(0.0) } else { 0.0 };
    // A side spanning angle `a` misses the circle by `r (1 - cos(a / 2))`.
    let n = if r <= CIRCLE_TOLERANCE {
        12
    } else {
        let a = 2.0 * (1.0 - CIRCLE_TOLERANCE / r).acos();
        ((std::f32::consts::TAU / a).ceil() as usize).clamp(12, 512)
    };
    (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            (x + r * a.cos(), y + r * a.sin())
        })
        .collect()
}

/// How far (px) a side of a [`circle_points`] polygon may fall inside the circle.
pub const CIRCLE_TOLERANCE: f32 = 0.15;

/// Outline of a marker centred at `(x, y)` with radius `r`. Pure.
pub fn marker_points(m: Marker, x: f32, y: f32, r: f32) -> Vec<(f32, f32)> {
    match m {
        Marker::None => vec![],
        Marker::Dot => circle_points(x, y, r),
        Marker::Square => vec![
            (x - r, y - r),
            (x + r, y - r),
            (x + r, y + r),
            (x - r, y + r),
        ],
        Marker::Diamond => vec![
            (x, y - r * 1.3),
            (x + r * 1.3, y),
            (x, y + r * 1.3),
            (x - r * 1.3, y),
        ],
        Marker::Triangle => vec![(x, y - r * 1.3), (x + r * 1.2, y + r), (x - r * 1.2, y + r)],
    }
}

/// `poly` clipped to the rectangle `(x0, y0)..(x1, y1)` (Sutherland-Hodgman): what of a
/// shape lies inside a plot. Empty when nothing does. Pure.
pub fn clip_polygon(
    poly: &[(f32, f32)],
    (x0, y0): (f32, f32),
    (x1, y1): (f32, f32),
) -> Vec<(f32, f32)> {
    type Edge = (
        fn((f32, f32), f32) -> bool,
        fn((f32, f32), (f32, f32), f32) -> (f32, f32),
        f32,
    );
    let at_x = |a: (f32, f32), b: (f32, f32), x: f32| {
        let t = (x - a.0) / (b.0 - a.0);
        (x, a.1 + (b.1 - a.1) * t)
    };
    let at_y = |a: (f32, f32), b: (f32, f32), y: f32| {
        let t = (y - a.1) / (b.1 - a.1);
        (a.0 + (b.0 - a.0) * t, y)
    };
    let edges: [Edge; 4] = [
        (|p, v| p.0 >= v, at_x, x0),
        (|p, v| p.0 <= v, at_x, x1),
        (|p, v| p.1 >= v, at_y, y0),
        (|p, v| p.1 <= v, at_y, y1),
    ];
    let mut out: Vec<(f32, f32)> = poly.to_vec();
    for (inside, cross, v) in edges {
        let input = std::mem::take(&mut out);
        for (i, &cur) in input.iter().enumerate() {
            let prev = input[(i + input.len() - 1) % input.len()];
            match (inside(cur, v), inside(prev, v)) {
                (true, true) => out.push(cur),
                (true, false) => out.extend([cross(prev, cur, v), cur]),
                (false, true) => out.push(cross(prev, cur, v)),
                (false, false) => {}
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipping_keeps_what_lies_inside_the_rectangle() {
        let area = |p: &[(f32, f32)]| {
            (0..p.len())
                .map(|i| {
                    let (a, b) = (p[i], p[(i + 1) % p.len()]);
                    a.0 * b.1 - b.0 * a.1
                })
                .sum::<f32>()
                .abs()
                / 2.0
        };
        // A triangle over the corner of a 10×10 box: the part inside is half the box.
        let tri = [(-5.0, -5.0), (15.0, -5.0), (-5.0, 15.0)];
        let c = clip_polygon(&tri, (0.0, 0.0), (10.0, 10.0));
        assert!((area(&c) - 50.0).abs() < 1e-3, "{}", area(&c));
        assert!(c
            .iter()
            .all(|p| (0.0..=10.0).contains(&p.0) && (0.0..=10.0).contains(&p.1)));
        let inside = [(1.0, 1.0), (2.0, 1.0), (2.0, 2.0)];
        assert_eq!(
            clip_polygon(&inside, (0.0, 0.0), (10.0, 10.0)),
            inside.to_vec()
        );
        assert!(clip_polygon(
            &[(20.0, 20.0), (30.0, 20.0), (30.0, 30.0)],
            (0.0, 0.0),
            (10.0, 10.0)
        )
        .is_empty());
        assert!(clip_polygon(&[], (0.0, 0.0), (1.0, 1.0)).is_empty());
    }

    #[test]
    fn marker_shapes() {
        assert!(marker_points(Marker::None, 0.0, 0.0, 2.0).is_empty());
        assert_eq!(marker_points(Marker::Square, 0.0, 0.0, 2.0).len(), 4);
        assert_eq!(marker_points(Marker::Triangle, 0.0, 0.0, 2.0).len(), 3);
        assert_eq!(marker_points(Marker::Diamond, 0.0, 0.0, 2.0).len(), 4);
        assert!(marker_points(Marker::Dot, 0.0, 0.0, 2.0).len() >= 12);
    }

    #[test]
    fn step_points_stairs() {
        assert_eq!(
            step_points(&[(0.0, 1.0), (2.0, 3.0)]),
            vec![(0.0, 1.0), (2.0, 1.0), (2.0, 3.0)]
        );
    }

    #[test]
    fn ticks_cover_range_and_are_nice() {
        let t = nice_ticks(0.0, 87.0, 4);
        assert_eq!(t, vec![0.0, 50.0, 100.0]);
        let t = nice_ticks(0.0, 1.0, 4);
        assert_eq!(*t.last().unwrap(), 1.0);
        assert!(nice_ticks(5.0, 5.0, 4).len() >= 2);
    }

    #[test]
    fn ticks_are_never_empty_and_never_repeat() {
        let cases = [
            (f32::MIN, f32::MAX),
            (f32::NAN, 1.0),
            (5.0, f32::NAN),
            (f32::NAN, f32::NAN),
            (f32::NEG_INFINITY, f32::INFINITY),
            (1e9, 1e9 + 64.0),
            (1e10, 1e10 + 1024.0),
            (16_777_216.0, 16_777_216.0),
            (-3e38, 3e38),
            (0.0, 1e-30),
            (-5.0, -5.0),
        ];
        for (lo, hi) in cases {
            let t = nice_ticks(lo, hi, 4);
            assert!(t.len() >= 2, "{lo}..{hi}: {t:?}");
            assert!(t.len() <= 66, "{lo}..{hi}: {} ticks", t.len());
            assert!(t.windows(2).all(|w| w[1] > w[0]), "{lo}..{hi}: {t:?}");
            assert!(t.iter().all(|v| v.is_finite()), "{lo}..{hi}: {t:?}");
        }
    }

    #[test]
    fn a_narrow_range_at_a_large_magnitude_gives_a_few_ticks() {
        let t = nice_ticks(1e9, 1e9 + 60.0, 4);
        assert!(t.len() <= 6, "{t:?}");
        assert!(t[0] <= 1e9 && *t.last().unwrap() >= 1e9 + 60.0);
    }

    #[test]
    fn number_format() {
        assert_eq!(fmt_num(3.0), "3");
        assert_eq!(fmt_num(2.5), "2.5");
        assert_eq!(fmt_num(1200.0), "1.2k");
        assert_eq!(fmt_num(2_000_000.0), "2M");
    }

    #[test]
    fn smoothing_hits_endpoints() {
        let p = [(0.0, 0.0), (1.0, 2.0), (2.0, 0.0)];
        let s = smooth_points(&p, 4);
        assert_eq!(s[0], p[0]);
        assert_eq!(*s.last().unwrap(), p[2]);
        assert_eq!(s.len(), 2 * 4 + 1);
        assert_eq!(smooth_points(&p[..2], 4).len(), 2);
    }

    #[test]
    fn pie_fractions_sum_to_one() {
        let f = pie_fractions(&[1.0, 1.0, 2.0]);
        assert_eq!(f[0], (0.0, 0.25));
        assert_eq!(f[2], (0.5, 1.0));
        assert!(pie_fractions(&[0.0, 0.0]).iter().all(|(a, b)| a == b));
        assert_eq!(pie_fractions(&[-1.0, 1.0])[1].1, 1.0);
    }

    #[test]
    fn polar_starts_at_top_clockwise() {
        let (x, y) = polar(10.0, 10.0, 5.0, 0.0);
        assert!((x - 10.0).abs() < 1e-4 && (y - 5.0).abs() < 1e-4);
        let (x, _) = polar(10.0, 10.0, 5.0, 0.25);
        assert!((x - 15.0).abs() < 1e-4);
    }

    #[test]
    fn norm_clamps() {
        assert_eq!(norm(5.0, 0.0, 10.0), 0.5);
        assert_eq!(norm(20.0, 0.0, 10.0), 1.0);
        assert_eq!(norm(1.0, 3.0, 3.0), 0.0);
    }
}

#[cfg(test)]
mod nice_ceiling_tests {
    use super::{circle_points, nice_ceiling, CIRCLE_TOLERANCE};

    #[test]
    fn a_scale_rounds_up_to_a_clean_top() {
        for (v, top) in [
            (420.0, 500.0),
            (96.0, 100.0),
            (1.3, 1.5),
            (500.0, 500.0),
            (0.07, 0.08),
        ] {
            let got = nice_ceiling(v);
            assert!(got >= v * 0.999_999, "{v} → {got}");
            assert!((got - top).abs() < top * 1e-4, "{v} → {got}, want {top}");
        }
        assert_eq!(nice_ceiling(0.0), 0.0);
        assert!(nice_ceiling(f32::NAN).is_nan());
        assert_eq!(nice_ceiling(f32::INFINITY), f32::INFINITY);
    }

    #[test]
    fn circles_stay_round_at_any_size() {
        for r in [0.0, 2.0, 4.0, 30.0, 150.0, 600.0] {
            let p = circle_points(0.0, 0.0, r);
            assert!(p.len() >= 12);
            for (k, a) in p.iter().enumerate() {
                let b = p[(k + 1) % p.len()];
                let mid = (((a.0 + b.0) / 2.0).powi(2) + ((a.1 + b.1) / 2.0).powi(2)).sqrt();
                assert!(
                    r - mid <= CIRCLE_TOLERANCE + 1e-3,
                    "r {r}: side bows {}",
                    r - mid
                );
            }
        }
        assert!(
            circle_points(0.0, 0.0, 4.0).len() < 24,
            "small dots stay cheap"
        );
        assert!(circle_points(0.0, 0.0, f32::NAN).iter().all(|p| p.0 == 0.0));
    }

    #[test]
    fn extent_skips_non_finite_values() {
        assert_eq!(
            crate::math::extent([f32::NAN, 2.0, f32::INFINITY, -1.0]),
            Some((-1.0, 2.0))
        );
        assert_eq!(crate::math::extent([f32::NAN]), None);
        assert_eq!(crate::math::tick_span([], 4).0, 0.0);
        let (lo, hi, t) = crate::math::tick_span([3.0, 3.0], 4);
        assert!(lo <= 3.0 && hi > lo && t.len() >= 2);
    }

    #[test]
    fn peaks_and_zero_anchored_extents() {
        assert_eq!(crate::math::peak([1.0, f32::NAN, 4.0, f32::INFINITY]), 4.0);
        assert_eq!(crate::math::peak([-3.0]), 0.0);
        assert_eq!(crate::math::peak([]), 0.0);
        assert_eq!(crate::math::peak_from([0.0, -1.0], 1e-6), 1e-6);
        assert_eq!(crate::math::extent_with_zero([2.0, 5.0]), (0.0, 5.0));
        assert_eq!(crate::math::extent_with_zero([-2.0, f32::NAN]), (-2.0, 0.0));
        assert_eq!(crate::math::extent_with_zero([]), (0.0, 0.0));
    }

    #[test]
    fn padded_extent_always_has_width() {
        assert_eq!(crate::math::padded_extent([]), (0.0, 1.0));
        assert_eq!(crate::math::padded_extent([f32::NAN, 3.0]), (2.0, 4.0));
        assert_eq!(crate::math::padded_extent([1.0, 5.0]), (1.0, 5.0));
    }

    #[test]
    fn area_radius_grows_with_the_square_root() {
        assert_eq!(crate::math::area_radius(4.0, 4.0, 2.0, 10.0), 10.0);
        assert_eq!(crate::math::area_radius(1.0, 4.0, 2.0, 10.0), 6.0);
        assert_eq!(crate::math::area_radius(0.0, 4.0, 2.0, 10.0), 2.0);
        assert_eq!(crate::math::area_radius(5.0, 0.0, 2.0, 10.0), 2.0);
    }
}
