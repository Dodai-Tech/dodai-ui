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
//! use dodai_ui_icons::{glyph, IconId, Pack};
//! let g = glyph(IconId::Heart, Pack::Material, false).unwrap();
//! assert!(g.svg(24, "currentColor").starts_with("<svg"));
//! assert_eq!(g.coverage(24).len(), 24 * 24);
//! ```

#[cfg(feature = "neobrutalism")]
mod bold;
#[cfg(feature = "cyberpunk")]
pub mod cyberpunk;
mod id;
#[cfg(test)]
mod id_tests;
pub mod material;
mod pack;
mod phosphor;
pub mod symbol;

pub use id::IconId;
pub use pack::{Glyph, IconPack, PackChain, Points, StrokeStyle, STROKE};
/// The icon set grouped into categories (for browsers and pickers).
pub use phosphor::CATEGORIES;

/// Cyberpunk's polyline modifiers (`Off`, `Add`, `Circle`, ...), derived from a drawn icon.
#[cfg(feature = "cyberpunk")]
pub use cyberpunk::Modifier;

/// Which design's pack draws the icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pack {
    /// Cyberpunk: hand-drawn straight segments and chamfers.
    Stroked,
    /// Material 3: Material Symbols.
    Material,
    /// Neo-Brutalism: bold Phosphor.
    Bold,
}

impl Pack {
    /// How this design's pack strokes the line symbols.
    pub const fn symbol_style(self) -> StrokeStyle {
        match self {
            Pack::Stroked => StrokeStyle::SQUARE,
            Pack::Material => StrokeStyle::ROUND,
            Pack::Bold => StrokeStyle::BOLD,
        }
    }
}

#[cfg(feature = "theme")]
impl From<dodai_ui_core::IconSet> for Pack {
    fn from(s: dodai_ui_core::IconSet) -> Self {
        match s {
            dodai_ui_core::IconSet::Stroked => Pack::Stroked,
            dodai_ui_core::IconSet::Symbols => Pack::Material,
            dodai_ui_core::IconSet::Bold => Pack::Bold,
        }
    }
}

/// The ordered packs of a design: first hit wins, the last covers every id.
pub fn chain(pack: Pack) -> &'static PackChain {
    static CHAINS: std::sync::OnceLock<[PackChain; 3]> = std::sync::OnceLock::new();
    let c = CHAINS.get_or_init(|| {
        let phosphor = || Box::new(PhosphorPack { faceted: false }) as Box<dyn IconPack>;
        [
            PackChain::new(vec![
                #[cfg(feature = "cyberpunk")]
                Box::new(CyberpunkPack),
                Box::new(PhosphorPack { faceted: true }),
            ]),
            PackChain::new(vec![
                #[cfg(feature = "material")]
                Box::new(MaterialPack),
                phosphor(),
            ]),
            PackChain::new(vec![
                #[cfg(feature = "neobrutalism")]
                Box::new(BoldPack),
                phosphor(),
            ]),
        ]
    });
    &c[pack as usize]
}

/// Resolve `id` for `pack`. `filled` picks the filled form where the pack has one.
pub fn glyph(id: IconId, pack: Pack, filled: bool) -> Option<Glyph> {
    // Resolving parses the SVG path (and facets it for angular designs); toolkits ask on every
    // frame, so each icon is resolved once.
    type Key = (IconId, usize, bool);
    static CACHE: std::sync::LazyLock<
        std::sync::Mutex<std::collections::HashMap<Key, Option<Glyph>>>,
    > = std::sync::LazyLock::new(Default::default);
    let key = (id, pack as usize, filled);
    if let Some(hit) = CACHE.lock().ok().and_then(|c| c.get(&key).cloned()) {
        return hit;
    }
    let out = chain(pack).glyph(id, filled);
    if let Ok(mut c) = CACHE.lock() {
        c.insert(key, out.clone());
    }
    out
}

#[cfg(feature = "theme")]
/// Resolve `id` for a design's declared [`IconSet`](dodai_ui_core::IconSet).
pub fn glyph_for(id: IconId, set: dodai_ui_core::IconSet, filled: bool) -> Option<Glyph> {
    glyph(id, set.into(), filled)
}

#[cfg(feature = "theme")]
/// Resolve `id` for `theme`'s design system (its `Look::icons`).
pub fn glyph_for_theme(id: IconId, theme: &dodai_ui_core::Theme, filled: bool) -> Option<Glyph> {
    glyph_for(id, theme.look().icons, filled)
}

