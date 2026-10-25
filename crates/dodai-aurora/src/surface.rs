//! Aurora surface styling: frosted glass lit from within. Textures are barely there (fine
//! brushing, dots, a grid); the light is the aurora gradient wash and a hairline of light
//! falling in from the borders. Nothing
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
    wash: true,
    strip: true,
    edge: true,
    ..EffectSet::NONE
};

/// The Aurora surface style.
pub fn style(dark: bool) -> SurfaceStyle {
    SurfaceStyle {
        preset: SurfaceFx {
            texture: Texture::None,
            strength: 0.0,
            container_texture: Texture::None,
            container_strength: 0.0,
            gloss: 0.0,
            wash: 0.8,
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
        backdrop: if dark { 0.55 } else { 0.38 },
        wash: {
            let g = if dark {
                crate::tokens::AURORA_DARK
            } else {
                crate::tokens::AURORA_LIGHT
            };
            g.stops.map(|s| s.color.color())
        },
    }
}
