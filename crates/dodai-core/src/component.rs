//! Resolved, framework-agnostic component styles derived from a [`Theme`].

use crate::{Color, Theme, Tone};

/// Button emphasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// Solid brand fill.
    Primary,
    /// Solid secondary (cyan) fill.
    Secondary,
    /// Outlined brand colour on transparent.
    Outline,
    /// Outlined secondary colour on transparent.
    OutlineSecondary,
    /// Solid danger fill.
    Danger,
    /// Text only.
    Ghost,
}

/// Interaction state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ControlState {
    #[default]
    Idle,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

/// Fully resolved colours for one button in one state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonTokens {
    /// `None` = transparent.
    pub background: Option<Color>,
    pub foreground: Color,
    /// `None` = no border.
    pub border: Option<Color>,
}

/// Panel (card) tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelTokens {
    pub background: Color,
    pub border: Color,
    pub title: Color,
}

/// Text-input tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputTokens {
    pub background: Color,
    pub border: Color,
    pub text: Color,
    pub placeholder: Color,
    pub selection: Color,
}

/// Tag / badge tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagTokens {
    pub background: Option<Color>,
    pub foreground: Color,
    pub border: Option<Color>,
}

/// Tab tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabTokens {
    pub background: Option<Color>,
    pub foreground: Color,
    /// Underline / indicator colour (active tab).
    pub indicator: Option<Color>,
}

/// Toggle / checkbox / radio tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToggleTokens {
    pub track: Color,
    pub thumb: Color,
    pub border: Color,
}

/// Progress bar tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressTokens {
    pub track: Color,
    pub fill: Color,
    pub border: Color,
}

/// List / menu row tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListItemTokens {
    pub background: Option<Color>,
    pub foreground: Color,
    /// Accent bar at the row's left edge when selected.
    pub marker: Option<Color>,
}

/// Card emphasis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CardVariant {
    /// Raised surface with hard shadow / glow.
    #[default]
    Elevated,
    /// Flat raised surface, no border.
    Filled,
    /// Transparent with a border.
    Outlined,
}

/// Progress of one step in a stepper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Done,
    Current,
    Todo,
}

impl StepState {
    /// State of step `i` when `current` is the active step.
    pub fn of(i: usize, current: usize) -> Self {
        match i.cmp(&current) {
            std::cmp::Ordering::Less => Self::Done,
            std::cmp::Ordering::Equal => Self::Current,
            std::cmp::Ordering::Greater => Self::Todo,
        }
    }
}

/// A generic surface (card, chip, badge, alert, dialog, sheet, popup).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceTokens {
    /// `None` = transparent.
    pub background: Option<Color>,
    /// `None` = no border.
    pub border: Option<Color>,
    /// Text colour on the surface.
    pub foreground: Color,
    /// Draw the theme glow around it.
    pub glow: bool,
    /// Draw the theme shadow behind it.
    pub shadow: bool,
}

/// One stepper node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepTokens {
    pub fill: Option<Color>,
    pub stroke: Color,
    /// Colour of the number / check inside the node.
    pub mark: Color,
    /// Colour of the label.
    pub text: Color,
    pub glow: bool,
}

impl Theme {
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

    fn tonal(&self) -> bool {
        self.shape.kind() == crate::ShapeKind::Rounded
    }

    /// Card. Rounded (Material) themes use the tonal container roles.
    pub fn card(&self, v: CardVariant) -> SurfaceTokens {
        let p = &self.palette;
        let r = &p.roles;
        let tonal = self.tonal();
        let line = if tonal { r.outline_variant } else { p.border };
        match v {
            CardVariant::Elevated => Self::surface(
                Some(if tonal {
                    r.surface_container_low
                } else {
                    p.background
                }),
                Some(line),
                p.text,
                false,
                true,
            ),
            CardVariant::Filled => Self::surface(
                Some(if tonal {
                    r.surface_container_highest
                } else {
                    p.surface_raised
                }),
                None,
                p.text,
                false,
                false,
            ),
            CardVariant::Outlined => Self::surface(None, Some(line), p.text, false, false),
        }
    }

    /// Filter chip.
    pub fn chip(&self, selected: bool, hovered: bool) -> SurfaceTokens {
        let p = &self.palette;
        if selected && self.tonal() {
            let r = &p.roles;
            Self::surface(
                Some(r.secondary_container),
                None,
                r.on_secondary_container,
                false,
                false,
            )
        } else if selected {
            Self::surface(Some(p.selection), Some(p.primary), p.text, false, false)
        } else {
            Self::surface(
                hovered.then_some(p.surface_raised),
                Some(if hovered { p.border_strong } else { p.border }),
                if hovered { p.text } else { p.text_muted },
                false,
                false,
            )
        }
    }

