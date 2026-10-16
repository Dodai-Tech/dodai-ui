//! Text helpers honouring `typography` sizes and `uppercase_labels`.

use iced::{widget::text, Theme as IcedBaseTheme};
use legendary_core::Tone;

use crate::{color, IcedTheme};

type Text<'a> = iced::widget::Text<'a, IcedBaseTheme>;

impl IcedTheme {
    /// Apply the theme's label casing (UPPERCASE when `uppercase_labels`).
    pub fn label(&self, s: &str) -> String {
        if self.theme.typography.uppercase_labels {
            s.to_uppercase()
        } else {
            s.to_string()
        }
    }

    /// Coloured status text style.
    pub fn tone(&self, tone: Tone) -> impl Fn(&IcedBaseTheme) -> text::Style + 'static {
        let c = color(self.theme.palette.tone(tone));
        move |_| text::Style { color: Some(c) }
    }

    /// Muted secondary text style.
    pub fn muted(&self) -> impl Fn(&IcedBaseTheme) -> text::Style + 'static {
        let c = color(self.theme.palette.text_muted);
        move |_| text::Style { color: Some(c) }
    }

    /// Brand-coloured heading text style.
    pub fn heading(&self) -> impl Fn(&IcedBaseTheme) -> text::Style + 'static {
        let c = color(self.theme.palette.primary);
        move |_| text::Style { color: Some(c) }
    }

    /// Heading: `size_xl`, brand colour, label casing.
    pub fn heading_text<'a>(&self, s: &str) -> Text<'a> {
        text(self.label(s))
            .size(self.theme.typography.size_xl)
            .font(self.font())
            .style(self.heading())
    }

    /// Label: `size_sm`, label casing, regular text colour.
    pub fn label_text<'a>(&self, s: &str) -> Text<'a> {
        let c = color(self.theme.palette.text);
        text(self.label(s))
            .size(self.theme.typography.size_sm)
            .style(move |_| text::Style { color: Some(c) })
    }

    /// Body: `size_md`, regular text colour, no casing change.
    pub fn body_text<'a>(&self, s: &str) -> Text<'a> {
        let c = color(self.theme.palette.text);
        text(s.to_string())
            .size(self.theme.typography.size_md)
            .style(move |_| text::Style { color: Some(c) })
    }

    /// Muted: `size_sm`, secondary colour, no casing change.
    pub fn muted_text<'a>(&self, s: &str) -> Text<'a> {
        text(s.to_string())
            .size(self.theme.typography.size_sm)
            .style(self.muted())
    }

    /// Monospace: `size_sm`, mono font.
    pub fn mono_text<'a>(&self, s: &str) -> Text<'a> {
        text(s.to_string())
            .size(self.theme.typography.size_sm)
            .font(self.font_mono())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn label_casing_from_token() {
        assert_eq!(IcedTheme::new(cyberpunk()).label("ok"), "OK");
        assert_eq!(IcedTheme::new(rounded_light()).label("ok"), "ok");
    }

    #[test]
    fn text_styles_use_palette() {
        let lt = IcedTheme::new(rounded_light());
        let t = lt.iced_theme();
        assert_eq!(
            lt.muted()(&t).color,
            Some(color(lt.theme().palette.text_muted))
        );
        assert_eq!(
            lt.heading()(&t).color,
            Some(color(lt.theme().palette.primary))
        );
        assert_eq!(
            lt.tone(Tone::Danger)(&t).color,
            Some(color(lt.theme().palette.danger))
        );
        // builders compile with typography sizes
        let _ = (
            lt.heading_text("a"),
            lt.label_text("a"),
            lt.body_text("a"),
            lt.muted_text("a"),
            lt.mono_text("a"),
        );
    }
}
