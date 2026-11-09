//! Pure token → style mapping (no ECS), shared by the systems and the tests.

use dodai_core::{Color as Rgba, Theme};

use crate::widgets::{Ink, TextRole, ToggleKind};

/// Resolved text style. `color == None` means "inherit the owner's foreground".
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    /// Font size px.
    pub size: f32,
    /// Fixed colour, or inherited.
    pub color: Option<Rgba>,
    /// Display UPPERCASE.
    pub upper: bool,
}

/// Map a role to a style from the theme's typography tokens.
pub fn text_style(theme: &Theme, role: TextRole) -> TextStyle {
    let ty = &theme.typography;
    let p = &theme.palette;
    let up = ty.uppercase_labels;
    let (size, color, upper) = match role {
        TextRole::Heading => (ty.size_xl, Some(p.text), up),
        TextRole::Subheading => (ty.size_lg, Some(p.text), up),
        TextRole::Body => (ty.size_md, Some(p.text), false),
        TextRole::Label => (ty.size_sm, Some(p.text), up),
        TextRole::Muted => (ty.size_sm, Some(p.text_muted), false),
        TextRole::Mono => (ty.size_sm, Some(p.text_muted), false),
        TextRole::Caption => (ty.size_xs, Some(p.text_muted), false),
        TextRole::Control => (ty.size_sm, None, up),
        TextRole::ControlSmall => (ty.size_xs, None, up),
        TextRole::Content => (ty.size_md, None, false),
        TextRole::Type(r) => (theme.type_style(r).size, None, false),
        TextRole::Styled {
            size, ink, upper, ..
        } => {
            let c = match ink {
                Ink::Text => p.text,
                Ink::Muted => p.text_muted,
                Ink::Disabled => p.text_disabled,
                Ink::Primary => p.primary,
                Ink::Link => p.link,
                Ink::Title => theme.panel(false).title,
                Ink::Tone(t) => match t {
                    dodai_core::Tone::Info => p.info,
                    dodai_core::Tone::Success => p.success,
                    dodai_core::Tone::Warning => p.warning,
                    dodai_core::Tone::Danger => p.danger,
                },
            };
            (size, Some(c), upper && up)
        }
    };
    TextStyle { size, color, upper }
}

/// The font stacks of a theme as Bevy font sources. Empty without the `system-fonts` feature
/// (the bundled face is used).
pub struct FontSources {
    #[cfg(feature = "system-fonts")]
    display: bevy::text::FontSource,
    #[cfg(feature = "system-fonts")]
    body: bevy::text::FontSource,
    #[cfg(feature = "system-fonts")]
    mono: bevy::text::FontSource,
}

/// A CSS family list with a generic fallback appended.
pub fn font_stack(stack: &str, generic: &str) -> String {
    format!("{stack}, {generic}")
}

impl FontSources {
    /// Build the three sources from the theme's typography tokens.
    pub fn of(theme: &Theme) -> Self {
        #[cfg(feature = "system-fonts")]
        {
            let ty = &theme.typography;
            use bevy::text::FontSource;
            Self {
                display: FontSource::families(font_stack(ty.display, "sans-serif")),
                body: FontSource::families(font_stack(ty.body, "sans-serif")),
                mono: FontSource::families(font_stack(ty.mono, "monospace")),
            }
        }
        #[cfg(not(feature = "system-fonts"))]
        {
            let _ = theme;
            Self {}
        }
    }

    /// The source for a text role, if fonts are resolved from the system.
    pub fn for_role(&self, role: TextRole) -> Option<&bevy::text::FontSource> {
        #[cfg(feature = "system-fonts")]
        {
            Some(match role {
                TextRole::Heading | TextRole::Subheading => &self.display,
                TextRole::Mono => &self.mono,
                TextRole::Type(r) if r.is_heading() => &self.display,
                TextRole::Styled { family, .. } => match family {
                    crate::widgets::Family::Display => &self.display,
                    crate::widgets::Family::Body => &self.body,
                    crate::widgets::Family::Mono => &self.mono,
                },
                _ => &self.body,
            })
        }
        #[cfg(not(feature = "system-fonts"))]
        {
            let _ = role;
            None
        }
    }
}

/// The string to display for `raw` under this style.
pub fn display_text(raw: &str, style: &TextStyle) -> String {
    if style.upper {
        raw.to_uppercase()
    } else {
        raw.to_owned()
    }
}

/// Outer size `(w, h)` of a toggle control, from shape tokens.
pub fn toggle_size(theme: &Theme, kind: ToggleKind) -> (f32, f32) {
    let h = (theme.shape.control_height * 0.5).max(16.0);
    match kind {
        ToggleKind::Switch => (h * 2.0, h),
        _ => (h, h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styled_text_takes_its_own_size_and_palette_colour() {
        let t = dodai_core::themes::all().remove(0);
        let s = text_style(
            &t,
            TextRole::Styled {
                size: 11.0,
                family: crate::widgets::Family::Mono,
                ink: Ink::Muted,
                upper: false,
            },
        );
        assert_eq!(
            (s.size, s.color, s.upper),
            (11.0, Some(t.palette.text_muted), false)
        );
    }
    use crate::testing::{rounded_light, square_theme};
    use dodai_core::themes;

    #[test]
    fn uppercase_follows_token_not_id() {
        let mut t = rounded_light();
        let s = text_style(&t, TextRole::Control);
        assert!(!s.upper);
        assert_eq!(display_text("go", &s), "go");
        t.typography.uppercase_labels = true;
        let s = text_style(&t, TextRole::Control);
        assert_eq!(display_text("go", &s), "GO");
        // body / content stay raw
        assert!(!text_style(&t, TextRole::Body).upper);
        assert!(!text_style(&t, TextRole::Content).upper);
    }

    #[test]
    fn font_stack_appends_a_generic_family() {
        assert_eq!(
            font_stack("Inter, Menlo", "sans-serif"),
            "Inter, Menlo, sans-serif"
        );
    }

    #[cfg(feature = "system-fonts")]
    #[test]
    fn headings_use_the_display_stack_and_code_the_mono_stack() {
        let f = FontSources::of(&rounded_light());
        assert_eq!(f.for_role(TextRole::Heading), Some(&f.display));
        assert_eq!(f.for_role(TextRole::Mono), Some(&f.mono));
        assert_eq!(f.for_role(TextRole::Body), Some(&f.body));
        assert_eq!(
            f.for_role(TextRole::Type(dodai_core::TypeRole::DisplayLarge)),
            Some(&f.display)
        );
        assert_eq!(
            f.for_role(TextRole::Type(dodai_core::TypeRole::LabelSmall)),
            Some(&f.body)
        );
    }

    #[test]
    fn sizes_come_from_typography() {
        for t in themes::all()
            .into_iter()
            .chain([rounded_light(), square_theme()])
        {
            assert_eq!(text_style(&t, TextRole::Heading).size, t.typography.size_xl);
            assert_eq!(text_style(&t, TextRole::Caption).size, t.typography.size_xs);
            assert_eq!(
                text_style(&t, TextRole::Muted).color,
                Some(t.palette.text_muted)
            );
            assert_eq!(text_style(&t, TextRole::Control).color, None);
        }
    }

    #[test]
    fn toggle_size_scales_with_control_height() {
        let (w, h) = toggle_size(&rounded_light(), ToggleKind::Switch);
        assert_eq!((w, h), (40.0, 20.0));
        let (w, h) = toggle_size(&square_theme(), ToggleKind::Checkbox);
        assert_eq!((w, h), (16.0, 16.0));
    }
}
