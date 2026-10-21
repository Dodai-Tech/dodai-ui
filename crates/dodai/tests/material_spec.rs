//! Material 3 conformance: component geometry and colour roles against the M3 spec
//! (m3.material.io component specs / the `md-comp-*` token sets). Every Material theme, light
//! and dark, must satisfy these; no component may look at a theme id.
use dodai::{
    themes, ButtonVariant, CardVariant, ControlState, ShapeKind, ShapeRole, Theme, ToggleKind,
};

fn material() -> Vec<Theme> {
    themes::all()
        .into_iter()
        .filter(|t| t.meta.family == "material")
        .collect()
}

fn radius(t: &Theme, role: ShapeRole) -> f32 {
    t.role_shape(role).radius
}

#[test]
fn there_are_light_and_dark_material_themes() {
    let all = material();
    assert_eq!(all.len(), 2 + 2 * dodai_material::PRESETS.len());
    for m in [dodai_core::Mode::Light, dodai_core::Mode::Dark] {
        assert!(all.iter().filter(|t| t.meta.mode == m).count() > 1);
    }
}

#[test]
fn shape_scale_per_component_matches_the_spec() {
    for t in material() {
        assert_eq!(t.shape.kind(), ShapeKind::Rounded);
        // md.sys.shape: XS 4, S 8, M 12, L 16, XL 28, full.
        let want = [
            (ShapeRole::Button, 999.0),     // buttons: full
            (ShapeRole::IconButton, 999.0), // icon buttons: full
            (ShapeRole::Fab, 16.0),         // FAB: large; extended FAB the same
            (ShapeRole::FabSmall, 12.0),    // small FAB: medium
            (ShapeRole::FabLarge, 28.0),    // large FAB: extra large
            (ShapeRole::Chip, 8.0),         // chips: small
            (ShapeRole::Card, 12.0),        // cards: medium
            (ShapeRole::Dialog, 28.0),      // dialogs: extra large
            (ShapeRole::Menu, 4.0),         // menus: extra small
            (ShapeRole::Input, 4.0),        // text fields: extra small
            (ShapeRole::Snackbar, 4.0),     // snackbar: extra small
            (ShapeRole::Tooltip, 4.0),      // plain tooltip: extra small
            (ShapeRole::RichTooltip, 12.0), // rich tooltip: medium
            (ShapeRole::Sheet, 28.0),       // bottom sheet: extra large (top)
            (ShapeRole::SideSheet, 16.0),   // side sheet: large
            (ShapeRole::Carousel, 28.0),    // carousel item: extra large
            (ShapeRole::SearchBar, 999.0),  // search bar: full
            (ShapeRole::Checkbox, 2.0),     // checkbox container: 2 px
            (ShapeRole::Indicator, 999.0),  // navigation / tab indicators
        ];
        for (role, px) in want {
            let r = radius(&t, role);
            if px >= 999.0 {
                assert!(
                    r >= 999.0,
                    "{}: {role:?} must be a full pill, got {r}",
                    t.id()
                );
            } else {
                assert_eq!(r, px, "{}: {role:?}", t.id());
            }
        }
    }
}

