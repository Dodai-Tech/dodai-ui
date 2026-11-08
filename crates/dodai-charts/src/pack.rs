//! Packing layouts: circles of different sizes along an axis (a sized beeswarm), circles
//! inside circles (circle packing) and proportional squares. Pure.

use crate::hierarchy::Node;

/// The sideways offset of each dot so that dots of radius `rs[i]` at `(xs[i], offset[i])` do
/// not overlap, each as close to the axis as possible. Big dots are placed first (ties in
/// order of position), so small ones settle into the gaps. Non-finite positions or radii
/// get offset 0. Pure.
pub fn swarm_sized(xs: &[f32], rs: &[f32]) -> Vec<f32> {
    let n = xs.len().min(rs.len());
    let mut out = vec![0.0; xs.len()];
    let r = |i: usize| {
        if rs[i].is_finite() {
            rs[i].max(0.0)
        } else {
            0.0
        }
    };
    let mut order: Vec<usize> = (0..n)
        .filter(|i| xs[*i].is_finite() && r(*i) > 0.0)
        .collect();
    order.sort_by(|a, b| r(*b).total_cmp(&r(*a)).then(xs[*a].total_cmp(&xs[*b])));
    let mut placed: Vec<(f32, f32, f32)> = Vec::new();
    for i in order {
        let (x, ri) = (xs[i], r(i));
        let near: Vec<(f32, f32, f32)> = placed
            .iter()
            .copied()
            .filter(|(px, _, pr)| (x - px).abs() < ri + pr)
            .collect();
        let mut candidates = vec![0.0f32];
        for (px, py, pr) in &near {
            let d = ri + pr;
            let dy = (d * d - (x - px).powi(2)).max(0.0).sqrt();
            candidates.push(py + dy);
            candidates.push(py - dy);
        }
        candidates.sort_by(|a, b| a.abs().total_cmp(&b.abs()));
        let y = candidates
            .into_iter()
            .find(|y| {
                near.iter().all(|(px, py, pr)| {
                    (x - px).powi(2) + (y - py).powi(2) >= (ri + pr).powi(2) - 1e-3
                })
            })
            .unwrap_or(0.0);
        out[i] = y;
        placed.push((x, y, ri));
    }
    out
}

/// The height of each circle's centre above a floor so that circles of radius `rs[i]` at
/// `xs[i]` pile up without overlapping: big ones first, each resting as low as it can (on the
/// floor or on circles already placed), so small ones settle into the gaps above. Every
/// height is at least the circle's radius. Non-finite positions or radii get height 0. Pure.
pub fn pile_up(xs: &[f32], rs: &[f32]) -> Vec<f32> {
    let n = xs.len().min(rs.len());
    let mut out = vec![0.0; xs.len()];
    let r = |i: usize| {
        if rs[i].is_finite() {
            rs[i].max(0.0)
        } else {
            0.0
        }
    };
    let mut order: Vec<usize> = (0..n)
        .filter(|i| xs[*i].is_finite() && r(*i) > 0.0)
        .collect();
    order.sort_by(|a, b| r(*b).total_cmp(&r(*a)).then(xs[*a].total_cmp(&xs[*b])));
    let mut placed: Vec<(f32, f32, f32)> = Vec::new();
    for i in order {
        let (x, ri) = (xs[i], r(i));
        let near: Vec<(f32, f32, f32)> = placed
            .iter()
            .copied()
            .filter(|(px, _, pr)| (x - px).abs() < ri + pr)
            .collect();
        let mut candidates = vec![ri];
        for (px, py, pr) in &near {
            let d = ri + pr;
            let dy = (d * d - (x - px).powi(2)).max(0.0).sqrt();
            candidates.extend([py + dy, py - dy].into_iter().filter(|y| *y >= ri));
        }
        candidates.sort_by(f32::total_cmp);
        let free = |y: f32| {
            near.iter()
                .all(|(px, py, pr)| (x - px).powi(2) + (y - py).powi(2) >= (ri + pr).powi(2) - 1e-3)
        };
        // The top of the pile always fits, so this finds a height.
        let y = candidates.into_iter().find(|y| free(*y)).unwrap_or(ri);
        out[i] = y;
        placed.push((x, y, ri));
    }
    out
}

