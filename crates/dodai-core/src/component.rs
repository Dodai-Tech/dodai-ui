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

impl Theme {
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
                border: if lit { p.border_strong } else { p.border },
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
        if selected {
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
        }
    }

    /// Divider / separator colour.
    pub fn divider(&self) -> Color {
        self.palette.border
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
        if bg.contrast(p.background) >= bg.contrast(p.text) {
            p.background
        } else {
            p.text
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
