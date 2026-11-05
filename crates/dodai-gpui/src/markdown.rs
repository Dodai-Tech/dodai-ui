//! [`Markdown`]: headings, lists, quotes, code, bold / italic / code / links.

use crate::content::*;
use crate::list::Divider;
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    div, px, App, ElementId, FontStyle, FontWeight, HighlightStyle, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, StyledText, UnderlineStyle, Window,
};
use std::ops::Range;

// ───────────────────────────── markdown ─────────────────────────────

/// Inline formatting kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MdInline {
    /// Bold text.
    Bold,
    /// Italic text.
    Italic,
    /// Inline code.
    Code,
    /// A link.
    Link,
}

/// Strip inline markers from `src`; returns the plain text and the formatted ranges into it.
pub fn parse_inline(src: &str) -> (String, Vec<(Range<usize>, MdInline)>) {
    let mut out = String::new();
    let mut spans = Vec::new();
    let mut i = 0;
    let mut prev: Option<char> = None;
    while i < src.len() {
        let rest = &src[i..];
        let mut take = |inner: &str, kind: MdInline, consumed: usize| {
            let at = out.len();
            out.push_str(inner);
            spans.push((at..out.len(), kind));
            consumed
        };
        let step = if let Some(r) = rest.strip_prefix("**") {
            r.find("**")
                .filter(|&e| e > 0)
                .map(|e| take(&r[..e], MdInline::Bold, e + 4))
        } else if let Some(r) = rest.strip_prefix('`') {
            r.find('`')
                .filter(|&e| e > 0)
                .map(|e| take(&r[..e], MdInline::Code, e + 2))
        } else if rest.starts_with('*') || (rest.starts_with('_') && !prev.is_some_and(is_ident)) {
            let m = &rest[..1];
            let r = &rest[1..];
            r.find(m)
                .filter(|&e| e > 0 && !r.starts_with(' '))
                .map(|e| take(&r[..e], MdInline::Italic, e + 2))
        } else if let Some(r) = rest.strip_prefix('[') {
            r.find("](").and_then(|e| {
                let url = &r[e + 2..];
                url.find(')')
                    .filter(|_| e > 0)
                    .map(|u| take(&r[..e], MdInline::Link, 1 + e + 2 + u + 1))
            })
        } else {
            None
        };
        match step {
            Some(n) => {
                prev = Some(' ');
                i += n;
            }
            None => {
                let ch = rest.chars().next().unwrap_or(' ');
                out.push(ch);
                prev = Some(ch);
                i += ch.len_utf8();
            }
        }
    }
    (out, spans)
}

/// A parsed Markdown block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MdBlock {
    /// A heading of the given level (1-6).
    Heading(u8, String),
    /// A paragraph.
    Paragraph(String),
    /// Bulleted / numbered items, possibly nested (see [`ListItem`]).
    List(Vec<ListItem>),
    /// A table.
    Table {
        /// Header cells.
        head: Vec<String>,
        /// Column alignments.
        aligns: Vec<Align>,
        /// Body rows.
        rows: Vec<Vec<String>>,
    },
    /// A block quote.
    Quote(String),
    /// A fenced code block.
    Code {
        /// Language tag after the opening fence.
        lang: String,
        /// The code.
        text: String,
    },
    /// A horizontal rule.
    Rule,
}

/// One list entry; `depth` counts two-space (or one-tab) indents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItem {
    /// Nesting depth, 0 for top level.
    pub depth: u8,
    /// Numbered (`true`) or bulleted.
    pub ordered: bool,
    /// Item text.
    pub text: String,
}

/// Column alignment from a table's `:---:` row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    /// Align left.
    #[default]
    Left,
    /// Centre.
    Center,
    /// Align right.
    Right,
}

fn split_row(line: &str) -> Vec<String> {
    let t = line.trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = t.strip_suffix('|').unwrap_or(t);
    t.split('|').map(|c| c.trim().to_string()).collect()
}

/// `|---|:-:|--:|` -> alignments (None if the line is not a separator row).
fn table_aligns(line: &str) -> Option<Vec<Align>> {
    let t = line.trim();
    if !t.contains('-') || !t.contains('|') && !t.starts_with(':') && !t.starts_with('-') {
        return None;
    }
    let cells = split_row(t);
    let mut out = Vec::new();
    for c in &cells {
        let (l, r) = (c.starts_with(':'), c.ends_with(':'));
        let core = c.trim_matches(':');
        if core.is_empty() || !core.chars().all(|ch| ch == '-') {
            return None;
        }
        out.push(match (l, r) {
            (true, true) => Align::Center,
            (false, true) => Align::Right,
            _ => Align::Left,
        });
    }
    Some(out)
}

fn list_marker(t: &str) -> Option<(bool, &str)> {
    if let Some(item) = ["- ", "* ", "+ "].iter().find_map(|m| t.strip_prefix(m)) {
        return Some((false, item));
    }
    numbered(t).map(|item| (true, item))
}

