//! A design's own surface styling, carried by the theme.

use dodai_surface::{EffectSet, SurfaceFx, SurfaceTuning, Texture};

/// A design's own surface styling: the mix it suggests and which textures and animated
/// effects it owns. Part of the theme, so it travels with it like colours and shapes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceStyle {
    /// The mix the "design preset" action applies (nothing is drawn until an app opts in).
    pub preset: SurfaceFx,
    /// Textures this design owns; others are never drawn for it.
    pub textures: &'static [Texture],
    /// Which of the animated and edge effects belong to this design.
    pub effects: EffectSet,
    /// Timing and sizes of the drawing.
    pub tuning: SurfaceTuning,
    /// The three colours of the design's gradient (the `wash` effect), start to end.
    pub wash: [crate::Color; 3],
    /// How strongly the same three colours light the canvas as a soft mesh behind everything
    /// (0 = a flat canvas). Panels that are translucent let it show through.
    pub backdrop: f32,
}

impl SurfaceStyle {
    /// A design with no textures or effects of its own.
    pub const NONE: Self = Self {
        preset: SurfaceFx::NONE,
        textures: &[],
        effects: EffectSet::NONE,
        tuning: SurfaceTuning::DEFAULT,
        wash: [crate::Color::rgba(0, 0, 0, 0); 3],
        backdrop: 0.0,
    };
}

impl Default for SurfaceStyle {
    fn default() -> Self {
        Self::NONE
    }
}
