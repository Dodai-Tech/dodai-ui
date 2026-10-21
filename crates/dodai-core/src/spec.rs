//! **Design-system spec**: how one design system resolves component tokens and geometry.
//!
//! Core only knows the *vocabulary* (token structs, [`Theme`] methods). A design system crate
//! (`dodai-material`, …) implements [`DesignSpec`] and attaches it to its themes with
//! [`Theme::with_spec`]. Every hook returns `None` by default, which falls back to the generic
//! baseline in [`Theme`]'s resolvers, so a spec only lists what it does differently.

use crate::{
    AppBarKind, AppBarTokens, ButtonTokens, ButtonVariant, CardVariant, Color, ControlState,
    InputTokens, ListItemTokens, ProgressTokens, StepState, StepTokens, SurfaceTokens, TabTokens,
    Theme, ToggleKind, ToggleTokens, Tone,
};

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
    /// Prefers the symbol-font icon set over the stroke-shape icons.
    pub symbol_icons: bool,
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
    fn progress(&self, t: &Theme, tone: Option<Tone>) -> Option<ProgressTokens> {
        None
    }
    fn list_item(&self, t: &Theme, selected: bool, hovered: bool) -> Option<ListItemTokens> {
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
    /// Keyboard focus indicator `(colour, thickness px)`.
    fn focus_ring(&self, t: &Theme) -> Option<(Color, f32)> {
        None
    }
    fn list_row_height(&self, t: &Theme, nested: bool) -> Option<f32> {
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
