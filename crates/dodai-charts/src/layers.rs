//! Stacked layers (stacked areas, stream graphs): where each layer's lower and upper edge sit
//! at every x. Pure, so every toolkit stacks the same way.

/// Where a stack of layers sits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StackOffset {
    /// On zero, one layer on top of the other (a stacked area).
    #[default]
    Zero,
    /// Every x stretched to a total of 1 (each layer's share, a 100% stacked area).
    Expand,
    /// Centred on zero: a symmetric stream graph.
    Silhouette,
    /// A stream graph whose baseline moves so the layers bend as little as possible (easier to
    /// follow one layer through the stream).
    Wiggle,
}

impl StackOffset {
    /// Whether the stack floats free of zero (a stream graph: its value axis means nothing).
    pub fn floats(self) -> bool {
        matches!(self, Self::Silhouette | Self::Wiggle)
    }
}

/// `(lower, upper)` per layer per x for `values` (one row per layer; missing or negative values
/// count as 0, so layers never cross). Pure.
pub fn stack_layers(values: &[Vec<f32>], offset: StackOffset) -> Vec<Vec<(f32, f32)>> {
    let n = values.iter().map(Vec::len).max().unwrap_or(0);
    let v = |k: usize, i: usize| {
        values[k]
            .get(i)
            .copied()
            .filter(|x| x.is_finite())
            .unwrap_or(0.0)
            .max(0.0)
    };
    let total: Vec<f32> = (0..n)
        .map(|i| (0..values.len()).map(|k| v(k, i)).sum())
        .collect();
    let base: Vec<f32> = match offset {
        StackOffset::Zero | StackOffset::Expand => vec![0.0; n],
        StackOffset::Silhouette => total.iter().map(|t| -t / 2.0).collect(),
        StackOffset::Wiggle => {
            // Byron & Wattenberg: move the baseline against the weighted slope of the layers.
            let mut base = vec![0.0; n];
            for i in 1..n {
                let (mut slope, mut weight) = (0.0, 0.0);
                let mut below = 0.0;
                for k in 0..values.len() {
                    let d = v(k, i) - v(k, i - 1);
                    slope += (below + d / 2.0) * v(k, i);
                    weight += v(k, i);
                    below += d;
                }
                base[i] = base[i - 1] - if weight > 0.0 { slope / weight } else { 0.0 };
            }
            // Centre the stream on zero over its run.
            let mid = (0..n).map(|i| base[i] + total[i] / 2.0).sum::<f32>() / n.max(1) as f32;
            base.iter().map(|b| b - mid).collect()
        }
    };
    let scale = |i: usize| match offset {
        StackOffset::Expand if total[i] > 0.0 => 1.0 / total[i],
        _ => 1.0,
    };
    let mut acc = base;
    (0..values.len())
        .map(|k| {
            (0..n)
                .map(|i| {
                    let lo = acc[i];
                    acc[i] += v(k, i) * scale(i);
                    (lo, acc[i])
                })
                .collect()
        })
        .collect()
}

/// The lowest lower and highest upper edge of `layers` (`(0, 1)` when empty). Pure.
pub fn layers_extent(layers: &[Vec<(f32, f32)>]) -> (f32, f32) {
    let (lo, hi) = layers
        .iter()
        .flatten()
        .fold((f32::MAX, f32::MIN), |(l, h), (a, b)| {
            (l.min(*a), h.max(*b))
        });
    if lo > hi {
        (0.0, 1.0)
    } else {
        (lo, hi)
    }
}

