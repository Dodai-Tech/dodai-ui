//! The showcase's state and the one `update` every toolkit routes its events through.

use std::collections::HashMap;

use dodai_core::{ControlColour, Mode, Theme, ThemeSwitcher};

use crate::catalog::{self, Section, CATEGORIES};
use crate::node::Bind;

/// A stored demo value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Switch, checkbox, chip.
    Bool(bool),
    /// Slider.
    Num(f32),
    /// Selected option, page, radio pick, counter.
    Index(usize),
    /// Stepper.
    Int(i32),
    /// Text field.
    Text(String),
}

/// Every interactive demo's value, by key.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Values(HashMap<String, Value>);

impl Values {
    /// The bool under `bind`, or its default.
    pub fn flag(&self, bind: &Bind<bool>) -> bool {
        match self.0.get(&bind.key) {
            Some(Value::Bool(b)) => *b,
            _ => bind.default,
        }
    }

    /// The number under `bind`, or its default.
    pub fn num(&self, bind: &Bind<f32>) -> f32 {
        match self.0.get(&bind.key) {
            Some(Value::Num(n)) => *n,
            _ => bind.default,
        }
    }

    /// The index under `bind`, or its default.
    pub fn index(&self, bind: &Bind<usize>) -> usize {
        self.index_of(&bind.key, bind.default)
    }

    /// The index under `key`, or `default`.
    pub fn index_of(&self, key: &str, default: usize) -> usize {
        match self.0.get(key) {
            Some(Value::Index(i)) => *i,
            _ => default,
        }
    }

    /// The integer under `bind`, or its default.
    pub fn int(&self, bind: &Bind<i32>) -> i32 {
        match self.0.get(&bind.key) {
            Some(Value::Int(i)) => *i,
            _ => bind.default,
        }
    }

    /// The text under `bind`, or its default.
    pub fn text<'a>(&'a self, bind: &'a Bind<String>) -> &'a str {
        match self.0.get(&bind.key) {
            Some(Value::Text(s)) => s,
            _ => &bind.default,
        }
    }

    /// Store `value` under `key`.
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.0.insert(key.into(), value);
    }
}

/// Value key of the theme-wide glow strength (Surface effects).
pub(crate) const GLOW_KEY: &str = "fx-glow";
/// Value key of the theme-wide scanlines switch (Surface effects).
pub(crate) const SCANLINES_KEY: &str = "fx-scanlines";

/// Which light/dark mode the page follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModePref {
    /// The operating system's (see [`Showcase::system_dark`]).
    #[default]
    System,
    /// Light.
    Light,
    /// Dark.
    Dark,
}

impl ModePref {
    /// Every choice, in picker order.
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];
    /// Picker label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}

/// Animation speed multipliers, in picker order.
pub const SPEEDS: [f32; 4] = [0.5, 1.0, 2.0, 4.0];

/// What a toolkit tells the showcase happened.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Show a category (`None` = all).
    Category(Option<usize>),
    /// The sidebar filter text changed.
    Filter(String),
    /// Pick a design (by index into [`Showcase::designs`]).
    Design(usize),
    /// Pick a theme of the design (by index into [`Showcase::variants`]).
    Variant(usize),
    /// Filled-control colour.
    Colour(ControlColour),
    /// Light/dark preference.
    Mode(ModePref),
    /// Animation on/off.
    Animate(bool),
    /// Animation speed (index into [`SPEEDS`]).
    Speed(usize),
    /// Right-to-left layout.
    Rtl(bool),
    /// Series patterns.
    Patterns(bool),
    /// Next (`true`) or previous theme.
    CycleTheme(bool),
    /// A demo changed value.
    Set(String, Value),
    /// A counting button was pressed.
    Press(String),
}

/// The showcase's whole state.
#[derive(Debug, Clone)]
pub struct Showcase {
    switcher: ThemeSwitcher,
    category: Option<usize>,
    filter: String,
    colour: ControlColour,
    mode: ModePref,
    /// Whether the system is in dark mode (the toolkit sets this from the OS).
    pub system_dark: bool,
    /// Animation on.
    pub animate: bool,
    speed: usize,
    /// Right-to-left layout.
    pub rtl: bool,
    /// Series patterns.
    pub patterns: bool,
    /// The demos' values.
    pub values: Values,
    revision: u64,
}

impl Default for Showcase {
    fn default() -> Self {
        Self::new()
    }
}

impl Showcase {
    /// Every built-in theme, starting on the first.
    pub fn new() -> Self {
        Self::with_switcher(dodai_core::switcher())
    }

