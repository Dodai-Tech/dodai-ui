//! **Design-system spec**: how one design system resolves component tokens and geometry.
//!
//! Core only knows the *vocabulary* (token structs, [`Theme`] methods). A design system crate
//! (the `material`, `cyberpunk`, … modules) implements [`DesignSpec`] and attaches it to its themes with
//! [`Theme::with_spec`]. Every hook returns `None` by default, which falls back to the generic
//! baseline in [`Theme`]'s resolvers, so a spec only lists what it does differently.

use crate::{
    AppBarKind, AppBarTokens, ButtonTokens, ButtonVariant, CardVariant, Color, ControlSize,
    ControlState, InputTokens, ListItemTokens, NavigationTokens, ProgressTokens, SizeMetrics,
    Sizes, StepState, StepTokens, SurfaceTokens, TabTokens, Theme, ToggleKind, ToggleTokens, Tone,
    TypeRole, TypeStyle,
};

/// The icon pack a design system draws from. A design names its own source, so no pack is a
/// silent fallback for another.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum IconSet {
    /// Hand-drawn stroked icons (the classic pack).
    #[default]
    Stroked,
    /// Material Symbols glyphs.
    Symbols,
    /// Heavy, chunky glyphs (the Neo-Brutalism pack).
    Bold,
}

/// Semantic capabilities a design system declares. Adapters ask these instead of guessing a
/// look from the corner geometry, so a new rounded design never inherits another design's layout.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Look {
    /// Tonal surfaces and the matching component structure: container-coloured pills, tonal
    /// selection, borderless cards, taller touch targets.
    pub tonal: bool,
    /// Blurred elevation shadows (levels 1-5) instead of hard offset shadows or none.
    pub soft_elevation: bool,
    /// Text fields with a floating label inside a filled container.
    pub floating_label: bool,
    /// Which icon pack the design draws its icons from.
    pub icons: IconSet,
    /// Hard-shadowed controls sink into their shadow on hover and press.
    pub sink: bool,
    /// Neon look: the design supports glow and scanline effects.
    pub luminous: bool,
    /// Corners may be cut (chamfered) per corner.
    pub cut_corners: bool,
    /// Panels carry an accent border segment alongside the contour.
    pub accent_border: bool,
}

/// The page grid of a design at a viewport width: columns between two outer margins.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridSpec {
    pub columns: u32,
    /// Gap between columns and rows.
    pub gap: f32,
    /// Outer margin on each side.
    pub margin: f32,
    /// The widest the grid grows; beyond it the page is centred.
    pub max_width: Option<f32>,
}

/// One layer of a soft shadow: offset down by `dy`, `blur` and `spread` in px, `alpha` absolute
/// (the colour is the theme's shadow colour).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShadowLayer {
    pub dy: f32,
    pub blur: f32,
    pub spread: f32,
    pub alpha: f32,
}

/// Frosted glass: a translucent `tint` laid over a backdrop blurred by `blur` px (CSS blur
/// sigma).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frost {
    pub tint: crate::Color,
    pub blur: f32,
    /// An outline around the glass `(colour, width px)`, for designs whose panels have one.
    pub edge: Option<(crate::Color, f32)>,
}

/// Type sizes and spacing a design resolves for one viewport width (fluid scales). Start from
/// the theme's own `typography` and `spacing` and change the sizes.
#[derive(Debug, Clone, PartialEq)]
pub struct Fluid {
    pub typography: crate::Typography,
    pub spacing: crate::Spacing,
}

/// One stop of a media fade: `alpha` of the fade colour at `at` (0 = the edge the text sits on,
/// 1 = the far edge).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadeStop {
    pub alpha: f32,
    pub at: f32,
}

/// Component rules of one design system. All hooks are optional overrides.
pub trait DesignSpec: Send + Sync + std::fmt::Debug {
    /// Display name, e.g. `"Material 3"`.
    fn name(&self) -> &'static str;

    /// The capabilities of this design (default: none, i.e. the classic look).
    fn look(&self) -> Look {
        Look::default()
    }

