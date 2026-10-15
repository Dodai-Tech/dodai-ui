//! Resolved, framework-agnostic component styles derived from a [`Theme`].

use crate::{Color, Theme};

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

impl Theme {
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
                p.text,
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
                    foreground: if lit { self.on(c) } else { c },
                    border: Some(c),
                }
            }
            Ghost => ButtonTokens {
                background: if lit { Some(p.surface_raised) } else { None },
                foreground: if lit { p.primary } else { p.text },
                border: None,
            },
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
            title: if focused { p.primary } else { p.text_muted },
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
