//! Typography helpers: [`Heading`], [`Label`], [`Muted`], [`Mono`].
//!
//! All honour `typography.uppercase_labels` (headings and labels only) and the
//! size scale. gpui 0.2.2 has no letter-spacing, so `label_tracking` is not applied.

use crate::{hsla, theme_of};
use dodai_ui_core::Typography;
use gpui::{div, px, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window};

/// Style `el` with a type-scale role: family, size, line height and weight.
pub fn typeset<E: Styled>(el: E, t: &dodai_ui_core::Theme, role: dodai_ui_core::TypeRole) -> E {
    let s = t.type_style(role);
    let fam = if role.is_heading() {
        t.typography.display
    } else {
        t.typography.body
    };
    el.font_family(first_family(fam))
        .text_size(px(s.size))
        .line_height(px(s.line_height))
        .font_weight(gpui::FontWeight(s.weight as f32))
}

/// Text in a type-scale role (Material 3: 15 roles; other designs derive them from their sizes).
#[derive(IntoElement)]
pub struct Text {
    text: SharedString,
    role: dodai_ui_core::TypeRole,
    muted: bool,
}

impl Text {
    /// Create a new [`Text`].
    pub fn new(role: dodai_ui_core::TypeRole, text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            role,
            muted: false,
        }
    }
    /// Use the secondary text colour.
    pub fn muted(mut self, on: bool) -> Self {
        self.muted = on;
        self
    }
}

impl RenderOnce for Text {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let c = if self.muted {
            t.palette.text_muted
        } else {
            t.palette.text
        };
        typeset(div(), &t, self.role)
            .text_color(hsla(c))
            .child(self.text)
    }
}

/// Apply `uppercase_labels` to `s`.
pub fn format_label(t: &Typography, s: &str) -> SharedString {
    if t.uppercase_labels {
        s.to_uppercase().into()
    } else {
        s.to_string().into()
    }
}

/// First family of a CSS-like comma list (`"A, B, sans-serif"` → `"A"`).
/// gpui falls back to its default font if the family is not installed.
pub fn first_family(list: &str) -> SharedString {
    list.split(',')
        .next()
        .unwrap_or(list)
        .trim()
        .to_string()
        .into()
}

/// Heading size for a level (1 = largest … 4 = smallest).
pub fn heading_size(t: &Typography, level: u8) -> f32 {
    match level {
        0 | 1 => t.size_xxl,
        2 => t.size_xl,
        3 => t.size_lg,
        _ => t.size_md,
    }
}

/// Display-font heading in the primary text colour.
#[derive(IntoElement)]
pub struct Heading {
    text: SharedString,
    level: u8,
}

impl Heading {
    /// Level 2 heading.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            level: 2,
        }
    }
    /// Level 1 (largest) … 4 (smallest).
    pub fn level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }
}

impl RenderOnce for Heading {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::current(cx);
        crate::motion::entry(self.build(cx), "Heading", motion)
    }
}

impl Heading {
    fn build(self, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        div()
            .font_family(first_family(t.typography.display))
            .text_size(px(heading_size(&t.typography, self.level)))
            .text_color(hsla(t.palette.text))
            .child(format_label(&t.typography, &self.text))
    }
}

/// Small body-font label (uppercased when the theme says so).
#[derive(IntoElement)]
pub struct Label(SharedString);

impl Label {
    /// Create a new [`Label`].
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

impl RenderOnce for Label {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::current(cx);
        crate::motion::entry(self.build(cx), "Label", motion)
    }
}

impl Label {
    fn build(self, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        div()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_sm))
            .text_color(hsla(t.palette.text))
            .child(format_label(&t.typography, &self.0))
    }
}

/// Secondary body text in `text_muted`.
#[derive(IntoElement)]
pub struct Muted(SharedString);

impl Muted {
    /// Create a new [`Muted`].
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

impl RenderOnce for Muted {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::current(cx);
        crate::motion::entry(self.build(cx), "Muted", motion)
    }
}

impl Muted {
    fn build(self, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        div()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_sm))
            .text_color(hsla(t.palette.text_muted))
            .child(self.0)
    }
}

/// Monospace text (never uppercased).
#[derive(IntoElement)]
pub struct Mono(SharedString);

impl Mono {
    /// Create a new [`Mono`].
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self(text.into())
    }
}

impl RenderOnce for Mono {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::current(cx);
        crate::motion::entry(self.build(cx), "Mono", motion)
    }
}

impl Mono {
    fn build(self, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        div()
            .font_family(first_family(t.typography.mono))
            .text_size(px(t.typography.size_sm))
            .text_color(hsla(t.palette.text))
            .child(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{light_rounded, square};

    #[test]
    fn uppercase_follows_token() {
        assert_eq!(format_label(&square().typography, "Hack").as_ref(), "HACK");
        assert_eq!(
            format_label(&light_rounded().typography, "Hack").as_ref(),
            "Hack"
        );
    }

    #[test]
    fn family_and_sizes() {
        assert_eq!(
            first_family("Blender Pro, Rajdhani, sans-serif").as_ref(),
            "Blender Pro"
        );
        let t = light_rounded().typography;
        assert!(heading_size(&t, 1) > heading_size(&t, 2));
        assert!(heading_size(&t, 3) > heading_size(&t, 4));
    }
}
