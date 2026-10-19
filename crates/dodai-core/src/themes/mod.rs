//! Built-in themes.
//!
//! **Adding a theme:** create `themes/<name>.rs` returning a [`crate::Theme`] — the
//! easiest way is [`Spec`], which derives hover/active/selection roles from a
//! handful of colours and lets you set typography, spacing, shape (`Square`,
//! `Rounded`, `Chamfered` + which corners) and effects (glow, scanlines).
//! Then `pub mod <name>;` and push it in [`all`]. Every adapter and every shared
//! test picks it up automatically; adapters never branch on theme ids.

pub mod cyberpunk;
pub mod material;

use crate::Theme;

/// Compact description of a theme; [`Spec::build`] derives the full token set.
/// Use this to add new themes without filling every palette role by hand.
pub struct Spec {
    pub meta: crate::ThemeMeta,
    pub mode: crate::Mode,
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

impl Spec {
    pub fn build(self) -> crate::Theme {
        use crate::{Color, Effects, Palette};
        let h = Color::hex;
        let primary = h(self.primary);
        let secondary = h(self.secondary);
        let bg = h(self.background);
        let dark = self.mode == crate::Mode::Dark;
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
        };
        crate::Theme {
            shape_system: None,
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
            },
        }
    }
}

/// Every built-in theme, in switcher order.
pub fn all() -> Vec<Theme> {
    let mut v = Vec::new();
    v.extend(cyberpunk::variants());
    v.extend(material::variants());
    v
}

/// Look up a built-in theme by id.
pub fn by_id(id: &str) -> Option<Theme> {
    all().into_iter().find(|t| t.id() == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_unique() {
        let ids: HashSet<_> = all().iter().map(|t| t.id()).collect();
        assert_eq!(ids.len(), all().len());
    }

    #[test]
    fn text_is_readable_everywhere() {
        for t in all() {
            let p = &t.palette;
            assert!(p.text.contrast(p.background) >= 7.0, "{}: text/bg", t.id());
            assert!(
                p.text.contrast(p.surface) >= 7.0,
                "{}: text/surface",
                t.id()
            );
            assert!(
                p.on_primary.contrast(p.primary) >= 4.5,
                "{}: on_primary",
                t.id()
            );
            assert!(
                p.on_secondary.contrast(p.secondary) >= 4.5,
                "{}: on_secondary",
                t.id()
            );
            assert!(
                p.text_muted.contrast(p.background) >= 4.5,
                "{}: muted",
                t.id()
            );
        }
    }

    /// A throw-away rounded light theme proves `Spec` can express a theme that
    /// differs from Cyberpunk in shape, type and effects, not just colour.
    fn sample_light() -> Theme {
        use crate::{Corners, Mode, Shape, Spacing, ThemeMeta, Typography};
        Spec {
            meta: ThemeMeta {
                id: "test-light",
                name: "Test Light",
                family: "test",
                description: "",
                mode: Mode::Light,
            },
            mode: Mode::Light,
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
                border_width: 1.0,
                control_height: 40.0,
            },
            glow: 0.0,
            glow_blur: 0.0,
            scanlines: false,
            hard_shadow: 2.0,
        }
        .build()
    }

    #[test]
    fn spec_builds_a_non_cyberpunk_theme() {
        let t = sample_light();
        assert!(!t.is_dark());
        assert_eq!(t.shape.kind(), crate::ShapeKind::Rounded);
        assert!(!t.effects.glow_enabled());
        assert!(t.palette.text.contrast(t.palette.background) >= 7.0);
        assert_ne!(t.shape.kind(), cyberpunk::night_city().shape.kind());
    }

    #[test]
    fn component_tokens_resolve_for_every_theme() {
        use crate::{ControlState, Tone};
        for t in all() {
            assert!(
                t.tag(Tone::Info, true).foreground.contrast(t.palette.info) >= 3.0,
                "{}",
                t.id()
            );
            assert!(t.tab(true, false).indicator.is_some());
            assert_ne!(
                t.toggle(true, ControlState::Idle).track,
                t.toggle(false, ControlState::Idle).track
            );
            assert_eq!(
                t.list_item(true, false).marker.is_some(),
                t.shape.kind() != crate::ShapeKind::Rounded,
                "{}: accent bar on chamfered themes only",
                t.id()
            );
        }
    }

    #[test]
    fn outline_follows_shape() {
        for t in all().into_iter().chain([sample_light()]) {
            let o = t.shape.outline(100.0, 50.0);
            match t.shape.kind() {
                crate::ShapeKind::Chamfered => assert!(o.len() > 4, "{}", t.id()),
                _ => assert_eq!(o.len(), 4, "{}", t.id()),
            }
        }
    }

    #[test]
    fn cyberpunk_has_light_and_dark() {
        use crate::Mode;
        let fam: Vec<_> = all()
            .into_iter()
            .filter(|t| t.meta.family == "cyberpunk")
            .collect();
        assert_eq!(fam.iter().filter(|t| t.meta.mode == Mode::Dark).count(), 3);
        let light: Vec<_> = fam.iter().filter(|t| t.meta.mode == Mode::Light).collect();
        assert_eq!(light.len(), 3);
        for t in light {
            let p = &t.palette;
            assert!(p.link.contrast(p.background) >= 4.5, "{}: link", t.id());
            assert!(p.background.luminance() > 0.5, "{}: paper", t.id());
            assert_eq!(t.shape.kind(), cyberpunk::night_city().shape.kind());
        }
    }

    #[test]
    fn lookup() {
        assert!(by_id("cyberpunk").is_some());
        assert!(by_id("missing").is_none());
    }
}
