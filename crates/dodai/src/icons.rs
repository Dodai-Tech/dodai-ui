//! Toolkit-neutral icons by name: the right glyph for a theme, as SVG or as a
//! coverage mask, so any adapter can draw it.
//!
//! * Rounded themes (Material) get the smooth Material Symbols glyph.
//! * Other themes (Cyberpunk) get the hand-drawn Cyberpunk icon when one
//!   exists for the name, otherwise the Material glyph reduced to straight
//!   facets.
//!
//! ```
//! # #[cfg(all(feature = "cyberpunk", feature = "material"))] {
//! let t = dodai::themes::neutral();
//! let g = dodai::icons::glyph("home", &t, false).unwrap();
//! assert!(g.svg(24, "currentColor").starts_with("<svg"));
//! assert_eq!(g.coverage(24).len(), 24 * 24);
//! # }
//! ```

use crate::{ShapeKind, Theme};

/// Points on the 24×24 design grid.
pub type Points = Vec<(f32, f32)>;

/// An icon resolved for a theme.
#[derive(Clone, Debug, PartialEq)]
pub enum Glyph {
    /// Closed outlines filled with the non-zero rule (holes wind opposite).
    Filled(Vec<Points>),
    /// Polylines `(points, closed)` drawn with a uniform stroke.
    Stroked(Vec<(Points, bool)>),
}

/// Stroke width of [`Glyph::Stroked`] on the 24 grid.
pub const STROKE: f32 = 1.75;

/// Resolve `name` (Cyberpunk or Material Symbols spelling) for `theme`.
/// `filled` picks the Material `-fill` variant when there is one.
pub fn glyph(name: &str, theme: &Theme, filled: bool) -> Option<Glyph> {
    let rounded = theme.shape.kind() == ShapeKind::Rounded;
    #[cfg(feature = "cyberpunk")]
    if !rounded {
        if let Some(i) = dodai_cyberpunk::icons::IconName::from_name(name) {
            return Some(Glyph::Stroked(i.shapes()));
        }
    }
    #[cfg(feature = "material")]
    {
        use dodai_material::icons as m;
        let n = m::resolve(name);
        let icon = (filled)
            .then(|| m::outlined::get(&format!("{n}-fill")))
            .flatten()
            .or_else(|| m::outlined::get(n))?;
        return Some(Glyph::Filled(if rounded {
            icon.polygons()
        } else {
            icon.faceted()
        }));
    }
    #[allow(unreachable_code)]
    {
        let _ = (name, filled, rounded);
        None
    }
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

#[cfg(all(test, feature = "cyberpunk", feature = "material"))]
mod tests {
    use super::*;

    fn ink(g: &Glyph, px: u32) -> usize {
        g.coverage(px).iter().filter(|&&a| a > 127).count()
    }

    #[test]
    fn every_pack_resolves_and_draws() {
        let cyber = crate::themes::cyberpunk::yellow();
        let mat = crate::themes::material::light();
        for t in [&cyber, &mat] {
            for n in ["home", "settings", "search", "favorite", "flight"] {
                let g = glyph(n, t, false).unwrap_or_else(|| panic!("{n}"));
                assert!(ink(&g, 24) > 20, "{n} drew nothing in {}", t.id());
                assert!(g.svg(24, "#fff").contains("<path"));
            }
        }
    }

    #[test]
    fn theme_shape_picks_the_style() {
        let cyber = crate::themes::cyberpunk::yellow();
        let mat = crate::themes::material::light();
        assert!(matches!(
            glyph("home", &cyber, false),
            Some(Glyph::Stroked(_))
        ));
        assert!(matches!(glyph("home", &mat, false), Some(Glyph::Filled(_))));
        // A catalogue-only name still resolves on Cyberpunk, as facets.
        assert!(matches!(
            glyph("sports_esports", &cyber, false),
            Some(Glyph::Filled(_))
        ));
        assert!(glyph("definitely_not_an_icon", &cyber, false).is_none());
    }

    #[test]
    fn filled_variant_has_more_ink() {
        let mat = crate::themes::material::light();
        let line = glyph("favorite", &mat, false).unwrap();
        let fill = glyph("favorite", &mat, true).unwrap();
        assert!(ink(&fill, 32) > ink(&line, 32));
    }

    #[test]
    fn holes_stay_empty() {
        // "circle" outlined is a ring: centre empty, edge inked.
        let mat = crate::themes::material::light();
        let c = glyph("circle", &mat, false).unwrap().coverage(24);
        assert_eq!(c[12 * 24 + 12], 0);
    }
}
