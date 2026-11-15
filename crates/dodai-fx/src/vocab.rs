//! Surface textures and effects: patterns and animated accents drawn over a control's fill,
//! clipped to its shape.
//!
//! These are runtime settings (the showcase and apps switch them), while a design suggests
//! its own mix through `Theme::surface_style`.

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
    /// Diagonal warning bands, like hazard tape.
    Hazard,
    /// A honeycomb of hexagons.
    HexMesh,
    /// Thin vertical streaks, like rain on glass.
    Rain,
    /// Smudges and scuffs, like worn metal.
    Grime,
    /// Rows of tiny data cells, a few lit.
    DataStream,
    /// Small rectangles that look displaced by a glitch.
    GlitchBlocks,
    /// A chip die: concentric squares with pins along the edges.
    ChipDie,
    /// Parallel wires that bend at right angles.
    Wiring,
    /// Concentric radar rings with crosshair.
    Radar,
    /// Soft concentric ripples spreading from a corner (Material).
    Ripples,
    /// Bold zigzag stripes (Neo-brutalism).
    Zigzag,
    /// Houndstooth: the small houndstooth check.
    Houndstooth,
}

impl Texture {
    /// Every texture, `None` first.
    pub const ALL: [Texture; 33] = [
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
        Texture::Hazard,
        Texture::HexMesh,
        Texture::Rain,
        Texture::Grime,
        Texture::DataStream,
        Texture::GlitchBlocks,
        Texture::ChipDie,
        Texture::Wiring,
        Texture::Radar,
        Texture::Ripples,
        Texture::Zigzag,
        Texture::Houndstooth,
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
            Texture::Hazard => "Hazard",
            Texture::HexMesh => "Hex mesh",
            Texture::Rain => "Rain",
            Texture::Grime => "Grime",
            Texture::DataStream => "Data stream",
            Texture::GlitchBlocks => "Glitch blocks",
            Texture::ChipDie => "Chip die",
            Texture::Wiring => "Wiring",
            Texture::Radar => "Radar",
            Texture::Ripples => "Ripples",
            Texture::Zigzag => "Zigzag",
            Texture::Houndstooth => "Houndstooth",
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
                | Texture::ChipDie
                | Texture::Wiring
                | Texture::Radar
                | Texture::Ripples
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

    /// Whether the app-wide surface frosts this class: only what overlays the layout (popups,
    /// dropdowns, tooltips, toasts: [`Panel`](Self::Panel)). Everything else keeps the design's
    /// gradient and is frosted only when asked ([`Surface::frost`]).
    pub const fn takes_mix_frost(self) -> bool {
        matches!(self, Self::Panel)
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

/// How the click ripple plays when a control is pressed (Material).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PressStyle {
    /// A filled circle of ink spreading from the touch point to the far corner (the M3 ripple).
    #[default]
    Ink,
    /// The same circle, but spreading from the centre of the control.
    Centre,
    /// An expanding outline wave from the touch point.
    Ring,
    /// Three staggered outline waves from the touch point.
    Echo,
    /// A soft band wiping across the control from the touch point.
    Sweep,
    /// A soft glow at the touch point that fades without spreading.
    Glow,
}

impl PressStyle {
    /// Every style, in display order.
    pub const ALL: [PressStyle; 6] = [
        Self::Ink,
        Self::Centre,
        Self::Ring,
        Self::Echo,
        Self::Sweep,
        Self::Glow,
    ];

    /// Display name.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ink => "Ink",
            Self::Centre => "Centre",
            Self::Ring => "Ring",
            Self::Echo => "Echo",
            Self::Sweep => "Sweep",
            Self::Glow => "Glow",
        }
    }
}

