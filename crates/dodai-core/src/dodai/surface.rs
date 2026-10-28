//! Dodai surface styling: frosted glass lit from within. Textures are barely there (fine
//! brushing, dots, a grid); the light is the dodai gradient wash and a hairline of light
//! falling in from the borders. Nothing
//! neon, nothing loud.

use crate::{EffectSet, SurfaceFx, SurfaceStyle, SurfaceTuning, Texture};

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
    rim: true,
    strip: true,
    edge: true,
    ..EffectSet::NONE
};

/// The Dodai surface style.
/// `hue` rotates the gradient wash (degrees; 0 is the signature violet).
pub fn style(dark: bool, hue: f32) -> SurfaceStyle {
    SurfaceStyle {
        preset: SurfaceFx {
            texture: Texture::None,
            strength: 0.0,
            container_texture: Texture::None,
            container_strength: 0.0,
            gloss: 0.0,
            wash: 0.8,
            rim: 0.6,
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
        gradient: 1.0,
        wash: {
            let g = if dark {
                crate::dodai::tokens::DODAI_DARK
            } else {
                crate::dodai::tokens::DODAI_LIGHT
            };
            g.stops
                .map(|s| crate::dodai::rotate_hue(s.color.color(), hue))
        },
    }
}
