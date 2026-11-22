//! The frame of a panel: contour width and strength, and the corner its accent (the hard offset
//! shadow) sits on. Pure values; [`Theme::panel_frame`] resolves them with the shape tune.

use crate::{Color, Theme};

/// Border of a section/panel: thickness and strength. Reusable value type;
/// every panel of a theme takes [`crate::Theme::panel_border`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderStyle {
    /// Line thickness in px (`0` = no border).
    pub width: f32,
    /// Colour strength `0..=1` (multiplies the theme border's alpha).
    pub opacity: f32,
    /// Corner where the accent (theme primary, the hard offset shadow) sits;
    /// the grey contour sits on the opposite edges. Same design, rotated.
    pub accent: AccentCorner,
    /// Thickness of the accent in px (`None` = the theme's own hard shadow).
    pub accent_width: Option<f32>,
}

/// A corner of a box (where the accent bracket sits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccentCorner {
    /// Top left.
    TopLeft,
    /// Top right.
    TopRight,
    /// Bottom left.
    BottomLeft,
    /// Bottom right.
    BottomRight,
}

impl AccentCorner {
    /// The four corners, clockwise from the top left.
    pub const ALL: [AccentCorner; 4] = [
        AccentCorner::TopLeft,
        AccentCorner::TopRight,
        AccentCorner::BottomLeft,
        AccentCorner::BottomRight,
    ];
    /// Unit direction `(dx, dy)` of this corner from the centre.
    pub fn dir(self) -> (f32, f32) {
        match self {
            AccentCorner::TopLeft => (-1.0, -1.0),
            AccentCorner::TopRight => (1.0, -1.0),
            AccentCorner::BottomLeft => (-1.0, 1.0),
            AccentCorner::BottomRight => (1.0, 1.0),
        }
    }
    /// The diagonally opposite corner (where the grey contour goes).
    pub fn opposite(self) -> AccentCorner {
        match self {
            AccentCorner::TopLeft => AccentCorner::BottomRight,
            AccentCorner::BottomRight => AccentCorner::TopLeft,
            AccentCorner::TopRight => AccentCorner::BottomLeft,
            AccentCorner::BottomLeft => AccentCorner::TopRight,
        }
    }
    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            AccentCorner::TopLeft => "Top left",
            AccentCorner::TopRight => "Top right",
            AccentCorner::BottomLeft => "Bottom left",
            AccentCorner::BottomRight => "Bottom right",
        }
    }
}

impl BorderStyle {
    /// Barely-there hairline.
    pub const SUBTLE: BorderStyle = BorderStyle {
        width: 0.75,
        opacity: 0.45,
        accent: AccentCorner::BottomRight,
        accent_width: None,
    };
    /// Default for sections: thin and soft.
    pub const THIN: BorderStyle = BorderStyle {
        width: 1.0,
        opacity: 0.6,
        accent: AccentCorner::BottomRight,
        accent_width: None,
    };
    /// The theme's own border, unchanged strength.
    pub const NORMAL: BorderStyle = BorderStyle {
        width: 1.5,
        opacity: 1.0,
        accent: AccentCorner::BottomRight,
        accent_width: None,
    };
    /// Heavy frame.
    pub const BOLD: BorderStyle = BorderStyle {
        width: 3.0,
        opacity: 1.0,
        accent: AccentCorner::BottomRight,
        accent_width: None,
    };
    /// No section border.
    pub const NONE: BorderStyle = BorderStyle {
        width: 0.0,
        opacity: 0.0,
        accent: AccentCorner::BottomRight,
        accent_width: None,
    };

    /// Create a new [`BorderStyle`].
    pub fn new(width: f32, opacity: f32) -> Self {
        Self {
            width,
            opacity,
            accent: AccentCorner::BottomRight,
            accent_width: None,
        }
        .clamped()
    }
    /// Same border with the accent moved to `corner` (grey goes opposite).
    pub fn accent(mut self, corner: AccentCorner) -> Self {
        self.accent = corner;
        self
    }
    /// Same border with an accent of `width` px.
    pub fn accent_width(mut self, width: f32) -> Self {
        self.accent_width = Some(width);
        self.clamped()
    }
    /// Within range: width 0..=8, opacity 0..=1, accent width 0.5..=8.
    pub fn clamped(self) -> Self {
        Self {
            width: self.width.clamp(0.0, 8.0),
            opacity: self.opacity.clamp(0.0, 1.0),
            accent: self.accent,
            accent_width: self.accent_width.map(|w| w.clamp(0.5, 8.0)),
        }
    }
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self::THIN
    }
}

/// A box outline: colour and width in px.
pub type Outline = (Color, f32);

