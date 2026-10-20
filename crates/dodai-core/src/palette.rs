//! Semantic colour tokens. Adapters never see raw brand colours, only roles.

use crate::Color;

/// Light vs dark base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Dark,
    Light,
}

/// Semantic status colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tone {
    Info,
    Success,
    Warning,
    Danger,
}

/// Every colour role a UI needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    // Surfaces (darkest → lightest)
    pub background: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub overlay: Color,
    // Lines
    pub border: Color,
    pub border_strong: Color,
    // Text
    pub text: Color,
    pub text_muted: Color,
    pub text_disabled: Color,
    // Brand
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_active: Color,
    pub on_primary: Color,
    pub secondary: Color,
    pub secondary_hover: Color,
    pub on_secondary: Color,
    pub accent: Color,
    // Status
    pub info: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    // Interaction
    pub selection: Color,
    pub focus_ring: Color,
    pub link: Color,
    /// Categorical series colours (charts, graph lanes, tags).
    pub series: [Color; 8],
    /// The extended (Material 3 style) roles: tonal surface containers, `on-*` colours,
    /// outlines, inverse and scrim. Derived from the roles above unless a design system
    /// sets its own ([`Roles::derive`]).
    pub roles: Roles,
}

macro_rules! roles {
    ($($field:ident => $name:literal),+ $(,)?) => {
        /// Extended colour roles, named after Material 3's. Every theme has all of them:
        /// [`Roles::derive`] fills them from the base palette, and a design system may
        /// override with its own values.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct Roles { $(pub $field: Color),+ }

        impl Roles {
            /// Placeholder (all black) for building a [`Palette`] by hand; follow with
            /// [`Palette::with_derived_roles`].
            pub fn placeholder() -> Self {
                Self { $($field: Color::hex(0)),+ }
            }

            /// `(kebab-case token name, colour)` for every role, in a stable order
            /// (the exporters use this, so a new role reaches CSS and DTCG automatically).
            pub fn list(&self) -> Vec<(&'static str, Color)> {
                vec![$(($name, self.$field)),+]
            }
        }
    };
}

roles! {
    surface_container_lowest => "surface-container-lowest",
    surface_container_low => "surface-container-low",
    surface_container => "surface-container",
    surface_container_high => "surface-container-high",
    surface_container_highest => "surface-container-highest",
    on_surface => "on-surface",
    on_surface_variant => "on-surface-variant",
    outline => "outline",
    outline_variant => "outline-variant",
    primary_container => "primary-container",
    on_primary_container => "on-primary-container",
    secondary_container => "secondary-container",
    on_secondary_container => "on-secondary-container",
    tertiary => "tertiary",
    on_tertiary => "on-tertiary",
    tertiary_container => "tertiary-container",
    on_tertiary_container => "on-tertiary-container",
    error_container => "error-container",
    on_error => "on-error",
    on_error_container => "on-error-container",
    inverse_surface => "inverse-surface",
    inverse_on_surface => "inverse-on-surface",
    inverse_primary => "inverse-primary",
    scrim => "scrim",
    shadow => "shadow",
}

/// Whichever of the theme's text / page colours reads better on `bg`.
fn readable_on(bg: Color, p: &Palette) -> Color {
    if p.text.contrast(bg) >= p.background.contrast(bg) {
        p.text
    } else {
        p.background
    }
}

impl Roles {
    /// Fill every role from a palette that has the base roles set (`roles` itself is
    /// ignored). `dark` picks which way tonal steps move.
    pub fn derive(p: &Palette, dark: bool) -> Self {
        let black = Color::hex(0x000000);
        let white = Color::hex(0xffffff);
        // Containers step from the page towards the text colour; "lowest" goes the other way.
        let lowest = p.background.mix(if dark { black } else { white }, 0.5);
        let tint = |base: Color| base.mix(p.background, 0.78);
        Self {
            surface_container_lowest: lowest,
            surface_container_low: p.surface,
            surface_container: p.surface.mix(p.surface_raised, 0.5),
            surface_container_high: p.surface_raised,
            surface_container_highest: p.surface_raised.mix(p.text, 0.06),
            on_surface: p.text,
            on_surface_variant: p.text_muted,
            outline: p.border_strong,
            outline_variant: p.border,
            primary_container: tint(p.primary),
            on_primary_container: p.text,
            secondary_container: tint(p.secondary),
            on_secondary_container: p.text,
            tertiary: p.accent,
            on_tertiary: readable_on(p.accent, p),
            tertiary_container: tint(p.accent),
            on_tertiary_container: p.text,
            error_container: tint(p.danger),
            on_error: readable_on(p.danger, p),
            on_error_container: p.text,
            inverse_surface: p.text,
            inverse_on_surface: p.background,
            inverse_primary: if dark {
                p.primary.darken(0.3)
            } else {
                p.primary.lighten(0.3)
            },
            scrim: black,
            shadow: black,
        }
    }
}

impl Palette {
    /// Colour for a status [`Tone`].
    pub fn tone(&self, tone: Tone) -> Color {
        match tone {
            Tone::Info => self.info,
            Tone::Success => self.success,
            Tone::Warning => self.warning,
            Tone::Danger => self.danger,
        }
    }

    /// Replace [`roles`](Self::roles) with ones derived from the base palette. Call after
    /// building a palette by hand.
    pub fn with_derived_roles(mut self, dark: bool) -> Self {
        self.roles = Roles::derive(&self, dark);
        self
    }

    /// Series colour, wrapping around.
    pub fn series_at(&self, i: usize) -> Color {
        self.series[i % self.series.len()]
    }
}