#[test]
fn buttons_use_the_m3_roles_and_state_layers() {
    for t in material() {
        let r = &t.palette.roles;
        let p = &t.palette;
        let b = |v, s| t.button(v, s);
        // Filled / tonal / outlined / text.
        assert_eq!(
            b(ButtonVariant::Primary, ControlState::Idle).background,
            Some(p.primary)
        );
        assert_eq!(
            b(ButtonVariant::Primary, ControlState::Idle).foreground,
            p.on_primary
        );
        assert_eq!(
            b(ButtonVariant::Secondary, ControlState::Idle).background,
            Some(r.secondary_container)
        );
        assert_eq!(
            b(ButtonVariant::Secondary, ControlState::Idle).foreground,
            r.on_secondary_container
        );
        let o = b(ButtonVariant::Outline, ControlState::Idle);
        assert!(
            o.background.is_none()
                && o.border == Some(r.outline)
                && o.foreground == t.legible(p.primary)
        );
        let g = b(ButtonVariant::Ghost, ControlState::Idle);
        assert!(
            g.background.is_none() && g.border.is_none() && g.foreground == t.legible(p.primary)
        );
        // State layers: hover 8 %, press / focus 10 % of the label colour over the container.
        let hover = b(ButtonVariant::Primary, ControlState::Hovered)
            .background
            .unwrap();
        assert_eq!(hover, p.primary.mix(p.on_primary, 0.08));
        let press = b(ButtonVariant::Primary, ControlState::Pressed)
            .background
            .unwrap();
        assert_eq!(press, p.primary.mix(p.on_primary, 0.10));
        assert_eq!(
            b(ButtonVariant::Ghost, ControlState::Hovered).background,
            Some(t.legible(p.primary).with_alpha(0.08))
        );
        // Disabled: container on-surface 12 %, label 38 %.
        let d = b(ButtonVariant::Primary, ControlState::Disabled);
        assert_eq!(d.background, Some(r.on_surface.with_alpha(0.12)));
        assert_eq!(d.foreground, r.on_surface.with_alpha(0.38));
        // Outlined disabled keeps a 12 % outline and no container.
        let od = b(ButtonVariant::Outline, ControlState::Disabled);
        assert!(od.background.is_none());
        assert_eq!(od.border, Some(r.on_surface.with_alpha(0.12)));
    }
}

#[test]
fn elevated_button_is_a_low_container_with_primary_label() {
    for t in material() {
        let r = &t.palette.roles;
        let e = t.elevated_button(ControlState::Idle);
        assert_eq!(e.background, Some(r.surface_container_low));
        assert_eq!(e.foreground, t.palette.primary);
        let d = t.elevated_button(ControlState::Disabled);
        assert_eq!(d.background, Some(r.on_surface.with_alpha(0.12)));
    }
    // Other systems fall back to the secondary button.
    for t in themes::all()
        .into_iter()
        .filter(|t| t.meta.family != "material")
    {
        let (a, b) = (
            t.elevated_button(ControlState::Idle),
            t.button(ButtonVariant::Secondary, ControlState::Idle),
        );
        assert_eq!(a.background, b.background, "{}", t.id());
    }
}

#[test]
fn fab_uses_container_roles() {
    for t in material() {
        let r = &t.palette.roles;
        let f = t.fab(ButtonVariant::Primary, ControlState::Idle);
        assert_eq!(f.background, Some(r.primary_container));
        assert_eq!(f.foreground, r.on_primary_container);
        let s = t.fab(ButtonVariant::Secondary, ControlState::Idle);
        assert_eq!(s.background, Some(r.secondary_container));
    }
}

#[test]
fn surfaces_use_the_m3_containers() {
    for t in material() {
        let r = &t.palette.roles;
        assert_eq!(t.dialog().background, Some(r.surface_container_high));
        assert!(t.dialog().border.is_none());
        assert_eq!(t.popup().background, Some(r.surface_container));
        assert!(t.popup().border.is_none());
        assert_eq!(t.sheet().background, Some(r.surface_container_low));
        assert_eq!(
            t.card(CardVariant::Elevated).background,
            Some(r.surface_container_low)
        );
        assert!(
            t.card(CardVariant::Elevated).border.is_none(),
            "elevated cards have no outline"
        );
        assert_eq!(
            t.card(CardVariant::Filled).background,
            Some(r.surface_container_highest)
        );
        assert_eq!(
            t.card(CardVariant::Outlined).border,
            Some(r.outline_variant)
        );
        assert_eq!(t.divider(), r.outline_variant);
        // Scrim: the `scrim` role at 32 %.
        assert_eq!(t.scrim(), r.scrim.with_alpha(0.32));
        // Snackbar: inverse surface.
        let (s, action) = t.snackbar();
        assert_eq!(s.background, Some(r.inverse_surface));
        assert_eq!(s.foreground, r.inverse_on_surface);
        assert_eq!(action, r.inverse_primary);
    }
}

