//! Smooth, endless, deterministic signals for live demos and idle motion. Pure functions of
//! time, so any host clock drives them.

/// A smooth wander in `-1..=1` at time `t` (seconds); each `key` takes its own path. Two slow
/// waves of unrelated speed, so it never visibly repeats.
pub fn wander(t: f32, key: f32) -> f32 {
    0.6 * (t * (0.35 + 0.05 * (key % 5.0)) + key * 1.7).sin()
        + 0.4 * (t * (0.83 + 0.07 * (key % 3.0)) + key * 2.9).sin()
}

/// [`wander`] mapped onto `lo..=hi`.
pub fn wander_between(t: f32, key: f32, lo: f32, hi: f32) -> f32 {
    lo + (hi - lo) * (0.5 + 0.5 * wander(t, key))
}

/// A live stream shaped like `base`: the window slides left continuously (1.4 samples a second)
/// over a smooth signal within `base`'s range, so it never restarts.
pub fn drift(base: &[f32], t: f32, key: f32) -> Vec<f32> {
    let lo = base.iter().copied().fold(f32::MAX, f32::min);
    let hi = base.iter().copied().fold(f32::MIN, f32::max);
    let (mid, amp) = ((lo + hi) / 2.0, (hi - lo) / 2.0);
    (0..base.len())
        .map(|i| {
            let s = i as f32 + t * 1.4 + key * 11.0;
            let n = 0.6 * (s * 0.62).sin() + 0.3 * (s * 1.37 + 1.1).sin() + 0.1 * (s * 2.9).sin();
            mid + amp * n
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wanders_stay_in_range_and_differ_per_key() {
        for i in 0..400 {
            let t = i as f32 * 0.37;
            assert!(wander(t, 3.0).abs() <= 1.0);
            let v = wander_between(t, 1.0, 10.0, 20.0);
            assert!((10.0..=20.0).contains(&v));
        }
        assert_ne!(wander(1.0, 1.0), wander(1.0, 2.0));
    }

    #[test]
    fn a_drift_stays_within_its_base_and_moves() {
        let base = [2.0, 8.0, 5.0, 3.0];
        let a = drift(&base, 0.0, 1.0);
        assert_eq!(a.len(), 4);
        assert!(a.iter().all(|v| (2.0 - 1e-3..=8.0 + 1e-3).contains(v)));
        assert_ne!(a, drift(&base, 0.5, 1.0));
    }
}