/// Centres for circles of radii `rs`, packed tightly around the origin without overlapping.
/// The largest goes in the middle; each next one takes the free spot touching two placed
/// circles that lies closest to the origin. Non-finite or negative radii count as 0. Meant
/// for up to a few hundred circles. Pure.
pub fn pack_siblings(rs: &[f32]) -> Vec<(f32, f32)> {
    let r = |i: usize| {
        if rs[i].is_finite() {
            rs[i].max(0.0)
        } else {
            0.0
        }
    };
    let mut order: Vec<usize> = (0..rs.len()).collect();
    order.sort_by(|a, b| r(*b).total_cmp(&r(*a)));
    let mut out = vec![(0.0, 0.0); rs.len()];
    let mut placed: Vec<(f32, f32, f32)> = Vec::new();
    for i in order {
        let ri = r(i);
        let at = match placed.len() {
            0 => (0.0, 0.0),
            1 => (placed[0].2 + ri, 0.0),
            _ => {
                let free = |(x, y): (f32, f32)| {
                    placed.iter().all(|(px, py, pr)| {
                        (x - px).powi(2) + (y - py).powi(2) >= (ri + pr).powi(2) * 0.9999 - 1e-6
                    })
                };
                let mut best: Option<((f32, f32), f32)> = None;
                for (a, pa) in placed.iter().enumerate() {
                    for pb in &placed[a + 1..] {
                        for c in touching(*pa, *pb, ri) {
                            let d = c.0 * c.0 + c.1 * c.1;
                            if best.is_none_or(|(_, bd)| d < bd) && free(c) {
                                best = Some((c, d));
                            }
                        }
                    }
                }
                // Always found for sane input; fall back to the outside edge.
                best.map_or_else(
                    || {
                        let far = placed
                            .iter()
                            .map(|(x, y, pr)| (x * x + y * y).sqrt() + pr)
                            .fold(0.0, f32::max);
                        (far + ri, 0.0)
                    },
                    |(c, _)| c,
                )
            }
        };
        out[i] = at;
        placed.push((at.0, at.1, ri));
    }
    out
}

/// The (up to two) centres of a circle of radius `r` touching both circles `a` and `b`.
fn touching(a: (f32, f32, f32), b: (f32, f32, f32), r: f32) -> Vec<(f32, f32)> {
    let (da, db) = (a.2 + r, b.2 + r);
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let d = (dx * dx + dy * dy).sqrt();
    if d <= 1e-6 || d > da + db || d < (da - db).abs() {
        return Vec::new();
    }
    let along = (da * da - db * db + d * d) / (2.0 * d);
    let h = (da * da - along * along).max(0.0).sqrt();
    let (mx, my) = (a.0 + dx * along / d, a.1 + dy * along / d);
    let (ox, oy) = (-dy * h / d, dx * h / d);
    vec![(mx + ox, my + oy), (mx - ox, my - oy)]
}

/// A circle `(x, y, r)` holding every circle `(x, y, r)` of `cs`: close to the smallest such
/// circle (refined from the bounding box towards the farthest circle). Pure.
pub fn enclose(cs: &[(f32, f32, f32)]) -> (f32, f32, f32) {
    if cs.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let lo = |f: fn(&(f32, f32, f32)) -> f32| cs.iter().map(f).fold(f32::INFINITY, f32::min);
    let hi = |f: fn(&(f32, f32, f32)) -> f32| cs.iter().map(f).fold(f32::NEG_INFINITY, f32::max);
    let (mut cx, mut cy) = (
        (lo(|c| c.0 - c.2) + hi(|c| c.0 + c.2)) / 2.0,
        (lo(|c| c.1 - c.2) + hi(|c| c.1 + c.2)) / 2.0,
    );
    let reach = |cx: f32, cy: f32, c: &(f32, f32, f32)| {
        ((c.0 - cx).powi(2) + (c.1 - cy).powi(2)).sqrt() + c.2
    };
    for k in 1..200 {
        let far = cs
            .iter()
            .max_by(|a, b| reach(cx, cy, a).total_cmp(&reach(cx, cy, b)))
            .copied()
            .unwrap_or((cx, cy, 0.0));
        let d = ((far.0 - cx).powi(2) + (far.1 - cy).powi(2)).sqrt();
        if d <= 1e-6 {
            break;
        }
        // The farthest point of that circle from the centre, approached by a shrinking step.
        let (px, py) = (
            far.0 + (far.0 - cx) / d * far.2,
            far.1 + (far.1 - cy) / d * far.2,
        );
        let step = 1.0 / (k as f32 + 1.0);
        cx += (px - cx) * step * 0.5;
        cy += (py - cy) * step * 0.5;
    }
    let r = cs.iter().map(|c| reach(cx, cy, c)).fold(0.0, f32::max);
    (cx, cy, r)
}

