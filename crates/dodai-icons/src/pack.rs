//! The contract every Dodai icon pack and design system shares.
//!
//! * [`Glyph`]: an icon drawn as polygons or polylines on a 24×24 grid, exportable as SVG or an
//!   anti-aliased coverage mask, so any toolkit can draw it.
//! * [`IconPack`]: one source of drawings (Material Symbols, Phosphor, a hand-drawn set, ...).
//!   A pack may cover only some [`IconId`]s.
//! * [`PackChain`]: packs in priority order, the first drawing wins. A design system is a chain
//!   whose last pack covers everything, so [`PackChain::missing`] is empty: that is the
//!   guarantee every design draws every [`IconId`].
//!
//! ```
//! use dodai_icons::{Glyph, IconId, IconPack, PackChain};
//! struct Square;
//! impl IconPack for Square {
//!     fn name(&self) -> &'static str { "square" }
//!     fn glyph(&self, _: IconId, _: bool) -> Option<Glyph> {
//!         Some(Glyph::Filled(vec![vec![(4.0, 4.0), (20.0, 4.0), (20.0, 20.0), (4.0, 20.0)]]))
//!     }
//! }
//! let chain = PackChain::new(vec![Box::new(Square)]);
//! assert!(chain.missing().is_empty());
//! assert!(chain.glyph(IconId::Heart, false).unwrap().svg(24, "red").starts_with("<svg"));
//! ```

use crate::IconId;

/// Points on the 24×24 design grid.
pub type Points = Vec<(f32, f32)>;

/// Stroke width of [`Glyph::Stroked`] on the 24 grid.
pub const STROKE: f32 = 1.75;

/// How a [`Glyph::Lines`] drawing is stroked: a design picks one so the same symbol takes on its
/// look (fine and soft, hard-edged, heavy).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokeStyle {
    /// Line width on the 24 grid.
    pub weight: f32,
    /// Round caps and joins (otherwise square caps and mitred joins).
    pub round: bool,
    /// Sharp corners are cut by this much on the 24 grid (0 keeps them). Only corners that turn
    /// by more than 40 degrees on long enough segments are cut, so faceted arcs stay smooth.
    pub chamfer: f32,
    /// Curves are redrawn with fewer, longer segments: a point stays only if dropping it would
    /// move the line by more than this on the 24 grid (0 keeps every point). A larger value
    /// turns round arcs into angular facets.
    pub facet: f32,
}

impl StrokeStyle {
    /// Fine and soft: detail stays open (Material).
    pub const ROUND: Self = Self {
        weight: 1.4,
        round: true,
        chamfer: 0.0,
        facet: 0.0,
    };
    /// Hard-edged, as the classic stroked pack (Cyberpunk).
    pub const SQUARE: Self = Self {
        weight: 1.3,
        round: false,
        chamfer: 0.9,
        facet: 0.4,
    };
    /// Heavy and soft (Neo-Brutalism).
    pub const BOLD: Self = Self {
        weight: 1.65,
        round: true,
        chamfer: 0.0,
        facet: 0.22,
    };
}

impl StrokeStyle {
    /// `lines` redrawn in this style: curves faceted, then sharp corners cut.
    pub fn shape(&self, lines: Vec<(Points, bool)>) -> Vec<(Points, bool)> {
        lines
            .into_iter()
            .map(|(p, c)| {
                let p = if self.facet > 0.0 && p.len() > 5 {
                    facet(&p, c, self.facet)
                } else {
                    p
                };
                let p = if self.chamfer > 0.0 {
                    self.cut_line(&p, c)
                } else {
                    p
                };
                (p, c)
            })
            .collect()
    }

    fn cut_line(&self, pts: &[(f32, f32)], closed: bool) -> Points {
        let d = self.chamfer;
        let n = pts.len();
        let mut out = Points::new();
        for i in 0..n {
            let p = pts[i];
            let edge = !closed && (i == 0 || i + 1 == n);
            let (a, b) = (pts[(i + n - 1) % n], pts[(i + 1) % n]);
            let (u, v) = ((p.0 - a.0, p.1 - a.1), (b.0 - p.0, b.1 - p.1));
            let (lu, lv) = (
                (u.0 * u.0 + u.1 * u.1).sqrt(),
                (v.0 * v.0 + v.1 * v.1).sqrt(),
            );
            let turn = if lu > 1e-4 && lv > 1e-4 {
                ((u.0 * v.0 + u.1 * v.1) / (lu * lv))
                    .clamp(-1.0, 1.0)
                    .acos()
                    .to_degrees()
            } else {
                0.0
            };
            if edge || turn <= 40.0 || lu < 2.5 * d || lv < 2.5 * d {
                out.push(p);
            } else {
                out.push((p.0 - u.0 / lu * d, p.1 - u.1 / lu * d));
                out.push((p.0 + v.0 / lv * d, p.1 + v.1 / lv * d));
            }
        }
        out
    }
}

