//! The [Material Symbols](https://fonts.google.com/icons) catalogue as
//! dependency-free vector data: 3,900+ icons, each also as a `-fill` variant,
//! in the three official styles (feature-gated so you only compile what you use).
//!
//! ```
//! use dodai_material_icons::{outlined, MaterialSymbol};
//! let home = outlined::get(MaterialSymbol::Home).unwrap();
//! assert!(home.svg().starts_with("<svg"));
//! // Flattened to polygons on a 24×24 grid, for renderers without an SVG parser.
//! assert!(!home.polygons().is_empty());
//! assert!(outlined::get_fill(MaterialSymbol::Home).is_some());
//! ```
//!
//! Icon artwork © Google, [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0).
//! The crate carries only path data (viewBox `0 -960 960 960`); there is no
//! font and no I/O. Icons are named by the typed [`MaterialSymbol`] enum (autocomplete, no
//! strings); the shared cross-design vocabulary is [`IconId`](dodai_icon_id::IconId).

mod aliases;
mod symbol;
#[doc(hidden)]
pub use aliases::alias_of;
pub use symbol::MaterialSymbol;

#[cfg(feature = "outlined")]
mod categories;
#[cfg(feature = "outlined")]
pub use categories::CATEGORIES;
#[cfg(feature = "outlined")]
pub mod outlined;
#[cfg(feature = "rounded")]
pub mod rounded;
#[cfg(feature = "sharp")]
pub mod sharp;

/// Side of the viewBox, in source units (`0 -960 960 960`).
pub const VIEW: f32 = 960.0;
/// A flattened subpath on the 24×24 grid: closed polygon points.
pub type Polygon = Vec<(f32, f32)>;

/// One icon: its catalogue name and SVG path data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Icon {
    /// Catalogue name, e.g. `home`, `home-fill`, `arrow_back`.
    pub name: &'static str,
    /// SVG path `d` attribute in viewBox `0 -960 960 960`.
    pub path: &'static str,
}

impl Icon {
    /// Standalone SVG at `size` px, filled with `currentColor`.
    pub fn svg(&self) -> String {
        self.svg_sized(24)
    }

    /// Standalone SVG at `size` px, filled with `currentColor`.
    pub fn svg_sized(&self, size: u32) -> String {
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" height=\"{size}\" \
             viewBox=\"0 -960 960 960\" fill=\"currentColor\"><path d=\"{}\"/></svg>",
            self.path
        )
    }

    /// The icon flattened to closed polygons on a 24×24 grid (curves are
    /// subdivided). Fill them with the **non-zero** rule: holes are wound
    /// opposite to their outline.
    pub fn polygons(&self) -> Vec<Polygon> {
        flatten(self.path)
            .into_iter()
            .map(|p| {
                p.into_iter()
                    .map(|(x, y)| (x * 24.0 / VIEW, (y + VIEW) * 24.0 / VIEW))
                    .collect()
            })
            .collect()
    }

    /// The icon with every curve reduced to a few straight facets (24×24
    /// grid, same winding rules as [`polygons`](Self::polygons)): circles
    /// become octagon-like polygons, rounded corners become chamfers. This is
    /// how the whole catalogue is drawn in an angular design language.
    pub fn faceted(&self) -> Vec<Polygon> {
        self.polygons()
            .into_iter()
            .filter_map(|p| facet_polygon(&p))
            .collect()
    }

    /// Whether this is a `-fill` variant.
    pub fn is_filled(&self) -> bool {
        self.name.ends_with("-fill")
    }
}

/// Max distance (24-grid units) a facet may drift from the smooth outline
/// before octilinear snapping.
pub const FACET_TOLERANCE: f32 = 0.55;

/// Smallest outline area (24-grid units²) worth keeping.
const MIN_AREA: f32 = 0.2;

