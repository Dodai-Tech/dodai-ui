//! Pure numeric and geometry helpers shared by every chart renderer.

/// "Nice" axis ticks covering `min..=max` with about `count` steps. Pure.
pub fn nice_ticks(min: f32, max: f32, count: usize) -> Vec<f32> {
    let count = count.max(1) as f32;
    let (lo, hi) = if max > min {
        (min, max)
    } else {
        (min, min + 1.0)
    };
    let raw = (hi - lo) / count;
    let mag = 10f32.powf(raw.log10().floor());
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
    let start = (lo / step).floor() * step;
    let mut out = Vec::new();
    let mut v = start;
    while v < hi + step * 0.999 {
        out.push((v / step).round() * step);
        v += step;
        if out.len() > 64 {
            break;
        }
    }
    out
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

/// Outline of a marker centred at `(x, y)` with radius `r`. Pure.
pub fn marker_points(m: Marker, x: f32, y: f32, r: f32) -> Vec<(f32, f32)> {
    match m {
        Marker::None => vec![],
        Marker::Dot => (0..16)
            .map(|i| {
                let a = i as f32 / 16.0 * std::f32::consts::TAU;
                (x + r * a.cos(), y + r * a.sin())
            })
            .collect(),
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

#[cfg(test)]
mod tests {
    use super::*;

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
