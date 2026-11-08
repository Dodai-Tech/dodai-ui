//! Histogram binning.

#[derive(Debug, Clone, PartialEq)]
pub struct Bin {
    pub lo: f32,
    pub hi: f32,
    pub count: usize,
}

/// Bin count via Freedman–Diaconis, falling back to Sturges. Pure.
pub fn auto_bin_count(values: &[f32]) -> usize {
    let mut v: Vec<f32> = values.iter().copied().filter(|x| x.is_finite()).collect();
    let n = v.len();
    if n < 2 {
        return 1;
    }
    v.sort_by(f32::total_cmp);
    let q = |p: f32| v[((v.len() - 1) as f32 * p).round() as usize];
    let range = v[v.len() - 1] - v[0];
    let iqr = q(0.75) - q(0.25);
    let sturges = ((n as f32).log2().ceil() as usize + 1).max(1);
    if iqr <= 0.0 || range <= 0.0 {
        return sturges;
    }
    let width = 2.0 * iqr / (n as f32).cbrt();
    ((range / width).ceil() as usize).clamp(1, 200)
}

/// Equal-width bins over the data range; `bins` of `None` picks automatically.
pub fn histogram(values: &[f32], bins: Option<usize>) -> Vec<Bin> {
    let v: Vec<f32> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return Vec::new();
    }
    let n = bins.unwrap_or_else(|| auto_bin_count(&v)).max(1);
    let lo = v.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = v.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let w = if hi > lo { (hi - lo) / n as f32 } else { 1.0 };
    let mut out: Vec<Bin> = (0..n)
        .map(|i| Bin {
            lo: lo + w * i as f32,
            hi: lo + w * (i + 1) as f32,
            count: 0,
        })
        .collect();
    for x in v {
        let i = (((x - lo) / w) as usize).min(n - 1);
        out[i].count += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_finite_values_are_ignored_not_fatal() {
        assert_eq!(auto_bin_count(&[f32::NAN, f32::INFINITY]), 1);
        assert_eq!(auto_bin_count(&[f32::NAN, 1.0, f32::NAN]), 1);
        assert!(histogram(&[f32::NAN, f32::NAN], None).is_empty());
    }

    #[test]
    fn counts_sum_and_edges() {
        let h = histogram(&[0.0, 1.0, 1.0, 2.0, 10.0], Some(5));
        assert_eq!(h.len(), 5);
        assert_eq!(h.iter().map(|b| b.count).sum::<usize>(), 5);
        assert_eq!(h[4].count, 1); // max lands in the last bin
        assert_eq!(h[0].lo, 0.0);
    }

    #[test]
    fn auto_scales_with_data() {
        let small: Vec<f32> = (0..20).map(|i| i as f32).collect();
        let big: Vec<f32> = (0..2000).map(|i| (i % 97) as f32).collect();
        assert!(auto_bin_count(&big) >= auto_bin_count(&small));
        assert!(histogram(&[], None).is_empty());
        assert_eq!(
            histogram(&[3.0, 3.0], None)
                .iter()
                .map(|b| b.count)
                .sum::<usize>(),
            2
        );
    }
}
