//! [`Tag`] / badge: solid or outlined per [`Tone`].

use crate::paint::{shaped, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::{Theme, Tone};
use gpui::{px, App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window};

/// Paint style + text colour of a tag (pure).
pub fn tag_style(theme: &Theme, tone: Tone, solid: bool) -> (ShapeStyle, gpui::Hsla) {
    let k = theme.tag(tone, solid);
    (
        ShapeStyle::from_tokens(theme, k.background, k.border, false, false),
        hsla(k.foreground),
    )
}

/// Small status badge.
#[derive(IntoElement)]
pub struct Tag {
    text: SharedString,
    tone: Tone,
    solid: bool,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Tag);

impl Tag {
    pub fn new(text: impl Into<SharedString>, tone: Tone) -> Self {
        Self {
            corners: None,
            text: text.into(),
            tone,
            solid: false,
        }
    }
    /// Solid fill (default is outlined with a tinted background).
    pub fn solid(mut self, solid: bool) -> Self {
        self.solid = solid;
        self
    }
}

impl Tag {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let (style, fg) = tag_style(&t, self.tone, self.solid);
        shaped(&t.shape, style)
            .px(px(t.spacing.sm))
            .py(px(t.spacing.xs / 2.0))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_xs))
            .text_color(fg)
            .child(format_label(&t.typography, &self.text))
    }
}

impl RenderOnce for Tag {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "Tag", motion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::square;

    #[test]
    fn solid_vs_outline() {
        let t = square();
        let (s, fg) = tag_style(&t, Tone::Success, true);
        assert_eq!(s.fill, Some(hsla(t.palette.success)));
        assert!(s.stroke.is_none());
        assert_eq!(fg, hsla(t.on(t.palette.success)));
        let (o, fg) = tag_style(&t, Tone::Danger, false);
        assert_eq!(o.stroke, Some(hsla(t.palette.danger)));
        assert_eq!(fg, hsla(t.palette.danger));
    }
}
