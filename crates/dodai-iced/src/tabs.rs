//! Tab bar.

use iced::{
    widget::{button, column, container, row, text},
    Background, Border, Element, Length, Theme as IcedBaseTheme,
};

use crate::{color, IcedTheme};

impl IcedTheme {
    /// Button style for one tab (`theme.tab(active, hovered)` tokens).
    pub fn tab_style(
        &self,
        active: bool,
    ) -> impl Fn(&IcedBaseTheme, button::Status) -> button::Style + 'static {
        let idle = self.theme.tab(active, false);
        let hover = self.theme.tab(active, true);
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
        let sp = &self.theme.spacing;
        let mut bar = row![].spacing(sp.xs);
        for (i, l) in labels.iter().enumerate() {
            let is_active = i == active;
            let tok = self.theme.tab(is_active, false);
            let underline = tok.indicator.map(color);
            let btn = button(text(self.label(l)).size(self.theme.typography.size_sm))
                .padding([sp.sm, sp.lg])
                .style(self.tab_style(is_active))
                .on_press(on_select(i));
            let rule =
                container("")
                    .width(Length::Fill)
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
    use crate::testutil::*;
    use dodai_core::Theme;

    fn st(t: Theme, active: bool, s: button::Status) -> button::Style {
        let lt = IcedTheme::new(t);
        lt.tab_style(active)(&lt.iced_theme(), s)
    }

    #[test]
    fn active_highlighted_by_tokens() {
        let t = cyberpunk();
        let want = t.tab(true, false);
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
