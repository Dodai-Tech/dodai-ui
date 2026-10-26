//! The Material 3 **design-system spec**: component tokens, geometry and shape language.
//!
//! Core only defines the vocabulary; everything Material-specific lives here and is attached to
//! the themes with [`dodai_core::Theme::with_spec`]. Values follow the `material-web` component
//! tokens (state layers 8 % hover / 10 % focus and press, disabled 12 % / 38 %).

use dodai_core::{
    AppBarKind, AppBarTokens, ButtonTokens, ButtonVariant, CardVariant, Color, ControlSize,
    ControlState, CornerStyle, Corners, DefaultShape, DesignSpec, IconSet, InputTokens,
    ListItemTokens, Look, NavigationTokens, ProgressTokens, RoleShape, ShapePreset, ShapeRole,
    ShapeScale, ShapeSize, ShapeSystem, SizeMetrics, SurfaceTokens, TabTokens, Theme, ToggleKind,
    ToggleTokens, Tone, TypeRole, TypeStyle,
};

const ALL: Corners = Corners::ALL;

/// The Material 3 spec (attach with [`dodai_core::Theme::with_spec`]).
#[derive(Debug)]
pub struct Material3;

/// Shared instance for `Theme::with_spec`.
pub static SPEC: Material3 = Material3;

/// M3 shape scale: 4 / 8 / 12 / 16 / 28.
pub const SHAPE_SCALE: ShapeScale = ShapeScale {
    xs: 4.0,
    sm: 8.0,
    md: 12.0,
    lg: 16.0,
    xl: 28.0,
};

const fn r(role: ShapeRole, size: ShapeSize, sides: Corners) -> RoleShape {
    RoleShape::new(role, CornerStyle::Round, size, sides)
}

const ROLES: &[RoleShape] = &[
    r(ShapeRole::Button, ShapeSize::Full, ALL),
    r(ShapeRole::Fab, ShapeSize::Large, ALL),
    r(ShapeRole::Chip, ShapeSize::Small, ALL),
    r(ShapeRole::Tag, ShapeSize::Small, ALL),
    r(ShapeRole::Badge, ShapeSize::Full, ALL),
    r(ShapeRole::Input, ShapeSize::ExtraSmall, Corners::TOP),
    r(ShapeRole::Card, ShapeSize::Medium, ALL),
    r(ShapeRole::Panel, ShapeSize::Medium, ALL),
    r(ShapeRole::Menu, ShapeSize::ExtraSmall, ALL),
    r(ShapeRole::Dialog, ShapeSize::ExtraLarge, ALL),
    r(ShapeRole::Sheet, ShapeSize::ExtraLarge, Corners::TOP),
    r(ShapeRole::Tooltip, ShapeSize::ExtraSmall, ALL),
    r(ShapeRole::IconButton, ShapeSize::Full, ALL),
    r(ShapeRole::FabSmall, ShapeSize::Medium, ALL),
    r(ShapeRole::FabLarge, ShapeSize::ExtraLarge, ALL),
    r(ShapeRole::SearchBar, ShapeSize::Full, ALL),
    r(ShapeRole::Snackbar, ShapeSize::ExtraSmall, ALL),
    r(ShapeRole::Carousel, ShapeSize::ExtraLarge, ALL),
    r(ShapeRole::SideSheet, ShapeSize::Large, ALL),
    r(ShapeRole::Checkbox, ShapeSize::Px(2.0), ALL),
    r(ShapeRole::Indicator, ShapeSize::Full, ALL),
    r(ShapeRole::RichTooltip, ShapeSize::Medium, ALL),
    r(ShapeRole::Toolbar, ShapeSize::Full, ALL),
    r(ShapeRole::AppBar, ShapeSize::None, ALL),
];

/// The Material 3 shape language: rounded scale, pill buttons, top-rounded fields.
pub fn shape_system() -> ShapeSystem {
    ShapeSystem {
        pill: ShapeSystem::PILL_OF_CONTROL,
        name: "Material 3",
        scale: SHAPE_SCALE,
        roles: std::borrow::Cow::Borrowed(ROLES),
        default: DefaultShape {
            style: CornerStyle::Round,
            size: ShapeSize::Medium,
            sides: ALL,
        },
    }
}

