//! Smooth, endless, deterministic signals for live demos and idle motion. Pure functions of
//! time, so any host clock drives them.

/// A repeatable pseudo-random value in `-1..=1` for lattice point `n` of stream `key`.
fn lattice(n: i64, key: f32) -> f32 {
    let mut x = (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ u64::from(key.to_bits()).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 31;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 29;
    (x >> 40) as f32 / (1u64 << 23) as f32 - 1.0
}

/// Smooth value noise in `-1..=1`: at each whole `x` a fresh random value, eased in between
/// (it settles on each value, then heads for the next). Never repeats.
fn noise(x: f32, key: f32) -> f32 {
    let (n, f) = (x.floor(), x - x.floor());
    let s = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let (a, b) = (lattice(n as i64, key), lattice(n as i64 + 1, key));
    a + (b - a) * s
}

/// A smooth wander in `-1..=1` at time `t` (seconds); each `key` takes its own path. It glides
/// to a new random value about every two seconds and settles there briefly: values change,
/// they do not swing back and forth.
pub fn wander(t: f32, key: f32) -> f32 {
    let rate = 0.4 + 0.2 * lattice(-1, key).abs();
    noise(t * rate + key * 7.31, key)
}

/// [`wander`] mapped onto `lo..=hi`.
pub fn wander_between(t: f32, key: f32, lo: f32, hi: f32) -> f32 {
    lo + (hi - lo) * (0.5 + 0.5 * wander(t, key))
}

/// A live series shaped like `base`: every value glides between nearby values (within about a
/// fifth of the series' range) on its own schedule, so the shape stays recognisable while the
/// numbers change. Never goes below zero when `base` does not.
pub fn drift(base: &[f32], t: f32, key: f32) -> Vec<f32> {
    let lo = base.iter().copied().fold(f32::MAX, f32::min);
    let hi = base.iter().copied().fold(f32::MIN, f32::max);
    let span = if hi - lo > 1e-6 {
        hi - lo
    } else {
        (hi.abs() * 0.5).max(1.0)
    };
    let amp = span * 0.2;
    let floor = if lo >= 0.0 {
        (lo - amp).max(0.0)
    } else {
        lo - amp
    };
    base.iter()
        .enumerate()
        .map(|(i, &b)| {
            let k = key * 31.0 + i as f32 * 1.618;
            (b + amp * wander(t, k)).clamp(floor, hi + amp)
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
    fn a_wander_is_smooth_and_does_not_oscillate_between_two_values() {
        let vs: Vec<f32> = (0..2000).map(|i| wander(i as f32 * 0.01, 5.0)).collect();
        assert!(vs.windows(2).all(|w| (w[1] - w[0]).abs() < 0.05), "smooth");
        // Where it settles (slope turns) it lands on many different values, not two extremes.
        let mut stops: Vec<f32> = vs
            .windows(3)
            .filter(|w| {
                (w[1] - w[0]) * (w[2] - w[1]) <= 0.0
                    && (w[1] - w[0]).abs() + (w[2] - w[1]).abs() > 0.0
            })
            .map(|w| w[1])
            .collect();
        stops.dedup_by(|a, b| (*a - *b).abs() < 0.05);
        assert!(stops.len() >= 6, "{stops:?}");
        let spread = stops.iter().fold(f32::MIN, |m, v| m.max(*v))
            - stops.iter().fold(f32::MAX, |m, v| m.min(*v));
        assert!(spread > 0.5);
    }

    #[test]
    fn a_drift_keeps_its_shape_and_its_values_move_on_their_own() {
        let base = [2.0, 8.0, 5.0, 3.0];
        let a = drift(&base, 0.0, 1.0);
        assert_eq!(a.len(), 4);
        for (v, b) in a.iter().zip(base) {
            assert!((v - b).abs() <= 6.0 * 0.2 + 1e-4 && *v >= 0.0);
        }
        let b = drift(&base, 3.0, 1.0);
        assert_ne!(a, b);
        // Not one wave sliding through: the points move by different amounts.
        let d: Vec<f32> = a.iter().zip(&b).map(|(x, y)| y - x).collect();
        assert!(d.iter().any(|v| *v > 0.0) || d.iter().any(|v| *v < 0.0));
        assert!(d.windows(2).any(|w| (w[0] - w[1]).abs() > 0.05));
        assert!(drift(&[100.0], 1.0, 2.0)[0] > 0.0);
        assert!(drift(&[], 1.0, 2.0).is_empty());
    }
}