/// Douglas-Peucker: `pts` without the points that lie within `tol` of the line between their
/// neighbours. A closed outline is split at its two farthest-apart anchors first.
fn facet(pts: &[(f32, f32)], closed: bool, tol: f32) -> Points {
    fn dist(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l = dx * dx + dy * dy;
        if l < 1e-9 {
            return ((p.0 - a.0).powi(2) + (p.1 - a.1).powi(2)).sqrt();
        }
        ((p.0 - a.0) * dy - (p.1 - a.1) * dx).abs() / l.sqrt()
    }
    fn run(pts: &[(f32, f32)], tol: f32, out: &mut Points) {
        let (a, b) = (pts[0], pts[pts.len() - 1]);
        let far = (1..pts.len() - 1)
            .map(|i| (i, dist(pts[i], a, b)))
            .max_by(|x, y| x.1.total_cmp(&y.1));
        match far {
            Some((i, d)) if d > tol => {
                run(&pts[..=i], tol, out);
                out.pop();
                run(&pts[i..], tol, out);
            }
            _ => {
                out.push(a);
                out.push(b);
            }
        }
    }
    let mut out = Points::new();
    if closed {
        let mut ring = pts.to_vec();
        let first = ring[0];
        ring.push(first);
        let mid = (1..ring.len() - 1)
            .max_by(|&i, &j| {
                let d = |k: usize| (ring[k].0 - first.0).powi(2) + (ring[k].1 - first.1).powi(2);
                d(i).total_cmp(&d(j))
            })
            .unwrap_or(1);
        run(&ring[..=mid], tol, &mut out);
        out.pop();
        run(&ring[mid..], tol, &mut out);
        out.pop();
    } else {
        run(pts, tol, &mut out);
    }
    out.dedup();
    if out.len() < 2 {
        return pts.to_vec();
    }
    out
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self::ROUND
    }
}

/// A source of icon drawings.
pub trait IconPack: Send + Sync {
    /// Short stable name (for diagnostics and tests).
    fn name(&self) -> &'static str;
    /// The drawing of `id`, or `None` if this pack does not have it. `filled` asks for the
    /// filled form where the pack has one.
    fn glyph(&self, id: IconId, filled: bool) -> Option<Glyph>;
}

/// Packs in priority order: the first one that has the icon draws it.
pub struct PackChain {
    packs: Vec<Box<dyn IconPack>>,
}

impl PackChain {
    pub fn new(packs: Vec<Box<dyn IconPack>>) -> Self {
        Self { packs }
    }

    /// First pack that has `id`.
    pub fn glyph(&self, id: IconId, filled: bool) -> Option<Glyph> {
        self.packs.iter().find_map(|p| p.glyph(id, filled))
    }

    /// Names of the packs, highest priority first.
    pub fn names(&self) -> Vec<&'static str> {
        self.packs.iter().map(|p| p.name()).collect()
    }

    /// Every [`IconId`] the chain cannot draw. A design system's chain must return none.
    pub fn missing(&self) -> Vec<IconId> {
        IconId::ALL
            .iter()
            .copied()
            .filter(|&id| self.glyph(id, false).is_none())
            .collect()
    }
}

/// An icon resolved for a design.
#[derive(Clone, Debug, PartialEq)]
pub enum Glyph {
    /// Closed outlines filled with the non-zero rule (holes wind opposite).
    Filled(Vec<Points>),
    /// Polylines `(points, closed)` drawn with a uniform stroke.
    Stroked(Vec<(Points, bool)>),
    /// Polylines `(points, closed)` drawn with a uniform stroke in the given [`StrokeStyle`].
    Lines(Vec<(Points, bool)>, StrokeStyle),
}

/// A stroke with round caps and joins as polygons: a quad per segment and a disc at every
/// vertex that turns (or ends). All wound the same way so a non-zero fill unions them.
fn round_polygons(lines: &[(Points, bool)], weight: f32) -> Vec<Points> {
    let h = weight / 2.0;
    let disc = |c: (f32, f32)| -> Points {
        (0..12)
            .map(|i| {
                let a = i as f32 * std::f32::consts::TAU / 12.0;
                (c.0 + h * a.cos(), c.1 + h * a.sin())
            })
            .collect()
    };
    let mut out = Vec::new();
    for (pts, closed) in lines {
        let n = pts.len();
        let segs = if *closed { n } else { n.saturating_sub(1) };
        let dir = |i: usize| {
            let (a, b) = (pts[i % n], pts[(i + 1) % n]);
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let l = (dx * dx + dy * dy).sqrt();
            (l > 1e-4).then(|| (dx / l, dy / l))
        };
        for i in 0..segs {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            let Some((ux, uy)) = dir(i) else { continue };
            let (nx, ny) = (-uy * h, ux * h);
            out.push(vec![
                (a.0 + nx, a.1 + ny),
                (b.0 + nx, b.1 + ny),
                (b.0 - nx, b.1 - ny),
                (a.0 - nx, a.1 - ny),
            ]);
        }
        for (i, &p) in pts.iter().enumerate() {
            let before = if i > 0 || *closed {
                dir((i + n - 1) % n)
            } else {
                None
            };
            let after = if i + 1 < n || *closed { dir(i) } else { None };
            let smooth =
                matches!((before, after), (Some(a), Some(b)) if a.0 * b.0 + a.1 * b.1 > 0.985);
            if !smooth {
                out.push(disc(p));
            }
        }
        if n == 1 {
            out.push(disc(pts[0]));
        }
    }
    out
}

