//! Split flow layout: a quantity flowing down through levels, each node splitting into its
//! children. Blocks are as wide as their value (one scale for every level); each child hangs
//! under its slice of the parent, nudged sideways only as far as needed to keep a gap between
//! blocks. Pure; the toolkit paints the blocks and the ribbons between them.

/// One node: its value and the node it splits from (an earlier index; `None` for a root).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Node {
    /// The node it flows from; must be an earlier index, otherwise it counts as a root.
    pub parent: Option<usize>,
    /// Its size; not finite or negative counts as 0.
    pub value: f32,
}

/// A laid-out block: its level and horizontal extent in `0..=width`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Block {
    /// Depth: roots are level 0.
    pub level: usize,
    /// Left edge.
    pub x0: f32,
    /// Right edge.
    pub x1: f32,
    /// The slice of the parent's bottom edge this node flows out of (`x0`, `x1`), if any.
    pub source: Option<(f32, f32)>,
}

fn clean(v: f32) -> f32 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}

/// The level of each node (a parent at or after the node counts as none).
pub fn levels(nodes: &[Node]) -> Vec<usize> {
    let mut lv = vec![0usize; nodes.len()];
    for i in 0..nodes.len() {
        if let Some(p) = nodes[i].parent.filter(|p| *p < i) {
            lv[i] = lv[p] + 1;
        }
    }
    lv
}

