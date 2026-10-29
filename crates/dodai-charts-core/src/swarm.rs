//! Beeswarm layout: dots along one axis nudged sideways so none overlap. Pure.

/// The sideways offset of each dot (same units as `xs`) so that dots of radius `r` placed at
/// `(xs[i], offset[i])` do not overlap, each as close to the axis as possible. Dots are placed
/// in order of position; non-finite positions get offset 0. Pure.
pub fn swarm(xs: &[f32], r: f32) -> Vec<f32> {
    let d = (2.0 * r).max(0.0);
    let mut out = vec![0.0; xs.len()];
    if d <= 0.0 {
        return out;
    }
    let mut order: Vec<usize> = (0..xs.len()).filter(|i| xs[*i].is_finite()).collect();
    order.sort_by(|a, b| xs[*a].total_cmp(&xs[*b]));
    let mut placed: Vec<(f32, f32)> = Vec::new();
    for i in order {
        let x = xs[i];
        let near: Vec<(f32, f32)> = placed
            .iter()
            .copied()
            .filter(|(px, _)| (x - px).abs() < d)
            .collect();
        let mut candidates = vec![0.0f32];
        for (px, py) in &near {
            let dy = (d * d - (x - px).powi(2)).max(0.0).sqrt();
            candidates.push(py + dy);
            candidates.push(py - dy);
        }
        candidates.sort_by(|a, b| a.abs().total_cmp(&b.abs()));
        let y = candidates
            .into_iter()
            .find(|y| {
                near.iter()
                    .all(|(px, py)| (x - px).powi(2) + (y - py).powi(2) >= d * d - 1e-3)
            })
            .unwrap_or(0.0);
        out[i] = y;
        placed.push((x, y));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_two_dots_overlap_and_lone_dots_stay_on_the_axis() {
        let xs = [0.0, 0.5, 1.0, 1.2, 10.0, 0.1, 0.2];
        let ys = swarm(&xs, 1.0);
        for i in 0..xs.len() {
            for j in i + 1..xs.len() {
                let d2 = (xs[i] - xs[j]).powi(2) + (ys[i] - ys[j]).powi(2);
                assert!(d2 >= 4.0 - 1e-2, "{i} and {j} overlap");
            }
        }
        assert_eq!(ys[4], 0.0);
        assert_eq!(swarm(&[f32::NAN, 1.0], 1.0), vec![0.0, 0.0]);
        assert_eq!(swarm(&[1.0, 1.0], 0.0), vec![0.0, 0.0]);
    }
}
