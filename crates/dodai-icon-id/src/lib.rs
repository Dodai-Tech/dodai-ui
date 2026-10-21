//! [`IconId`]: the shared, typed icon vocabulary of the Dodai design systems.
//!
//! One enum names every icon any design system can draw: the hand-drawn Cyberpunk names and every
//! [Phosphor](https://phosphoricons.com) name. Pick one in your editor (`IconId::Heart`,
//! `IconId::HouseLine`) and hand it to a toolkit's `Icon`; each design system draws it in its own
//! style. A typo is a compile error, and there are no icon-name strings to keep in sync.
//!
//! This crate holds only the names. The artwork lives in the icon packs, and
//! `dodai-icons` resolves an id to a drawing.
//!
//! ```
//! use dodai_icon_id::IconId;
//! assert_eq!(IconId::ArrowRight.name(), "arrow_right");
//! assert!(IconId::ALL.len() > 1500);
//! ```

mod id;

pub use id::IconId;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_unique_and_round_trip() {
        let mut seen = std::collections::HashSet::new();
        for &i in IconId::ALL {
            assert!(seen.insert(i.name()), "{} twice", i.name());
            assert_eq!(IconId::from_key(i.name()), Some(i));
        }
        assert_eq!(IconId::from_key("definitely_not_an_icon"), None);
    }

    #[test]
    fn sorted_by_name() {
        assert!(IconId::ALL.windows(2).all(|w| w[0].name() < w[1].name()));
    }
}
