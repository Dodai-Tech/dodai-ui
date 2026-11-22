//! Markdown source as showcase nodes, laid out like the gpui `Markdown` component: the blocks
//! come from [`dodai_ui_core::markdown`], so every toolkit shows the same document.

use dodai_ui_core::markdown::{
    list_markers, parse_blocks, parse_inline, Align as MdAlign, MdBlock, MdInline,
};
use dodai_ui_core::Theme;

use crate::b::*;
use crate::node::{Align, Column, Font, Gap, Ink, Node, Span, Table, TextRole, TextStyle};

/// Inline Markdown as styled runs.
pub fn spans(src: &str) -> Vec<Span> {
    let (text, marks) = parse_inline(src);
    // Cut the text at every range edge, then style each piece by the ranges covering it.
    let mut cuts: Vec<usize> = vec![0, text.len()];
    for (r, _) in &marks {
        cuts.extend([r.start, r.end]);
    }
    cuts.sort_unstable();
    cuts.dedup();
    cuts.windows(2)
        .filter(|w| w[0] < w[1])
        .map(|w| {
            let mut s = Span {
                text: text[w[0]..w[1]].to_string(),
                ..Span::default()
            };
            for (r, k) in &marks {
                if r.start <= w[0] && w[1] <= r.end {
                    match k {
                        MdInline::Bold => s.bold = true,
                        MdInline::Italic => s.italic = true,
                        MdInline::Code => s.code = true,
                        MdInline::Link => s.link = true,
                    }
                }
            }
            s
        })
        .collect()
}

fn rich(src: &str, role: TextRole) -> Node {
    Node::Rich {
        spans: spans(src),
        role,
    }
}

/// The document as a column of blocks.
pub fn document(t: &Theme, src: &str) -> Node {
    let ty = &t.typography;
    let blocks = parse_blocks(src)
        .into_iter()
        .map(|b| match b {
            MdBlock::Heading(level, s) => {
                let size = match level {
                    1 => ty.size_xl * 1.3,
                    2 => ty.size_xl,
                    3 => ty.size_lg,
                    _ => ty.size_md,
                };
                rich(
                    &s,
                    TextRole::Styled(TextStyle {
                        size,
                        font: Font::Display,
                        ink: Ink::Title,
                        caps: false,
                    }),
                )
            }
            MdBlock::Paragraph(s) => rich(&s, TextRole::Body),
            MdBlock::List(items) => {
                let marks = list_markers(&items);
                col(items
                    .iter()
                    .zip(marks)
                    .map(|(it, mark)| {
                        line(vec![
                            tile(it.depth as f32 * 18.0, vec![]),
                            tile(
                                22.0,
                                vec![styled(mark, ty.size_md, Font::Body, Ink::Primary)],
                            ),
                            rich(&it.text, TextRole::Body),
                        ])
                        .gap(Gap::Sm)
                        .start()
                    })
                    .collect())
                .gap(Gap::Xs)
            }
            MdBlock::Table { head, aligns, rows } => Node::Table(Table {
                columns: head
                    .into_iter()
                    .enumerate()
                    .map(|(i, title)| Column {
                        title,
                        width: None,
                        align: match aligns.get(i).copied().unwrap_or_default() {
                            MdAlign::Left => Align::Left,
                            MdAlign::Center => Align::Center,
                            MdAlign::Right => Align::Right,
                        },
                    })
                    .collect(),
                rows,
                height: None,
            }),
            MdBlock::Quote(s) => Node::Quote(Box::new(rich(&s, TextRole::Muted))),
            MdBlock::Code { lang, text } => Node::Code { text, lang },
            MdBlock::Rule => Node::Divider,
        })
        .collect();
    col(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_keep_the_text_and_mark_each_run() {
        let s = spans("a **b** `c` [d](x)");
        let text: String = s.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(text, "a b c d");
        assert!(s.iter().any(|s| s.text == "b" && s.bold));
        assert!(s.iter().any(|s| s.text == "c" && s.code));
        assert!(s.iter().any(|s| s.text == "d" && s.link));
        assert!(s.iter().any(|s| s.text == "a " && !s.bold && !s.code));
    }

    #[test]
    fn plain_text_is_one_run_and_empty_text_none() {
        assert_eq!(spans("plain").len(), 1);
        assert!(spans("").is_empty());
    }

    #[test]
    fn a_document_has_a_node_per_block() {
        let t = dodai_ui_core::themes::neutral();
        let Node::Stack(s) = document(
            &t,
            "# T\n\npara\n\n- a\n  - b\n\n> q\n\n---\n\n```rust\nfn f() {}\n```",
        ) else {
            panic!()
        };
        assert_eq!(s.items.len(), 6);
        assert!(matches!(s.items[4], Node::Divider));
        assert!(matches!(&s.items[5], Node::Code { lang, .. } if lang == "rust"));
    }
}