fn indent_depth(raw: &str) -> u8 {
    let mut w = 0usize;
    for c in raw.chars() {
        match c {
            ' ' => w += 1,
            '\t' => w += 2,
            _ => break,
        }
    }
    (w / 2).min(6) as u8
}

fn numbered(line: &str) -> Option<&str> {
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    (digits > 0)
        .then(|| line[digits..].strip_prefix(". "))
        .flatten()
}

/// Parse the Markdown subset: headings, paragraphs, `-`/`*` and `1.` lists (nested by
/// indentation), pipe tables, `>` quotes, fenced code, `---` rules.
pub fn parse_blocks(src: &str) -> Vec<MdBlock> {
    let mut out: Vec<MdBlock> = Vec::new();
    let mut para: Vec<&str> = Vec::new();
    let flush = |para: &mut Vec<&str>, out: &mut Vec<MdBlock>| {
        if !para.is_empty() {
            out.push(MdBlock::Paragraph(para.join(" ")));
            para.clear();
        }
    };
    let mut lines = src.lines().peekable();
    let mut gap = false; // a blank line ends the current list
    while let Some(raw) = lines.next() {
        let line = raw.trim_end();
        let t = line.trim_start();
        let was_gap = std::mem::replace(&mut gap, t.is_empty());
        if let Some(lang) = t.strip_prefix("```") {
            flush(&mut para, &mut out);
            let mut text = Vec::new();
            for l in lines.by_ref() {
                if l.trim_start().starts_with("```") {
                    break;
                }
                text.push(l);
            }
            out.push(MdBlock::Code {
                lang: lang.trim().to_string(),
                text: text.join("\n"),
            });
        } else if t.is_empty() {
            flush(&mut para, &mut out);
        } else if matches!(t, "---" | "***" | "___") {
            flush(&mut para, &mut out);
            out.push(MdBlock::Rule);
        } else if t.starts_with('#') && t.trim_start_matches('#').starts_with(' ') {
            flush(&mut para, &mut out);
            let level = t.chars().take_while(|&c| c == '#').count().min(6) as u8;
            out.push(MdBlock::Heading(
                level,
                t[level as usize..].trim().to_string(),
            ));
        } else if let Some(q) = t.strip_prefix('>') {
            flush(&mut para, &mut out);
            let q = q.trim().to_string();
            match out.last_mut() {
                Some(MdBlock::Quote(prev)) => {
                    prev.push(' ');
                    prev.push_str(&q);
                }
                _ => out.push(MdBlock::Quote(q)),
            }
        } else if t.contains('|') && lines.peek().is_some_and(|n| table_aligns(n).is_some()) {
            flush(&mut para, &mut out);
            let head = split_row(t);
            let mut aligns = table_aligns(lines.next().unwrap_or("")).unwrap_or_default();
            aligns.resize(head.len(), Align::Left);
            let mut rows = Vec::new();
            while let Some(r) = lines.peek() {
                if r.trim().is_empty() || !r.contains('|') {
                    break;
                }
                let mut cells = split_row(r);
                cells.resize(head.len(), String::new());
                rows.push(cells);
                lines.next();
            }
            out.push(MdBlock::Table { head, aligns, rows });
        } else if let Some((ordered, item)) = list_marker(t) {
            flush(&mut para, &mut out);
            let it = ListItem {
                depth: indent_depth(line),
                ordered,
                text: item.to_string(),
            };
            match out.last_mut() {
                Some(MdBlock::List(v)) if !was_gap => v.push(it),
                _ => out.push(MdBlock::List(vec![it])),
            }
        } else {
            para.push(t);
        }
    }
    flush(&mut para, &mut out);
    out
}

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

