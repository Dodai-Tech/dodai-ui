//! Chord diagram layout: groups around a circle and ribbons between them. Angles are in
//! turns (`0..1`, clockwise from the top). Pure.

/// One ribbon between two groups.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ribbon {
    /// The group the ribbon leaves.
    pub from: usize,
    /// The group it reaches (may equal `from` for flow within a group).
    pub to: usize,
    /// Span on `from`'s arc: the flow `from → to`.
    pub source: (f32, f32),
    /// Span on `to`'s arc: the flow `to → from`.
    pub target: (f32, f32),
    /// Total flow both ways.
    pub value: f32,
}

/// The layout: one arc per group and the ribbons.
#[derive(Clone, Debug, PartialEq)]
pub struct Chords {
    /// Each group's `(start, end)` arc in turns.
    pub groups: Vec<(f32, f32)>,
    /// Ribbons, one per connected pair.
    pub ribbons: Vec<Ribbon>,
}

/// Lays out a square flow matrix (`m[i][j]` = flow from `i` to `j`): each group's arc is as
/// long as its outgoing total, split in target order; arcs are separated by `pad` turns.
/// Missing, non-finite or negative entries count as 0. Pure.
pub fn chord(m: &[Vec<f32>], pad: f32) -> Chords {
    let n = m.len();
    let at = |i: usize, j: usize| {
        m.get(i)
            .and_then(|r| r.get(j))
            .copied()
            .filter(|v| v.is_finite())
            .unwrap_or(0.0)
            .max(0.0)
    };
    let total: f32 = (0..n)
        .flat_map(|i| (0..n).map(move |j| (i, j)))
        .map(|(i, j)| at(i, j))
        .sum();
    let pad = if n > 1 {
        pad.clamp(0.0, 0.5 / n as f32)
    } else {
        0.0
    };
    let room = (1.0 - pad * n as f32).max(0.0);
    let k = if total > 0.0 { room / total } else { 0.0 };
    let mut groups = Vec::with_capacity(n);
    let mut sub = vec![vec![(0.0, 0.0); n]; n];
    let mut a = 0.0;
    for (i, row) in sub.iter_mut().enumerate() {
        let start = a;
        for (j, s) in row.iter_mut().enumerate() {
            let w = at(i, j) * k;
            *s = (a, a + w);
            a += w;
        }
        groups.push((start, a));
        a += pad;
    }
    let ribbons = (0..n)
        .flat_map(|i| (i..n).map(move |j| (i, j)))
        .filter_map(|(i, j)| {
            let value = at(i, j) + if i == j { 0.0 } else { at(j, i) };
            (value > 0.0).then(|| Ribbon {
                from: i,
                to: j,
                source: sub[i][j],
                target: sub[j][i],
                value,
            })
        })
        .collect();
    Chords { groups, ribbons }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arcs_follow_outgoing_totals_and_ribbons_pair_the_flows() {
        let c = chord(&[vec![0.0, 2.0], vec![2.0, 0.0]], 0.0);
        assert_eq!(c.groups, vec![(0.0, 0.5), (0.5, 1.0)]);
        assert_eq!(c.ribbons.len(), 1);
        let r = c.ribbons[0];
        assert_eq!((r.from, r.to, r.value), (0, 1, 4.0));
        assert_eq!(r.source, (0.0, 0.5));
        assert_eq!(r.target, (0.5, 1.0));
    }

    #[test]
    fn padding_and_hostile_input_are_safe() {
        let c = chord(&[vec![1.0, f32::NAN, 1.0], vec![1.0], vec![]], 0.05);
        assert_eq!(c.groups.len(), 3);
        assert!((c.groups[1].0 - c.groups[0].1 - 0.05).abs() < 1e-6);
        assert!(chord(&[], 0.1).ribbons.is_empty());
        assert!(chord(&[vec![0.0]], 0.1).ribbons.is_empty());
    }
}
