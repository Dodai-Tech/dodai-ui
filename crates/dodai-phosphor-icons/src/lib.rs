//! The whole [Phosphor Icons](https://phosphoricons.com) set (MIT), `regular` weight plus
//! `-fill`, under Phosphor's own names (`address-book`, `airplane-tilt-fill`).
//!
//! It is the shared icon vocabulary: every Dodai design system can draw every name here,
//! in its own style.
//!
//! ```
//! assert!(dodai_phosphor_icons::get("address-book").is_some());
//! assert!(dodai_phosphor_icons::get("address-book-fill").is_some());
//! ```

mod categories;
mod regular;

pub use categories::CATEGORIES;
pub use dodai_material_icons::Icon;
pub use regular::ALL;

/// Look an icon up by name (`"acorn"`, `"acorn-fill"`).
pub fn get(name: &str) -> Option<&'static Icon> {
    dodai_material_icons::find(ALL, name)
}

/// Every outline name (no `-fill`), in category order.
pub fn names() -> impl Iterator<Item = &'static str> {
    CATEGORIES.iter().flat_map(|(_, n)| n.iter().copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_and_unique() {
        assert!(ALL.windows(2).all(|w| w[0].name < w[1].name));
    }

    #[test]
    fn every_name_is_drawn_once() {
        let mut seen = std::collections::HashSet::new();
        for n in names() {
            assert!(seen.insert(n), "{n} listed twice");
            assert!(get(n).is_some(), "{n} missing");
        }
        assert!(seen.len() > 1400);
    }
}
