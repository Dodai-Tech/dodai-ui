//! Syntax highlighting for code listings: token ranges by language, toolkit-free.

use std::ops::Range;

/// Colour class of a highlighted span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeToken {
    /// A language keyword.
    Keyword,
    /// A string literal.
    Str,
    /// A comment.
    Comment,
    /// A number literal.
    Number,
    /// A type name.
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

pub(crate) fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Highlight spans over the whole `code` (sorted, non-overlapping).
pub fn highlight(code: &str, lang: &str) -> Vec<(Range<usize>, CodeToken)> {
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
            out.push((i..i + e, CodeToken::Comment));
            i += e;
            prev = None;
            continue;
        }
        if l.block_comments && rest.starts_with("/*") {
            let e = rest.find("*/").map_or(rest.len(), |p| p + 2);
            out.push((i..i + e, CodeToken::Comment));
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
            out.push((i..i + e, CodeToken::Str));
            i += e;
            prev = Some(ch);
            continue;
        }
        // numbers and identifiers
        if ch.is_ascii_digit() && !prev.is_some_and(is_ident) {
            let e = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_'))
                .unwrap_or(rest.len());
            out.push((i..i + e, CodeToken::Number));
            i += e;
            prev = Some('0');
            continue;
        }
        if is_ident(ch) {
            let e = rest.find(|c: char| !is_ident(c)).unwrap_or(rest.len());
            let word = &rest[..e];
            if l.keywords.contains(&word) {
                out.push((i..i + e, CodeToken::Keyword));
            } else if word.chars().next().is_some_and(char::is_uppercase) && !l.keywords.is_empty()
            {
                out.push((i..i + e, CodeToken::Type));
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
pub fn highlight_lines(code: &str, lang: &str) -> Vec<Vec<(Range<usize>, CodeToken)>> {
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
}
