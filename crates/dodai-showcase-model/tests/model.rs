use std::collections::HashMap;

use dodai_core::ControlColour;
use dodai_showcase_model::{
    b, sections, Action, Bind, Ctx, ModePref, Node, Showcase, Value, Values, CATEGORIES,
};

fn build_all(sc: &Showcase) -> Vec<(&'static str, Node)> {
    let theme = sc.theme();
    let ctx = Ctx {
        theme: &theme,
        values: &sc.values,
    };
    sections()
        .iter()
        .map(|s| (s.title, (s.build)(&ctx)))
        .collect()
}

#[test]
fn titles_are_unique_and_categories_known() {
    let mut seen = std::collections::HashSet::new();
    for s in sections() {
        assert!(seen.insert(s.title), "duplicate title {}", s.title);
        assert!(
            CATEGORIES.contains(&s.category),
            "{} has unknown category",
            s.title
        );
    }
    for c in CATEGORIES {
        assert!(
            sections().iter().any(|s| s.category == c),
            "{c} has no section"
        );
    }
}

#[test]
fn sections_are_grouped_in_category_order() {
    let order: Vec<usize> = sections()
        .iter()
        .map(|s| CATEGORIES.iter().position(|c| *c == s.category).unwrap())
        .collect();
    assert!(
        order.windows(2).all(|w| w[0] <= w[1]),
        "sections not grouped by category"
    );
}

#[test]
fn every_section_builds_under_every_theme_and_is_not_empty() {
    let mut sc = Showcase::new();
    let n = sc.designs().len();
    for d in 0..n {
        sc.update(Action::Design(d));
        for v in 0..sc.variants().len() {
            sc.update(Action::Variant(v));
            for (title, node) in build_all(&sc) {
                let mut count = 0;
                node.walk(&mut |_| count += 1);
                assert!(
                    count > 1 || !matches!(node, Node::Stack(_)),
                    "{title} is empty"
                );
            }
        }
    }
}

#[test]
fn bind_keys_keep_one_type() {
    let sc = Showcase::new();
    let mut kinds: HashMap<String, &'static str> = HashMap::new();
    let mut note = |key: &str, kind: &'static str| {
        let prev = kinds.insert(key.to_string(), kind);
        assert!(
            prev.is_none() || prev == Some(kind),
            "key {key} used as {prev:?} and {kind}"
        );
    };
    for (_, node) in build_all(&sc) {
        node.walk(&mut |n| match n {
            Node::Choice(c) | Node::Tabs(c) => note(&c.bind.key, "index"),
            Node::Toggle(t) => note(&t.bind.key, "bool"),
            Node::Slider(s) => note(&s.bind.key, "num"),
            Node::Knob { bind, .. } => note(&bind.key, "num"),
            Node::Input(i) => note(&i.bind.key, "text"),
            Node::Chip { bind, .. } => note(&bind.key, "bool"),
            Node::Stepper(b) => note(&b.key, "int"),
            Node::Pagination { bind, .. } => note(&bind.key, "index"),
            Node::Button(b) => {
                if let Some(k) = &b.counter {
                    note(k, "index");
                }
            }
            _ => {}
        });
    }
}

#[test]
fn radio_groups_have_one_default_and_contiguous_indices() {
    let sc = Showcase::new();
    for (title, node) in build_all(&sc) {
        let mut groups: HashMap<String, Vec<(usize, bool)>> = HashMap::new();
        node.walk(&mut |n| {
            if let Node::Toggle(t) = n {
                if let Some((g, i)) = &t.radio {
                    groups
                        .entry(g.clone())
                        .or_default()
                        .push((*i, t.bind.default));
                }
            }
        });
        for (g, mut m) in groups {
            m.sort();
            assert!(
                m.iter().enumerate().all(|(k, (i, _))| k == *i),
                "{title}/{g} indices"
            );
            assert_eq!(
                m.iter().filter(|(_, d)| *d).count(),
                1,
                "{title}/{g} defaults"
            );
        }
    }
}

#[test]
fn choices_and_pagination_defaults_are_in_range() {
    let sc = Showcase::new();
    for (title, node) in build_all(&sc) {
        node.walk(&mut |n| match n {
            Node::Choice(c) | Node::Tabs(c) => {
                assert!(c.bind.default < c.labels.len(), "{title}: {}", c.bind.key)
            }
            Node::Pagination { pages, bind } => assert!(bind.default < *pages, "{title}"),
            Node::Steps { steps, current } => assert!(current < &steps.len(), "{title}"),
            Node::Slider(s) => assert!(
                (s.min..=s.max).contains(&s.bind.default) && s.min < s.max,
                "{title}: {}",
                s.bind.key
            ),
            _ => {}
        });
    }
}

