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

// ---------------------------------------------------------------------------------------------
// The raster field: every pixel of a box owned by a site, for the variants a straight-edged
// polygon cannot draw (other metrics, power diagrams, farthest site, k-NN regions, relaxation).
// ---------------------------------------------------------------------------------------------

/// Marks a pixel that belongs to nobody (outside a clip) or has no category winner.
pub const NONE: u32 = u32::MAX;

/// The distance that decides which site is nearest.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Metric {
    /// Straight-line distance: straight-edged convex cells.
    #[default]
    Euclidean,
    /// `|dx| + |dy|`: diamond-flavoured cells.
    Manhattan,
    /// `max(|dx|, |dy|)`: boxy cells.
    Chebyshev,
    /// Order `p`, clamped to `0.5..=16`; 1 is Manhattan, 2 Euclidean, large `p` Chebyshev.
    Minkowski(f32),
}

impl Metric {
    /// The next metric, wrapping round.
    pub fn next(self) -> Self {
        match self {
            Self::Euclidean => Self::Manhattan,
            Self::Manhattan => Self::Chebyshev,
            Self::Chebyshev => Self::Minkowski(3.0),
            Self::Minkowski(_) => Self::Euclidean,
        }
    }

    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Euclidean => "Euclidean",
            Self::Manhattan => "Manhattan",
            Self::Chebyshev => "Chebyshev",
            Self::Minkowski(_) => "Minkowski",
        }
    }

    /// Distance for the absolute offsets `dx`, `dy`.
    pub fn dist(self, dx: f32, dy: f32) -> f32 {
        match self {
            Self::Euclidean => dx.hypot(dy),
            Self::Manhattan => dx + dy,
            Self::Chebyshev => dx.max(dy),
            Self::Minkowski(p) => {
                let p = if p.is_finite() {
                    p.clamp(0.5, 16.0)
                } else {
                    2.0
                };
                (dx.powf(p) + dy.powf(p)).powf(1.0 / p)
            }
        }
    }

    /// A key that orders like the distance; with `r` it is the power-diagram key.
    fn key(self, dx: f32, dy: f32, r: f32) -> f32 {
        match self {
            Self::Euclidean => dx * dx + dy * dy - r * r,
            m => m.dist(dx, dy) - r,
        }
    }
}

/// One site, in the pixels of the raster.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Site {
    pub x: f32,
    pub y: f32,
    /// Size of the site's cell in a weighted diagram.
    pub weight: Option<f32>,
    /// Class the site belongs to: same-class cells merge and k-NN votes use it.
    pub category: Option<usize>,
}

/// How the field is computed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spec {
    pub metric: Metric,
    /// Heavier sites own larger cells (a power diagram).
    pub weighted: bool,
    /// Every pixel takes its farthest site instead of its nearest.
    pub farthest: bool,
    /// Majority category among the `knn` nearest sites; `<2` is the plain diagram.
    pub knn: usize,
    /// Lloyd steps: sites move to their cells' centroids (capped at 30).
    pub relax: usize,
    /// Same-category cells share one border-free region.
    pub merge: bool,
    /// Only the inscribed disc is filled.
    pub clip_circle: bool,
    /// Drift phase; 0 leaves the sites where they are.
    pub phase: f32,
    /// A temporary extra site (the cursor), shown as the last active site.
    pub probe: Option<(f32, f32)>,
}

impl Default for Spec {
    fn default() -> Self {
        Self {
            metric: Metric::Euclidean,
            weighted: false,
            farthest: false,
            knn: 1,
            relax: 0,
            merge: true,
            clip_circle: false,
            phase: 0.0,
            probe: None,
        }
    }
}

impl Spec {
    /// Whether the exact polygon [`diagram`] can draw this (a plain nearest-site split).
    pub fn is_plain(&self) -> bool {
        self.metric == Metric::Euclidean
            && !self.weighted
            && !self.farthest
            && self.knn < 2
            && self.relax == 0
            && !self.clip_circle
            && self.phase == 0.0
            && self.probe.is_none()
    }
}

