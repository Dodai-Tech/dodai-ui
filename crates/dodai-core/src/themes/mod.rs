//! Theme building blocks and the registry of the enabled design systems
//! (features `cyberpunk`, `material`, `neobrutalism`, `dodai`); [`neutral`] is the fallback.
//!
//! **Adding a theme:** return a [`crate::Theme`] — the
//! easiest way is [`ThemeBuilder`], which derives hover/active/selection roles from a
//! handful of colours and lets you set typography, spacing, shape (`Square`,
//! `Rounded`, `Chamfered` + which corners) and effects (glow, scanlines).
//! Adapters never branch on theme ids.

use crate::Theme;

#[cfg(feature = "cyberpunk")]
pub use crate::cyberpunk;
#[cfg(feature = "dodai")]
pub use crate::dodai;
#[cfg(feature = "material")]
pub use crate::material;
#[cfg(feature = "neobrutalism")]
pub use crate::neobrutalism;

/// Compact description of a theme; [`ThemeBuilder::build`] derives the full token set.
/// Use this to add new themes without filling every palette role by hand.
pub struct ThemeBuilder {
    /// Id, name, family, description and light/dark mode.
    pub meta: crate::ThemeMeta,
    // surfaces / lines / text (hex)
    pub background: u32,
    pub surface: u32,
    pub raised: u32,
    pub border: u32,
    pub border_strong: u32,
    pub text: u32,
    pub muted: u32,
    pub disabled: u32,
    // brand
    pub primary: u32,
    pub primary_hover: u32,
    pub on_primary: u32,
    pub secondary: u32,
    pub on_secondary: u32,
    pub accent: u32,
    // status
    pub info: u32,
    pub success: u32,
    pub warning: u32,
    pub danger: u32,
    pub series: [u32; 8],
    pub typography: crate::Typography,
    pub spacing: crate::Spacing,
    pub shape: crate::Shape,
    pub glow: f32,
    pub glow_blur: f32,
    pub scanlines: bool,
    pub hard_shadow: f32,
}

impl ThemeBuilder {
    pub fn build(self) -> crate::Theme {
        use crate::{Color, Effects, Palette};
        let h = Color::hex;
        let primary = h(self.primary);
        let secondary = h(self.secondary);
        let bg = h(self.background);
        let dark = self.meta.mode == crate::Mode::Dark;
        let palette = Palette {
            background: bg,
            surface: h(self.surface),
            surface_raised: h(self.raised),
            overlay: bg.with_alpha(0.88),
            border: h(self.border),
            border_strong: h(self.border_strong),
            text: h(self.text),
            text_muted: h(self.muted),
            text_disabled: h(self.disabled),
            primary,
            primary_hover: h(self.primary_hover),
            primary_active: if dark {
                primary.darken(0.18)
            } else {
                primary.darken(0.12)
            },
            on_primary: h(self.on_primary),
            secondary,
            secondary_hover: if dark {
                secondary.lighten(0.2)
            } else {
                secondary.darken(0.1)
            },
            on_secondary: h(self.on_secondary),
            accent: h(self.accent),
            info: h(self.info),
            success: h(self.success),
            warning: h(self.warning),
            danger: h(self.danger),
            selection: primary.with_alpha(0.28),
            focus_ring: secondary,
            link: secondary,
            series: self.series.map(h),
            roles: crate::Roles::placeholder(),
        }
        .with_derived_roles(dark);
        let fx = palette.effect_colours(dark);
        crate::Theme {
            motion: crate::Motion::default(),
            shape_system: None,
            spec: Default::default(),
            surface_style: Default::default(),
            density: 1.0,
            meta: self.meta,
            palette,
            typography: self.typography,
            spacing: self.spacing,
            shape: self.shape,
            effects: Effects {
                glow: primary.with_alpha(self.glow),
                glow_blur: self.glow_blur,
                glow_intensity: 1.0,
                scanlines: self.scanlines,
                hard_shadow: self.hard_shadow,
                shadow: fx.0,
                scanline: fx.1,
                scanline_pitch: crate::Effects::SCANLINE_PITCH,
                highlight: fx.2,
            },
        }
    }
}

