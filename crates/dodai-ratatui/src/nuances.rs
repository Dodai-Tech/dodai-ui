//! The ecosystem hook: [`TuiTheme`] as a `tui_nuances::ThemeProvider`, so every
//! widget crate that speaks `Themed` (`tui-chart`, `tui-piechart`, `tui-spinner`,
//! `tui-rating`, `tui-slider`, `tui-checkbox`) is styled from Dodai tokens with
//! one call: `widget.themed(&tui)`.

use crate::TuiTheme;
use tui_nuances::{Theme, ThemeProvider};

impl ThemeProvider for TuiTheme {
    fn nuance_theme(&self) -> Theme {
        let th = self.theme();
        let p = &th.palette;
        let c = |x| self.color(x);
        let panel = th.panel(false);
        let focused = th.panel(true);
        Theme {
            name: th.name().to_string(),
            background: Some(c(p.background)),
            surface: Some(c(p.surface)),
            foreground: c(p.text),
            muted: c(p.text_muted),
            disabled: c(p.text_disabled),
            border: c(panel.border),
            border_focused: c(focused.border),
            title: c(panel.title),
            primary: c(p.primary),
            secondary: c(p.secondary),
            accent: c(p.accent),
            success: c(p.success),
            warning: c(p.warning),
            error: c(p.danger),
            info: c(p.info),
            selection_bg: c(p.selection),
            selection_fg: c(p.text),
            highlight: c(p.focus_ring),
            palette: p.series.iter().map(|&s| c(s)).collect(),
            bull: c(p.success),
            bear: c(p.danger),
            neutral: c(p.info),
            axis: c(p.border_strong),
            grid: c(p.border),
            crosshair: c(p.focus_ring),
            tooltip_fg: c(p.text),
            tooltip_bg: c(p.overlay),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_core::{Color, Theme as DodaiTheme};
    use ratatui::style::Color as Tc;

    #[test]
    fn provider_maps_tokens_for_every_theme() {
        for th in dodai::themes::all() {
            // Explicit depth: `TuiTheme::new` reads `COLORTERM`, which CI runners do not set.
            let tui = TuiTheme::with_depth(th.clone(), crate::ColorDepth::TrueColor);
            let n = tui.nuance_theme();
            let rgb = |c: Color| Tc::Rgb(c.r, c.g, c.b);
            assert_eq!(n.name, th.name());
            assert_eq!(n.primary, rgb(th.palette.primary));
            assert_eq!(n.error, rgb(th.palette.danger));
            assert_eq!(n.palette.len(), th.palette.series.len());
            assert_eq!(n.palette[0], rgb(th.palette.series[0]));
            assert_eq!(n.grid, rgb(th.palette.border));
        }
    }

    #[test]
    fn ansi256_depth_keeps_indexed_colours() {
        let tui = TuiTheme::with_depth(DodaiTheme::default(), crate::ColorDepth::Ansi256);
        assert!(matches!(tui.nuance_theme().primary, Tc::Indexed(_)));
    }
}