#[test]
fn selection_controls_follow_the_spec() {
    for t in material() {
        let r = &t.palette.roles;
        let p = &t.palette;
        let k = |kind, on| t.choice_toggle(kind, on, ControlState::Idle);
        // Switch: unselected track surface-container-highest + outline handle; selected primary.
        let off = k(ToggleKind::Switch, false);
        assert_eq!(
            (off.track, off.border, off.thumb),
            (r.surface_container_highest, r.outline, r.outline)
        );
        let on = k(ToggleKind::Switch, true);
        assert_eq!((on.track, on.thumb), (p.primary, p.on_primary));
        // Checkbox: unselected outline on-surface-variant, no fill; selected primary + on-primary mark.
        let off = k(ToggleKind::Checkbox, false);
        assert_eq!(off.border, r.on_surface_variant);
        assert_eq!(off.track.a, 0);
        let on = k(ToggleKind::Checkbox, true);
        assert_eq!((on.track, on.thumb), (p.primary, p.on_primary));
        // Radio: on-surface-variant ring; selected primary ring + dot.
        assert_eq!(k(ToggleKind::Radio, false).border, r.on_surface_variant);
        let on = k(ToggleKind::Radio, true);
        assert_eq!((on.border, on.thumb), (p.primary, p.primary));
        // Disabled controls are on-surface at 38 % (12 % for switch containers).
        let d = t.choice_toggle(ToggleKind::Checkbox, false, ControlState::Disabled);
        assert_eq!(d.border, r.on_surface.with_alpha(0.38));
    }
}

#[test]
fn tabs_lists_inputs_and_progress_follow_the_spec() {
    for t in material() {
        let r = &t.palette.roles;
        let p = &t.palette;
        // Primary tab: active label + indicator primary, inactive on-surface-variant.
        let on = t.tab(true, false);
        assert_eq!(
            (on.foreground, on.indicator),
            (t.legible(p.primary), Some(p.primary))
        );
        assert_eq!(t.tab(false, false).foreground, r.on_surface_variant);
        // Lists: on-surface text, selected is the secondary-container pill.
        assert_eq!(t.list_item(false, false).foreground, r.on_surface);
        assert_eq!(
            t.list_item(true, false).background,
            Some(r.secondary_container)
        );
        // Outlined text field: outline border, primary when focused.
        assert_eq!(t.input(false).border, r.outline);
        assert_eq!(t.input(true).border, p.primary);
        // Progress track: surface-container-highest.
        assert_eq!(t.progress(None).track, r.surface_container_highest);
        // Filter chip: unselected outline, selected secondary-container.
        assert_eq!(t.chip(false, false).border, Some(r.outline));
        assert_eq!(t.chip(true, false).background, Some(r.secondary_container));
    }
}

/// Non-Material themes keep their own shape for every role (no component may assume M3).
#[test]
fn other_design_systems_keep_their_geometry() {
    for t in themes::all()
        .into_iter()
        .filter(|t| t.meta.family != "material")
    {
        let legacy = t.shape.clone();
        for role in [
            ShapeRole::Button,
            ShapeRole::Dialog,
            ShapeRole::Fab,
            ShapeRole::Checkbox,
        ] {
            let s = t.role_shape(role);
            assert_eq!(
                (s.radius, s.chamfer, s.corners),
                (legacy.radius, legacy.chamfer, legacy.corners),
                "{}: {role:?}",
                t.id()
            );
        }
    }
}

