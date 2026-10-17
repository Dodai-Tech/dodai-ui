//! Order-book depth: cumulative bid/ask curves.

/// `(price, size)` level.
pub type Level = (f32, f32);
/// Cumulative curve as `(price, cumulative size)` points.
pub type Curve = Vec<(f32, f32)>;

/// Cumulative curves `(bids, asks)` as `(price, cumulative size)` points.
/// Bids run from the best (highest) price outward (descending); asks from the
/// best (lowest) outward (ascending). Pure.
pub fn depth_curves(bids: &[Level], asks: &[Level]) -> (Curve, Curve) {
    let mut b = bids.to_vec();
    b.sort_by(|x, y| y.0.total_cmp(&x.0));
    let mut a = asks.to_vec();
    a.sort_by(|x, y| x.0.total_cmp(&y.0));
    let acc = |v: Vec<Level>| {
        let mut sum = 0.0;
        v.into_iter()
            .map(|(p, s)| {
                sum += s.max(0.0);
                (p, sum)
            })
            .collect()
    };
    (acc(b), acc(a))
}

/// Mid price between best bid and best ask.
pub fn mid_price(bids: &[Level], asks: &[Level]) -> Option<f32> {
    let b = bids.iter().map(|l| l.0).fold(f32::NEG_INFINITY, f32::max);
    let a = asks.iter().map(|l| l.0).fold(f32::INFINITY, f32::min);
    (b.is_finite() && a.is_finite()).then(|| (a + b) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cumulates_away_from_spread() {
        let (b, a) = depth_curves(&[(99.0, 2.0), (100.0, 1.0)], &[(102.0, 3.0), (101.0, 1.0)]);
        assert_eq!(b, vec![(100.0, 1.0), (99.0, 3.0)]);
        assert_eq!(a, vec![(101.0, 1.0), (102.0, 4.0)]);
    }

    #[test]
    fn mid() {
        assert_eq!(mid_price(&[(99.0, 1.0)], &[(101.0, 1.0)]), Some(100.0));
        assert_eq!(mid_price(&[], &[(1.0, 1.0)]), None);
    }
}
