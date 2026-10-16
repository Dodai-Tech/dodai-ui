//! Token-driven small controls: [`Tag`], [`Toggle`], [`Progress`], [`Divider`].

use crate::{shape::cuts_bottom_right, TuiTheme};
use dodai_core::{ControlState, ShapeKind, Tone};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

/// Tag / badge: solid tone fill, or outlined (tinted fill + edge glyphs).
pub struct Tag<'a> {
    theme: &'a TuiTheme,
    text: String,
    tone: Tone,
    solid: bool,
}

impl<'a> Tag<'a> {
    /// Solid tag of `tone`.
    pub fn new(theme: &'a TuiTheme, text: &str, tone: Tone) -> Self {
        Self {
            theme,
            text: theme.label(text),
            tone,
            solid: true,
        }
    }

    /// Solid (`true`) or outline (`false`).
    pub fn solid(mut self, solid: bool) -> Self {
        self.solid = solid;
        self
    }

    /// Outline shortcut.
    pub fn outline(self) -> Self {
        self.solid(false)
    }

    /// Cell width.
    pub fn width(&self) -> u16 {
        self.line().width() as u16
    }

    /// As a [`Line`] for embedding in other text.
    pub fn line(&self) -> Line<'static> {
        let th = self.theme;
        let t = th.theme().tag(self.tone, self.solid);
        let fg = th.color(t.foreground);
        let mut body = Style::new().fg(fg).add_modifier(Modifier::BOLD);
        if let Some(bg) = t.background {
            body = body.bg(th.color(bg));
        }
        let label = Span::styled(format!(" {} ", self.text), body);
        let Some(b) = t.border else {
            return Line::from(label);
        };
        let edge = Style::new().fg(th.color(b)).bg(body.bg.unwrap_or_default());
        let (l, r) = match th.theme().shape.kind() {
            ShapeKind::Square => ("▌", "▐"),
            ShapeKind::Rounded => ("(", ")"),
            ShapeKind::Chamfered if cuts_bottom_right(&th.theme().shape) => ("▌", "◤"),
            ShapeKind::Chamfered => ("▌", "▐"),
        };
        let r_style = if r == "◤" {
            Style::new().fg(th.color(b))
        } else {
            edge
        };
        Line::from(vec![Span::styled(l, edge), label, Span::styled(r, r_style)])
    }
}

impl Widget for Tag<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.line().render(area, buf);
    }
}

/// Which glyph family a [`Toggle`] uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleKind {
    /// `[x]` / `[ ]`
    Checkbox,
    /// `(●)` / `( )`
    Radio,
    /// `[━●]` / `[●━]`-style switch.
    Switch,
}

/// Checkbox / radio / switch styled from the toggle tokens.
pub struct Toggle<'a> {
    theme: &'a TuiTheme,
    kind: ToggleKind,
    on: bool,
    state: ControlState,
    label: String,
}

impl<'a> Toggle<'a> {
    pub fn new(theme: &'a TuiTheme, kind: ToggleKind, on: bool, label: &str) -> Self {
        Self {
            theme,
            kind,
            on,
            state: ControlState::Idle,
            label: theme.label(label),
        }
    }

    /// Interaction state (focus/hover/disabled).
    pub fn state(mut self, state: ControlState) -> Self {
        self.state = state;
        self
    }

    /// The glyph part only, e.g. `[x]`.
    pub fn glyph(&self) -> &'static str {
        match (self.kind, self.on) {
            (ToggleKind::Checkbox, true) => "[x]",
            (ToggleKind::Checkbox, false) => "[ ]",
            (ToggleKind::Radio, true) => "(●)",
            (ToggleKind::Radio, false) => "( )",
            (ToggleKind::Switch, true) => "[ ●]",
            (ToggleKind::Switch, false) => "[● ]",
        }
    }

    /// As a [`Line`].
    pub fn line(&self) -> Line<'static> {
        let th = self.theme;
        let t = th.theme().toggle(self.on, self.state);
        let g = Style::new()
            .fg(th.color(t.thumb))
            .bg(th.color(t.track))
            .add_modifier(Modifier::BOLD);
        let text = if self.state == ControlState::Disabled {
            th.disabled()
        } else {
            Style::new().fg(th.color(th.theme().palette.text))
        };
        Line::from(vec![
            Span::styled(self.glyph(), g),
            Span::styled(format!(" {}", self.label), text),
        ])
    }
}

