//! Force-directed graph layout (Fruchterman–Reingold): linked nodes pull together, all nodes
//! push apart. Deterministic: starts from a circle, cools over a fixed number of steps. Pure.

/// Positions in `0..=1` for `n` nodes linked by `edges` (`(a, b, weight)`), after `steps`
/// iterations, scaled to fill the unit square with a 5 % margin. Edges to missing nodes and
/// non-finite weights are ignored. Pure.
pub fn force_layout(n: usize, edges: &[(usize, usize, f32)], steps: usize) -> Vec<(f32, f32)> {
    if n == 0 {
        return Vec::new();
    }
    let mut p: Vec<(f32, f32)> = (0..n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            (0.5 + 0.4 * a.cos(), 0.5 + 0.4 * a.sin())
        })
        .collect();
    if n == 1 {
        return vec![(0.5, 0.5)];
    }
    let edges: Vec<(usize, usize, f32)> = edges
        .iter()
        .copied()
        .filter(|(a, b, w)| *a < n && *b < n && a != b && w.is_finite() && *w > 0.0)
        .collect();
    let k = (1.0 / n as f32).sqrt();
    let mut temp = 0.1;
    let cool = temp / (steps.max(1) as f32 + 1.0);
    for _ in 0..steps {
        let mut d = vec![(0.0f32, 0.0f32); n];
        for i in 0..n {
            for j in i + 1..n {
                let (dx, dy) = (p[i].0 - p[j].0, p[i].1 - p[j].1);
                let dist = (dx * dx + dy * dy).sqrt().max(1e-3);
                let f = k * k / dist;
                let (fx, fy) = (dx / dist * f, dy / dist * f);
                d[i].0 += fx;
                d[i].1 += fy;
                d[j].0 -= fx;
                d[j].1 -= fy;
            }
        }
        for &(a, b, w) in &edges {
            let (dx, dy) = (p[a].0 - p[b].0, p[a].1 - p[b].1);
            let dist = (dx * dx + dy * dy).sqrt().max(1e-3);
            let f = dist * dist / k * w.min(4.0);
            let (fx, fy) = (dx / dist * f, dy / dist * f);
            d[a].0 -= fx;
            d[a].1 -= fy;
            d[b].0 += fx;
            d[b].1 += fy;
        }
        for (pi, di) in p.iter_mut().zip(&d) {
            let len = (di.0 * di.0 + di.1 * di.1).sqrt().max(1e-6);
            let step = len.min(temp);
            pi.0 += di.0 / len * step;
            pi.1 += di.1 / len * step;
        }
        temp = (temp - cool).max(1e-4);
    }
    // Fit to the unit square, keeping the aspect.
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for (x, y) in &p {
        x0 = x0.min(*x);
        y0 = y0.min(*y);
        x1 = x1.max(*x);
        y1 = y1.max(*y);
    }
    let span = (x1 - x0).max(y1 - y0).max(1e-6);
    let (ox, oy) = ((span - (x1 - x0)) / 2.0, (span - (y1 - y0)) / 2.0);
    p.into_iter()
        .map(|(x, y)| {
            (
                0.05 + 0.9 * (x - x0 + ox) / span,
                0.05 + 0.9 * (y - y0 + oy) / span,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
        ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
    }

    #[test]
    fn linked_nodes_end_closer_than_unlinked_ones() {
        // Two triangles joined by one edge.
        let e = [
            (0, 1, 1.0),
            (1, 2, 1.0),
            (2, 0, 1.0),
            (3, 4, 1.0),
            (4, 5, 1.0),
            (5, 3, 1.0),
            (2, 3, 1.0),
        ];
        let p = force_layout(6, &e, 300);
        assert!(p
            .iter()
            .all(|(x, y)| (0.0..=1.0).contains(x) && (0.0..=1.0).contains(y)));
        assert!(dist(p[0], p[1]) < dist(p[0], p[5]));
        assert_eq!(p, force_layout(6, &e, 300), "deterministic");
    }

    #[test]
    fn tiny_and_hostile_graphs_are_safe() {
        assert!(force_layout(0, &[], 10).is_empty());
        assert_eq!(force_layout(1, &[], 10), vec![(0.5, 0.5)]);
        let p = force_layout(2, &[(0, 9, 1.0), (0, 1, f32::NAN), (0, 0, 1.0)], 10);
        assert!(p.iter().all(|(x, y)| x.is_finite() && y.is_finite()));
    }
}
