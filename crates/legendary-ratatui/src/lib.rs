//! # legendary-ratatui
//!
//! Turns a [`legendary_core::Theme`] into Ratatui styles and widgets:
//! chamfered (cut-corner) panels, outline-to-fill buttons, tabs, badges,
//! status bars and a CRT scanline overlay — not just colours.
//!
//! ```
//! use legendary_core::Theme;
//! use legendary_ratatui::TuiTheme;
//! let tui = TuiTheme::new(Theme::default());
//! let _block = tui.block("Dashboard", true);
//! let _style = tui.selected();
//! ```

mod widgets;

pub use widgets::{Scanlines, ThemeBadge};

use legendary_core::{ButtonVariant, Color as LColor, ControlState, Theme, Tone};
use ratatui::{
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders},
};

/// How many colours the terminal can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorDepth {
    /// 24-bit colour.
    #[default]
    TrueColor,
    /// xterm-256 fallback (e.g. macOS Terminal.app, tmux without `Tc`).
    Ansi256,
}

impl ColorDepth {
    /// Detect from `COLORTERM` (`truecolor` / `24bit`); otherwise 256-colour.
    pub fn detect() -> Self {
        match std::env::var("COLORTERM").as_deref() {
            Ok("truecolor") | Ok("24bit") => Self::TrueColor,
            _ => Self::Ansi256,
        }
    }
}

/// Cut-corner border: only the bottom-right corner is cut (`╱`), like cyberpunk.net buttons.
pub const CHAMFER_BORDER: border::Set<'static> = border::Set {
    top_left: "┌",
    top_right: "┐",
    bottom_left: "└",
    bottom_right: "╱",
    vertical_left: "│",
    vertical_right: "│",
    horizontal_top: "─",
    horizontal_bottom: "─",
};

/// Ratatui view of a [`Theme`].
#[derive(Debug, Clone)]
pub struct TuiTheme {
    theme: Theme,
    depth: ColorDepth,
}

