//! Content components: [`CodeBlock`] (syntax highlighted, copyable) and
//! [`Markdown`] (headings, lists, quotes, code, bold / italic / code / links).

use crate::list::Divider;
use crate::paint::{shaped, ShapeStyle};
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    div, px, App, ClipboardItem, ElementId, FontStyle, FontWeight, HighlightStyle,
    InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, StyledText, UnderlineStyle, Window,
};
use std::ops::Range;

// ───────────────────────────── syntax highlighting ─────────────────────────────

/// Colour class of a highlighted span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tok {
    Keyword,
    Str,
    Comment,
    Number,
    Type,
}

const RUST: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while",
];
const JS: &[&str] = &[
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "default",
    "else",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "interface",
    "let",
    "new",
    "null",
    "of",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typeof",
    "undefined",
    "var",
    "while",
    "yield",
];
const PY: &[&str] = &[
    "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del", "elif",
    "else", "except", "False", "finally", "for", "from", "global", "if", "import", "in", "is",
    "lambda", "None", "not", "or", "pass", "raise", "return", "True", "try", "while", "with",
    "yield",
];
const SH: &[&str] = &[
    "case", "cd", "do", "done", "echo", "elif", "else", "esac", "export", "fi", "for", "function",
    "if", "in", "local", "return", "then", "while",
];
const DATA: &[&str] = &["true", "false", "null"];

struct Lang {
    keywords: &'static [&'static str],
    line_comment: &'static str,
    block_comments: bool,
    single_quotes: bool,
    /// Rust: `'a` is a lifetime unless it closes like a char literal.
    char_literals: bool,
}