/// A hard offset shadow: colour and `(dx, dy)` in px.
pub type HardShadow = (Color, (f32, f32));

/// A panel resolved against a theme: what every toolkit paints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelFrame {
    /// Fill.
    pub fill: Color,
    /// Contour (`None` = none).
    pub outline: Option<Outline>,
    /// Hard offset shadow: colour and `(dx, dy)` in px (`None` = none).
    pub shadow: Option<HardShadow>,
}

impl Theme {
    /// The outline a box with `border` (and a fill when `filled`) draws under the shape tune:
    /// the tune's tint and width, and an outline for every filled box with "outline all".
    pub fn outline(&self, border: Option<Color>, filled: bool) -> Option<Outline> {
        use crate::BorderTint as T;
        let p = &self.palette;
        let tint = match self.tune.tint {
            T::Design => None,
            T::Strong => Some(p.border_strong),
            T::Accent => Some(p.primary),
            T::Text => Some(p.text),
        };
        let all = self.tune.outline_all;
        let c = match border {
            Some(c) => tint.unwrap_or(c),
            None if all && filled => tint.unwrap_or(p.border),
            None => return None,
        };
        let w = if all && self.tune.border_width.is_none() {
            self.shape.border_width.max(1.0)
        } else {
            self.shape.border_width
        };
        Some((c, w))
    }

    /// The hard offset shadow `(dx, dy)` in px, in the tune's direction (`None` on designs
    /// without one).
    pub fn hard_offset(&self) -> Option<(f32, f32)> {
        let s = self.effects.hard_shadow;
        (s > 0.0).then(|| {
            let (dx, dy) = self.tune.shadow_dir.dir();
            (s * dx, s * dy)
        })
    }

    /// A panel (`focused` highlights it) framed by [`Theme::panel_border`]: the contour softened
    /// and resized, the hard shadow moved to the accent corner.
    pub fn panel_frame(&self, focused: bool) -> PanelFrame {
        self.framed(self.panel(focused).border, self.panel_border)
            .with_fill(self.panel(focused).background)
    }

    /// A box with `border` framed by `b` (see [`Theme::panel_frame`]).
    pub fn framed(&self, border: Color, b: BorderStyle) -> PanelFrame {
        let b = b.clamped();
        let outline = self
            .outline(Some(border), true)
            .filter(|_| b.width > 0.0 && b.opacity > 0.0)
            .map(|(c, _)| (c.with_alpha(c.a as f32 / 255.0 * b.opacity), b.width));
        let shadow = (self.effects.hard_shadow > 0.0).then(|| {
            let off = b.accent_width.unwrap_or(self.effects.hard_shadow);
            let (dx, dy) = b.accent.dir();
            (self.hard_shadow_color(), (off * dx, off * dy))
        });
        PanelFrame {
            fill: self.palette.surface,
            outline,
            shadow,
        }
    }
}

impl PanelFrame {
    /// The same frame filled with `fill`.
    pub fn with_fill(self, fill: Color) -> Self {
        Self { fill, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::themes;

    #[test]
    fn corners_are_opposite_pairs() {
        for c in AccentCorner::ALL {
            assert_ne!(c, c.opposite());
            assert_eq!(c, c.opposite().opposite());
        }
    }

    #[test]
    fn frame_softens_resizes_and_moves_the_accent() {
        let t = themes::cyberpunk::yellow();
        let base = t.panel(false).border;
        let f = t.framed(base, BorderStyle::new(0.5, 0.5));
        let (c, w) = f.outline.unwrap();
        assert_eq!(w, 0.5);
        assert!((c.a as f32 - base.a as f32 * 0.5).abs() <= 1.0);
        assert!(t.framed(base, BorderStyle::NONE).outline.is_none());
        let a = BorderStyle::THIN
            .accent(AccentCorner::TopLeft)
            .accent_width(99.0);
        assert_eq!(t.framed(base, a).shadow.unwrap().1, (-8.0, -8.0));
        assert_eq!(
            t.framed(base, BorderStyle::THIN).shadow.unwrap().1,
            (4.0, 4.0)
        );
    }

    #[test]
    fn tune_tints_and_points_the_shadow() {
        let t = themes::cyberpunk::yellow().tuned(&crate::ShapeTune {
            tint: crate::BorderTint::Text,
            shadow_dir: crate::ShadowDir::TopLeft,
            ..crate::ShapeTune::DEFAULT
        });
        assert_eq!(
            t.outline(Some(t.palette.border), false).unwrap().0,
            t.palette.text
        );
        assert_eq!(t.hard_offset(), Some((-4.0, -4.0)));
        assert!(t.outline(None, true).is_none());
    }
}
