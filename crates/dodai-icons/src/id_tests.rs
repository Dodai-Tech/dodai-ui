//! Checks on the generated `IconId` table.

use crate::IconId;

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
