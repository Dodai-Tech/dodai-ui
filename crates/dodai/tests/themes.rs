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
    let n = 3 + dodai_cyberpunk::PRESETS.len();
    assert_eq!(fam.iter().filter(|t| t.meta.mode == Mode::Dark).count(), n);
    let light: Vec<_> = fam.iter().filter(|t| t.meta.mode == Mode::Light).collect();
    assert_eq!(light.len(), n);
    for t in light {
        let p = &t.palette;
        assert!(p.link.contrast(p.background) >= 4.5, "{}: link", t.id());
        assert!(p.background.luminance() > 0.5, "{}: paper", t.id());
        assert_eq!(t.shape.kind(), themes::cyberpunk::yellow().shape.kind());
    }
}

#[test]
fn lookup_and_switcher_families() {
    assert!(themes::by_id("missing").is_none());
    let mut s = dodai::switcher();
    let systems = themes::systems();
    assert_eq!(
        s.families(),
        systems.iter().map(|d| d.family).collect::<Vec<_>>()
    );
    for d in &systems {
        assert!(themes::by_id((d.themes)()[0].id()).is_some());
        assert_eq!(s.family_indices(d.family).len(), (d.themes)().len());
        assert!(s.select_family(d.family));
        assert_eq!(s.current().meta.family, d.family);
    }
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

#[test]
fn extended_roles_are_complete_unique_and_readable() {
    for t in themes::all() {
        let r = &t.palette.roles;
        let list = r.list();
        let names: HashSet<_> = list.iter().map(|(n, _)| *n).collect();
        assert_eq!(names.len(), list.len(), "{}: duplicate role name", t.id());
        assert_ne!(
            *r,
            dodai::Roles::placeholder(),
            "{}: roles never set",
            t.id()
        );
        for (fg, bg, what) in [
            (
                r.on_surface,
                r.surface_container_lowest,
                "on-surface/lowest",
            ),
            (r.on_surface, r.surface_container, "on-surface/container"),
            (
                r.on_surface,
                r.surface_container_highest,
                "on-surface/highest",
            ),
            (
                r.on_surface_variant,
                r.surface_container,
                "variant/container",
            ),
            (
                r.on_primary_container,
                r.primary_container,
                "on-primary-container",
            ),
            (
                r.on_secondary_container,
                r.secondary_container,
                "on-secondary-container",
            ),
            (
                r.on_tertiary_container,
                r.tertiary_container,
                "on-tertiary-container",
            ),
            (
                r.on_error_container,
                r.error_container,
                "on-error-container",
            ),
            (r.on_tertiary, r.tertiary, "on-tertiary"),
            (r.on_error, t.palette.danger, "on-error"),
            (r.inverse_on_surface, r.inverse_surface, "inverse"),
        ] {
            assert!(
                fg.contrast(bg) >= 4.5,
                "{}: {what} {:.2}",
                t.id(),
                fg.contrast(bg)
            );
        }
    }
}

#[test]
fn container_surfaces_step_in_order() {
    // Material: each container is further from the page than the one below it.
    for t in themes::all()
        .into_iter()
        .filter(|t| t.meta.family == "material")
    {
        let r = &t.palette.roles;
        let lum = |c: dodai::Color| c.luminance();
        let steps = [
            r.surface_container_low,
            r.surface_container,
            r.surface_container_high,
            r.surface_container_highest,
        ];
        let dark = t.meta.mode == Mode::Dark;
        for w in steps.windows(2) {
            assert!(
                if dark {
                    lum(w[1]) > lum(w[0])
                } else {
                    lum(w[1]) < lum(w[0])
                },
                "{}: containers out of order",
                t.id()
            );
        }
    }
}

#[test]
fn surface_tokens_resolve_for_every_theme() {
    use dodai::{CardVariant, StepState};
    for t in themes::all() {
        let p = &t.palette;
        // Text on every surface that has a fill stays readable.
        let surfaces = [
            ("card/elevated", t.card(CardVariant::Elevated)),
            ("card/filled", t.card(CardVariant::Filled)),
            ("chip/selected", t.chip(true, false)),
            ("badge", t.badge(Tone::Danger)),
            ("alert/banner", t.alert(Tone::Warning, true)),
            ("alert/tinted", t.alert(Tone::Info, false)),
            ("dialog", t.dialog()),
            ("sheet", t.sheet()),
            ("popup", t.popup()),
        ];
        for (what, s) in surfaces {
            // A tinted fill is translucent, so measure against the page.
            let bg = s.background.unwrap_or(p.background);
            let bg = if bg.a < 255 { p.background } else { bg };
            assert!(
                s.foreground.contrast(bg) >= 4.5,
                "{}: {what} {:.2}",
                t.id(),
                s.foreground.contrast(bg)
            );
        }
        assert!(t.card(CardVariant::Outlined).background.is_none());
        assert!(t.card(CardVariant::Outlined).border.is_some());
        assert!(t.scrim().a < 255 && t.scrim().a > 0, "{}: scrim", t.id());
        // Stepper: done is filled, todo is hollow, only the current node glows.
        assert!(t.step(StepState::Done).fill.is_some());
        assert!(t.step(StepState::Todo).fill.is_none());
        assert!(!t.step(StepState::Done).glow, "{}", t.id());
        assert_eq!(StepState::of(0, 1), StepState::Done);
        assert_eq!(StepState::of(1, 1), StepState::Current);
        assert_eq!(StepState::of(2, 1), StepState::Todo);
    }
}

#[test]
fn cards_and_dividers_resolve_for_every_theme() {
    use dodai::CardVariant;
    for t in themes::all() {
        let f = t.card(CardVariant::Filled);
        assert!(f.background.is_some(), "{}", t.id());
        assert!(t.divider().a > 0, "{}", t.id());
    }
}

#[test]
fn effect_colours_are_set() {
    for t in themes::all() {
        let fx = &t.effects;
        assert!(
            fx.shadow.a > 0 && fx.scanline.a > 0 && fx.highlight.a > 0,
            "{}",
            t.id()
        );
    }
}

#[test]
fn every_theme_has_its_own_sane_motion() {
    use std::time::Duration;
    for t in themes::all() {
        let d = t.motion.duration;
        assert!(
            d >= Duration::from_millis(80) && d <= Duration::from_millis(500),
            "{}: {d:?}",
            t.id()
        );
    }
    // Each design system moves in its own way.
    let feels: Vec<_> = themes::systems()
        .iter()
        .map(|d| (d.themes)()[0].motion)
        .collect();
    for (i, a) in feels.iter().enumerate() {
        assert!(feels[i + 1..].iter().all(|b| a != b), "motion repeats");
    }
}

#[test]
fn banners_and_snackbars_are_readable() {
    for t in themes::all() {
        let (sb, action) = t.snackbar();
        let b = t.alert(Tone::Warning, true);
        let (bg, fg) = (b.background.unwrap(), b.foreground);
        assert!(fg.contrast(bg) >= 4.5, "{}: banner text", t.meta.id);
        let (sbg, sfg) = (sb.background.unwrap(), sb.foreground);
        assert!(sfg.contrast(sbg) >= 4.5, "{}: snackbar text", t.meta.id);
        assert!(
            action.contrast(sbg) >= 3.0,
            "{}: snackbar action",
            t.meta.id
        );
    }
}

#[cfg(feature = "cyberpunk")]
#[test]
fn neon_schemes_stay_readable_for_any_accents() {
    use dodai::Color;
    use dodai_cyberpunk::{Backdrop, Neon};
    let seeds = [
        0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0x808080, 0x000000, 0xffffff, 0x3a2a7a,
    ];
    for backdrop in Backdrop::ALL {
        for p in seeds {
            for s in seeds {
                let neon = Neon::new(Color::hex(p), Color::hex(s), Color::hex(0xff2a8a))
                    .backdrop(backdrop);
                for mode in [Mode::Dark, Mode::Light] {
                    let meta = dodai::ThemeMeta {
                        id: "t",
                        name: "t",
                        family: "cyberpunk",
                        description: "",
                        mode,
                    };
                    let t = if mode == Mode::Dark {
                        neon.dark(meta)
                    } else {
                        neon.light(meta)
                    };
                    let pl = &t.palette;
                    let tag = format!("{backdrop:?} {p:06x}/{s:06x} {mode:?}");
                    assert!(pl.text.contrast(pl.background) >= 7.0, "{tag}: text");
                    assert!(
                        pl.on_primary.contrast(pl.primary) >= 4.5,
                        "{tag}: on_primary"
                    );
                    assert!(
                        pl.on_secondary.contrast(pl.secondary) >= 4.5,
                        "{tag}: on_secondary"
                    );
                    assert!(pl.link.contrast(pl.background) >= 4.5, "{tag}: link");
                }
            }
        }
    }
}

/// Every design resolves every icon name the library ships: a design names its own pack, and a
/// name that pack lacks is a hole, not something another design's pack should fill in.
#[cfg(all(feature = "icons", feature = "cyberpunk"))]
#[test]
fn every_design_draws_every_icon() {
    let names: Vec<&str> = dodai::cyberpunk::icons::ALL
        .iter()
        .map(|i| i.name())
        .collect();
    assert!(names.len() > 300);
    for sys in dodai::themes::systems() {
        let t = (sys.themes)().into_iter().next().expect("a theme");
        let missing: Vec<&&str> = names
            .iter()
            .filter(|n| dodai::icons::glyph(n, &t, false).is_none())
            .collect();
        assert!(missing.is_empty(), "{}: {missing:?}", t.id());
    }
}