/// A square-capped, mitre-joined stroke as one quad per segment, each extended by half the width
/// so corners meet.
fn square_polygons(lines: &[(Points, bool)], weight: f32) -> Vec<Points> {
    let h = weight / 2.0;
    let mut out = Vec::new();
    for (pts, closed) in lines {
        let n = pts.len();
        let segs = if *closed { n } else { n.saturating_sub(1) };
        for i in 0..segs {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let l = (dx * dx + dy * dy).sqrt();
            if l < 1e-4 {
                continue;
            }
            let (ux, uy) = (dx / l * h, dy / l * h);
            let (nx, ny) = (-uy, ux);
            let (a, b) = ((a.0 - ux, a.1 - uy), (b.0 + ux, b.1 + uy));
            out.push(vec![
                (a.0 + nx, a.1 + ny),
                (b.0 + nx, b.1 + ny),
                (b.0 - nx, b.1 - ny),
                (a.0 - nx, a.1 - ny),
            ]);
        }
    }
    out
}

impl Glyph {
    /// Polygons covering the glyph (strokes expanded to quads), all wound the
    /// same way so a non-zero fill unions them.
    pub fn polygons(&self) -> Vec<Points> {
        match self {
            Glyph::Filled(p) => p.clone(),
            Glyph::Stroked(lines) => square_polygons(lines, STROKE),
            Glyph::Lines(lines, st) if st.round => round_polygons(lines, st.weight),
            Glyph::Lines(lines, st) => square_polygons(lines, st.weight),
        }
    }

    /// Standalone SVG. `color` is any SVG paint (`"currentColor"`, `"#fcee0a"`).
    pub fn svg(&self, size: u32, color: &str) -> String {
        let head = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{size}\" height=\"{size}\" viewBox=\"0 0 24 24\""
        );
        match self {
            Glyph::Filled(polys) => {
                let d: String = polys.iter().map(|p| path(p, true)).collect();
                format!("{head} fill=\"{color}\"><path fill-rule=\"nonzero\" d=\"{d}\"/></svg>")
            }
            Glyph::Lines(lines, st) => {
                let d: String = lines.iter().map(|(p, c)| path(p, *c)).collect();
                let (w, cap, join) = if st.round {
                    (st.weight, "round", "round")
                } else {
                    (st.weight, "square", "miter")
                };
                format!(
                    "{head} fill=\"none\" stroke=\"{color}\" stroke-width=\"{w}\" \
                     stroke-linecap=\"{cap}\" stroke-linejoin=\"{join}\"><path d=\"{d}\"/></svg>"
                )
            }
            Glyph::Stroked(lines) => {
                let d: String = lines.iter().map(|(p, c)| path(p, *c)).collect();
                format!(
                    "{head} fill=\"none\" stroke=\"{color}\" stroke-width=\"{STROKE}\" \
                     stroke-linecap=\"square\" stroke-linejoin=\"miter\"><path d=\"{d}\"/></svg>"
                )
            }
        }
    }

    /// Anti-aliased alpha mask, `px × px` bytes, row-major (4×4 supersampling).
    /// Tint it with the theme colour: this is how toolkits without vector
    /// drawing (bevy, terminals with images, …) show icons.
    pub fn coverage(&self, px: u32) -> Vec<u8> {
        const SS: u32 = 4;
        let polys = self.polygons();
        let k = 24.0 / px as f32;
        let mut out = vec![0u8; (px * px) as usize];
        for y in 0..px {
            for x in 0..px {
                let mut hit = 0;
                for sy in 0..SS {
                    for sx in 0..SS {
                        let fx = (x * SS + sx) as f32 + 0.5;
                        let fy = (y * SS + sy) as f32 + 0.5;
                        let (gx, gy) = (fx * k / SS as f32, fy * k / SS as f32);
                        if polys.iter().map(|p| winding(p, gx, gy)).sum::<i32>() != 0 {
                            hit += 1;
                        }
                    }
                }
                out[(y * px + x) as usize] = (hit * 255 / (SS * SS)) as u8;
            }
        }
        out
    }
}

fn path(p: &[(f32, f32)], closed: bool) -> String {
    let mut s = String::new();
    for (i, (x, y)) in p.iter().enumerate() {
        s.push(if i == 0 { 'M' } else { 'L' });
        s.push_str(&format!("{:.2} {:.2}", x, y));
    }
    if closed {
        s.push('Z');
    }
    s
}

/// Winding number of `p` around `(x, y)`.
fn winding(p: &[(f32, f32)], x: f32, y: f32) -> i32 {
    let mut w = 0;
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        let cross = (b.0 - a.0) * (y - a.1) - (x - a.0) * (b.1 - a.1);
        if a.1 <= y {
            if b.1 > y && cross > 0.0 {
                w += 1;
            }
        } else if b.1 <= y && cross < 0.0 {
            w -= 1;
        }
    }
    w
}