    fn card(&self, _t: &Theme, _v: CardVariant) -> Option<SurfaceTokens> {
        None
    }
    fn chip(&self, _t: &Theme, _selected: bool, _hovered: bool) -> Option<SurfaceTokens> {
        None
    }
    fn alert(&self, _t: &Theme, _tone: Tone, _banner: bool) -> Option<SurfaceTokens> {
        None
    }
    fn snackbar(&self, _t: &Theme) -> Option<(SurfaceTokens, Color)> {
        None
    }
    fn dialog(&self, _t: &Theme) -> Option<SurfaceTokens> {
        None
    }
    fn sheet(&self, _t: &Theme) -> Option<SurfaceTokens> {
        None
    }
    fn popup(&self, _t: &Theme) -> Option<SurfaceTokens> {
        None
    }
    fn scrim(&self, _t: &Theme) -> Option<Color> {
        None
    }
    fn app_bar(&self, _t: &Theme, _kind: AppBarKind, _scrolled: bool) -> Option<AppBarTokens> {
        None
    }
    fn step(&self, _t: &Theme, _state: StepState) -> Option<StepTokens> {
        None
    }
    fn tab(&self, _t: &Theme, _active: bool, _hovered: bool) -> Option<TabTokens> {
        None
    }
    fn toggle(&self, _t: &Theme, _on: bool, _state: ControlState) -> Option<ToggleTokens> {
        None
    }
    fn choice_toggle(
        &self,
        _t: &Theme,
        _kind: ToggleKind,
        _on: bool,
        _state: ControlState,
    ) -> Option<ToggleTokens> {
        None
    }
    fn badge(&self, _t: &Theme, _tone: Tone) -> Option<SurfaceTokens> {
        None
    }
    fn progress(&self, _t: &Theme, _tone: Option<Tone>) -> Option<ProgressTokens> {
        None
    }
    fn list_item(&self, _t: &Theme, _selected: bool, _hovered: bool) -> Option<ListItemTokens> {
        None
    }
    fn sizes(&self, _t: &Theme) -> Option<Sizes> {
        None
    }
    fn navigation(&self, _t: &Theme) -> Option<NavigationTokens> {
        None
    }
    fn navigation_item(
        &self,
        _t: &Theme,
        _selected: bool,
        _hovered: bool,
    ) -> Option<ListItemTokens> {
        None
    }
    fn divider(&self, _t: &Theme) -> Option<Color> {
        None
    }
    fn button(&self, _t: &Theme, _v: ButtonVariant, _s: ControlState) -> Option<ButtonTokens> {
        None
    }
    fn elevated_button(&self, _t: &Theme, _s: ControlState) -> Option<ButtonTokens> {
        None
    }
    fn fab(&self, _t: &Theme, _v: ButtonVariant, _s: ControlState) -> Option<ButtonTokens> {
        None
    }
    /// Adjust the icon-button tokens `base` (already resolved as a plain button).
    fn icon_button(
        &self,
        _t: &Theme,
        _v: ButtonVariant,
        _s: ControlState,
        _selected: bool,
        _base: ButtonTokens,
    ) -> Option<ButtonTokens> {
        None
    }
    fn input(&self, _t: &Theme, _focused: bool) -> Option<InputTokens> {
        None
    }
    /// How charts draw lines, outlines, gridlines and fills (default: derived from the palette).
    fn chart_style(&self, _t: &Theme) -> Option<crate::ChartStyle> {
        None
    }
    /// Colour of the hard offset shadow (default: the pressed primary).
    fn hard_shadow_color(&self, _t: &Theme) -> Option<Color> {
        None
    }
    /// Soft shadow layers for an elevation `level` (1-5), when the design has its own shadow
    /// tokens instead of the two Material layers.
    fn elevation_layers(&self, _t: &Theme, _level: u8) -> Option<Vec<ShadowLayer>> {
        None
    }
    /// The page grid at a viewport width (default: the theme's own spacing, 12 columns, or 4 on
    /// a narrow viewport).
    fn page_grid(&self, _t: &Theme, _viewport: f32) -> Option<GridSpec> {
        None
    }
    /// Type sizes and spacing at a viewport width, for designs with fluid scales (default: the static theme values).
    fn fluid(&self, _t: &Theme, _viewport: f32) -> Option<Fluid> {
        None
    }
    /// The design's frosted glass (default: the surface colour at 60% over a 24 px blur).
    fn frost(&self, _t: &Theme) -> Option<Frost> {
        None
    }
    /// Stops of the dark fade laid over media so text on it stays legible (default: a two-stop
    /// linear fade).
    fn fade_stops(&self, _t: &Theme) -> Option<Vec<FadeStop>> {
        None
    }
    /// Keyboard focus indicator `(colour, thickness px)`.
    fn focus_ring(&self, _t: &Theme) -> Option<(Color, f32)> {
        None
    }
    fn list_row_height(&self, _t: &Theme, _nested: bool) -> Option<f32> {
        None
    }
    fn list_lines_height(&self, _t: &Theme, _lines: u8) -> Option<f32> {
        None
    }
    fn menu_row_height(&self, _t: &Theme) -> Option<f32> {
        None
    }
    /// Tooltip `(container, label)`.
    fn tooltip(&self, _t: &Theme, _rich: bool) -> Option<(Color, Color)> {
        None
    }
    /// Horizontal padding of a button label.
    fn control_metrics(&self, _t: &Theme, _size: ControlSize) -> Option<SizeMetrics> {
        None
    }
    fn type_style(&self, _t: &Theme, _role: TypeRole) -> Option<TypeStyle> {
        None
    }
    fn button_padding(&self, _t: &Theme) -> Option<f32> {
        None
    }
}

/// The spec attached to a [`Theme`]: `None` means the generic baseline.
#[derive(Clone, Copy, Default)]
pub struct SpecRef(Option<&'static dyn DesignSpec>);

impl SpecRef {
    /// Attach a design-system spec (usually a `static` unit struct).
    pub const fn new(spec: &'static dyn DesignSpec) -> Self {
        Self(Some(spec))
    }
    /// The spec, if the theme has one.
    pub fn get(&self) -> Option<&'static dyn DesignSpec> {
        self.0
    }
    /// Name of the spec (`"baseline"` when none is set).
    pub fn name(&self) -> &'static str {
        self.0.map_or("baseline", |s| s.name())
    }
}

impl std::fmt::Debug for SpecRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpecRef({})", self.name())
    }
}

impl PartialEq for SpecRef {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}