/// The surface: one texture plus animated effects. Every amount is `0.0..=1.0`
/// (`0.0` = off), so a design preset and a slider speak the same language.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedSurface {
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
    /// Breathing light along the border.
    pub pulse: f32,
    /// A bright segment running around the border.
    pub chase: f32,
    /// Light falling inward from the edges.
    pub edge: f32,
    /// Bracket marks in the corners.
    pub brackets: f32,
    /// Inner shadow: the surface looks sunk or lit from within.
    pub depth: f32,
    /// Material: strength of the click ripple (`0.0` = none), played in `press` style.
    pub ripple: f32,
    /// Material: how the click ripple plays.
    pub press: PressStyle,
    /// Material: a tonal wash of the accent colour, like an elevation tint.
    pub tint: f32,
    /// Neo-brutalism: a hard offset block along the bottom and right inner edges.
    pub pop: f32,
    /// Neo-brutalism: strips of masking tape across two corners.
    pub tape: f32,
    /// A clear-coat reflection across the top of the surface.
    pub gloss: f32,
    /// A thin light strip gliding along the top edge.
    pub strip: f32,
    /// A frosted gradient wash: the design's own three-colour gradient laid over the surface.
    pub wash: f32,
    /// A gradient rim: the outline lit by the design's gradient, brightest where the light falls
    /// in (top left).
    pub rim: f32,
    /// How much the texture fades towards the centre, keeping text clean (`0.0` = even).
    pub clear: f32,
    /// Frosted glass: the fill turns as see-through as the design's glass and a blurred copy
    /// of the app's backdrop shows under it (`0.0` = an opaque fill). Every design has glass,
    /// so no design limits it; it reaches containers and input fields, unless a component sets it itself.
    pub frost: f32,
    /// Texture for containers (cards, panels, bars); `texture` is for UI components.
    pub container_texture: Texture,
    /// Strength of the container texture.
    pub container_strength: f32,
    /// Scale of the container texture.
    pub container_scale: f32,
    /// Which kinds of surface receive the mix.
    pub targets: SurfaceTargets,
}

impl Default for ResolvedSurface {
    fn default() -> Self {
        Self::NONE
    }
}

impl ResolvedSurface {
    /// Nothing: no texture, no effects.
    pub const NONE: ResolvedSurface = ResolvedSurface {
        texture: Texture::None,
        strength: 0.5,
        scale: 1.0,
        glitch: 0.0,
        sheen: 0.0,
        flicker: 0.0,
        bloom: 0.0,
        neon: 0.0,
        pulse: 0.0,
        chase: 0.0,
        edge: 0.0,
        brackets: 0.0,
        depth: 0.0,
        ripple: 0.0,
        press: PressStyle::Ink,
        tint: 0.0,
        pop: 0.0,
        tape: 0.0,
        gloss: 0.0,
        strip: 0.0,
        wash: 0.0,
        rim: 0.0,
        clear: 0.0,
        frost: 0.0,
        container_texture: Texture::None,
        container_strength: 0.5,
        container_scale: 1.0,
        targets: SurfaceTargets::DEFAULT,
    };

    /// The mix restricted to what a design owns: textures outside `textures` become
    /// [`Texture::None`], and the animated effects are zeroed unless `effects`.
    pub fn limited(mut self, textures: &[Texture], effects: EffectSet) -> Self {
        if !textures.contains(&self.texture) {
            self.texture = Texture::None;
        }
        if !textures.contains(&self.container_texture) {
            self.container_texture = Texture::None;
        }
        let keep = |on: bool, v: f32| if on { v } else { 0.0 };
        self.glitch = keep(effects.glitch, self.glitch);
        self.sheen = keep(effects.sheen, self.sheen);
        self.flicker = keep(effects.flicker, self.flicker);
        self.bloom = keep(effects.bloom, self.bloom);
        self.pulse = keep(effects.pulse, self.pulse);
        self.chase = keep(effects.chase, self.chase);
        self.edge = keep(effects.edge, self.edge);
        self.brackets = keep(effects.brackets, self.brackets);
        self.depth = keep(effects.depth, self.depth);
        self.neon = keep(effects.neon, self.neon);
        self.ripple = keep(effects.ripple, self.ripple);
        self.tint = keep(effects.tint, self.tint);
        self.pop = keep(effects.pop, self.pop);
        self.tape = keep(effects.tape, self.tape);
        self.gloss = keep(effects.gloss, self.gloss);
        self.strip = keep(effects.strip, self.strip);
        self.wash = keep(effects.wash, self.wash);
        self.rim = keep(effects.rim, self.rim);
        self
    }

