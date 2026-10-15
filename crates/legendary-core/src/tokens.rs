//! Non-colour design tokens: type, spacing, shape and effects.
//! This is what makes a *design system* rather than a colour switcher.

use crate::Color;

/// Typography tokens. Sizes are in logical pixels (TUI adapters ignore them).
#[derive(Debug, Clone, PartialEq)]
pub struct Typography {
    /// Display / heading family, comma-separated fallback list.
    pub display: &'static str,
    pub body: &'static str,
    pub mono: &'static str,
    pub size_xs: f32,
    pub size_sm: f32,
    pub size_md: f32,
    pub size_lg: f32,
    pub size_xl: f32,
    pub size_xxl: f32,
    /// Headings, buttons and labels render UPPERCASE.
    pub uppercase_labels: bool,
    /// Letter spacing for labels in `em`.
    pub label_tracking: f32,
}

/// Spacing scale in logical pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

/// Shape tokens.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// Plain corner radius (0 for hard-edged themes).
    pub radius: f32,
    /// Cut-corner size for chamfered panels/buttons (0 = none).
    pub chamfer: f32,
    pub border_width: f32,
    pub control_height: f32,
}

/// Glow / overlay effects.
#[derive(Debug, Clone, PartialEq)]
pub struct Effects {
    /// Glow colour for focus and primary elements.
    pub glow: Color,
    /// Glow blur radius in px (0 disables).
    pub glow_blur: f32,
    /// Draw CRT scanlines where the framework supports it.
    pub scanlines: bool,
    /// Hard offset shadow (px) typical of cyberpunk UI, 0 = none.
    pub hard_shadow: f32,
}
