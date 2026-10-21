//! The whole [Phosphor Icons](https://phosphoricons.com) set (MIT), `regular` weight plus the
//! filled form, named by the typed [`IconId`].
//!
//! It is the shared icon vocabulary: every Dodai design system can draw every [`IconId`], in its
//! own style. Filled forms are `get(id, true)`; there are no name strings to get wrong.
//!
//! ```
//! use dodai_phosphor_icons::{get, IconId};
//! assert!(get(IconId::AddressBook, false).is_some());
//! assert!(get(IconId::AddressBook, true).is_some());
//! ```

mod categories;
mod regular;

pub use categories::CATEGORIES;
pub use dodai_icon_id::IconId;
pub use dodai_material_icons::Icon;
#[doc(hidden)]
pub use regular::ALL;

/// Phosphor's drawing of `id`; `filled` picks the filled form, falling back to the outline.
/// `None` when the id is a Dodai name Phosphor has no icon of that name for.
pub fn get(id: IconId, filled: bool) -> Option<&'static Icon> {
    let name = id.name().replace('_', "-");
    let pick = |n: &str| dodai_material_icons::find(ALL, n);
    filled
        .then(|| pick(&format!("{name}-fill")))
        .flatten()
        .or_else(|| pick(&name))
}

/// Every Phosphor icon, in category order.
pub fn ids() -> impl Iterator<Item = IconId> {
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
    fn every_id_is_drawn_once() {
        let mut seen = std::collections::HashSet::new();
        for id in ids() {
            assert!(seen.insert(id), "{id:?} listed twice");
            assert!(get(id, false).is_some(), "{id:?} missing");
            assert!(get(id, true).is_some(), "{id:?} has no filled form");
        }
        assert!(seen.len() > 1400);
    }
}
