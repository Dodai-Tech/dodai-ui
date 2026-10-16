//! Panel / card.

use iced::{
    widget::{column, container, text},
    Background, Element, Padding, Theme as IcedBaseTheme,
};

use crate::{color, BoxStyle, IcedTheme};

impl IcedTheme {
    /// [`BoxStyle`] of a panel (shared by [`IcedTheme::panel`] and
    /// [`IcedTheme::panel_view`]).
    pub fn panel_box(&self, focused: bool) -> BoxStyle {
        let t = self.theme.panel(focused);
        BoxStyle {
            fill: Some(t.background),
            stroke: Some(t.border),
            glow: focused,
            hard_shadow: true,
            padding: Padding::new(self.theme.spacing.lg),
        }
    }

    /// Native bordered panel container style (rounded / square). Border is
    /// brighter and glows when `focused`; draws `effects.hard_shadow` if > 0.
    pub fn panel(&self, focused: bool) -> impl Fn(&IcedBaseTheme) -> container::Style + 'static {
        let t = self.theme.panel(focused);
        let fg = color(self.theme.palette.text);
        let border = self.border(t.border);
        let shadow = self.shadow(focused, true);
        move |_| container::Style {
            background: Some(Background::Color(color(t.background))),
            text_color: Some(fg),
            border,
            shadow,
            snap: true,
        }
    }

    /// Card with an optional title, shaped per theme (chamfer / rounded /
    /// square), with border, glow when `focused`, and hard shadow.
    pub fn panel_view<'a, Message: 'a>(
        &self,
        title: Option<&str>,
        content: impl Into<Element<'a, Message>>,
        focused: bool,
    ) -> Element<'a, Message> {
        let tok = self.theme.panel(focused);
        let mut col = column![].spacing(self.theme.spacing.md);
        if let Some(t) = title {
            col = col.push(
                text(self.label(t))
                    .size(self.theme.typography.size_sm)
                    .color(color(tok.title)),
            );
        }
        col = col.push(content);
        let mut s = self.panel_box(focused);
        s.padding = Padding::new(self.theme.spacing.lg);
        self.shaped_box(col, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;
    use dodai_core::Theme;

    fn style(t: Theme, focused: bool) -> container::Style {
        let lt = IcedTheme::new(t);
        lt.panel(focused)(&lt.iced_theme())
    }

    #[test]
    fn panel_colours_from_tokens() {
        let t = cyberpunk();
        let tok = t.panel(true);
        let s = style(t, true);
        assert_eq!(s.border.color, color(tok.border));
        assert_eq!(s.background, Some(Background::Color(color(tok.background))));
        assert!(s.shadow.blur_radius > 0.0);
        assert_eq!(style(cyberpunk(), false).shadow.blur_radius, 0.0);
    }

    #[test]
    fn panel_adapts_to_shape() {
        assert_eq!(style(rounded_light(), false).border.radius, 8.0.into());
        assert_eq!(style(hard_square(), false).border.radius, 0.0.into());
        assert_eq!(style(hard_square(), false).border.width, 2.0);
        // rounded theme: hard shadow token, no glow
        assert_eq!(style(rounded_light(), true).shadow.offset.x, 2.0);
    }

    #[test]
    fn panel_box_padding_uses_spacing() {
        let b = IcedTheme::new(rounded_light()).panel_box(false);
        assert_eq!(b.padding.top, 20.0);
        assert!(!b.glow);
        assert!(IcedTheme::new(rounded_light()).panel_box(true).glow);
    }
}
