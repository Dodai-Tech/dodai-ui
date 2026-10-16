//! Built-in themes. Add new families as sibling modules and list them in [`all`].

pub mod arctic;
pub mod cyberpunk;
pub mod daylight;
pub mod phosphor;
pub mod synthwave;

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
    v.push(synthwave::synthwave());
    v.push(arctic::arctic());
    v.push(phosphor::phosphor());
    v.push(daylight::daylight());
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

    #[test]
    fn themes_differ_in_more_than_colour() {
        use std::collections::HashSet;
        let kinds: HashSet<_> = all()
            .iter()
            .map(|t| format!("{:?}", t.shape.kind()))
            .collect();
        assert!(
            kinds.len() >= 3,
            "expected square/rounded/chamfered, got {kinds:?}"
        );
        assert!(all().iter().any(|t| !t.is_dark()), "need a light theme");
        assert!(all().iter().any(|t| t.effects.scanlines));
        assert!(all().iter().any(|t| !t.effects.glow_enabled()));
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
            assert!(t.list_item(true, false).marker.is_some());
        }
    }

    #[test]
    fn outline_follows_shape() {
        for t in all() {
            let o = t.shape.outline(100.0, 50.0);
            match t.shape.kind() {
                crate::ShapeKind::Chamfered => assert!(o.len() > 4, "{}", t.id()),
                _ => assert_eq!(o.len(), 4, "{}", t.id()),
            }
        }
    }

    #[test]
    fn lookup() {
        assert!(by_id("cyberpunk").is_some());
        assert!(by_id("missing").is_none());
    }
}