#[test]
fn values_fall_back_to_the_default_and_hold_what_was_set() {
    let mut v = Values::default();
    let flag = Bind::new("a", true);
    assert!(v.flag(&flag));
    v.set("a", Value::Bool(false));
    assert!(!v.flag(&flag));
    // A value of the wrong type is ignored, not a panic.
    v.set("a", Value::Num(1.0));
    assert!(v.flag(&flag));
}

#[test]
fn switching_design_changes_family_and_bumps_revision() {
    let mut sc = Showcase::new();
    let rev = sc.revision();
    let first = sc.theme().meta.family;
    let other = (0..sc.designs().len())
        .find(|&i| sc.designs()[i] != first)
        .unwrap();
    sc.update(Action::Design(other));
    assert_ne!(sc.theme().meta.family, first);
    assert_eq!(sc.design_index(), other);
    assert!(sc.revision() > rev);
    // An out-of-range design is a no-op.
    let rev = sc.revision();
    sc.update(Action::Design(999));
    assert_eq!(sc.revision(), rev);
}

#[test]
fn variant_picks_a_theme_of_the_design_and_follows_its_mode() {
    let mut sc = Showcase::new();
    let family = sc.theme().meta.family;
    for i in 0..sc.variants().len() {
        sc.update(Action::Variant(i));
        assert_eq!(sc.theme().meta.family, family);
        assert_eq!(sc.variant_index(), i);
        if i == 0 {
            assert_eq!(sc.mode(), ModePref::System);
            continue;
        }
        let want = match sc.theme().meta.mode {
            dodai_core::Mode::Light => ModePref::Light,
            dodai_core::Mode::Dark => ModePref::Dark,
        };
        assert_eq!(sc.mode(), want);
    }
    sc.update(Action::Variant(999));
    assert_eq!(sc.variant_index(), sc.variants().len() - 1);
}

#[test]
fn mode_preference_selects_the_matching_theme_when_the_design_has_one() {
    let mut sc = Showcase::new();
    let material = sc.designs().iter().position(|d| *d == "material").unwrap();
    sc.update(Action::Design(material));
    sc.update(Action::Mode(ModePref::Light));
    assert_eq!(sc.theme().meta.mode, dodai_core::Mode::Light);
    sc.update(Action::Mode(ModePref::Dark));
    assert_eq!(sc.theme().meta.mode, dodai_core::Mode::Dark);
    // System follows the OS flag.
    sc.update(Action::Mode(ModePref::System));
    sc.set_system_dark(false);
    assert_eq!(sc.theme().meta.mode, dodai_core::Mode::Light);
    sc.set_system_dark(true);
    assert_eq!(sc.theme().meta.mode, dodai_core::Mode::Dark);
}

#[test]
fn system_dark_is_ignored_unless_following_the_system() {
    let mut sc = Showcase::new();
    sc.update(Action::Mode(ModePref::Light));
    let rev = sc.revision();
    sc.set_system_dark(false);
    assert_eq!(sc.revision(), rev);
}

#[test]
fn control_colour_retints_primary_and_falls_back_when_not_offered() {
    let mut sc = Showcase::new();
    let base = sc.theme().palette.primary;
    let offered = sc.colours();
    assert_eq!(offered[0], ControlColour::Design);
    if let Some(&other) = offered.get(1) {
        sc.update(Action::Colour(other));
        assert_ne!(sc.theme().palette.primary, base);
        assert_eq!(sc.colour(), other);
    }
    // A choice the theme does not offer is shown as the design's own.
    let missing = ControlColour::ALL
        .into_iter()
        .find(|c| !offered.contains(c));
    if let Some(m) = missing {
        sc.update(Action::Colour(m));
        assert_eq!(sc.colour(), ControlColour::Design);
    }
}

#[test]
fn filter_narrows_sections_and_counts() {
    let mut sc = Showcase::new();
    let all = sc.visible().len();
    let theme = sc.theme();
    let shown = sections().iter().filter(|s| (s.shown)(&theme)).count();
    assert_eq!(all, shown);
    assert_eq!(sc.count(None), all);
    sc.update(Action::Filter("  BUTTON ".into()));
    let hits = sc.visible();
    assert!(!hits.is_empty() && hits.len() < all);
    assert!(hits
        .iter()
        .all(|s| s.title.to_lowercase().contains("button")));
    assert_eq!(sc.count(None), hits.len());
    sc.update(Action::Filter("zzz-nothing".into()));
    assert!(sc.visible().is_empty());
}

#[test]
fn category_selection_limits_sections_and_rejects_unknown() {
    let mut sc = Showcase::new();
    sc.update(Action::Category(Some(1)));
    assert!(sc.visible().iter().all(|s| s.category == CATEGORIES[1]));
    assert_eq!(sc.visible().len(), sc.count(Some(1)));
    sc.update(Action::Category(Some(99)));
    assert_eq!(sc.category(), None);
}