/// Smooth outline → angular polygon in the Cyberpunk language:
///
/// 1. round rings become regular octagons (flat top, like the hand-drawn pack);
/// 2. everything else is simplified (Douglas–Peucker) and snapped so edges are
///    horizontal, vertical or 45° ([`octilinear`]);
/// 3. a snapped result that drifted from the outline or self-intersects falls
///    back to the plain simplified polygon, so no icon is ever mangled.
pub fn facet_polygon(p: &[(f32, f32)]) -> Option<Polygon> {
    if let Some(o) = octagon_for(p) {
        return Some(o);
    }
    // Coarse to fine: the first snapped polygon that stays faithful wins; thin
    // strokes need the finer pass, and at worst we keep the plain facets.
    let mut best = None;
    for tol in [FACET_TOLERANCE, 0.38, 0.25] {
        let dp = simplify_closed(p, tol);
        let snapped = octilinear(&dp);
        if faithful(&snapped, p) {
            best = Some(snapped);
            break;
        }
        best = Some(dp);
    }
    best.filter(|b| b.len() >= 3 && area(b).abs() >= MIN_AREA)
}

/// A regular octagon when `p` is (nearly) a circle.
fn octagon_for(p: &[(f32, f32)]) -> Option<Polygon> {
    if p.len() < 8 {
        return None;
    }
    let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for &(x, y) in p {
        (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
    }
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let r = ((x1 - x0) + (y1 - y0)) / 4.0;
    if r < 0.6 || ((x1 - x0) - (y1 - y0)).abs() > 0.15 * r {
        return None;
    }
    let (lo, hi) = p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &(x, y)| {
        let d = (x - cx).hypot(y - cy);
        (lo.min(d), hi.max(d))
    });
    if hi - lo > 0.06 * r.max(1.0) {
        return None;
    }
    let rc = r / (std::f32::consts::PI / 8.0).cos();
    let flip = if area(p) < 0.0 { -1.0 } else { 1.0 };
    Some(
        (0..8)
            .map(|i| {
                let a = (22.5 + 45.0 * i as f32 * flip).to_radians();
                (cx + rc * a.cos(), cy + rc * a.sin())
            })
            .collect(),
    )
}

/// `cand` stays within `FAITHFUL` of `orig` both ways, keeps its area and does
/// not cross itself.
fn faithful(cand: &[(f32, f32)], orig: &[(f32, f32)]) -> bool {
    const FAITHFUL: f32 = 0.5;
    if cand.len() < 3 {
        return false;
    }
    let (ac, ao) = (area(cand), area(orig));
    if ac.signum() != ao.signum() || (ac - ao).abs() > 0.2 * ao.abs().max(1.0) {
        return false;
    }
    if cand.iter().any(|&q| dist_to_ring(q, orig) > FAITHFUL)
        || orig.iter().any(|&q| dist_to_ring(q, cand) > FAITHFUL)
    {
        return false;
    }
    !self_intersects(cand)
}

fn dist_to_ring(q: (f32, f32), ring: &[(f32, f32)]) -> f32 {
    let n = ring.len();
    (0..n)
        .map(|i| {
            let (a, b) = (ring[i], ring[(i + 1) % n]);
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let l2 = dx * dx + dy * dy;
            let t = if l2 < 1e-9 {
                0.0
            } else {
                (((q.0 - a.0) * dx + (q.1 - a.1) * dy) / l2).clamp(0.0, 1.0)
            };
            (q.0 - (a.0 + dx * t)).hypot(q.1 - (a.1 + dy * t))
        })
        .fold(f32::MAX, f32::min)
}

fn self_intersects(p: &[(f32, f32)]) -> bool {
    let n = p.len();
    let ccw = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    };
    for i in 0..n {
        let (a, b) = (p[i], p[(i + 1) % n]);
        for j in i + 2..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            let (c, d) = (p[j], p[(j + 1) % n]);
            if ccw(a, b, c) * ccw(a, b, d) < 0.0 && ccw(c, d, a) * ccw(c, d, b) < 0.0 {
                return true;
            }
        }
    }
    false
}

fn area(p: &[(f32, f32)]) -> f32 {
    let n = p.len();
    (0..n)
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % n]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum::<f32>()
        / 2.0
}

