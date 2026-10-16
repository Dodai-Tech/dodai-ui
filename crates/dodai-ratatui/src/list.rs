//! Selectable list: [`SelectList`].

use crate::TuiTheme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

/// One row per item; the selected row gets the list-item background and a
/// marker bar (`▌`) in the marker colour. Scrolls to keep selection visible.
pub struct SelectList<'a> {
    theme: &'a TuiTheme,
    items: Vec<String>,
    selected: Option<usize>,
}

impl<'a> SelectList<'a> {
    pub fn new<S: AsRef<str>>(theme: &'a TuiTheme, items: &[S]) -> Self {
        Self {
            theme,
            items: items.iter().map(|s| theme.label(s.as_ref())).collect(),
            selected: None,
        }
    }

    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self
    }
}

impl Widget for SelectList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let th = self.theme;
        let h = area.height as usize;
        let off = self.selected.map_or(0, |s| (s + 1).saturating_sub(h));
        for (row, (i, item)) in self.items.iter().enumerate().skip(off).enumerate() {
            if row >= h {
                break;
            }
            let sel = self.selected == Some(i);
            let t = th.theme().list_item(sel, false);
            let mut st = Style::new().fg(th.color(t.foreground));
            if let Some(bg) = t.background {
                st = st.bg(th.color(bg));
            }
            if sel {
                st = st.add_modifier(Modifier::BOLD);
            }
            let y = area.y + row as u16;
            buf.set_style(Rect::new(area.x, y, area.width, 1), st);
            let marker = match t.marker {
                Some(m) => Span::styled("▌", st.fg(th.color(m))),
                None => Span::styled(" ", st),
            };
            Line::from(vec![marker, Span::styled(format!(" {item}"), st)])
                .render(Rect::new(area.x, y, area.width, 1), buf);
        }
    }
}
