//! Theme install + [`ThemeSwitcher`] integration via a Dioxus context signal.

use freya::prelude::*;
use legendary_core::{themes, Theme, ThemeSwitcher};

/// Reactive state stored in context: the switcher plus an optional glow override.
#[derive(Clone)]
pub struct LegendaryState {
    pub switcher: ThemeSwitcher,
    /// When `Some`, replaces `effects.glow_intensity` of whichever theme is active.
    pub glow_override: Option<f32>,
}

impl LegendaryState {
    /// Start on `theme`; it is added to the registered themes if not among them.
    pub fn new(theme: Theme) -> Self {
        let mut all = themes::all();
        if !all.iter().any(|t| t.id() == theme.id()) {
            all.push(theme.clone());
        }
        let mut switcher = ThemeSwitcher::with_themes(all);
        switcher.select(theme.id());
        Self {
            switcher,
            glow_override: None,
        }
    }

    /// Active theme with the glow override applied.
    pub fn theme(&self) -> Theme {
        let t = self.switcher.current().clone();
        match self.glow_override {
            Some(g) => t.with_glow_intensity(g),
            None => t,
        }
    }
}

/// Copyable handle to the installed theme state.
#[derive(Clone, Copy)]
pub struct Legendary(Signal<LegendaryState>);

impl Legendary {
    /// Active theme (subscribes the calling component to changes).
    pub fn theme(&self) -> Theme {
        self.0.read().theme()
    }
    /// Registered themes (id, name) in switcher order.
    pub fn list(&self) -> Vec<(&'static str, &'static str)> {
        self.0
            .read()
            .switcher
            .themes()
            .iter()
            .map(|t| (t.id(), t.name()))
            .collect()
    }
    pub fn current_index(&self) -> usize {
        self.0.read().switcher.index()
    }
    pub fn next(&self) {
        let mut s = self.0;
        s.write().switcher.cycle_next();
    }
    pub fn prev(&self) {
        let mut s = self.0;
        s.write().switcher.cycle_prev();
    }
    pub fn select(&self, id: &str) -> bool {
        let mut s = self.0;
        let ok = s.write().switcher.select(id);
        ok
    }
    pub fn select_index(&self, i: usize) {
        let mut s = self.0;
        s.write().switcher.select_index(i);
    }
    /// Effective glow intensity of the active theme.
    pub fn glow_intensity(&self) -> f32 {
        self.theme().effects.glow_intensity
    }
    /// Override glow intensity (survives theme switching). Clamped to `>= 0`.
    pub fn set_glow_intensity(&self, v: f32) {
        let mut s = self.0;
        s.write().glow_override = Some(v.max(0.0));
    }
}

/// Install the theme state in context (call once at the app root).
pub fn use_init_legendary(theme: Theme) -> Legendary {
    Legendary(use_context_provider(|| {
        Signal::new(LegendaryState::new(theme))
    }))
}

/// Fetch the handle installed by [`use_init_legendary`].
pub fn use_legendary() -> Legendary {
    Legendary(use_context::<Signal<LegendaryState>>())
}

/// Active theme, subscribing the component to switches / glow changes.
pub fn use_legendary_theme() -> Theme {
    use_legendary().theme()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::testing::square;

    #[test]
    fn state_selects_given_theme_and_overrides_glow() {
        let st = LegendaryState::new(square());
        assert_eq!(st.theme().id(), "test-square");
        let mut st2 = LegendaryState::new(themes::cyberpunk::night_city());
        st2.glow_override = Some(2.0);
        assert_eq!(st2.theme().effects.glow_intensity, 2.0);
        assert!(!st2.switcher.themes().is_empty());
    }
}
