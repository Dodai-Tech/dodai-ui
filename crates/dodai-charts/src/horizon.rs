//! Horizon chart folding: a value split into stacked bands of equal size, so a tall range reads
//! in a short strip (darker = further from zero). Pure.

/// How full each of `bands` bands is for `v` with bands `band` units tall: band `k` covers
/// `k * band ..= (k + 1) * band` of `|v|`, the last band soaks up anything beyond. Each entry is
/// a fill fraction in `0..=1`. Non-finite or non-positive `band`: all empty. Pure.
pub fn fold(v: f32, band: f32, bands: usize) -> Vec<f32> {
    if !(band > 0.0 && band.is_finite() && v.is_finite()) {
        return vec![0.0; bands];
    }
    let a = v.abs();
    (0..bands)
        .map(|k| ((a - k as f32 * band) / band).clamp(0.0, 1.0))
        .collect()
}

/// The band size that fits the largest `|value|` into `bands` bands (at least a tiny positive
/// number, so an all-zero series still folds). Pure.
pub fn band_for(values: &[f32], bands: usize) -> f32 {
    let peak = values
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold(0.0f32, |m, v| m.max(v.abs()));
    (peak / bands.max(1) as f32).max(f32::MIN_POSITIVE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_fills_the_bands_from_the_bottom() {
        assert_eq!(fold(2.5, 1.0, 3), vec![1.0, 1.0, 0.5]);
        assert_eq!(fold(-0.5, 1.0, 3), vec![0.5, 0.0, 0.0]);
        assert_eq!(fold(9.0, 1.0, 2), vec![1.0, 1.0]);
    }

    #[test]
    fn the_band_fits_the_peak_and_hostile_input_stays_empty() {
        assert_eq!(band_for(&[1.0, -6.0, f32::NAN], 3), 2.0);
        assert!(band_for(&[0.0], 3) > 0.0);
        assert_eq!(fold(f32::NAN, 1.0, 2), vec![0.0, 0.0]);
        assert_eq!(fold(1.0, 0.0, 2), vec![0.0, 0.0]);
    }
}
