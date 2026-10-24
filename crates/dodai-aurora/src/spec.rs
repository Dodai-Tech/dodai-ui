//! The Aurora **design-system spec**: glassy filled and outlined controls, frosted tone fills,
//! tinted soft shadows, 16 px controls and 30 px containers.

use dodai_core::{
    AppBarKind, AppBarTokens, ButtonTokens, ButtonVariant, CardVariant, Color, ControlState,
    CornerStyle, Corners, DefaultShape, DesignSpec, IconSet, InputTokens, ListItemTokens, Look,
    Mode, NavigationTokens, ProgressTokens, RoleShape, ShapePreset, ShapeRole, ShapeScale,
    ShapeSize, ShapeSystem, Sizes, StepState, StepTokens, SurfaceTokens, TabTokens, Theme,
    ToggleKind, ToggleTokens, Tone, TypeRole,
};

const ALL: Corners = Corners::ALL;

/// The Aurora spec (attach with [`dodai_core::Theme::with_spec`]).
#[derive(Debug)]
pub struct Aurora;

/// Shared instance for `Theme::with_spec`.
pub static SPEC: Aurora = Aurora;

/// Aurora radii (the `sm … 3xl` tokens): 6 / 10 / 16 / 22 / 30. Generous corners.
pub const SHAPE_SCALE: ShapeScale = ShapeScale {
    xs: 6.0,
    sm: 10.0,
    md: 16.0,
    lg: 22.0,
    xl: 30.0,
};

const fn r(role: ShapeRole, size: ShapeSize, sides: Corners) -> RoleShape {
    RoleShape::new(role, CornerStyle::Round, size, sides)
}

const ROLES: &[RoleShape] = &[
    // Interactive controls: the medium radius.
    r(ShapeRole::Button, ShapeSize::Medium, ALL),
    r(ShapeRole::IconButton, ShapeSize::Medium, ALL),
    r(ShapeRole::Input, ShapeSize::Medium, ALL),
    r(ShapeRole::SearchBar, ShapeSize::Medium, ALL),
    r(ShapeRole::Fab, ShapeSize::Medium, ALL),
    r(ShapeRole::FabSmall, ShapeSize::Medium, ALL),
    r(ShapeRole::FabLarge, ShapeSize::Large, ALL),
    r(ShapeRole::Menu, ShapeSize::Medium, ALL),
    r(ShapeRole::Chip, ShapeSize::Small, ALL),
    r(ShapeRole::Checkbox, ShapeSize::ExtraSmall, ALL),
    // Standalone indicators are pills.
    r(ShapeRole::Tag, ShapeSize::Full, ALL),
    r(ShapeRole::Badge, ShapeSize::Full, ALL),
    r(ShapeRole::Indicator, ShapeSize::Full, ALL),
    // Notifications: the large radius; card-like containers: the extra-large radius.
    r(ShapeRole::Snackbar, ShapeSize::Large, ALL),
    r(ShapeRole::RichTooltip, ShapeSize::Large, ALL),
    r(ShapeRole::Tooltip, ShapeSize::Small, ALL),
    r(ShapeRole::Card, ShapeSize::ExtraLarge, ALL),
    r(ShapeRole::Panel, ShapeSize::ExtraLarge, ALL),
    r(ShapeRole::Dialog, ShapeSize::ExtraLarge, ALL),
    r(ShapeRole::Sheet, ShapeSize::ExtraLarge, Corners::TOP),
    r(ShapeRole::SideSheet, ShapeSize::ExtraLarge, ALL),
    r(ShapeRole::Carousel, ShapeSize::ExtraLarge, ALL),
    r(ShapeRole::Toolbar, ShapeSize::Medium, ALL),
    r(ShapeRole::AppBar, ShapeSize::None, ALL),
];

/// The Aurora shape language: 16 px controls, 30 px containers, pill indicators.
pub fn shape_system() -> ShapeSystem {
    ShapeSystem {
        pill: ShapeSystem::PILL_OF_CONTROL,
        name: "Aurora",
        scale: SHAPE_SCALE,
        roles: std::borrow::Cow::Borrowed(ROLES),
        default: DefaultShape {
            style: CornerStyle::Round,
            size: ShapeSize::Medium,
            sides: ALL,
        },
    }
}

