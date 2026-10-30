//! Runtime theme switching shared by all front-ends.

use crate::Theme;

/// Holds the available themes and the active one. Cycling wraps around.
#[derive(Debug, Clone)]
pub struct ThemeSwitcher {
    themes: Vec<Theme>,
    index: usize,
}

impl ThemeSwitcher {
    /// Switcher over a custom list; an empty list gets the neutral theme, so there is always
    /// a current one.
    pub fn with_themes(themes: Vec<Theme>) -> Self {
        let themes = if themes.is_empty() {
            vec![crate::themes::neutral()]
        } else {
            themes
        };
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

    /// Distinct design-system families, in registration order.
    pub fn families(&self) -> Vec<&'static str> {
        let mut out: Vec<&'static str> = Vec::new();
        for t in &self.themes {
            if !out.contains(&t.meta.family) {
                out.push(t.meta.family);
            }
        }
        out
    }

    /// Family of the active theme.
    pub fn current_family(&self) -> &'static str {
        self.current().meta.family
    }

    /// Indices (into [`themes`](Self::themes)) of the variants in `family`.
    pub fn family_indices(&self, family: &str) -> Vec<usize> {
        (0..self.themes.len())
            .filter(|&i| self.themes[i].meta.family == family)
            .collect()
    }

    /// Switch to the first variant of `family`; `false` if unknown.
    pub fn select_family(&mut self, family: &str) -> bool {
        match self.family_indices(family).first() {
            Some(&i) => {
                self.index = i;
                true
            }
            None => false,
        }
    }

    /// Index of the same theme in the opposite light/dark mode: same family, and the id
    /// (minus a `-dark` / `-light` suffix) matches when there is one, else the family's first
    /// theme of that mode.
    pub fn counterpart(&self) -> Option<usize> {
        let cur = self.current();
        let base = |id: &str| {
            id.strip_suffix("-dark")
                .or_else(|| id.strip_suffix("-light"))
                .unwrap_or(id)
                .to_owned()
        };
        let want = base(cur.meta.id);
        let others: Vec<usize> = self
            .family_indices(cur.meta.family)
            .into_iter()
            .filter(|&i| self.themes[i].meta.mode != cur.meta.mode)
            .collect();
        others
            .iter()
            .copied()
            .find(|&i| base(self.themes[i].meta.id) == want)
            .or_else(|| others.first().copied())
    }

    /// Switch to the opposite light/dark mode of the current theme; `false` if it has none.
    pub fn toggle_mode(&mut self) -> bool {
        match self.counterpart() {
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
        Self::with_themes(vec![crate::themes::neutral()])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn an_empty_list_falls_back_to_the_neutral_theme() {
        let s = super::ThemeSwitcher::with_themes(Vec::new());
        assert_eq!(s.themes().len(), 1);
        assert_eq!(s.current().id(), crate::themes::neutral().id());
    }

    use super::*;

    fn fam(id: &'static str, family: &'static str) -> Theme {
        let mut t = crate::themes::neutral();
        t.meta.id = id;
        t.meta.family = family;
        t
    }

    fn sample() -> ThemeSwitcher {
        ThemeSwitcher::with_themes(vec![fam("a1", "a"), fam("a2", "a"), fam("b1", "b")])
    }

    #[test]
    fn toggle_mode_finds_the_matching_sibling() {
        let mut s = crate::switcher();
        for i in 0..s.themes().len() {
            s.select_index(i);
            let (id, mode, family) = (
                s.current().meta.id,
                s.current().meta.mode,
                s.current_family(),
            );
            if s.toggle_mode() {
                assert_ne!(s.current().meta.mode, mode, "{id}");
                assert_eq!(s.current_family(), family, "{id}");
                assert!(s.toggle_mode());
                assert_eq!(s.current().meta.mode, mode, "{id} comes back to its mode");
            }
        }
        if s.select("dodai-forest") {
            s.toggle_mode();
            assert_eq!(s.current().meta.id, "dodai-forest-dark");
        }
    }

    #[test]
    fn families_group_variants() {
        let mut s = sample();
        assert_eq!(s.families(), vec!["a", "b"]);
        assert_eq!(s.family_indices("a"), vec![0, 1]);
        assert!(s.select_family("b"));
        assert_eq!(s.current().id(), "b1");
        assert!(!s.select_family("nope"));
    }

    #[test]
    fn cycles_and_wraps() {
        let mut s = sample();
        for _ in 0..3 {
            s.cycle_next();
        }
        assert_eq!(s.current().id(), "a1");
        s.cycle_prev();
        assert_eq!(s.index(), 2);
    }

    #[test]
    fn select_by_id_and_clamped_index() {
        let mut s = sample();
        assert!(s.select("a2"));
        assert!(!s.select("zzz"));
        assert_eq!(s.current().id(), "a2");
        s.select_index(99);
        assert_eq!(s.index(), 2);
    }

    #[test]
    fn default_has_the_neutral_theme() {
        assert_eq!(ThemeSwitcher::default().current().id(), "default");
    }
}