    /// Solid status badge.
    pub fn badge(&self, tone: Tone) -> SurfaceTokens {
        let c = self.palette.tone(tone);
        Self::surface(Some(c), None, self.on(c), false, false)
    }

    /// Alert: tinted and outlined, or a solid banner.
    pub fn alert(&self, tone: Tone, banner: bool) -> SurfaceTokens {
        let c = self.palette.tone(tone);
        if banner && self.shape.kind() == crate::ShapeKind::Rounded {
            // Rounded (Material): a tonal container, not a saturated slab.
            let fill = match tone {
                Tone::Danger => self.palette.roles.error_container,
                _ => self.palette.roles.surface_container_high.mix(c, 0.25),
            };
            Self::surface(Some(fill), None, self.on(fill), false, false)
        } else if banner {
            Self::surface(Some(c), None, self.on(c), false, false)
        } else {
            Self::surface(
                Some(c.with_alpha(0.12)),
                Some(c),
                self.palette.text,
                false,
                false,
            )
        }
    }

    /// Snackbar: `(surface, action colour)`. Rounded (Material) uses the inverse surface.
    pub fn snackbar(&self) -> (SurfaceTokens, Color) {
        let p = &self.palette;
        if self.shape.kind() == crate::ShapeKind::Rounded {
            (
                Self::surface(
                    Some(p.roles.inverse_surface),
                    None,
                    p.roles.inverse_on_surface,
                    false,
                    true,
                ),
                p.roles.inverse_primary,
            )
        } else {
            let bg = p.surface_raised;
            (
                Self::surface(Some(bg), None, p.text, false, false),
                if p.primary.contrast(bg) >= 3.0 {
                    p.primary
                } else {
                    p.text
                },
            )
        }
    }

    /// Dialog box.
    pub fn dialog(&self) -> SurfaceTokens {
        let p = &self.palette;
        Self::surface(Some(p.surface), Some(p.primary), p.text, true, true)
    }

    /// Side / bottom sheet.
    pub fn sheet(&self) -> SurfaceTokens {
        let p = &self.palette;
        Self::surface(Some(p.surface), Some(p.border_strong), p.text, false, true)
    }

    /// Floating popup (menu, dropdown list).
    pub fn popup(&self) -> SurfaceTokens {
        let p = &self.palette;
        Self::surface(
            Some(p.surface_raised),
            Some(p.border_strong),
            p.text,
            true,
            true,
        )
    }

    /// Modal scrim: the page colour, never fully opaque.
    pub fn scrim(&self) -> Color {
        self.palette.background.with_alpha(0.7)
    }

    /// Stepper node.
    pub fn step(&self, state: StepState) -> StepTokens {
        let p = &self.palette;
        match state {
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
        }
    }

    /// Tag / badge: `solid` fills with the tone, otherwise outlined.
    pub fn tag(&self, tone: Tone, solid: bool) -> TagTokens {
        let c = self.palette.tone(tone);
        if solid {
            TagTokens {
                background: Some(c),
                foreground: self.on(c),
                border: None,
            }
        } else {
            TagTokens {
                background: Some(c.with_alpha(0.12)),
                foreground: c,
                border: Some(c),
            }
        }
    }

    /// Tab: the active tab is highlighted with an indicator.
    pub fn tab(&self, active: bool, hovered: bool) -> TabTokens {
        let p = &self.palette;
        if active {
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
        }
    }

    /// Toggle / checkbox / radio.
    pub fn toggle(&self, on: bool, state: ControlState) -> ToggleTokens {
        let p = &self.palette;
        if state == ControlState::Disabled {
            return ToggleTokens {
                track: p.surface_raised,
                thumb: p.text_disabled,
                border: p.border,
            };
        }
        let lit = matches!(
            state,
            ControlState::Hovered | ControlState::Pressed | ControlState::Focused
        );
        if on {
            ToggleTokens {
                track: if lit { p.primary_hover } else { p.primary },
                thumb: p.on_primary,
                border: p.primary,
            }
        } else {
            ToggleTokens {
                track: p.surface_raised,
                thumb: if lit { p.text } else { p.text_muted },
                // Rounded (Material): the unselected outline is the `outline` role.
                border: if self.shape.kind() == crate::ShapeKind::Rounded {
                    if lit {
                        p.roles.on_surface
                    } else {
                        p.roles.outline
                    }
                } else if lit {
                    p.border_strong
                } else {
                    p.border
                },
            }
        }
    }

