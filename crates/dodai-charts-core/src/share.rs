//! Part-of-a-whole layouts: waffle cell counts and marimekko (mosaic) rectangles. Pure.

/// How many of `cells` cells each value gets, proportional to its share (largest remainder,
/// so the counts always add up to `cells` when any value is positive). Non-finite or
/// negative values get none. Pure.
pub fn waffle_cells(values: &[f32], cells: usize) -> Vec<usize> {
    let v: Vec<f32> = values
        .iter()
        .map(|v| if v.is_finite() { v.max(0.0) } else { 0.0 })
        .collect();
    let total: f32 = v.iter().sum();
    if total <= 0.0 {
        return vec![0; v.len()];
    }
    let exact: Vec<f32> = v.iter().map(|x| x / total * cells as f32).collect();
    let mut out: Vec<usize> = exact.iter().map(|e| e.floor() as usize).collect();
    let mut left = cells.saturating_sub(out.iter().sum());
    let mut order: Vec<usize> = (0..v.len()).collect();
    order.sort_by(|a, b| {
        (exact[*b] - exact[*b].floor()).total_cmp(&(exact[*a] - exact[*a].floor()))
    });
    for i in order {
        if left == 0 {
            break;
        }
        if v[i] > 0.0 {
            out[i] += 1;
            left -= 1;
        }
    }
    out
}

/// One rectangle of a marimekko chart, all in `0..=1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tile {
    /// Left edge.
    pub x0: f32,
    /// Right edge.
    pub x1: f32,
    /// Top edge (0 = top).
    pub y0: f32,
    /// Bottom edge.
    pub y1: f32,
}

/// Marimekko layout: each column is as wide as its share of the grand total, and split top to
/// bottom by its segments' shares. `columns[c][s]` is segment `s` of column `c`. Non-finite or
/// negative values count as 0. Pure.
pub fn marimekko(columns: &[Vec<f32>]) -> Vec<Vec<Tile>> {
    let clean = |v: f32| if v.is_finite() { v.max(0.0) } else { 0.0 };
    let totals: Vec<f32> = columns
        .iter()
        .map(|c| c.iter().map(|v| clean(*v)).sum())
        .collect();
    let grand: f32 = totals.iter().sum();
    let mut x = 0.0;
    columns
        .iter()
        .zip(&totals)
        .map(|(c, tot)| {
            let w = if grand > 0.0 { tot / grand } else { 0.0 };
            let mut y = 0.0;
            let tiles = c
                .iter()
                .map(|v| {
                    let h = if *tot > 0.0 { clean(*v) / tot } else { 0.0 };
                    let t = Tile {
                        x0: x,
                        x1: x + w,
                        y0: y,
                        y1: y + h,
                    };
                    y += h;
                    t
                })
                .collect();
            x += w;
            tiles
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waffle_cells_always_fill_the_grid() {
        assert_eq!(waffle_cells(&[1.0, 1.0, 1.0], 100), vec![34, 33, 33]);
        assert_eq!(waffle_cells(&[5.0, f32::NAN, -2.0], 10), vec![10, 0, 0]);
        assert_eq!(waffle_cells(&[0.0], 10), vec![0]);
        let c = waffle_cells(&[0.15, 0.62, 0.23], 100);
        assert_eq!(c.iter().sum::<usize>(), 100);
    }

    #[test]
    fn marimekko_widths_follow_totals_and_heights_follow_shares() {
        let m = marimekko(&[vec![1.0, 3.0], vec![4.0, 0.0]]);
        assert_eq!((m[0][0].x0, m[0][0].x1), (0.0, 0.5));
        assert_eq!((m[0][0].y0, m[0][0].y1), (0.0, 0.25));
        assert_eq!((m[0][1].y0, m[0][1].y1), (0.25, 1.0));
        assert_eq!((m[1][0].x0, m[1][0].x1, m[1][0].y1), (0.5, 1.0, 1.0));
        assert!(marimekko(&[vec![f32::NAN]])[0][0].x1 == 0.0);
    }
}
