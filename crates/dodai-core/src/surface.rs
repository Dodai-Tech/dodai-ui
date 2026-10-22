//! Surface textures and effects: patterns and animated accents drawn over a control's fill,
//! clipped to its shape.
//!
//! These are runtime settings (the showcase and apps switch them), while a design suggests
//! its own mix through [`crate::DesignSpec::surface_preset`].

/// A static pattern over a surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Texture {
    /// No texture.
    #[default]
    None,
    /// Film or paper grain: scattered specks.
    Grain,
    /// A dot grid whose dots grow towards the bottom.
    Halftone,
    /// A 2×2 ordered-dither checker.
    Dither,
    /// Diagonal hatching.
    Hatch,
    /// Blueprint grid with a heavier line every fifth cell.
    Grid,
    /// Long horizontal micro-lines, like brushed metal.
    Brushed,
    /// A small diagonal checker, like carbon-fibre weave.
    Carbon,
    /// A regular pattern of round holes.
    Perforation,
    /// Fine horizontal scanlines.
    Scanlines,
}

impl Texture {
    /// Every texture, `None` first.
    pub const ALL: [Texture; 10] = [
        Texture::None,
        Texture::Grain,
        Texture::Halftone,
        Texture::Dither,
        Texture::Hatch,
        Texture::Grid,
        Texture::Brushed,
        Texture::Carbon,
        Texture::Perforation,
        Texture::Scanlines,
    ];

    /// A short label for pickers.
    pub fn label(self) -> &'static str {
        match self {
            Texture::None => "None",
            Texture::Grain => "Grain",
            Texture::Halftone => "Halftone",
            Texture::Dither => "Dither",
            Texture::Hatch => "Hatch",
            Texture::Grid => "Grid",
            Texture::Brushed => "Brushed",
            Texture::Carbon => "Carbon",
            Texture::Perforation => "Perforation",
            Texture::Scanlines => "Scanlines",
        }
    }
}

/// The surface mix: one texture plus animated effects. Every amount is `0.0..=1.0`
/// (`0.0` = off), so a design preset and a slider speak the same language.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceFx {
    /// The static pattern.
    pub texture: Texture,
    /// How visible the texture is.
    pub strength: f32,
    /// Pattern size multiplier (`1.0` = as designed).
    pub scale: f32,
    /// Glitch bursts: RGB split and displaced slices.
    pub glitch: f32,
    /// A light band that sweeps across the surface.
    pub sheen: f32,
    /// Brief dimming, like a failing screen.
    pub flicker: f32,
    /// Soft bloom around the outline.
    pub bloom: f32,
}

impl Default for SurfaceFx {
    fn default() -> Self {
        Self::NONE
    }
}

impl SurfaceFx {
    /// Nothing: no texture, no effects.
    pub const NONE: SurfaceFx = SurfaceFx {
        texture: Texture::None,
        strength: 0.5,
        scale: 1.0,
        glitch: 0.0,
        sheen: 0.0,
        flicker: 0.0,
        bloom: 0.0,
    };

    /// True when a texture is set and visible.
    pub fn has_texture(&self) -> bool {
        self.texture != Texture::None && self.strength > 0.0
    }

    /// True when any animated effect is on (the surface needs frames).
    pub fn animated(&self) -> bool {
        self.glitch > 0.0 || self.sheen > 0.0 || self.flicker > 0.0
    }

    /// True when nothing would be drawn.
    pub fn is_none(&self) -> bool {
        !self.has_texture() && !self.animated() && self.bloom <= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_draws_nothing() {
        let d = SurfaceFx::default();
        assert!(d.is_none() && !d.animated() && !d.has_texture());
        assert!(!SurfaceFx { glitch: 0.4, ..d }.is_none());
        assert!(SurfaceFx { glitch: 0.4, ..d }.animated());
        assert!(SurfaceFx {
            texture: Texture::Grain,
            ..d
        }
        .has_texture());
    }

    #[test]
    fn every_texture_is_listed_once_with_a_label() {
        let mut labels: Vec<_> = Texture::ALL.iter().map(|t| t.label()).collect();
        labels.sort();
        labels.dedup();
        assert_eq!(labels.len(), Texture::ALL.len());
    }
}
