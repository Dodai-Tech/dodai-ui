//! Kernel density estimate for violin plots and histogram overlays.

/// A density curve sampled over `lo..=hi`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Density {
    pub lo: f32,
    pub hi: f32,
    /// `points` samples, normalised so the peak is `1.0`.
    pub y: Vec<f32>,
}

/// Gaussian KDE with Silverman's bandwidth, sampled at `points` positions
/// across the data range extended by three bandwidths. Non-finite values are
/// ignored; fewer than two values yields an empty curve. Pure.
pub fn kde(values: &[f32], points: usize) -> Density {
    let v: Vec<f32> = values.iter().copied().filter(|x| x.is_finite()).collect();
    let n = v.len();
    if n < 2 || points < 2 {
        return Density::default();
    }
    let mean = v.iter().sum::<f32>() / n as f32;
    let sd = (v.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / (n - 1) as f32).sqrt();
    let mut s = v.clone();
    s.sort_by(f32::total_cmp);
    let q = |p: f32| s[((n - 1) as f32 * p).round() as usize];
    let iqr = q(0.75) - q(0.25);
    let spread = if iqr > 0.0 { sd.min(iqr / 1.34) } else { sd };
    let h = if spread > 0.0 {
        0.9 * spread * (n as f32).powf(-0.2)
    } else {
        1.0
    };
    let (lo, hi) = (s[0] - 3.0 * h, s[n - 1] + 3.0 * h);
    let mut y: Vec<f32> = (0..points)
        .map(|i| {
            let x = lo + (hi - lo) * i as f32 / (points - 1) as f32;
            v.iter().map(|d| (-0.5 * ((x - d) / h).powi(2)).exp()).sum()
        })
        .collect();
    let peak = y.iter().copied().fold(0.0f32, f32::max);
    if peak > 0.0 {
        y.iter_mut().for_each(|d| *d /= peak);
    }
    Density { lo, hi, y }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peak_is_one_and_range_covers_data() {
        let d = kde(&[1.0, 2.0, 2.0, 3.0, 9.0], 64);
        assert_eq!(d.y.len(), 64);
        assert!((d.y.iter().copied().fold(0.0, f32::max) - 1.0).abs() < 1e-6);
        assert!(d.lo < 1.0 && d.hi > 9.0);
    }

    #[test]
    fn mode_sits_near_the_cluster() {
        let d = kde(&[5.0, 5.1, 4.9, 5.0, 5.2, 0.0], 101);
        let i = d.y.iter().position(|v| *v == 1.0).unwrap();
        let x = d.lo + (d.hi - d.lo) * i as f32 / 100.0;
        assert!((x - 5.0).abs() < 1.0);
    }

    #[test]
    fn degenerate_inputs() {
        assert_eq!(kde(&[], 10), Density::default());
        assert_eq!(kde(&[1.0], 10), Density::default());
        assert_eq!(kde(&[f32::NAN, 1.0], 10), Density::default());
        let flat = kde(&[2.0, 2.0, 2.0], 10);
        assert!(flat.y.iter().all(|v| v.is_finite()));
    }
}