/// Snap every edge direction to a multiple of 45° (lines through the edge
/// midpoints, corners re-intersected). Corners that cannot be intersected
/// sanely keep their original position.
pub fn octilinear(p: &[(f32, f32)]) -> Polygon {
    let n = p.len();
    if n < 3 {
        return p.to_vec();
    }
    // (midpoint, unit direction) per edge.
    let lines: Vec<((f32, f32), (f32, f32))> = (0..n)
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % n]);
            let ang = (b.1 - a.1).atan2(b.0 - a.0);
            let q = (ang / std::f32::consts::FRAC_PI_4).round() * std::f32::consts::FRAC_PI_4;
            (((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0), (q.cos(), q.sin()))
        })
        .collect();
    let mut out: Polygon = Vec::with_capacity(n);
    for i in 0..n {
        let (m0, d0) = lines[(i + n - 1) % n];
        let (m1, d1) = lines[i];
        let cross = d0.0 * d1.1 - d0.1 * d1.0;
        let v = p[i];
        let pt = if cross.abs() < 0.2 {
            // Same direction: slide the vertex onto the shared line.
            let (dx, dy) = d1;
            let t = (v.0 - m1.0) * dx + (v.1 - m1.1) * dy;
            (m1.0 + dx * t, m1.1 + dy * t)
        } else {
            let t = ((m1.0 - m0.0) * d1.1 - (m1.1 - m0.1) * d1.0) / cross;
            (m0.0 + d0.0 * t, m0.1 + d0.1 * t)
        };
        let far = (pt.0 - v.0).hypot(pt.1 - v.1) > 1.6;
        out.push(if far { v } else { pt });
    }
    // Drop repeats and collinear points.
    let mut clean: Polygon = Vec::with_capacity(out.len());
    for &q in &out {
        if clean
            .last()
            .is_none_or(|l| (l.0 - q.0).hypot(l.1 - q.1) > 0.12)
        {
            clean.push(q);
        }
    }
    if clean.len() > 1 {
        let (f, l) = (clean[0], clean[clean.len() - 1]);
        if (f.0 - l.0).hypot(f.1 - l.1) <= 0.12 {
            clean.pop();
        }
    }
    let m = clean.len();
    if m < 3 {
        return clean;
    }
    (0..m)
        .filter(|&i| {
            let (a, b, c) = (clean[(i + m - 1) % m], clean[i], clean[(i + 1) % m]);
            ((b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0)).abs() > 0.02
        })
        .map(|i| clean[i])
        .collect()
}

/// Douglas–Peucker for a closed ring: anchor on the first point and the one
/// farthest from it, simplify both halves.
pub fn simplify_closed(p: &[(f32, f32)], tol: f32) -> Polygon {
    if p.len() < 4 {
        return p.to_vec();
    }
    let d2 = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2);
    let far = (1..p.len())
        .max_by(|&i, &j| d2(p[0], p[i]).total_cmp(&d2(p[0], p[j])))
        .unwrap_or(1);
    let mut first = p[..=far].to_vec();
    let mut second = p[far..].to_vec();
    second.push(p[0]);
    let mut out = dp(&first, tol);
    out.pop();
    let mut tail = dp(&second, tol);
    tail.pop();
    out.extend(tail);
    first.clear();
    out
}

fn dp(p: &[(f32, f32)], tol: f32) -> Vec<(f32, f32)> {
    if p.len() < 3 {
        return p.to_vec();
    }
    let (a, b) = (p[0], p[p.len() - 1]);
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = (dx * dx + dy * dy).sqrt();
    let dist = |q: (f32, f32)| {
        if len < 1e-6 {
            ((q.0 - a.0).powi(2) + (q.1 - a.1).powi(2)).sqrt()
        } else {
            ((q.0 - a.0) * dy - (q.1 - a.1) * dx).abs() / len
        }
    };
    let (mut at, mut best) = (0, 0.0f32);
    for (i, &q) in p.iter().enumerate().take(p.len() - 1).skip(1) {
        let d = dist(q);
        if d > best {
            (at, best) = (i, d);
        }
    }
    // `at == 0` means nothing is farther than the chord (also for a negative or NaN
    // `tol`): without it the split below would recurse on the same slice forever.
    if at == 0 || best <= tol {
        return vec![a, b];
    }
    let mut l = dp(&p[..=at], tol);
    l.pop();
    l.extend(dp(&p[at..], tol));
    l
}