#[test]
fn list_menu_tooltip_and_focus_follow_the_spec() {
    for t in material() {
        let r = &t.palette.roles;
        assert_eq!(t.list_row_height(false), 56.0);
        assert_eq!(t.menu_row_height(), 48.0);
        assert_eq!(t.tooltip(false), (r.inverse_surface, r.inverse_on_surface));
        assert_eq!(t.tooltip(true).0, r.surface_container);
        assert_eq!(t.focus_ring(), (t.palette.secondary, 3.0));
    }
    for t in themes::all()
        .into_iter()
        .filter(|t| t.meta.family != "material")
    {
        assert_eq!(
            t.menu_row_height(),
            t.shape.control_height * 0.8,
            "{}",
            t.id()
        );
        if t.meta.family == "cyberpunk" {
            assert_eq!(t.tooltip(false).0, t.palette.surface_raised, "{}", t.id());
        } else {
            assert_eq!(
                t.tooltip(false),
                (t.palette.overlay, t.palette.text),
                "{}",
                t.id()
            );
        }
        assert_eq!(t.focus_ring().0, t.palette.focus_ring, "{}", t.id());
    }
}

#[test]
fn the_spec_lives_with_the_design_system() {
    for t in themes::all() {
        let want = if t.meta.family == "material" {
            "Material 3"
        } else {
            "Cyberpunk"
        };
        assert_eq!(t.spec.name(), want, "{}", t.id());
    }
}

#[test]
fn look_flags_belong_to_the_design() {
    for t in material() {
        let l = t.look();
        assert!(l.tonal && l.soft_elevation && l.floating_label && l.symbol_icons);
    }
    for t in themes::all()
        .into_iter()
        .filter(|t| t.meta.family != "material")
    {
        assert_eq!(t.look(), dodai::Look::default(), "{}", t.id());
    }
}

#[test]
fn registry_covers_every_theme() {
    let n: usize = themes::systems().iter().map(|s| (s.themes)().len()).sum();
    assert_eq!(n, themes::all().len());
    for s in themes::systems() {
        assert!((s.themes)().iter().all(|t| t.meta.family == s.family));
    }
}

#[test]
fn every_scheme_style_is_readable_for_any_seed() {
    use dodai_core::{Color, Mode};
    use dodai_material::{Scheme, SchemeStyle};
    let seeds = [
        0x6750a4, 0xff0000, 0x00ff00, 0x0000ff, 0xffff00, 0x808080, 0x000000, 0xffffff, 0x00ffff,
        0xff00ff,
    ];
    for style in SchemeStyle::ALL {
        for seed in seeds {
            for mode in [Mode::Light, Mode::Dark] {
                let t = Scheme::new(Color::hex(seed), style, mode).theme(dodai_core::ThemeMeta {
                    id: "t",
                    name: "t",
                    family: "material",
                    description: "",
                    mode,
                });
                let (p, r) = (&t.palette, &t.palette.roles);
                let tag = format!("{style:?} {seed:06x} {mode:?}");
                assert!(p.text.contrast(p.background) >= 7.0, "{tag}: text");
                assert!(p.on_primary.contrast(p.primary) >= 4.5, "{tag}: on_primary");
                assert!(
                    p.on_secondary.contrast(p.secondary) >= 4.5,
                    "{tag}: on_secondary"
                );
                assert!(
                    r.on_primary_container.contrast(r.primary_container) >= 7.0,
                    "{tag}: container"
                );
                assert!(
                    r.on_secondary_container.contrast(r.secondary_container) >= 7.0,
                    "{tag}: sec container"
                );
                assert!(r.on_tertiary.contrast(r.tertiary) >= 4.5, "{tag}: tertiary");
                assert!(
                    r.on_error_container.contrast(r.error_container) >= 7.0,
                    "{tag}: error"
                );
                assert!(
                    r.on_surface_variant.contrast(r.surface_container_highest) >= 4.5,
                    "{tag}: variant"
                );
                assert!(
                    r.inverse_on_surface.contrast(r.inverse_surface) >= 7.0,
                    "{tag}: inverse"
                );
            }
        }
    }
}