    /// The showcase configured from the environment: `DODAI_THEME` (a theme id), `DODAI_CAT`
    /// (`0` = all, `1..` = a category) and `DODAI_FILTER` (the sidebar filter). What a
    /// screenshot script sets so every toolkit opens on the same page.
    pub fn from_env() -> Self {
        let var = |k: &str| std::env::var(k).ok();
        Self::new().configured(
            var("DODAI_THEME").as_deref(),
            var("DODAI_CAT").and_then(|c| c.parse().ok()),
            var("DODAI_FILTER").as_deref(),
        )
    }

    /// This showcase opened on `theme` (a theme id), `category` (`0` = all, `1..` = a category)
    /// and `filter`; what is `None` or unknown is left as it is.
    pub fn configured(
        mut self,
        theme: Option<&str>,
        category: Option<usize>,
        filter: Option<&str>,
    ) -> Self {
        if let Some(id) = theme {
            self.select_id(id);
        }
        if let Some(c) = category {
            self.update(Action::Category(c.checked_sub(1)));
        }
        if let Some(f) = filter {
            self.update(Action::Filter(f.to_string()));
        }
        self
    }

    /// Over a given set of themes.
    pub fn with_switcher(switcher: ThemeSwitcher) -> Self {
        Self {
            switcher,
            category: None,
            filter: String::new(),
            colour: ControlColour::Design,
            mode: ModePref::System,
            system_dark: true,
            animate: true,
            speed: 1,
            rtl: false,
            patterns: false,
            values: Values::default(),
            revision: 0,
        }
    }

    /// Counts up whenever the theme changes; toolkits rebuild their theme when it moves.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The active theme with the chosen control colour, glow and scanlines applied.
    pub fn theme(&self) -> Theme {
        let mut t = self
            .switcher
            .current()
            .clone()
            .with_control_colour(self.colour);
        if let Some(Value::Num(g)) = self.values.0.get(GLOW_KEY) {
            t = t.with_glow_intensity(*g);
        }
        if let Some(Value::Bool(on)) = self.values.0.get(SCANLINES_KEY) {
            t = t.with_scanlines(*on);
        }
        t
    }

