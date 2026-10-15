//! Runtime theme switching shared by all front-ends.

use crate::{themes, Theme};

/// Holds the available themes and the active one. Cycling wraps around.
#[derive(Debug, Clone)]
pub struct ThemeSwitcher {
    themes: Vec<Theme>,
    index: usize,
}

impl ThemeSwitcher {
    /// Switcher over every built-in theme, starting at the first.
    pub fn new() -> Self {
        Self::with_themes(themes::all())
    }

    /// Switcher over a custom list. Panics if `themes` is empty.
    pub fn with_themes(themes: Vec<Theme>) -> Self {
        assert!(!themes.is_empty(), "ThemeSwitcher needs at least one theme");
        Self { themes, index: 0 }
    }

    /// The active theme.
    pub fn current(&self) -> &Theme {
        &self.themes[self.index]
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn themes(&self) -> &[Theme] {
        &self.themes
    }

    /// Advance to the next theme (wraps) and return it.
    pub fn cycle_next(&mut self) -> &Theme {
        self.index = (self.index + 1) % self.themes.len();
        self.current()
    }

    /// Go to the previous theme (wraps) and return it.
    pub fn cycle_prev(&mut self) -> &Theme {
        self.index = (self.index + self.themes.len() - 1) % self.themes.len();
        self.current()
    }

    /// Select by id; returns `false` (and keeps the current theme) if unknown.
    pub fn select(&mut self, id: &str) -> bool {
        match self.themes.iter().position(|t| t.meta.id == id) {
            Some(i) => {
                self.index = i;
                true
            }
            None => false,
        }
    }

    /// Select by index, clamped.
    pub fn select_index(&mut self, i: usize) {
        self.index = i.min(self.themes.len() - 1);
    }
}

impl Default for ThemeSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_and_wraps() {
        let mut s = ThemeSwitcher::new();
        let n = s.themes().len();
        let first = s.current().id();
        for _ in 0..n {
            s.cycle_next();
        }
        assert_eq!(s.current().id(), first);
        s.cycle_prev();
        assert_eq!(s.index(), n - 1);
    }

    #[test]
    fn select_by_id() {
        let mut s = ThemeSwitcher::new();
        assert!(s.select("cyberpunk-netrunner"));
        assert_eq!(s.current().id(), "cyberpunk-netrunner");
        assert!(!s.select("nope"));
        assert_eq!(s.current().id(), "cyberpunk-netrunner");
    }
}
