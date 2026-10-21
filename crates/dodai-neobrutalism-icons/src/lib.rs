//! The icon pack of the Dodai Neo-Brutalism design system: chunky, heavy-stroked glyphs.
//!
//! Every icon is [Phosphor Icons](https://phosphoricons.com) (MIT): the `bold` weight is the
//! outline icon and the `fill` weight is its `-fill` variant. Names follow the Dodai /
//! Material spelling (`home`, `arrow_back`, `home-fill`), so a theme can swap packs without
//! renaming anything. Curves are pre-flattened to straight segments on the Material pack's
//! `0 -960 960 960` grid, so the same renderer draws both.
//!
//! ```
//! let home = dodai_neobrutalism_icons::get("home").unwrap();
//! assert!(!home.polygons().is_empty());
//! assert!(dodai_neobrutalism_icons::get("home-fill").is_some());
//! ```
//!
//! The whole Phosphor set is here: the Dodai / Material spellings above, plus every other
//! Phosphor icon under its own name (`acorn`, `address-book`, `airplane-tilt-fill`), grouped by
//! Phosphor's own [`CATEGORIES`].
//!
//! Icon artwork © Phosphor Icons, MIT (see `LICENSE-MIT`).

mod bold;

pub use bold::ALL;
pub use dodai_material_icons::Icon;
pub use dodai_phosphor_icons::CATEGORIES;

/// Look an icon up by name (e.g. `"home"`, `"home-fill"`).
pub fn get(name: &str) -> Option<&'static Icon> {
    dodai_material_icons::find(ALL, name)
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
        for (_, names) in CATEGORIES {
            for n in *names {
                assert!(seen.insert(*n), "{n} listed twice");
                assert!(get(n).is_some(), "{n} missing");
            }
        }
        assert!(seen.len() > 1400);
    }

    #[test]
    fn outline_and_fill_exist() {
        for n in ["home", "search", "close", "chevron-down"] {
            assert!(get(n).is_some(), "{n}");
        }
        assert!(get("home-fill").is_some());
    }
}
