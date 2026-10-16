//! List items, divider, scrollable and tooltip.

use iced::{
    widget::{
        button, container, row, rule, scrollable, text,
        tooltip::{self, Tooltip},
        Rule,
    },
    Background, Border, Element, Length, Theme as IcedBaseTheme,
};

use crate::{color, IcedTheme};

impl IcedTheme {
    /// Button style for a selectable row (`theme.list_item` tokens).
    pub fn list_item_style(
        &self,
        selected: bool,
    ) -> impl Fn(&IcedBaseTheme, button::Status) -> button::Style + 'static {
        let idle = self.theme.list_item(selected, false);
        let hover = self.theme.list_item(selected, true);
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

    /// Selectable row with a left accent marker when `selected`.
    pub fn list_item<'a, Message: Clone + 'a>(
        &self,
        label: &str,
        selected: bool,
        on_press: Message,
    ) -> Element<'a, Message> {
        let sp = &self.theme.spacing;
        let marker = self.theme.list_item(selected, false).marker.map(color);
        let bar = container("")
            .width(3)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: marker.map(Background::Color),
                ..container::Style::default()
            });
        let btn = button(text(self.label(label)).size(self.theme.typography.size_md))
            .width(Length::Fill)
            .padding([sp.sm, sp.md])
            .style(self.list_item_style(selected))
            .on_press(on_press);
        row![bar, btn].height(Length::Shrink).into()
    }

    /// Horizontal divider coloured by `theme.divider()`.
    pub fn divider<'a>(&self) -> Rule<'a, IcedBaseTheme> {
        let c = color(self.theme.divider());
        rule::horizontal(1).style(move |_| rule::Style {
            color: c,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        })
    }

    /// Scrollable style: thin rail, scroller lights up on hover / drag.
    pub fn scrollable(
        &self,
    ) -> impl Fn(&IcedBaseTheme, scrollable::Status) -> scrollable::Style + 'static {
        let p = self.theme.palette.clone();
        let r = self.radius().min(4.0);
        move |_t, status| {
            let scroller = |c| scrollable::Scroller {
                background: Background::Color(color(c)),
                border: Border {
                    radius: r.into(),
                    ..Border::default()
                },
            };
            let rail = |c| scrollable::Rail {
                background: Some(Background::Color(color(p.surface))),
                border: Border {
                    radius: r.into(),
                    ..Border::default()
                },
                scroller: scroller(c),
            };
            let c = match status {
                scrollable::Status::Active { .. } => p.border_strong,
                scrollable::Status::Hovered { .. } => p.primary,
                scrollable::Status::Dragged { .. } => p.primary_active,
            };
            scrollable::Style {
                container: container::Style::default(),
                vertical_rail: rail(c),
                horizontal_rail: rail(c),
                gap: None,
                auto_scroll: scrollable::AutoScroll {
                    background: Background::Color(color(p.overlay)),
                    border: Border {
                        color: color(p.border),
                        width: 1.0,
                        radius: r.into(),
                    },
                    shadow: iced::Shadow::default(),
                    icon: color(p.text),
                },
            }
        }
    }

    /// Tooltip bubble container style.
    pub fn tooltip_style(&self) -> impl Fn(&IcedBaseTheme) -> container::Style + 'static {
        let p = &self.theme.palette;
        let bg = color(p.surface_raised);
        let fg = color(p.text);
        let border = self.border(p.border_strong);
        let shadow = self.shadow(false, true);
        move |_| container::Style {
            background: Some(Background::Color(bg)),
            text_color: Some(fg),
            border,
            shadow,
            snap: true,
        }
    }

    /// `content` with a themed tooltip bubble showing `tip`.
    pub fn tooltip<'a, Message: 'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        tip: &str,
        position: tooltip::Position,
    ) -> Tooltip<'a, Message> {
        let sp = &self.theme.spacing;
        let bubble = container(text(self.label(tip)).size(self.theme.typography.size_xs))
            .padding([sp.xs, sp.sm])
            .style(self.tooltip_style());
        Tooltip::new(content, bubble, position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;
    use dodai_core::Theme;

    fn li(t: Theme, sel: bool, s: button::Status) -> button::Style {
        let lt = IcedTheme::new(t);
        lt.list_item_style(sel)(&lt.iced_theme(), s)
    }

    #[test]
    fn list_item_tokens() {
        let t = cyberpunk();
        let want = t.list_item(true, false);
        let s = li(t, true, button::Status::Active);
        assert_eq!(
            s.background,
            want.background.map(|c| Background::Color(color(c)))
        );
        assert!(li(cyberpunk(), false, button::Status::Active)
            .background
            .is_none());
        assert!(li(cyberpunk(), false, button::Status::Hovered)
            .background
            .is_some());
        assert_eq!(
            li(rounded_light(), true, button::Status::Active)
                .border
                .radius,
            8.0.into()
        );
        assert_eq!(
            li(hard_square(), true, button::Status::Active)
                .border
                .radius,
            0.0.into()
        );
    }

    #[test]
    fn divider_colour() {
        let lt = IcedTheme::new(rounded_light());
        let _ = lt.divider(); // builds
        assert_eq!(
            color(lt.theme().divider()),
            color(lt.theme().palette.border)
        );
    }

    #[test]
    fn scrollable_hover_lights_up() {
        let lt = IcedTheme::new(cyberpunk());
        let f = lt.scrollable();
        let a = f(
            &lt.iced_theme(),
            scrollable::Status::Active {
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );
        let h = f(
            &lt.iced_theme(),
            scrollable::Status::Hovered {
                is_horizontal_scrollbar_hovered: false,
                is_vertical_scrollbar_hovered: true,
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );
        assert_ne!(
            a.vertical_rail.scroller.background,
            h.vertical_rail.scroller.background
        );
        assert_eq!(
            h.vertical_rail.scroller.background,
            Background::Color(color(lt.theme().palette.primary))
        );
    }

    #[test]
    fn tooltip_style_adapts() {
        let lt = IcedTheme::new(rounded_light());
        let s = lt.tooltip_style()(&lt.iced_theme());
        assert_eq!(s.border.radius, 8.0.into());
        assert_eq!(s.shadow.offset.x, 2.0);
    }
}