/// One node of a circle packing.
#[derive(Clone, Debug, PartialEq)]
pub struct Disc {
    /// 0 for the root's children, 1 for theirs, and so on.
    pub depth: usize,
    /// Centre and radius in the unit square (the root fills the circle of radius 0.5 at the
    /// centre).
    pub x: f32,
    /// See `x`.
    pub y: f32,
    /// See `x`.
    pub r: f32,
    /// The node's total.
    pub value: f32,
    /// The node's label.
    pub label: String,
    /// Which of the root's children this disc descends from (for colour).
    pub branch: usize,
    /// Index of the parent disc, `None` at depth 0.
    pub parent: Option<usize>,
    /// Whether the node is a leaf.
    pub leaf: bool,
}

/// Every node below `root` as a circle: leaves have area proportional to their value, each
/// branch is a circle enclosing its children with a small margin, siblings are packed
/// together. Scaled so the whole fits the unit square. Parents come before children;
/// zero-sized nodes are left out. Pure.
pub fn circle_pack(root: &Node) -> Vec<Disc> {
    /// A packed subtree in local units: its radius and children at offsets from its centre.
    struct Local {
        r: f32,
        kids: Vec<(f32, f32, usize, Local)>,
    }
    fn local(n: &Node) -> Local {
        let live: Vec<(usize, &Node)> = n
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| c.total() > 0.0)
            .collect();
        if live.is_empty() {
            return Local {
                r: n.total().sqrt(),
                kids: Vec::new(),
            };
        }
        let subs: Vec<Local> = live.iter().map(|(_, c)| local(c)).collect();
        let big = subs.iter().map(|s| s.r).fold(0.0, f32::max);
        // A hairline gap between siblings, then a margin inside the parent's edge.
        let gap = big * 0.04;
        let rs: Vec<f32> = subs.iter().map(|s| s.r + gap).collect();
        let at = pack_siblings(&rs);
        let ring: Vec<(f32, f32, f32)> = at.iter().zip(&rs).map(|(p, r)| (p.0, p.1, *r)).collect();
        let (cx, cy, r) = enclose(&ring);
        Local {
            r: r + big * 0.06,
            kids: live
                .iter()
                .zip(at)
                .zip(subs)
                .map(|(((i, _), p), s)| (p.0 - cx, p.1 - cy, *i, s))
                .collect(),
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn emit(
        n: &Node,
        l: &Local,
        (x, y, k): (f32, f32, f32),
        depth: usize,
        branch: Option<usize>,
        parent: Option<usize>,
        out: &mut Vec<Disc>,
    ) {
        for (dx, dy, i, sub) in &l.kids {
            let c = &n.children[*i];
            let b = branch.unwrap_or(*i);
            let (cx, cy) = (x + dx * k, y + dy * k);
            out.push(Disc {
                depth,
                x: cx,
                y: cy,
                r: sub.r * k,
                value: c.total(),
                label: c.label.clone(),
                branch: b,
                parent,
                leaf: c.children.is_empty(),
            });
            let me = out.len() - 1;
            emit(c, sub, (cx, cy, k), depth + 1, Some(b), Some(me), out);
        }
    }
    let l = local(root);
    let mut out = Vec::new();
    if l.r > 0.0 && l.r.is_finite() {
        emit(root, &l, (0.5, 0.5, 0.5 / l.r), 0, None, None, &mut out);
    }
    out
}

/// A placed square: top-left corner and side, in units of the packing width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Square {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Side length.
    pub side: f32,
}

/// Proportional squares: one square per value (area proportional to it), largest first,
/// packed bottom-left into a box of width 1 and height `aspect` (height / width), as large as
/// fits. Non-finite or negative values get a zero square at the origin. Pure.
pub fn square_pack(values: &[f32], aspect: f32) -> Vec<Square> {
    let v = |i: usize| {
        if values[i].is_finite() {
            values[i].max(0.0)
        } else {
            0.0
        }
    };
    let total: f32 = (0..values.len()).map(v).sum();
    let aspect = if aspect.is_finite() {
        aspect.clamp(0.05, 20.0)
    } else {
        1.0
    };
    let empty = vec![
        Square {
            x: 0.0,
            y: 0.0,
            side: 0.0
        };
        values.len()
    ];
    if total <= 0.0 {
        return empty;
    }
    let sides = |k: f32| -> Vec<f32> {
        (0..values.len())
            .map(|i| (v(i) / total).sqrt() * k)
            .collect()
    };
    // Scale k: the packed squares cover k² of area; at most the whole box.
    let (mut lo, mut hi) = (0.0f32, aspect.sqrt().min(1.0) * 1.0001);
    let mut best = empty;
    for _ in 0..28 {
        let k = (lo + hi) / 2.0;
        match skyline(&sides(k)) {
            Some((sq, h)) if h <= aspect + 1e-5 => {
                best = sq;
                lo = k;
            }
            _ => hi = k,
        }
    }
    best
}

