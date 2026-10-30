//! Milestone helpers for time scatters: the record frontier (the best value so far, as a
//! staircase), per-group trails in time order, and the nearest point to the pointer.

/// The record staircase of `points` (`(x, y)`): sorted by `x`, a step up at each point that
/// beats every earlier one, held flat to the last `x`. Non-finite points are skipped.
pub fn frontier(points: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut p: Vec<(f32, f32)> = points
        .iter()
        .copied()
        .filter(|(x, y)| x.is_finite() && y.is_finite())
        .collect();
    p.sort_by(|a, b| a.0.total_cmp(&b.0));
    let Some(&last) = p.last() else {
        return Vec::new();
    };
    let mut out: Vec<(f32, f32)> = Vec::new();
    for (x, y) in p {
        match out.last() {
            None => out.push((x, y)),
            Some(&(_, best)) if y > best => {
                out.push((x, best));
                out.push((x, y));
            }
            _ => {}
        }
    }
    if let Some(&(x, y)) = out.last() {
        if last.0 > x {
            out.push((last.0, y));
        }
    }
    out
}

/// The indices of each group's points in `x` order (`groups[i]` is point `i`'s group),
/// skipping points with a non-finite coordinate; groups with fewer than two are dropped.
pub fn trails(points: &[(f32, f32)], groups: &[usize]) -> Vec<Vec<usize>> {
    let n = groups.iter().copied().max().map_or(0, |m| m + 1);
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, g) in groups.iter().enumerate() {
        if points
            .get(i)
            .is_some_and(|(x, y)| x.is_finite() && y.is_finite())
        {
            out[*g].push(i);
        }
    }
    for t in &mut out {
        t.sort_by(|a, b| points[*a].0.total_cmp(&points[*b].0));
    }
    out.retain(|t| t.len() > 1);
    out
}

/// The least-squares line through the finite `points`, as `(slope, intercept)`; `None` with
/// fewer than two distinct `x`.
pub fn trend(points: &[(f32, f32)]) -> Option<(f32, f32)> {
    let p: Vec<(f64, f64)> = points
        .iter()
        .filter(|(x, y)| x.is_finite() && y.is_finite())
        .map(|(x, y)| (f64::from(*x), f64::from(*y)))
        .collect();
    let n = p.len() as f64;
    let (mx, my) = p
        .iter()
        .fold((0.0, 0.0), |a, q| (a.0 + q.0 / n, a.1 + q.1 / n));
    let sxx: f64 = p.iter().map(|q| (q.0 - mx).powi(2)).sum();
    let sxy: f64 = p.iter().map(|q| (q.0 - mx) * (q.1 - my)).sum();
    (p.len() > 1 && sxx > 0.0).then(|| {
        let slope = sxy / sxx;
        (slope as f32, (my - slope * mx) as f32)
    })
}

/// The point nearest `q` within `radius`, if any.
pub fn nearest(points: &[(f32, f32)], q: (f32, f32), radius: f32) -> Option<usize> {
    points
        .iter()
        .enumerate()
        .map(|(i, p)| (i, (p.0 - q.0).hypot(p.1 - q.1)))
        .filter(|(_, d)| d.is_finite() && *d <= radius)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frontier_steps_up_at_each_record() {
        let pts = [
            (3.0, 5.0),
            (1.0, 2.0),
            (2.0, 1.0),
            (4.0, 9.0),
            (5.0, 4.0),
            (f32::NAN, 99.0),
        ];
        assert_eq!(
            frontier(&pts),
            [
                (1.0, 2.0),
                (3.0, 2.0),
                (3.0, 5.0),
                (4.0, 5.0),
                (4.0, 9.0),
                (5.0, 9.0)
            ]
        );
        assert!(frontier(&[]).is_empty());
        assert_eq!(frontier(&[(1.0, 1.0)]), [(1.0, 1.0)]);
    }

    #[test]
    fn trails_follow_time_within_each_group() {
        let pts = [
            (3.0, 0.0),
            (1.0, 0.0),
            (2.0, 0.0),
            (0.0, f32::NAN),
            (5.0, 1.0),
        ];
        assert_eq!(trails(&pts, &[0, 0, 0, 1, 1]), [vec![1, 2, 0]]);
        assert!(trails(&[], &[]).is_empty());
    }

    #[test]
    fn the_trend_fits_a_line() {
        let pts = [(0.0, 1.0), (1.0, 3.0), (2.0, 5.0), (f32::NAN, 0.0)];
        assert_eq!(trend(&pts), Some((2.0, 1.0)));
        assert_eq!(trend(&[(1.0, 1.0), (1.0, 2.0)]), None);
        assert_eq!(trend(&[]), None);
    }

    #[test]
    fn nearest_only_within_the_radius() {
        let pts = [(0.0, 0.0), (10.0, 0.0), (f32::NAN, 0.0)];
        assert_eq!(nearest(&pts, (8.0, 1.0), 5.0), Some(1));
        assert_eq!(nearest(&pts, (50.0, 0.0), 5.0), None);
    }
}
