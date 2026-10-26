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