/// Bottom-left skyline packing of squares in a strip of width 1 (y grows downwards from 0),
/// largest first. Returns the squares and the height used, or `None` if one is wider than 1.
fn skyline(sides: &[f32]) -> Option<(Vec<Square>, f32)> {
    let mut order: Vec<usize> = (0..sides.len()).collect();
    order.sort_by(|a, b| sides[*b].total_cmp(&sides[*a]));
    // Skyline segments (x, width, top), left to right, covering 0..1.
    let mut sky: Vec<(f32, f32, f32)> = vec![(0.0, 1.0, 0.0)];
    let mut out = vec![
        Square {
            x: 0.0,
            y: 0.0,
            side: 0.0
        };
        sides.len()
    ];
    let mut height = 0.0f32;
    for i in order {
        let s = sides[i];
        if s <= 0.0 {
            continue;
        }
        if s > 1.0 + 1e-6 {
            return None;
        }
        let mut best: Option<(f32, f32)> = None;
        for seg in &sky {
            let x = seg.0;
            if x + s > 1.0 + 1e-6 {
                break;
            }
            let y = sky
                .iter()
                .filter(|o| o.0 < x + s - 1e-6 && o.0 + o.1 > x + 1e-6)
                .map(|o| o.2)
                .fold(0.0, f32::max);
            if best.is_none_or(|(by, bx)| y < by - 1e-6 || ((y - by).abs() <= 1e-6 && x < bx)) {
                best = Some((y, x));
            }
        }
        let (y, x) = best?;
        out[i] = Square { x, y, side: s };
        height = height.max(y + s);
        // Raise the skyline over x..x+s to y+s.
        let (a, b) = (x, x + s);
        let mut next = Vec::with_capacity(sky.len() + 2);
        for &(sx, sw, st) in &sky {
            let (l, r) = (sx, sx + sw);
            if r <= a + 1e-6 || l >= b - 1e-6 {
                next.push((sx, sw, st));
                continue;
            }
            if l < a {
                next.push((l, a - l, st));
            }
            if r > b {
                next.push((b, r - b, st));
            }
        }
        next.push((a, s, y + s));
        next.sort_by(|p, q| p.0.total_cmp(&q.0));
        sky = next;
    }
    Some((out, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piles_rest_on_the_floor_and_never_overlap() {
        let xs = [0.0, 1.0, 2.0, 30.0, 0.5, f32::NAN];
        let rs = [5.0, 3.0, 2.0, 4.0, 1.0, 2.0];
        let ys = pile_up(&xs, &rs);
        assert_eq!(ys[0], 5.0, "the biggest sits on the floor");
        assert_eq!(ys[3], 4.0, "a lone circle sits on the floor");
        assert_eq!(ys[5], 0.0);
        for i in 0..5 {
            assert!(ys[i] >= rs[i] - 1e-3);
            for j in 0..i {
                let d = ((xs[i] - xs[j]).powi(2) + (ys[i] - ys[j]).powi(2)).sqrt();
                assert!(d >= rs[i] + rs[j] - 1e-2, "{i} and {j} overlap");
            }
        }
    }

    fn apart(a: (f32, f32, f32), b: (f32, f32, f32)) -> bool {
        (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2) >= (a.2 + b.2).powi(2) * 0.998 - 1e-4
    }

    #[test]
    fn sized_swarm_never_overlaps_and_matches_the_uniform_one() {
        let xs = [0.0, 0.5, 1.0, 1.2, 10.0, 0.1, 0.2];
        let rs = [2.0, 0.5, 1.0, 1.5, 1.0, 0.3, 0.8];
        let ys = swarm_sized(&xs, &rs);
        for i in 0..xs.len() {
            for j in i + 1..xs.len() {
                assert!(
                    apart((xs[i], ys[i], rs[i]), (xs[j], ys[j], rs[j])),
                    "{i} {j}"
                );
            }
        }
        assert_eq!(ys[0], 0.0, "the biggest sits on the axis");
        assert_eq!(swarm_sized(&[1.0, f32::NAN], &[1.0, 1.0]), vec![0.0, 0.0]);
        let uni = crate::swarm::swarm(&xs, 1.0);
        assert_eq!(uni, swarm_sized(&xs, &[1.0; 7]));
    }

    #[test]
    fn packed_siblings_touch_without_overlapping_and_stay_compact() {
        let rs = [5.0, 3.0, 3.0, 2.0, 1.0, 1.0, 4.0, 0.5, 2.5];
        let at = pack_siblings(&rs);
        let cs: Vec<_> = at.iter().zip(rs).map(|(p, r)| (p.0, p.1, r)).collect();
        for i in 0..cs.len() {
            for j in i + 1..cs.len() {
                assert!(apart(cs[i], cs[j]), "{i} {j}");
            }
        }
        let (_, _, big) = enclose(&cs);
        let area: f32 = rs.iter().map(|r| r * r).sum();
        assert!(area / (big * big) > 0.5, "dense: {}", area / (big * big));
        assert!(pack_siblings(&[]).is_empty());
        assert_eq!(pack_siblings(&[2.0]), vec![(0.0, 0.0)]);
    }

    #[test]
    fn enclose_holds_every_circle_and_is_near_minimal() {
        let cs = [(0.0, 0.0, 1.0), (4.0, 0.0, 1.0)];
        let (x, y, r) = enclose(&cs);
        for c in cs {
            assert!(((c.0 - x).powi(2) + (c.1 - y).powi(2)).sqrt() + c.2 <= r + 1e-3);
        }
        assert!(r < 3.1, "{r}");
        assert_eq!(enclose(&[]), (0.0, 0.0, 0.0));
    }

    #[test]
    fn circle_pack_nests_children_inside_parents() {
        let root = Node::branch(
            "all",
            [
                Node::branch(
                    "a",
                    [
                        Node::leaf("a1", 4.0),
                        Node::leaf("a2", 1.0),
                        Node::leaf("a3", 2.0),
                    ],
                ),
                Node::leaf("b", 3.0),
                Node::leaf("zero", 0.0),
            ],
        );
        let ds = circle_pack(&root);
        assert_eq!(ds.len(), 5, "zero-sized nodes left out");
        for d in &ds {
            assert!(d.x - d.r >= -1e-3 && d.x + d.r <= 1.0 + 1e-3, "{d:?}");
            if let Some(p) = d.parent {
                let p = &ds[p];
                let reach = ((d.x - p.x).powi(2) + (d.y - p.y).powi(2)).sqrt() + d.r;
                assert!(reach <= p.r + 1e-3, "{} outside {}", d.label, p.label);
            }
        }
        let leaf = |l: &str| ds.iter().find(|d| d.label == l).unwrap().r;
        assert!((leaf("a1") / leaf("a2") - 2.0).abs() < 1e-3, "area ∝ value");
        assert_eq!(ds.iter().find(|d| d.label == "a2").unwrap().branch, 0);
        assert!(circle_pack(&Node::leaf("x", 1.0)).is_empty());
        assert!(circle_pack(&Node::branch("e", [])).is_empty());
    }

    #[test]
    fn squares_are_proportional_inside_the_box_and_never_overlap() {
        let vs = [100.0, 40.0, 25.0, 25.0, 9.0, 4.0, 1.0, f32::NAN, 0.0];
        let sq = square_pack(&vs, 0.6);
        for s in &sq {
            assert!(s.x >= -1e-4 && s.x + s.side <= 1.0 + 1e-4);
            assert!(s.y >= -1e-4 && s.y + s.side <= 0.6 + 1e-4);
        }
        for i in 0..sq.len() {
            for j in i + 1..sq.len() {
                let (a, b) = (sq[i], sq[j]);
                let overlap = a.x < b.x + b.side - 1e-4
                    && b.x < a.x + a.side - 1e-4
                    && a.y < b.y + b.side - 1e-4
                    && b.y < a.y + a.side - 1e-4;
                assert!(!overlap, "{i} {j}");
            }
        }
        assert!((sq[0].side / sq[1].side - (100.0f32 / 40.0).sqrt()).abs() < 1e-3);
        assert_eq!(sq[7].side, 0.0);
        let used: f32 = sq.iter().map(|s| s.side * s.side).sum();
        assert!(used > 0.6 * 0.5, "fills the box reasonably: {used}");
        assert!(square_pack(&[], 1.0).is_empty());
        assert!(square_pack(&[0.0], 1.0)[0].side == 0.0);
    }
}
