//! The Cyberpunk **design-system spec**: hard-edged panels with glow, an accent bar on the
//! selected row, outlines that invert to solid on hover, and a solid active tab.
//!
//! Core's baseline is deliberately plain; everything that gives Cyberpunk its character is here.

use dodai_core::{
    ButtonTokens, ButtonVariant, CardVariant, Color, ControlState, DesignSpec, InputTokens,
    ListItemTokens, StepState, StepTokens, SurfaceTokens, TabTokens, Theme,
};

/// The Cyberpunk spec (attach with [`dodai_core::Theme::with_spec`]).
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
    fn name(&self) -> &'static str {
        "Cyberpunk"
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
            }
        } else if hovered {
            TabTokens {
                background: Some(p.surface_raised),
                foreground: p.text,
                indicator: None,
            }
        } else {
            TabTokens {
                background: None,
                foreground: p.text_muted,
                indicator: None,
            }
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
            }
        } else if hovered {
            ListItemTokens {
                background: Some(p.surface_raised),
                foreground: p.text,
                marker: None,
            }
        } else {
            ListItemTokens {
                background: None,
                foreground: p.text_muted,
                marker: None,
            }
        })
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
                    background: lit.then_some(c),
                    foreground: if lit { t.on(c) } else { t.legible(c) },
                    border: Some(c),
                }
            }
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
        for t in crate::variants() {
            assert_eq!(t.spec.name(), "Cyberpunk", "{}", t.id());
        }
    }

    #[test]
    fn glow_accent_bar_and_inversion() {
        for t in crate::variants() {
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
            assert_eq!(hovered.background, Some(t.palette.primary));
            assert_eq!(t.scrim(), t.palette.background.with_alpha(0.7));
        }
    }
}
