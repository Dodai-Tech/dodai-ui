//! Secondary-axis tick alignment.

use crate::math::nice_ticks;

/// Ticks for a secondary axis that line up with the primary's gridlines:
/// both get the same number of intervals. Returns `(primary, secondary)`. Pure.
pub fn aligned_ticks(p: (f32, f32), s: (f32, f32), count: usize) -> (Vec<f32>, Vec<f32>) {
    let primary = nice_ticks(p.0, p.1, count);
    let n = primary.len().saturating_sub(1).max(1);
    let span = (s.1 - s.0).max(f32::EPSILON);
    let raw = span / n as f32;
    let mag = 10f32.powf(raw.log10().floor());
    let r = raw / mag;
    let step = mag
        * if r <= 1.0 {
            1.0
        } else if r <= 2.0 {
            2.0
        } else if r <= 5.0 {
            5.0
        } else {
            10.0
        };
    let start = (s.0 / step).floor() * step;
    let secondary = (0..=n).map(|i| start + step * i as f32).collect();
    (primary, secondary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_interval_count() {
        let (a, b) = aligned_ticks((0.0, 100.0), (0.0, 7.3), 4);
        assert_eq!(a.len(), b.len());
        assert!(*b.last().unwrap() >= 7.3);
        assert_eq!(b[0], 0.0);
    }
}
