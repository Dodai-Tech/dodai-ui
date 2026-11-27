//! Voronoi diagram: a rectangle split into one convex cell per site, each cell holding the points
//! nearer to its site than to any other, plus the Delaunay links (sites whose cells share an
//! edge). Pure and deterministic; `O(n²)`, meant for tens of sites, not thousands.

use crate::shards::{clip, contains, Poly};

/// A Voronoi diagram of some sites.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Diagram {
    /// Cell `i` belongs to site `i`; empty when two sites coincide and the later one lost out.
    pub cells: Vec<Poly>,
    /// Delaunay edges `(i, j)` with `i < j`: the sites whose cells share an edge.
    pub links: Vec<(usize, usize)>,
}

/// The diagram of `sites` clipped to the box `(0, 0)`–`(w, h)`.
pub fn diagram(sites: &[(f32, f32)], w: f32, h: f32) -> Diagram {
    let frame: Poly = vec![(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)];
    let mut cells = Vec::with_capacity(sites.len());
    for (i, s) in sites.iter().enumerate() {
        let mut cell = frame.clone();
        for (j, t) in sites.iter().enumerate() {
            if i == j || cell.len() < 3 {
                continue;
            }
            if t == s {
                // a coincident site: the earlier one keeps the plane
                if j < i {
                    cell.clear();
                }
                continue;
            }
            let n = (2.0 * (t.0 - s.0), 2.0 * (t.1 - s.1));
            let d = t.0 * t.0 + t.1 * t.1 - s.0 * s.0 - s.1 * s.1;
            cell = clip(&cell, n, d);
        }
        cells.push(if cell.len() < 3 { Vec::new() } else { cell });
    }
    let mut links = Vec::new();
    for i in 0..sites.len() {
        for j in i + 1..sites.len() {
            if shares_edge(&cells[i], sites[i], sites[j])
                && shares_edge(&cells[j], sites[j], sites[i])
            {
                links.push((i, j));
            }
        }
    }
    Diagram { cells, links }
}

/// Whether `cell` (of site `s`) has an edge on the bisector with `t`: two distinct corners on it.
fn shares_edge(cell: &[(f32, f32)], s: (f32, f32), t: (f32, f32)) -> bool {
    let (nx, ny) = (t.0 - s.0, t.1 - s.1);
    let len = nx.hypot(ny);
    if cell.len() < 3 || len <= 0.0 {
        return false;
    }
    // distance of a corner from the bisector line
    let mid = ((s.0 + t.0) / 2.0, (s.1 + t.1) / 2.0);
    let on: Vec<(f32, f32)> = cell
        .iter()
        .copied()
        .filter(|q| (((q.0 - mid.0) * nx + (q.1 - mid.1) * ny) / len).abs() < 1e-3 * (1.0 + len))
        .collect();
    on.iter().any(|a| {
        on.iter()
            .any(|b| (a.0 - b.0).hypot(a.1 - b.1) > 1e-2 * (1.0 + len))
    })
}

/// The site nearest to `q`, if there are any (the first of equally near ones).
pub fn nearest(sites: &[(f32, f32)], q: (f32, f32)) -> Option<usize> {
    sites
        .iter()
        .enumerate()
        .map(|(i, s)| (i, (s.0 - q.0).powi(2) + (s.1 - q.1).powi(2)))
        .filter(|(_, d)| d.is_finite())
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// The cell under `q`; the same site as [`nearest`], by the cell's own outline.
pub fn hit(d: &Diagram, q: (f32, f32)) -> Option<usize> {
    d.cells.iter().position(|c| contains(c, q))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shards::area;

    const SITES: [(f32, f32); 6] = [
        (10.0, 12.0),
        (80.0, 20.0),
        (45.0, 50.0),
        (20.0, 85.0),
        (90.0, 90.0),
        (60.0, 75.0),
    ];

    #[test]
    fn the_cells_tile_the_box() {
        let d = diagram(&SITES, 100.0, 100.0);
        let total: f32 = d.cells.iter().map(|c| area(c)).sum();
        assert!((total - 10_000.0).abs() < 1.0, "{total}");
    }

    #[test]
    fn a_cell_holds_the_points_nearest_its_site() {
        let d = diagram(&SITES, 100.0, 100.0);
        for k in 0..400 {
            let q = ((k % 20) as f32 * 5.0 + 2.5, (k / 20) as f32 * 5.0 + 2.5);
            let by_cell = hit(&d, q).unwrap();
            let by_distance = nearest(&SITES, q).unwrap();
            let gap = |i: usize| (SITES[i].0 - q.0).hypot(SITES[i].1 - q.1);
            assert!((gap(by_cell) - gap(by_distance)).abs() < 1e-2, "{q:?}");
        }
    }

    #[test]
    fn two_sites_split_the_box_down_the_middle() {
        let d = diagram(&[(25.0, 50.0), (75.0, 50.0)], 100.0, 100.0);
        assert!((area(&d.cells[0]) - 5_000.0).abs() < 1.0);
        assert_eq!(d.links, [(0, 1)]);
    }

    #[test]
    fn links_join_neighbours_only() {
        // three in a row: the ends are not neighbours
        let d = diagram(&[(10.0, 50.0), (50.0, 50.0), (90.0, 50.0)], 100.0, 100.0);
        assert_eq!(d.links, [(0, 1), (1, 2)]);
    }

    #[test]
    fn a_coincident_site_gets_no_cell() {
        let d = diagram(&[(50.0, 50.0), (50.0, 50.0)], 100.0, 100.0);
        assert!(!d.cells[0].is_empty() && d.cells[1].is_empty());
    }

    #[test]
    fn no_sites_no_cells() {
        assert_eq!(diagram(&[], 10.0, 10.0), Diagram::default());
        assert_eq!(nearest(&[], (0.0, 0.0)), None);
    }
}