/// A computed field: `w`×`h` pixels, row 0 on top.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Raster {
    pub w: usize,
    pub h: usize,
    /// Owning active site per pixel, or [`NONE`].
    pub own: Vec<u32>,
    /// Distance from the pixel to its owner.
    pub dist: Vec<f32>,
    /// Winning category per pixel under k-NN voting, else [`NONE`].
    pub cat: Vec<u32>,
    /// Region id per pixel: borders sit where it changes.
    pub group: Vec<u32>,
    /// Where each active site ended up (after relaxing and drifting), then the probe if any.
    pub sites: Vec<(f32, f32)>,
    /// Neighbouring sites with the number of pixel edges they share.
    pub links: Vec<((usize, usize), usize)>,
    /// Typical spacing between sites, in pixels.
    pub spacing: f32,
}

impl Raster {
    /// The owner of the pixel at `(x, y)`, if any.
    pub fn owner_at(&self, x: f32, y: f32) -> Option<usize> {
        if x < 0.0 || y < 0.0 || x >= self.w as f32 || y >= self.h as f32 {
            return None;
        }
        let o = self.own[y as usize * self.w + x as usize];
        (o != NONE).then_some(o as usize)
    }

    /// Share of the filled area each active site owns, summing to 1.
    pub fn areas(&self) -> Vec<f32> {
        let mut n = vec![0usize; self.sites.len()];
        for &o in self.own.iter().filter(|&&o| o != NONE) {
            n[o as usize] += 1;
        }
        let total = n.iter().sum::<usize>().max(1) as f32;
        n.into_iter().map(|c| c as f32 / total).collect()
    }
}

struct Active {
    pos: (f32, f32),
    radius: f32,
}

fn assign(act: &[Active], w: usize, h: usize, spec: &Spec) -> (Vec<u32>, Vec<f32>) {
    let mut own = vec![NONE; w * h];
    let mut dist = vec![0.0; w * h];
    let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
    let rr = cx.min(cy);
    for py in 0..h {
        for px in 0..w {
            let (qx, qy) = (px as f32 + 0.5, py as f32 + 0.5);
            if spec.clip_circle && (qx - cx).powi(2) + (qy - cy).powi(2) > rr * rr {
                continue;
            }
            let mut best = (if spec.farthest { f32::MIN } else { f32::MAX }, 0usize);
            for (k, a) in act.iter().enumerate() {
                let (dx, dy) = ((qx - a.pos.0).abs(), (qy - a.pos.1).abs());
                if spec.farthest {
                    let d = spec.metric.dist(dx, dy);
                    if d > best.0 {
                        best = (d, k);
                    }
                } else {
                    let key = spec.metric.key(dx, dy, a.radius);
                    if key < best.0 {
                        best = (key, k);
                    }
                }
            }
            let a = &act[best.1];
            own[py * w + px] = best.1 as u32;
            dist[py * w + px] = spec.metric.dist((qx - a.pos.0).abs(), (qy - a.pos.1).abs());
        }
    }
    (own, dist)
}