    /// The mix as seen by a `class` surface: containers use the container texture.
    pub fn for_class(self, class: SurfaceClass) -> Self {
        let base = if class.is_container() {
            Self {
                texture: self.container_texture,
                strength: self.container_strength,
                scale: self.container_scale,
                ..self
            }
        } else {
            self
        };
        if base.frost > 0.0 && class.takes_mix_frost() {
            base
        } else {
            Self { frost: 0.0, ..base }
        }
    }

    /// True when a texture is set and visible.
    pub fn has_texture(&self) -> bool {
        self.texture != Texture::None && self.strength > 0.0
    }

    /// True when any animated effect is on (the surface needs frames).
    pub fn animated(&self) -> bool {
        self.glitch > 0.0
            || self.sheen > 0.0
            || self.flicker > 0.0
            || self.pulse > 0.0
            || self.chase > 0.0
            || self.strip > 0.0
    }

    /// True when nothing would be drawn.
    pub fn is_none(&self) -> bool {
        !self.has_texture()
            && !self.animated()
            && self.bloom <= 0.0
            && self.edge <= 0.0
            && self.brackets <= 0.0
            && self.depth <= 0.0
            && self.ripple <= 0.0
            && self.tint <= 0.0
            && self.pop <= 0.0
            && self.tape <= 0.0
            && self.gloss <= 0.0
            && self.wash <= 0.0
            && self.rim <= 0.0
            && self.frost <= 0.0
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
        let fx = ResolvedSurface {
            texture: Texture::Grain,
            container_texture: Texture::Grid,
            ..ResolvedSurface::NONE
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
        let d = ResolvedSurface::default();
        assert!(d.is_none() && !d.animated() && !d.has_texture());
        assert!(!ResolvedSurface { glitch: 0.4, ..d }.is_none());
        assert!(ResolvedSurface { glitch: 0.4, ..d }.animated());
        assert!(ResolvedSurface {
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

/// Which light effects a design owns. A design draws only the ones it lists, so each keeps
/// its own character (a neon design glitches, a material one only shimmers).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectSet {
    /// Glitch bursts.
    pub glitch: bool,
    /// Light sweeping across.
    pub sheen: bool,
    /// Flicker.
    pub flicker: bool,
    /// Bloom past the edge.
    pub bloom: bool,
    /// A pulsing border light.
    pub pulse: bool,
    /// A light chasing around the border.
    pub chase: bool,
    /// Light falling in from the edges.
    pub edge: bool,
    /// Corner brackets.
    pub brackets: bool,
    /// Inner shadow.
    pub depth: bool,
    /// Lit lines in textures.
    pub neon: bool,
    /// Click ripple.
    pub ripple: bool,
    /// Tonal wash.
    pub tint: bool,
    /// Hard offset block.
    pub pop: bool,
    /// Masking-tape corners.
    pub tape: bool,
    /// Clear-coat reflection.
    pub gloss: bool,
    /// Gliding light strip.
    pub strip: bool,
    /// Gradient wash.
    pub wash: bool,
    /// Gradient rim.
    pub rim: bool,
}

impl EffectSet {
    /// No effects.
    pub const NONE: Self = Self {
        glitch: false,
        sheen: false,
        flicker: false,
        bloom: false,
        pulse: false,
        chase: false,
        edge: false,
        brackets: false,
        depth: false,
        neon: false,
        ripple: false,
        tint: false,
        pop: false,
        tape: false,
        gloss: false,
        strip: false,
        wash: false,
        rim: false,
    };
    /// Every effect.
    pub const ALL: Self = Self {
        glitch: true,
        sheen: true,
        flicker: true,
        bloom: true,
        pulse: true,
        chase: true,
        edge: true,
        brackets: true,
        depth: true,
        neon: true,
        ripple: true,
        tint: true,
        pop: true,
        tape: true,
        gloss: true,
        strip: true,
        wash: true,
        rim: true,
    };
    /// Whether any effect is owned.
    pub const fn any(&self) -> bool {
        self.glitch
            || self.sheen
            || self.flicker
            || self.bloom
            || self.pulse
            || self.chase
            || self.edge
            || self.brackets
            || self.depth
            || self.neon
            || self.ripple
            || self.tint
            || self.pop
            || self.tape
            || self.gloss
            || self.strip
            || self.wash
            || self.rim
    }
}

/// How a design's textures and effects are timed and sized. The defaults are the neutral values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceTuning {
    /// Multiplier on every texture's cell and line spacing (1 = as drawn).
    pub cell: f32,
    /// Width in px of the halo around lit lines.
    pub halo: f32,
    /// Width in px a full bloom spreads past the edge.
    pub bloom_spread: f32,
    /// Flicker steps per second.
    pub flicker_rate: f32,
    /// Chance per step, at full flicker, that the surface dims.
    pub flicker_chance: f32,
    /// Seconds between sheen sweeps.
    pub sheen_period: f32,
    /// Seconds between glitch bursts.
    pub glitch_period: f32,
    /// Seconds a glitch burst lasts at the lowest setting (grows with the amount).
    pub glitch_burst: f32,
    /// Seconds for one pulse of the border light.
    pub pulse_period: f32,
    /// Seconds for the border chase to run once around.
    pub chase_period: f32,
}

impl SurfaceTuning {
    /// The neutral timing and sizes.
    pub const DEFAULT: Self = Self {
        cell: 1.0,
        halo: 2.0,
        bloom_spread: 3.0,
        flicker_rate: 18.0,
        flicker_chance: 0.18,
        sheen_period: 3.6,
        glitch_period: 2.6,
        glitch_burst: 0.1,
        pulse_period: 2.4,
        chase_period: 4.0,
    };
}

impl Default for SurfaceTuning {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A per-component override of the app-wide surface, set with a component's
/// `.surface(..)` method.
///
/// Every field is optional: what is not set falls back to the app-wide mix
/// ([`ResolvedSurface`]), which falls back to the design's own. Setting any field puts the
/// component in the mix even when its kind is not under "Apply to"; [`Surface::NONE`]
/// takes it out. A design still draws only the textures and effects it owns.
///
/// ```
/// use dodai_fx::{Surface, Texture};
/// let calm = Surface::new().texture(Texture::Grain).strength(0.3);
/// let plain = Surface::NONE;
/// assert!(calm.is_set() && plain.is_off());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Surface {
    on: Option<bool>,
    texture: Option<Texture>,
    strength: Option<f32>,
    scale: Option<f32>,
    neon: Option<f32>,
    glitch: Option<f32>,
    sheen: Option<f32>,
    flicker: Option<f32>,
    bloom: Option<f32>,
    pulse: Option<f32>,
    chase: Option<f32>,
    edge: Option<f32>,
    brackets: Option<f32>,
    depth: Option<f32>,
    ripple: Option<f32>,
    press: Option<PressStyle>,
    tint: Option<f32>,
    pop: Option<f32>,
    tape: Option<f32>,
    gloss: Option<f32>,
    strip: Option<f32>,
    wash: Option<f32>,
    rim: Option<f32>,
    clear: Option<f32>,
    frost: Option<f32>,
    glow: Option<f32>,
    opacity: Option<f32>,
    container_texture: Option<Texture>,
    container_strength: Option<f32>,
    container_scale: Option<f32>,
    targets: Option<SurfaceTargets>,
}

impl Surface {
    /// Follow the app-wide mix (the default).
    pub const INHERIT: Self = Self {
        on: None,
        texture: None,
        strength: None,
        scale: None,
        neon: None,
        glitch: None,
        sheen: None,
        flicker: None,
        bloom: None,
        pulse: None,
        chase: None,
        edge: None,
        brackets: None,
        depth: None,
        ripple: None,
        press: None,
        tint: None,
        pop: None,
        tape: None,
        gloss: None,
        strip: None,
        wash: None,
        rim: None,
        clear: None,
        frost: None,
        glow: None,
        opacity: None,
        container_texture: None,
        container_strength: None,
        container_scale: None,
        targets: None,
    };
    /// No texture or effect on this component, whatever the app-wide mix is.
    pub const NONE: Self = Self {
        on: Some(false),
        ..Self::INHERIT
    };

    /// Frosted glass at full strength: containers and input fields turn see-through over a
    /// blurred backdrop. Chain more effects onto it: `Surface::frosted().glitch(0.6)`.
    pub const fn frosted() -> Self {
        Self::INHERIT.frost(1.0)
    }
    /// Frosted glass at `amount` (`0.0..=1.0`).
    pub const fn frosted_by(amount: f32) -> Self {
        Self::INHERIT.frost(amount)
    }

    /// An override that sets nothing yet; chain the setters below.
    pub const fn new() -> Self {
        Self::INHERIT
    }
    /// Use this texture (also puts the component in the mix).
    pub const fn texture(mut self, t: Texture) -> Self {
        self.texture = Some(t);
        self
    }
    /// Texture visibility, `0.0..=1.0`.
    pub const fn strength(mut self, v: f32) -> Self {
        self.strength = Some(v);
        self
    }
    /// Pattern size multiplier (`1.0` = as designed).
    pub const fn scale(mut self, v: f32) -> Self {
        self.scale = Some(v);
        self
    }
    /// Lit lines on line textures, `0.0..=1.0`.
    pub const fn neon(mut self, v: f32) -> Self {
        self.neon = Some(v);
        self
    }
    /// Glitch bursts, `0.0..=1.0`.
    pub const fn glitch(mut self, v: f32) -> Self {
        self.glitch = Some(v);
        self
    }
    /// Sheen sweep, `0.0..=1.0`.
    pub const fn sheen(mut self, v: f32) -> Self {
        self.sheen = Some(v);
        self
    }
    /// Flicker, `0.0..=1.0`.
    pub const fn flicker(mut self, v: f32) -> Self {
        self.flicker = Some(v);
        self
    }
    /// Bloom around the outline, `0.0..=1.0`.
    pub const fn bloom(mut self, v: f32) -> Self {
        self.bloom = Some(v);
        self
    }
    /// Breathing border light, `0.0..=1.0`.
    pub const fn pulse(mut self, v: f32) -> Self {
        self.pulse = Some(v);
        self
    }
    /// A light segment running around the border, `0.0..=1.0`.
    pub const fn chase(mut self, v: f32) -> Self {
        self.chase = Some(v);
        self
    }
    /// Light falling inward from the edges, `0.0..=1.0`.
    pub const fn edge(mut self, v: f32) -> Self {
        self.edge = Some(v);
        self
    }
    /// Corner brackets, `0.0..=1.0`.
    pub const fn brackets(mut self, v: f32) -> Self {
        self.brackets = Some(v);
        self
    }
    /// Click ripple, `0.0..=1.0`.
    /// How the click ripple plays.
    pub const fn press(mut self, v: PressStyle) -> Self {
        self.press = Some(v);
        self
    }
    /// Click ripple strength, `0.0..=1.0`.
    pub const fn ripple(mut self, v: f32) -> Self {
        self.ripple = Some(v);
        self
    }
    /// Tonal wash, `0.0..=1.0`.
    pub const fn tint(mut self, v: f32) -> Self {
        self.tint = Some(v);
        self
    }
    /// Hard offset block, `0.0..=1.0`.
    pub const fn pop(mut self, v: f32) -> Self {
        self.pop = Some(v);
        self
    }
    /// Masking-tape corners, `0.0..=1.0`.
    pub const fn tape(mut self, v: f32) -> Self {
        self.tape = Some(v);
        self
    }
    /// Clear-coat reflection, `0.0..=1.0`.
    pub const fn gloss(mut self, v: f32) -> Self {
        self.gloss = Some(v);
        self
    }
    /// Gradient wash, `0.0..=1.0`.
    pub const fn wash(mut self, v: f32) -> Self {
        self.wash = Some(v);
        self
    }
    /// Gradient rim, `0.0..=1.0`.
    pub const fn rim(mut self, v: f32) -> Self {
        self.rim = Some(v);
        self
    }

    /// Gliding light strip, `0.0..=1.0`.
    pub const fn strip(mut self, v: f32) -> Self {
        self.strip = Some(v);
        self
    }
    /// Inner shadow, `0.0..=1.0`.
    pub const fn depth(mut self, v: f32) -> Self {
        self.depth = Some(v);
        self
    }
    /// Fade the texture towards the centre, `0.0..=1.0`.
    pub const fn clear(mut self, v: f32) -> Self {
        self.clear = Some(v);
        self
    }
    /// Frosted glass, `0.0..=1.0`: see-through fill over a blurred backdrop. Works on any
    /// component, buttons and inputs included (the app-wide mix only frosts containers).
    pub const fn frost(mut self, v: f32) -> Self {
        self.frost = Some(v);
        self
    }
    /// How opaque the component's fill is, `0.0..=1.0` (`1.0` = as designed): the fill alone
    /// turns see-through, content and border stay. Works with or without frost and does not
    /// put the component in the texture mix.
    pub const fn opacity(mut self, v: f32) -> Self {
        self.opacity = Some(v);
        self
    }
    /// The fill opacity, when set.
    pub fn opacity_value(&self) -> Option<f32> {
        self.opacity
    }
    /// Scale the design's glow on this component (`0.0` = none, `1.0` = as designed).
    pub const fn glow(mut self, v: f32) -> Self {
        self.glow = Some(v);
        self
    }
    /// Texture for containers (cards, panels, bars); [`texture`](Self::texture) is for controls.
    pub const fn container_texture(mut self, t: Texture) -> Self {
        self.container_texture = Some(t);
        self
    }
    /// Strength of the container texture, `0.0..=1.0`.
    pub const fn container_strength(mut self, v: f32) -> Self {
        self.container_strength = Some(v);
        self
    }
    /// Scale of the container texture.
    pub const fn container_scale(mut self, v: f32) -> Self {
        self.container_scale = Some(v);
        self
    }
    /// Which kinds of surface receive the app-wide mix (ignored on a single component).
    pub const fn targets(mut self, t: SurfaceTargets) -> Self {
        self.targets = Some(t);
        self
    }
    /// Apply the app-wide mix to every kind of surface, buttons and tags included.
    pub const fn everywhere(self) -> Self {
        self.targets(SurfaceTargets::ALL)
    }
    /// Force the component into (`true`) or out of (`false`) the mix without changing it.
    pub const fn enabled(mut self, on: bool) -> Self {
        self.on = Some(on);
        self
    }

    /// Whether anything is overridden.
    pub fn is_set(&self) -> bool {
        *self != Self::INHERIT
    }
    /// Whether the component is explicitly out of the mix.
    pub fn is_off(&self) -> bool {
        self.on == Some(false)
    }
    /// The glow scale, when set.
    pub fn glow_scale(&self) -> Option<f32> {
        self.glow
    }

    /// Whether this override puts the component in the mix regardless of "Apply to".
    pub fn forces_on(&self) -> bool {
        self.on == Some(true) || (self.is_set() && !self.is_off() && self.sets_fx())
    }

    fn sets_fx(&self) -> bool {
        self.texture.is_some()
            || self.strength.is_some()
            || self.scale.is_some()
            || self.neon.is_some()
            || self.glitch.is_some()
            || self.sheen.is_some()
            || self.flicker.is_some()
            || self.bloom.is_some()
            || self.pulse.is_some()
            || self.chase.is_some()
            || self.edge.is_some()
            || self.brackets.is_some()
            || self.depth.is_some()
            || self.ripple.is_some()
            || self.press.is_some()
            || self.tint.is_some()
            || self.pop.is_some()
            || self.tape.is_some()
            || self.gloss.is_some()
            || self.strip.is_some()
            || self.wash.is_some()
            || self.rim.is_some()
            || self.frost.is_some()
    }

    /// `fx` (already resolved for the component's class) with this override on top.
    pub fn over(&self, mut fx: ResolvedSurface) -> ResolvedSurface {
        if let Some(v) = self.texture {
            fx.texture = v;
        }
        if let Some(v) = self.strength {
            fx.strength = v;
        }
        if let Some(v) = self.scale {
            fx.scale = v;
        }
        if let Some(v) = self.neon {
            fx.neon = v;
        }
        if let Some(v) = self.glitch {
            fx.glitch = v;
        }
        if let Some(v) = self.sheen {
            fx.sheen = v;
        }
        if let Some(v) = self.flicker {
            fx.flicker = v;
        }
        if let Some(v) = self.bloom {
            fx.bloom = v;
        }
        if let Some(v) = self.pulse {
            fx.pulse = v;
        }
        if let Some(v) = self.chase {
            fx.chase = v;
        }
        if let Some(v) = self.edge {
            fx.edge = v;
        }
        if let Some(v) = self.brackets {
            fx.brackets = v;
        }
        if let Some(v) = self.depth {
            fx.depth = v;
        }
        if let Some(v) = self.ripple {
            fx.ripple = v;
        }
        if let Some(v) = self.press {
            fx.press = v;
        }
        if let Some(v) = self.tint {
            fx.tint = v;
        }
        if let Some(v) = self.pop {
            fx.pop = v;
        }
        if let Some(v) = self.tape {
            fx.tape = v;
        }
        if let Some(v) = self.gloss {
            fx.gloss = v;
        }
        if let Some(v) = self.strip {
            fx.strip = v;
        }
        if let Some(v) = self.wash {
            fx.wash = v;
        }
        if let Some(v) = self.rim {
            fx.rim = v;
        }
        if let Some(v) = self.clear {
            fx.clear = v;
        }
        if let Some(v) = self.frost {
            fx.frost = v;
        }
        fx
    }

    /// The app-wide mix: `base` (the design's own) with every field this sets on top, including
    /// the container texture and where it applies.
    pub fn apply(&self, base: ResolvedSurface) -> ResolvedSurface {
        let mut fx = self.over(base);
        if let Some(v) = self.container_texture {
            fx.container_texture = v;
        }
        if let Some(v) = self.container_strength {
            fx.container_strength = v;
        }
        if let Some(v) = self.container_scale {
            fx.container_scale = v;
        }
        if let Some(v) = self.targets {
            fx.targets = v;
        }
        fx
    }

    /// This with every field `other` sets on top: `Surface::frosted().and(Surface::new().glitch(0.6))`.
    #[must_use]
    pub fn and(self, other: Self) -> Self {
        macro_rules! pick {
            ($($f:ident),*) => { Self { $($f: other.$f.or(self.$f)),* } };
        }
        pick!(
            on,
            texture,
            strength,
            scale,
            neon,
            glitch,
            sheen,
            flicker,
            bloom,
            pulse,
            chase,
            edge,
            brackets,
            depth,
            ripple,
            press,
            tint,
            pop,
            tape,
            gloss,
            strip,
            wash,
            rim,
            clear,
            frost,
            glow,
            opacity,
            container_texture,
            container_strength,
            container_scale,
            targets
        )
    }
}

impl From<ResolvedSurface> for Surface {
    /// Every value set: the mix is exactly `fx`, whatever the design would have done.
    fn from(fx: ResolvedSurface) -> Self {
        Self {
            on: None,
            texture: Some(fx.texture),
            strength: Some(fx.strength),
            scale: Some(fx.scale),
            neon: Some(fx.neon),
            glitch: Some(fx.glitch),
            sheen: Some(fx.sheen),
            flicker: Some(fx.flicker),
            bloom: Some(fx.bloom),
            pulse: Some(fx.pulse),
            chase: Some(fx.chase),
            edge: Some(fx.edge),
            brackets: Some(fx.brackets),
            depth: Some(fx.depth),
            ripple: Some(fx.ripple),
            press: Some(fx.press),
            tint: Some(fx.tint),
            pop: Some(fx.pop),
            tape: Some(fx.tape),
            gloss: Some(fx.gloss),
            strip: Some(fx.strip),
            wash: Some(fx.wash),
            rim: Some(fx.rim),
            clear: Some(fx.clear),
            frost: Some(fx.frost),
            glow: None,
            opacity: None,
            container_texture: Some(fx.container_texture),
            container_strength: Some(fx.container_strength),
            container_scale: Some(fx.container_scale),
            targets: Some(fx.targets),
        }
    }
}

#[cfg(test)]
mod surface_override_tests {
    use super::*;

    #[test]
    fn inherit_changes_nothing_and_none_is_off() {
        let fx = ResolvedSurface {
            texture: Texture::Grid,
            strength: 0.7,
            ..ResolvedSurface::NONE
        };
        assert_eq!(Surface::INHERIT.over(fx), fx);
        assert!(!Surface::INHERIT.is_set() && !Surface::INHERIT.forces_on());
        assert!(Surface::NONE.is_off() && !Surface::NONE.forces_on());
    }

    #[test]
    fn set_fields_win_and_force_the_component_in() {
        let fx = ResolvedSurface {
            texture: Texture::Grid,
            strength: 0.7,
            ..ResolvedSurface::NONE
        };
        let s = Surface::new().texture(Texture::Hatch).glitch(0.5);
        let o = s.over(fx);
        assert_eq!(o.texture, Texture::Hatch);
        assert_eq!(o.strength, 0.7);
        assert_eq!(o.glitch, 0.5);
        assert!(s.forces_on());
        // A glow-only override does not pull a component into the texture mix.
        assert!(!Surface::new().glow(0.0).forces_on());
        // Nor does an opacity-only one.
        let o = Surface::new().opacity(0.4);
        assert!(o.is_set() && !o.forces_on());
        assert_eq!(o.opacity_value(), Some(0.4));
    }

    #[test]
    fn the_mix_frosts_overlays_and_a_component_can_frost_itself() {
        let fx = ResolvedSurface {
            frost: 1.0,
            ..ResolvedSurface::NONE
        };
        assert!(!fx.is_none(), "frost alone is something to draw");
        assert_eq!(fx.for_class(SurfaceClass::Panel).frost, 1.0);
        assert_eq!(fx.for_class(SurfaceClass::Card).frost, 0.0);
        assert_eq!(fx.for_class(SurfaceClass::Input).frost, 0.0);
        assert_eq!(fx.for_class(SurfaceClass::Button).frost, 0.0);
        assert_eq!(fx.for_class(SurfaceClass::Tag).frost, 0.0);
        let own = Surface::new().frost(0.5);
        assert!(own.forces_on());
        assert_eq!(own.over(fx.for_class(SurfaceClass::Button)).frost, 0.5);
        // No design limits it.
        assert_eq!(fx.limited(&[], EffectSet::NONE).frost, 1.0);
    }
}

#[cfg(test)]
mod surface_api_tests {
    use super::*;

    #[test]
    fn frosted_sets_only_frost_and_chains() {
        let s = Surface::frosted().glitch(0.6);
        let fx = s.apply(ResolvedSurface::NONE);
        assert_eq!((fx.frost, fx.glitch), (1.0, 0.6));
        assert_eq!(fx.neon, 0.0);
        assert_eq!(
            Surface::frosted_by(0.4).apply(ResolvedSurface::NONE).frost,
            0.4
        );
    }

    #[test]
    fn and_prefers_the_later_value_and_keeps_the_rest() {
        let a = Surface::frosted_by(0.3).glitch(0.2);
        let b = Surface::new().glitch(0.9).neon(1.0);
        let m = a.and(b);
        let fx = m.apply(ResolvedSurface::NONE);
        assert_eq!((fx.frost, fx.glitch, fx.neon), (0.3, 0.9, 1.0));
        assert_eq!(Surface::INHERIT.and(a), a);
        assert_eq!(a.and(Surface::INHERIT), a);
    }

    #[test]
    fn apply_sets_container_texture_and_targets_but_over_does_not() {
        let s = Surface::new()
            .container_texture(Texture::Grid)
            .container_strength(0.7)
            .container_scale(2.0)
            .everywhere();
        let fx = s.apply(ResolvedSurface::NONE);
        assert_eq!(fx.container_texture, Texture::Grid);
        assert_eq!((fx.container_strength, fx.container_scale), (0.7, 2.0));
        assert_eq!(fx.targets, SurfaceTargets::ALL);
        // A single component never changes the app-wide container settings.
        assert_eq!(
            s.over(ResolvedSurface::NONE).container_texture,
            Texture::None
        );
    }

    #[test]
    fn a_resolved_surface_round_trips() {
        let fx = ResolvedSurface {
            texture: Texture::Hatch,
            glitch: 0.5,
            frost: 0.8,
            container_texture: Texture::Dither,
            targets: SurfaceTargets::ALL,
            ..ResolvedSurface::NONE
        };
        assert_eq!(Surface::from(fx).apply(ResolvedSurface::NONE), fx);
        // Even over a busy base: every value is set, so none leaks through.
        let busy = ResolvedSurface {
            neon: 1.0,
            ..ResolvedSurface::NONE
        };
        assert_eq!(Surface::from(fx).apply(busy), fx);
    }
}
