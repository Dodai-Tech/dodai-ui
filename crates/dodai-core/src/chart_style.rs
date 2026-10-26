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

/// A whole instrument cluster style. Every instrument of one cluster follows the same style, so a
/// set never mixes looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClusterKind {
    /// Round pods around a big inverted tachometer.
    #[default]
    Pods,
    /// A cut-corner speedometer with radiating hairlines.
    Cut,
    /// A capsule with a gradient sweep, side arcs and a gear selector.
    Capsule,
    /// A clock-style seven-segment panel with LED bars.
    Segment,
    /// Five equal round pods.
    Round,
    /// Four inverted dials: bold ring, large numerals.
    Bold,
    /// Two large analog dials flanked by two small ones.
    Classic,
    /// The same layout with arc dials.
    Arc,
    /// The same layout with segmented LED dials.
    Segmented,
    /// Numeric tiles with segment bars.
    Digital,
}

impl ClusterKind {
    /// Every style, in showcase order.
    pub const ALL: [ClusterKind; 10] = [
        Self::Cut,
        Self::Segment,
        Self::Capsule,
        Self::Pods,
        Self::Round,
        Self::Bold,
        Self::Classic,
        Self::Arc,
        Self::Segmented,
        Self::Digital,
    ];

    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Pods => "Pods",
            Self::Segment => "Segment",
            Self::Cut => "Cut",
            Self::Capsule => "Capsule",
            Self::Round => "Round",
            Self::Bold => "Bold",
            Self::Classic => "Classic",
            Self::Arc => "Arc",
            Self::Segmented => "Segmented",
            Self::Digital => "Digital",
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
    /// The default instrument cluster.
    pub cluster: ClusterKind,
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
            cluster: ClusterKind::Pods,
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
