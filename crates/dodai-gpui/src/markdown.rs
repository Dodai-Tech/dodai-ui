//! [`Markdown`]: headings, lists, quotes, code, bold / italic / code / links.

use crate::content::*;
use crate::list::Divider;
use crate::text::first_family;
use crate::{hsla, theme_of};
pub use dodai_core::markdown::*;
use dodai_core::Theme;
use gpui::{
    div, px, App, ElementId, FontStyle, FontWeight, HighlightStyle, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, StyledText, UnderlineStyle, Window,
};

/// Rich text from Markdown source.
#[derive(IntoElement)]
pub struct Markdown {
    id: ElementId,
    src: SharedString,
}

impl Markdown {
    /// Create a new [`Markdown`].
    pub fn new(id: impl Into<ElementId>, src: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            src: src.into(),
        }
    }
}

fn inline_text(t: &Theme, src: &str) -> StyledText {
    let (text, spans) = parse_inline(src);
    let p = &t.palette;
    let hl: Vec<_> = spans
        .into_iter()
        .map(|(r, k)| {
            let s = match k {
                MdInline::Bold => HighlightStyle {
                    font_weight: Some(FontWeight::BOLD),
                    ..Default::default()
                },
                MdInline::Italic => HighlightStyle {
                    font_style: Some(FontStyle::Italic),
                    ..Default::default()
                },
                MdInline::Code => HighlightStyle {
                    color: Some(hsla(p.primary)),
                    background_color: Some(hsla(p.surface_raised)),
                    ..Default::default()
                },
                MdInline::Link => HighlightStyle {
                    color: Some(hsla(p.link)),
                    underline: Some(UnderlineStyle {
                        thickness: crate::HAIRLINE,
                        color: None,
                        wavy: false,
                    }),
                    ..Default::default()
                },
            };
            (r, s)
        })
        .collect();
    StyledText::new(text).with_highlights(hl)
}

impl RenderOnce for Markdown {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let p = &t.palette;
        let mut col = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.md))
            .w_full()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(hsla(p.text));
        for (bi, b) in parse_blocks(&self.src).into_iter().enumerate() {
            let el = match b {
                MdBlock::Heading(l, s) => {
                    let size = match l {
                        1 => t.typography.size_xl * 1.3,
                        2 => t.typography.size_xl,
                        3 => t.typography.size_lg,
                        _ => t.typography.size_md,
                    };
                    div()
                        .font_family(first_family(t.typography.display))
                        .text_size(px(size))
                        .text_color(hsla(t.panel(false).title))
                        .child(inline_text(&t, &s))
                        .into_any_element()
                }
                MdBlock::Paragraph(s) => div().child(inline_text(&t, &s)).into_any_element(),
                MdBlock::List(items) => list(&t, &items),
                MdBlock::Table { head, aligns, rows } => table(&t, &head, &aligns, &rows),
                MdBlock::Quote(s) => div()
                    .pl(px(t.spacing.md))
                    .border_l_2()
                    .border_color(hsla(p.primary))
                    .text_color(hsla(p.text_muted))
                    .child(inline_text(&t, &s))
                    .into_any_element(),
                MdBlock::Code { lang, text } => {
                    CodeBlock::new((self.id.clone(), format!("code{bi}")), text)
                        .language(lang)
                        .into_any_element()
                }
                MdBlock::Rule => Divider::new().into_any_element(),
            };
            col = col.child(el);
        }
        col
    }
}

fn list(t: &Theme, items: &[ListItem]) -> gpui::AnyElement {
    let marks = list_markers(items);
    let mut col = div().flex().flex_col().gap(px(t.spacing.xs));
    for (it, mark) in items.iter().zip(marks) {
        col = col.child(
            div()
                .flex()
                .gap(px(t.spacing.sm))
                .pl(px(it.depth as f32 * 18.0))
                .child(
                    div()
                        .w(px(22.0))
                        .flex_none()
                        .text_color(hsla(t.palette.primary))
                        .child(mark),
                )
                .child(div().flex_1().child(inline_text(t, &it.text))),
        );
    }
    col.into_any_element()
}

fn table(t: &Theme, head: &[String], aligns: &[Align], rows: &[Vec<String>]) -> gpui::AnyElement {
    let p = &t.palette;
    let cell = |text: &str, a: Align, bold: bool| {
        let d = div().flex_1().min_w_0().px(px(t.spacing.sm)).py(px(6.0));
        let d = match a {
            Align::Left => d,
            Align::Center => d.text_center(),
            Align::Right => d.text_right(),
        };
        if bold {
            d.font_weight(FontWeight::BOLD).child(inline_text(t, text))
        } else {
            d.child(inline_text(t, text))
        }
    };
    let mut head_row = div()
        .flex()
        .bg(hsla(p.surface_raised))
        .text_color(hsla(p.text_muted));
    for (i, h) in head.iter().enumerate() {
        head_row = head_row.child(cell(h, aligns.get(i).copied().unwrap_or_default(), true));
    }
    let mut col = div()
        .flex()
        .flex_col()
        .w_full()
        .border_1()
        .border_color(hsla(p.border))
        .child(head_row);
    for r in rows {
        let mut row = div().flex().border_t_1().border_color(hsla(p.border));
        for (i, c) in r.iter().enumerate() {
            row = row.child(cell(c, aligns.get(i).copied().unwrap_or_default(), false));
        }
        col = col.child(row);
    }
    col.into_any_element()
}