/// Binary-search `name` in a catalogue sorted by name. For the icon pack crates' own tables.
#[doc(hidden)]
pub fn find(all: &'static [Icon], name: &str) -> Option<&'static Icon> {
    all.binary_search_by(|i| i.name.cmp(name))
        .ok()
        .map(|i| &all[i])
}

/// The `-fill` form of the icon `name`.
fn find_fill(all: &'static [Icon], name: &str) -> Option<&'static Icon> {
    find(all, &format!("{name}-fill"))
}

/// Material's drawing of a shared-vocabulary icon: the Material icon a Dodai name is an alias of
/// (`arrow-left` is `arrow_back`), else the Material icon of the same name. `None` when
/// Material has neither.
pub fn lookup(
    all: &'static [Icon],
    id: dodai_icon_id::IconId,
    filled: bool,
) -> Option<&'static Icon> {
    let key = id.name();
    let kebab = key.replace('_', "-");
    let target = aliases::resolve(&kebab);
    let pick = |n: &str| {
        (filled)
            .then(|| find_fill(all, n))
            .flatten()
            .or_else(|| find(all, n))
    };
    pick(target).or_else(|| pick(key))
}

/// Segments per quadratic curve.
const CURVE_STEPS: usize = 8;

/// Parse the subset of SVG path syntax Material Symbols uses
/// (`M L H V Q T Z`, absolute and relative) into closed polygons in source units.
pub fn flatten(d: &str) -> Vec<Polygon> {
    let b = d.as_bytes();
    let mut i = 0;
    let (mut x, mut y) = (0.0f32, 0.0f32);
    let (mut sx, mut sy) = (0.0f32, 0.0f32);
    // Last quadratic control point, for `T` reflection.
    let mut ctrl: Option<(f32, f32)> = None;
    let mut cmd = b'M';
    let mut cur: Polygon = Vec::new();
    let mut out: Vec<Polygon> = Vec::new();
    let num = |i: &mut usize| -> Option<f32> {
        while *i < b.len() && (b[*i] == b' ' || b[*i] == b',') {
            *i += 1;
        }
        let s = *i;
        if *i < b.len() && (b[*i] == b'-' || b[*i] == b'+') {
            *i += 1;
        }
        let mut dot = false;
        while *i < b.len() {
            match b[*i] {
                b'0'..=b'9' => *i += 1,
                b'.' if !dot => {
                    dot = true;
                    *i += 1;
                }
                _ => break,
            }
        }
        d[s..*i].parse().ok()
    };
    let flush = |cur: &mut Polygon, out: &mut Vec<Polygon>| {
        if cur.len() > 2 {
            out.push(std::mem::take(cur));
        } else {
            cur.clear();
        }
    };
    while i < b.len() {
        while i < b.len() && (b[i] == b' ' || b[i] == b',') {
            i += 1;
        }
        if i >= b.len() {
            break;
        }
        if b[i].is_ascii_alphabetic() {
            cmd = b[i];
            i += 1;
            if cmd == b'Z' || cmd == b'z' {
                flush(&mut cur, &mut out);
                (x, y) = (sx, sy);
                ctrl = None;
                continue;
            }
        }
        let rel = cmd.is_ascii_lowercase();
        let (ox, oy) = if rel { (x, y) } else { (0.0, 0.0) };
        match cmd.to_ascii_uppercase() {
            b'M' | b'L' => {
                let (Some(px), Some(py)) = (num(&mut i), num(&mut i)) else {
                    break;
                };
                if cmd.eq_ignore_ascii_case(&b'M') {
                    flush(&mut cur, &mut out);
                    (sx, sy) = (ox + px, oy + py);
                    cmd = if rel { b'l' } else { b'L' };
                }
                (x, y) = (ox + px, oy + py);
                cur.push((x, y));
                ctrl = None;
            }
            b'H' => {
                let Some(px) = num(&mut i) else { break };
                x = ox + px;
                cur.push((x, y));
                ctrl = None;
            }
            b'V' => {
                let Some(py) = num(&mut i) else { break };
                y = oy + py;
                cur.push((x, y));
                ctrl = None;
            }
            c @ (b'Q' | b'T') => {
                let (cx, cy) = if c == b'Q' {
                    let (Some(a), Some(bb)) = (num(&mut i), num(&mut i)) else {
                        break;
                    };
                    (ox + a, oy + bb)
                } else {
                    ctrl.map_or((x, y), |(qx, qy)| (2.0 * x - qx, 2.0 * y - qy))
                };
                let (Some(a), Some(bb)) = (num(&mut i), num(&mut i)) else {
                    break;
                };
                let (ex, ey) = (ox + a, oy + bb);
                for k in 1..=CURVE_STEPS {
                    let t = k as f32 / CURVE_STEPS as f32;
                    let u = 1.0 - t;
                    cur.push((
                        u * u * x + 2.0 * u * t * cx + t * t * ex,
                        u * u * y + 2.0 * u * t * cy + t * t * ey,
                    ));
                }
                (x, y) = (ex, ey);
                ctrl = Some((cx, cy));
            }
            _ => break,
        }
    }
    flush(&mut cur, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flatten_rectangle_absolute_and_relative() {
        let abs = flatten("M0 0H10V10H0Z");
        let rel = flatten("m0 0h10v10h-10Z");
        assert_eq!(abs, rel);
        assert_eq!(abs.len(), 1);
        assert_eq!(
            abs[0],
            vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]
        );
    }

    #[test]
    fn flatten_handles_compact_numbers_and_implicit_lineto() {
        // "-" separates numbers; extra pairs after M are implicit L.
        let p = flatten("M10-20 30 40-50 60Z");
        assert_eq!(p[0], vec![(10.0, -20.0), (30.0, 40.0), (-50.0, 60.0)]);
    }

    #[test]
    fn flatten_subdivides_quadratics_and_reflects_t() {
        let p = flatten("M0 0Q10 10 20 0T40 0Z");
        // 1 start + 8 + 8 points; the T control mirrors (10,10) to (30,-10).
        assert_eq!(p[0].len(), 17);
        assert!(
            p[0].iter().any(|&(_, y)| y < -1.0),
            "second arc bulges the other way"
        );
    }

    #[test]
    fn flatten_survives_garbage() {
        for d in [
            "", "M", "M1", "Q1 2", "Z", "X1 2", "M0 0 L", "M0 0L1e", "...", "M-",
        ] {
            let _ = flatten(d);
        }
    }

    #[test]
    fn faceted_is_simpler_and_stays_close() {
        #[cfg(feature = "outlined")]
        for n in [
            MaterialSymbol::Circle,
            MaterialSymbol::Settings,
            MaterialSymbol::Home,
            MaterialSymbol::Favorite,
            MaterialSymbol::Search,
        ] {
            let i = crate::outlined::get(n).unwrap();
            let smooth: usize = i.polygons().iter().map(Vec::len).sum();
            let facet = i.faceted();
            let pts: usize = facet.iter().map(Vec::len).sum();
            assert!(pts > 2 && pts <= smooth, "{n:?}: {pts} vs {smooth}");
            for (x, y) in facet.iter().flatten() {
                assert!(
                    (-0.5..=24.5).contains(x) && (-0.5..=24.5).contains(y),
                    "{n:?}"
                );
            }
        }
    }

    /// Whole catalogue: valid geometry and, overall, an angular (octilinear)
    /// look rather than arbitrary angles.
    #[cfg(feature = "outlined")]
    #[test]
    fn faceted_catalogue_is_valid_and_mostly_octilinear() {
        let (mut edges, mut octi, mut empty) = (0usize, 0usize, 0usize);
        for i in crate::outlined::ALL {
            let f = i.faceted();
            if f.is_empty() && !i.polygons().is_empty() {
                empty += 1;
            }
            for p in &f {
                assert!(p.len() >= 3, "{}", i.name);
                for &(x, y) in p {
                    assert!(x.is_finite() && y.is_finite(), "{}", i.name);
                    assert!(
                        (-2.0..=26.0).contains(&x) && (-2.0..=26.0).contains(&y),
                        "{}",
                        i.name
                    );
                }
                for k in 0..p.len() {
                    let (a, b) = (p[k], p[(k + 1) % p.len()]);
                    let ang = (b.1 - a.1).atan2(b.0 - a.0).to_degrees().rem_euclid(45.0);
                    edges += 1;
                    if ang < 2.0 || ang > 43.0 {
                        octi += 1;
                    }
                }
            }
        }
        assert!(
            empty * 100 <= crate::outlined::ALL.len(),
            "{empty} icons vanished"
        );
        let share = octi as f32 / edges as f32;
        assert!(share > 0.7, "only {:.0}% octilinear edges", share * 100.0);
    }

    #[test]
    fn polygons_land_on_the_24_grid() {
        let i = Icon {
            name: "t",
            path: "M0-960H960V0H0Z",
        };
        let p = i.polygons();
        assert_eq!(
            p[0],
            vec![(0.0, 0.0), (24.0, 0.0), (24.0, 24.0), (0.0, 24.0)]
        );
    }

    fn check(all: &[Icon]) {
        assert!(all.len() > 7000);
        assert!(
            all.windows(2).all(|w| w[0].name < w[1].name),
            "sorted, no dups"
        );
        for ic in all {
            assert!(!ic.name.is_empty() && !ic.path.is_empty(), "{}", ic.name);
            let polys = ic.polygons();
            assert!(!polys.is_empty(), "{} has geometry", ic.name);
            for &(x, y) in polys.iter().flatten() {
                // Some glyphs bleed slightly past the 24px box.
                assert!(
                    (-2.0..=26.0).contains(&x) && (-2.0..=26.0).contains(&y),
                    "{}: {x},{y}",
                    ic.name
                );
            }
        }
    }

    #[cfg(feature = "outlined")]
    #[test]
    fn outlined_catalogue_is_well_formed() {
        check(outlined::ALL);
    }

    #[cfg(feature = "rounded")]
    #[test]
    fn rounded_catalogue_is_well_formed() {
        check(rounded::ALL);
    }

    #[cfg(feature = "sharp")]
    #[test]
    fn sharp_catalogue_is_well_formed() {
        check(sharp::ALL);
    }

    #[cfg(feature = "outlined")]
    #[test]
    fn lookup_finds_known_icons_and_fill_variants() {
        use MaterialSymbol as M;
        for n in [
            M::Home,
            M::Search,
            M::Settings,
            M::ArrowBack,
            M::Close,
            M::Menu,
            M::Favorite,
        ] {
            assert!(outlined::get(n).is_some(), "{n:?}");
            assert!(outlined::get_fill(n).is_some(), "{n:?} fill");
        }
        assert!(outlined::get_fill(M::Home).unwrap().is_filled());
    }

    /// Every catalogue symbol has its outline form in the table.
    #[cfg(feature = "outlined")]
    #[test]
    fn every_symbol_is_in_the_catalogue() {
        for &m in MaterialSymbol::ALL {
            assert!(outlined::get(m).is_some(), "{m:?}");
        }
    }

    /// A Dodai name resolves to Material's own icon for it (`arrow-left` is `arrow_back`).
    #[cfg(feature = "outlined")]
    #[test]
    fn shared_vocabulary_resolves_through_aliases() {
        use dodai_icon_id::IconId;
        let back = outlined::get(MaterialSymbol::ArrowBack).unwrap();
        assert_eq!(outlined::get_id(IconId::ArrowLeft, false), Some(back));
        assert!(outlined::get_id(IconId::Home, true).unwrap().is_filled());
    }

    /// Found by fuzzing: a negative or NaN tolerance used to recurse forever.
    #[test]
    fn simplify_survives_degenerate_tolerances() {
        let ring = [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (2.0, 5.0), (0.0, 4.0)];
        for tol in [-1.0, f32::NAN, f32::INFINITY, 0.0] {
            let out = simplify_closed(&ring, tol);
            assert!(out.len() >= 2, "tol {tol}");
        }
    }

    #[cfg(feature = "outlined")]
    #[test]
    fn every_icon_is_in_exactly_one_category() {
        let mut seen = std::collections::HashMap::new();
        for (cat, syms) in CATEGORIES {
            for n in *syms {
                assert!(seen.insert(*n, *cat).is_none(), "{n:?} listed twice");
            }
        }
        for &m in MaterialSymbol::ALL {
            assert!(seen.contains_key(&m), "{m:?} has no category");
        }
    }
}