/// The field of `sites` (in pixels of a `w`×`h` box) under `spec`.
pub fn raster(sites: &[Site], w: usize, h: usize, spec: &Spec) -> Raster {
    let ok: Vec<&Site> = sites
        .iter()
        .filter(|s| s.x.is_finite() && s.y.is_finite())
        .collect();
    if ok.is_empty() || w == 0 || h == 0 {
        return Raster {
            w,
            h,
            ..Raster::default()
        };
    }
    let spacing = ((w * h) as f32 / ok.len() as f32).sqrt();
    let wmax = ok
        .iter()
        .filter_map(|s| s.weight)
        .filter(|v| v.is_finite())
        .fold(0.0, f32::max);
    let mut act: Vec<Active> = ok
        .iter()
        .map(|s| Active {
            pos: (s.x, s.y),
            radius: match s.weight {
                Some(v) if spec.weighted && v.is_finite() && wmax > 0.0 => {
                    (v.max(0.0) / wmax).sqrt() * 0.6 * spacing
                }
                _ => 0.0,
            },
        })
        .collect();
    for _ in 0..spec.relax.min(30) {
        let (own, _) = assign(&act, w, h, spec);
        let mut sum = vec![(0.0, 0.0, 0usize); act.len()];
        for (p, &o) in own.iter().enumerate().filter(|(_, &o)| o != NONE) {
            let e = &mut sum[o as usize];
            e.0 += (p % w) as f32 + 0.5;
            e.1 += (p / w) as f32 + 0.5;
            e.2 += 1;
        }
        for (a, s) in act.iter_mut().zip(&sum) {
            if s.2 > 0 {
                a.pos = (s.0 / s.2 as f32, s.1 / s.2 as f32);
            }
        }
    }
    if spec.phase != 0.0 && spec.phase.is_finite() {
        for (k, a) in act.iter_mut().enumerate() {
            let (kf, amp) = (k as f32, 0.28 * spacing);
            a.pos.0 = (a.pos.0 + amp * (spec.phase + kf * 1.7).sin()).clamp(0.0, w as f32 - 1.0);
            a.pos.1 =
                (a.pos.1 + amp * (spec.phase * 0.8 + kf * 2.3).cos()).clamp(0.0, h as f32 - 1.0);
        }
    }
    let real = act.len();
    if let Some((x, y)) = spec.probe.filter(|(x, y)| x.is_finite() && y.is_finite()) {
        act.push(Active {
            pos: (x.clamp(0.0, w as f32 - 1.0), y.clamp(0.0, h as f32 - 1.0)),
            radius: 0.0,
        });
    }
    let (own, dist) = assign(&act, w, h, spec);
    let cats: Vec<Option<usize>> = (0..act.len())
        .map(|k| if k < real { ok[k].category } else { None })
        .collect();
    let ncat = cats.iter().flatten().max().map_or(0, |m| m + 1);
    let voting = spec.knn >= 2 && ncat > 0;
    let n = w * h;
    let mut cat = vec![NONE; n];
    let mut group = vec![NONE; n];
    for p in 0..n {
        let o = own[p];
        if o == NONE {
            continue;
        }
        let o = o as usize;
        if voting {
            let (qx, qy) = ((p % w) as f32 + 0.5, (p / w) as f32 + 0.5);
            let mut ds: Vec<(f32, usize)> = act
                .iter()
                .enumerate()
                .filter(|(k, _)| cats[*k].is_some())
                .map(|(k, a)| {
                    (
                        spec.metric.dist((qx - a.pos.0).abs(), (qy - a.pos.1).abs()),
                        k,
                    )
                })
                .collect();
            ds.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut votes = vec![0usize; ncat];
            for (_, k) in ds.iter().take(spec.knn) {
                if let Some(c) = cats[*k] {
                    votes[c] += 1;
                }
            }
            let top = votes.iter().copied().max().unwrap_or(0);
            // ties go to the nearest site's category
            let win = ds
                .iter()
                .filter_map(|(_, k)| cats[*k])
                .find(|c| votes[*c] == top)
                .unwrap_or(0);
            cat[p] = win as u32;
            group[p] = win as u32;
        } else {
            group[p] = match (spec.merge, cats[o]) {
                (true, Some(c)) => c as u32,
                _ => (ncat + o) as u32,
            };
        }
    }
    let mut adj: std::collections::BTreeMap<(usize, usize), usize> = Default::default();
    for p in 0..n {
        let a = own[p];
        if a == NONE {
            continue;
        }
        let right = (p % w + 1 < w).then(|| own[p + 1]);
        let down = (p / w + 1 < h).then(|| own[p + w]);
        for b in [right, down].into_iter().flatten() {
            if b != NONE && b != a {
                *adj.entry((a.min(b) as usize, a.max(b) as usize))
                    .or_default() += 1;
            }
        }
    }
    Raster {
        w,
        h,
        own,
        dist,
        cat,
        group,
        sites: act.iter().map(|a| a.pos).collect(),
        links: adj.into_iter().collect(),
        spacing,
    }
}