    /// The design families, in picker order.
    pub fn designs(&self) -> Vec<&'static str> {
        self.switcher.families()
    }

    /// Display names of the designs, in the order of [`designs`](Self::designs) ("Material 3",
    /// not the family id `material`).
    pub fn design_names(&self) -> Vec<&'static str> {
        let systems = dodai_core::themes::systems();
        self.designs()
            .into_iter()
            .map(|f| systems.iter().find(|s| s.family == f).map_or(f, |s| s.name))
            .collect()
    }

    /// Index of the active design in [`designs`](Self::designs).
    pub fn design_index(&self) -> usize {
        let family = self.switcher.current_family();
        self.designs()
            .iter()
            .position(|d| *d == family)
            .unwrap_or(0)
    }

    /// Names of the active design's themes.
    pub fn variants(&self) -> Vec<&'static str> {
        self.switcher
            .family_indices(self.switcher.current_family())
            .into_iter()
            .map(|i| self.switcher.themes()[i].meta.name)
            .collect()
    }

    /// Index of the active theme among [`variants`](Self::variants).
    pub fn variant_index(&self) -> usize {
        let at = self.switcher.index();
        self.switcher
            .family_indices(self.switcher.current_family())
            .iter()
            .position(|&i| i == at)
            .unwrap_or(0)
    }

    /// Choices the colour picker offers on the active theme.
    pub fn colours(&self) -> Vec<ControlColour> {
        ControlColour::offered(self.switcher.current())
    }

    /// The control colour in effect (a choice the theme does not offer falls back to the
    /// design's own).
    pub fn colour(&self) -> ControlColour {
        if self.colours().contains(&self.colour) {
            self.colour
        } else {
            ControlColour::Design
        }
    }

    /// The mode preference.
    pub fn mode(&self) -> ModePref {
        self.mode
    }

    /// The animation speed multiplier.
    pub fn speed(&self) -> f32 {
        SPEEDS[self.speed]
    }

    /// Index of the speed in [`SPEEDS`].
    pub fn speed_index(&self) -> usize {
        self.speed
    }

    /// The shown category (`None` = all).
    pub fn category(&self) -> Option<usize> {
        self.category
    }

    /// The sidebar filter.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// The sections on show: the chosen category's (or all), matching the filter.
    pub fn visible(&self) -> Vec<&'static Section> {
        let q = self.filter.trim().to_lowercase();
        catalog::sections()
            .iter()
            .filter(|s| match self.category {
                Some(c) => s.category == CATEGORIES[c],
                None => true,
            })
            .filter(|s| {
                q.is_empty()
                    || s.title.to_lowercase().contains(&q)
                    || s.category.to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Sections of category `c` that match the filter (the sidebar count).
    pub fn count(&self, c: Option<usize>) -> usize {
        let q = self.filter.trim().to_lowercase();
        catalog::sections()
            .iter()
            .filter(|s| c.is_none_or(|c| s.category == CATEGORIES[c]))
            .filter(|s| q.is_empty() || s.title.to_lowercase().contains(&q))
            .count()
    }

    fn bump(&mut self) {
        self.revision += 1;
    }

    fn apply_mode(&mut self) {
        let dark = match self.mode {
            ModePref::System => self.system_dark,
            ModePref::Light => false,
            ModePref::Dark => true,
        };
        let want = if dark { Mode::Dark } else { Mode::Light };
        self.switcher.select_mode(want);
    }

    /// Re-applies the mode after the toolkit learned the system's.
    pub fn set_system_dark(&mut self, dark: bool) {
        if self.system_dark != dark {
            self.system_dark = dark;
            if self.mode == ModePref::System {
                self.apply_mode();
                self.bump();
            }
        }
    }

    /// Switch to the theme with this id (a built-in id such as `material-dark`); `false` when
    /// there is none.
    pub fn select_id(&mut self, id: &str) -> bool {
        let found = self.switcher.select(id);
        if found {
            self.mode = match self.switcher.current().meta.mode {
                Mode::Light => ModePref::Light,
                Mode::Dark => ModePref::Dark,
            };
            self.bump();
        }
        found
    }

    /// Apply what happened.
    pub fn update(&mut self, action: Action) {
        match action {
            Action::Category(c) => self.category = c.filter(|&c| c < CATEGORIES.len()),
            Action::Filter(f) => self.filter = f,
            Action::Design(i) => {
                if let Some(d) = self.designs().get(i).copied() {
                    if self.switcher.select_family(d) {
                        self.apply_mode();
                        self.bump();
                    }
                }
            }
            Action::Variant(i) => {
                let idx = self.switcher.family_indices(self.switcher.current_family());
                if let Some(&at) = idx.get(i) {
                    self.switcher.select_index(at);
                    self.mode = match self.switcher.current().meta.mode {
                        Mode::Light => ModePref::Light,
                        Mode::Dark => ModePref::Dark,
                    };
                    self.bump();
                }
            }
            Action::Colour(c) => {
                self.colour = c;
                self.bump();
            }
            Action::Mode(m) => {
                self.mode = m;
                self.apply_mode();
                self.bump();
            }
            Action::Animate(b) => self.animate = b,
            Action::Speed(i) => self.speed = i.min(SPEEDS.len() - 1),
            Action::Rtl(b) => self.rtl = b,
            Action::Patterns(b) => self.patterns = b,
            Action::CycleTheme(next) => {
                if next {
                    self.switcher.cycle_next();
                } else {
                    self.switcher.cycle_prev();
                }
                self.bump();
            }
            Action::Set(key, v) => {
                let theme_wide = key == GLOW_KEY || key == SCANLINES_KEY;
                self.values.set(key, v);
                if theme_wide {
                    self.bump();
                }
            }
            Action::Press(key) => {
                let n = self.values.index_of(&key, 0);
                self.values.set(key, Value::Index(n + 1));
            }
        }
    }
}

#[cfg(test)]
mod surface_tests {
    use super::*;

    #[test]
    fn glow_and_scanlines_values_reach_the_theme() {
        let mut m = Showcase::default();
        let rev = m.revision();
        m.update(Action::Set(GLOW_KEY.into(), Value::Num(1.5)));
        m.update(Action::Set(SCANLINES_KEY.into(), Value::Bool(true)));
        assert_eq!(m.theme().effects.glow_intensity, 1.5);
        assert!(m.theme().effects.scanlines);
        assert!(m.revision() > rev);
        m.update(Action::Set(SCANLINES_KEY.into(), Value::Bool(false)));
        assert!(!m.theme().effects.scanlines);
    }

    #[test]
    fn other_values_leave_the_revision_alone() {
        let mut m = Showcase::default();
        let rev = m.revision();
        m.update(Action::Set("sl-a".into(), Value::Num(3.0)));
        assert_eq!(m.revision(), rev);
    }
}
