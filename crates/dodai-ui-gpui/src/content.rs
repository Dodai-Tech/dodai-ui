//! Content components: [`CodeBlock`] (syntax highlighted, copyable) and
//! [`Markdown`] (headings, lists, quotes, code, bold / italic / code / links).

use crate::paint::{shaped, ShapeStyle};
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_ui_core::Theme;
use gpui::{
    div, px, App, ClipboardItem, ElementId, HighlightStyle, Hsla, InteractiveElement, IntoElement,
    MouseButton, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    StyledText, Window,
};
use std::ops::Range;

pub use crate::markdown::*;

pub use dodai_ui_core::syntax::{highlight, highlight_lines, CodeToken};

fn tok_color(t: &Theme, k: CodeToken) -> gpui::Hsla {
    let p = &t.palette;
    hsla(match k {
        CodeToken::Keyword => p.primary,
        CodeToken::Str => p.success,
        CodeToken::Comment => p.text_muted,
        CodeToken::Number => p.warning,
        CodeToken::Type => p.info,
    })
}

// ───────────────────────────── code block ─────────────────────────────

/// A position in the code: line, byte offset in that line.
type Pos = (usize, usize);

/// Selection anchor and head, in document order only after [`ordered`].
type Sel = Option<(Pos, Pos)>;

fn ordered(a: Pos, b: Pos) -> (Pos, Pos) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// The part of `line` (of `len` bytes) a selection covers, if any.
fn line_selection(sel: (Pos, Pos), line: usize, len: usize) -> Option<Range<usize>> {
    let (a, b) = ordered(sel.0, sel.1);
    if line < a.0 || line > b.0 || a == b {
        return None;
    }
    let from = if line == a.0 { a.1 } else { 0 };
    let to = if line == b.0 { b.1 } else { len };
    (from < to).then_some(from..to.min(len))
}

/// The word (letters, digits, `_`) around byte `at` in `line`; a lone symbol selects itself.
fn word_range(line: &str, at: usize) -> Range<usize> {
    let at = at.min(line.len());
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let start = line[..at]
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_word(*c))
        .last()
        .map_or(at, |(i, _)| i);
    let end = line[at..]
        .char_indices()
        .take_while(|(_, c)| is_word(*c))
        .last()
        .map_or(at, |(i, c)| at + i + c.len_utf8());
    if start == end {
        // On a symbol or space: select that one character.
        let next = line[at..].chars().next().map_or(0, char::len_utf8);
        return at..at + next;
    }
    start..end
}

