//! Semantic colour tokens. Adapters never see raw brand colours, only roles.

use crate::Color;

/// Light vs dark base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Dark,
    Light,
}

/// Semantic status colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tone {
    Info,
    Success,
    Warning,
    Danger,
}

/// Every colour role a UI needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    // Surfaces (darkest → lightest)
    pub background: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub overlay: Color,
    // Lines
    pub border: Color,
    pub border_strong: Color,
    // Text
    pub text: Color,
    pub text_muted: Color,
    pub text_disabled: Color,
    // Brand
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_active: Color,
    pub on_primary: Color,
    pub secondary: Color,
    pub secondary_hover: Color,
    pub on_secondary: Color,
    pub accent: Color,
    // Status
    pub info: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    // Interaction
    pub selection: Color,
    pub focus_ring: Color,
    pub link: Color,
    /// Categorical series colours (charts, graph lanes, tags).
    pub series: [Color; 8],
}

impl Palette {
    /// Colour for a status [`Tone`].
    pub fn tone(&self, tone: Tone) -> Color {
        match tone {
            Tone::Info => self.info,
            Tone::Success => self.success,
            Tone::Warning => self.warning,
            Tone::Danger => self.danger,
        }
    }

    /// Series colour, wrapping around.
    pub fn series_at(&self, i: usize) -> Color {
        self.series[i % self.series.len()]
    }
}