fn filled_glyph(icon: &crate::material::Icon) -> Glyph {
    Glyph::Filled(icon.polygons())
}

/// Phosphor's drawing, plain or faceted into straight segments (Cyberpunk).
struct PhosphorPack {
    faceted: bool,
}

impl IconPack for PhosphorPack {
    fn name(&self) -> &'static str {
        if self.faceted {
            "phosphor-faceted"
        } else {
            "phosphor"
        }
    }
    fn glyph(&self, id: IconId, filled: bool) -> Option<Glyph> {
        phosphor::get(id, filled).map(|i| {
            if self.faceted {
                Glyph::Filled(i.faceted())
            } else {
                filled_glyph(i)
            }
        })
    }
}

/// Cyberpunk's hand-drawn, curve-free icons.
#[cfg(feature = "cyberpunk")]
struct CyberpunkPack;

#[cfg(feature = "cyberpunk")]
impl IconPack for CyberpunkPack {
    fn name(&self) -> &'static str {
        "cyberpunk"
    }
    fn glyph(&self, id: IconId, _: bool) -> Option<Glyph> {
        cyber_shapes(id, None).map(Glyph::Stroked)
    }
}

/// Material Symbols.
#[cfg(feature = "material")]
struct MaterialPack;

#[cfg(feature = "material")]
impl IconPack for MaterialPack {
    fn name(&self) -> &'static str {
        "material-symbols"
    }
    fn glyph(&self, id: IconId, filled: bool) -> Option<Glyph> {
        crate::material::outlined::get_id(id, filled).map(filled_glyph)
    }
}

/// Neo-Brutalism's bold Phosphor.
#[cfg(feature = "neobrutalism")]
struct BoldPack;

#[cfg(feature = "neobrutalism")]
impl IconPack for BoldPack {
    fn name(&self) -> &'static str {
        "phosphor-bold"
    }
    fn glyph(&self, id: IconId, filled: bool) -> Option<Glyph> {
        bold::get(id, filled).map(filled_glyph)
    }
}

/// The hand-drawn shapes of `id` (its own name, a synonym, or the Material spelling it mirrors),
/// with an optional derived [`Modifier`].
#[cfg(feature = "cyberpunk")]
pub fn cyber_shapes(id: IconId, modifier: Option<Modifier>) -> Option<Vec<(Points, bool)>> {
    use crate::cyberpunk as c;
    let key = id.name();
    let kebab = key.replace('_', "-");
    let alias = &crate::material::alias_of;
    let shapes = c::lookup_with(&kebab, alias).or_else(|| c::lookup_with(key, alias))?;
    Some(match modifier {
        Some(m) => m.apply(shapes),
        None => shapes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_pack_strokes_symbols_in_its_own_style() {
        assert_eq!(Pack::Stroked.symbol_style(), StrokeStyle::SQUARE);
        assert_eq!(Pack::Material.symbol_style(), StrokeStyle::ROUND);
        assert_eq!(Pack::Bold.symbol_style(), StrokeStyle::BOLD);
    }

    #[cfg(feature = "material")]
    fn ink(g: &Glyph, px: u32) -> usize {
        g.coverage(px).iter().filter(|&&a| a > 127).count()
    }

    #[cfg(all(feature = "cyberpunk", feature = "material", feature = "neobrutalism"))]
    const PACKS: [Pack; 3] = [Pack::Stroked, Pack::Material, Pack::Bold];

    /// Every id of the shared vocabulary draws in every pack: that is what the type promises.
    #[cfg(all(feature = "cyberpunk", feature = "material", feature = "neobrutalism"))]
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

    #[cfg(all(feature = "cyberpunk", feature = "material", feature = "neobrutalism"))]
    #[test]
    fn chains_end_in_a_pack_that_covers_everything() {
        for pack in PACKS {
            let c = chain(pack);
            assert!(c.missing().is_empty(), "{:?}", c.names());
            assert!(
                c.names().len() >= 2
                    || !cfg!(all(
                        feature = "cyberpunk",
                        feature = "material",
                        feature = "neobrutalism"
                    ))
            );
        }
    }

    #[cfg(all(feature = "cyberpunk", feature = "material", feature = "neobrutalism"))]
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