impl TuiTheme {
    /// Uses [`ColorDepth::detect`].
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            depth: ColorDepth::detect(),
        }
    }

    /// Force a colour depth.
    pub fn with_depth(theme: Theme, depth: ColorDepth) -> Self {
        Self { theme, depth }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Swap the underlying theme (e.g. after `ThemeSwitcher::next`).
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Convert a token colour; translucent colours are blended onto the background.
    pub fn color(&self, c: LColor) -> Color {
        let c = if c.a < 255 {
            self.theme
                .palette
                .background
                .mix(LColor::rgba(c.r, c.g, c.b, 255), c.a as f32 / 255.0)
        } else {
            c
        };
        match self.depth {
            ColorDepth::TrueColor => Color::Rgb(c.r, c.g, c.b),
            ColorDepth::Ansi256 => Color::Indexed(c.to_ansi256()),
        }
    }

    // ── Text styles ───────────────────────────────────────────────────────

    /// Default text on the app background — use to paint the whole frame.
    pub fn base(&self) -> Style {
        let p = &self.theme.palette;
        Style::new()
            .fg(self.color(p.text))
            .bg(self.color(p.background))
    }

    pub fn surface(&self) -> Style {
        let p = &self.theme.palette;
        Style::new()
            .fg(self.color(p.text))
            .bg(self.color(p.surface))
    }

    pub fn muted(&self) -> Style {
        Style::new().fg(self.color(self.theme.palette.text_muted))
    }

    pub fn disabled(&self) -> Style {
        Style::new().fg(self.color(self.theme.palette.text_disabled))
    }

    /// Bold primary-coloured heading text.
    pub fn title(&self) -> Style {
        Style::new()
            .fg(self.color(self.theme.palette.primary))
            .add_modifier(Modifier::BOLD)
    }

    pub fn link(&self) -> Style {
        Style::new()
            .fg(self.color(self.theme.palette.link))
            .add_modifier(Modifier::UNDERLINED)
    }

    /// Selected row / list item: translucent primary wash, bold.
    pub fn selected(&self) -> Style {
        let p = &self.theme.palette;
        Style::new()
            .fg(self.color(p.primary))
            .bg(self.color(p.selection))
            .add_modifier(Modifier::BOLD)
    }

    /// Hard highlight: primary background, inverted text (table cursors, active tab).
    pub fn highlight(&self) -> Style {
        let p = &self.theme.palette;
        Style::new()
            .fg(self.color(p.on_primary))
            .bg(self.color(p.primary))
            .add_modifier(Modifier::BOLD)
    }

    /// Status colour.
    pub fn tone(&self, tone: Tone) -> Style {
        Style::new().fg(self.color(self.theme.palette.tone(tone)))
    }

    /// Categorical series colour (chart lines, graph lanes).
    pub fn series(&self, i: usize) -> Style {
        Style::new().fg(self.color(self.theme.palette.series_at(i)))
    }

    /// Border style for focused / unfocused panels.
    pub fn border(&self, focused: bool) -> Style {
        let t = self.theme.panel(focused);
        Style::new().fg(self.color(t.border))
    }

    /// Cursor / caret style for inputs.
    pub fn cursor(&self) -> Style {
        let p = &self.theme.palette;
        Style::new()
            .fg(self.color(p.background))
            .bg(self.color(p.secondary))
    }

    // ── Components ────────────────────────────────────────────────────────

    fn label(&self, s: &str) -> String {
        if self.theme.typography.uppercase_labels {
            s.to_uppercase()
        } else {
            s.to_string()
        }
    }

    /// Themed panel. Chamfered corners when the theme's `shape.chamfer > 0`.
    pub fn block<'a>(&self, title: impl Into<String>, focused: bool) -> Block<'a> {
        let t = self.theme.panel(focused);
        let title = self.label(&title.into());
        let b = Block::new()
            .borders(Borders::ALL)
            .style(Style::new().bg(self.color(t.background)))
            .border_style(Style::new().fg(self.color(t.border)))
            .title(Line::from(vec![
                Span::styled(" ", Style::new()),
                Span::styled(
                    title,
                    Style::new()
                        .fg(self.color(t.title))
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ", Style::new()),
            ]));
        if self.theme.shape.chamfer > 0.0 {
            b.border_set(CHAMFER_BORDER)
        } else {
            b.border_type(BorderType::Plain)
        }
    }

    /// Resolved button style (fg/bg/border-as-bg) for a state.
    pub fn button_style(&self, variant: ButtonVariant, state: ControlState) -> Style {
        let t = self.theme.button(variant, state);
        let mut s = Style::new()
            .fg(self.color(t.foreground))
            .add_modifier(Modifier::BOLD);
        if let Some(bg) = t.background {
            s = s.bg(self.color(bg));
        } else if let Some(border) = t.border {
            s = s.fg(self.color(border));
        }
        s
    }

    /// Single-line chamfered button `▌ LABEL ◤` (square left edge, bottom-right corner cut).
    ///
    /// Solid variants fill the whole cell run; outline variants sit on a raised
    /// surface with a border-coloured label and edges so they still read as buttons.
    pub fn button<'a>(&self, label: &str, variant: ButtonVariant, state: ControlState) -> Line<'a> {
        let t = self.theme.button(variant, state);
        let label = self.label(label);
        let edge_color = t.background.or(t.border).unwrap_or(t.foreground);
        let mut edge = Style::new().fg(self.color(edge_color));
        let mut body = self.button_style(variant, state);
        if t.background.is_none() {
            let raised = self.color(self.theme.palette.surface_raised);
            body = body.bg(raised);
            edge = edge.bg(raised);
        }
        Line::from(vec![
            if t.background.is_some() {
                Span::styled(" ", body)
            } else {
                Span::styled("▌", edge)
            },
            Span::styled(format!(" {label} "), body),
            Span::styled("◤", edge),
        ])
    }

    /// Three-row chamfered slab button (closest a terminal gets to cyberpunk.net).
    ///
    /// Solid variants are a filled slab (`▄` / `▀` caps) with the bottom-right corner
    /// cut via `◤`; outline variants are a box whose bottom-right corner is `╱`.
    /// Width = label + 6 cells; render into a 3-row area.
    pub fn button_block<'a>(
        &self,
        label: &str,
        variant: ButtonVariant,
        state: ControlState,
    ) -> Text<'a> {
        let t = self.theme.button(variant, state);
        let label = self.label(label);
        let w = label.chars().count() + 6;
        let inner = w - 2;
        let body = self.button_style(variant, state);
        match t.background {
            Some(bg) => {
                let cap = Style::new().fg(self.color(bg));
                Text::from(vec![
                    Line::styled("▄".repeat(w), cap),
                    Line::from(vec![
                        Span::styled(format!("{label:^width$}", width = w - 1), body),
                        Span::styled("◤", cap),
                    ]),
                    Line::styled(format!("{}  ", "▀".repeat(w - 2)), cap),
                ])
            }
            None => {
                let edge_c = t.border.unwrap_or(t.foreground);
                let edge = Style::new().fg(self.color(edge_c));
                Text::from(vec![
                    Line::styled(format!("┌{}┐", "─".repeat(inner)), edge),
                    Line::from(vec![
                        Span::styled("│", edge),
                        Span::styled(format!("{label:^inner$}"), body),
                        Span::styled("│", edge),
                    ]),
                    Line::styled(format!("└{}╱", "─".repeat(inner)), edge),
                ])
            }
        }
    }

    /// Tab strip. Active tab is a solid chamfered ` LABEL ◤`; inactive tabs are
    /// muted, same width, so nothing shifts when you switch.
    pub fn tabs<'a>(&self, labels: &[&str], active: usize) -> Line<'a> {
        let mut spans = Vec::new();
        for (i, l) in labels.iter().enumerate() {
            let text = format!(" {} ", self.label(l));
            if i == active {
                let hl = self.highlight();
                let edge = Style::new().fg(hl.bg.unwrap_or_default());
                spans.push(Span::styled(" ", hl));
                spans.push(Span::styled(text, hl));
                spans.push(Span::styled("◤", edge));
            } else {
                spans.push(Span::styled(format!(" {text} "), self.muted()));
            }
            spans.push(Span::raw(" "));
        }
        Line::from(spans)
    }

    /// Small status badge, e.g. `▌ERROR▐`.
    pub fn badge<'a>(&self, text: &str, tone: Tone) -> Span<'a> {
        let c = self.theme.palette.tone(tone);
        Span::styled(
            format!(" {} ", self.label(text)),
            Style::new()
                .fg(self.color(self.theme.on(c)))
                .bg(self.color(c))
                .add_modifier(Modifier::BOLD),
        )
    }

    /// Footer / status line: `left` … `right`, padded to `width`.
    pub fn status_bar<'a>(&self, left: &str, right: &str, width: u16) -> Line<'a> {
        let p = &self.theme.palette;
        let style = Style::new()
            .fg(self.color(p.text_muted))
            .bg(self.color(p.surface_raised));
        let used = left.chars().count() + right.chars().count() + 2;
        let pad = (width as usize).saturating_sub(used);
        Line::from(vec![
            Span::styled(format!(" {left}"), style),
            Span::styled(" ".repeat(pad), style),
            Span::styled(format!("{right} "), style),
        ])
    }

    /// Text-input line with placeholder and block cursor when focused.
    pub fn input<'a>(&self, value: &str, placeholder: &str, focused: bool) -> Line<'a> {
        let t = self.theme.input(focused);
        let mut spans = vec![Span::styled("▸ ", Style::new().fg(self.color(t.border)))];
        if value.is_empty() {
            spans.push(Span::styled(
                placeholder.to_string(),
                Style::new().fg(self.color(t.placeholder)),
            ));
        } else {
            spans.push(Span::styled(
                value.to_string(),
                Style::new().fg(self.color(t.text)),
            ));
        }
        if focused {
            spans.push(Span::styled(" ", self.cursor()));
        }
        Line::from(spans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tui() -> TuiTheme {
        TuiTheme::with_depth(Theme::default(), ColorDepth::TrueColor)
    }

    #[test]
    fn truecolor_maps_exactly() {
        let t = tui();
        assert_eq!(t.color(LColor::hex(0xfcee0a)), Color::Rgb(0xfc, 0xee, 0x0a));
    }

    #[test]
    fn ansi256_is_indexed() {
        let t = TuiTheme::with_depth(Theme::default(), ColorDepth::Ansi256);
        assert!(matches!(t.color(LColor::hex(0xfcee0a)), Color::Indexed(_)));
    }

    #[test]
    fn translucent_blends_onto_background() {
        let t = tui();
        let c = t.color(LColor::hex(0xffffff).with_alpha(0.0));
        assert_eq!(c, Color::Rgb(0, 0, 0));
    }

    #[test]
    fn button_idle_outline_is_transparent_lit_is_solid() {
        let t = tui();
        let idle = t.button_style(ButtonVariant::Outline, ControlState::Idle);
        let hot = t.button_style(ButtonVariant::Outline, ControlState::Hovered);
        assert_eq!(idle.bg, None);
        assert!(hot.bg.is_some());
    }

    #[test]
    fn button_block_is_three_equal_rows() {
        let tui = TuiTheme::new(legendary_core::themes::cyberpunk::night_city());
        for v in [ButtonVariant::Primary, ButtonVariant::Outline] {
            let b = tui.button_block("Go", v, ControlState::Idle);
            assert_eq!(b.lines.len(), 3);
            let w: Vec<usize> = b.lines.iter().map(Line::width).collect();
            assert_eq!(w[0], w[1], "{v:?}");
            assert_eq!(w[0], w[2], "{v:?}");
            assert!(b.lines[1].to_string().contains("GO"));
        }
    }

    #[test]
    fn button_line_has_chamfer_edges_and_uppercase() {
        let l = tui().button("ok", ButtonVariant::Primary, ControlState::Idle);
        let s: String = l.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(s, "  OK ◤");
    }

    #[test]
    fn status_bar_fills_width() {
        let l = tui().status_bar("a", "b", 20);
        let s: String = l.spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(s.chars().count(), 20);
    }
}
