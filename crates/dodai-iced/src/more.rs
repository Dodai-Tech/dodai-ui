//! Named components built from the shared kit recipes: the same set gpui exposes, in iced.

use dodai_core::{ButtonVariant, CardVariant, Kit, StepState, Tone};
use iced::{
    widget::{column, container, row, text, Space},
    Alignment, Element, Length,
};

use crate::{color, IcedTheme};

#[cfg(feature = "icons")]
use dodai_icons::IconId;

impl IcedTheme {
    /// A small glyph: an icon with the `icons` feature, `fallback` text without.
    fn mark<'a, Message: 'a>(
        &self,
        #[cfg(feature = "icons")] id: IconId,
        #[cfg(not(feature = "icons"))] _id: (),
        fallback: &str,
        size: f32,
    ) -> Element<'a, Message> {
        #[cfg(feature = "icons")]
        {
            let _ = fallback;
            self.icon(id, size, None)
        }
        #[cfg(not(feature = "icons"))]
        {
            text(fallback.to_string()).size(size).into()
        }
    }

    /// A code listing in the design's code box.
    pub fn code_block<'a, Message: 'a>(&self, code: &str) -> Element<'a, Message> {
        let t = &self.theme.typography;
        self.kit_box(
            Kit::Code,
            text(code.to_string())
                .size(t.size_sm)
                .font(iced::Font::MONOSPACE),
        )
    }

    /// Body text.
    pub fn text<'a, Message: 'a>(&self, s: &str) -> Element<'a, Message> {
        self.body_text(s).into()
    }

    /// Monospace text.
    pub fn mono<'a, Message: 'a>(&self, s: &str) -> Element<'a, Message> {
        self.mono_text(s).into()
    }

    /// A pill button around one icon.
    pub fn icon_button<'a, Message: Clone + 'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        on_press: Message,
    ) -> Element<'a, Message> {
        self.kit_button(Kit::Chip { selected: false }, content, on_press)
    }

    /// The floating primary action.
    pub fn fab<'a, Message: Clone + 'a>(&self, on_press: Message) -> Element<'a, Message> {
        let plus = self.mark(
            #[cfg(feature = "icons")]
            IconId::Plus,
            #[cfg(not(feature = "icons"))]
            (),
            "+",
            24.0,
        );
        self.button_sized(
            "",
            Some(plus),
            ButtonVariant::Primary,
            dodai_core::ControlSize::Large,
            Some(on_press),
        )
    }

    /// A determinate progress bar, `value` in 0..=1.
    pub fn progress_bar<'a, Message: 'a>(
        &self,
        value: f32,
        tone: Option<Tone>,
    ) -> Element<'a, Message> {
        iced::widget::progress_bar(0.0..=1.0, value.clamp(0.0, 1.0))
            .girth(6.0)
            .style(self.progress(tone))
            .into()
    }

    /// Page buttons: `count` pages, `current` selected, one message per page.
    pub fn pagination<'a, Message: Clone + 'a>(
        &self,
        count: usize,
        current: usize,
        on_page: impl Fn(usize) -> Message + 'a,
    ) -> Element<'a, Message> {
        let mut r = row![]
            .spacing(self.theme.spacing.xs)
            .align_y(Alignment::Center);
        for i in 0..count {
            let label = text((i + 1).to_string()).size(self.theme.typography.size_sm);
            r = r.push(self.kit_button(
                Kit::Page {
                    selected: i == current,
                },
                label,
                on_page(i),
            ));
        }
        r.into()
    }

    /// A star rating: `value` of `max` stars lit.
    pub fn rating<'a, Message: 'a>(&self, value: usize, max: usize) -> Element<'a, Message> {
        let p = &self.theme.palette;
        let mut r = row![].spacing(self.theme.spacing.xs / 2.0);
        for i in 0..max {
            let ink = if i < value { p.primary } else { p.text_muted };
            #[cfg(feature = "icons")]
            let star: Element<'a, Message> =
                self.icon_styled(IconId::Star, 20.0, Some(ink), i < value);
            #[cfg(not(feature = "icons"))]
            let star: Element<'a, Message> = text("*").size(20.0).color(color(ink)).into();
            r = r.push(star);
        }
        r.into()
    }

    /// A step row: numbered dots joined by rails, labels beneath.
    pub fn stepper<'a, Message: 'a>(
        &self,
        labels: &[&str],
        current: usize,
    ) -> Element<'a, Message> {
        let p = &self.theme.palette;
        let rail = color(p.border);
        let mut r = row![]
            .spacing(self.theme.spacing.sm)
            .align_y(Alignment::Center);
        for (i, l) in labels.iter().enumerate() {
            let state = StepState::of(i, current);
            let dot = self.kit_box(
                Kit::StepDot(state),
                text((i + 1).to_string()).size(self.theme.typography.size_xs),
            );
            let ink = if state == StepState::Todo {
                p.text_muted
            } else {
                p.text
            };
            r = r.push(dot).push(
                text(l.to_string())
                    .size(self.theme.typography.size_sm)
                    .color(color(ink)),
            );
            if i + 1 < labels.len() {
                r = r.push(
                    container(Space::new().width(24.0).height(1.0)).style(move |_| {
                        container::Style {
                            background: Some(iced::Background::Color(rail)),
                            ..container::Style::default()
                        }
                    }),
                );
            }
        }
        r.into()
    }

    /// A closed search field: icon and placeholder in a pill.
    pub fn search_bar<'a, Message: 'a>(&self, placeholder: &str) -> Element<'a, Message> {
        let p = &self.theme.palette;
        let glass = self.mark(
            #[cfg(feature = "icons")]
            IconId::MagnifyingGlass,
            #[cfg(not(feature = "icons"))]
            (),
            "?",
            self.theme.typography.size_md,
        );
        self.kit_box(
            Kit::Chip { selected: false },
            row![
                glass,
                text(placeholder.to_string())
                    .size(self.theme.typography.size_md)
                    .color(color(p.text_muted))
            ]
            .spacing(self.theme.spacing.sm)
            .align_y(Alignment::Center),
        )
    }

    /// A collapsible section: a header row, the body only when `open`.
    pub fn accordion<'a, Message: Clone + 'a>(
        &self,
        title: &str,
        body: impl Into<Element<'a, Message>>,
        open: bool,
        on_toggle: Message,
    ) -> Element<'a, Message> {
        let chevron = self.mark(
            #[cfg(feature = "icons")]
            if open {
                IconId::CaretUp
            } else {
                IconId::CaretDown
            },
            #[cfg(not(feature = "icons"))]
            (),
            if open { "^" } else { "v" },
            self.theme.typography.size_md,
        );
        let head = self.kit_button(
            Kit::Row { selected: false },
            row![text(title.to_string()).width(Length::Fill), chevron].align_y(Alignment::Center),
            on_toggle,
        );
        let mut c = column![head].spacing(self.theme.spacing.xs);
        if open {
            c = c.push(self.card(CardVariant::Outlined, body));
        }
        c.into()
    }

    /// A row of actions on a raised plate.
    pub fn toolbar<'a, Message: 'a>(
        &self,
        items: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        self.kit_box(
            Kit::Card(CardVariant::Filled),
            row(items)
                .spacing(self.theme.spacing.sm)
                .align_y(Alignment::Center),
        )
    }

    /// A top bar: title on the left, actions on the right.
    pub fn top_app_bar<'a, Message: 'a>(
        &self,
        title: &str,
        actions: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let head = text(self.label(title))
            .size(self.theme.typography.size_lg)
            .width(Length::Fill);
        self.kit_box(
            Kit::Card(CardVariant::Filled),
            row![head, row(actions).spacing(self.theme.spacing.sm)].align_y(Alignment::Center),
        )
    }

    /// A bottom navigation bar: one entry per label, `current` picked.
    pub fn navigation_bar<'a, Message: Clone + 'a>(
        &self,
        labels: &[&str],
        current: usize,
        on_pick: impl Fn(usize) -> Message + 'a,
    ) -> Element<'a, Message> {
        let mut r = row![].spacing(self.theme.spacing.sm);
        for (i, l) in labels.iter().enumerate() {
            let label = text(self.label(l)).size(self.theme.typography.size_sm);
            r = r.push(self.kit_button(
                Kit::Row {
                    selected: i == current,
                },
                label,
                on_pick(i),
            ));
        }
        self.kit_box(Kit::Card(CardVariant::Filled), r)
    }

    /// One row of a tree: indented by `depth`, with a caret when it has children.
    pub fn tree_row<'a, Message: Clone + 'a>(
        &self,
        label: &str,
        depth: usize,
        open: Option<bool>,
        selected: bool,
        on_press: Message,
    ) -> Element<'a, Message> {
        let caret: Element<'a, Message> = match open {
            Some(o) => self.mark(
                #[cfg(feature = "icons")]
                if o {
                    IconId::CaretDown
                } else {
                    IconId::CaretRight
                },
                #[cfg(not(feature = "icons"))]
                (),
                if o { "v" } else { ">" },
                self.theme.typography.size_sm,
            ),
            None => Space::new().width(self.theme.typography.size_sm).into(),
        };
        let content = row![
            Space::new().width(depth as f32 * self.theme.spacing.md),
            caret,
            text(label.to_string()).size(self.theme.typography.size_md)
        ]
        .spacing(self.theme.spacing.xs)
        .align_y(Alignment::Center);
        self.kit_button(Kit::Row { selected }, content, on_press)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn every_component_builds_under_every_shape() {
        for theme in [cyberpunk(), rounded_light(), hard_square()] {
            let t = IcedTheme::new(theme);
            let _: Element<'_, u8> = t.code_block("fn main() {}");
            let _: Element<'_, u8> = t.fab(0);
            let _: Element<'_, u8> = t.progress_bar(0.4, None);
            let _: Element<'_, u8> = t.pagination(5, 2, |i| i as u8);
            let _: Element<'_, u8> = t.rating(3, 5);
            let _: Element<'_, u8> = t.stepper(&["a", "b", "c"], 1);
            let _: Element<'_, u8> = t.search_bar("Search");
            let _: Element<'_, u8> = t.accordion("T", t.text("body"), true, 0);
            let _: Element<'_, u8> = t.top_app_bar("Title", vec![t.mono("x")]);
            let _: Element<'_, u8> = t.navigation_bar(&["A", "B"], 0, |i| i as u8);
            let _: Element<'_, u8> = t.tree_row("node", 1, Some(true), false, 0);
        }
    }
}
