//! Material 3 surface styling: tonal, soft and quiet.
//!
//! Material has no neon and no hard graphics. Its surfaces are paper and dots, a tonal wash of
//! the primary colour (the elevation tint), a click ripple and a gentle inner
//! shadow. Nothing here is shared with the other designs.

use crate::{EffectSet, PressStyle, ResolvedSurface, SurfaceStyle, SurfaceTuning, Texture};

/// Textures this design owns.
pub const TEXTURES: &[Texture] = &[
    Texture::None,
    Texture::Paper,
    Texture::DotGrid,
    Texture::Ripples,
];

/// The effects this design owns.
pub const EFFECTS: EffectSet = EffectSet {
    tint: true,
    ripple: true,
    depth: true,
    ..EffectSet::NONE
};

/// The Material 3 inspired surface style.
pub fn style(dark: bool) -> SurfaceStyle {
    // Ink shows more on dark tonal surfaces, so it is a touch quieter there.
    let calm = if dark { 0.8 } else { 1.0 };
    SurfaceStyle {
        preset: ResolvedSurface {
            texture: Texture::Paper,
            strength: 0.4 * calm,
            container_texture: Texture::Ripples,
            container_strength: 0.3 * calm,
            tint: 0.35,
            ripple: 0.7,
            press: PressStyle::Ink,
            depth: 0.15,
            clear: 0.4,
            ..ResolvedSurface::NONE
        },
        textures: TEXTURES,
        effects: EFFECTS,
        tuning: SurfaceTuning {
            pulse_period: 2.8,
            ..SurfaceTuning::DEFAULT
        },
        ..SurfaceStyle::NONE
    }
}
