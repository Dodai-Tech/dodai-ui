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

/// Which corners of a control are cut (chamfered).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Corners {
    pub top_left: bool,
    pub top_right: bool,
    pub bottom_right: bool,
    pub bottom_left: bool,
}

impl Corners {
    pub const NONE: Self = Self {
        top_left: false,
        top_right: false,
        bottom_right: false,
        bottom_left: false,
    };
    /// Only bottom-right (cyberpunk.net buttons).
    pub const BOTTOM_RIGHT: Self = Self {
        bottom_right: true,
        ..Self::NONE
    };
    /// Top-left and bottom-right (diagonal).
    pub const DIAGONAL: Self = Self {
        top_left: true,
        bottom_right: true,
        ..Self::NONE
    };
    pub const ALL: Self = Self {
        top_left: true,
        top_right: true,
        bottom_right: true,
        bottom_left: true,
    };

    pub fn any(self) -> bool {
        self.top_left || self.top_right || self.bottom_right || self.bottom_left
    }
}

/// How a theme draws control outlines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    /// Hard 90° corners.
    Square,
    /// Rounded corners (`Shape::radius`).
    Rounded,
    /// Cut corners (`Shape::chamfer` on `Shape::corners`).
    Chamfered,
}

/// Shape tokens.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// Which corners are chamfered when `chamfer > 0`.
    pub corners: Corners,
    /// Plain corner radius (0 for hard-edged themes).
    pub radius: f32,
    /// Cut-corner size for chamfered panels/buttons (0 = none).
    pub chamfer: f32,
    pub border_width: f32,
    pub control_height: f32,
}

impl Shape {
    /// Outline style implied by the tokens.
    pub fn kind(&self) -> ShapeKind {
        if self.chamfer > 0.0 && self.corners.any() {
            ShapeKind::Chamfered
        } else if self.radius > 0.0 {
            ShapeKind::Rounded
        } else {
            ShapeKind::Square
        }
    }

    /// Outline polygon (clockwise from top-left) for a `w`×`h` box with this
    /// shape's chamfer, clamped to a third of the shorter side. Shared by every
    /// adapter so all frameworks cut identical corners. For non-chamfered shapes
    /// this is the plain rectangle.
    pub fn outline(&self, w: f32, h: f32) -> Vec<(f32, f32)> {
        let c = if self.kind() == ShapeKind::Chamfered {
            self.chamfer.min(w.min(h) / 3.0).max(0.0)
        } else {
            0.0
        };
        let k = self.corners;
        let mut p = Vec::with_capacity(8);
        if k.top_left && c > 0.0 {
            p.push((0.0, c));
            p.push((c, 0.0));
        } else {
            p.push((0.0, 0.0));
        }
        if k.top_right && c > 0.0 {
            p.push((w - c, 0.0));
            p.push((w, c));
        } else {
            p.push((w, 0.0));
        }
        if k.bottom_right && c > 0.0 {
            p.push((w, h - c));
            p.push((w - c, h));
        } else {
            p.push((w, h));
        }
        if k.bottom_left && c > 0.0 {
            p.push((c, h));
            p.push((0.0, h - c));
        } else {
            p.push((0.0, h));
        }
        p
    }
}

/// Glow / overlay effects.
#[derive(Debug, Clone, PartialEq)]
pub struct Effects {
    /// Glow colour for focus and primary elements.
    pub glow: Color,
    /// Glow blur radius in px at intensity 1.0 (0 disables).
    pub glow_blur: f32,
    /// Glow strength multiplier: scales both alpha and radius. `0.0` = off,
    /// `1.0` = as designed, `>1.0` = stronger (alpha is clamped to opaque).
    pub glow_intensity: f32,
    /// Draw CRT scanlines where the framework supports it.
    pub scanlines: bool,
    /// Hard offset shadow (px) typical of cyberpunk UI, 0 = none.
    pub hard_shadow: f32,
}

impl Effects {
    /// Glow colour with alpha scaled by [`Effects::glow_intensity`].
    pub fn glow_color(&self) -> Color {
        let k = self.glow_intensity.max(0.0);
        self.glow.with_alpha(self.glow.a as f32 / 255.0 * k)
    }

    /// Glow radius in px, scaled by [`Effects::glow_intensity`].
    pub fn glow_radius(&self) -> f32 {
        self.glow_blur * self.glow_intensity.max(0.0)
    }

    /// True when a glow should be drawn at all.
    pub fn glow_enabled(&self) -> bool {
        self.glow_radius() > 0.0 && self.glow_color().a > 0
    }
}

#[cfg(test)]
mod glow_tests {
    use super::*;

    fn fx(i: f32) -> Effects {
        Effects {
            glow: Color::hex(0xfcee0a).with_alpha(0.5),
            glow_blur: 10.0,
            glow_intensity: i,
            scanlines: false,
            hard_shadow: 0.0,
        }
    }

    #[test]
    fn intensity_scales_alpha_and_radius() {
        assert_eq!(fx(1.0).glow_radius(), 10.0);
        assert_eq!(fx(2.0).glow_radius(), 20.0);
        assert!(fx(2.0).glow_color().a > fx(1.0).glow_color().a);
        assert!(fx(0.5).glow_color().a < fx(1.0).glow_color().a);
    }

    #[test]
    fn zero_disables() {
        assert!(!fx(0.0).glow_enabled());
        assert!(fx(1.0).glow_enabled());
    }
}
