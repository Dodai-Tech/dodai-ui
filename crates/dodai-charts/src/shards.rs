//! Shard map layout: a convex outline (circle, hexagon, square) cut into one convex cell per
//! item, each with an area proportional to its value, items of one group sharing a region.
//!
//! Cells form a weighted Voronoi (power) diagram: sites start at the centres of a recursive
//! area-proportional cut ([`split`]), then move to their cells' centroids while their weights
//! grow or shrink until every area matches its value ([`voronoi`]). Pure and deterministic.

/// A polygon, as its corners in order.
pub type Poly = Vec<(f32, f32)>;

/// The outline a shard map fills.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Outline {
    /// A circle (a 96-gon).
    #[default]
    Circle,
    /// A hexagon, pointy side up.
    Hexagon,
    /// A square.
    Square,
}

impl Outline {
    /// The outline round `(0, 0)` with radius 1 (the square's half side is 1).
    pub fn polygon(self) -> Poly {
        let ring = |n: usize, turn0: f32| {
            (0..n)
                .map(|i| {
                    let a = (turn0 + i as f32 / n as f32) * std::f32::consts::TAU;
                    (a.sin(), -a.cos())
                })
                .collect()
        };
        match self {
            Outline::Circle => ring(96, 0.0),
            Outline::Hexagon => ring(6, 0.0),
            Outline::Square => vec![(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)],
        }
    }
}

/// Area of `p` (positive whatever the winding).
pub fn area(p: &[(f32, f32)]) -> f32 {
    let n = p.len();
    if n < 3 {
        return 0.0;
    }
    let s: f32 = (0..n)
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % n]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum();
    (s / 2.0).abs()
}

/// The centroid of `p` (its vertex mean when it has no area).
pub fn centroid(p: &[(f32, f32)]) -> (f32, f32) {
    let n = p.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    let (mut cx, mut cy, mut a2) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let (a, b) = (p[i], p[(i + 1) % n]);
        let k = a.0 * b.1 - b.0 * a.1;
        a2 += k;
        cx += (a.0 + b.0) * k;
        cy += (a.1 + b.1) * k;
    }
    if a2.abs() < 1e-9 {
        let s = p.iter().fold((0.0, 0.0), |s, q| (s.0 + q.0, s.1 + q.1));
        return (s.0 / n as f32, s.1 / n as f32);
    }
    (cx / (3.0 * a2), cy / (3.0 * a2))
}

/// The part of convex `p` where `n · q <= d`.
pub fn clip(p: &[(f32, f32)], n: (f32, f32), d: f32) -> Poly {
    let side = |q: (f32, f32)| n.0 * q.0 + n.1 * q.1 - d;
    let mut out = Vec::with_capacity(p.len() + 2);
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        let (sa, sb) = (side(a), side(b));
        if sa <= 0.0 {
            out.push(a);
        }
        if (sa < 0.0) != (sb < 0.0) && sa != sb {
            let t = sa / (sa - sb);
            out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        }
    }
    out
}

/// Whether `q` is inside convex `p` (either winding).
pub fn contains(p: &[(f32, f32)], q: (f32, f32)) -> bool {
    if p.len() < 3 {
        return false;
    }
    let mut sign = 0.0f32;
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        let c = (b.0 - a.0) * (q.1 - a.1) - (b.1 - a.1) * (q.0 - a.0);
        if c != 0.0 {
            if sign != 0.0 && c.signum() != sign {
                return false;
            }
            sign = c.signum();
        }
    }
    true
}

/// A small repeatable turn for cut `seed`, in radians (`±0.3`).
fn wobble(seed: u32) -> f32 {
    let mut x = seed.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    ((x & 0xFFFF) as f32 / 65_535.0 - 0.5) * 0.6
}

/// The unit normal across which `p` is widest (of 12 directions), turned by `wobble(seed)`.
fn widest(p: &[(f32, f32)], seed: u32) -> (f32, f32) {
    let extent = |a: f32| {
        let n = (a.cos(), a.sin());
        let (lo, hi) = p
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(l, h), q| {
                let d = n.0 * q.0 + n.1 * q.1;
                (l.min(d), h.max(d))
            });
        hi - lo
    };
    let best = (0..12)
        .map(|i| i as f32 * std::f32::consts::PI / 12.0)
        .max_by(|a, b| extent(*a).total_cmp(&extent(*b)))
        .unwrap_or(0.0)
        + wobble(seed);
    (best.cos(), best.sin())
}

/// Cuts convex `p` along normal `n` so the `n · q <= d` side holds `share` of its area.
fn cut(p: &[(f32, f32)], n: (f32, f32), share: f32) -> (Poly, Poly) {
    let total = area(p);
    let (mut lo, mut hi) = p
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(l, h), q| {
            let d = n.0 * q.0 + n.1 * q.1;
            (l.min(d), h.max(d))
        });
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if area(&clip(p, n, mid)) < share * total {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let d = (lo + hi) / 2.0;
    (clip(p, n, d), clip(p, (-n.0, -n.1), -d))
}