impl ThemeBuilder {
    /// The neutral light builder (the colours, type and spacing of [`neutral`]) under `meta`:
    /// a base for custom and test themes, e.g.
    /// `ThemeBuilder { glow: 1.0, ..ThemeBuilder::neutral(meta) }.build()`.
    pub fn neutral(meta: crate::ThemeMeta) -> Self {
        use crate::{Corners, Shape, Spacing, Typography};
        ThemeBuilder {
            meta,
            background: 0xf6f7f9,
            surface: 0xffffff,
            raised: 0xeceef2,
            border: 0xd5d9e0,
            border_strong: 0xaab1bd,
            text: 0x14171c,
            muted: 0x4a5260,
            disabled: 0x8a92a0,
            primary: 0x1f5fd6,
            primary_hover: 0x1a50b5,
            on_primary: 0xffffff,
            secondary: 0x0a7d6e,
            on_secondary: 0xffffff,
            accent: 0xb45309,
            info: 0x1f6fd6,
            success: 0x15803d,
            warning: 0xb45309,
            danger: 0xc81e3a,
            series: [
                0x1f5fd6, 0x0a7d6e, 0xb45309, 0xc81e3a, 0x7c3aed, 0x0e7490, 0xbe185d, 0x4a5260,
            ],
            typography: Typography {
                display: "Inter",
                body: "Inter",
                mono: "Menlo",
                size_xs: 11.0,
                size_sm: 13.0,
                size_md: 15.0,
                size_lg: 18.0,
                size_xl: 24.0,
                size_xxl: 36.0,
                uppercase_labels: false,
                label_tracking: 0.0,
            },
            spacing: Spacing {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 20.0,
                xl: 32.0,
            },
            shape: Shape {
                corners: Corners::NONE,
                radius: 8.0,
                chamfer: 0.0,
                scoop: false,
                border_width: 1.0,
                control_height: 40.0,
            },
            glow: 0.0,
            glow_blur: 0.0,
            scanlines: false,
            hard_shadow: 2.0,
        }
    }
}

/// Neutral light theme: the fallback for [`Theme::default`]. Rounded, mixed-case,
/// no glow. Real design systems live in their own crates.
pub fn neutral() -> Theme {
    ThemeBuilder::neutral(crate::ThemeMeta {
        id: "default",
        name: "Default",
        family: "default",
        description: "Neutral light fallback theme",
        mode: crate::Mode::Light,
    })
    .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_is_a_rounded_light_theme() {
        let t = neutral();
        assert!(!t.is_dark());
        assert_eq!(t.shape.kind(), crate::ShapeKind::Rounded);
        assert!(!t.effects.glow_enabled());
        assert!(t.palette.text.contrast(t.palette.background) >= 7.0);
        assert_eq!(Theme::default().id(), "default");
        assert_eq!(t.shape.outline(100.0, 50.0).len(), 4);
    }
}

/// A registered design system: its family id, display name and themes.
#[derive(Clone, Copy)]
pub struct DesignSystem {
    /// Family id shared by its themes (`Theme::meta.family`).
    pub family: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Builds its themes (light/dark/accent variants).
    pub themes: fn() -> Vec<crate::Theme>,
}

/// The enabled design systems, in switcher order. **The one place a new design system is
/// registered**: add a feature-gated entry here; the switcher, `all()`, `by_id()` and the
/// `dist` export pick it up.
pub fn systems() -> Vec<DesignSystem> {
    let all: Vec<Option<DesignSystem>> = Vec::from([
        #[cfg(feature = "dodai")]
        Some(DesignSystem {
            family: "dodai",
            name: "Dodai",
            themes: crate::dodai::variants,
        }),
        #[cfg(feature = "cyberpunk")]
        Some(DesignSystem {
            family: "cyberpunk",
            name: "Cyberpunk",
            themes: crate::cyberpunk::variants,
        }),
        #[cfg(feature = "material")]
        Some(DesignSystem {
            family: "material",
            name: "Material 3",
            themes: crate::material::variants,
        }),
        #[cfg(feature = "neobrutalism")]
        Some(DesignSystem {
            family: "neobrutalism",
            name: "Neo-Brutalism",
            themes: crate::neobrutalism::variants,
        }),
    ]);
    all.into_iter().flatten().collect()
}

/// Every theme of the enabled design systems, in switcher order.
/// Falls back to [`neutral`] if none is enabled.
pub fn all() -> Vec<crate::Theme> {
    let mut v: Vec<crate::Theme> = systems().iter().flat_map(|s| (s.themes)()).collect();
    if v.is_empty() {
        v.push(neutral());
    }
    v
}

/// Look up a built-in theme by id.
pub fn by_id(id: &str) -> Option<crate::Theme> {
    all().into_iter().find(|t| t.id() == id)
}
