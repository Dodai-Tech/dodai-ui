//! Resolves a typed [`IconId`] to a [`Glyph`] in the style of a design system.
//!
//! Each design system has its own pack and a fixed fallback, so every [`IconId`] draws in every
//! design (a test enforces it):
//!
//! * **Stroked** (Cyberpunk): the hand-drawn, curve-free icon; else the Phosphor drawing, faceted
//!   into straight segments.
//! * **Material** (Material 3): the Material Symbols icon, with the spec's own art for every name
//!   Material has; else Phosphor, as is.
//! * **Bold** (Neo-Brutalism): Phosphor's bold weight (`-fill` for filled).
//!
//! ```
//! use dodai_icons::{glyph, IconId, Pack};
//! let g = glyph(IconId::Heart, Pack::Material, false).unwrap();
//! assert!(g.svg(24, "currentColor").starts_with("<svg"));
//! assert_eq!(g.coverage(24).len(), 24 * 24);
//! ```

pub use dodai_icon_id::IconId;

/// Points on the 24×24 design grid.
pub type Points = Vec<(f32, f32)>;

/// Cyberpunk's polyline modifiers (`Off`, `Add`, `Circle`, ...), derived from a drawn icon.
#[cfg(feature = "cyberpunk")]
pub use dodai_cyberpunk_icons::Modifier;

/// Which design's pack draws the icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pack {
    /// Cyberpunk: hand-drawn straight segments and chamfers.
    Stroked,
    /// Material 3: Material Symbols.
    Material,
    /// Neo-Brutalism: bold Phosphor.
    Bold,
}

impl From<dodai_core::IconSet> for Pack {
    fn from(s: dodai_core::IconSet) -> Self {
        match s {
            dodai_core::IconSet::Stroked => Pack::Stroked,
            dodai_core::IconSet::Symbols => Pack::Material,
            dodai_core::IconSet::Bold => Pack::Bold,
        }
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

/// Stroke width of [`Glyph::Stroked`] on the 24 grid.
pub const STROKE: f32 = 1.75;

/// Resolve `id` for `pack`. `filled` picks the filled form where the pack has one.
pub fn glyph(id: IconId, pack: Pack, filled: bool) -> Option<Glyph> {
    match pack {
        Pack::Stroked => stroked(id, filled),
        Pack::Material => material(id, filled),
        Pack::Bold => bold(id, filled),
    }
}

/// Resolve `id` for a design's declared [`IconSet`](dodai_core::IconSet).
pub fn glyph_for(id: IconId, set: dodai_core::IconSet, filled: bool) -> Option<Glyph> {
    glyph(id, set.into(), filled)
}

fn filled_glyph(icon: &dodai_material_icons::Icon) -> Glyph {
    Glyph::Filled(icon.polygons())
}

/// Phosphor's drawing, as is.
fn phosphor(id: IconId, filled: bool) -> Option<&'static dodai_phosphor_icons::Icon> {
    dodai_phosphor_icons::get(id, filled)
}

/// Cyberpunk: the hand-drawn icon, else Phosphor faceted.
#[allow(unused_variables)]
fn stroked(id: IconId, filled: bool) -> Option<Glyph> {
    #[cfg(feature = "cyberpunk")]
    if let Some(s) = cyber_shapes(id, None) {
        return Some(Glyph::Stroked(s));
    }
    phosphor(id, filled).map(|i| Glyph::Filled(i.faceted()))
}

/// The hand-drawn shapes of `id` (its own name, a synonym, or the Material spelling it mirrors),
/// with an optional derived [`Modifier`].
#[cfg(feature = "cyberpunk")]
pub fn cyber_shapes(id: IconId, modifier: Option<Modifier>) -> Option<Vec<(Points, bool)>> {
    use dodai_cyberpunk_icons as c;
    let key = id.name();
    let kebab = key.replace('_', "-");
    let alias = &dodai_material_icons::alias_of;
    let shapes = c::lookup_with(&kebab, alias).or_else(|| c::lookup_with(key, alias))?;
    Some(match modifier {
        Some(m) => m.apply(shapes),
        None => shapes,
    })
}

/// Material 3: the Material Symbols icon, else Phosphor as is.
fn material(id: IconId, filled: bool) -> Option<Glyph> {
    #[cfg(feature = "material")]
    if let Some(i) = dodai_material_icons::outlined::get_id(id, filled) {
        return Some(filled_glyph(i));
    }
    phosphor(id, filled).map(filled_glyph)
}

/// Neo-Brutalism: bold Phosphor.
#[allow(unused_variables)]
fn bold(id: IconId, filled: bool) -> Option<Glyph> {
    #[cfg(feature = "neobrutalism")]
    if let Some(i) = dodai_neobrutalism_icons::get(id, filled) {
        return Some(filled_glyph(i));
    }
    phosphor(id, filled).map(filled_glyph)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ink(g: &Glyph, px: u32) -> usize {
        g.coverage(px).iter().filter(|&&a| a > 127).count()
    }

    const PACKS: [Pack; 3] = [Pack::Stroked, Pack::Material, Pack::Bold];

    /// Every id of the shared vocabulary draws in every pack: that is what the type promises.
    #[test]
    fn every_id_draws_in_every_pack() {
        for pack in PACKS {
            let missing: Vec<_> = IconId::ALL
                .iter()
                .filter(|&&id| glyph(id, pack, false).is_none())
                .collect();
            assert!(missing.is_empty(), "{pack:?}: {missing:?}");
        }
    }

    #[test]
    fn every_pack_draws_ink() {
        for pack in PACKS {
            for id in [
                IconId::Home,
                IconId::Search,
                IconId::Heart,
                IconId::Settings,
            ] {
                let g = glyph(id, pack, false).unwrap_or_else(|| panic!("{id:?}"));
                assert!(ink(&g, 24) > 20, "{id:?} drew nothing in {pack:?}");
                assert!(g.svg(24, "#fff").contains("<path"));
            }
        }
    }

    #[cfg(all(feature = "cyberpunk", feature = "material"))]
    #[test]
    fn pack_picks_the_style() {
        assert!(matches!(
            glyph(IconId::Home, Pack::Stroked, false),
            Some(Glyph::Stroked(_))
        ));
        assert!(matches!(
            glyph(IconId::Home, Pack::Material, false),
            Some(Glyph::Filled(_))
        ));
        // A Phosphor-only name is faceted on Cyberpunk, so it is a fill there too.
        assert!(matches!(
            glyph(IconId::Acorn, Pack::Stroked, false),
            Some(Glyph::Filled(_))
        ));
    }

    #[cfg(feature = "cyberpunk")]
    #[test]
    fn modifiers_derive_variants_of_drawn_icons() {
        let plain = cyber_shapes(IconId::Heart, None).unwrap();
        for m in Modifier::ALL {
            let v = cyber_shapes(IconId::Heart, Some(m)).unwrap();
            assert_ne!(v, plain, "{m:?}");
        }
    }

    #[cfg(feature = "material")]
    #[test]
    fn filled_variant_has_more_ink() {
        let line = glyph(IconId::Heart, Pack::Material, false).unwrap();
        let fill = glyph(IconId::Heart, Pack::Material, true).unwrap();
        assert!(ink(&fill, 32) > ink(&line, 32));
    }

    #[test]
    fn holes_stay_empty() {
        // A ring: the centre is empty, the edge inked.
        let c = glyph(IconId::Circle, Pack::Material, false)
            .unwrap()
            .coverage(24);
        assert_eq!(c[12 * 24 + 12], 0);
    }
}