    /// Progress bar in a status tone (`None` = brand colour).
    pub fn progress(&self, tone: Option<Tone>) -> ProgressTokens {
        let p = &self.palette;
        ProgressTokens {
            track: p.surface_raised,
            fill: tone.map_or(p.primary, |t| p.tone(t)),
            border: p.border,
        }
    }

    /// List / menu row.
    pub fn list_item(&self, selected: bool, hovered: bool) -> ListItemTokens {
        let p = &self.palette;
        let rounded = self.shape.kind() == crate::ShapeKind::Rounded;
        if selected {
            ListItemTokens {
                // Rounded (Material): the secondary-container pill.
                background: Some(if rounded {
                    p.roles.secondary_container
                } else {
                    p.selection
                }),
                foreground: if rounded {
                    p.roles.on_secondary_container
                } else {
                    p.text
                },
                // Chamfered themes mark the selection with an accent bar;
                // rounded (Material) ones rely on the filled pill alone.
                marker: (self.shape.kind() != crate::ShapeKind::Rounded).then_some(p.primary),
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
        }
    }

    /// Divider / separator colour.
    pub fn divider(&self) -> Color {
        if self.shape.kind() == crate::ShapeKind::Rounded {
            self.palette.roles.outline_variant
        } else {
            self.palette.border
        }
    }

    /// Resolve a button style.
    pub fn button(&self, variant: ButtonVariant, state: ControlState) -> ButtonTokens {
        let p = &self.palette;
        use ButtonVariant::*;
        use ControlState::*;
        if state == Disabled {
            return ButtonTokens {
                background: Some(p.surface_raised),
                foreground: p.text_disabled,
                border: None,
            };
        }
        let lit = matches!(state, Hovered | Pressed | Focused);
        let (solid, solid_hi, solid_lo, on) = match variant {
            Primary => (p.primary, p.primary_hover, p.primary_active, p.on_primary),
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
                self.on(p.danger),
            ),
            _ => (p.primary, p.primary_hover, p.primary_active, p.on_primary),
        };
        match variant {
            Primary | Secondary | Danger => ButtonTokens {
                background: Some(match state {
                    Pressed => solid_lo,
                    Hovered | Focused => solid_hi,
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
                    // Cyberpunk "fill on hover": outline inverts to solid.
                    background: if lit { Some(c) } else { None },
                    foreground: if lit { self.on(c) } else { self.legible(c) },
                    border: Some(c),
                }
            }
            Ghost => ButtonTokens {
                background: if lit { Some(p.surface_raised) } else { None },
                foreground: if lit { self.legible(p.primary) } else { p.text },
                border: None,
            },
        }
    }

    /// `c` as text on the page background: itself when it holds 3:1 contrast,
    /// otherwise the theme's text colour (a pale brand colour on a light page).
    pub fn legible(&self, c: Color) -> Color {
        if c.contrast(self.palette.background) >= 3.0 {
            c
        } else {
            self.palette.text
        }
    }

    /// Readable text colour on top of `bg`.
    pub fn on(&self, bg: Color) -> Color {
        let p = &self.palette;
        let best = if bg.contrast(p.background) >= bg.contrast(p.text) {
            p.background
        } else {
            p.text
        };
        if best.contrast(bg) >= 4.5 {
            return best;
        }
        // Neither theme colour reads well enough (a mid-tone fill): use black or white.
        let (black, white) = (Color::hex(0x000000), Color::hex(0xffffff));
        if black.contrast(bg) >= white.contrast(bg) {
            black
        } else {
            white
        }
    }

    /// Resolve a panel style; `focused` highlights the border.
    pub fn panel(&self, focused: bool) -> PanelTokens {
        let p = &self.palette;
        PanelTokens {
            background: p.surface,
            border: if focused { p.primary } else { p.border },
            title: if focused {
                self.legible(p.primary)
            } else {
                p.text_muted
            },
        }
    }

    /// Resolve a text-input style.
    pub fn input(&self, focused: bool) -> InputTokens {
        let p = &self.palette;
        InputTokens {
            background: p.background,
            border: if focused {
                p.secondary
            } else {
                p.border_strong
            },
            text: p.text,
            placeholder: p.text_disabled,
            selection: p.selection,
        }
    }
}
