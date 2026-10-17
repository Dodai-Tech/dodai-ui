//! The [`Theme`] value: identity + palette + design tokens.

use crate::{Effects, Mode, Palette, Shape, Spacing, Typography};

/// Stable identity of a theme (safe to persist in config files).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThemeMeta {
    /// Machine id, e.g. `"cyberpunk"`; use for persistence / CSS `data-theme`.
    pub id: &'static str,
    /// Display name, e.g. `"Cyberpunk · Yellow"`.
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
    /// Explicit shape language; `None` derives one from [`Theme::shape`].
    pub shape_system: Option<crate::ShapeSystem>,
}

impl Theme {
    /// Builder: set glow strength (0 = off, 1 = as designed, >1 = stronger).
    pub fn with_glow_intensity(mut self, intensity: f32) -> Self {
        self.effects.glow_intensity = intensity.max(0.0);
        self
    }

    /// Builder: turn the fine horizontal scanline texture on or off.
    pub fn with_scanlines(mut self, on: bool) -> Self {
        self.effects.scanlines = on;
        self
    }

    /// Builder: use this shape system (per-role corners, mix and match with
    /// any palette).
    pub fn with_shape_system(mut self, sys: crate::ShapeSystem) -> Self {
        self.shape_system = Some(sys);
        self
    }

    /// The shape system in effect (explicit, or derived from the legacy tokens).
    pub fn shape_system(&self) -> crate::ShapeSystem {
        self.shape_system
            .clone()
            .unwrap_or_else(|| crate::ShapeSystem::from_shape(&self.shape))
    }

    /// Resolved corners for a component role.
    pub fn shape_for(&self, role: crate::ShapeRole) -> crate::CornerSpec {
        self.shape_system().spec(role)
    }

    /// Builder: cut these corners on chamfered controls (see [`crate::Corners`]).
    pub fn with_corners(mut self, c: crate::Corners) -> Self {
        self.shape.corners = c;
        self
    }

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
