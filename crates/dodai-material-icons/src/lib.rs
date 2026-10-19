//! The [Material Symbols](https://fonts.google.com/icons) catalogue as
//! dependency-free vector data: 3,900+ icons, each also as a `-fill` variant,
//! in the three official styles (feature-gated so you only compile what you use).
//!
//! ```
//! use dodai_material_icons::outlined;
//! let home = outlined::get("home").unwrap();
//! assert!(home.svg().starts_with("<svg"));
//! // Flattened to polygons on a 24×24 grid, for renderers without an SVG parser.
//! assert!(!home.polygons().is_empty());
//! assert!(outlined::get("home-fill").is_some());
//! ```
//!
//! Icon artwork © Google, [Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0).
//! The crate carries only path data (viewBox `0 -960 960 960`); there is no
//! font, no I/O and no dependency.

mod aliases;
pub use aliases::{resolve, ALIASES};

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
            .map(|p| simplify_closed(&p, FACET_TOLERANCE))
            .filter(|p| p.len() >= 3)
            .collect()
    }

    /// Whether this is a `-fill` variant.
    pub fn is_filled(&self) -> bool {
        self.name.ends_with("-fill")
    }
}

/// Max distance (24-grid units) a facet may drift from the smooth outline.
pub const FACET_TOLERANCE: f32 = 0.9;

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
    if best <= tol {
        return vec![a, b];
    }
    let mut l = dp(&p[..=at], tol);
    l.pop();
    l.extend(dp(&p[at..], tol));
    l
}

/// Binary-search `name` in a catalogue sorted by name.
pub fn find(all: &'static [Icon], name: &str) -> Option<&'static Icon> {
    all.binary_search_by(|i| i.name.cmp(name))
        .ok()
        .map(|i| &all[i])
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
        for n in ["circle", "settings", "home", "favorite", "search"] {
            let i = crate::outlined::get(n).unwrap();
            let smooth: usize = i.polygons().iter().map(Vec::len).sum();
            let facet = i.faceted();
            let pts: usize = facet.iter().map(Vec::len).sum();
            assert!(pts > 2 && pts <= smooth, "{n}: {pts} vs {smooth}");
            for (x, y) in facet.iter().flatten() {
                assert!(
                    (-0.5..=24.5).contains(x) && (-0.5..=24.5).contains(y),
                    "{n}"
                );
            }
        }
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
        for n in [
            "home",
            "home-fill",
            "search",
            "settings",
            "arrow_back",
            "close",
            "menu",
            "favorite",
        ] {
            assert!(outlined::get(n).is_some(), "{n}");
        }
        assert!(outlined::get("home-fill").unwrap().is_filled());
        assert!(outlined::get("no_such_icon").is_none());
    }
}