/// Compact mode: interactive controls drop to the `lg` (12 px) radius.
pub fn shape_system_compact() -> ShapeSystem {
    ShapeSystem {
        pill: ShapeSystem::PILL_OF_CONTROL,
        name: "Aurora compact",
        scale: ShapeScale {
            md: crate::tokens::radius("lg").unwrap_or(12.0),
            ..SHAPE_SCALE
        },
        ..shape_system()
    }
}

/// Shape presets: the default, its compact mode and a squarer take.
pub fn shape_presets() -> Vec<ShapePreset> {
    vec![
        ShapePreset::new("aurora", "Aurora", shape_system()),
        ShapePreset::new("compact", "Aurora compact", shape_system_compact()),
        ShapePreset::new("sharp", "Sharp", ShapeSystem::sharp()),
    ]
}

fn surface(
    background: Option<Color>,
    border: Option<Color>,
    foreground: Color,
    shadow: bool,
) -> SurfaceTokens {
    SurfaceTokens {
        background,
        border,
        foreground,
        glow: false,
        shadow,
    }
}

/// The page colour in light, the lifted surface in dark: what floating layers sit on.
fn floating(t: &Theme) -> Color {
    if t.meta.mode == Mode::Dark {
        t.palette.surface
    } else {
        t.palette.background
    }
}

fn lit(state: ControlState) -> bool {
    matches!(state, ControlState::Hovered | ControlState::Focused)
}

/// The colour tokens of the theme's mode.
fn mode_tokens(t: &Theme) -> &'static crate::tokens::ColorTokens {
    if t.meta.mode == Mode::Dark {
        &crate::tokens::DARK
    } else {
        &crate::tokens::LIGHT
    }
}

/// The status token set of a tone in the theme's mode.
fn status_tokens(t: &Theme, tone: Tone) -> &'static crate::tokens::StatusTokens {
    let m = mode_tokens(t);
    match tone {
        Tone::Info => &m.info,
        Tone::Success => &m.success,
        Tone::Warning => &m.warning,
        Tone::Danger => &m.error,
    }
}

/// The tone's `frosted` (banner) or `frosted_soft` (inline) fill, composited over the canvas
/// so ink on it stays legible.
fn tone_fill(t: &Theme, tone: Tone, banner: bool) -> Color {
    let s = status_tokens(t, tone);
    (if banner { s.frosted } else { s.frosted_soft }).over(t.palette.background)
}

/// The `frosted_soft`, `frosted` or `frosted_strong` glass fill, over the canvas.
fn frosted(t: &Theme, strength: u8) -> Color {
    let m = mode_tokens(t);
    let f = match strength {
        0 => m.frosted_soft,
        1 => m.frosted,
        _ => m.frosted_strong,
    };
    f.over(t.palette.background)
}

/// Checkbox / radio / switch tokens, shared by `toggle` and `choice_toggle`.
fn selection(t: &Theme, kind: ToggleKind, on: bool, state: ControlState) -> ToggleTokens {
    let p = &t.palette;
    let clear = p.text.with_alpha(0.0);
    if state == ControlState::Disabled {
        return ToggleTokens {
            track: if on { p.text_disabled } else { clear },
            thumb: if on { p.background } else { clear },
            border: p.text_disabled,
        };
    }
    let outline = if lit(state) { p.text } else { p.border_strong };
    let fill = if lit(state) {
        p.primary_hover
    } else {
        p.primary
    };
    match (kind, on) {
        (ToggleKind::Switch, true) => ToggleTokens {
            track: fill,
            thumb: p.on_primary,
            border: fill,
        },
        (ToggleKind::Switch, false) => ToggleTokens {
            track: p.surface_raised,
            thumb: p.border_strong,
            border: p.surface_raised,
        },
        (ToggleKind::Checkbox, true) => ToggleTokens {
            track: fill,
            thumb: p.on_primary,
            border: fill,
        },
        (ToggleKind::Radio, true) => ToggleTokens {
            track: clear,
            thumb: p.primary,
            border: p.primary,
        },
        (ToggleKind::Checkbox | ToggleKind::Radio, false) => ToggleTokens {
            track: clear,
            thumb: clear,
            border: outline,
        },
    }
}