#[test]
fn pastel_is_lighter_and_softer_than_the_baseline() {
    use dodai_core::{tonal::to_lch, Mode};
    let base = dodai_material::light();
    let pastel = dodai_material::PRESETS[0].theme(Mode::Light);
    assert!(to_lch(pastel.palette.primary).0 > to_lch(base.palette.primary).0 + 15.0);
    let dark_base = dodai_material::dark();
    let dark_pastel = dodai_material::PRESETS[0].theme(Mode::Dark);
    assert!(
        to_lch(dark_pastel.palette.background).0 > to_lch(dark_base.palette.background).0 + 5.0
    );
}

#[test]
fn app_bars_follow_the_spec() {
    use dodai_core::AppBarKind::*;
    for t in material() {
        let r = &t.palette.roles;
        for (k, h) in [
            (CenterAligned, 64.0),
            (Small, 64.0),
            (Medium, 112.0),
            (Large, 152.0),
            (Bottom, 80.0),
        ] {
            let b = t.app_bar(k, false);
            assert_eq!(b.height, h, "{} {k:?}", t.id());
            assert!(b.border.is_none(), "app bars are borderless");
            assert_eq!(b.title, r.on_surface);
        }
        assert_eq!(
            t.app_bar(Small, false).background,
            Some(t.palette.background)
        );
        assert_eq!(t.app_bar(Small, true).background, Some(r.surface_container));
        assert_eq!(
            t.app_bar(Bottom, false).background,
            Some(r.surface_container)
        );
        assert_eq!(t.app_bar(Large, false).title_size, 28.0);
        assert_eq!(t.role_shape(ShapeRole::AppBar).radius, 0.0);
    }
}

#[test]
fn fixed_and_dim_roles_are_readable_and_ordered() {
    use dodai_core::{Color, Mode};
    use dodai_material::{Scheme, SchemeStyle};
    let mut all = material();
    for style in SchemeStyle::ALL {
        for mode in [Mode::Light, Mode::Dark] {
            all.push(
                Scheme::new(Color::hex(0x3366cc), style, mode).theme(dodai_core::ThemeMeta {
                    id: "t",
                    name: "t",
                    family: "material",
                    description: "",
                    mode,
                }),
            );
        }
    }
    for t in all {
        let r = &t.palette.roles;
        let id = t.id();
        for (on, bg, min, what) in [
            (r.on_primary_fixed, r.primary_fixed, 7.0, "primary"),
            (
                r.on_primary_fixed_variant,
                r.primary_fixed,
                4.5,
                "primary variant",
            ),
            (r.on_secondary_fixed, r.secondary_fixed, 7.0, "secondary"),
            (
                r.on_secondary_fixed_variant,
                r.secondary_fixed,
                4.5,
                "secondary variant",
            ),
            (r.on_tertiary_fixed, r.tertiary_fixed, 7.0, "tertiary"),
            (
                r.on_tertiary_fixed_variant,
                r.tertiary_fixed,
                4.5,
                "tertiary variant",
            ),
        ] {
            assert!(on.contrast(bg) >= min, "{id}: fixed {what}");
        }
        assert!(
            r.primary_fixed_dim.luminance() <= r.primary_fixed.luminance(),
            "{id}"
        );
        assert!(
            r.surface_bright.luminance() >= r.surface_dim.luminance(),
            "{id}"
        );
    }
}

#[test]
fn list_items_grow_with_their_lines() {
    for t in material() {
        assert_eq!(
            [1u8, 2, 3].map(|n| t.list_lines_height(n)),
            [56.0, 72.0, 88.0]
        );
    }
    // The baseline scales from the base row height.
    let base = dodai::themes::neutral();
    assert!(base.list_lines_height(3) > base.list_lines_height(2));
    assert!(base.list_lines_height(2) > base.list_lines_height(1));
}
