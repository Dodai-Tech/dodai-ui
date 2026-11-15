//! Neo-brutalism surface styling: loud, flat and printed.
//!
//! Black-ink halftone, hatching, checks and zigzags; a hard offset block that makes a surface
//! pop off the page; strips of masking tape stuck across corners; registration brackets and a
//! marquee light. No gradients, glow or soft light: everything stays flat and graphic.

use crate::{EffectSet, ResolvedSurface, SurfaceStyle, SurfaceTuning, Texture};

/// Textures this design owns.
pub const TEXTURES: &[Texture] = &[
    Texture::None,
    Texture::Halftone,
    Texture::Hatch,
    Texture::Crosshatch,
    Texture::Stripes,
    Texture::Checker,
    Texture::Dither,
    Texture::Hazard,
    Texture::Zigzag,
];

/// The effects this design owns: graphic ones only.
pub const EFFECTS: EffectSet = EffectSet {
    pop: true,
    tape: true,
    brackets: true,
    chase: true,
    ..EffectSet::NONE
};

/// The neo-brutalist inspired surface style.
pub fn style(dark: bool) -> SurfaceStyle {
    let calm = if dark { 0.8 } else { 1.0 };
    SurfaceStyle {
        preset: ResolvedSurface {
            texture: Texture::Halftone,
            strength: 0.35 * calm,
            container_texture: Texture::Zigzag,
            container_strength: 0.2 * calm,
            container_scale: 1.5,
            pop: 0.6,
            tape: 0.0,
            brackets: 0.0,
            clear: 0.4,
            ..ResolvedSurface::NONE
        },
        textures: TEXTURES,
        effects: EFFECTS,
        tuning: SurfaceTuning {
            halo: 0.0,
            chase_period: 3.0,
            ..SurfaceTuning::DEFAULT
        },
        ..SurfaceStyle::NONE
    }
}
