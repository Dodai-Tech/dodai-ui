//! Aurora surface styling: frosted glass lit from within. Textures are barely there (fine
//! brushing, dots, a grid); the light is a clear-coat gloss across the top, a thin
//! strip of light along the edge and a hairline of light falling in from the borders. Nothing
//! neon, nothing loud.

use dodai_core::{EffectSet, SurfaceFx, SurfaceStyle, SurfaceTuning, Texture};

/// Textures this design owns: glass is mostly clean.
pub const TEXTURES: &[Texture] = &[
    Texture::None,
    Texture::Brushed,
    Texture::Perforation,
    Texture::Grid,
];

/// The effects this design owns.
pub const EFFECTS: EffectSet = EffectSet {
    gloss: true,
    strip: true,
    edge: true,
    ..EffectSet::NONE
};

/// The Aurora surface style.
pub fn style(dark: bool) -> SurfaceStyle {
    let calm = if dark { 0.9 } else { 1.0 };
    SurfaceStyle {
        preset: SurfaceFx {
            texture: Texture::None,
            strength: 0.0,
            container_texture: Texture::None,
            container_strength: 0.0,
            gloss: 0.3 * calm,
            edge: 0.2,
            clear: 0.45,
            ..SurfaceFx::NONE
        },
        textures: TEXTURES,
        effects: EFFECTS,
        tuning: SurfaceTuning {
            halo: 1.0,
            sheen_period: 4.5,
            ..SurfaceTuning::DEFAULT
        },
    }
}
