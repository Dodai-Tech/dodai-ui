//! Single-line text field: [`InputField`].

use crate::{shape::border_set, TuiTheme};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

/// Text input drawn from the input tokens. With a 3+ row area it gets a
/// shape-aware border; with 1 row it renders `▸ value`. The cursor is a block
/// cell at char index `cursor` (default: end of value) when focused.
pub struct InputField<'a> {
    theme: &'a TuiTheme,
    value: &'a str,
    placeholder: &'a str,
    focused: bool,
    cursor: Option<usize>,
}

impl<'a> InputField<'a> {
    pub fn new(theme: &'a TuiTheme, value: &'a str) -> Self {
        Self {
            theme,
            value,
            placeholder: "",
            focused: false,
            cursor: None,
        }
    }

    pub fn placeholder(mut self, p: &'a str) -> Self {
        self.placeholder = p;
        self
    }

    pub fn focused(mut self, f: bool) -> Self {
        self.focused = f;
        self
    }

    /// Cursor position in chars (clamped to the value length).
    pub fn cursor(mut self, c: usize) -> Self {
        self.cursor = Some(c);
        self
    }

    /// The one-line rendering as a [`Line`].
    pub fn line(&self) -> Line<'static> {
        let th = self.theme;
        let t = th.theme().input(self.focused);
        let base = Style::new().bg(th.color(t.background));
        let mut spans = vec![Span::styled("▸ ", base.fg(th.color(t.border)))];
        if self.value.is_empty() {
            if self.focused {
                spans.push(Span::styled(" ", th.cursor()));
            }
            spans.push(Span::styled(
                self.placeholder.to_string(),
                base.fg(th.color(t.placeholder)),
            ));
        } else {
            let chars: Vec<char> = self.value.chars().collect();
            let at = self.cursor.unwrap_or(chars.len()).min(chars.len());
            let txt = base.fg(th.color(t.text));
            let head: String = chars[..at].iter().collect();
            spans.push(Span::styled(head, txt));
            if self.focused {
                let cur = chars.get(at).map_or(" ".to_string(), |c| c.to_string());
                spans.push(Span::styled(cur, th.cursor()));
                let tail: String = chars.iter().skip(at + 1).collect();
                spans.push(Span::styled(tail, txt));
            } else {
                let tail: String = chars[at..].iter().collect();
                spans.push(Span::styled(tail, txt));
            }
        }
        Line::from(spans)
    }
}

impl Widget for InputField<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let th = self.theme;
        let t = th.theme().input(self.focused);
        buf.set_style(area, Style::new().bg(th.color(t.background)));
        let inner = if area.height >= 3 {
            let b = Block::new()
                .borders(Borders::ALL)
                .border_set(border_set(&th.theme().shape))
                .border_style(Style::new().fg(th.color(t.border)));
            let i = b.inner(area);
            b.render(area, buf);
            i
        } else {
            area
        };
        self.line().render(inner, buf);
    }
}
