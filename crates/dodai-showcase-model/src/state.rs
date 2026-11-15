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
pub struct Values(pub(crate) HashMap<String, Value>);

/// Value key: the sidebar folded to an icon rail.
pub const RAIL: &str = "ui:rail";
/// Value key: the open page's section list folded in the sidebar.
pub const FOLDED: &str = "ui:folded";

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

/// First entry of the theme dropdown: follow the operating system's light or dark.
pub const SYSTEM: &str = "System";

/// Value key of the theme-wide glow strength (Surface effects · Light FX).
pub(crate) const GLOW_KEY: &str = "fx-glow";
/// Press-key prefix of the surface "apply to" shortcuts (`default`, `all`, `none`).
pub(crate) const TARGETS: &str = "sfx:targets:";

/// Display title of a design family id, as the gpui picker shows it (`"material"` →
/// `"Material"`).
pub fn family_title(id: &str) -> String {
    let mut c = id.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

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
    /// Filled-control colour (kept in the values with the shape tune).
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
    /// A section was picked in the sidebar: show it and scroll to it.
    Section(&'static str),
    /// The page was scrolled and this section is now at the top.
    Visible(&'static str),
    /// The user scrolled the page themselves: a picked section no longer holds the highlight.
    Unpin,
}

/// The showcase's whole state.
#[derive(Debug, Clone)]
pub struct Showcase {
    switcher: ThemeSwitcher,
    category: Option<usize>,
    filter: String,
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
    active: Option<&'static str>,
    jump: Option<&'static str>,
    /// The picked section keeps the highlight until the user scrolls (it may be too near the
    /// end of the page to reach the top).
    pinned: Option<&'static str>,
    effect: Option<crate::icon_actions::Effect>,
    /// The page is scrolled to its very top (the landing content is showing).
    at_top: bool,
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
    /// (`0` = all, `1..` = a category), `DODAI_FILTER` (the sidebar filter) and `DODAI_RAIL`
    /// (the sidebar folded). What a
    /// screenshot script sets so every toolkit opens on the same page.
    pub fn from_env() -> Self {
        let var = |k: &str| std::env::var(k).ok();
        Self::new()
            .configured(
                var("DODAI_THEME").as_deref(),
                var("DODAI_CAT").and_then(|c| c.parse().ok()),
                var("DODAI_FILTER").as_deref(),
            )
            .with_surface_spec(&var("DODAI_SFX").unwrap_or_default())
            .with_rail(var("DODAI_RAIL").is_some())
    }

    /// Test aid (`DODAI_RAIL=1`): open with the sidebar folded to its rail.
    pub fn with_rail(mut self, rail: bool) -> Self {
        if rail {
            self.values.set(RAIL, Value::Bool(true));
        }
        self
    }

    /// Test aid: applies a `DODAI_SFX` spec to the surface (see
    /// [`surface_state::apply_spec`](crate::surface_state::apply_spec)).
    pub fn with_surface_spec(mut self, spec: &str) -> Self {
        let t = self.theme();
        crate::surface_state::apply_spec(&mut self.values, &t, spec);
        self
    }

    /// Whether the app shows the glass picture behind everything (the frost demo's backdrop).
    pub fn wallpaper(&self) -> bool {
        crate::surface_state::wallpaper(&self.values)
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
            mode: ModePref::System,
            system_dark: true,
            animate: true,
            speed: 1,
            rtl: false,
            patterns: false,
            values: Values::default(),
            revision: 0,
            active: None,
            jump: None,
            effect: None,
            at_top: true,
            pinned: None,
        }
    }

    /// Counts up whenever the theme changes; toolkits rebuild their theme when it moves.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// How often the toolkit should redraw on its own: every frame while a looping animation
    /// shows or a timer, stopwatch or beat runs on the page, every quarter second while a clock
    /// shows, else never.
    pub fn redraw_every(&self) -> Option<std::time::Duration> {
        use std::time::Duration;
        let visible = self.visible();
        if !visible.iter().any(|s| s.live) {
            return None;
        }
        if visible.iter().any(|s| s.animated)
            || crate::live_state::running(&self.values, crate::widgets::elapsed())
        {
            Some(Duration::from_millis(33))
        } else {
            // The clock's second hand.
            Some(Duration::from_millis(250))
        }
    }

    /// Whether the sidebar is folded to its icon rail.
    pub fn rail(&self) -> bool {
        self.values.flag(&crate::Bind::new(RAIL, false))
    }

    /// The press key that folds or unfolds the sidebar rail.
    pub fn rail_toggle() -> String {
        crate::commands::flip(RAIL, false)
    }

    /// Whether the sidebar lists the sections of page `c` under it (the open page, unless it was
    /// folded by picking it again).
    pub fn lists(&self, c: usize) -> bool {
        self.current_category() == Some(c) && !self.values.flag(&crate::Bind::new(FOLDED, false))
    }

    /// Where the reader is, for the sidebar: the open page, or on "All" the category of the
    /// section being read (`None` while the landing content shows at the top).
    pub fn current_category(&self) -> Option<usize> {
        if self.category.is_some() || self.at_top || !self.filter.trim().is_empty() {
            return self.category;
        }
        let title = self.active_section()?;
        let cat = catalog::sections()
            .iter()
            .find(|s| s.title == title)?
            .category;
        CATEGORIES.iter().position(|c| *c == cat)
    }

    /// The sections the sidebar lists under category `c` (empty unless it is open).
    pub fn listed(&self, c: usize) -> Vec<&'static Section> {
        if !self.lists(c) {
            return Vec::new();
        }
        self.visible()
            .into_iter()
            .filter(|s| s.category == CATEGORIES[c])
            .collect()
    }

    /// Whether the page is scrolled to its very top.
    pub fn at_top(&self) -> bool {
        self.at_top
    }

    /// Tell the model whether the page is scrolled to its very top; `true` when that changed
    /// what the sidebar shows.
    pub fn set_at_top(&mut self, at_top: bool) -> bool {
        let before = self.current_category();
        self.at_top = at_top;
        before != self.current_category()
    }

    /// The landing content above the sections of "All" (no page picked, no search), naming the
    /// toolkit that draws it.
    pub fn overview(&self, toolkit: &str) -> Option<crate::node::Node> {
        (self.category.is_none() && self.filter.trim().is_empty())
            .then(|| crate::overview::overview(&self.theme(), toolkit))
    }

    /// The strip pinned above the page, between the top bar and the sections: real components
    /// that restyle the moment a Shape or Surface-effects control changes.
    pub fn pinned(&self) -> Option<crate::node::Node> {
        matches!(
            self.category().map(|c| CATEGORIES[c]),
            Some("Shape" | "Surface effects")
        )
        .then(crate::sections::shape::live_preview)
    }

    /// The app-wide [`Surface`](dodai_core::Surface) the Surface effects page set
    /// ([`INHERIT`](dodai_core::Surface::INHERIT) until a control changes something).
    pub fn surface(&self) -> dodai_core::Surface {
        crate::surface_state::surface(&self.values, &self.theme())
    }

    /// [`surface`](Self::surface) resolved under the active theme: what the toolkits draw.
    pub fn resolved_surface(&self) -> dodai_core::ResolvedSurface {
        crate::surface_state::mix(&self.values, &self.theme())
    }

    /// The active theme with the chosen control colour and glow applied.
    pub fn theme(&self) -> Theme {
        let mut t = crate::tune_state::apply(&self.values, self.switcher.current().clone());
        if let Some(Value::Num(g)) = self.values.0.get(GLOW_KEY) {
            t = t.with_glow_intensity(*g);
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
        self.designs()
            .into_iter()
            .map(|f| dodai_core::intern(family_title(f)))
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

    /// What the theme dropdown offers: [`SYSTEM`] first (follow the operating system's light or
    /// dark), then the names of the active design's themes.
    pub fn variants(&self) -> Vec<&'static str> {
        std::iter::once(SYSTEM)
            .chain(
                self.switcher
                    .family_indices(self.switcher.current_family())
                    .into_iter()
                    .map(|i| self.switcher.themes()[i].meta.name),
            )
            .collect()
    }

    /// Index of the active choice among [`variants`](Self::variants): 0 while following the
    /// system, otherwise the theme picked.
    pub fn variant_index(&self) -> usize {
        if self.mode == ModePref::System {
            return 0;
        }
        let at = self.switcher.index();
        1 + self
            .switcher
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
        let c = crate::tune_state::colour(&self.values);
        if self.colours().contains(&c) {
            c
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

    /// The section the page is at (the first shown one until the page is scrolled or one is
    /// picked).
    pub fn active_section(&self) -> Option<&'static str> {
        let shown = self.visible();
        self.active
            .filter(|a| shown.iter().any(|s| s.title == *a))
            .or_else(|| shown.first().map(|s| s.title))
    }

    /// What the last action needs the toolkit to do (a clipboard write), once.
    pub fn take_effect(&mut self) -> Option<crate::icon_actions::Effect> {
        self.effect.take()
    }

    /// The section the sidebar asked to scroll to, once.
    pub fn take_jump(&mut self) -> Option<&'static str> {
        self.jump.take()
    }

    /// The section a sidebar pick holds the highlight on, until [`Action::Unpin`].
    pub fn held_section(&self) -> Option<&'static str> {
        self.pinned
    }

    /// Follow the page's scroll: `tops` are the shown sections' tops in document order,
    /// `view_top` the page's top edge (same space), `at_end` that the scroll range is used up.
    /// Moves the highlight by the shared [`dodai_core::spy::pick`] rule.
    pub fn follow_scroll(
        &mut self,
        tops: &[(&'static str, f32)],
        view_top: f32,
        at_end: bool,
        slack: f32,
    ) {
        let now = dodai_core::spy::pick(tops, self.held_section(), view_top, at_end, slack);
        if let Some(t) = now.filter(|t| self.active_section() != Some(*t)) {
            self.update(Action::Visible(t));
        }
    }

    /// The sidebar filter.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// The sections on show: the chosen category's (or all), matching the filter.
    pub fn visible(&self) -> Vec<&'static Section> {
        let q = self.filter.trim().to_lowercase();
        let theme = self.theme();
        catalog::sections()
            .iter()
            .filter(|s| (s.shown)(&theme))
            .filter(|s| match self.category {
                Some(c) => s.category == CATEGORIES[c],
                // Loading and Shaders animate all the time: their own page only (a search still
                // finds the shaders).
                None if q.is_empty() => !crate::overview::OWN_PAGE.contains(&s.category),
                None => s.category != "Loading",
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
        let theme = self.theme();
        catalog::sections()
            .iter()
            .filter(|s| (s.shown)(&theme))
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
            Action::Category(c) => {
                let c = c.filter(|&c| c < CATEGORIES.len());
                self.at_top = true;
                // Picking the open page again folds its section list in the sidebar (or opens it).
                if c.is_some() && c == self.category {
                    let now = self.values.flag(&crate::Bind::new(FOLDED, false));
                    self.values.set(FOLDED, Value::Bool(!now));
                } else {
                    self.values.0.remove(FOLDED);
                }
                self.category = c;
                self.active = None;
                self.pinned = None;
            }
            Action::Section(title) => {
                // A title may repeat on another page: the open page's own comes first.
                let here = self.category.map(|c| CATEGORIES[c]);
                let all = catalog::sections();
                let found = all
                    .iter()
                    .find(|s| s.title == title && Some(s.category) == here)
                    .or_else(|| all.iter().find(|s| s.title == title));
                if let Some(s) = found {
                    if self.category.is_some() {
                        self.category = CATEGORIES.iter().position(|c| *c == s.category);
                    }
                    self.active = Some(s.title);
                    self.jump = Some(s.title);
                    self.pinned = Some(s.title);
                }
            }
            Action::Visible(title) => self.active = Some(title),
            Action::Unpin => self.pinned = None,
            Action::Filter(f) => self.filter = f,
            Action::Design(i) => {
                if let Some(d) = self.designs().get(i).copied() {
                    if self.switcher.select_family(d) {
                        self.apply_mode();
                        self.bump();
                    }
                }
            }
            Action::Variant(0) => {
                self.mode = ModePref::System;
                self.apply_mode();
                self.bump();
            }
            Action::Variant(i) => {
                let idx = self.switcher.family_indices(self.switcher.current_family());
                if let Some(&at) = idx.get(i - 1) {
                    self.switcher.select_index(at);
                    self.mode = match self.switcher.current().meta.mode {
                        Mode::Light => ModePref::Light,
                        Mode::Dark => ModePref::Dark,
                    };
                    self.bump();
                }
            }
            Action::Colour(c) => {
                crate::tune_state::set_colour(&mut self.values, c);
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
            Action::Set(key, Value::Index(i)) if key == crate::shader_state::FX_ADD => {
                self.values.add_effect_at(i);
            }
            Action::Set(key, Value::Index(i)) if key == crate::surface_state::MODE => {
                let theme = self.theme();
                crate::surface_state::set_mode(&mut self.values, &theme, i);
            }
            Action::Set(key, Value::Bool(on))
                if crate::commands::toggle(&mut self.values, &key, on) => {}
            Action::Set(key, Value::Index(i))
                if crate::tune_state::set(&mut self.values, self.switcher.current(), &key, i) =>
            {
                self.bump();
            }
            Action::Set(key, v) => {
                let theme_wide = key == GLOW_KEY || key.starts_with(crate::tune_state::PREFIX);
                self.values.set(key, v);
                if theme_wide {
                    self.bump();
                }
            }
            Action::Press(key) if key.starts_with(crate::icon_actions::PREFIX) => {
                let theme = self.theme();
                let what = &key[crate::icon_actions::PREFIX.len()..];
                self.effect = crate::icon_actions::run(&mut self.values, &theme, what, None);
                self.bump();
            }
            Action::Press(key) if key.starts_with("sh:") => self.shader_press(&key[3..]),
            Action::Press(key) if key.starts_with(TARGETS) => {
                crate::surface_state::set_targets(&mut self.values, &key[TARGETS.len()..]);
            }
            Action::Press(key) if key.starts_with(crate::overview::OPEN_CATEGORY) => {
                let i = key[crate::overview::OPEN_CATEGORY.len()..].parse().ok();
                self.update(Action::Category(i));
            }
            Action::Press(key) if key.starts_with(crate::overview::OPEN_SECTION) => {
                let title = &key[crate::overview::OPEN_SECTION.len()..];
                if let Some(s) = catalog::sections().iter().find(|s| s.title == title) {
                    // Sections that only live on their own page open it first.
                    if crate::overview::OWN_PAGE.contains(&s.category) {
                        self.category = CATEGORIES.iter().position(|c| *c == s.category);
                    }
                    self.update(Action::Section(s.title));
                }
            }
            Action::Press(key) if key == crate::tune_state::RESET => {
                crate::tune_state::reset(&mut self.values);
                self.bump();
            }
            Action::Press(key)
                if crate::live_state::press(&mut self.values, &key, crate::widgets::elapsed()) => {}
            Action::Press(key) if crate::commands::press(&mut self.values, &key) => {}
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
    fn the_glow_value_reaches_the_theme() {
        let mut m = Showcase::default();
        let rev = m.revision();
        m.update(Action::Set(GLOW_KEY.into(), Value::Num(1.5)));
        assert_eq!(m.theme().effects.glow_intensity, 1.5);
        assert!(m.revision() > rev);
    }

    #[test]
    fn the_surface_starts_as_the_designs_own_and_follows_the_mode_and_targets() {
        let mut m = Showcase::default();
        assert_eq!(m.surface(), dodai_core::Surface::INHERIT);
        assert_eq!(m.resolved_surface(), m.theme().default_surface());
        let mode = |m: &mut Showcase, i| {
            m.update(Action::Set(
                crate::surface_state::MODE.into(),
                Value::Index(i),
            ))
        };
        mode(&mut m, 3);
        assert_eq!(m.resolved_surface(), m.theme().signature_surface());
        mode(&mut m, 0);
        assert!(m.resolved_surface().is_none());
        m.update(Action::Press(format!("{TARGETS}none")));
        assert_eq!(
            m.resolved_surface().targets,
            dodai_core::SurfaceTargets::NONE
        );
        mode(&mut m, 1);
        assert_eq!(m.surface(), dodai_core::Surface::INHERIT);
    }

    #[test]
    fn other_values_leave_the_revision_alone() {
        let mut m = Showcase::default();
        let rev = m.revision();
        m.update(Action::Set("sl-a".into(), Value::Num(3.0)));
        assert_eq!(m.revision(), rev);
    }
}

#[cfg(test)]
mod system_theme_tests {
    use super::*;

    #[test]
    fn the_theme_dropdown_starts_with_system() {
        let mut m = Showcase::default();
        assert_eq!(m.variants()[0], SYSTEM);
        assert_eq!(m.variant_index(), 0);
        m.update(Action::Variant(1));
        assert_eq!(m.variant_index(), 1);
        assert_eq!(
            m.variants().len(),
            1 + m.switcher.family_indices(m.switcher.current_family()).len()
        );
        m.update(Action::Variant(0));
        assert_eq!(m.mode(), ModePref::System);
        assert_eq!(m.variant_index(), 0);
    }

    #[test]
    fn system_follows_the_operating_system() {
        let mut m = Showcase::default();
        m.update(Action::Variant(0));
        m.set_system_dark(true);
        let dark = m.theme().meta.mode;
        m.set_system_dark(false);
        assert_ne!(m.theme().meta.mode, dark);
    }
}

#[cfg(test)]
mod redraw_tests {
    use super::*;
    use std::time::Duration;

    fn page(name: &str) -> Option<usize> {
        crate::CATEGORIES.iter().position(|c| *c == name)
    }

    #[test]
    fn the_animation_page_redraws_every_frame() {
        let mut m = Showcase::default();
        m.update(Action::Category(page("Animation")));
        assert!(m.visible().iter().all(|s| s.animated && s.live));
        assert_eq!(m.redraw_every(), Some(Duration::from_millis(33)));
    }

    #[test]
    fn a_page_without_live_sections_never_redraws_on_its_own() {
        let mut m = Showcase::default();
        m.update(Action::Category(page("Shape")));
        assert_eq!(m.redraw_every(), None);
    }
}

#[cfg(test)]
mod section_nav_tests {
    use super::*;

    #[test]
    fn picking_a_section_jumps_once_and_scrolling_moves_the_highlight() {
        let mut m = Showcase::default();
        m.update(Action::Category(Some(0)));
        let titles: Vec<_> = m.visible().iter().map(|s| s.title).collect();
        assert_eq!(m.active_section(), Some(titles[0]));
        m.update(Action::Section(titles[2]));
        assert_eq!(m.active_section(), Some(titles[2]));
        assert_eq!(m.take_jump(), Some(titles[2]));
        assert_eq!(m.take_jump(), None);
        m.update(Action::Visible(titles[3]));
        assert_eq!(m.active_section(), Some(titles[3]));
        // Another page forgets it.
        m.update(Action::Category(Some(1)));
        assert_ne!(m.active_section(), Some(titles[3]));
    }

    #[test]
    fn a_picked_section_holds_the_highlight_until_the_user_scrolls() {
        let mut m = Showcase::default();
        m.update(Action::Category(Some(0)));
        let t: Vec<_> = m.visible().iter().map(|s| s.title).collect();
        let tops: Vec<_> = t
            .iter()
            .enumerate()
            .map(|(i, s)| (*s, i as f32 * 300.0 - 100.0))
            .collect();
        // Picked, but the page cannot bring it to the top: the scroll must not undo the pick.
        m.update(Action::Section(t[2]));
        m.follow_scroll(&tops, 0.0, false, 60.0);
        assert_eq!(m.active_section(), Some(t[2]));
        // The user scrolls: the highlight follows the page, and the last section wins at the end.
        m.update(Action::Unpin);
        m.follow_scroll(&tops, 0.0, false, 60.0);
        assert_eq!(m.active_section(), Some(t[0]));
        m.follow_scroll(&tops, 0.0, true, 60.0);
        assert_eq!(m.active_section(), t.last().copied());
    }

    #[test]
    fn on_all_the_sidebar_follows_the_scroll_into_a_category() {
        let mut m = Showcase::default();
        assert_eq!(m.current_category(), None);
        assert!(m.listed(0).is_empty());
        let first = m.visible()[0];
        m.set_at_top(false);
        m.update(Action::Visible(first.title));
        let c = CATEGORIES.iter().position(|c| *c == first.category);
        assert_eq!(m.current_category(), c);
        let listed = m.listed(c.unwrap());
        assert!(listed.iter().all(|s| s.category == first.category));
        assert!(listed.iter().any(|s| s.title == first.title));
        // Back at the top, "All" again; another page is its own.
        assert!(m.set_at_top(true));
        assert_eq!(m.current_category(), None);
        m.update(Action::Category(Some(2)));
        assert_eq!(m.current_category(), Some(2));
    }
}
