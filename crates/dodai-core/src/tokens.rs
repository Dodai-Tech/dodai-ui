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

impl Spacing {
    /// Half of [`xs`](Self::xs): the smallest gap between touching parts.
    pub fn half(&self) -> f32 {
        self.xs * 0.5
    }
}

/// Opacity of the overlay a control shows for its interaction state (Material state layers).
/// One definition, so hover, focus, press and selection read the same everywhere.
#[derive(Debug, Clone, Copy)]
pub struct StateLayer;

impl StateLayer {
    /// Pointer over the control.
    pub const HOVER: f32 = 0.08;
    /// Keyboard focus.
    pub const FOCUS: f32 = 0.10;
    /// Pressed.
    pub const PRESSED: f32 = 0.10;
    /// Selected or active row.
    pub const SELECTED: f32 = 0.12;
}

/// Named multiples of the control height. Components size themselves from these instead of
/// carrying their own ratios, so density follows the theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    /// Half a control (slider value bubble).
    Half,
    /// Dense data rows.
    Dense,
    /// Compact rows and small cells (colour cells, list rows in menus).
    Compact,
    /// Table and list headers.
    Header,
    /// A result or option row.
    Row,
    /// A square input cell (one-time-code digit).
    Cell,
    /// A tall field (one-time-code, text area line block).
    Field,
    /// A tall preview or swatch.
    Tall,
    /// A wide bar or preview.
    Wide,
    /// Offset of a top-anchored overlay from the window edge.
    Offset,
    /// Minimum width of a small panel.
    Panel,
    /// Width of a popup picker.
    Popup,
    /// Tallest a dropdown list grows before it scrolls.
    ListMax,
}

impl Scale {
    /// Every step, smallest first.
    pub const ALL: [Self; 13] = [
        Self::Half,
        Self::Dense,
        Self::Compact,
        Self::Header,
        Self::Row,
        Self::Cell,
        Self::Field,
        Self::Tall,
        Self::Wide,
        Self::Offset,
        Self::Panel,
        Self::Popup,
        Self::ListMax,
    ];

    /// The multiple of the control height.
    pub fn ratio(self) -> f32 {
        match self {
            Self::Half => 0.5,
            Self::Dense => 0.7,
            Self::Compact => 0.8,
            Self::Header => 0.85,
            Self::Row => 0.9,
            Self::Cell => 1.05,
            Self::Field => 1.3,
            Self::Tall => 1.4,
            Self::Wide => 1.6,
            Self::Offset => 2.0,
            Self::Panel => 4.5,
            Self::Popup => 5.5,
            Self::ListMax => 7.0,
        }
    }
}

/// Fixed dimensions shared by components, in logical pixels (the "size scale" next to [`Spacing`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Sizes {
    /// Small icon (inline with text, compact controls).
    pub icon_sm: f32,
    /// Standard icon.
    pub icon: f32,
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

pub use dodai_surface::{Clip, Corners, ShapeKind};

/// Shape tokens.
#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    /// Which corners are chamfered when `chamfer > 0`.
    pub corners: Corners,
    /// Plain corner radius (0 for hard-edged themes).
    pub radius: f32,
    /// Cut-corner size for chamfered panels/buttons (0 = none).
    pub chamfer: f32,
    /// Draw the chamfered corners as concave quarter-circle arcs (a scoop) instead of straight cuts.
    pub scoop: bool,
    pub border_width: f32,
    pub control_height: f32,
}
impl Shape {
    /// The outline effects are clipped to.
    pub fn clip(&self) -> Clip {
        Clip {
            corners: self.corners,
            radius: self.radius,
            chamfer: self.chamfer,
            scoop: self.scoop,
            border_width: self.border_width,
        }
    }

    /// Outline style implied by the tokens.
    pub fn kind(&self) -> ShapeKind {
        self.clip().kind()
    }

    /// Outline polygon (clockwise from top-left) for a `w`×`h` box with this shape's chamfer.
    /// Shared by every adapter so all frameworks cut identical corners.
    pub fn outline(&self, w: f32, h: f32) -> Vec<(f32, f32)> {
        self.clip().outline(w, h)
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
    pub const SCANLINE_PITCH: f32 = dodai_surface::SCANLINE_PITCH;
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

#[cfg(test)]
mod scale_tests {
    use super::*;

    #[test]
    fn scale_steps_grow_and_are_positive() {
        assert!(Scale::ALL.windows(2).all(|w| w[0].ratio() < w[1].ratio()));
        assert!(Scale::ALL[0].ratio() > 0.0);
    }

    #[test]
    fn half_is_half_of_xs() {
        let s = Spacing {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
        };
        assert_eq!(s.half(), 2.0);
    }
}
