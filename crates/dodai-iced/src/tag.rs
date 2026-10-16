//! Tag / badge.

use iced::{widget::text, Element, Padding};
use dodai_core::Tone;

use crate::{color, BoxStyle, IcedTheme};

impl IcedTheme {
    /// [`BoxStyle`] for a tag: `solid` fills with the tone, otherwise outlined.
    pub fn tag_box(&self, tone: Tone, solid: bool) -> BoxStyle {
        let t = self.theme.tag(tone, solid);
        BoxStyle {
            fill: t.background,
            stroke: t.border,
            glow: false,
            hard_shadow: false,
            padding: Padding::from([self.theme.spacing.xs, self.theme.spacing.sm]),
        }
    }

    /// Tag / badge shaped per theme, in a status [`Tone`].
    pub fn tag<'a, Message: 'a>(
        &self,
        label: &str,
        tone: Tone,
        solid: bool,
    ) -> Element<'a, Message> {
        let t = self.theme.tag(tone, solid);
        let content = text(self.label(label))
            .size(self.theme.typography.size_xs)
            .color(color(t.foreground));
        self.shaped_box(content, self.tag_box(tone, solid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn solid_vs_outline() {
        let lt = IcedTheme::new(cyberpunk());
        let solid = lt.tag_box(Tone::Success, true);
        assert_eq!(solid.fill, Some(lt.theme().palette.success));
        assert!(solid.stroke.is_none());
        let out = lt.tag_box(Tone::Success, false);
        assert_eq!(out.stroke, Some(lt.theme().palette.success));
        assert!(out.fill.unwrap().a < 255);
    }

    #[test]
    fn tone_colours_per_theme() {
        let a = IcedTheme::new(cyberpunk()).tag_box(Tone::Danger, true);
        let b = IcedTheme::new(rounded_light()).tag_box(Tone::Danger, true);
        assert_ne!(a.fill, b.fill);
        assert_eq!(b.padding.left, 8.0);
        // box style radius follows shape
        assert_eq!(
            IcedTheme::new(rounded_light()).box_style(&b).border.radius,
            8.0.into()
        );
    }
}
