//! # dodai-ratatui
//!
//! Turns a [`dodai_core::Theme`] into Ratatui styles and widgets:
//! chamfered (cut-corner) panels, outline-to-fill buttons, tabs, badges,
//! status bars and a CRT scanline overlay — not just colours.
//!
//! ```
//! use dodai_core::Theme;
//! use dodai_ratatui::TuiTheme;
//! let tui = TuiTheme::new(Theme::default());
//! let _block = tui.block("Dashboard", true);
//! let _style = tui.selected();
//! ```

mod checkbox;
mod controls;
mod dialog;
mod input;
mod list;
mod nuances;
mod shape;
mod widgets;

pub use tui_nuances::{ThemeProvider, Themed};
/// The widget crates this adapter themes. Style any of their widgets with
/// `widget.themed(&tui)` ([`tui_nuances::Themed`]); [`TuiTheme`] is the
/// [`tui_nuances::ThemeProvider`].
pub use {tui_chart, tui_checkbox, tui_nuances, tui_piechart, tui_rating, tui_slider, tui_spinner};

pub use checkbox::{checkbox_design, Checkbox};
pub use controls::{Divider, Progress, Tag, Toggle, ToggleKind};
pub use dialog::{centered, fit, paint_background, Dialog, DialogTone};
pub use input::InputField;
pub use list::SelectList;
pub use shape::{border_set, cuts_bottom_right, CHAMFER_BORDER, ROUNDED_BORDER, SQUARE_BORDER};
pub use tui_checkbox::cyber::Design as CheckboxDesign;
pub use widgets::{dim_color, Scanlines, ThemeBadge};

use dodai_core::{ButtonVariant, Color as LColor, ControlState, ShapeKind, Theme, Tone};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders},
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

    /// Apply the theme's `uppercase_labels` token.
    pub fn label(&self, s: &str) -> String {
        if self.theme.typography.uppercase_labels {
            s.to_uppercase()
        } else {
            s.to_string()
        }
    }

    /// Themed panel; border glyphs follow `Shape::kind()` (square / rounded / chamfered corners).
    pub fn block<'a>(&self, title: impl Into<String>, focused: bool) -> Block<'a> {
        let t = self.theme.panel(focused);
        let title = self.label(&title.into());
        Block::new()
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
            ]))
            .border_set(border_set(&self.theme.shape))
    }

    /// True when the theme asks for CRT scanlines (`effects.scanlines`).
    pub fn scanlines_enabled(&self) -> bool {
        self.theme.effects.scanlines
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
            if cuts_bottom_right(&self.theme.shape) {
                Span::styled("◤", edge)
            } else {
                Span::styled(" ", body)
            },
        ])
    }

    /// Three-row chamfered slab button (chamfered, cyberpunk style).
    ///
    /// Solid variants are a filled 3-row slab with the bottom-right corner
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
        let cut = cuts_bottom_right(&self.theme.shape);
        let kind = self.theme.shape.kind();
        match t.background {
            Some(bg) => {
                let fill = Style::new().bg(self.color(bg));
                let cut_style = Style::new().fg(self.color(bg));
                Text::from(vec![
                    Line::from(Span::styled(" ".repeat(w), fill)),
                    Line::from(Span::styled(format!("{label:^w$}"), body)),
                    Line::from(vec![
                        Span::styled(" ".repeat(w - 1), fill),
                        if cut {
                            Span::styled("◤", cut_style)
                        } else {
                            Span::styled(" ", fill)
                        },
                    ]),
                ])
            }
            None => {
                let edge_c = self.color(t.border.unwrap_or(t.foreground));
                let raised = self.color(self.theme.palette.surface_raised);
                let edge = Style::new().fg(edge_c).bg(raised);
                let cut_style = Style::new().fg(edge_c);
                let line = |l: &str, mid: &str, r: &str, rs: Style| {
                    Line::from(vec![
                        Span::styled(l.to_string(), edge),
                        Span::styled(mid.to_string(), edge),
                        Span::styled(r.to_string(), rs),
                    ])
                };
                // Rounded themes use box-drawing arcs; others quadrant blocks.
                let (tl, th, tr, bl, bh, br, ml, mr) = if kind == ShapeKind::Rounded {
                    ("╭", "─", "╮", "╰", "─", "╯", "│", "│")
                } else {
                    ("▛", "▀", "▜", "▙", "▄", "▟", "▌", "▐")
                };
                let (br, brs) = if cut { ("◤", cut_style) } else { (br, edge) };
                Text::from(vec![
                    line(tl, &th.repeat(inner), tr, edge),
                    Line::from(vec![
                        Span::styled(ml, edge),
                        Span::styled(format!("{label:^inner$}"), body.bg(raised)),
                        Span::styled(mr, edge),
                    ]),
                    line(bl, &bh.repeat(inner), br, brs),
                ])
            }
        }
    }

    /// Tab strip. Active tab is a solid chamfered ` LABEL ◤`; inactive tabs are
    /// muted, same width, so nothing shifts when you switch.
    pub fn tabs<'a>(&self, labels: &[&str], active: usize) -> Line<'a> {
        self.tabs_editable(labels, active, false, false)
    }

    /// [`tabs`](Self::tabs) with editing: `closable` puts a `×` after every label, `addable` a
    /// trailing `+`. Terminals have no pointer routing here, so map a mouse column to the
    /// part that was hit with [`tab_hit`].
    pub fn tabs_editable<'a>(
        &self,
        labels: &[&str],
        active: usize,
        closable: bool,
        addable: bool,
    ) -> Line<'a> {
        let mut spans = Vec::new();
        for (i, l) in labels.iter().enumerate() {
            let text = format!(" {} ", self.label(l));
            if i == active {
                let hl = self.highlight();
                let edge = Style::new().fg(hl.bg.unwrap_or_default());
                spans.push(Span::styled(" ", hl));
                spans.push(Span::styled(text, hl));
                if cuts_bottom_right(&self.theme.shape) {
                    spans.push(Span::styled("◤", edge));
                } else {
                    spans.push(Span::styled(" ", hl));
                }
            } else {
                spans.push(Span::styled(format!(" {text} "), self.muted()));
            }
            if closable {
                spans.push(Span::styled("× ", self.muted()));
            }
            spans.push(Span::raw(" "));
        }
        if addable {
            spans.push(Span::styled(" + ", self.muted()));
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
        InputField::new(self, value)
            .placeholder(placeholder)
            .focused(focused)
            .line()
    }
}

