//! The [`Theme`] value: identity + palette + design tokens.

use crate::{Effects, Mode, Palette, Shape, Spacing, Typography};

/// Stable identity of a theme (safe to persist in config files).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThemeMeta {
    /// Machine id, e.g. `"cyberpunk"`; use for persistence / CSS `data-theme`.
    pub id: &'static str,
    /// Display name, e.g. `"Cyberpunk · Night City"`.
    pub name: &'static str,
    /// Theme family, e.g. `"cyberpunk"`.
    pub family: &'static str,
    pub description: &'static str,
    pub mode: Mode,
}

/// A complete design-system theme.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub meta: ThemeMeta,
    pub palette: Palette,
    pub typography: Typography,
    pub spacing: Spacing,
    pub shape: Shape,
    pub effects: Effects,
}

impl Theme {
    pub fn id(&self) -> &'static str {
        self.meta.id
    }
    pub fn name(&self) -> &'static str {
        self.meta.name
    }
    pub fn is_dark(&self) -> bool {
        self.meta.mode == Mode::Dark
    }
}

impl Default for Theme {
    fn default() -> Self {
        crate::themes::cyberpunk::night_city()
    }
}