/// A pie from a Voronoi diagram: one weighted site per item on a ring, placed at its wedge's
/// middle, inside a disc. The wedges come out roughly in proportion to the values.
pub fn ring_sites(values: &[f32], w: f32, h: f32) -> Vec<Site> {
    let total: f32 = values.iter().filter(|v| v.is_finite() && **v > 0.0).sum();
    let (cx, cy, r) = (w / 2.0, h / 2.0, w.min(h) / 2.0);
    let mut acc = 0.0;
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let v = if v.is_finite() { v.max(0.0) } else { 0.0 };
            let mid = (acc + v / 2.0) / total.max(f32::EPSILON) * std::f32::consts::TAU
                - std::f32::consts::FRAC_PI_2;
            acc += v;
            Site {
                x: cx + 0.55 * r * mid.cos(),
                y: cy + 0.55 * r * mid.sin(),
                weight: Some(v),
                category: Some(i),
            }
        })
        .collect()
}

/// Mosaic sites: `n` points sampled from `density[row][col]` (row 0 on top) and relaxed `relax`
/// times toward the density-weighted centroids, so they crowd where the density is high.
/// Returns `(x, y, density there)` with `y` upward, in the grid's units. Deterministic per seed.
pub fn stipple(density: &[Vec<f32>], n: usize, relax: usize, seed: u64) -> Vec<(f32, f32, f32)> {
    let rows = density.len();
    let cols = density.iter().map(Vec::len).max().unwrap_or(0);
    let d = |r: usize, c: usize| {
        density
            .get(r)
            .and_then(|row| row.get(c))
            .copied()
            .filter(|v| v.is_finite() && *v > 0.0)
            .unwrap_or(0.0)
    };
    let total: f32 = (0..rows)
        .flat_map(|r| (0..cols).map(move |c| (r, c)))
        .map(|(r, c)| d(r, c))
        .sum();
    if rows == 0 || cols == 0 || n == 0 || total <= 0.0 {
        return Vec::new();
    }
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let mut rnd = move || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 40) as f32 / (1u64 << 24) as f32
    };
    let mut cdf = Vec::with_capacity(rows * cols);
    let mut acc = 0.0;
    for r in 0..rows {
        for c in 0..cols {
            acc += d(r, c);
            cdf.push(acc);
        }
    }
    let mut pos: Vec<(f32, f32)> = (0..n)
        .map(|_| {
            let t = rnd() * total;
            let i = cdf.partition_point(|v| *v < t).min(cdf.len() - 1);
            ((i % cols) as f32 + rnd(), (i / cols) as f32 + rnd())
        })
        .collect();
    for _ in 0..relax.min(60) {
        let mut sum = vec![(0.0, 0.0, 0.0); n];
        for r in 0..rows {
            for c in 0..cols {
                let wgt = d(r, c);
                if wgt == 0.0 {
                    continue;
                }
                let (qx, qy) = (c as f32 + 0.5, r as f32 + 0.5);
                let k = pos
                    .iter()
                    .enumerate()
                    .min_by(|a, b| {
                        let da = (a.1 .0 - qx).powi(2) + (a.1 .1 - qy).powi(2);
                        let db = (b.1 .0 - qx).powi(2) + (b.1 .1 - qy).powi(2);
                        da.total_cmp(&db)
                    })
                    .map_or(0, |x| x.0);
                sum[k].0 += qx * wgt;
                sum[k].1 += qy * wgt;
                sum[k].2 += wgt;
            }
        }
        for (p, s) in pos.iter_mut().zip(&sum) {
            if s.2 > 0.0 {
                *p = (s.0 / s.2, s.1 / s.2);
            }
        }
    }
    pos.into_iter()
        .map(|(x, y)| {
            let (c, r) = ((x as usize).min(cols - 1), (y as usize).min(rows - 1));
            (x, rows as f32 - y, d(r, c))
        })
        .collect()
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

    fn site(x: f32, y: f32) -> Site {
        Site {
            x,
            y,
            ..Site::default()
        }
    }

    fn share(r: &Raster, k: usize) -> f32 {
        r.areas()[k]
    }

    #[test]
    fn two_sites_split_the_raster_in_half() {
        let r = raster(
            &[site(20.0, 20.0), site(60.0, 20.0)],
            80,
            40,
            &Spec::default(),
        );
        assert!((share(&r, 0) - 0.5).abs() < 0.02);
    }

    #[test]
    fn a_heavier_site_owns_more_when_weighted() {
        let mk = |wa, wb| {
            [
                Site {
                    weight: Some(wa),
                    ..site(20.0, 20.0)
                },
                Site {
                    weight: Some(wb),
                    ..site(60.0, 20.0)
                },
            ]
        };
        let spec = Spec {
            weighted: true,
            ..Spec::default()
        };
        let r = raster(&mk(9.0, 1.0), 80, 40, &spec);
        assert!(share(&r, 0) > share(&r, 1) + 0.1, "{:?}", r.areas());
        let even = raster(&mk(9.0, 1.0), 80, 40, &Spec::default());
        assert!(
            (share(&even, 0) - 0.5).abs() < 0.02,
            "weights are ignored unless asked for"
        );
    }

    #[test]
    fn every_metric_fills_the_box() {
        for m in [
            Metric::Euclidean,
            Metric::Manhattan,
            Metric::Chebyshev,
            Metric::Minkowski(3.0),
        ] {
            let r = raster(
                &[site(10.0, 10.0), site(50.0, 30.0)],
                60,
                40,
                &Spec {
                    metric: m,
                    ..Spec::default()
                },
            );
            assert!(r.own.iter().all(|&o| o != NONE), "{m:?}");
        }
        assert_eq!(
            Metric::Euclidean.next().next().next().next(),
            Metric::Euclidean
        );
    }

    #[test]
    fn minkowski_two_is_euclidean() {
        let s = [site(10.0, 10.0), site(50.0, 30.0), site(20.0, 35.0)];
        let a = raster(&s, 60, 40, &Spec::default());
        let b = raster(
            &s,
            60,
            40,
            &Spec {
                metric: Metric::Minkowski(2.0),
                ..Spec::default()
            },
        );
        let same = a.own.iter().zip(&b.own).filter(|(x, y)| x == y).count();
        assert!(same as f32 / a.own.len() as f32 > 0.99);
    }

    #[test]
    fn farthest_flips_ownership() {
        let s = [site(10.0, 20.0), site(70.0, 20.0)];
        let r = raster(
            &s,
            80,
            40,
            &Spec {
                farthest: true,
                ..Spec::default()
            },
        );
        assert_eq!(r.owner_at(5.0, 20.0), Some(1));
        assert_eq!(r.owner_at(75.0, 20.0), Some(0));
    }

    #[test]
    fn same_category_cells_merge_into_one_group() {
        let c = |x, k| Site {
            category: Some(k),
            ..site(x, 20.0)
        };
        let s = [c(10.0, 0), c(30.0, 0), c(70.0, 1)];
        let r = raster(&s, 80, 40, &Spec::default());
        assert_eq!(r.group[20 * 80 + 5], r.group[20 * 80 + 35]);
        assert_ne!(r.group[20 * 80 + 5], r.group[20 * 80 + 75]);
        let apart = raster(
            &s,
            80,
            40,
            &Spec {
                merge: false,
                ..Spec::default()
            },
        );
        assert_ne!(apart.group[20 * 80 + 5], apart.group[20 * 80 + 35]);
    }

    #[test]
    fn knn_votes_by_majority() {
        // a lone B among three As: its own pixel flips to A once three neighbours vote
        let c = |x, y, k| Site {
            category: Some(k),
            ..site(x, y)
        };
        let s = [
            c(10.0, 10.0, 0),
            c(20.0, 12.0, 0),
            c(14.0, 22.0, 0),
            c(16.0, 14.0, 1),
        ];
        let one = raster(&s, 40, 30, &Spec::default());
        assert_eq!(one.cat[14 * 40 + 16], NONE);
        let three = raster(
            &s,
            40,
            30,
            &Spec {
                knn: 3,
                ..Spec::default()
            },
        );
        assert_eq!(three.cat[14 * 40 + 16], 0);
    }

    #[test]
    fn relaxing_evens_out_cells() {
        let s = [
            site(5.0, 5.0),
            site(8.0, 6.0),
            site(40.0, 30.0),
            site(70.0, 35.0),
        ];
        let spread = |r: &Raster| {
            let a = r.areas();
            a.iter().cloned().fold(0.0, f32::max) - a.iter().cloned().fold(1.0, f32::min)
        };
        let before = raster(&s, 80, 40, &Spec::default());
        let after = raster(
            &s,
            80,
            40,
            &Spec {
                relax: 12,
                ..Spec::default()
            },
        );
        assert!(spread(&after) < spread(&before));
    }

    #[test]
    fn a_probe_steals_area_and_the_disc_clips_the_corners() {
        let s = [site(20.0, 20.0), site(60.0, 20.0)];
        let r = raster(
            &s,
            80,
            40,
            &Spec {
                probe: Some((40.0, 5.0)),
                ..Spec::default()
            },
        );
        assert_eq!(r.sites.len(), 3);
        assert!(share(&r, 2) > 0.05);
        let d = raster(
            &s,
            80,
            40,
            &Spec {
                clip_circle: true,
                ..Spec::default()
            },
        );
        assert_eq!(d.own[0], NONE);
        assert_ne!(d.own[20 * 80 + 40], NONE);
    }

    #[test]
    fn neighbours_are_found_and_the_phase_moves_sites() {
        let r = raster(
            &[site(10.0, 20.0), site(40.0, 20.0), site(70.0, 20.0)],
            80,
            40,
            &Spec::default(),
        );
        let pairs: Vec<_> = r.links.iter().map(|l| l.0).collect();
        assert_eq!(pairs, [(0, 1), (1, 2)]);
        let moved = raster(
            &[site(10.0, 20.0)],
            80,
            40,
            &Spec {
                phase: 1.0,
                ..Spec::default()
            },
        );
        assert_ne!(moved.sites[0], (10.0, 20.0));
    }

    #[test]
    fn a_ring_follows_the_values() {
        let sites = ring_sites(&[3.0, 1.0, 1.0], 100.0, 100.0);
        let r = raster(
            &sites,
            100,
            100,
            &Spec {
                weighted: true,
                clip_circle: true,
                ..Spec::default()
            },
        );
        let a = r.areas();
        assert!(a[0] > a[1] && a[0] > a[2], "{a:?}");
    }

    #[test]
    fn stipple_is_deterministic_and_follows_density() {
        let mut d = vec![vec![0.0; 10]; 10];
        for row in d.iter_mut() {
            for v in row.iter_mut().skip(6) {
                *v = 1.0;
            }
        }
        let a = stipple(&d, 40, 3, 7);
        assert_eq!(a, stipple(&d, 40, 3, 7));
        assert_eq!(a.len(), 40);
        assert!(a.iter().all(|p| p.0 >= 5.9), "all dots in the dense side");
        assert!(stipple(&[], 5, 1, 1).is_empty());
    }

    #[test]
    fn degenerate_input_does_not_panic() {
        let r = raster(&[site(f32::NAN, 1.0)], 10, 10, &Spec::default());
        assert!(r.own.is_empty());
        let _ = raster(&[site(1.0, 1.0)], 0, 0, &Spec::default());
        assert!(ring_sites(&[], 10.0, 10.0).is_empty());
    }
}
