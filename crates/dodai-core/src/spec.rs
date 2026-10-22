//! **Design-system spec**: how one design system resolves component tokens and geometry.
//!
//! Core only knows the *vocabulary* (token structs, [`Theme`] methods). A design system crate
//! (`dodai-material`, …) implements [`DesignSpec`] and attaches it to its themes with
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
pub struct PageGrid {
    pub columns: u32,
    /// Gap between columns and rows.
    pub gap: f32,
    /// Outer margin on each side.
    pub margin: f32,
    /// The widest the grid grows; beyond it the page is centred.
    pub max_width: Option<f32>,
}

/// Type sizes and spacing a design resolves for one viewport width (fluid scales).
#[derive(Debug, Clone, PartialEq)]
pub struct Fluid {
    pub typography: [f32; 6],
    pub spacing: [f32; 5],
}

/// Component rules of one design system. All hooks are optional overrides.
#[allow(unused_variables)]
pub trait DesignSpec: Send + Sync + std::fmt::Debug {
    /// Display name, e.g. `"Material 3"`.
    fn name(&self) -> &'static str;

    /// The capabilities of this design (default: none, i.e. the classic look).
    fn look(&self) -> Look {
        Look::default()
    }

    fn card(&self, t: &Theme, v: CardVariant) -> Option<SurfaceTokens> {
        None
    }
    fn chip(&self, t: &Theme, selected: bool, hovered: bool) -> Option<SurfaceTokens> {
        None
    }
    fn alert(&self, t: &Theme, tone: Tone, banner: bool) -> Option<SurfaceTokens> {
        None
    }
    fn snackbar(&self, t: &Theme) -> Option<(SurfaceTokens, Color)> {
        None
    }
    fn dialog(&self, t: &Theme) -> Option<SurfaceTokens> {
        None
    }
    fn sheet(&self, t: &Theme) -> Option<SurfaceTokens> {
        None
    }
    fn popup(&self, t: &Theme) -> Option<SurfaceTokens> {
        None
    }
    fn scrim(&self, t: &Theme) -> Option<Color> {
        None
    }
    fn app_bar(&self, t: &Theme, kind: AppBarKind, scrolled: bool) -> Option<AppBarTokens> {
        None
    }
    fn step(&self, t: &Theme, state: StepState) -> Option<StepTokens> {
        None
    }
    fn tab(&self, t: &Theme, active: bool, hovered: bool) -> Option<TabTokens> {
        None
    }
    fn toggle(&self, t: &Theme, on: bool, state: ControlState) -> Option<ToggleTokens> {
        None
    }
    fn choice_toggle(
        &self,
        t: &Theme,
        kind: ToggleKind,
        on: bool,
        state: ControlState,
    ) -> Option<ToggleTokens> {
        None
    }
    fn badge(&self, t: &Theme, tone: Tone) -> Option<SurfaceTokens> {
        None
    }
    fn progress(&self, t: &Theme, tone: Option<Tone>) -> Option<ProgressTokens> {
        None
    }
    fn list_item(&self, t: &Theme, selected: bool, hovered: bool) -> Option<ListItemTokens> {
        None
    }
    fn sizes(&self, t: &Theme) -> Option<Sizes> {
        None
    }
    fn navigation(&self, t: &Theme) -> Option<NavigationTokens> {
        None
    }
    fn navigation_item(&self, t: &Theme, selected: bool, hovered: bool) -> Option<ListItemTokens> {
        None
    }
    fn divider(&self, t: &Theme) -> Option<Color> {
        None
    }
    fn button(&self, t: &Theme, v: ButtonVariant, s: ControlState) -> Option<ButtonTokens> {
        None
    }
    fn elevated_button(&self, t: &Theme, s: ControlState) -> Option<ButtonTokens> {
        None
    }
    fn fab(&self, t: &Theme, v: ButtonVariant, s: ControlState) -> Option<ButtonTokens> {
        None
    }
    /// Adjust the icon-button tokens `base` (already resolved as a plain button).
    fn icon_button(
        &self,
        t: &Theme,
        v: ButtonVariant,
        s: ControlState,
        selected: bool,
        base: ButtonTokens,
    ) -> Option<ButtonTokens> {
        None
    }
    fn input(&self, t: &Theme, focused: bool) -> Option<InputTokens> {
        None
    }
    /// Colour of the hard offset shadow (default: the pressed primary).
    fn hard_shadow_color(&self, t: &Theme) -> Option<Color> {
        None
    }
    /// Soft shadow layers `(offset_y, blur, spread, alpha)` for an elevation `level` (1-5), when
    /// the design has its own shadow tokens instead of the two Material layers. The alpha is
    /// absolute; the colour is the theme's shadow colour.
    fn elevation_layers(&self, t: &Theme, level: u8) -> Option<Vec<(f32, f32, f32, f32)>> {
        None
    }
    /// The page grid at a viewport width (default: the theme's own spacing, 12 columns, or 4 on
    /// a narrow viewport).
    fn page_grid(&self, t: &Theme, viewport: f32) -> Option<PageGrid> {
        None
    }
    /// Type sizes `[xs, sm, md, lg, xl, xxl]` and spacing `[xs, sm, md, lg, xl]` at a viewport
    /// width, for designs with fluid scales (default: the static theme values).
    fn fluid(&self, t: &Theme, viewport: f32) -> Option<Fluid> {
        None
    }
    /// Stops `(alpha, position 0..=1)` of the dark fade laid over media so text on it stays
    /// legible (default: a two-stop linear fade). Position 0 is the edge the text sits on, 1 the far edge.
    fn fade_stops(&self, t: &Theme) -> Option<Vec<(f32, f32)>> {
        None
    }
    /// Keyboard focus indicator `(colour, thickness px)`.
    fn focus_ring(&self, t: &Theme) -> Option<(Color, f32)> {
        None
    }
    fn list_row_height(&self, t: &Theme, nested: bool) -> Option<f32> {
        None
    }
    fn list_lines_height(&self, t: &Theme, lines: u8) -> Option<f32> {
        None
    }
    fn menu_row_height(&self, t: &Theme) -> Option<f32> {
        None
    }
    /// Tooltip `(container, label)`.
    fn tooltip(&self, t: &Theme, rich: bool) -> Option<(Color, Color)> {
        None
    }
    /// Horizontal padding of a button label.
    fn control_metrics(&self, t: &Theme, size: ControlSize) -> Option<SizeMetrics> {
        None
    }
    fn type_style(&self, t: &Theme, role: TypeRole) -> Option<TypeStyle> {
        None
    }
    fn button_padding(&self, t: &Theme) -> Option<f32> {
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
