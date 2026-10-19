//! Cross-design-system checks: every built-in theme must satisfy these.
use dodai::{themes, ControlState, Mode, ShapeKind, Tone};
use std::collections::HashSet;

#[test]
fn ids_unique() {
    let ids: HashSet<_> = themes::all().iter().map(|t| t.id()).collect();
    assert_eq!(ids.len(), themes::all().len());
}

#[test]
fn text_is_readable_everywhere() {
    for t in themes::all() {
        let p = &t.palette;
        assert!(p.text.contrast(p.background) >= 7.0, "{}: text/bg", t.id());
        assert!(
            p.text.contrast(p.surface) >= 7.0,
            "{}: text/surface",
            t.id()
        );
        assert!(
            p.on_primary.contrast(p.primary) >= 4.5,
            "{}: on_primary",
            t.id()
        );
        assert!(
            p.on_secondary.contrast(p.secondary) >= 4.5,
            "{}: on_secondary",
            t.id()
        );
        assert!(
            p.text_muted.contrast(p.background) >= 4.5,
            "{}: muted",
            t.id()
        );
    }
}

#[test]
fn component_tokens_resolve_for_every_theme() {
    for t in themes::all() {
        assert!(
            t.tag(Tone::Info, true).foreground.contrast(t.palette.info) >= 3.0,
            "{}",
            t.id()
        );
        assert!(t.tab(true, false).indicator.is_some());
        assert_ne!(
            t.toggle(true, ControlState::Idle).track,
            t.toggle(false, ControlState::Idle).track
        );
        assert_eq!(
            t.list_item(true, false).marker.is_some(),
            t.shape.kind() != ShapeKind::Rounded,
            "{}: accent bar on chamfered themes only",
            t.id()
        );
    }
}

#[test]
fn outline_follows_shape() {
    for t in themes::all() {
        let o = t.shape.outline(100.0, 50.0);
        match t.shape.kind() {
            ShapeKind::Chamfered => assert!(o.len() > 4, "{}", t.id()),
            _ => assert_eq!(o.len(), 4, "{}", t.id()),
        }
    }
}

#[cfg(feature = "cyberpunk")]
#[test]
fn cyberpunk_has_light_and_dark() {
    let fam: Vec<_> = themes::all()
        .into_iter()
        .filter(|t| t.meta.family == "cyberpunk")
        .collect();
    assert_eq!(fam.iter().filter(|t| t.meta.mode == Mode::Dark).count(), 3);
    let light: Vec<_> = fam.iter().filter(|t| t.meta.mode == Mode::Light).collect();
    assert_eq!(light.len(), 3);
    for t in light {
        let p = &t.palette;
        assert!(p.link.contrast(p.background) >= 4.5, "{}: link", t.id());
        assert!(p.background.luminance() > 0.5, "{}: paper", t.id());
        assert_eq!(t.shape.kind(), themes::cyberpunk::night_city().shape.kind());
    }
}

#[cfg(all(feature = "cyberpunk", feature = "material"))]
#[test]
fn lookup_and_switcher_families() {
    assert!(themes::by_id("cyberpunk").is_some());
    assert!(themes::by_id("missing").is_none());
    let mut s = dodai::switcher();
    assert_eq!(s.families(), vec!["cyberpunk", "material"]);
    assert_eq!(s.family_indices("cyberpunk").len(), 6);
    assert_eq!(s.family_indices("material").len(), 2);
    assert!(s.select_family("material"));
    assert_eq!(s.current().meta.family, "material");
}

#[test]
fn every_theme_exports_dtcg() {
    let all = themes::all();
    let j = dodai::export::design_tokens_json(&all);
    for t in &all {
        assert!(
            j.contains(&format!("\"{}\":{{", t.id())),
            "missing {}",
            t.id()
        );
    }
}

#[cfg(all(feature = "cyberpunk", feature = "icons"))]
#[test]
fn design_systems_carry_their_icons() {
    assert!(!dodai::cyberpunk::icons::ALL.is_empty());
}

#[cfg(all(feature = "material", feature = "icons"))]
#[test]
fn material_icons_are_reachable() {
    assert!(dodai::material::icons::outlined::get("home").is_some());
}
