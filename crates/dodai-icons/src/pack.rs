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
}

impl Glyph {
    /// Polygons covering the glyph (strokes expanded to quads), all wound the
    /// same way so a non-zero fill unions them.
    pub fn polygons(&self) -> Vec<Points> {
        match self {
            Glyph::Filled(p) => p.clone(),
            Glyph::Stroked(lines) => {
                let h = STROKE / 2.0;
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
                        // Extend by h so corners join without gaps.
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
