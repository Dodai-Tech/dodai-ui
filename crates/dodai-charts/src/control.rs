//! Statistical process control (individuals chart): centre line, ±3σ limits from the moving
//! range, and the points that signal a process out of control. Pure.

/// Control limits of a series.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limits {
    /// Centre line (mean of the finite values).
    pub mean: f32,
    /// Estimated σ: mean moving range / 1.128.
    pub sigma: f32,
    /// Lower control limit, `mean - k·σ`.
    pub lower: f32,
    /// Upper control limit, `mean + k·σ`.
    pub upper: f32,
}

/// Limits `k` σ around the mean, σ estimated from the average moving range (robust to drift,
/// the usual choice for one value per sample). Non-finite values are skipped. Pure.
pub fn limits(values: &[f32], k: f32) -> Limits {
    let v: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
    let mean = if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f32>() / v.len() as f32
    };
    let mr: Vec<f32> = v.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
    let sigma = if mr.is_empty() {
        0.0
    } else {
        mr.iter().sum::<f32>() / mr.len() as f32 / 1.128
    };
    Limits {
        mean,
        sigma,
        lower: mean - k * sigma,
        upper: mean + k * sigma,
    }
}

/// How long a run on one side of the centre line has to be to signal a shift.
pub const RUN: usize = 9;

/// Which points signal: outside the limits, or part of a run of [`RUN`] or more in a row on
/// the same side of the centre. Non-finite points never signal and break a run. Pure.
pub fn signals(values: &[f32], l: &Limits) -> Vec<bool> {
    let mut out: Vec<bool> = values
        .iter()
        .map(|v| v.is_finite() && (*v > l.upper || *v < l.lower))
        .collect();
    let side = |v: f32| {
        if !v.is_finite() || v == l.mean {
            0
        } else if v > l.mean {
            1
        } else {
            -1
        }
    };
    let mut start = 0;
    for i in 0..=values.len() {
        let same =
            i < values.len() && side(values[i]) != 0 && side(values[i]) == side(values[start]);
        if !same {
            if i - start >= RUN && side(values[start]) != 0 {
                out[start..i].iter_mut().for_each(|o| *o = true);
            }
            start = i;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_come_from_the_moving_range() {
        let l = limits(&[10.0, 12.0, 10.0, 12.0], 3.0);
        assert_eq!(l.mean, 11.0);
        assert!((l.sigma - 2.0 / 1.128).abs() < 1e-5);
        assert!((l.upper - (11.0 + 3.0 * 2.0 / 1.128)).abs() < 1e-4);
        assert_eq!(limits(&[], 3.0).sigma, 0.0);
    }

    #[test]
    fn outliers_and_long_runs_signal() {
        let mut v = vec![10.0, 12.0, 10.0, 12.0, 30.0];
        let l = limits(&[10.0, 12.0, 10.0, 12.0], 3.0);
        assert_eq!(signals(&v, &l), vec![false, false, false, false, true]);
        v = vec![11.5; RUN];
        v.insert(0, 10.0);
        let s = signals(&v, &l);
        assert!(!s[0] && s[1..].iter().all(|x| *x));
        assert_eq!(signals(&[f32::NAN], &l), vec![false]);
    }
}
