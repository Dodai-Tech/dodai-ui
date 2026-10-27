//! The icon packs against the design systems that draw them.
#![cfg(all(feature = "cyberpunk", feature = "material", feature = "neobrutalism"))]

use dodai_core::themes;
use dodai_icons::{glyph_for_theme, IconId};

#[test]
fn cyberpunk_pack_is_not_empty() {
    assert!(!dodai_icons::cyberpunk::ALL.is_empty());
}

#[test]
fn material_symbols_are_reachable() {
    use dodai_icons::material::MaterialSymbol;
    assert!(dodai_icons::material::outlined::get(MaterialSymbol::Home).is_some());
}

/// Every design draws every [`IconId`] of the shared vocabulary: a design uses its own pack and a
/// fixed fallback, so naming an icon never yields a hole.
#[test]
fn every_design_draws_every_icon() {
    assert!(IconId::ALL.len() > 1500);
    for sys in themes::systems() {
        let t = (sys.themes)().into_iter().next().expect("a theme");
        for filled in [false, true] {
            let missing: Vec<_> = IconId::ALL
                .iter()
                .filter(|&&id| glyph_for_theme(id, &t, filled).is_none())
                .collect();
            assert!(
                missing.is_empty(),
                "{} filled={filled}: {missing:?}",
                t.id()
            );
        }
    }
}