/// Marker text for each item: bullets alternate `•`/`◦` with depth, numbers
/// restart whenever the list drops back to a shallower level.
pub fn list_markers(items: &[ListItem]) -> Vec<String> {
    let mut counters = [0usize; 8];
    items
        .iter()
        .enumerate()
        .map(|(i, it)| {
            let d = (it.depth as usize).min(7);
            // Entering a deeper level starts a fresh count there.
            if i == 0 || items[i - 1].depth < it.depth {
                counters[d] = 0;
            }
            for c in counters.iter_mut().skip(d + 1) {
                *c = 0;
            }
            counters[d] += 1;
            if it.ordered {
                format!("{}.", counters[d])
            } else if d % 2 == 0 {
                "•".into()
            } else {
                "◦".into()
            }
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(code: &str, lang: &str) -> Vec<(String, CodeToken)> {
        highlight(code, lang)
            .into_iter()
            .map(|(r, k)| (code[r].to_string(), k))
            .collect()
    }

    #[test]
    fn rust_tokens() {
        let k = kinds("let x: Vec<i32> = 42; // hi", "rust");
        assert!(k.contains(&("let".into(), CodeToken::Keyword)));
        assert!(k.contains(&("Vec".into(), CodeToken::Type)));
        assert!(k.contains(&("42".into(), CodeToken::Number)));
        assert!(k.contains(&("// hi".into(), CodeToken::Comment)));
    }

    #[test]
    fn strings_comments_and_lifetimes() {
        let k = kinds("fn f<'a>(s: &'a str) -> char { 'x' } \"a // b\"", "rust");
        assert!(k.contains(&("'x'".into(), CodeToken::Str)));
        assert!(!k
            .iter()
            .any(|(s, t)| s.starts_with("'a") && *t == CodeToken::Str));
        assert!(k.contains(&("\"a // b\"".into(), CodeToken::Str)));
        assert!(!k.iter().any(|(_, t)| *t == CodeToken::Comment));
    }

    #[test]
    fn lines_split_block_comments() {
        let l = highlight_lines("a /* x\ny */ b", "js");
        assert_eq!(l.len(), 2);
        assert_eq!(l[0], vec![(2..6, CodeToken::Comment)]);
        assert_eq!(l[1], vec![(0..4, CodeToken::Comment)]);
    }

    #[test]
    fn unknown_language_stays_plain() {
        assert!(highlight("let x = 1", "klingon")
            .iter()
            .all(|(_, t)| *t == CodeToken::Number));
    }

    #[test]
    fn inline_markers() {
        let (t, s) = parse_inline("a **bold** and *it* `c` [go](http://x) snake_case");
        assert_eq!(t, "a bold and it c go snake_case");
        let get = |i: usize| (&t[s[i].0.clone()], s[i].1);
        assert_eq!(get(0), ("bold", MdInline::Bold));
        assert_eq!(get(1), ("it", MdInline::Italic));
        assert_eq!(get(2), ("c", MdInline::Code));
        assert_eq!(get(3), ("go", MdInline::Link));
        assert_eq!(s.len(), 4);
        assert_eq!(parse_inline("2 * 3 * 4").0, "2 * 3 * 4");
        assert_eq!(parse_inline("**open").0, "**open");
    }

    #[test]
    fn blocks() {
        let b = parse_blocks("# T\n\npara one\ncontinues\n\n- a\n- b\n\n1. x\n2. y\n> q\n> r\n---\n```rust\nfn f() {}\n```");
        assert_eq!(b[0], MdBlock::Heading(1, "T".into()));
        assert_eq!(b[1], MdBlock::Paragraph("para one continues".into()));
        let item = |d: u8, o: bool, t: &str| ListItem {
            depth: d,
            ordered: o,
            text: t.into(),
        };
        assert_eq!(
            b[2],
            MdBlock::List(vec![item(0, false, "a"), item(0, false, "b")])
        );
        assert_eq!(
            b[3],
            MdBlock::List(vec![item(0, true, "x"), item(0, true, "y")])
        );
        assert_eq!(b[4], MdBlock::Quote("q r".into()));
        assert_eq!(b[5], MdBlock::Rule);
        assert_eq!(
            b[6],
            MdBlock::Code {
                lang: "rust".into(),
                text: "fn f() {}".into()
            }
        );
    }

    #[test]
    fn nested_lists_and_markers() {
        let b = parse_blocks("- a\n  - b\n  - c\n    1. d\n    2. e\n- f");
        let MdBlock::List(items) = &b[0] else {
            panic!("{b:?}")
        };
        let depths: Vec<u8> = items.iter().map(|i| i.depth).collect();
        assert_eq!(depths, vec![0, 1, 1, 2, 2, 0]);
        assert_eq!(list_markers(items), vec!["•", "◦", "◦", "1.", "2.", "•"]);
        // numbering restarts after dropping back out of a nested level
        let b = parse_blocks("1. x\n   1. y\n2. z\n   1. w");
        let MdBlock::List(items) = &b[0] else {
            panic!()
        };
        assert_eq!(list_markers(items), vec!["1.", "1.", "2.", "1."]);
    }

    #[test]
    fn tables() {
        let b = parse_blocks(
            "intro\n\n| Name | Qty | Note |\n|:--|--:|:-:|\n| a | 1 | x |\n| b | 22 |\n\nafter",
        );
        assert_eq!(b[0], MdBlock::Paragraph("intro".into()));
        let MdBlock::Table { head, aligns, rows } = &b[1] else {
            panic!("{b:?}")
        };
        assert_eq!(head, &["Name", "Qty", "Note"]);
        assert_eq!(aligns, &[Align::Left, Align::Right, Align::Center]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1], vec!["b", "22", ""]); // short rows are padded
        assert_eq!(b[2], MdBlock::Paragraph("after".into()));
        // a lone pipe line without a separator row stays a paragraph
        assert!(matches!(parse_blocks("a | b")[0], MdBlock::Paragraph(_)));
    }
}