/// Splits convex `p` into one shard per weight (input order); weights that are not positive
/// and finite get an empty shard.
pub fn split(p: &[(f32, f32)], weights: &[f32], seed: u32) -> Vec<Poly> {
    let mut out = vec![Vec::new(); weights.len()];
    let live: Vec<usize> = (0..weights.len())
        .filter(|i| weights[*i].is_finite() && weights[*i] > 0.0)
        .collect();
    deal(p, weights, live, seed, &mut out);
    out
}

fn deal(p: &[(f32, f32)], w: &[f32], mut idx: Vec<usize>, seed: u32, out: &mut [Poly]) {
    match idx.len() {
        0 => return,
        1 => {
            out[idx[0]] = p.to_vec();
            return;
        }
        _ => {}
    }
    idx.sort_by(|a, b| w[*b].total_cmp(&w[*a]));
    let (mut left, mut right) = (Vec::new(), Vec::new());
    let (mut wl, mut wr) = (0.0f32, 0.0f32);
    for i in idx {
        if wl <= wr {
            wl += w[i];
            left.push(i);
        } else {
            wr += w[i];
            right.push(i);
        }
    }
    let (a, b) = cut(p, widest(p, seed), wl / (wl + wr));
    deal(&a, w, left, seed.wrapping_mul(2).wrapping_add(1), out);
    deal(&b, w, right, seed.wrapping_mul(2).wrapping_add(2), out);
}

/// The power diagram of `sites` with weights `w`, clipped to convex `p`: cell `i` holds the
/// points where `|q - s_i|² - w_i` is least.
pub fn power_cells(p: &[(f32, f32)], sites: &[(f32, f32)], w: &[f32]) -> Vec<Poly> {
    sites
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let mut cell = p.to_vec();
            for (j, t) in sites.iter().enumerate() {
                if j == i || cell.len() < 3 {
                    continue;
                }
                let n = (2.0 * (t.0 - s.0), 2.0 * (t.1 - s.1));
                let d = (t.0 * t.0 + t.1 * t.1) - (s.0 * s.0 + s.1 * s.1) - w[j] + w[i];
                cell = clip(&cell, n, d);
            }
            if cell.len() < 3 {
                Vec::new()
            } else {
                cell
            }
        })
        .collect()
}

/// Splits convex `p` into one Voronoi-like cell per weight (input order), each cell's area
/// proportional to its weight (within about 1 %); weights that are not positive and finite get
/// an empty cell.
pub fn voronoi(p: &[(f32, f32)], weights: &[f32], seed: u32) -> Vec<Poly> {
    let mut out = vec![Vec::new(); weights.len()];
    let live: Vec<usize> = (0..weights.len())
        .filter(|i| weights[*i].is_finite() && weights[*i] > 0.0)
        .collect();
    if live.len() < 2 {
        if let Some(&i) = live.first() {
            out[i] = p.to_vec();
        }
        return out;
    }
    let total = area(p);
    let sum: f32 = live.iter().map(|i| weights[*i]).sum();
    let target: Vec<f32> = live.iter().map(|i| total * weights[*i] / sum).collect();
    let start = split(p, weights, seed);
    let mut sites: Vec<(f32, f32)> = live.iter().map(|i| centroid(&start[*i])).collect();
    let mut w: Vec<f32> = vec![total / live.len() as f32 / 2.0; live.len()];
    let floor = total * 1e-6;
    let cap = |sites: &[(f32, f32)], w: &mut [f32]| {
        for i in 0..sites.len() {
            let near = (0..sites.len())
                .filter(|j| *j != i)
                .map(|j| (sites[i].0 - sites[j].0).powi(2) + (sites[i].1 - sites[j].1).powi(2))
                .fold(f32::INFINITY, f32::min);
            w[i] = w[i].min(near).max(floor);
        }
    };
    let mut cells = power_cells(p, &sites, &w);
    for _ in 0..160 {
        // Positions: each site to its cell's centroid.
        for (s, c) in sites.iter_mut().zip(&cells) {
            if c.len() > 2 {
                *s = centroid(c);
            }
        }
        cap(&sites, &mut w);
        cells = power_cells(p, &sites, &w);
        // Weights: grow the cells that are too small, shrink the big ones.
        for k in 0..w.len() {
            let a = area(&cells[k]);
            let ratio = if a > 0.0 { target[k] / a } else { 2.0 };
            w[k] = (w[k] * ratio.clamp(0.5, 2.0)).max(floor);
        }
        cap(&sites, &mut w);
        cells = power_cells(p, &sites, &w);
        let err = cells
            .iter()
            .zip(&target)
            .map(|(c, t)| (area(c) - t).abs())
            .sum::<f32>()
            / total;
        if err < 0.01 {
            break;
        }
    }
    for (k, i) in live.into_iter().enumerate() {
        out[i] = std::mem::take(&mut cells[k]);
    }
    out
}

