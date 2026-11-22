//! How a design draws data marks: line weight, mark outlines, gridlines and area fill.
//!
//! Plain data, read by every toolkit's charts, so one design looks the same in gpui, iced and
//! any later toolkit. A design overrides [`crate::DesignSpec::chart_style`]; the default is
//! derived from the theme's palette.

use crate::{Color, Theme};

/// How a design draws dials and level bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialLook {
    /// Zone band, tick marks and a needle: a classic analogue instrument.
    #[default]
    Analog,
    /// A single arc filled up to the value, no needle.
    Arc,
    /// A ring of lit segments, like an LED bar.
    Segmented,
}

/// How a design draws scopes, spectra and radar: line weight, grid, fills and cell shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScopeLook {
    /// Thin glowing traces, a full graticule and a fading trail.
    #[default]
    Neon,
    /// Soft traces with a translucent fill underneath and a faint graticule.
    Frosted,
    /// Plain medium traces with a light fill and solid bars.
    Clean,
    /// Heavy ink traces, an opaque minimal grid and solid blocks.
    Ink,
}

/// How instrument needles, tapes and horizons follow a changing value: the feel of the movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NeedleResponse {
    /// Jumps straight to the value.
    Instant,
    /// Quick and exact, no overshoot.
    #[default]
    Snappy,
    /// Gentle and unhurried.
    Smooth,
    /// A slow, damped needle with inertia.
    Heavy,
    /// Quick with a visible overshoot, like a spring.
    Bouncy,
}

impl NeedleResponse {
    /// Every response, in menu order.
    pub const ALL: [NeedleResponse; 5] = [
        Self::Instant,
        Self::Snappy,
        Self::Smooth,
        Self::Heavy,
        Self::Bouncy,
    ];
    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Instant => "Instant",
            Self::Snappy => "Snappy",
            Self::Smooth => "Smooth",
            Self::Heavy => "Heavy",
            Self::Bouncy => "Bouncy",
        }
    }
    /// Spring parameters `(response seconds, bounce)`, or `None` to jump. Pure.
    pub fn spring(self) -> Option<(f32, f32)> {
        match self {
            Self::Instant => None,
            Self::Snappy => Some((0.22, 0.0)),
            Self::Smooth => Some((0.6, 0.0)),
            Self::Heavy => Some((1.1, -0.2)),
            Self::Bouncy => Some((0.45, 0.55)),
        }
    }
}

/// Chart drawing tokens of one theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChartStyle {
    /// Stroke width of series lines, px.
    pub line_width: f32,
    /// Outline colour of bars, slices and bubbles (`None`: no outline).
    pub mark_outline: Option<Color>,
    /// Outline width, px (ignored without an outline).
    pub mark_outline_width: f32,
    /// Gridline colour.
    pub grid_color: Color,
    /// Gridline width, px.
    pub grid_width: f32,
    /// Dashed gridlines.
    pub grid_dashed: bool,
    /// Opacity of the fill under area series, `0..=1`.
    pub area_alpha: f32,
    /// The default look of dial instruments.
    pub dial: DialLook,
    /// The look of scopes, spectra and radar.
    pub scope: ScopeLook,
    /// The look of vehicle instruments given none of their own.
    pub vehicle: crate::vehicle_look::Look,
    /// How instruments follow a changing value.
    pub response: NeedleResponse,
}

impl ChartStyle {
    /// The neutral style derived from a theme.
    pub fn from_theme(t: &Theme) -> Self {
        Self {
            line_width: 2.0,
            mark_outline: None,
            mark_outline_width: 0.0,
            grid_color: t.palette.border,
            grid_width: 1.0,
            grid_dashed: false,
            area_alpha: 0.18,
            dial: DialLook::Analog,
            scope: ScopeLook::Neon,
            vehicle: crate::vehicle_look::Look::Sport,
            response: NeedleResponse::Snappy,
        }
    }
}

impl Theme {
    /// The chart drawing tokens: the design's own, else derived from the palette.
    pub fn chart_style(&self) -> ChartStyle {
        self.spec
            .get()
            .and_then(|s| s.chart_style(self))
            .unwrap_or_else(|| ChartStyle::from_theme(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_style_follows_the_palette() {
        let t = Theme::default();
        let s = t.chart_style();
        assert_eq!(s.grid_color, t.palette.border);
        assert!(s.mark_outline.is_none());
        assert!((0.0..=1.0).contains(&s.area_alpha));
    }
}
