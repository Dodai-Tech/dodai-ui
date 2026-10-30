//! Peak range helpers: overlapping series drawn as mountains on one horizon. The tallest
//! peaks stand at the back so smaller ones stay visible in front; each summit carries its
//! label.

/// The highest finite sample of `values`, as `(index, value)`.
pub fn summit(values: &[f32]) -> Option<(usize, f32)> {
    values
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, v)| v.is_finite())
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// Painting order: tallest summit first (at the back); series without one go last.
pub fn back_to_front(series: &[Vec<f32>]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..series.len()).collect();
    let top = |i: usize| summit(&series[i]).map_or(f32::NEG_INFINITY, |s| s.1);
    order.sort_by(|a, b| top(*b).total_cmp(&top(*a)));
    order
}

/// `values` read at `f` (`0..=1` across the samples), linearly interpolated; non-finite
/// samples count as 0.
pub fn at(values: &[f32], f: f32) -> f32 {
    let clean = |v: f32| if v.is_finite() { v } else { 0.0 };
    match values.len() {
        0 => 0.0,
        1 => clean(values[0]),
        n => {
            let x = (if f.is_finite() { f } else { 0.0 }).clamp(0.0, 1.0) * (n - 1) as f32;
            let i = (x.floor() as usize).min(n - 2);
            let t = x - i as f32;
            clean(values[i]) * (1.0 - t) + clean(values[i + 1]) * t
        }
    }
}

/// The front-most series (last in `order`) whose mountain covers height `v` at `f`. Series
/// `i` rises from `base[i]` (0 when missing), so lanes of mountains can stand one above another.
pub fn hit(series: &[Vec<f32>], base: &[f32], order: &[usize], f: f32, v: f32) -> Option<usize> {
    if !(f.is_finite() && v.is_finite()) {
        return None;
    }
    order.iter().rev().copied().find(|&i| {
        let b = lane(base, i);
        series
            .get(i)
            .is_some_and(|s| v >= b && at(s, f) >= v - b && at(s, f) > 0.0)
    })
}

/// The baseline of series `i` (0 when missing or not finite).
fn lane(base: &[f32], i: usize) -> f32 {
    base.get(i)
        .copied()
        .filter(|b| b.is_finite())
        .unwrap_or(0.0)
}

/// Painting order for lanes: the highest baseline first (at the back), then
/// [`back_to_front`] within a lane.
pub fn lane_order(series: &[Vec<f32>], base: &[f32]) -> Vec<usize> {
    let mut order = back_to_front(series);
    order.sort_by(|x, y| lane(base, *y).total_cmp(&lane(base, *x)));
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summits_and_order() {
        let s = vec![
            vec![1.0, 3.0, 2.0],
            vec![0.0, 9.0, 0.0],
            vec![f32::NAN],
            vec![],
        ];
        assert_eq!(summit(&s[0]), Some((1, 3.0)));
        assert_eq!(summit(&s[2]), None);
        assert_eq!(back_to_front(&s)[..2], [1, 0]);
    }

    #[test]
    fn reading_and_hitting_prefer_the_front() {
        let s = vec![vec![0.0, 10.0, 0.0], vec![0.0, 4.0, 0.0]];
        let order = back_to_front(&s);
        assert_eq!(at(&s[0], 0.25), 5.0);
        assert_eq!(
            hit(&s, &[], &order, 0.5, 3.0),
            Some(1),
            "the small one is in front"
        );
        assert_eq!(hit(&s, &[], &order, 0.5, 6.0), Some(0));
        assert_eq!(hit(&s, &[], &order, 0.5, 11.0), None);
        assert_eq!(hit(&s, &[], &order, f32::NAN, 1.0), None);
        // Lifted on a lane, the small one covers 2..6 and stands behind.
        let base = [0.0, 2.0];
        let order = lane_order(&s, &base);
        assert_eq!(order, [1, 0]);
        assert_eq!(hit(&s, &base, &order, 0.5, 1.0), Some(0));
        assert_eq!(hit(&s, &base, &order, 0.5, 9.0), Some(0));
        assert_eq!(at(&[], 0.5), 0.0);
        assert_eq!(at(&[f32::NAN, 2.0], f32::INFINITY), 0.0);
    }
}