fn lang_of(name: &str) -> Lang {
    match name.to_ascii_lowercase().as_str() {
        "rust" | "rs" => Lang {
            keywords: RUST,
            line_comment: "//",
            block_comments: true,
            single_quotes: false,
            char_literals: true,
        },
        "js" | "javascript" | "ts" | "typescript" | "java" | "kotlin" | "c" | "cpp" | "go"
        | "swift" => Lang {
            keywords: JS,
            line_comment: "//",
            block_comments: true,
            single_quotes: true,
            char_literals: false,
        },
        "py" | "python" => Lang {
            keywords: PY,
            line_comment: "#",
            block_comments: false,
            single_quotes: true,
            char_literals: false,
        },
        "sh" | "bash" | "shell" | "toml" | "yaml" | "yml" => Lang {
            keywords: if name.starts_with('s') || name.starts_with('b') {
                SH
            } else {
                DATA
            },
            line_comment: "#",
            block_comments: false,
            single_quotes: true,
            char_literals: false,
        },
        "json" => Lang {
            keywords: DATA,
            line_comment: "",
            block_comments: false,
            single_quotes: false,
            char_literals: false,
        },
        _ => Lang {
            keywords: &[],
            line_comment: "",
            block_comments: false,
            single_quotes: false,
            char_literals: false,
        },
    }
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Highlight spans over the whole `code` (sorted, non-overlapping).
pub fn highlight(code: &str, lang: &str) -> Vec<(Range<usize>, Tok)> {
    let l = lang_of(lang);
    let mut out = Vec::new();
    let mut i = 0;
    let mut prev: Option<char> = None;
    while i < code.len() {
        let rest = &code[i..];
        let ch = rest.chars().next().unwrap_or(' ');
        // comments
        if !l.line_comment.is_empty() && rest.starts_with(l.line_comment) {
            let e = rest.find('\n').unwrap_or(rest.len());
            out.push((i..i + e, Tok::Comment));
            i += e;
            prev = None;
            continue;
        }
        if l.block_comments && rest.starts_with("/*") {
            let e = rest.find("*/").map_or(rest.len(), |p| p + 2);
            out.push((i..i + e, Tok::Comment));
            i += e;
            prev = None;
            continue;
        }
        // strings
        let quote = ch == '"' || (ch == '\'' && l.single_quotes);
        let char_lit = ch == '\'' && l.char_literals && {
            let mut it = rest.chars().skip(1);
            match (it.next(), it.next(), it.next()) {
                (Some('\\'), Some(_), Some('\'')) => true,
                (Some(c), Some('\''), _) => c != '\'',
                _ => false,
            }
        };
        if quote || char_lit {
            let mut e = ch.len_utf8();
            let mut esc = false;
            for c in rest[e..].chars() {
                e += c.len_utf8();
                if esc {
                    esc = false;
                } else if c == '\\' {
                    esc = true;
                } else if c == ch || c == '\n' {
                    break;
                }
            }
            out.push((i..i + e, Tok::Str));
            i += e;
            prev = Some(ch);
            continue;
        }
        // numbers and identifiers
        if ch.is_ascii_digit() && !prev.is_some_and(is_ident) {
            let e = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_'))
                .unwrap_or(rest.len());
            out.push((i..i + e, Tok::Number));
            i += e;
            prev = Some('0');
            continue;
        }
        if is_ident(ch) {
            let e = rest.find(|c: char| !is_ident(c)).unwrap_or(rest.len());
            let word = &rest[..e];
            if l.keywords.contains(&word) {
                out.push((i..i + e, Tok::Keyword));
            } else if word.chars().next().is_some_and(char::is_uppercase) && !l.keywords.is_empty()
            {
                out.push((i..i + e, Tok::Type));
            }
            i += e;
            prev = Some('a');
            continue;
        }
        i += ch.len_utf8();
        prev = Some(ch);
    }
    out
}

/// Per-line spans (ranges relative to each line) of `code`.
pub fn highlight_lines(code: &str, lang: &str) -> Vec<Vec<(Range<usize>, Tok)>> {
    let spans = highlight(code, lang);
    let mut starts = Vec::new();
    let mut s = 0;
    for line in code.split('\n') {
        starts.push((s, s + line.len()));
        s += line.len() + 1;
    }
    starts
        .iter()
        .map(|&(a, b)| {
            spans
                .iter()
                .filter_map(|(r, t)| {
                    let (x, y) = (r.start.max(a), r.end.min(b));
                    (x < y).then(|| (x - a..y - a, *t))
                })
                .collect()
        })
        .collect()
}

fn tok_color(t: &Theme, k: Tok) -> gpui::Hsla {
    let p = &t.palette;
    hsla(match k {
        Tok::Keyword => p.primary,
        Tok::Str => p.success,
        Tok::Comment => p.text_muted,
        Tok::Number => p.warning,
        Tok::Type => p.info,
    })
}

// ───────────────────────────── code block ─────────────────────────────

/// Monospace code with line numbers, highlighting and a copy button.
#[derive(IntoElement)]
pub struct CodeBlock {
    id: ElementId,
    code: SharedString,
    lang: SharedString,
    numbers: bool,
    copy: bool,
}

impl CodeBlock {
    pub fn new(id: impl Into<ElementId>, code: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            lang: "".into(),
            numbers: true,
            copy: true,
        }
    }
    /// `rust`, `js`, `ts`, `python`, `bash`, `toml`, `json`, … (others stay plain).
    pub fn language(mut self, l: impl Into<SharedString>) -> Self {
        self.lang = l.into();
        self
    }
    pub fn line_numbers(mut self, on: bool) -> Self {
        self.numbers = on;
        self
    }
    pub fn copy_button(mut self, on: bool) -> Self {
        self.copy = on;
        self
    }
}

