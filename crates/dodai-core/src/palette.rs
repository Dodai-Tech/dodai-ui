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
    surface_dim => "surface-dim",
    surface_bright => "surface-bright",
    primary_fixed => "primary-fixed",
    primary_fixed_dim => "primary-fixed-dim",
    on_primary_fixed => "on-primary-fixed",
    on_primary_fixed_variant => "on-primary-fixed-variant",
    secondary_fixed => "secondary-fixed",
    secondary_fixed_dim => "secondary-fixed-dim",
    on_secondary_fixed => "on-secondary-fixed",
    on_secondary_fixed_variant => "on-secondary-fixed-variant",
    tertiary_fixed => "tertiary-fixed",
    tertiary_fixed_dim => "tertiary-fixed-dim",
    on_tertiary_fixed => "on-tertiary-fixed",
    on_tertiary_fixed_variant => "on-tertiary-fixed-variant",
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
            // Dim / bright surfaces: the page, pushed away from / towards the text colour.
            surface_dim: if dark {
                p.background
            } else {
                p.background.mix(p.text, 0.1)
            },
            surface_bright: if dark {
                p.background.mix(p.text, 0.12)
            } else {
                p.background
            },
            // Fixed accents keep their tone in light and dark.
            primary_fixed: p.primary.mix(white, 0.75),
            primary_fixed_dim: p.primary.mix(white, 0.55),
            on_primary_fixed: p.primary.mix(black, 0.85),
            on_primary_fixed_variant: p.primary.mix(black, 0.65),
            secondary_fixed: p.secondary.mix(white, 0.75),
            secondary_fixed_dim: p.secondary.mix(white, 0.55),
            on_secondary_fixed: p.secondary.mix(black, 0.85),
            on_secondary_fixed_variant: p.secondary.mix(black, 0.65),
            tertiary_fixed: p.accent.mix(white, 0.75),
            tertiary_fixed_dim: p.accent.mix(white, 0.55),
            on_tertiary_fixed: p.accent.mix(black, 0.85),
            on_tertiary_fixed_variant: p.accent.mix(black, 0.65),
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

    /// `(shadow, scanline, highlight)` effect colours that suit this palette: the shadow and
    /// scanlines use the `shadow` / `scrim` roles (lighter on light themes), the shimmer
    /// highlight is the colour that reads on a primary fill.
    pub fn effect_colours(&self, dark: bool) -> (Color, Color, Color) {
        (
            self.roles.shadow.with_alpha(if dark { 0.16 } else { 0.12 }),
            self.roles.scrim.with_alpha(if dark { 0.18 } else { 0.07 }),
            self.on_primary.with_alpha(0.4),
        )
    }

    /// Replace [`roles`](Self::roles) with ones derived from the base palette. Call after
    /// building a palette by hand.
    pub fn with_derived_roles(mut self, dark: bool) -> Self {
        self.roles = Roles::derive(&self, dark);
        self
    }

    /// This palette with the series colours turned into a text-to-background ramp, so charts
    /// read by lightness alone (print, colour-blind, or a calm look).
    #[must_use]
    pub fn monochrome(&self) -> Self {
        let n = self.series.len();
        let mut p = self.clone();
        p.series = std::array::from_fn(|i| {
            self.text
                .mix(self.background, 0.7 * i as f32 / (n - 1) as f32)
        });
        p
    }

    /// Series colour, wrapping around.
    pub fn series_at(&self, i: usize) -> Color {
        self.series[i % self.series.len()]
    }
}

#[cfg(test)]
mod mono_tests {
    use crate::Theme;

    #[test]
    fn monochrome_ramps_from_the_text_colour_towards_the_background() {
        let p = crate::Palette::monochrome;
        let t = Theme::default();
        let m = p(&t.palette);
        assert_eq!(m.series[0], t.palette.text);
        assert_ne!(m.series[7], t.palette.text);
        assert_ne!(m.series[1], m.series[2]);
        assert_eq!(m.text, t.palette.text);
    }
}