/// Lays `nodes` out across `width` px with `gap` px between neighbouring blocks.
pub fn layout(nodes: &[Node], width: f32, gap: f32) -> Vec<Block> {
    let width = if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    };
    let gap = if gap.is_finite() { gap.max(0.0) } else { 0.0 };
    let lv = levels(nodes);
    let depth = lv.iter().copied().max().map_or(0, |m| m + 1);
    let rows: Vec<Vec<usize>> = (0..depth)
        .map(|l| (0..nodes.len()).filter(|i| lv[*i] == l).collect())
        .collect();
    // One scale for all levels: the widest row (values plus gaps) fits.
    let scale = rows
        .iter()
        .filter(|r| !r.is_empty())
        .map(|r| {
            let sum: f32 = r.iter().map(|i| clean(nodes[*i].value)).sum();
            let room = (width - gap * (r.len() - 1) as f32).max(0.0);
            if sum > 0.0 {
                room / sum
            } else {
                f32::INFINITY
            }
        })
        .fold(f32::INFINITY, f32::min);
    let scale = if scale.is_finite() { scale } else { 0.0 };
    let mut out: Vec<Block> = (0..nodes.len())
        .map(|i| Block {
            level: lv[i],
            x0: 0.0,
            x1: 0.0,
            source: None,
        })
        .collect();
    // Where the next child of each node starts on its bottom edge.
    let mut next_out = vec![0.0f32; nodes.len()];
    for row in &rows {
        // Children follow their parents' order; roots are centred as one row.
        let mut row = row.clone();
        row.sort_by(|a, b| {
            let key = |i: usize| nodes[i].parent.filter(|p| *p < i).map(|p| out[p].x0);
            key(*a)
                .partial_cmp(&key(*b))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let roots_w: f32 = row
            .iter()
            .filter(|i| nodes[**i].parent.filter(|p| *p < **i).is_none())
            .map(|i| clean(nodes[*i].value) * scale + gap)
            .sum::<f32>()
            - gap;
        let mut root_at = ((width - roots_w) / 2.0).max(0.0);
        let mut ideal = Vec::with_capacity(row.len());
        for &i in &row {
            let w = clean(nodes[i].value) * scale;
            let x = match nodes[i].parent.filter(|p| *p < i) {
                Some(p) => {
                    let from = out[p].x0 + next_out[p];
                    let src = (from.min(out[p].x1), (from + w).min(out[p].x1));
                    out[i].source = Some(src);
                    next_out[p] += w;
                    from
                }
                None => {
                    let x = root_at;
                    root_at += w + gap;
                    x
                }
            };
            ideal.push((i, x, w));
        }
        // Push right to keep the gaps, then back left from the edge.
        let mut placed: Vec<(usize, f32, f32)> = Vec::with_capacity(ideal.len());
        for (i, x, w) in ideal {
            let x = placed.last().map_or(x, |&(_, px, pw)| x.max(px + pw + gap));
            placed.push((i, x, w));
        }
        let mut limit = width;
        for (_, x, w) in placed.iter_mut().rev() {
            if *x + *w > limit {
                *x = limit - *w;
            }
            *x = x.max(0.0);
            limit = *x - gap;
        }
        for (i, x, w) in placed {
            out[i].x0 = x;
            out[i].x1 = x + w;
        }
    }
    out
}

/// Points along a vertical ribbon from the slice `top` at height `y0` down to the slice
/// `bottom` at `y1`, as a closed polygon (`steps` points per edge, eased like a cubic S).
pub fn ribbon(
    top: (f32, f32),
    bottom: (f32, f32),
    y0: f32,
    y1: f32,
    steps: usize,
) -> Vec<(f32, f32)> {
    let steps = steps.max(2);
    let ease = |t: f32| t * t * (3.0 - 2.0 * t);
    let edge = |a: f32, b: f32| {
        (0..steps).map(move |k| {
            let t = k as f32 / (steps - 1) as f32;
            (a + (b - a) * ease(t), y0 + (y1 - y0) * t)
        })
    };
    let mut p: Vec<(f32, f32)> = edge(top.0, bottom.0).collect();
    let right: Vec<(f32, f32)> = edge(top.1, bottom.1).collect();
    p.extend(right.into_iter().rev());
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(parent: Option<usize>, value: f32) -> Node {
        Node { parent, value }
    }

    #[test]
    fn widths_share_one_scale_and_children_hang_under_their_slice() {
        let nodes = [n(None, 100.0), n(Some(0), 60.0), n(Some(0), 40.0)];
        let b = layout(&nodes, 210.0, 10.0);
        // The widest row is the children's: 100 units + one gap in 210 px.
        assert_eq!((b[0].x1 - b[0].x0, b[1].x1 - b[1].x0), (200.0, 120.0));
        assert_eq!(b[1].source, Some((b[0].x0, b[0].x0 + 120.0)));
        assert_eq!(b[2].source, Some((b[0].x0 + 120.0, b[0].x1)));
        assert!(b[2].x0 >= b[1].x1 + 10.0 - 1e-4 && b[2].x1 <= 210.0 + 1e-4);
        assert_eq!(levels(&nodes), [0, 1, 1]);
    }

    #[test]
    fn rows_never_overlap_or_leave_the_width() {
        let nodes = [
            n(None, 10.0),
            n(Some(0), 5.0),
            n(Some(0), 5.0),
            n(Some(1), 1.0),
            n(Some(1), 1.0),
            n(Some(1), 1.0),
            n(Some(2), 4.0),
            n(Some(2), 1.0),
        ];
        let b = layout(&nodes, 300.0, 8.0);
        for l in 0..3 {
            let mut row: Vec<&Block> = b.iter().filter(|x| x.level == l).collect();
            row.sort_by(|a, c| a.x0.total_cmp(&c.x0));
            for w in row.windows(2) {
                assert!(w[1].x0 >= w[0].x1 + 8.0 - 1e-3, "{l}: {w:?}");
            }
            assert!(row.iter().all(|x| x.x0 >= 0.0 && x.x1 <= 300.0 + 1e-3));
        }
    }

    #[test]
    fn hostile_input_stays_finite() {
        let nodes = [
            n(Some(5), f32::NAN),
            n(Some(0), f32::INFINITY),
            n(Some(1), -4.0),
        ];
        for w in [f32::NAN, 0.0, 100.0] {
            for b in layout(&nodes, w, f32::NAN) {
                assert!(b.x0.is_finite() && b.x1.is_finite());
            }
        }
        assert!(layout(&[], 100.0, 4.0).is_empty());
        assert_eq!(ribbon((0.0, 1.0), (2.0, 3.0), 0.0, 10.0, 0).len(), 4);
    }
}