impl RenderOnce for CodeBlock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let p = &t.palette;
        let copied = window.use_keyed_state((self.id.clone(), "copied"), cx, |_, _| false);
        let lines = highlight_lines(&self.code, &self.lang);
        let n = lines.len();
        let gutter = (n.to_string().len() as f32) * 8.0 + 14.0;
        let mut body = div().flex().flex_col().py(px(t.spacing.sm));
        for (i, (text, spans)) in self.code.split('\n').zip(lines).enumerate() {
            let hl = spans.into_iter().map(|(r, k)| {
                (
                    r,
                    HighlightStyle {
                        color: Some(tok_color(&t, k)),
                        ..Default::default()
                    },
                )
            });
            let mut row = div().flex().min_h(px(t.typography.size_sm * 1.6));
            if self.numbers {
                row = row.child(
                    div()
                        .w(px(gutter))
                        .flex_none()
                        .pr(px(t.spacing.sm))
                        .flex()
                        .justify_end()
                        .text_color(hsla(p.text_disabled))
                        .child((i + 1).to_string()),
                );
            }
            row = row.child(
                div()
                    .pl(px(if self.numbers {
                        t.spacing.xs
                    } else {
                        t.spacing.md
                    }))
                    .pr(px(t.spacing.md))
                    .whitespace_nowrap()
                    .child(StyledText::new(text.to_string()).with_highlights(hl)),
            );
            body = body.child(row);
        }
        let style =
            ShapeStyle::from_tokens(&t, Some(p.surface_raised), Some(p.border), false, false);
        let mut head = div()
            .flex()
            .items_center()
            .justify_between()
            .px(px(t.spacing.md))
            .py(px(t.spacing.xs))
            .text_size(px(t.typography.size_xs))
            .text_color(hsla(p.text_muted))
            .child(self.lang.to_uppercase());
        if self.copy {
            let code = self.code.clone();
            let done = *copied.read(cx);
            head = head.child(
                div()
                    .id((self.id.clone(), "copy"))
                    .cursor_pointer()
                    .text_color(hsla(if done { p.success } else { p.text_muted }))
                    .child(if done { "Copied ✓" } else { "Copy" })
                    .on_click(move |_, _, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(code.to_string()));
                        copied.update(cx, |c, cx| {
                            *c = true;
                            cx.notify();
                        });
                    }),
            );
        }
        shaped(&t.shape, style)
            .w_full()
            .flex()
            .flex_col()
            .font_family(first_family(t.typography.mono))
            .text_size(px(t.typography.size_sm))
            .text_color(hsla(p.text))
            .child(head)
            .child(
                div()
                    .id((self.id.clone(), "body"))
                    .overflow_x_scroll()
                    .child(body),
            )
    }
}

// ───────────────────────────── markdown ─────────────────────────────

/// Inline formatting kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inline {
    Bold,
    Italic,
    Code,
    Link,
}