/// Shape presets offered by Material 3.
pub fn shape_presets() -> Vec<ShapePreset> {
    vec![
        ShapePreset::new("m3", "Material", shape_system()),
        ShapePreset::new("m3-cut", "Cut", shape_system().with_style(CornerStyle::Cut)),
        ShapePreset::new(
            "m3-scoop",
            "Scoop",
            shape_system().with_style(CornerStyle::Scoop),
        ),
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

/// State-layer opacity: 8 % hover, 10 % focus and press.
fn layer(state: ControlState) -> f32 {
    match state {
        ControlState::Hovered => 0.08,
        ControlState::Pressed | ControlState::Focused => 0.10,
        _ => 0.0,
    }
}

/// Frosted glass: blur sigma in px, tint opacity, and how much primary colours the tint.
const FROST_BLUR: f32 = 28.0;
const FROST_ALPHA: f32 = 0.7;
const FROST_PRIMARY_MIX: f32 = 0.06;

impl DesignSpec for Material3 {
    fn name(&self) -> &'static str {
        "Material 3"
    }

    /// Soft: slightly heavier lines, quiet outline-variant gridlines, a light area tint.
    fn chart_style(&self, t: &Theme) -> Option<dodai_core::ChartStyle> {
        Some(dodai_core::ChartStyle {
            line_width: 2.5,
            grid_color: t.palette.border.with_alpha(0.5),
            area_alpha: 0.12,
            dial: dodai_core::DialLook::Arc,
            cluster: dodai_core::ClusterKind::Arc,
            ..dodai_core::ChartStyle::from_theme(t)
        })
    }

    /// Glass for scrims, sheets and floating bars: the high surface container, a touch of
    /// primary, 70% opaque over a 28 px blur (M3 has no glass spec; this stays tonal).
    fn frost(&self, t: &Theme) -> Option<dodai_core::Frost> {
        let r = &t.palette.roles;
        Some(dodai_core::Frost {
            tint: r
                .surface_container_high
                .mix(t.palette.primary, FROST_PRIMARY_MIX)
                .with_alpha(FROST_ALPHA),
            blur: FROST_BLUR,
            edge: None,
        })
    }

    /// The Material 3 window size classes: columns and margins grow with the window.
    fn page_grid(&self, _t: &Theme, viewport: f32) -> Option<dodai_core::GridSpec> {
        let (columns, gap, margin) = window_class(viewport);
        Some(dodai_core::GridSpec {
            columns,
            gap,
            margin,
            max_width: None,
        })
    }

    fn look(&self) -> Look {
        Look {
            tonal: true,
            soft_elevation: true,
            floating_label: true,
            icons: IconSet::Symbols,
            sink: false,
            ..Look::default()
        }
    }

    fn card(&self, t: &Theme, v: CardVariant) -> Option<SurfaceTokens> {
        let r = &t.palette.roles;
        let text = t.palette.text;
        Some(match v {
            CardVariant::Elevated => surface(Some(r.surface_container_low), None, text, true),
            CardVariant::Filled => surface(Some(r.surface_container_highest), None, text, false),
            CardVariant::Outlined => surface(None, Some(r.outline_variant), text, false),
        })
    }

    fn chip(&self, t: &Theme, selected: bool, hovered: bool) -> Option<SurfaceTokens> {
        let r = &t.palette.roles;
        Some(if selected {
            surface(
                Some(r.secondary_container),
                None,
                r.on_secondary_container,
                false,
            )
        } else {
            // Unselected: transparent with an `outline` border; hover is a state layer.
            surface(
                hovered.then(|| r.on_surface_variant.with_alpha(0.08)),
                Some(r.outline),
                r.on_surface_variant,
                false,
            )
        })
    }

    fn alert(&self, t: &Theme, tone: Tone, banner: bool) -> Option<SurfaceTokens> {
        if !banner {
            return None;
        }
        // A tonal container, not a saturated slab.
        let c = t.palette.tone(tone);
        let fill = match tone {
            Tone::Danger => t.palette.roles.error_container,
            _ => t.palette.roles.surface_container_high.mix(c, 0.25),
        };
        Some(surface(Some(fill), None, t.on(fill), false))
    }

    fn snackbar(&self, t: &Theme) -> Option<(SurfaceTokens, Color)> {
        let r = &t.palette.roles;
        Some((
            surface(Some(r.inverse_surface), None, r.inverse_on_surface, true),
            r.inverse_primary,
        ))
    }

    fn dialog(&self, t: &Theme) -> Option<SurfaceTokens> {
        let r = &t.palette.roles;
        Some(surface(
            Some(r.surface_container_high),
            None,
            r.on_surface,
            true,
        ))
    }

    fn sheet(&self, t: &Theme) -> Option<SurfaceTokens> {
        let r = &t.palette.roles;
        Some(surface(
            Some(r.surface_container_low),
            None,
            r.on_surface,
            true,
        ))
    }

    fn popup(&self, t: &Theme) -> Option<SurfaceTokens> {
        let r = &t.palette.roles;
        Some(surface(Some(r.surface_container), None, r.on_surface, true))
    }

    fn scrim(&self, t: &Theme) -> Option<Color> {
        Some(t.palette.roles.scrim.with_alpha(0.32))
    }

    fn tab(&self, t: &Theme, active: bool, hovered: bool) -> Option<TabTokens> {
        // Primary tab: transparent, `primary` label + indicator when active.
        let p = &t.palette;
        let r = &p.roles;
        Some(TabTokens {
            background: hovered
                .then(|| if active { p.primary } else { r.on_surface }.with_alpha(0.08)),
            foreground: if active {
                t.legible(p.primary)
            } else {
                r.on_surface_variant
            },
            indicator: active.then_some(p.primary),
            outline: false,
        })
    }

    fn toggle(&self, t: &Theme, on: bool, state: ControlState) -> Option<ToggleTokens> {
        // Same as the baseline, but the unselected outline is the `outline` role.
        let p = &t.palette;
        if state == ControlState::Disabled {
            return None;
        }
        let lit = matches!(
            state,
            ControlState::Hovered | ControlState::Pressed | ControlState::Focused
        );
        if on {
            return None;
        }
        Some(ToggleTokens {
            track: p.surface_raised,
            thumb: if lit { p.text } else { p.text_muted },
            border: if lit {
                p.roles.on_surface
            } else {
                p.roles.outline
            },
        })
    }

    fn choice_toggle(
        &self,
        t: &Theme,
        kind: ToggleKind,
        on: bool,
        state: ControlState,
    ) -> Option<ToggleTokens> {
        let r = &t.palette.roles;
        let p = &t.palette;
        let clear = r.on_surface.with_alpha(0.0);
        let lit = matches!(
            state,
            ControlState::Hovered | ControlState::Pressed | ControlState::Focused
        );
        if state == ControlState::Disabled {
            let d = r.on_surface;
            return Some(match (kind, on) {
                (ToggleKind::Switch, true) => ToggleTokens {
                    track: d.with_alpha(0.12),
                    thumb: p.surface,
                    border: d.with_alpha(0.12),
                },
                (ToggleKind::Switch, false) => ToggleTokens {
                    track: r.surface_container_highest.with_alpha(0.12),
                    thumb: d.with_alpha(0.38),
                    border: d.with_alpha(0.12),
                },
                (ToggleKind::Checkbox, true) => ToggleTokens {
                    track: d.with_alpha(0.38),
                    thumb: p.surface,
                    border: d.with_alpha(0.38),
                },
                (_, _) => ToggleTokens {
                    track: clear,
                    thumb: if on { d.with_alpha(0.38) } else { clear },
                    border: d.with_alpha(0.38),
                },
            });
        }
        let outline = if lit {
            r.on_surface
        } else {
            r.on_surface_variant
        };
        Some(match (kind, on) {
            (ToggleKind::Switch, true) => ToggleTokens {
                track: p.primary,
                thumb: if lit {
                    r.primary_container
                } else {
                    p.on_primary
                },
                border: p.primary,
            },
            (ToggleKind::Switch, false) => ToggleTokens {
                track: r.surface_container_highest,
                thumb: if lit { r.on_surface_variant } else { r.outline },
                border: r.outline,
            },
            (ToggleKind::Checkbox, true) => ToggleTokens {
                track: p.primary,
                thumb: p.on_primary,
                border: p.primary,
            },
            (ToggleKind::Checkbox, false) | (ToggleKind::Radio, false) => ToggleTokens {
                track: clear,
                thumb: clear,
                border: outline,
            },
            (ToggleKind::Radio, true) => ToggleTokens {
                track: clear,
                thumb: p.primary,
                border: p.primary,
            },
        })
    }

    fn progress(&self, t: &Theme, tone: Option<Tone>) -> Option<ProgressTokens> {
        let p = &t.palette;
        let track = p.roles.surface_container_highest;
        Some(ProgressTokens {
            track,
            fill: tone.map_or(p.primary, |x| p.tone(x)),
            border: track,
            segments: 0,
        })
    }

    /// M3 navigation drawer / rail: 56 px pills, 360 px drawer, 80 px rail, 56×32 indicator.
    fn navigation(&self, t: &Theme) -> Option<NavigationTokens> {
        Some(NavigationTokens {
            item_height: 56.0,
            nested_height: 40.0,
            headline_height: 56.0,
            item_padding: 16.0,
            container_padding: 12.0,
            item_content_gap: 12.0,
            item_gap: 0.0,
            indent: 16.0,
            icon_size: t.sizes().icon,
            drawer_max_width: 360.0,
            rail_width: 80.0,
            indicator: (56.0, 32.0),
            rail_label_gap: 4.0,
            bar_height: 80.0,
            bar_indicator_width: 64.0,
            pill: true,
            container: Some(t.palette.roles.surface_container_low),
            roles: (
                TypeRole::LabelLarge,
                TypeRole::LabelMedium,
                TypeRole::LabelMedium,
                TypeRole::TitleSmall,
            ),
        })
    }

    /// Inactive destinations use `on-surface-variant`; the active one a secondary-container pill.
    fn navigation_item(&self, t: &Theme, selected: bool, hovered: bool) -> Option<ListItemTokens> {
        let r = &t.palette.roles;
        Some(if selected {
            ListItemTokens {
                background: Some(r.secondary_container),
                foreground: r.on_secondary_container,
                marker: None,
                outline: false,
            }
        } else {
            ListItemTokens {
                background: hovered.then(|| r.on_surface_variant.with_alpha(0.08)),
                foreground: r.on_surface_variant,
                marker: None,
                outline: false,
            }
        })
    }

    fn list_item(&self, t: &Theme, selected: bool, hovered: bool) -> Option<ListItemTokens> {
        let r = &t.palette.roles;
        Some(if selected {
            // The secondary-container pill; no accent bar.
            ListItemTokens {
                background: Some(r.secondary_container),
                foreground: r.on_secondary_container,
                marker: None,
                outline: false,
            }
        } else {
            ListItemTokens {
                background: hovered.then(|| r.on_surface.with_alpha(0.08)),
                foreground: r.on_surface,
                marker: None,
                outline: false,
            }
        })
    }

    fn divider(&self, t: &Theme) -> Option<Color> {
        Some(t.palette.roles.outline_variant)
    }

    /// Filled, tonal, outlined and text buttons.
    fn button(
        &self,
        t: &Theme,
        variant: ButtonVariant,
        state: ControlState,
    ) -> Option<ButtonTokens> {
        use ButtonVariant::*;
        use ControlState::*;
        let r = &t.palette.roles;
        let p = &t.palette;
        // (container, label, outlined)
        let (container, label, outlined) = match variant {
            Primary => (Some(p.primary), p.on_primary, false),
            Secondary => (Some(r.secondary_container), r.on_secondary_container, false),
            Danger => (Some(p.danger), r.on_error, false),
            Outline | Scrim => (None, t.legible(p.primary), true),
            OutlineSecondary => (None, t.legible(p.secondary), true),
            Ghost => (None, t.legible(p.primary), false),
        };
        if state == Disabled {
            let disabled = r.on_surface;
            return Some(ButtonTokens {
                background: container.map(|_| disabled.with_alpha(0.12)),
                foreground: disabled.with_alpha(0.38),
                border: outlined.then(|| disabled.with_alpha(0.12)),
            });
        }
        let l = layer(state);
        let background = match container {
            Some(c) => Some(c.mix(label, l)),
            None if l > 0.0 => Some(label.with_alpha(l)),
            None => None,
        };
        Some(ButtonTokens {
            background,
            foreground: label,
            border: outlined.then_some(if state == Focused { label } else { r.outline }),
        })
    }

    /// `surface-container-low` container, `primary` label (the caller adds the shadow).
    fn elevated_button(&self, t: &Theme, state: ControlState) -> Option<ButtonTokens> {
        let r = &t.palette.roles;
        if state == ControlState::Disabled {
            return Some(ButtonTokens {
                background: Some(r.on_surface.with_alpha(0.12)),
                foreground: r.on_surface.with_alpha(0.38),
                border: None,
            });
        }
        Some(ButtonTokens {
            background: Some(r.surface_container_low.mix(t.palette.primary, layer(state))),
            foreground: t.palette.primary,
            border: None,
        })
    }

    /// Container roles: primary / secondary / tertiary; Outline and Ghost use the surface FAB.
    fn fab(&self, t: &Theme, variant: ButtonVariant, state: ControlState) -> Option<ButtonTokens> {
        use ButtonVariant::*;
        let r = &t.palette.roles;
        let (bg, fg) = match variant {
            Primary => (r.primary_container, r.on_primary_container),
            Secondary => (r.secondary_container, r.on_secondary_container),
            Danger => (r.error_container, r.on_error_container),
            Outline | OutlineSecondary | Scrim => (r.tertiary_container, r.on_tertiary_container),
            Ghost => (r.surface_container_high, t.palette.primary),
        };
        Some(ButtonTokens {
            background: Some(bg.mix(fg, layer(state))),
            foreground: fg,
            border: None,
        })
    }

    /// Standard / outlined icons are `on-surface-variant` (`primary` when selected).
    fn icon_button(
        &self,
        t: &Theme,
        variant: ButtonVariant,
        state: ControlState,
        selected: bool,
        mut base: ButtonTokens,
    ) -> Option<ButtonTokens> {
        if state == ControlState::Disabled {
            return Some(base);
        }
        if matches!(
            variant,
            ButtonVariant::Ghost
                | ButtonVariant::Outline
                | ButtonVariant::OutlineSecondary
                | ButtonVariant::Scrim
        ) {
            let fg = if selected {
                t.palette.primary
            } else {
                t.palette.roles.on_surface_variant
            };
            let l = layer(state);
            base.foreground = fg;
            base.background = (l > 0.0).then(|| fg.with_alpha(l));
        }
        Some(base)
    }

    /// Outlined text field: `outline` border (`primary` when focused), `on-surface` text.
    fn input(&self, t: &Theme, focused: bool) -> Option<InputTokens> {
        let p = &t.palette;
        let r = &p.roles;
        Some(InputTokens {
            background: p.background,
            border: if focused { p.primary } else { r.outline },
            text: r.on_surface,
            placeholder: r.on_surface_variant,
            selection: p.selection,
        })
    }

    /// A 3 px `secondary` ring.
    fn focus_ring(&self, t: &Theme) -> Option<(Color, f32)> {
        Some((t.palette.secondary, 3.0))
    }

    /// One-line rows are 1.4 × the control height (56 px at 40), nested rows 1.0 ×.
    fn list_row_height(&self, t: &Theme, nested: bool) -> Option<f32> {
        Some(t.shape.control_height * if nested { 1.0 } else { 1.4 })
    }

    /// Each extra line adds 0.4 × the control height (one line 56, two 72, three 88 at 40).
    fn list_lines_height(&self, t: &Theme, lines: u8) -> Option<f32> {
        Some(
            t.list_row_height(false)
                + t.shape.control_height * 0.4 * (lines.clamp(1, 3) - 1) as f32,
        )
    }

    /// Menu rows are 1.2 × the control height (48 px at 40).
    fn menu_row_height(&self, t: &Theme) -> Option<f32> {
        Some(t.shape.control_height * 1.2)
    }

    /// Plain tooltips use the inverse surface, rich ones a tonal container.
    fn tooltip(&self, t: &Theme, rich: bool) -> Option<(Color, Color)> {
        let r = &t.palette.roles;
        Some(if rich {
            (r.surface_container, r.on_surface)
        } else {
            (r.inverse_surface, r.inverse_on_surface)
        })
    }

    /// Surface-coloured and borderless; tonal once scrolled. Heights 64 / 112 / 152, bottom 80.
    fn app_bar(&self, t: &Theme, kind: AppBarKind, scrolled: bool) -> Option<AppBarTokens> {
        let r = &t.palette.roles;
        let (height, title_size, background) = match kind {
            AppBarKind::CenterAligned | AppBarKind::Small => (64.0, 22.0, t.palette.background),
            AppBarKind::Medium => (112.0, 24.0, t.palette.background),
            AppBarKind::Large => (152.0, 28.0, t.palette.background),
            AppBarKind::Bottom => (80.0, 22.0, r.surface_container),
        };
        let background = if scrolled && kind != AppBarKind::Bottom {
            r.surface_container
        } else {
            background
        };
        Some(AppBarTokens {
            row_height: if kind == AppBarKind::Bottom {
                80.0
            } else {
                64.0
            },
            height,
            background: Some(background),
            border: None,
            title: r.on_surface,
            leading: if kind == AppBarKind::Bottom {
                r.on_surface_variant
            } else {
                r.on_surface
            },
            trailing: r.on_surface_variant,
            title_size,
            collapsed_title_size: 22.0,
            glow: false,
        })
    }

    /// The M3 type scale, exactly.
    fn type_style(&self, _t: &Theme, role: TypeRole) -> Option<TypeStyle> {
        // (size, line height, weight, tracking)
        const S: [(f32, f32, u16, f32); 15] = [
            (57.0, 64.0, 400, -0.25),
            (45.0, 52.0, 400, 0.0),
            (36.0, 44.0, 400, 0.0),
            (32.0, 40.0, 400, 0.0),
            (28.0, 36.0, 400, 0.0),
            (24.0, 32.0, 400, 0.0),
            (22.0, 28.0, 400, 0.0),
            (16.0, 24.0, 500, 0.15),
            (14.0, 20.0, 500, 0.1),
            (16.0, 24.0, 400, 0.5),
            (14.0, 20.0, 400, 0.25),
            (12.0, 16.0, 400, 0.4),
            (14.0, 20.0, 500, 0.1),
            (12.0, 16.0, 500, 0.5),
            (11.0, 16.0, 500, 0.5),
        ];
        let (size, line_height, weight, tracking) = S[role as usize];
        Some(TypeStyle {
            size,
            line_height,
            weight,
            tracking,
        })
    }

    /// M3 Expressive: XS 32, S 40, M 56, L 96, XL 136.
    fn control_metrics(&self, _t: &Theme, size: ControlSize) -> Option<SizeMetrics> {
        let i = size as usize;
        Some(SizeMetrics {
            height: [32.0, 40.0, 56.0, 96.0, 136.0][i],
            padding: [12.0, 16.0, 24.0, 48.0, 64.0][i],
            text_size: [14.0, 14.0, 16.0, 24.0, 32.0][i],
            icon_size: [20.0, 20.0, 24.0, 32.0, 40.0][i],
            square_radius: [12.0, 12.0, 16.0, 28.0, 28.0][i],
        })
    }

    fn button_padding(&self, _t: &Theme) -> Option<f32> {
        Some(24.0)
    }
}

/// Window size classes (lower bound in px, columns, gutter, margin), after the Material 3
/// layout guidance: compact, medium, expanded, large and extra large.
pub const WINDOW_CLASSES: [(f32, u32, f32, f32); 5] = [
    (0.0, 4, 8.0, 16.0),
    (600.0, 8, 16.0, 24.0),
    (840.0, 12, 16.0, 24.0),
    (1200.0, 12, 24.0, 24.0),
    (1600.0, 12, 24.0, 32.0),
];

/// Columns, gutter and margin of the window size class that holds `viewport`.
pub fn window_class(viewport: f32) -> (u32, f32, f32) {
    let c = WINDOW_CLASSES
        .iter()
        .rev()
        .find(|c| viewport >= c.0)
        .unwrap_or(&WINDOW_CLASSES[0]);
    (c.1, c.2, c.3)
}

#[cfg(test)]
mod tests {
    #[test]
    fn window_classes_grow_with_the_window() {
        assert_eq!(super::window_class(360.0).0, 4);
        assert_eq!(super::window_class(700.0).0, 8);
        assert_eq!(super::window_class(1000.0).0, 12);
        assert!(super::window_class(1700.0).2 >= super::window_class(500.0).2);
    }

    use super::*;
    use dodai_core::{Corner, ShapeRole};

    #[test]
    fn roles_map_as_expected() {
        let m3 = shape_system();
        assert_eq!(
            m3.spec(ShapeRole::Button).uniform_radius(),
            Some(dodai_core::FULL)
        );
        assert_eq!(m3.spec(ShapeRole::Card).uniform_radius(), Some(12.0));
        let input = m3.spec(ShapeRole::Input);
        assert_eq!(input.top_left, Corner::Round(4.0));
        assert_eq!(input.bottom_left, Corner::Square);
    }
}
