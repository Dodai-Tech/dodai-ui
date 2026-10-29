//! Standings for a bump chart: each series' rank at every x. Pure.

/// The rank (0 = highest value) of every series at every x, one row per series. Ties keep the
/// series order; a missing or non-finite value ranks last. Pure.
pub fn ranks(rows: &[Vec<f32>]) -> Vec<Vec<usize>> {
    let n = rows.iter().map(Vec::len).max().unwrap_or(0);
    // One standing per step: the rank of each series at that step.
    let standings: Vec<Vec<usize>> = (0..n)
        .map(|i| {
            let value = |k: usize| rows[k].get(i).copied().filter(|v| v.is_finite());
            let mut order: Vec<usize> = (0..rows.len()).collect();
            // Stable: equal values keep the series order.
            order.sort_by(|a, b| match (value(*a), value(*b)) {
                (Some(x), Some(y)) => y.total_cmp(&x),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            });
            let mut rank = vec![0; rows.len()];
            for (r, k) in order.into_iter().enumerate() {
                rank[k] = r;
            }
            rank
        })
        .collect();
    (0..rows.len())
        .map(|k| standings.iter().map(|s| s[k]).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_highest_value_ranks_first_at_every_x() {
        let r = ranks(&[vec![1.0, 9.0], vec![5.0, 2.0], vec![3.0, 4.0]]);
        assert_eq!(r, vec![vec![2, 0], vec![0, 2], vec![1, 1]]);
    }

    #[test]
    fn ties_keep_the_series_order_and_gaps_rank_last() {
        let r = ranks(&[vec![2.0, f32::NAN], vec![2.0], vec![1.0, 0.0]]);
        assert_eq!(r, vec![vec![0, 1], vec![1, 2], vec![2, 0]]);
        assert!(ranks(&[]).is_empty());
    }
}