/// Diverging stack of one row: segment spans relative to a centre at 0, the first `split`
/// segments to the left (negative) and the rest to the right. A fractional `split` puts that
/// segment across the centre (2.5 of 5 = a neutral middle answer, half each side). Values are
/// used as magnitudes; non-finite counts as 0. Pure.
pub fn diverging(row: &[f32], split: f32) -> Vec<(f32, f32)> {
    let v: Vec<f32> = row
        .iter()
        .map(|v| if v.is_finite() { v.abs() } else { 0.0 })
        .collect();
    let split = if split.is_finite() {
        split.clamp(0.0, v.len() as f32)
    } else {
        0.0
    };
    let whole = split.floor() as usize;
    // Everything left of the centre, measured from the centre outwards.
    let left: f32 =
        v[..whole].iter().sum::<f32>() + v.get(whole).map_or(0.0, |x| x * (split - whole as f32));
    let mut at = -left;
    v.iter()
        .map(|x| {
            let s = (at, at + x);
            at += x;
            s
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<Vec<f32>> {
        vec![vec![1.0, 3.0, 2.0], vec![2.0, 1.0, 4.0]]
    }

    #[test]
    fn zero_stacks_one_layer_on_the_other() {
        let l = stack_layers(&rows(), StackOffset::Zero);
        assert_eq!(l[0], vec![(0.0, 1.0), (0.0, 3.0), (0.0, 2.0)]);
        assert_eq!(l[1], vec![(1.0, 3.0), (3.0, 4.0), (2.0, 6.0)]);
    }

    #[test]
    fn expand_fills_every_x_to_one() {
        let l = stack_layers(&rows(), StackOffset::Expand);
        assert!(l[1].iter().all(|p| (p.1 - 1.0).abs() < 1e-6));
        assert!((l[0][0].1 - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn a_silhouette_is_centred_on_zero() {
        let l = stack_layers(&rows(), StackOffset::Silhouette);
        for (bottom, top) in l[0].iter().zip(&l[1]) {
            assert!((bottom.0 + top.1).abs() < 1e-6, "symmetric");
        }
    }

    #[test]
    fn a_wiggle_keeps_thickness_and_moves_less_than_a_silhouette() {
        let rows = vec![
            vec![1.0, 1.0, 1.0, 1.0],
            vec![0.0, 4.0, 8.0, 12.0],
            vec![2.0, 2.0, 2.0, 2.0],
        ];
        let w = stack_layers(&rows, StackOffset::Wiggle);
        let s = stack_layers(&rows, StackOffset::Silhouette);
        for (k, row) in rows.iter().enumerate() {
            for (i, v) in row.iter().enumerate() {
                assert!((w[k][i].1 - w[k][i].0 - v).abs() < 1e-5, "thickness kept");
            }
        }
        // Its whole point: the layers' midlines bend less, weighted by thickness.
        let bend = |l: &Vec<Vec<(f32, f32)>>| {
            let mid = |p: (f32, f32)| (p.0 + p.1) / 2.0;
            let mut sum = 0.0;
            for (k, layer) in l.iter().enumerate() {
                for i in 1..layer.len() {
                    sum += rows[k][i] * (mid(layer[i]) - mid(layer[i - 1])).powi(2);
                }
            }
            sum
        };
        assert!(bend(&w) < bend(&s), "{} vs {}", bend(&w), bend(&s));
    }

    #[test]
    fn hostile_and_ragged_rows_stay_finite() {
        let rows = vec![vec![f32::NAN, -3.0, 2.0], vec![1.0]];
        for o in [
            StackOffset::Zero,
            StackOffset::Expand,
            StackOffset::Silhouette,
            StackOffset::Wiggle,
        ] {
            let l = stack_layers(&rows, o);
            assert!(l
                .iter()
                .flatten()
                .all(|(a, b)| a.is_finite() && b.is_finite() && a <= b));
        }
        assert!(stack_layers(&[], StackOffset::Wiggle).is_empty());
        assert_eq!(layers_extent(&[]), (0.0, 1.0));
    }

    #[test]
    fn diverging_splits_left_and_right_of_the_centre() {
        assert_eq!(
            diverging(&[1.0, 2.0, 4.0], 2.0),
            vec![(-3.0, -2.0), (-2.0, 0.0), (0.0, 4.0)]
        );
        // A neutral middle answer straddles the centre.
        assert_eq!(
            diverging(&[2.0, 2.0, 2.0], 1.5),
            vec![(-3.0, -1.0), (-1.0, 1.0), (1.0, 3.0)]
        );
        assert_eq!(diverging(&[f32::NAN], f32::NAN), vec![(0.0, 0.0)]);
    }
}
