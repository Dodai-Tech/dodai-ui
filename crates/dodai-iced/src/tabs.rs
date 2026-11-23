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
    /// Tabs (by index) that show a close button; used by [`IcedTheme::tabs_editable`].
    pub closable: Vec<usize>,
}

impl TabsLook {
    /// Every one of `n` tabs gets a close button.
    pub fn closable_all(mut self, n: usize) -> Self {
        self.closable = (0..n).collect();
        self
    }
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
        self.tabs_editable(labels, active, look, on_select, |_| None, None)
    }

    /// [`tabs_with`](Self::tabs_with) plus editing: tabs listed in [`TabsLook::closable`] get a
    /// close button that sends `on_close(i)` (return `None` to ignore), and `on_add` (when
    /// `Some`) appends a "+" button after the last tab. Controlled like every tab bar: the app
    /// removes or adds the label itself.
    pub fn tabs_editable<'a, Message: Clone + 'a>(
        &self,
        labels: &[&str],
        active: usize,
        look: &TabsLook,
        on_select: impl Fn(usize) -> Message + 'a,
        on_close: impl Fn(usize) -> Option<Message> + 'a,
        on_add: Option<Message>,
    ) -> Element<'a, Message> {
        let sp = &self.theme.spacing;
        let close_d = self.theme.sizes().icon_sm;
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
            let closer = if look.closable.contains(&i) {
                on_close(i)
            } else {
                None
            };
            let pad_r = if closer.is_some() { sp.xs } else { sp.lg };
            let mut btn = button(content)
                .padding(iced::Padding {
                    top: sp.sm,
                    bottom: sp.sm,
                    left: sp.lg,
                    right: pad_r,
                })
                .style(style);
            if !off {
                btn = btn.on_press(on_select(i));
            }
            let extra = if closer.is_some() {
                close_d + sp.sm
            } else {
                0.0
            };
            let head: Element<'a, Message> = match closer {
                Some(msg) => {
                    let x = button(self.icon(dodai_icons::IconId::Close, close_d, ink))
                        .padding(sp.xs)
                        .style(self.tab_style(false))
                        .on_press(msg);
                    row![btn, x]
                        .align_y(iced::alignment::Vertical::Center)
                        .into()
                }
                None => btn.into(),
            };
            // A fixed width (the label's estimated width plus its padding): a `Fill` rule made
            // every tab claim an equal share of the row.
            let rule = container("")
                .width(Length::Fixed(
                    tab_width(l, self.theme.typography.size_sm, sp.lg) + extra,
                ))
                .height(2)
                .style(move |_| container::Style {
                    background: underline.map(Background::Color),
                    ..container::Style::default()
                });
            bar = bar.push(column![head, rule]);
        }
        if let Some(msg) = on_add {
            let plus = button(self.icon(dodai_icons::IconId::Plus, close_d, None))
                .padding(sp.sm)
                .style(self.tab_style(false))
                .on_press(msg);
            bar = bar.push(plus);
        }
        bar.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editable_tabs_build_with_close_and_add() {
        let lt = IcedTheme::new(crate::testutil::cyberpunk());
        let look = TabsLook::default().closable_all(2);
        let _: Element<'_, u8> = lt.tabs_editable(
            &["a", "b"],
            0,
            &look,
            |i| i as u8,
            |i| Some(10 + i as u8),
            Some(99),
        );
        assert_eq!(look.closable, vec![0, 1]);
    }

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

/// The tab a key event moves to: the arrows, Home and End through `n` tabs from `active`,
/// skipping `disabled` ones (the shared roving rule of `dodai-core`). Call it from the app's
/// keyboard subscription and map the result to your tab-select message; `None` for any other
/// key or when nothing enabled lies that way.
pub fn tab_key(
    key: &iced::keyboard::Key,
    active: usize,
    n: usize,
    disabled: &[usize],
) -> Option<usize> {
    use iced::keyboard::key::Named;
    let name = match key.as_ref() {
        iced::keyboard::Key::Named(Named::ArrowLeft) => "left",
        iced::keyboard::Key::Named(Named::ArrowRight) => "right",
        iced::keyboard::Key::Named(Named::Home) => "home",
        iced::keyboard::Key::Named(Named::End) => "end",
        _ => return None,
    };
    dodai_core::roving::next(name, dodai_core::roving::Axis::Horizontal, active, n, |i| {
        !disabled.contains(&i)
    })
}

#[cfg(test)]
mod key_tests {
    use super::*;
    use iced::keyboard::{key::Named, Key};

    #[test]
    fn arrows_skip_disabled_tabs() {
        let right = Key::Named(Named::ArrowRight);
        assert_eq!(tab_key(&right, 0, 3, &[]), Some(1));
        assert_eq!(tab_key(&right, 0, 3, &[1]), Some(2));
        assert_eq!(tab_key(&Key::Named(Named::Home), 2, 3, &[0]), Some(1));
        assert_eq!(tab_key(&Key::Named(Named::Enter), 0, 3, &[]), None);
    }
}