impl DesignSpec for Aurora {
    fn name(&self) -> &'static str {
        "Aurora"
    }

    fn look(&self) -> Look {
        Look {
            soft_elevation: true,
            icons: IconSet::Symbols,
            ..Look::default()
        }
    }

    fn sizes(&self, _: &Theme) -> Option<Sizes> {
        Some(Sizes {
            selection_box: 24.0,
            switch_track: (52.0, 32.0),
            switch_handle: (24.0, 24.0),
            slider_thumb: 22.0,
            segmented_height: 48.0,
            ..Sizes::default()
        })
    }

    fn button_padding(&self, _: &Theme) -> Option<f32> {
        Some(24.0)
    }

    fn card(&self, t: &Theme, v: CardVariant) -> Option<SurfaceTokens> {
        let p = &t.palette;
        Some(match v {
            CardVariant::Elevated => surface(Some(floating(t)), None, p.text, true),
            CardVariant::Filled => surface(Some(p.surface), None, p.text, false),
            CardVariant::Outlined => surface(None, Some(p.border), p.text, false),
        })
    }

    /// Chips are frosted; the selected one fills with the primary ink.
    fn chip(&self, t: &Theme, selected: bool, hovered: bool) -> Option<SurfaceTokens> {
        let p = &t.palette;
        Some(if selected {
            surface(Some(p.primary), None, p.on_primary, false)
        } else {
            surface(
                Some(frosted(t, if hovered { 2 } else { 1 })),
                None,
                p.text,
                false,
            )
        })
    }

    /// Frosted tone fill with ink text (banner / inline notification).
    fn alert(&self, t: &Theme, tone: Tone, banner: bool) -> Option<SurfaceTokens> {
        Some(surface(
            Some(tone_fill(t, tone, banner)),
            None,
            t.palette.text,
            false,
        ))
    }

    /// The toast is inverted ink.
    fn snackbar(&self, t: &Theme) -> Option<(SurfaceTokens, Color)> {
        let p = &t.palette;
        // The toast is inverted, so its action uses the other mode's info colour.
        let other = if t.meta.mode == Mode::Dark {
            &crate::tokens::LIGHT
        } else {
            &crate::tokens::DARK
        };
        let action = other.info.base.color();
        Some((surface(Some(p.primary), None, p.on_primary, true), action))
    }

    fn dialog(&self, t: &Theme) -> Option<SurfaceTokens> {
        Some(surface(Some(floating(t)), None, t.palette.text, true))
    }

    fn sheet(&self, t: &Theme) -> Option<SurfaceTokens> {
        Some(surface(Some(floating(t)), None, t.palette.text, true))
    }

    fn popup(&self, t: &Theme) -> Option<SurfaceTokens> {
        Some(surface(Some(floating(t)), None, t.palette.text, true))
    }

    /// The backdrop colour.
    fn scrim(&self, t: &Theme) -> Option<Color> {
        Some(mode_tokens(t).backdrop.color())
    }

    fn step(&self, t: &Theme, state: StepState) -> Option<StepTokens> {
        let p = &t.palette;
        Some(match state {
            StepState::Done => StepTokens {
                fill: Some(p.primary),
                stroke: p.primary,
                mark: p.on_primary,
                text: p.text,
                glow: false,
            },
            StepState::Current => StepTokens {
                fill: None,
                stroke: p.primary,
                mark: p.text,
                text: p.text,
                glow: false,
            },
            StepState::Todo => StepTokens {
                fill: None,
                stroke: p.border_strong,
                mark: p.text_muted,
                text: p.text_muted,
                glow: false,
            },
        })
    }

    /// Ink label with a 2 px underline when active; frosted on hover.
    fn tab(&self, t: &Theme, active: bool, hovered: bool) -> Option<TabTokens> {
        let p = &t.palette;
        Some(TabTokens {
            background: hovered.then_some(p.surface),
            foreground: if active { p.text } else { p.text_muted },
            indicator: active.then_some(p.primary),
            outline: false,
        })
    }

    fn toggle(&self, t: &Theme, on: bool, state: ControlState) -> Option<ToggleTokens> {
        Some(selection(t, ToggleKind::Switch, on, state))
    }

    fn choice_toggle(
        &self,
        t: &Theme,
        kind: ToggleKind,
        on: bool,
        state: ControlState,
    ) -> Option<ToggleTokens> {
        Some(selection(t, kind, on, state))
    }

    /// Tags: frosted tone fill, tone-coloured label, pill.
    fn badge(&self, t: &Theme, tone: Tone) -> Option<SurfaceTokens> {
        let c = t.palette.tone(tone);
        let low = status_tokens(t, tone).low.over(t.palette.background);
        Some(surface(Some(low), None, t.legible(c), false))
    }

    fn progress(&self, t: &Theme, tone: Option<Tone>) -> Option<ProgressTokens> {
        let p = &t.palette;
        Some(ProgressTokens {
            track: frosted(t, 1),
            fill: tone.map_or(p.primary, |x| p.tone(x)),
            border: frosted(t, 1),
            segments: 0,
        })
    }

    fn navigation(&self, t: &Theme) -> Option<NavigationTokens> {
        Some(NavigationTokens {
            item_height: 48.0,
            nested_height: 40.0,
            headline_height: 48.0,
            item_padding: 16.0,
            container_padding: 8.0,
            item_content_gap: 12.0,
            item_gap: 4.0,
            indent: 16.0,
            icon_size: t.sizes().icon,
            drawer_max_width: 320.0,
            rail_width: 80.0,
            indicator: (56.0, 32.0),
            rail_label_gap: 4.0,
            bar_height: 72.0,
            bar_indicator_width: 56.0,
            pill: false,
            container: None,
            roles: (
                TypeRole::LabelLarge,
                TypeRole::LabelMedium,
                TypeRole::LabelMedium,
                TypeRole::TitleSmall,
            ),
        })
    }

    fn navigation_item(&self, t: &Theme, selected: bool, hovered: bool) -> Option<ListItemTokens> {
        self.list_item(t, selected, hovered)
    }

    /// Selected rows are a frosted block; hover is a lighter one.
    fn list_item(&self, t: &Theme, selected: bool, hovered: bool) -> Option<ListItemTokens> {
        let p = &t.palette;
        Some(ListItemTokens {
            background: if selected {
                Some(p.surface_raised)
            } else {
                hovered.then_some(p.surface)
            },
            foreground: p.text,
            marker: None,
            outline: false,
        })
    }

    fn divider(&self, t: &Theme) -> Option<Color> {
        Some(t.palette.border)
    }

    fn app_bar(&self, t: &Theme, kind: AppBarKind, _scrolled: bool) -> Option<AppBarTokens> {
        let p = &t.palette;
        let (row, height, title) = match kind {
            AppBarKind::CenterAligned | AppBarKind::Small => (64.0, 64.0, 20.0),
            AppBarKind::Medium => (64.0, 112.0, 28.0),
            AppBarKind::Large => (64.0, 152.0, 40.0),
            AppBarKind::Bottom => (72.0, 72.0, 16.0),
        };
        Some(AppBarTokens {
            row_height: row,
            height,
            background: Some(p.background),
            border: Some(p.border),
            title: p.text,
            leading: p.text,
            trailing: p.text,
            title_size: title,
            collapsed_title_size: 20.0,
            glow: false,
        })
    }

    /// Primary: filled ink. Secondary / outline: 2 px ink outline. Ghost: bare label.
    fn button(
        &self,
        t: &Theme,
        variant: ButtonVariant,
        state: ControlState,
    ) -> Option<ButtonTokens> {
        use ButtonVariant::*;
        let p = &t.palette;
        let outlined = matches!(variant, Secondary | Outline | OutlineSecondary | Scrim);
        if state == ControlState::Disabled {
            return Some(ButtonTokens {
                background: (!outlined && variant != Ghost).then_some(p.surface_raised),
                foreground: p.text_disabled,
                border: outlined.then_some(p.text_disabled),
            });
        }
        let hot = matches!(state, ControlState::Hovered | ControlState::Pressed);
        let (bg, fg, border) = match variant {
            Primary => (
                Some(if hot { p.primary_hover } else { p.primary }),
                p.on_primary,
                None,
            ),
            Danger => (
                Some(if hot {
                    p.danger.mix(p.text, 0.2)
                } else {
                    p.danger
                }),
                t.on(p.danger),
                None,
            ),
            Secondary | Outline | OutlineSecondary | Scrim => {
                (hot.then(|| p.text.with_alpha(0.08)), p.text, Some(p.text))
            }
            Ghost => (hot.then(|| p.text.with_alpha(0.08)), p.text, None),
        };
        Some(ButtonTokens {
            background: bg,
            foreground: fg,
            border,
        })
    }

    fn fab(&self, t: &Theme, variant: ButtonVariant, state: ControlState) -> Option<ButtonTokens> {
        let mut b = self.button(t, variant, state)?;
        if b.background.is_none() && state != ControlState::Disabled {
            b.background = Some(floating(t));
        }
        Some(b)
    }

    fn icon_button(
        &self,
        t: &Theme,
        variant: ButtonVariant,
        state: ControlState,
        selected: bool,
        base: ButtonTokens,
    ) -> Option<ButtonTokens> {
        let p = &t.palette;
        if state == ControlState::Disabled {
            return Some(base);
        }
        if matches!(variant, ButtonVariant::Ghost) {
            return Some(ButtonTokens {
                background: if selected {
                    Some(p.surface_raised)
                } else {
                    lit(state)
                        .then(|| p.text.with_alpha(0.08))
                        .or((state == ControlState::Pressed).then(|| p.text.with_alpha(0.12)))
                },
                foreground: p.text,
                border: None,
            });
        }
        Some(base)
    }

    /// A 2 px outline in medium contrast; ink when focused. Fields stay on the page colour.
    fn input(&self, t: &Theme, focused: bool) -> Option<InputTokens> {
        let p = &t.palette;
        Some(InputTokens {
            background: p.background,
            border: if focused { p.text } else { p.border_strong },
            text: p.text,
            placeholder: p.text_muted,
            selection: status_tokens(t, Tone::Info).low.color(),
        })
    }

    /// The focus colour, 2 px.
    fn focus_ring(&self, _t: &Theme) -> Option<(Color, f32)> {
        Some((crate::tokens::LIGHT.focus, crate::tokens::FOCUS_WIDTH))
    }

    /// The page grid: outer margins, fluid gap, 4 or 12 columns, 2560 px at most.
    fn page_grid(&self, _t: &Theme, viewport: f32) -> Option<dodai_core::GridSpec> {
        use crate::tokens as k;
        Some(dodai_core::GridSpec {
            columns: k::grid_columns(viewport),
            gap: k::grid_gap(viewport),
            margin: k::grid_safe_zone(viewport),
            max_width: Some(k::GRID_WIDTH_MAX),
        })
    }

    /// The fluid type scale (2xs … xl) and fluid spacing (xs … xl) at the viewport width.
    fn fluid(&self, t: &Theme, viewport: f32) -> Option<dodai_core::Fluid> {
        use crate::tokens as k;
        let ty = |n| k::type_size(n, viewport).unwrap_or(16.0);
        let sp = |n| k::spacing_fluid(n, viewport).unwrap_or(16.0);
        let mut typography = t.typography.clone();
        typography.size_xs = ty("2xs");
        typography.size_sm = ty("xs");
        typography.size_md = ty("sm");
        typography.size_lg = ty("md");
        typography.size_xl = ty("lg");
        typography.size_xxl = ty("xl");
        let mut spacing = t.spacing.clone();
        spacing.xs = sp("xs");
        spacing.sm = sp("sm");
        spacing.md = sp("md");
        spacing.lg = sp("lg");
        spacing.xl = sp("xl");
        Some(dodai_core::Fluid {
            typography,
            spacing,
        })
    }

    /// The frosted glass fill over the frosted blur (28 px).
    fn frost(&self, t: &Theme) -> Option<dodai_core::Frost> {
        Some(dodai_core::Frost {
            tint: mode_tokens(t).frosted.color(),
            blur: crate::tokens::BLUR_FROSTED,
            edge: None,
        })
    }

    /// `$gradient-stops-fade-dark`.
    fn fade_stops(&self, _t: &Theme) -> Option<Vec<dodai_core::FadeStop>> {
        Some(
            crate::tokens::GRADIENT_FADE_DARK
                .iter()
                .map(|(a, p)| dodai_core::FadeStop {
                    alpha: *a,
                    at: *p / 100.0,
                })
                .collect(),
        )
    }

    /// The shadow tiers `sm / md / lg`: levels 1 → sm, 2-3 → md, 4-5 → lg.
    fn elevation_layers(&self, _t: &Theme, level: u8) -> Option<Vec<dodai_core::ShadowLayer>> {
        use crate::tokens::{SHADOW_LG, SHADOW_MD, SHADOW_SM};
        let s = match level {
            0 => return Some(Vec::new()),
            1 => SHADOW_SM,
            2 | 3 => SHADOW_MD,
            _ => SHADOW_LG,
        };
        Some(vec![dodai_core::ShadowLayer {
            dy: s.y,
            blur: s.blur,
            spread: 0.0,
            alpha: s.alpha,
        }])
    }

    fn tooltip(&self, t: &Theme, _rich: bool) -> Option<(Color, Color)> {
        Some((t.palette.primary, t.palette.on_primary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_scale_is_the_radius_tokens() {
        use crate::tokens::radius;
        let sc = SHAPE_SCALE;
        assert_eq!(Some(sc.xs), radius("sm"));
        assert_eq!(Some(sc.sm), radius("md"));
        assert_eq!(Some(sc.md), radius("xl"));
        assert_eq!(Some(sc.lg), radius("2xl"));
        assert_eq!(Some(sc.xl), radius("3xl"));
        assert_eq!(shape_system_compact().scale.md, 12.0);
    }

    #[test]
    fn responsive_theme_follows_the_viewport() {
        let t = crate::light();
        let (narrow, wide) = (t.responsive(480.0), t.responsive(1920.0));
        assert!(narrow.typography.size_lg < wide.typography.size_lg);
        assert!(narrow.spacing.lg < wide.spacing.lg);
        assert_eq!(narrow.typography.size_md, 15.0);
        assert_eq!(t.page_grid(1440.0).columns, 12);
        assert_eq!(t.page_grid(600.0).columns, 4);
        assert_eq!(
            t.frost().blur,
            28.0,
            "the design's own frost, not the default"
        );
        let f = t.fade_stops();
        assert_eq!((f[0].alpha, f[0].at), (0.8, 0.0));
        let last = f.last().unwrap();
        assert_eq!((last.alpha, last.at), (0.0, 1.0));
    }

    #[test]
    fn shadows_use_the_tiers() {
        let t = crate::light();
        let l = |n| SPEC.elevation_layers(&t, n).unwrap();
        let at = |n| {
            let l = &l(n)[0];
            (l.dy, l.blur, l.spread, l.alpha)
        };
        assert_eq!(at(1), (2.0, 8.0, 0.0, 0.10));
        assert_eq!(at(3), (6.0, 20.0, 0.0, 0.12));
        assert_eq!(at(5), (16.0, 48.0, 0.0, 0.16));
        assert!(l(0).is_empty());
    }

    #[test]
    fn typography_and_spacing_come_from_the_tokens() {
        let t = crate::light();
        assert_eq!(t.typography.size_md, 15.0);
        assert_eq!(t.typography.size_sm, 14.0);
        assert_eq!(t.typography.size_xs, 12.0);
        assert_eq!(t.spacing.lg, 24.0);
        assert_eq!(t.motion.duration.as_millis(), 200);
    }

    #[test]
    fn two_unique_themes() {
        let v = crate::variants();
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].id(), "aurora");
        assert_eq!(v[1].id(), "aurora-dark");
        for t in &v {
            assert_eq!(t.spec.name(), "Aurora");
            assert_eq!(t.meta.family, "aurora");
            assert!(!t.effects.glow_enabled() && t.effects.hard_shadow == 0.0);
        }
    }

    #[test]
    fn text_is_legible_on_its_surfaces() {
        for t in crate::variants() {
            let p = &t.palette;
            for (bg, fg) in [
                (p.background, p.text),
                (p.surface, p.text),
                (p.primary, p.on_primary),
                (p.background, p.text_muted),
            ] {
                assert!(fg.contrast(bg) >= 4.5, "{} {:?} on {:?}", t.id(), fg, bg);
            }
        }
    }

    #[test]
    fn look_is_soft_and_flat() {
        for t in crate::variants() {
            let l = t.look();
            assert!(
                l.soft_elevation && !l.tonal && !l.sink && !l.luminous,
                "{}",
                t.id()
            );
            assert_eq!(l.icons, IconSet::Symbols);
        }
    }

    #[test]
    fn controls_are_sixteen_and_cards_thirty() {
        let t = crate::light();
        let ss = t.shape_system.as_ref().unwrap();
        assert_eq!(ss.scale.md, 16.0);
        assert_eq!(ss.scale.xl, 30.0);
    }

    #[test]
    fn primary_is_filled_and_secondary_outlined() {
        for t in crate::variants() {
            let pr = t.button(ButtonVariant::Primary, ControlState::Idle);
            assert_eq!(pr.background, Some(t.palette.primary));
            assert_eq!(pr.border, None);
            let se = t.button(ButtonVariant::Secondary, ControlState::Idle);
            assert_eq!(se.background, None);
            assert_eq!(se.border, Some(t.palette.text));
        }
    }
}
