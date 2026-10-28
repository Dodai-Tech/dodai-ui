//! The Cyberpunk **design-system spec**: hard-edged panels with glow, an accent bar on the
//! selected row, outlines that invert to solid on hover, and a solid active tab.
//!
//! Core's baseline is deliberately plain; everything that gives Cyberpunk its character is here.

use crate::{
    AppBarKind, AppBarTokens, ButtonTokens, ButtonVariant, CardVariant, Color, ControlSize,
    ControlState, DesignSpec, InputTokens, ListItemTokens, Look, NavigationTokens, ProgressTokens,
    SizeMetrics, StepState, StepTokens, SurfaceTokens, TabTokens, Theme, ToggleTokens, Tone,
    TypeRole,
};

/// Frosted glass: blur sigma in px and tint opacity. Dark glass lets the neon behind it bleed
/// through, so it is the most transparent of the designs.
const FROST_BLUR: f32 = 16.0;
const FROST_ALPHA: f32 = 0.55;
/// Hairline neon edge of a frosted panel, px.
const FROST_EDGE: f32 = 1.0;

/// The Cyberpunk spec (attach with [`crate::Theme::with_spec`]).
#[derive(Debug)]
pub struct Cyberpunk;

/// Shared instance for `Theme::with_spec`.
pub static SPEC: Cyberpunk = Cyberpunk;

fn surface(
    background: Option<Color>,
    border: Option<Color>,
    foreground: Color,
    glow: bool,
    shadow: bool,
) -> SurfaceTokens {
    SurfaceTokens {
        background,
        border,
        foreground,
        glow,
        shadow,
    }
}

impl DesignSpec for Cyberpunk {
    fn look(&self) -> Look {
        Look {
            luminous: true,
            cut_corners: true,
            accent_border: true,
            selection: crate::motion::SelectionMotion::Glitch,
            ..Look::default()
        }
    }

