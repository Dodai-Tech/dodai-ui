//! Technical indicators. Outputs are `None` until enough data exists.

/// Simple moving average.
pub fn sma(v: &[f32], period: usize) -> Vec<Option<f32>> {
    let p = period.max(1);
    (0..v.len())
        .map(|i| (i + 1 >= p).then(|| v[i + 1 - p..=i].iter().sum::<f32>() / p as f32))
        .collect()
}

/// Exponential moving average (seeded with the first `period` SMA).
pub fn ema(v: &[f32], period: usize) -> Vec<Option<f32>> {
    let p = period.max(1);
    let k = 2.0 / (p as f32 + 1.0);
    let mut out = vec![None; v.len()];
    if v.len() < p {
        return out;
    }
    let mut prev = v[..p].iter().sum::<f32>() / p as f32;
    out[p - 1] = Some(prev);
    for i in p..v.len() {
        prev = v[i] * k + prev * (1.0 - k);
        out[i] = Some(prev);
    }
    out
}

/// Bollinger bands `(middle, upper, lower)` using population deviation.
#[allow(clippy::type_complexity)]
pub fn bollinger(
    v: &[f32],
    period: usize,
    k: f32,
) -> (Vec<Option<f32>>, Vec<Option<f32>>, Vec<Option<f32>>) {
    let p = period.max(1);
    let mid = sma(v, p);
    let mut up = vec![None; v.len()];
    let mut lo = vec![None; v.len()];
    for i in 0..v.len() {
        if let Some(m) = mid[i] {
            let var = v[i + 1 - p..=i]
                .iter()
                .map(|x| (x - m).powi(2))
                .sum::<f32>()
                / p as f32;
            let sd = var.sqrt();
            up[i] = Some(m + k * sd);
            lo[i] = Some(m - k * sd);
        }
    }
    (mid, up, lo)
}

/// Wilder RSI in `0..=100`.
pub fn rsi(v: &[f32], period: usize) -> Vec<Option<f32>> {
    let p = period.max(1);
    let mut out = vec![None; v.len()];
    if v.len() <= p {
        return out;
    }
    let (mut gain, mut loss) = (0.0, 0.0);
    for i in 1..=p {
        let d = v[i] - v[i - 1];
        if d >= 0.0 {
            gain += d
        } else {
            loss -= d
        }
    }
    let (mut ag, mut al) = (gain / p as f32, loss / p as f32);
    let val = |ag: f32, al: f32| {
        if al == 0.0 {
            100.0
        } else {
            100.0 - 100.0 / (1.0 + ag / al)
        }
    };
    out[p] = Some(val(ag, al));
    for i in p + 1..v.len() {
        let d = v[i] - v[i - 1];
        let (g, l) = if d >= 0.0 { (d, 0.0) } else { (0.0, -d) };
        ag = (ag * (p as f32 - 1.0) + g) / p as f32;
        al = (al * (p as f32 - 1.0) + l) / p as f32;
        out[i] = Some(val(ag, al));
    }
    out
}

/// MACD `(macd line, signal, histogram)`.
#[allow(clippy::type_complexity)]
pub fn macd(
    v: &[f32],
    fast: usize,
    slow: usize,
    signal: usize,
) -> (Vec<Option<f32>>, Vec<Option<f32>>, Vec<Option<f32>>) {
    let (f, s) = (ema(v, fast), ema(v, slow));
    let line: Vec<Option<f32>> = f
        .iter()
        .zip(&s)
        .map(|(a, b)| a.zip(*b).map(|(a, b)| a - b))
        .collect();
    let first = line.iter().position(Option::is_some).unwrap_or(line.len());
    let dense: Vec<f32> = line[first..].iter().map(|x| x.unwrap_or(0.0)).collect();
    let mut sig = vec![None; line.len()];
    for (i, x) in ema(&dense, signal).into_iter().enumerate() {
        sig[first + i] = x;
    }
    let hist = line
        .iter()
        .zip(&sig)
        .map(|(a, b)| a.zip(*b).map(|(a, b)| a - b))
        .collect();
    (line, sig, hist)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sma_warmup_and_value() {
        let s = sma(&[1.0, 2.0, 3.0, 4.0], 2);
        assert_eq!(s, vec![None, Some(1.5), Some(2.5), Some(3.5)]);
    }

    #[test]
    fn ema_seeds_with_sma() {
        let e = ema(&[1.0, 2.0, 3.0, 4.0], 2);
        assert_eq!(e[0], None);
        assert_eq!(e[1], Some(1.5));
        let k = 2.0 / 3.0;
        assert!((e[2].unwrap() - (3.0 * k + 1.5 * (1.0 - k))).abs() < 1e-5);
    }

    #[test]
    fn bollinger_flat_series_has_zero_width() {
        let (m, u, l) = bollinger(&[5.0; 6], 3, 2.0);
        assert_eq!(m[5], Some(5.0));
        assert_eq!(u[5], l[5]);
        assert_eq!(u[0], None);
    }

    #[test]
    fn rsi_extremes() {
        let up: Vec<f32> = (0..20).map(|i| i as f32).collect();
        assert_eq!(rsi(&up, 14)[19], Some(100.0));
        let down: Vec<f32> = (0..20).map(|i| 20.0 - i as f32).collect();
        assert!(rsi(&down, 14)[19].unwrap() < 1.0);
        assert_eq!(rsi(&up, 14)[3], None);
    }

    #[test]
    fn macd_histogram_is_line_minus_signal() {
        let v: Vec<f32> = (0..60)
            .map(|i| (i as f32 * 0.3).sin() * 5.0 + i as f32 * 0.1)
            .collect();
        let (m, s, h) = macd(&v, 12, 26, 9);
        let i = 50;
        assert!((h[i].unwrap() - (m[i].unwrap() - s[i].unwrap())).abs() < 1e-5);
        assert_eq!(m[10], None);
    }
}