/// The selected text of `code`.
fn selected_text(code: &str, sel: (Pos, Pos)) -> String {
    code.split('\n')
        .enumerate()
        .filter_map(|(i, l)| line_selection(sel, i, l.len()).map(|r| l[r].to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Token colours over `sel`, as non-overlapping styled spans: tokens keep their colour and the
/// selected part gets `bg` behind it.
fn styled_spans(
    tokens: &[(Range<usize>, Hsla)],
    sel: Option<Range<usize>>,
    bg: Hsla,
    len: usize,
) -> Vec<(Range<usize>, HighlightStyle)> {
    let mut cuts = vec![0, len];
    for (r, _) in tokens {
        cuts.extend([r.start, r.end]);
    }
    if let Some(r) = &sel {
        cuts.extend([r.start, r.end]);
    }
    cuts.retain(|c| *c <= len);
    cuts.sort_unstable();
    cuts.dedup();
    cuts.windows(2)
        .filter_map(|w| {
            let piece = w[0]..w[1];
            let color = tokens
                .iter()
                .find(|(r, _)| r.start <= piece.start && piece.end <= r.end)
                .map(|(_, c)| *c);
            let selected = sel
                .as_ref()
                .is_some_and(|r| r.start <= piece.start && piece.end <= r.end);
            (color.is_some() || selected).then(|| {
                (
                    piece,
                    HighlightStyle {
                        color,
                        background_color: selected.then_some(bg),
                        ..Default::default()
                    },
                )
            })
        })
        .collect()
}

/// Monospace code with line numbers, highlighting, a copy button, scrolling in both directions
/// (optionally with scrollbars) and selectable text (drag, then Cmd/Ctrl-C).
#[derive(IntoElement)]
pub struct CodeBlock {
    id: ElementId,
    code: SharedString,
    lang: SharedString,
    numbers: bool,
    copy: bool,
    max_height: Option<f32>,
    scrollbar: bool,
}

impl CodeBlock {
    /// Create a new [`CodeBlock`].
    pub fn new(id: impl Into<ElementId>, code: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            code: code.into(),
            lang: "".into(),
            numbers: true,
            copy: true,
            max_height: None,
            scrollbar: false,
        }
    }
    /// `rust`, `js`, `ts`, `python`, `bash`, `toml`, `json`, … (others stay plain).
    pub fn language(mut self, l: impl Into<SharedString>) -> Self {
        self.lang = l.into();
        self
    }
    /// Show line numbers.
    pub fn line_numbers(mut self, on: bool) -> Self {
        self.numbers = on;
        self
    }
    /// Show a copy button.
    pub fn copy_button(mut self, on: bool) -> Self {
        self.copy = on;
        self
    }
    /// Scroll vertically past `h` px instead of growing with the code.
    pub fn max_height(mut self, h: f32) -> Self {
        self.max_height = Some(h);
        self
    }
    /// Draw scrollbars (off by default; wheel and trackpad scrolling always work).
    pub fn scrollbar(mut self, on: bool) -> Self {
        self.scrollbar = on;
        self
    }
}

/// The shared selection state of a code block.
struct CodeState {
    sel: gpui::Entity<Sel>,
    selecting: gpui::Entity<bool>,
    focus: gpui::FocusHandle,
}

impl CodeBlock {
    /// Line `i` of the code: gutter number, highlighted text, and the selection handlers.
    fn row(
        &self,
        t: &dodai_ui_core::Theme,
        (i, text, spans): (usize, &str, Vec<(Range<usize>, CodeToken)>),
        cur: Sel,
        state: &CodeState,
        gutter: f32,
    ) -> gpui::Div {
        let p = &t.palette;
        let select_bg = p.primary.with_alpha(0.3);
        let (sel, selecting, focus) = (&state.sel, &state.selecting, &state.focus);
        let tokens: Vec<(Range<usize>, Hsla)> = spans
            .into_iter()
            .map(|(r, k)| (r, tok_color(t, k)))
            .collect();
        let hl = styled_spans(
            &tokens,
            cur.and_then(|s| line_selection(s, i, text.len())),
            hsla(select_bg),
            text.len(),
        );
        let styled = StyledText::new(text.to_string()).with_highlights(hl);
        let layout = styled.layout().clone();
        // Byte offset in this line under the pointer (clamped to the line).
        let at = {
            let layout = layout.clone();
            move |pos| layout.index_for_position(pos).unwrap_or_else(|e| e)
        };
        let mut row = div()
            .flex()
            .min_h(px(t.typography.size_sm * 1.6))
            .on_mouse_down(MouseButton::Left, {
                let (sel, selecting, focus, at) =
                    (sel.clone(), selecting.clone(), focus.clone(), at.clone());
                let line_text = text.to_string();
                move |e, window, cx| {
                    window.focus(&focus);
                    let here = (i, at(e.position));
                    // Double-click selects a word, triple-click the line.
                    let picked = match e.click_count {
                        2 => Some(word_range(&line_text, here.1)),
                        n if n >= 3 => Some(0..line_text.len()),
                        _ => None,
                    };
                    if let Some(r) = picked {
                        sel.update(cx, |s, cx| {
                            *s = Some(((i, r.start), (i, r.end)));
                            cx.notify();
                        });
                        return;
                    }
                    let anchor = match (*sel.read(cx), e.modifiers.shift) {
                        (Some((a, _)), true) => a,
                        _ => here,
                    };
                    sel.update(cx, |s, cx| {
                        *s = Some((anchor, here));
                        cx.notify();
                    });
                    selecting.update(cx, |s, _| *s = true);
                }
            })
            .on_mouse_move({
                let (sel, selecting) = (sel.clone(), selecting.clone());
                move |e, _, cx| {
                    if !*selecting.read(cx) {
                        return;
                    }
                    let here = (i, at(e.position));
                    sel.update(cx, |s, cx| {
                        if let Some((a, _)) = *s {
                            *s = Some((a, here));
                            cx.notify();
                        }
                    });
                }
            });
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
                .child(styled),
        );
        row
    }

    /// The header: language name and the copy button.
    fn head(&self, t: &dodai_ui_core::Theme, copied: gpui::Entity<bool>, cx: &App) -> gpui::Div {
        let p = &t.palette;
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
        head
    }
}

impl RenderOnce for CodeBlock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let p = &t.palette;
        let copied = window.use_keyed_state((self.id.clone(), "copied"), cx, |_, _| false);
        let sel = window.use_keyed_state((self.id.clone(), "sel"), cx, |_, _| None::<(Pos, Pos)>);
        let selecting = window.use_keyed_state((self.id.clone(), "selecting"), cx, |_, _| false);
        let focus =
            window.use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| cx.focus_handle());
        let focus = focus.read(cx).clone();
        let cur: Sel = *sel.read(cx);
        let state = CodeState {
            sel: sel.clone(),
            selecting: selecting.clone(),
            focus: focus.clone(),
        };
        let lines = highlight_lines(&self.code, &self.lang);
        let n = lines.len();
        let gutter = (n.to_string().len() as f32) * 8.0 + 14.0;
        let mut body = div().flex().flex_col().py(px(t.spacing.sm)).cursor_text();
        for (i, (text, spans)) in self.code.split('\n').zip(lines).enumerate() {
            body = body.child(self.row(&t, (i, text, spans), cur, &state, gutter));
        }
        let style =
            ShapeStyle::from_tokens(&t, Some(p.surface_raised), Some(p.border), false, false)
                .with_surface(&t, dodai_ui_core::SurfaceClass::Card);
        let head = self.head(&t, copied, cx);
        let area = crate::display::ScrollArea::new((self.id.clone(), "scroll"))
            .max_height(self.max_height.unwrap_or(f32::from(px(100_000.0))))
            .horizontal(true)
            .scrollbar(self.scrollbar)
            .child(body);
        let code = self.code.clone();
        shaped(&t.shape, style)
            .w_full()
            .flex()
            .flex_col()
            .key_context("DodaiCode")
            .track_focus(&focus)
            .on_action({
                let sel = sel.clone();
                move |_: &crate::text_input::Copy, _, cx| {
                    if let Some(s) = *sel.read(cx) {
                        let text = selected_text(&code, s);
                        if !text.is_empty() {
                            cx.write_to_clipboard(ClipboardItem::new_string(text));
                        }
                    }
                }
            })
            .on_action({
                let (sel, code) = (sel.clone(), self.code.clone());
                move |_: &crate::text_input::SelectAll, _, cx| {
                    let last = code.split('\n').count().saturating_sub(1);
                    let end = code.split('\n').next_back().map_or(0, str::len);
                    sel.update(cx, |s, cx| {
                        *s = Some(((0, 0), (last, end)));
                        cx.notify();
                    });
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let selecting = selecting.clone();
                move |_, _, cx| selecting.update(cx, |s, _| *s = false)
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                selecting.update(cx, |s, _| *s = false)
            })
            .font_family(first_family(t.typography.mono))
            .text_size(px(t.typography.size_sm))
            .text_color(hsla(p.text))
            .child(head)
            .child(area)
    }
}

