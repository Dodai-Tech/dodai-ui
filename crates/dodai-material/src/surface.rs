//! Material 3 surface styling: soft, tonal textures and quiet light, nothing neon.
//!
//! Inspired by the Material 3 look: paper and dot textures that stay behind the content, a
//! slow sheen like a highlight moving across a lacquered surface, breathing borders and light
//! that falls in from the edges. There is no glitch, flicker or bracket.

use dodai_core::{EffectSet, SurfaceFx, SurfaceStyle, SurfaceTuning, Texture};

/// Textures this design owns.
pub const TEXTURES: &[Texture] = &[
    Texture::None,
    Texture::Grain,
    Texture::Paper,
    Texture::Dither,
    Texture::Halftone,
    Texture::DotGrid,
    Texture::Perforation,
    Texture::Stripes,
    Texture::Grid,
    Texture::Hatch,
];

/// The effects this design owns: gentle ones only.
pub const EFFECTS: EffectSet = EffectSet {
    sheen: true,
    pulse: true,
    edge: true,
    depth: true,
    bloom: true,
    ..EffectSet::NONE
};

/// The Material 3 inspired surface style.
pub fn style(dark: bool) -> SurfaceStyle {
    // Ink shows more on dark tonal surfaces, so it is a touch quieter there.
    let calm = if dark { 0.8 } else { 1.0 };
    SurfaceStyle {
        preset: SurfaceFx {
            texture: Texture::Paper,
            strength: 0.4 * calm,
            container_texture: Texture::DotGrid,
            container_strength: 0.25 * calm,
            sheen: 0.2,
            edge: 0.18,
            depth: 0.2,
            clear: 0.4,
            ..SurfaceFx::NONE
        },
        textures: TEXTURES,
        effects: EFFECTS,
        tuning: SurfaceTuning {
            halo: 1.0,
            bloom_spread: 4.0,
            sheen_period: 5.0,
            pulse_period: 3.4,
            ..SurfaceTuning::DEFAULT
        },
    }
}
