//! Neo-brutalism surface styling: bold, flat patterns and marquee lights.
//!
//! Inspired by the neo-brutalist look: halftone, hatching and checks in black ink, corner
//! registration marks, a light marching around a border and a hard pulse. No glow, bloom or
//! flicker: everything stays flat and graphic.

use dodai_core::{EffectSet, SurfaceFx, SurfaceStyle, SurfaceTuning, Texture};

/// Textures this design owns.
pub const TEXTURES: &[Texture] = &[
    Texture::None,
    Texture::Halftone,
    Texture::Hatch,
    Texture::Crosshatch,
    Texture::Stripes,
    Texture::Checker,
    Texture::Dither,
    Texture::DotGrid,
    Texture::Grid,
    Texture::Hazard,
    Texture::Perforation,
    Texture::Grain,
    Texture::Paper,
    Texture::Barcode,
];

/// The effects this design owns: graphic ones only.
pub const EFFECTS: EffectSet = EffectSet {
    brackets: true,
    chase: true,
    pulse: true,
    depth: true,
    ..EffectSet::NONE
};

/// The neo-brutalist inspired surface style.
pub fn style(dark: bool) -> SurfaceStyle {
    let calm = if dark { 0.8 } else { 1.0 };
    SurfaceStyle {
        preset: SurfaceFx {
            texture: Texture::Halftone,
            strength: 0.35 * calm,
            container_texture: Texture::Hatch,
            container_strength: 0.2 * calm,
            container_scale: 1.5,
            brackets: 0.5,
            clear: 0.4,
            ..SurfaceFx::NONE
        },
        textures: TEXTURES,
        effects: EFFECTS,
        tuning: SurfaceTuning {
            halo: 0.0,
            pulse_period: 1.8,
            chase_period: 3.0,
            ..SurfaceTuning::DEFAULT
        },
    }
}
