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
    /// Evenly spaced small dots, like a notebook.
    DotGrid,
    /// Fine grid with heavier lines, corner registration crosses.
    Blueprint,
    /// Hatching in both diagonal directions.
    Crosshatch,
    /// Wide diagonal bands.
    Stripes,
    /// A large two-tone checkerboard.
    Checker,
    /// Right-angle traces ending in solder pads.
    Circuit,
    /// Raised diamond tread, like industrial floor metal.
    DiamondPlate,
    /// Soft fibre flecks, like paper stock.
    Paper,
    /// A grid of round lamps, some lit.
    LedMatrix,
    /// A barcode strip along the bottom edge.
    Barcode,
    /// Horizontal bands of broadcast noise.
    Static,
}

impl Texture {
    /// Every texture, `None` first.
    pub const ALL: [Texture; 21] = [
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
        Texture::DotGrid,
        Texture::Blueprint,
        Texture::Crosshatch,
        Texture::Stripes,
        Texture::Checker,
        Texture::Circuit,
        Texture::DiamondPlate,
        Texture::Paper,
        Texture::LedMatrix,
        Texture::Barcode,
        Texture::Static,
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
            Texture::DotGrid => "Dot grid",
            Texture::Blueprint => "Blueprint",
            Texture::Crosshatch => "Crosshatch",
            Texture::Stripes => "Stripes",
            Texture::Checker => "Checker",
            Texture::Circuit => "Circuit",
            Texture::DiamondPlate => "Diamond plate",
            Texture::Paper => "Paper",
            Texture::LedMatrix => "LED matrix",
            Texture::Barcode => "Barcode",
            Texture::Static => "Static",
        }
    }
}

impl Texture {
    /// Whether the pattern is small enough to sit under a button label or an input.
    /// Large layouts (blueprint, circuit, barcode, paper, static) are for containers only.
    pub const fn for_components(self) -> bool {
        !matches!(
            self,
            Texture::Blueprint
                | Texture::Circuit
                | Texture::Barcode
                | Texture::Paper
                | Texture::Static
        )
    }
}

/// A kind of surface a texture or effect can be applied to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceClass {
    /// Buttons of every variant.
    Button,
    /// Cards.
    Card,
    /// Panels and popups.
    Panel,
    /// Text inputs and selects.
    Input,
    /// Tags and badges.
    Tag,
    /// App bars and toolbars.
    Bar,
}

impl SurfaceClass {
    /// Every class, in display order.
    pub const ALL: [SurfaceClass; 6] = [
        Self::Button,
        Self::Card,
        Self::Panel,
        Self::Input,
        Self::Tag,
        Self::Bar,
    ];

    /// Display name.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Button => "Buttons",
            Self::Card => "Cards",
            Self::Panel => "Panels",
            Self::Input => "Inputs",
            Self::Tag => "Tags",
            Self::Bar => "Bars",
        }
    }

    /// Containers hold other content (cards, panels, bars); the rest are UI components.
    pub const fn is_container(self) -> bool {
        matches!(self, Self::Card | Self::Panel | Self::Bar)
    }

    const fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// A set of [`SurfaceClass`]es.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceTargets(u8);

impl SurfaceTargets {
    /// No surface.
    pub const NONE: Self = Self(0);
    /// Buttons, cards and panels: where a texture reads best.
    pub const DEFAULT: Self =
        Self(SurfaceClass::Button.bit() | SurfaceClass::Card.bit() | SurfaceClass::Panel.bit());
    /// Every surface.
    pub const ALL: Self = Self(0x3f);

    /// Whether `c` is in the set.
    pub const fn contains(self, c: SurfaceClass) -> bool {
        self.0 & c.bit() != 0
    }

    /// The set with `c` added or removed.
    pub const fn with(self, c: SurfaceClass, on: bool) -> Self {
        if on {
            Self(self.0 | c.bit())
        } else {
            Self(self.0 & !c.bit())
        }
    }
}

impl Default for SurfaceTargets {
    fn default() -> Self {
        Self::DEFAULT
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
    /// Lit lines: line-based textures glow in the accent colour (`0.0..=1.0`).
    pub neon: f32,
    /// Texture for containers (cards, panels, bars); `texture` is for UI components.
    pub container_texture: Texture,
    /// Strength of the container texture.
    pub container_strength: f32,
    /// Scale of the container texture.
    pub container_scale: f32,
    /// Which kinds of surface receive the mix.
    pub targets: SurfaceTargets,
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
        neon: 0.0,
        container_texture: Texture::None,
        container_strength: 0.5,
        container_scale: 1.0,
        targets: SurfaceTargets::DEFAULT,
    };

    /// The mix restricted to what a design owns: textures outside `textures` become
    /// [`Texture::None`], and the animated effects are zeroed unless `effects`.
    pub fn limited(mut self, textures: &[Texture], effects: bool) -> Self {
        if !textures.contains(&self.texture) {
            self.texture = Texture::None;
        }
        if !textures.contains(&self.container_texture) {
            self.container_texture = Texture::None;
        }
        if !effects {
            self.glitch = 0.0;
            self.sheen = 0.0;
            self.flicker = 0.0;
            self.bloom = 0.0;
        }
        self
    }

    /// The mix as seen by a `class` surface: containers use the container texture.
    pub fn for_class(self, class: SurfaceClass) -> Self {
        if class.is_container() {
            Self {
                texture: self.container_texture,
                strength: self.container_strength,
                scale: self.container_scale,
                ..self
            }
        } else {
            self
        }
    }

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
    #[test]
    fn large_layouts_are_container_only_and_labels_are_unique() {
        use super::*;
        assert!(Texture::Grain.for_components());
        assert!(!Texture::Circuit.for_components());
        let mut l: Vec<_> = Texture::ALL.iter().map(|t| t.label()).collect();
        l.sort();
        l.dedup();
        assert_eq!(l.len(), Texture::ALL.len());
    }

    #[test]
    fn containers_use_their_own_texture() {
        use super::*;
        let fx = SurfaceFx {
            texture: Texture::Grain,
            container_texture: Texture::Grid,
            ..SurfaceFx::NONE
        };
        assert_eq!(fx.for_class(SurfaceClass::Button).texture, Texture::Grain);
        assert_eq!(fx.for_class(SurfaceClass::Card).texture, Texture::Grid);
        assert_eq!(fx.for_class(SurfaceClass::Input).texture, Texture::Grain);
    }

    #[test]
    fn targets_toggle_independently() {
        use super::*;
        let t = SurfaceTargets::NONE.with(SurfaceClass::Tag, true);
        assert!(t.contains(SurfaceClass::Tag) && !t.contains(SurfaceClass::Button));
        assert!(!t.with(SurfaceClass::Tag, false).contains(SurfaceClass::Tag));
        assert!(SurfaceClass::ALL
            .iter()
            .all(|c| SurfaceTargets::ALL.contains(*c)));
    }

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
