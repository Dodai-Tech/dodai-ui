//! Tab bar.

use iced::{
    widget::{button, column, container, row, text},
    Background, Border, Element, Length, Theme as IcedBaseTheme,
};

use crate::{color, IcedTheme};

/// How a tab bar looks beyond its labels.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TabsLook {
    /// An underline under the active tab instead of its fill.
    pub secondary: bool,
    /// Tabs that cannot be picked (greyed, no message).
    pub disabled: Vec<usize>,
    /// An icon over each label.
    pub icons: Vec<dodai_icons::IconId>,
}

/// Estimated width of a tab labelled `label` (no font metrics outside layout: ~0.62 em per
/// character) with `pad` px of padding on each side.
pub fn tab_width(label: &str, size: f32, pad: f32) -> f32 {
    label.chars().count() as f32 * size * 0.62 + 2.0 * pad
}

impl IcedTheme {
    /// Button style for one tab (`theme.tab(dodai_core::ItemState::of(active, hovered))` tokens).
    pub fn tab_style(
        &self,
        active: bool,
    ) -> impl Fn(&IcedBaseTheme, button::Status) -> button::Style + 'static {
        let idle = self.theme.tab(dodai_core::ItemState::of(active, false));
        let hover = self.theme.tab(dodai_core::ItemState::of(active, true));
        let radius = self.radius();
        move |_t, status| {
            let t = match status {
                button::Status::Hovered | button::Status::Pressed => hover,
                _ => idle,
            };
            button::Style {
                background: t.background.map(|c| Background::Color(color(c))),
                text_color: color(t.foreground),
                border: Border {
                    radius: radius.into(),
                    ..Border::default()
                },
                ..button::Style::default()
            }
        }
    }

    /// Tab bar: `labels`, the `active` index and an `on_select` message
    /// constructor. The active tab is filled and underlined per theme tokens.
    pub fn tabs<'a, Message: Clone + 'a>(
        &self,
        labels: &[&str],
        active: usize,
        on_select: impl Fn(usize) -> Message + 'a,
    ) -> Element<'a, Message> {
        self.tabs_with(labels, active, &TabsLook::default(), on_select)
    }

    /// [`tabs`](Self::tabs) with a [`TabsLook`]: the secondary style, disabled tabs, icons.
    pub fn tabs_with<'a, Message: Clone + 'a>(
        &self,
        labels: &[&str],
        active: usize,
        look: &TabsLook,
        on_select: impl Fn(usize) -> Message + 'a,
    ) -> Element<'a, Message> {
        let sp = &self.theme.spacing;
        let mut bar = row![].spacing(sp.xs);
        for (i, l) in labels.iter().enumerate() {
            let is_active = i == active;
            let off = look.disabled.contains(&i);
            let tok = self.theme.tab(dodai_core::ItemState::of(is_active, false));
            let underline = if look.secondary && is_active {
                Some(color(self.theme.palette.primary))
            } else {
                tok.indicator.map(color)
            };
            let ink = if off {
                Some(self.theme.palette.text_disabled)
            } else if look.secondary && is_active {
                Some(self.theme.palette.primary)
            } else {
                None
            };
            let mut label = text(self.label(l)).size(self.theme.typography.size_sm);
            if let Some(c) = ink {
                label = label.color(color(c));
            }
            let content: Element<'a, Message> = match look.icons.get(i) {
                Some(&ic) => column![self.icon(ic, 18.0, ink), label]
                    .spacing(sp.xs)
                    .align_x(iced::alignment::Horizontal::Center)
                    .into(),
                None => label.into(),
            };
            // The secondary style never fills the active tab.
            let style = self.tab_style(is_active && !look.secondary);
            let mut btn = button(content).padding([sp.sm, sp.lg]).style(style);
            if !off {
                btn = btn.on_press(on_select(i));
            }
            // A fixed width (the label's estimated width plus its padding): a `Fill` rule made
            // every tab claim an equal share of the row.
            let rule = container("")
                .width(Length::Fixed(tab_width(
                    l,
                    self.theme.typography.size_sm,
                    sp.lg,
                )))
                .height(2)
                .style(move |_| container::Style {
                    background: underline.map(Background::Color),
                    ..container::Style::default()
                });
            bar = bar.push(column![btn, rule]);
        }
        bar.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_width_grows_with_the_label_and_the_padding() {
        assert!(tab_width("Overview", 14.0, 8.0) > tab_width("Usage", 14.0, 8.0));
        assert!(tab_width("Usage", 14.0, 16.0) > tab_width("Usage", 14.0, 8.0));
        assert_eq!(tab_width("", 14.0, 8.0), 16.0);
    }
    use crate::testutil::*;
    use dodai_core::Theme;

    fn st(t: Theme, active: bool, s: button::Status) -> button::Style {
        let lt = IcedTheme::new(t);
        lt.tab_style(active)(&lt.iced_theme(), s)
    }

    #[test]
    fn active_highlighted_by_tokens() {
        let t = cyberpunk();
        let want = t.tab(dodai_core::ItemState::SELECTED);
        let a = st(t, true, button::Status::Active);
        assert_eq!(
            a.background,
            want.background.map(|c| Background::Color(color(c)))
        );
        assert_eq!(a.text_color, color(want.foreground));
        let i = st(cyberpunk(), false, button::Status::Active);
        assert!(i.background.is_none());
        let h = st(cyberpunk(), false, button::Status::Hovered);
        assert!(h.background.is_some());
    }

    #[test]
    fn tab_radius_follows_shape() {
        assert_eq!(
            st(rounded_light(), true, button::Status::Active)
                .border
                .radius,
            8.0.into()
        );
        assert_eq!(
            st(hard_square(), true, button::Status::Active)
                .border
                .radius,
            0.0.into()
        );
    }
}