impl Widget for Toggle<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.line().render(area, buf);
    }
}

/// Eighth-block partials, index = eighths filled.
const EIGHTHS: [&str; 9] = [" ", "▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"];

/// Horizontal progress bar with sub-cell precision and optional `NN%` label.
pub struct Progress<'a> {
    theme: &'a TuiTheme,
    ratio: f64,
    tone: Option<Tone>,
    label: Option<String>,
}

impl<'a> Progress<'a> {
    /// `ratio` is clamped to `0.0..=1.0`.
    pub fn new(theme: &'a TuiTheme, ratio: f64) -> Self {
        Self {
            theme,
            ratio: ratio.clamp(0.0, 1.0),
            tone: None,
            label: None,
        }
    }

    /// Status tone (default: brand colour).
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }

    /// Text shown right of the bar.
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(self.theme.label(label));
        self
    }

    /// Show `NN%` as the label.
    pub fn percent(mut self) -> Self {
        self.label = Some(format!("{:>3}%", (self.ratio * 100.0).round() as u32));
        self
    }

    /// Bar glyphs for a `width`-cell bar (no label).
    pub fn bar_text(&self, width: u16) -> String {
        let total = (self.ratio * f64::from(width) * 8.0).round() as usize;
        let (full, part) = (total / 8, total % 8);
        (0..width as usize)
            .map(|i| match i.cmp(&full) {
                std::cmp::Ordering::Less => EIGHTHS[8],
                std::cmp::Ordering::Equal => EIGHTHS[part],
                std::cmp::Ordering::Greater => EIGHTHS[0],
            })
            .collect()
    }
}

impl Widget for Progress<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let th = self.theme;
        let t = th.theme().progress(self.tone);
        let lab_w = self
            .label
            .as_ref()
            .map_or(0, |l| l.chars().count() as u16 + 1);
        let bar_w = area.width.saturating_sub(lab_w).max(1).min(area.width);
        let style = Style::new().fg(th.color(t.fill)).bg(th.color(t.track));
        let text = self.bar_text(bar_w);
        Line::from(Span::styled(text, style)).render(Rect::new(area.x, area.y, bar_w, 1), buf);
        if let (Some(l), true) = (&self.label, area.width > bar_w) {
            let r = Rect::new(area.x + bar_w, area.y, area.width - bar_w, 1);
            Line::from(Span::styled(
                format!(" {l}"),
                Style::new().fg(th.color(th.theme().palette.text_muted)),
            ))
            .render(r, buf);
        }
    }
}

/// Horizontal rule with optional centred label, in the divider colour.
pub struct Divider<'a> {
    theme: &'a TuiTheme,
    label: Option<String>,
}

impl<'a> Divider<'a> {
    pub fn new(theme: &'a TuiTheme) -> Self {
        Self { theme, label: None }
    }

    /// Centred label.
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(self.theme.label(label));
        self
    }
}

impl Widget for Divider<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let th = self.theme;
        let st = Style::new().fg(th.color(th.theme().divider()));
        let w = area.width as usize;
        let line = match &self.label {
            Some(l) if l.chars().count() + 4 <= w => {
                let rest = w - l.chars().count() - 2;
                let left = rest / 2;
                Line::from(vec![
                    Span::styled("─".repeat(left), st),
                    Span::styled(format!(" {l} "), th.muted()),
                    Span::styled("─".repeat(rest - left), st),
                ])
            }
            _ => Line::from(Span::styled("─".repeat(w), st)),
        };
        line.render(Rect::new(area.x, area.y, area.width, 1), buf);
    }
}
