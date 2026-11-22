//! The shared drawing source behind the icon vocabulary. Private: not part of the public API.

mod categories;
mod regular;

use crate::material::Icon;
use crate::IconId;
pub use categories::CATEGORIES;
use regular::ALL;

/// Phosphor's drawing of `id`; `filled` picks the filled form, falling back to the outline.
/// `None` when the id is a Dodai name Phosphor has no icon of that name for.
pub(crate) fn get(id: IconId, filled: bool) -> Option<&'static Icon> {
    let name = id.name().replace('_', "-");
    let pick = |n: &str| crate::material::find(ALL, n);
    filled
        .then(|| pick(&format!("{name}-fill")))
        .flatten()
        .or_else(|| pick(&name))
}

/// Every Phosphor icon, in category order.
#[cfg(test)]
pub(crate) fn ids() -> impl Iterator<Item = IconId> {
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