/// Strip inline markers from `src`; returns the plain text and the formatted ranges into it.
pub fn parse_inline(src: &str) -> (String, Vec<(Range<usize>, Inline)>) {
    let mut out = String::new();
    let mut spans = Vec::new();
    let mut i = 0;
    let mut prev: Option<char> = None;
    while i < src.len() {
        let rest = &src[i..];
        let mut take = |inner: &str, kind: Inline, consumed: usize| {
            let at = out.len();
            out.push_str(inner);
            spans.push((at..out.len(), kind));
            consumed
        };
        let step = if let Some(r) = rest.strip_prefix("**") {
            r.find("**")
                .filter(|&e| e > 0)
                .map(|e| take(&r[..e], Inline::Bold, e + 4))
        } else if let Some(r) = rest.strip_prefix('`') {
            r.find('`')
                .filter(|&e| e > 0)
                .map(|e| take(&r[..e], Inline::Code, e + 2))
        } else if rest.starts_with('*') || (rest.starts_with('_') && !prev.is_some_and(is_ident)) {
            let m = &rest[..1];
            let r = &rest[1..];
            r.find(m)
                .filter(|&e| e > 0 && !r.starts_with(' '))
                .map(|e| take(&r[..e], Inline::Italic, e + 2))
        } else if let Some(r) = rest.strip_prefix('[') {
            r.find("](").and_then(|e| {
                let url = &r[e + 2..];
                url.find(')')
                    .filter(|_| e > 0)
                    .map(|u| take(&r[..e], Inline::Link, 1 + e + 2 + u + 1))
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
pub enum Block {
    Heading(u8, String),
    Paragraph(String),
    Bullets(Vec<String>),
    Numbered(Vec<String>),
    Quote(String),
    Code { lang: String, text: String },
    Rule,
}

fn numbered(line: &str) -> Option<&str> {
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    (digits > 0)
        .then(|| line[digits..].strip_prefix(". "))
        .flatten()
}

/// Parse the Markdown subset: headings, paragraphs, `-`/`*` and `1.` lists, `>` quotes,
/// fenced code, `---` rules.
pub fn parse_blocks(src: &str) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    let mut para: Vec<&str> = Vec::new();
    let flush = |para: &mut Vec<&str>, out: &mut Vec<Block>| {
        if !para.is_empty() {
            out.push(Block::Paragraph(para.join(" ")));
            para.clear();
        }
    };
    let mut lines = src.lines();
    while let Some(raw) = lines.next() {
        let line = raw.trim_end();
        let t = line.trim_start();
        if let Some(lang) = t.strip_prefix("```") {
            flush(&mut para, &mut out);
            let mut text = Vec::new();
            for l in lines.by_ref() {
                if l.trim_start().starts_with("```") {
                    break;
                }
                text.push(l);
            }
            out.push(Block::Code {
                lang: lang.trim().to_string(),
                text: text.join("\n"),
            });
        } else if t.is_empty() {
            flush(&mut para, &mut out);
        } else if matches!(t, "---" | "***" | "___") {
            flush(&mut para, &mut out);
            out.push(Block::Rule);
        } else if t.starts_with('#') && t.trim_start_matches('#').starts_with(' ') {
            flush(&mut para, &mut out);
            let level = t.chars().take_while(|&c| c == '#').count().min(6) as u8;
            out.push(Block::Heading(
                level,
                t[level as usize..].trim().to_string(),
            ));
        } else if let Some(q) = t.strip_prefix('>') {
            flush(&mut para, &mut out);
            let q = q.trim().to_string();
            match out.last_mut() {
                Some(Block::Quote(prev)) => {
                    prev.push(' ');
                    prev.push_str(&q);
                }
                _ => out.push(Block::Quote(q)),
            }
        } else if let Some(item) = ["- ", "* ", "+ "].iter().find_map(|m| t.strip_prefix(m)) {
            flush(&mut para, &mut out);
            match out.last_mut() {
                Some(Block::Bullets(v)) => v.push(item.to_string()),
                _ => out.push(Block::Bullets(vec![item.to_string()])),
            }
        } else if let Some(item) = numbered(t) {
            flush(&mut para, &mut out);
            match out.last_mut() {
                Some(Block::Numbered(v)) => v.push(item.to_string()),
                _ => out.push(Block::Numbered(vec![item.to_string()])),
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
                Inline::Bold => HighlightStyle {
                    font_weight: Some(FontWeight::BOLD),
                    ..Default::default()
                },
                Inline::Italic => HighlightStyle {
                    font_style: Some(FontStyle::Italic),
                    ..Default::default()
                },
                Inline::Code => HighlightStyle {
                    color: Some(hsla(p.primary)),
                    background_color: Some(hsla(p.surface_raised)),
                    ..Default::default()
                },
                Inline::Link => HighlightStyle {
                    color: Some(hsla(p.link)),
                    underline: Some(UnderlineStyle {
                        thickness: px(1.0),
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
                Block::Heading(l, s) => {
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
                Block::Paragraph(s) => div().child(inline_text(&t, &s)).into_any_element(),
                Block::Bullets(items) | Block::Numbered(items) if items.is_empty() => {
                    div().into_any_element()
                }
                Block::Bullets(items) => list(&t, items, false),
                Block::Numbered(items) => list(&t, items, true),
                Block::Quote(s) => div()
                    .pl(px(t.spacing.md))
                    .border_l_2()
                    .border_color(hsla(p.primary))
                    .text_color(hsla(p.text_muted))
                    .child(inline_text(&t, &s))
                    .into_any_element(),
                Block::Code { lang, text } => {
                    CodeBlock::new((self.id.clone(), format!("code{bi}")), text)
                        .language(lang)
                        .into_any_element()
                }
                Block::Rule => Divider::new().into_any_element(),
            };
            col = col.child(el);
        }
        col
    }
}

fn list(t: &Theme, items: Vec<String>, ordered: bool) -> gpui::AnyElement {
    let mut col = div().flex().flex_col().gap(px(t.spacing.xs));
    for (i, it) in items.iter().enumerate() {
        col = col.child(
            div()
                .flex()
                .gap(px(t.spacing.sm))
                .child(
                    div()
                        .w(px(22.0))
                        .flex_none()
                        .text_color(hsla(t.palette.primary))
                        .child(if ordered {
                            format!("{}.", i + 1)
                        } else {
                            "•".to_string()
                        }),
                )
                .child(div().flex_1().child(inline_text(t, it))),
        );
    }
    col.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(code: &str, lang: &str) -> Vec<(String, Tok)> {
        highlight(code, lang)
            .into_iter()
            .map(|(r, k)| (code[r].to_string(), k))
            .collect()
    }

    #[test]
    fn rust_tokens() {
        let k = kinds("let x: Vec<i32> = 42; // hi", "rust");
        assert!(k.contains(&("let".into(), Tok::Keyword)));
        assert!(k.contains(&("Vec".into(), Tok::Type)));
        assert!(k.contains(&("42".into(), Tok::Number)));
        assert!(k.contains(&("// hi".into(), Tok::Comment)));
    }

    #[test]
    fn strings_comments_and_lifetimes() {
        let k = kinds("fn f<'a>(s: &'a str) -> char { 'x' } \"a // b\"", "rust");
        assert!(k.contains(&("'x'".into(), Tok::Str)));
        assert!(!k.iter().any(|(s, t)| s.starts_with("'a") && *t == Tok::Str));
        assert!(k.contains(&("\"a // b\"".into(), Tok::Str)));
        assert!(!k.iter().any(|(_, t)| *t == Tok::Comment));
    }

    #[test]
    fn lines_split_block_comments() {
        let l = highlight_lines("a /* x\ny */ b", "js");
        assert_eq!(l.len(), 2);
        assert_eq!(l[0], vec![(2..6, Tok::Comment)]);
        assert_eq!(l[1], vec![(0..4, Tok::Comment)]);
    }

    #[test]
    fn unknown_language_stays_plain() {
        assert!(highlight("let x = 1", "klingon")
            .iter()
            .all(|(_, t)| *t == Tok::Number));
    }

    #[test]
    fn inline_markers() {
        let (t, s) = parse_inline("a **bold** and *it* `c` [go](http://x) snake_case");
        assert_eq!(t, "a bold and it c go snake_case");
        let get = |i: usize| (&t[s[i].0.clone()], s[i].1);
        assert_eq!(get(0), ("bold", Inline::Bold));
        assert_eq!(get(1), ("it", Inline::Italic));
        assert_eq!(get(2), ("c", Inline::Code));
        assert_eq!(get(3), ("go", Inline::Link));
        assert_eq!(s.len(), 4);
        assert_eq!(parse_inline("2 * 3 * 4").0, "2 * 3 * 4");
        assert_eq!(parse_inline("**open").0, "**open");
    }

    #[test]
    fn blocks() {
        let b = parse_blocks("# T\n\npara one\ncontinues\n\n- a\n- b\n\n1. x\n2. y\n> q\n> r\n---\n```rust\nfn f() {}\n```");
        assert_eq!(b[0], Block::Heading(1, "T".into()));
        assert_eq!(b[1], Block::Paragraph("para one continues".into()));
        assert_eq!(b[2], Block::Bullets(vec!["a".into(), "b".into()]));
        assert_eq!(b[3], Block::Numbered(vec!["x".into(), "y".into()]));
        assert_eq!(b[4], Block::Quote("q r".into()));
        assert_eq!(b[5], Block::Rule);
        assert_eq!(
            b[6],
            Block::Code {
                lang: "rust".into(),
                text: "fn f() {}".into()
            }
        );
    }
}
