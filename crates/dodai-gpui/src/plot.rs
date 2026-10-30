//! Drawing pieces shared by the chart and instrument crates: the app-wide chart glow and
//! glowing strokes.

use crate::paint::{glow_layers, scale_alpha, stroke};
use dodai_core::Theme;
use gpui::{Hsla, Pixels, Window};

/// Glow `(radius)` when the theme glows. Pure.
pub fn glow_of(t: &Theme) -> Option<f32> {
    t.effects.glow_enabled().then(|| t.effects.glow_radius())
}

/// How much glow (the soft "shadow" under lines and lit cells) a chart draws.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ChartGlow {
    /// Follow the theme: its glow when the design has one, none otherwise.
    #[default]
    Theme,
    /// No glow, whatever the theme.
    Off,
    /// A glow of this radius (px), even on a theme without one.
    Radius(f32),
}

thread_local! {
    static GLOW: std::cell::Cell<ChartGlow> = const { std::cell::Cell::new(ChartGlow::Theme) };
}

/// The glow every chart and instrument uses unless it sets its own. Starts as
/// [`ChartGlow::Theme`].
pub fn chart_glow() -> ChartGlow {
    GLOW.with(std::cell::Cell::get)
}

/// Sets the glow for all charts and instruments at once, and repaints.
pub fn set_chart_glow(cx: &mut gpui::App, glow: ChartGlow) {
    GLOW.with(|c| c.set(glow));
    cx.refresh_windows();
}

impl ChartGlow {
    /// The radius to draw, or `None` for no glow.
    pub fn resolve(self, t: &Theme) -> Option<f32> {
        match self {
            Self::Theme => match chart_glow() {
                Self::Theme => glow_of(t),
                other => other.resolve(t),
            },
            Self::Off => None,
            Self::Radius(r) => (r > 0.0).then_some(r),
        }
    }
}

/// Stroke with glow layers underneath.
pub fn glow_stroke(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    p: &[(f32, f32)],
    width: f32,
    c: Hsla,
    glow: Option<f32>,
) {
    if let Some(r) = glow {
        for l in glow_layers(r, 0.0) {
            stroke(w, o, p, width + l.width, scale_alpha(c, l.alpha), false);
        }
    }
    stroke(w, o, p, width, c, false);
}

/// The four corners of an axis-aligned rectangle, clockwise from the top left.
pub fn rect(x: f32, y: f32, w: f32, h: f32) -> [(f32, f32); 4] {
    [(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
}
