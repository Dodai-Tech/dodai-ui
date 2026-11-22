//! Cyberpunk and Neo-Brutalism set their own type weights over the shared scale; the sizes stay
//! the theme's, and body text keeps its weight.

use dodai_ui_core::{themes, TypeRole};

#[test]
fn heavier_headings_over_the_shared_sizes() {
    let mut seen = std::collections::BTreeSet::new();
    for t in themes::all() {
        let name = t.spec.name();
        if !matches!(name, "Cyberpunk" | "Neo-Brutalism") {
            continue;
        }
        seen.insert(name);
        for role in TypeRole::ALL {
            let (own, base) = (t.type_style(role), t.baseline_type_style(role));
            assert_eq!(own.size, base.size, "{name} {role:?} keeps the theme size");
            if role.is_heading() {
                assert!(own.weight > base.weight, "{name} {role:?} is heavier");
            } else {
                assert!(own.weight >= base.weight, "{name} {role:?} is not lighter");
            }
        }
        assert_eq!(t.type_style(TypeRole::BodyMedium).weight, 400);
    }
    assert_eq!(seen.len(), 2, "both designs were checked: {seen:?}");
}