#[test]
fn cycling_themes_wraps_and_bumps() {
    let mut sc = Showcase::new();
    let n = dodai_core::switcher().themes().len();
    let first = sc.theme().meta.id;
    for _ in 0..n {
        sc.update(Action::CycleTheme(true));
    }
    assert_eq!(sc.theme().meta.id, first);
    sc.update(Action::CycleTheme(false));
    assert_ne!(sc.theme().meta.id, first);
}

#[test]
fn press_counts_and_set_stores() {
    let mut sc = Showcase::new();
    for _ in 0..3 {
        sc.update(Action::Press("k".into()));
    }
    assert_eq!(sc.values.index_of("k", 0), 3);
    sc.update(Action::Set("t".into(), Value::Text("x".into())));
    assert_eq!(sc.values.text(&Bind::new("t", String::new())), "x");
}

#[test]
fn speed_is_clamped() {
    let mut sc = Showcase::new();
    sc.update(Action::Speed(99));
    assert_eq!(sc.speed(), *dodai_showcase_model::SPEEDS.last().unwrap());
}

#[test]
fn builders_compose() {
    let n = b::row(vec![b::button("x", dodai_core::ButtonVariant::Primary)
        .loading()
        .node()]);
    let mut c = 0;
    n.walk(&mut |_| c += 1);
    assert_eq!(c, 2);
}

#[test]
fn select_id_finds_a_theme_and_follows_its_mode() {
    let mut sc = Showcase::new();
    let rev = sc.revision();
    assert!(sc.select_id("material-dark"));
    assert_eq!(sc.theme().meta.id, "material-dark");
    assert_eq!(sc.mode(), ModePref::Dark);
    assert!(sc.revision() > rev);
    let rev = sc.revision();
    assert!(!sc.select_id("no-such-theme"));
    assert_eq!(sc.theme().meta.id, "material-dark");
    assert_eq!(sc.revision(), rev);
}

#[test]
fn design_names_line_up_with_designs_and_read_like_names() {
    let sc = Showcase::new();
    let (ids, names) = (sc.designs(), sc.design_names());
    assert_eq!(ids.len(), names.len());
    for (id, name) in ids.iter().zip(&names) {
        assert!(
            name.chars().next().unwrap().is_uppercase(),
            "{id} -> {name}"
        );
    }
    assert!(names.contains(&"Material 3"));
}

#[test]
fn configured_opens_on_the_given_page() {
    let sc = Showcase::new().configured(Some("material-dark"), Some(2), Some("tab"));
    assert_eq!(sc.theme().meta.id, "material-dark");
    assert_eq!(sc.category(), Some(1));
    assert_eq!(sc.filter(), "tab");
    // Zero is "all"; unknown themes and out-of-range pages change nothing.
    let sc = Showcase::new().configured(Some("nope"), Some(0), None);
    assert_eq!(sc.category(), None);
    assert_eq!(sc.theme().meta.id, Showcase::new().theme().meta.id);
    let sc = Showcase::new().configured(None, Some(999), None);
    assert_eq!(sc.category(), None);
}

#[test]
fn walk_reaches_through_scroll_areas_and_popups() {
    let n = b::scroll(100.0, b::col(vec![b::body("inside")]));
    let mut texts = 0;
    n.walk(&mut |x| texts += matches!(x, Node::Text { .. }) as usize);
    assert_eq!(texts, 1);
    let p = Node::Popup(Box::new(b::body("pop")));
    let mut seen = 0;
    p.walk(&mut |_| seen += 1);
    assert_eq!(seen, 2);
}

#[test]
fn knobs_stay_inside_their_range_and_vertical_sliders_exist() {
    let sc = Showcase::new();
    let (mut knobs, mut vertical, mut scrolls) = (0, 0, 0);
    for (_, node) in build_all(&sc) {
        node.walk(&mut |n| match n {
            Node::Knob {
                bind,
                min,
                max,
                size,
                ..
            } => {
                knobs += 1;
                assert!(min < max && *size > 0.0 && (*min..=*max).contains(&bind.default));
            }
            Node::Slider(s) if s.vertical => vertical += 1,
            Node::Scroll { height, .. } => {
                scrolls += 1;
                assert!(*height > 0.0);
            }
            _ => {}
        });
    }
    assert!(knobs >= 3 && vertical >= 3 && scrolls >= 1);
}

#[test]
fn design_only_sections_show_for_their_design_alone() {
    let mut sc = Showcase::new();
    let owned = |sc: &Showcase| {
        sc.visible()
            .iter()
            .any(|s| s.title == "Button groups · Split button")
    };
    for (i, d) in sc.designs().into_iter().enumerate() {
        sc.update(Action::Design(i));
        assert_eq!(owned(&sc), d == "material", "{d}");
    }
}