/// One laid-out shard: item `item` of group `group`.
#[derive(Debug, Clone, PartialEq)]
pub struct Shard {
    /// Index of the group.
    pub group: usize,
    /// Index of the item within its group.
    pub item: usize,
    /// Its polygon, in the outline's coordinates (empty for items without a positive value).
    pub poly: Poly,
}

/// Lays `groups` (each a list of item values) out in `outline`: first one region per group,
/// then one shard per item inside it. Also returns the group regions.
pub fn layout(groups: &[Vec<f32>], outline: &[(f32, f32)]) -> (Vec<Shard>, Vec<Poly>) {
    let totals: Vec<f32> = groups
        .iter()
        .map(|g| g.iter().filter(|v| v.is_finite() && **v > 0.0).sum())
        .collect();
    let regions = voronoi(outline, &totals, 7);
    let mut shards = Vec::new();
    for (gi, (g, region)) in groups.iter().zip(&regions).enumerate() {
        for (ii, poly) in voronoi(region, g, 101 + gi as u32 * 31)
            .into_iter()
            .enumerate()
        {
            shards.push(Shard {
                group: gi,
                item: ii,
                poly,
            });
        }
    }
    (shards, regions)
}

/// The shard under `q`, if any.
pub fn hit(shards: &[Shard], q: (f32, f32)) -> Option<usize> {
    shards.iter().position(|s| contains(&s.poly, q))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn shard_areas_follow_the_values() {
        for outline in [Outline::Circle, Outline::Hexagon, Outline::Square] {
            let o = outline.polygon();
            let total = area(&o);
            let w = [5.0, 3.0, 1.0, 1.0, 0.5, 9.0];
            let sum: f32 = w.iter().sum();
            let cells = split(&o, &w, 1);
            for (c, v) in cells.iter().zip(w) {
                assert!(
                    close(area(c), total * v / sum, total * 0.002),
                    "{outline:?}"
                );
            }
        }
    }

    #[test]
    fn groups_hold_their_items_and_shards_tile_the_outline() {
        let o = Outline::Hexagon.polygon();
        let (shards, regions) = layout(&[vec![4.0, 2.0], vec![3.0], vec![1.0, 1.0, 1.0]], &o);
        assert_eq!(shards.len(), 6);
        let sum: f32 = shards.iter().map(|s| area(&s.poly)).sum();
        assert!(close(sum, area(&o), 0.01));
        for s in &shards {
            let c = centroid(&s.poly);
            assert!(contains(&regions[s.group], c));
            assert_eq!(
                hit(&shards, c).map(|k| (shards[k].group, shards[k].item)),
                Some((s.group, s.item))
            );
        }
    }

    #[test]
    fn voronoi_cells_follow_the_values_and_tile_the_outline() {
        let o = Outline::Circle.polygon();
        let total = area(&o);
        let w = [
            940.0, 485.0, 380.0, 380.0, 345.0, 236.0, 123.0, 86.0, 50.0, 22.0, 12.0,
        ];
        let sum: f32 = w.iter().sum();
        let cells = voronoi(&o, &w, 1);
        let mut err = 0.0;
        for (c, v) in cells.iter().zip(w) {
            assert!(c.len() > 2);
            err += (area(c) - total * v / sum).abs();
        }
        assert!(err / total < 0.03, "{}", err / total);
        let covered: f32 = cells.iter().map(|c| area(c)).sum();
        assert!(close(covered, total, total * 0.001));
    }

    #[test]
    fn hostile_values_get_empty_shards_and_nothing_panics() {
        let o = Outline::Circle.polygon();
        let cells = split(&o, &[f32::NAN, -1.0, 0.0, f32::INFINITY, 2.0], 3);
        assert!(cells[..4].iter().all(|c| c.is_empty()));
        assert!(close(area(&cells[4]), area(&o), 1e-4));
        let (shards, _) = layout(&[vec![], vec![f32::NAN]], &o);
        assert!(shards.iter().all(|s| s.poly.is_empty()));
        assert_eq!(hit(&shards, (0.0, 0.0)), None);
        assert!(split(&o, &[], 0).is_empty());
    }

    #[test]
    fn clip_and_contains_agree() {
        let sq = Outline::Square.polygon();
        let left = clip(&sq, (1.0, 0.0), 0.0);
        assert!(close(area(&left), 2.0, 1e-5));
        assert!(contains(&left, (-0.5, 0.0)) && !contains(&left, (0.5, 0.0)));
        assert_eq!(centroid(&sq), (0.0, 0.0));
    }
}
