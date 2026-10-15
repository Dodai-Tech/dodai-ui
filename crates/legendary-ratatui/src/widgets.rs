//! Stateless Ratatui widgets.

use crate::TuiTheme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    text::{Line, Span},
    widgets::Widget,
};

/// CRT scanline overlay: darkens every other row. Render **last**.
/// No-op when the theme has `effects.scanlines == false`.
pub struct Scanlines<'a> {
    theme: &'a TuiTheme,
    strength: f32,
}

impl<'a> Scanlines<'a> {
    pub fn new(theme: &'a TuiTheme) -> Self {
        Self {
            theme,
            strength: 0.25,
        }
    }

    /// Darkening amount `0.0..=1.0`.
    pub fn strength(mut self, s: f32) -> Self {
        self.strength = s.clamp(0.0, 1.0);
        self
    }
}

fn dim(c: Color, k: f32) -> Color {
    match c {
        Color::Rgb(r, g, b) => {
            let f = |v: u8| (v as f32 * (1.0 - k)).round() as u8;
            Color::Rgb(f(r), f(g), f(b))
        }
        other => other,
    }
}

impl Widget for Scanlines<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.theme.theme().effects.scanlines {
            return;
        }
        for y in (area.top()..area.bottom()).filter(|y| y % 2 == 1) {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_fg(dim(cell.fg, self.strength));
                }
            }
        }
    }
}

/// `◢ NIGHT CITY ◤  t: next theme` badge for headers/footers.
pub struct ThemeBadge<'a> {
    theme: &'a TuiTheme,
    hint: Option<&'a str>,
}

impl<'a> ThemeBadge<'a> {
    pub fn new(theme: &'a TuiTheme) -> Self {
        Self {
            theme,
            hint: Some("t: next theme"),
        }
    }

    /// Replace (or remove with `None`) the key hint.
    pub fn hint(mut self, hint: Option<&'a str>) -> Self {
        self.hint = hint;
        self
    }

    /// As a `Line`, for embedding in titles.
    pub fn line(&self) -> Line<'static> {
        let t = self.theme;
        let name = t.theme().name().to_uppercase();
        let edge = ratatui::style::Style::new().fg(t.color(t.theme().palette.primary));
        let mut spans = vec![
            Span::styled("◢", edge),
            Span::styled(format!(" {name} "), t.highlight()),
            Span::styled("◤", edge),
        ];
        if let Some(h) = self.hint {
            spans.push(Span::styled(format!("  {h}"), t.muted()));
        }
        Line::from(spans)
    }
}

impl Widget for ThemeBadge<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.line().render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorDepth;
    use legendary_core::Theme;

    #[test]
    fn scanlines_dim_odd_rows_only() {
        let t = TuiTheme::with_depth(Theme::default(), ColorDepth::TrueColor);
        let area = Rect::new(0, 0, 2, 2);
        let mut buf = Buffer::empty(area);
        for y in 0..2 {
            for x in 0..2 {
                buf[(x, y)].set_fg(Color::Rgb(200, 200, 200));
            }
        }
        Scanlines::new(&t).strength(0.5).render(area, &mut buf);
        assert_eq!(buf[(0, 0)].fg, Color::Rgb(200, 200, 200));
        assert_eq!(buf[(0, 1)].fg, Color::Rgb(100, 100, 100));
    }

    #[test]
    fn badge_shows_name() {
        let t = TuiTheme::with_depth(Theme::default(), ColorDepth::TrueColor);
        let s: String = ThemeBadge::new(&t)
            .line()
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect();
        assert!(s.contains("NIGHT CITY"));
    }
}
