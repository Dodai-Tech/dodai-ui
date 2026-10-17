//! Stacking several charts with a shared gutter and linked axis.

/// Split `total` along one axis between panes by `weights`, with `gap`
/// between them. Returns `(offset, length)` per pane. Pure.
pub fn split(total: f32, weights: &[f32], gap: f32) -> Vec<(f32, f32)> {
    let n = weights.len();
    if n == 0 {
        return Vec::new();
    }
    let free = (total - gap * (n as f32 - 1.0)).max(0.0);
    let sum: f32 = weights.iter().map(|w| w.max(0.0)).sum();
    let mut at = 0.0;
    weights
        .iter()
        .map(|w| {
            let len = if sum > 0.0 {
                free * w.max(0.0) / sum
            } else {
                free / n as f32
            };
            let r = (at, len);
            at += len + gap;
            r
        })
        .collect()
}

/// The shared y-gutter: the widest axis-label column across panes, so plot
/// areas line up vertically. Pure.
pub fn shared_gutter(label_widths: &[f32]) -> f32 {
    label_widths.iter().copied().fold(0.0, f32::max)
}

/// Map an x index/value to a plot x position, identical for all panes that
/// share `(lo, hi)`.
pub fn linked_x(v: f32, lo: f32, hi: f32, plot_w: f32) -> f32 {
    if hi > lo {
        (v - lo) / (hi - lo) * plot_w
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_by_weight_with_gap() {
        let s = split(110.0, &[2.0, 1.0], 10.0);
        assert_eq!(
            s,
            vec![(0.0, 400.0 / 6.0), (400.0 / 6.0 + 10.0, 200.0 / 6.0)]
        );
        assert!(split(10.0, &[], 1.0).is_empty());
        let eq = split(100.0, &[0.0, 0.0], 0.0);
        assert_eq!(eq[1], (50.0, 50.0));
    }

    #[test]
    fn gutter_and_link() {
        assert_eq!(shared_gutter(&[20.0, 34.0, 8.0]), 34.0);
        assert_eq!(linked_x(5.0, 0.0, 10.0, 200.0), 100.0);
        assert_eq!(linked_x(1.0, 3.0, 3.0, 50.0), 0.0);
    }
}
