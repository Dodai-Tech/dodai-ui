//! Sankey flow layout: columns, node boxes and ribbon offsets.
//!
//! All positions are fractions (`0..=1`) of the plot height; the toolkit only
//! maps them to pixels and paints.

/// A weighted flow between two node indices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Link {
    pub from: usize,
    pub to: usize,
    pub value: f32,
}

/// A node box: its column, top and height (fractions of the plot height).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NodeBox {
    pub col: usize,
    pub y: f32,
    pub h: f32,
    /// `max(inflow, outflow)`.
    pub total: f32,
}

/// A link ribbon: where it leaves the source and enters the target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ribbon {
    pub link: usize,
    pub from: usize,
    pub to: usize,
    /// Top edge at the source node (fraction of height).
    pub y_from: f32,
    /// Top edge at the target node (fraction of height).
    pub y_to: f32,
    /// Thickness (same at both ends).
    pub h: f32,
}

/// The computed layout.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    pub columns: usize,
    pub nodes: Vec<NodeBox>,
    pub ribbons: Vec<Ribbon>,
}

/// Lay out `n` nodes and `links`. `pad` is the vertical gap between nodes in a
/// column, as a fraction of the height. Invalid links (out of range, non
/// positive, self loops) are ignored; cycles don't hang (back edges are
/// placed in the column right of their source). Pure.
pub fn layout(n: usize, links: &[Link], pad: f32) -> Layout {
    let ok: Vec<(usize, Link)> = links
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, l)| {
            l.from < n && l.to < n && l.from != l.to && l.value.is_finite() && l.value > 0.0
        })
        .collect();
    if n == 0 {
        return Layout::default();
    }
    // Longest-path columns, bounded so cycles terminate.
    let mut col = vec![0usize; n];
    for _ in 0..n {
        let mut changed = false;
        for (_, l) in &ok {
            if col[l.from] + 1 > col[l.to] && col[l.from] + 1 < n {
                col[l.to] = col[l.from] + 1;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let columns = col.iter().copied().max().unwrap_or(0) + 1;
    let (mut inflow, mut outflow) = (vec![0.0f32; n], vec![0.0f32; n]);
    for (_, l) in &ok {
        outflow[l.from] += l.value;
        inflow[l.to] += l.value;
    }
    let total: Vec<f32> = (0..n).map(|i| inflow[i].max(outflow[i])).collect();
    // One scale for every column so ribbons keep their thickness.
    let mut k = f32::INFINITY;
    for c in 0..columns {
        let ids: Vec<usize> = (0..n).filter(|&i| col[i] == c).collect();
        let sum: f32 = ids.iter().map(|&i| total[i]).sum();
        if sum > 0.0 {
            let room = (1.0 - pad * ids.len().saturating_sub(1) as f32).max(0.05);
            k = k.min(room / sum);
        }
    }
    if !k.is_finite() {
        k = 0.0;
    }
    let mut nodes = vec![NodeBox::default(); n];
    for c in 0..columns {
        let ids: Vec<usize> = (0..n).filter(|&i| col[i] == c).collect();
        let used: f32 = ids.iter().map(|&i| total[i] * k).sum::<f32>()
            + pad * ids.len().saturating_sub(1) as f32;
        let mut y = ((1.0 - used) / 2.0).max(0.0);
        for &i in &ids {
            nodes[i] = NodeBox {
                col: c,
                y,
                h: total[i] * k,
                total: total[i],
            };
            y += total[i] * k + pad;
        }
    }
    // Ribbon offsets: outgoing sorted by target y, incoming by source y.
    let mut ribbons: Vec<Ribbon> = ok
        .iter()
        .map(|&(i, l)| Ribbon {
            link: i,
            from: l.from,
            to: l.to,
            y_from: 0.0,
            y_to: 0.0,
            h: l.value * k,
        })
        .collect();
    for node in 0..n {
        let mut out: Vec<usize> = (0..ribbons.len())
            .filter(|&r| ribbons[r].from == node)
            .collect();
        out.sort_by(|&a, &b| nodes[ribbons[a].to].y.total_cmp(&nodes[ribbons[b].to].y));
        let mut y = nodes[node].y;
        for r in out {
            ribbons[r].y_from = y;
            y += ribbons[r].h;
        }
        let mut inc: Vec<usize> = (0..ribbons.len())
            .filter(|&r| ribbons[r].to == node)
            .collect();
        inc.sort_by(|&a, &b| {
            nodes[ribbons[a].from]
                .y
                .total_cmp(&nodes[ribbons[b].from].y)
        });
        let mut y = nodes[node].y;
        for r in inc {
            ribbons[r].y_to = y;
            y += ribbons[r].h;
        }
    }
    Layout {
        columns,
        nodes,
        ribbons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(from: usize, to: usize, value: f32) -> Link {
        Link { from, to, value }
    }

    #[test]
    fn columns_follow_longest_path() {
        let lay = layout(
            4,
            &[l(0, 1, 5.0), l(1, 2, 5.0), l(0, 2, 1.0), l(2, 3, 6.0)],
            0.04,
        );
        let cols: Vec<usize> = lay.nodes.iter().map(|n| n.col).collect();
        assert_eq!(cols, vec![0, 1, 2, 3]);
        assert_eq!(lay.columns, 4);
    }

    #[test]
    fn node_total_is_max_of_in_and_out() {
        let lay = layout(3, &[l(0, 1, 4.0), l(1, 2, 3.0)], 0.0);
        assert_eq!(lay.nodes[1].total, 4.0);
    }

    #[test]
    fn nodes_fit_and_ribbons_stay_inside_their_nodes() {
        let links = [l(0, 2, 3.0), l(0, 3, 2.0), l(1, 2, 1.0), l(1, 3, 4.0)];
        let lay = layout(4, &links, 0.05);
        for n in &lay.nodes {
            assert!(n.y >= -1e-4 && n.y + n.h <= 1.0 + 1e-4);
        }
        for r in &lay.ribbons {
            let (s, t) = (lay.nodes[r.from], lay.nodes[r.to]);
            assert!(r.y_from >= s.y - 1e-4 && r.y_from + r.h <= s.y + s.h + 1e-4);
            assert!(r.y_to >= t.y - 1e-4 && r.y_to + r.h <= t.y + t.h + 1e-4);
        }
    }

    #[test]
    fn bad_links_and_cycles_are_safe() {
        let lay = layout(
            3,
            &[
                l(0, 1, 1.0),
                l(1, 0, 1.0),
                l(5, 1, 1.0),
                l(2, 2, 1.0),
                l(0, 2, -1.0),
                l(0, 2, f32::NAN),
            ],
            0.04,
        );
        assert_eq!(lay.ribbons.len(), 2);
        assert!(lay.nodes.iter().all(|n| n.y.is_finite() && n.h.is_finite()));
        assert_eq!(layout(0, &[], 0.1), Layout::default());
    }
}
