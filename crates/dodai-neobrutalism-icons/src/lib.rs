//! The icon pack of the Dodai Neo-Brutalism design system: chunky, heavy-stroked glyphs.
//!
//! Every icon is [Phosphor Icons](https://phosphoricons.com) (MIT): the `bold` weight is the
//! outline icon and the `fill` weight is its `-fill` variant. Names follow the Dodai /
//! Material spelling (`home`, `arrow_back`, `home-fill`), so a theme can swap packs without
//! renaming anything. Curves are pre-flattened to straight segments on the Material pack's
//! `0 -960 960 960` grid, so the same renderer draws both.
//!
//! ```
//! use dodai_neobrutalism_icons::{get, IconId};
//! let home = get(IconId::Home, false).unwrap();
//! assert!(!home.polygons().is_empty());
//! assert!(get(IconId::Home, true).is_some());
//! ```
//!
//! The whole Phosphor set is here: the Dodai / Material spellings above, plus every other
//! Phosphor icon under its own name, grouped by Phosphor's own [`CATEGORIES`]. Icons are named by
//! the typed [`IconId`].
//!
//! Icon artwork © Phosphor Icons, MIT (see `LICENSE-MIT`).

mod bold;

#[doc(hidden)]
pub use bold::ALL;
pub use dodai_icon_id::IconId;
pub use dodai_material_icons::Icon;
pub use dodai_phosphor_icons::CATEGORIES;

/// The bold drawing of `id`; `filled` picks the `-fill` weight, falling back to the outline.
/// Dodai names are stored under their Dodai spelling (`arrow_drop_down`), the rest under
/// Phosphor's (`address-book`); both are tried.
pub fn get(id: IconId, filled: bool) -> Option<&'static Icon> {
    let key = id.name();
    let kebab = key.replace('_', "-");
    let pick = |n: &str| {
        filled
            .then(|| dodai_material_icons::find(ALL, &format!("{n}-fill")))
            .flatten()
            .or_else(|| dodai_material_icons::find(ALL, n))
    };
    pick(key).or_else(|| pick(&kebab))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_and_unique() {
        assert!(ALL.windows(2).all(|w| w[0].name < w[1].name));
    }

    #[test]
    fn every_icon_has_geometry() {
        for i in ALL {
            assert!(!i.polygons().is_empty(), "{}", i.name);
            for p in i.polygons() {
                for (x, y) in p {
                    assert!(
                        (-1.0..=25.0).contains(&x) && (-1.0..=25.0).contains(&y),
                        "{}",
                        i.name
                    );
                }
            }
        }
    }

    #[test]
    fn every_phosphor_icon_is_in_one_category_and_drawn() {
        let mut seen = std::collections::HashSet::new();
        for (_, ids) in CATEGORIES {
            for &id in *ids {
                assert!(seen.insert(id), "{id:?} listed twice");
                assert!(get(id, false).is_some(), "{id:?} missing");
            }
        }
        assert!(seen.len() > 1400);
    }

    #[test]
    fn outline_and_fill_exist() {
        for id in [
            IconId::Home,
            IconId::Search,
            IconId::Close,
            IconId::ChevronDown,
        ] {
            assert!(get(id, false).is_some(), "{id:?}");
        }
        assert!(get(IconId::Home, true).unwrap().is_filled());
    }
}