#[cfg(test)]
mod component_tests;
#[cfg(test)]
mod tests {
    use super::*;

    fn tui() -> TuiTheme {
        TuiTheme::with_depth(
            dodai_core::themes::cyberpunk::yellow(),
            ColorDepth::TrueColor,
        )
    }

    #[test]
    fn truecolor_maps_exactly() {
        let t = tui();
        assert_eq!(t.color(LColor::hex(0xfcee0a)), Color::Rgb(0xfc, 0xee, 0x0a));
    }

    #[test]
    fn ansi256_is_indexed() {
        let t = TuiTheme::with_depth(dodai_core::themes::cyberpunk::yellow(), ColorDepth::Ansi256);
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
        let tui = TuiTheme::new(dodai_core::themes::cyberpunk::yellow());
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

/// What a column of a [`TuiTheme::tabs_editable`] strip lies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabHit {
    /// The body of tab `i`.
    Select(usize),
    /// The `×` of tab `i`.
    Close(usize),
    /// The trailing `+`.
    Add,
}

/// Which part of a strip drawn by `tabs_editable(labels, _, closable, addable)` covers the
/// 0-based `column` (relative to the strip's left edge); `None` over a gap.
pub fn tab_hit(labels: &[&str], closable: bool, addable: bool, column: u16) -> Option<TabHit> {
    let mut x = 0usize;
    let col = column as usize;
    for (i, l) in labels.iter().enumerate() {
        // " {" l "} " plus the `ˍ` margin of the active tab or the muted padding: l + 4 cells.
        let body = l.chars().count() + 4;
        if col < x + body {
            return Some(TabHit::Select(i));
        }
        x += body;
        if closable {
            if col < x + 2 {
                return Some(TabHit::Close(i));
            }
            x += 2;
        }
        x += 1;
    }
    (addable && (x..x + 3).contains(&col)).then_some(TabHit::Add)
}

/// The tab a key press moves to: the arrows, Home and End through `n` tabs from `active`,
/// skipping tabs `enabled` rejects (the shared roving rule of `dodai-core`). `None`: not a
/// navigation key, or nothing enabled lies that way.
pub fn tab_key(
    code: ratatui::crossterm::event::KeyCode,
    active: usize,
    n: usize,
    enabled: impl Fn(usize) -> bool,
) -> Option<usize> {
    use ratatui::crossterm::event::KeyCode;
    let key = match code {
        KeyCode::Left => "left",
        KeyCode::Right => "right",
        KeyCode::Home => "home",
        KeyCode::End => "end",
        _ => return None,
    };
    dodai_core::roving::next(
        key,
        dodai_core::roving::Axis::Horizontal,
        active,
        n,
        enabled,
    )
}