#[cfg(test)]
mod selection_tests {
    use super::*;

    #[test]
    fn selection_covers_the_right_part_of_each_line() {
        let sel = ((0, 2), (2, 1));
        assert_eq!(line_selection(sel, 0, 5), Some(2..5));
        assert_eq!(line_selection(sel, 1, 3), Some(0..3));
        assert_eq!(line_selection(sel, 2, 4), Some(0..1));
        assert_eq!(line_selection(sel, 3, 4), None);
        // Dragged backwards is the same selection.
        assert_eq!(line_selection(((2, 1), (0, 2)), 0, 5), Some(2..5));
        assert_eq!(line_selection(((1, 2), (1, 2)), 1, 5), None, "empty");
    }

    #[test]
    fn double_click_picks_the_word() {
        let line = "let foo_bar = 1;";
        assert_eq!(word_range(line, 6), 4..11);
        assert_eq!(word_range(line, 4), 4..11);
        assert_eq!(word_range(line, 11), 4..11, "just after the word");
        assert_eq!(word_range(line, 12), 12..13, "a symbol selects itself");
        assert_eq!(word_range("", 0), 0..0);
    }

    #[test]
    fn selected_text_joins_lines() {
        let code = "abc\nde\nfgh";
        assert_eq!(selected_text(code, ((0, 1), (2, 2))), "bc\nde\nfg");
        assert_eq!(selected_text(code, ((1, 0), (1, 2))), "de");
    }

    #[test]
    fn spans_keep_token_colour_and_add_the_selection() {
        let c = Hsla {
            h: 0.5,
            s: 1.0,
            l: 0.5,
            a: 1.0,
        };
        let bg = Hsla {
            h: 0.0,
            s: 1.0,
            l: 0.5,
            a: 0.3,
        };
        let out = styled_spans(&[(0..3, c)], Some(2..5), bg, 6);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].0, 0..2);
        assert!(out[0].1.background_color.is_none() && out[0].1.color == Some(c));
        assert_eq!(out[1].0, 2..3);
        assert!(out[1].1.background_color.is_some() && out[1].1.color == Some(c));
        assert_eq!(out[2].0, 3..5);
        assert!(out[2].1.color.is_none() && out[2].1.background_color.is_some());
    }
}
