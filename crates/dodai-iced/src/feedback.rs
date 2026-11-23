//! Badge, alert, chip, card, snackbar and dialog, drawn from the shared recipes in
//! `dodai_core::component` (the same ones gpui draws), shaped per theme.

use dodai_core::{CardVariant, ItemState, SurfaceTokens, Tone};
use iced::{
    widget::{column, row, text},
    Alignment, Element, Length, Padding,
};

use crate::{color, BoxStyle, IcedTheme};

impl IcedTheme {
    /// [`BoxStyle`] for a recipe surface with the given padding.
    fn surface_box(&self, s: SurfaceTokens, padding: Padding) -> BoxStyle {
        BoxStyle {
            fill: s.background,
            stroke: s.border,
            glow: s.glow,
            hard_shadow: s.shadow,
            frame: None,
            padding,
        }
    }

    /// Solid status badge (a count or a short label).
    pub fn badge<'a, Message: 'a>(&self, label: &str, tone: Tone) -> Element<'a, Message> {
        let s = self.theme.badge(tone);
        let sp = &self.theme.spacing;
        let content = text(self.label(label))
            .size(self.theme.typography.size_xs)
            .color(color(s.foreground));
        self.shaped_box(
            content,
            self.surface_box(s, Padding::from([sp.xs / 2.0, sp.sm])),
        )
    }

    /// Alert: tinted and outlined, or a solid `banner`. A bold title over the message.
    pub fn alert<'a, Message: 'a>(
        &self,
        title: &str,
        message: &str,
        tone: Tone,
        banner: bool,
    ) -> Element<'a, Message> {
        let s = self.theme.alert(tone, banner);
        let t = &self.theme.typography;
        let content = column![
            text(title.to_string())
                .size(t.size_md)
                .color(color(s.foreground)),
            text(message.to_string())
                .size(t.size_sm)
                .color(color(s.foreground)),
        ]
        .spacing(self.theme.spacing.xs)
        .width(Length::Fill);
        let pad = Padding::from([self.theme.spacing.sm, self.theme.spacing.md]);
        self.shaped_box(content, self.surface_box(s, pad))
    }

    /// Filter chip; `state` carries selected and hovered.
    pub fn chip<'a, Message: 'a>(&self, label: &str, state: ItemState) -> Element<'a, Message> {
        let s = self.theme.chip(state);
        let sp = &self.theme.spacing;
        let content = text(self.label(label))
            .size(self.theme.typography.size_sm)
            .color(color(s.foreground));
        self.shaped_box(content, self.surface_box(s, Padding::from([sp.xs, sp.md])))
    }

    /// Card in one of the three variants, around `content`.
    pub fn card<'a, Message: 'a>(
        &self,
        variant: CardVariant,
        content: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let s = self.theme.card(variant);
        self.shaped_box(
            content,
            self.surface_box(s, Padding::from(self.theme.spacing.md)),
        )
    }

    /// Snackbar: a message with one action, on the raised surface.
    pub fn snackbar<'a, Message: 'a>(&self, message: &str, action: &str) -> Element<'a, Message> {
        let (s, action_colour) = self.theme.snackbar();
        let t = &self.theme.typography;
        let content = row![
            text(message.to_string())
                .size(t.size_sm)
                .color(color(s.foreground)),
            text(self.label(action))
                .size(t.size_sm)
                .color(color(action_colour)),
        ]
        .spacing(self.theme.spacing.lg)
        .align_y(Alignment::Center);
        let pad = Padding::from([self.theme.spacing.sm, self.theme.spacing.md]);
        self.shaped_box(content, self.surface_box(s, pad))
    }

    /// Dialog box: a title, a body and a row of actions.
    pub fn dialog<'a, Message: 'a>(
        &self,
        title: &str,
        body: &str,
        actions: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let s = self.theme.dialog();
        let t = &self.theme.typography;
        let content = column![
            text(title.to_string())
                .size(t.size_lg)
                .color(color(s.foreground)),
            text(body.to_string())
                .size(t.size_sm)
                .color(color(s.foreground)),
            actions.into(),
        ]
        .spacing(self.theme.spacing.md);
        self.shaped_box(
            content,
            self.surface_box(s, Padding::from(self.theme.spacing.lg)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn badge_box_follows_the_recipe() {
        for theme in [cyberpunk(), rounded_light()] {
            let lt = IcedTheme::new(theme);
            let s = lt.theme().badge(Tone::Danger);
            let b = lt.surface_box(s, Padding::ZERO);
            assert_eq!((b.fill, b.stroke), (s.background, s.border));
        }
    }

    #[test]
    fn alert_is_tinted_unless_banner() {
        let lt = IcedTheme::new(rounded_light());
        let tinted = lt.surface_box(lt.theme().alert(Tone::Info, false), Padding::ZERO);
        let banner = lt.surface_box(lt.theme().alert(Tone::Info, true), Padding::ZERO);
        assert!(tinted.stroke.is_some() && tinted.fill.unwrap().a < 255);
        assert!(banner.stroke.is_none());
    }

    #[test]
    fn builds_every_widget() {
        let lt = IcedTheme::new(rounded_light());
        let _: Element<'_, ()> = lt.badge("3", Tone::Info);
        let _: Element<'_, ()> = lt.alert("Title", "Body", Tone::Warning, false);
        let _: Element<'_, ()> = lt.chip("Filter", ItemState::default());
        let _: Element<'_, ()> = lt.card(CardVariant::Filled, text("x"));
        let _: Element<'_, ()> = lt.snackbar("Saved", "Undo");
        let _: Element<'_, ()> = lt.dialog("T", "B", text("ok"));
    }
}
