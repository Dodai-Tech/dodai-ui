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

/// Fixed dimensions shared by components, in logical pixels (the "size scale" next to [`Spacing`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Sizes {
    /// Small icon (inline with text, compact controls).
    pub icon_sm: f32,
    /// Standard icon.
    pub icon: f32,
    /// Large icon.
    pub icon_lg: f32,
    /// Minimum interactive target.
    pub touch_target: f32,
    /// State-layer diameter of icon-sized controls (switch handle, checkbox).
    pub state_layer: f32,
    /// Switch track `(width, height)`.
    pub switch_track: (f32, f32),
    /// Switch handle diameter `(off, on)`.
    pub switch_handle: (f32, f32),
    /// Checkbox / radio box edge.
    pub selection_box: f32,
    /// Slider handle diameter.
    pub slider_thumb: f32,
    /// Segmented-button height.
    pub segmented_height: f32,
    /// Slider / range track thickness.
    pub slider_track: f32,
    /// Range tick length (across the track).
    pub tick_len: f32,
    /// Range tick thickness.
    pub tick_width: f32,
    /// Chart grid line thickness.
    pub hairline: f32,
    /// Legend swatch and dot-badge edge.
    pub legend_swatch: f32,
    /// Avatar status dot diameter.
    pub status_dot: f32,
    /// Badge minimum width and height.
    pub badge_min: f32,
    /// Key-cap / chip height and minimum width.
    pub chip_height: f32,
    /// Key-cap / badge horizontal padding.
    pub chip_pad: f32,
    /// Chart value-axis gutter width.
    pub axis_gutter: f32,
    /// Wide chart category-axis gutter width.
    pub axis_gutter_lg: f32,
    /// Heatmap row-label column width.
    pub axis_label: f32,
    /// Stacked-bar thickness.
    pub chart_bar: f32,
    /// Grouped-bar / heatmap header thickness.
    pub chart_cell: f32,
    /// Radial chart padding.
    pub chart_pad: f32,
    /// Scatter point diameter.
    pub scatter_dot: f32,
    /// Thickness of scrollbars, resize handles and carousel dots.
    pub bar_thickness: f32,
    /// Bottom-sheet drag handle `(width, height)`.
    pub sheet_handle: (f32, f32),
    /// Width of command palettes and similar centred dialogs.
    pub dialog_width: f32,
}

impl Default for Sizes {
    fn default() -> Self {
        Self {
            icon_sm: 18.0,
            icon: 24.0,
            icon_lg: 32.0,
            touch_target: 48.0,
            state_layer: 40.0,
            switch_track: (52.0, 32.0),
            switch_handle: (16.0, 24.0),
            selection_box: 20.0,
            slider_thumb: 16.0,
            segmented_height: 40.0,
            slider_track: 6.0,
            tick_len: 10.0,
            tick_width: 2.0,
            hairline: 1.0,
            legend_swatch: 10.0,
            status_dot: 11.0,
            badge_min: 18.0,
            chip_height: 22.0,
            chip_pad: 6.0,
            axis_gutter: 36.0,
            axis_gutter_lg: 56.0,
            axis_label: 48.0,
            chart_bar: 18.0,
            chart_cell: 14.0,
            chart_pad: 24.0,
            scatter_dot: 8.0,
            bar_thickness: 6.0,
            sheet_handle: (32.0, 4.0),
            dialog_width: 520.0,
        }
    }
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
    /// Only top-left (UL).
    pub const TOP_LEFT: Self = Self {
        top_left: true,
        ..Self::NONE
    };
    /// Only top-right (UR).
    pub const TOP_RIGHT: Self = Self {
        top_right: true,
        ..Self::NONE
    };
    /// Only bottom-left (LL).
    pub const BOTTOM_LEFT: Self = Self {
        bottom_left: true,
        ..Self::NONE
    };
    /// Both top corners.
    pub const TOP: Self = Self {
        top_left: true,
        top_right: true,
        ..Self::NONE
    };
    /// Both bottom corners.
    pub const BOTTOM: Self = Self {
        bottom_left: true,
        bottom_right: true,
        ..Self::NONE
    };
    /// Both left corners.
    pub const LEFT: Self = Self {
        top_left: true,
        bottom_left: true,
        ..Self::NONE
    };
    /// Both right corners.
    pub const RIGHT: Self = Self {
        top_right: true,
        bottom_right: true,
        ..Self::NONE
    };
    /// Top-right and bottom-left (the other diagonal).
    pub const ANTI_DIAGONAL: Self = Self {
        top_right: true,
        bottom_left: true,
        ..Self::NONE
    };
    /// Only bottom-right (LR).
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

    /// Copy with the corner `top_left` / `top_right` / `bottom_right` /
    /// `bottom_left` (index 0..4, clockwise from top-left) set to `on`.
    pub fn set(mut self, corner: usize, on: bool) -> Self {
        match corner {
            0 => self.top_left = on,
            1 => self.top_right = on,
            2 => self.bottom_right = on,
            _ => self.bottom_left = on,
        }
        self
    }
    /// Whether corner `i` (clockwise from top-left) is cut.
    pub fn get(self, corner: usize) -> bool {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ][corner.min(3)]
    }

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
    /// Hard offset shadow (px) for hard-edged designs, 0 = none.
    pub hard_shadow: f32,
    /// Soft drop-shadow colour (lifts cards on themes without a hard shadow).
    pub shadow: Color,
    /// Colour of the CRT scanline bars.
    pub scanline: Color,
    /// Distance in px between scanline bars.
    pub scanline_pitch: f32,
    /// Light sweep over filled controls (shimmer).
    pub highlight: Color,
}

impl Effects {
    /// Neutral distance between scanline bars: 3 px.
    pub const SCANLINE_PITCH: f32 = 3.0;
    /// Neutral soft shadow: black at 16 %.
    pub const SHADOW: Color = Color::rgba(0, 0, 0, 41);
    /// Neutral scanline bar: black at 18 %.
    pub const SCANLINE: Color = Color::rgba(0, 0, 0, 46);
    /// Neutral highlight: white at 40 %.
    pub const HIGHLIGHT: Color = Color::rgba(255, 255, 255, 102);

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
            shadow: Effects::SHADOW,
            scanline: Effects::SCANLINE,
            scanline_pitch: Effects::SCANLINE_PITCH,
            highlight: Effects::HIGHLIGHT,
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

#[cfg(test)]
mod corner_tests {
    use super::Corners;

    #[test]
    fn corners_set_get() {
        let c = Corners::NONE.set(1, true).set(3, true);
        assert_eq!(c, Corners::ANTI_DIAGONAL);
        assert!(c.get(1) && !c.get(0));
    }
}