    fn name(&self) -> &'static str {
        "Cyberpunk"
    }

    /// Technical: dashed gridlines under thin neon lines.
    fn chart_style(&self, t: &Theme) -> Option<crate::ChartStyle> {
        Some(crate::ChartStyle {
            grid_dashed: true,
            dial: crate::DialLook::Segmented,
            cluster: crate::ClusterKind::Cut,
            scope: crate::ScopeLook::Neon,
            response: crate::NeedleResponse::Snappy,
            grid_color: t.palette.border.with_alpha(0.7),
            ..crate::ChartStyle::from_theme(t)
        })
    }

    /// Smoked glass: the page-dark surface at 55% over a 16 px blur, with a one-pixel neon edge.
    fn frost(&self, t: &Theme) -> Option<crate::Frost> {
        Some(crate::Frost {
            tint: t.palette.surface.with_alpha(FROST_ALPHA),
            blur: FROST_BLUR,
            edge: Some((t.palette.primary, FROST_EDGE)),
            saturation: 1.0,
        })
    }

    /// Elevated: page-coloured slab with a hard shadow; filled: raised, borderless.
    fn card(&self, t: &Theme, v: CardVariant) -> Option<SurfaceTokens> {
        let p = &t.palette;
        Some(match v {
            CardVariant::Elevated => {
                surface(Some(p.background), Some(p.border), p.text, false, true)
            }
            CardVariant::Filled => surface(Some(p.surface_raised), None, p.text, false, false),
            CardVariant::Outlined => surface(None, Some(p.border), p.text, false, false),
        })
    }

    /// Selected chips are outlined in the brand colour.
    fn chip(&self, t: &Theme, selected: bool, hovered: bool) -> Option<SurfaceTokens> {
        let p = &t.palette;
        Some(if selected {
            surface(Some(p.selection), Some(p.primary), p.text, false, false)
        } else {
            surface(
                hovered.then_some(p.surface_raised),
                Some(if hovered { p.border_strong } else { p.border }),
                if hovered { p.text } else { p.text_muted },
                false,
                false,
            )
        })
    }

    /// Brand-outlined, glowing dialog.
    fn dialog(&self, t: &Theme) -> Option<SurfaceTokens> {
        let p = &t.palette;
        Some(surface(
            Some(p.surface),
            Some(p.primary),
            p.text,
            true,
            true,
        ))
    }

    fn sheet(&self, t: &Theme) -> Option<SurfaceTokens> {
        let p = &t.palette;
        Some(surface(
            Some(p.surface),
            Some(p.border_strong),
            p.text,
            false,
            true,
        ))
    }

    fn popup(&self, t: &Theme) -> Option<SurfaceTokens> {
        let p = &t.palette;
        Some(surface(
            Some(p.surface_raised),
            Some(p.border_strong),
            p.text,
            true,
            true,
        ))
    }

    /// The page colour at 70 %, so the scene dims into the background.
    fn scrim(&self, t: &Theme) -> Option<Color> {
        Some(t.palette.background.with_alpha(0.7))
    }

    /// Done = solid, current = glowing, todo = hollow.
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
                fill: Some(p.selection),
                stroke: p.primary,
                mark: p.text,
                text: p.text,
                glow: true,
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

    /// The active tab is a solid brand block.
    fn tab(&self, t: &Theme, active: bool, hovered: bool) -> Option<TabTokens> {
        let p = &t.palette;
        Some(if active {
            TabTokens {
                background: Some(p.primary),
                foreground: p.on_primary,
                indicator: Some(p.primary),
                outline: false,
            }
        } else if hovered {
            TabTokens {
                background: Some(p.surface_raised),
                foreground: p.text,
                indicator: None,
                outline: false,
            }
        } else {
            TabTokens {
                background: None,
                foreground: p.text_muted,
                indicator: None,
                outline: false,
            }
        })
    }

    /// Cyberpunk drawer / rail geometry, fixed in px (control height 48.0).
    fn navigation(&self, t: &Theme) -> Option<NavigationTokens> {
        Some(NavigationTokens {
            item_height: 38.4,
            nested_height: 31.2,
            headline_height: 38.4,
            item_padding: 16.0,
            container_padding: 8.0,
            item_content_gap: 8.0,
            item_gap: 8.0,
            indent: 16.0,
            icon_size: t.sizes().icon_sm,
            drawer_max_width: 360.0,
            rail_width: 80.0,
            indicator: (84.0, 48.0),
            rail_label_gap: 8.0,
            bar_height: 80.0,
            bar_indicator_width: 92.0,
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

    fn divider(&self, t: &Theme) -> Option<Color> {
        Some(t.palette.border)
    }

    fn fab(&self, t: &Theme, v: ButtonVariant, s: ControlState) -> Option<ButtonTokens> {
        self.button(t, v, s)
    }

    /// XS to XL control sizes, fixed in px.
    fn control_metrics(&self, t: &Theme, size: ControlSize) -> Option<SizeMetrics> {
        let i = size as usize;
        let ty = &t.typography;
        Some(SizeMetrics {
            height: [36.0, 48.0, 60.0, 76.8, 96.0][i],
            padding: [12.0, 16.0, 20.0, 28.0, 36.0][i],
            text_size: [ty.size_sm, ty.size_md, ty.size_lg, ty.size_xl, ty.size_xxl][i],
            icon_size: [21.6, 24.0, 24.0, 26.4, 31.2][i],
            square_radius: [12.0, 12.0, 16.0, 28.0, 28.0][i],
        })
    }

    /// The selected row carries an accent bar at its edge.
    fn list_item(&self, t: &Theme, selected: bool, hovered: bool) -> Option<ListItemTokens> {
        let p = &t.palette;
        Some(if selected {
            ListItemTokens {
                background: Some(p.selection),
                foreground: p.text,
                marker: Some(p.primary),
                outline: false,
            }
        } else if hovered {
            ListItemTokens {
                background: Some(p.surface_raised),
                foreground: p.text,
                marker: None,
                outline: false,
            }
        } else {
            ListItemTokens {
                background: None,
                foreground: p.text_muted,
                marker: None,
                outline: false,
            }
        })
    }

    /// A HUD header: raised plate, neon title, cyan icons; it lights up (border + glow) once the
    /// content scrolls beneath it. Compact rows (56) with 104 / 136 for the tall variants.
    fn app_bar(&self, t: &Theme, kind: AppBarKind, scrolled: bool) -> Option<AppBarTokens> {
        let p = &t.palette;
        let (row, height, title) = match kind {
            AppBarKind::CenterAligned | AppBarKind::Small => (56.0, 56.0, 20.0),
            AppBarKind::Medium => (56.0, 104.0, 26.0),
            AppBarKind::Large => (56.0, 136.0, 32.0),
            AppBarKind::Bottom => (64.0, 64.0, 18.0),
        };
        let lit = scrolled && kind != AppBarKind::Bottom;
        Some(AppBarTokens {
            row_height: row,
            height,
            background: Some(if kind == AppBarKind::Bottom {
                p.surface_raised
            } else {
                p.surface
            }),
            border: Some(if lit { p.primary } else { p.border_strong }),
            title: t.legible(p.primary),
            leading: t.legible(p.secondary),
            trailing: p.text,
            title_size: title,
            collapsed_title_size: 20.0,
            glow: lit,
        })
    }

    /// Outlined, tinted tags with a glow instead of solid blobs.
    fn badge(&self, t: &Theme, tone: Tone) -> Option<SurfaceTokens> {
        let c = t.legible(t.palette.tone(tone));
        Some(surface(Some(c.with_alpha(0.16)), Some(c), c, true, false))
    }

    /// Hollow switches outlined in the secondary colour; lit solid when on.
    fn toggle(&self, t: &Theme, on: bool, state: ControlState) -> Option<ToggleTokens> {
        let p = &t.palette;
        if state == ControlState::Disabled {
            return None;
        }
        let lit = matches!(
            state,
            ControlState::Hovered | ControlState::Pressed | ControlState::Focused
        );
        Some(if on {
            ToggleTokens {
                track: if lit { p.primary_hover } else { p.primary },
                thumb: p.on_primary,
                border: p.primary,
            }
        } else {
            let edge = t.legible(p.secondary);
            ToggleTokens {
                track: p.background,
                thumb: if lit { edge } else { p.text_muted },
                border: if lit { edge } else { p.border_strong },
            }
        })
    }

    /// A segmented HUD gauge.
    fn progress(&self, t: &Theme, tone: Option<Tone>) -> Option<ProgressTokens> {
        let p = &t.palette;
        Some(ProgressTokens {
            track: p.surface_raised,
            fill: tone.map_or(p.primary, |x| p.tone(x)),
            border: p.border_strong,
            segments: 10,
        })
    }

    /// A heavier ring in the secondary colour.
    fn focus_ring(&self, t: &Theme) -> Option<(Color, f32)> {
        Some((t.legible(t.palette.secondary), 3.0))
    }

    /// Readout-style tooltips: raised plate, cyan text.
    fn tooltip(&self, t: &Theme, rich: bool) -> Option<(Color, Color)> {
        let p = &t.palette;
        Some(if rich {
            (p.surface_raised, p.text)
        } else {
            (p.surface_raised, t.legible(p.secondary))
        })
    }

    /// A glowing, brand-outlined notification plate.
    fn snackbar(&self, t: &Theme) -> Option<(SurfaceTokens, Color)> {
        let p = &t.palette;
        Some((
            surface(
                Some(p.surface_raised),
                Some(t.legible(p.primary)),
                p.text,
                true,
                true,
            ),
            t.legible(p.secondary),
        ))
    }

    /// Tinted alerts glow in their tone; banners stay solid.
    fn alert(&self, t: &Theme, tone: Tone, banner: bool) -> Option<SurfaceTokens> {
        if banner {
            return None;
        }
        let c = t.palette.tone(tone);
        Some(surface(
            Some(c.with_alpha(0.12)),
            Some(c),
            t.palette.text,
            true,
            false,
        ))
    }

    /// Solid fills brighten, outlines invert to solid on hover.
    fn button(
        &self,
        t: &Theme,
        variant: ButtonVariant,
        state: ControlState,
    ) -> Option<ButtonTokens> {
        use ButtonVariant::*;
        use ControlState::*;
        let p = &t.palette;
        if state == Disabled {
            return Some(ButtonTokens {
                background: Some(p.surface_raised),
                foreground: p.text_disabled,
                border: None,
            });
        }
        let lit = matches!(state, Hovered | Pressed | Focused);
        let (solid, hi, lo, on) = match variant {
            Secondary => (
                p.secondary,
                p.secondary_hover,
                p.secondary.darken(0.2),
                p.on_secondary,
            ),
            Danger => (
                p.danger,
                p.danger.lighten(0.15),
                p.danger.darken(0.2),
                t.on(p.danger),
            ),
            _ => (p.primary, p.primary_hover, p.primary_active, p.on_primary),
        };
        Some(match variant {
            Primary | Secondary | Danger => ButtonTokens {
                background: Some(match state {
                    Pressed => lo,
                    Hovered | Focused => hi,
                    _ => solid,
                }),
                foreground: on,
                border: None,
            },
            Outline | OutlineSecondary => {
                let c = if variant == Outline {
                    p.primary
                } else {
                    p.secondary
                };
                ButtonTokens {
                    // A faint tint on hover; the outline stays and the glow does the rest.
                    background: lit
                        .then(|| c.with_alpha(if state == Pressed { 0.3 } else { 0.14 })),
                    foreground: t.legible(c),
                    border: Some(c),
                }
            }
            Scrim => ButtonTokens {
                background: Some(if lit {
                    p.primary
                } else {
                    p.background.with_alpha(0.5)
                }),
                foreground: if lit {
                    p.on_primary
                } else {
                    t.legible(p.primary)
                },
                border: Some(p.primary),
            },
            Ghost => ButtonTokens {
                background: lit.then_some(p.surface_raised),
                foreground: if lit { t.legible(p.primary) } else { p.text },
                border: None,
            },
        })
    }

    /// The focused field is outlined in the secondary (cyan) colour.
    fn input(&self, t: &Theme, focused: bool) -> Option<InputTokens> {
        let p = &t.palette;
        Some(InputTokens {
            background: p.background,
            border: if focused {
                p.secondary
            } else {
                p.border_strong
            },
            text: p.text,
            placeholder: p.text_disabled,
            selection: p.selection,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_attaches_the_spec() {
        for t in crate::cyberpunk::variants() {
            assert_eq!(t.spec.name(), "Cyberpunk", "{}", t.id());
        }
    }

    #[test]
    fn frost_is_smoked_glass_with_a_neon_edge() {
        for t in crate::cyberpunk::variants() {
            let f = t.frost();
            assert_eq!(f.tint.a, (FROST_ALPHA * 255.0).round() as u8, "{}", t.id());
            assert_eq!(f.blur, FROST_BLUR);
            assert_eq!(f.edge, Some((t.palette.primary, FROST_EDGE)));
        }
    }

    #[test]
    fn glow_accent_bar_and_inversion() {
        for t in crate::cyberpunk::variants() {
            assert!(t.dialog().glow && t.popup().glow);
            assert!(t.list_item(true, false).marker.is_some());
            assert!(t.step(StepState::Current).glow);
            assert_eq!(
                t.tab(true, false).background,
                Some(t.palette.primary),
                "{}",
                t.id()
            );
            let hovered = t.button(ButtonVariant::Outline, ControlState::Hovered);
            assert_eq!(hovered.background, Some(t.palette.primary.with_alpha(0.14)));
            assert_eq!(t.scrim(), t.palette.background.with_alpha(0.7));
        }
    }

    #[test]
    fn app_bars_are_compact_huds_that_light_up_when_scrolled() {
        use crate::AppBarKind::*;
        for t in crate::cyberpunk::variants() {
            for k in [CenterAligned, Small, Medium, Large] {
                let rest = t.app_bar(k, false);
                let lit = t.app_bar(k, true);
                assert!(!rest.glow && lit.glow, "{} {k:?}", t.id());
                assert_eq!(lit.border, Some(t.palette.primary));
                assert!(rest.border.is_some(), "HUD plates keep their outline");
                assert!(rest.row_height < 64.0, "compact rows");
            }
            assert!(t.app_bar(Large, false).height > t.app_bar(Medium, false).height);
            assert!(!t.app_bar(Bottom, true).glow);
        }
    }

    #[test]
    fn presets_ship_a_dark_and_a_light_theme_each() {
        let all = crate::cyberpunk::preset_themes();
        assert_eq!(all.len(), 2 * crate::cyberpunk::PRESETS.len());
        let ids: std::collections::HashSet<_> = all.iter().map(|t| t.id()).collect();
        assert_eq!(ids.len(), all.len(), "ids are unique");
    }

    #[test]
    fn plain_look_and_raised_surfaces() {
        use crate::{CardVariant, Look};
        for t in crate::cyberpunk::variants() {
            assert_eq!(
                t.look(),
                Look {
                    luminous: true,
                    cut_corners: true,
                    accent_border: true,
                    selection: crate::motion::SelectionMotion::Glitch,
                    ..Look::default()
                },
                "{}",
                t.id()
            );
            assert_eq!(
                t.card(CardVariant::Filled).background,
                Some(t.palette.surface_raised),
                "{}",
                t.id()
            );
            assert_eq!(t.tooltip(false).0, t.palette.surface_raised, "{}", t.id());
            assert_eq!(t.divider(), t.palette.border, "{}", t.id());
        }
    }
}
