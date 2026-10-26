//! Clip geometry and noise shared by the effects.

use crate::{Clip, ShapeKind};

/// Upper bound on quads one surface may paint per frame.
pub(crate) const MAX_QUADS: f32 = 1800.0;

/// Horizontal extent `(x0, x1)` of `shape` at row `y` in a `w`×`h` box, or
/// `None` when the row is outside. Follows chamfers and rounded corners.
pub fn row_span(shape: &Clip, w: f32, h: f32, y: f32) -> Option<(f32, f32)> {
    if y < 0.0 || y > h || w <= 0.0 {
        return None;
    }
    match shape.kind() {
        ShapeKind::Chamfered => poly_span(&shape.outline(w, h), y),
        ShapeKind::Rounded => {
            let r = shape.radius.min(w / 2.0).min(h / 2.0);
            let dy = if y < r {
                r - y
            } else if y > h - r {
                y - (h - r)
            } else {
                0.0
            };
            let inset = if dy > 0.0 && r > 0.0 {
                r - (r * r - dy * dy).max(0.0).sqrt()
            } else {
                0.0
            };
            Some((inset, w - inset))
        }
        ShapeKind::Square => Some((0.0, w)),
    }
}

/// Horizontal extent of a polygon at row `y` (min and max x over the edges it crosses).
pub fn poly_span(pts: &[(f32, f32)], y: f32) -> Option<(f32, f32)> {
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        let (y0, y1) = (a.1.min(b.1), a.1.max(b.1));
        if y < y0 || y > y1 {
            continue;
        }
        if (b.1 - a.1).abs() < 1e-6 {
            // horizontal edge: both ends count
            lo = lo.min(a.0.min(b.0));
            hi = hi.max(a.0.max(b.0));
            continue;
        }
        let x = a.0 + (b.0 - a.0) * (y - a.1) / (b.1 - a.1);
        lo = lo.min(x);
        hi = hi.max(x);
    }
    (lo <= hi).then_some((lo, hi))
}

/// Deterministic noise in `0..1` from two integers.
pub(crate) fn noise(a: i32, b: i32) -> f32 {
    let mut x = (a as u32).wrapping_mul(0x9E37_79B1) ^ (b as u32).wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    (x & 0xFFFF) as f32 / 65535.0
}

/// Pitch grown so that `count(pitch)` stays under [`MAX_QUADS`].
pub(crate) fn fit(pitch: f32, count: impl Fn(f32) -> f32) -> f32 {
    let n = count(pitch);
    if n <= MAX_QUADS {
        pitch
    } else {
        pitch * (n / MAX_QUADS).sqrt()
    }
}

/// The shape's outline polygon; rounded shapes get their corner arcs flattened too.
pub(crate) fn outline_of(shape: &Clip, w: f32, h: f32) -> Vec<(f32, f32)> {
    if shape.kind() != ShapeKind::Rounded {
        return shape.outline(w, h);
    }
    let r = shape.radius.min(w / 2.0).min(h / 2.0);
    let n = ((r / 2.0) as usize).clamp(3, 12);
    let mut p = Vec::with_capacity(4 * (n + 1));
    // Corner centres, clockwise from top-left, with the arc's start angle.
    for (cx, cy, a0) in [
        (r, r, 180.0_f32),
        (w - r, r, 270.0),
        (w - r, h - r, 0.0),
        (r, h - r, 90.0),
    ] {
        for i in 0..=n {
            let a = (a0 + 90.0 * i as f32 / n as f32).to_radians();
            p.push((cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_stays_in_unit_range_and_varies() {
        let v: Vec<f32> = (0..200).map(|i| noise(i, 7)).collect();
        assert!(v.iter().all(|x| (0.0..=1.0).contains(x)));
        assert!(v.iter().any(|x| *x < 0.3) && v.iter().any(|x| *x > 0.7));
        assert_eq!(noise(3, 4), noise(3, 4));
    }

    #[test]
    fn fit_keeps_the_quad_count_bounded() {
        let count = |p: f32| 400.0 * 300.0 / (p * p);
        let p = fit(2.0, count);
        assert!(count(p) <= MAX_QUADS * 1.01, "{}", count(p));
        assert_eq!(fit(50.0, count), 50.0);
    }
}
