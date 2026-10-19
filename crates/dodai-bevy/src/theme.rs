//! The theme resource and colour conversion.

use bevy::prelude::{Color as BevyColor, Deref, Resource};
use dodai_core::{Color, Theme, ThemeSwitcher};

/// Convert a core colour to a Bevy sRGB colour.
pub fn color(c: Color) -> BevyColor {
    BevyColor::srgba_u8(c.r, c.g, c.b, c.a)
}

/// Active theme + switcher + glow level. Mutate through the methods (hot
/// switching): every styling system reacts to the resource changing.
#[derive(Resource, Deref, Debug, Clone)]
pub struct DodaiTheme {
    #[deref]
    theme: Theme,
    switcher: ThemeSwitcher,
    glow: Option<f32>,
}

impl DodaiTheme {
    /// Start on the first built-in theme with the switcher over all built-ins.
    pub fn new() -> Self {
        Self::with_switcher(dodai::switcher())
    }

    /// Use a custom switcher (e.g. including your own themes).
    pub fn with_switcher(switcher: ThemeSwitcher) -> Self {
        let theme = switcher.current().clone();
        Self {
            theme,
            switcher,
            glow: None,
        }
    }

    /// The resolved active theme (glow override applied).
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// The underlying switcher.
    pub fn switcher(&self) -> &ThemeSwitcher {
        &self.switcher
    }

    fn rebuild(&mut self) {
        let mut t = self.switcher.current().clone();
        if let Some(g) = self.glow {
            t = t.with_glow_intensity(g);
        }
        self.theme = t;
    }

    /// Next theme (wraps).
    pub fn cycle_next(&mut self) {
        self.switcher.cycle_next();
        self.rebuild();
    }

    /// Previous theme (wraps).
    pub fn cycle_prev(&mut self) {
        self.switcher.cycle_prev();
        self.rebuild();
    }

    /// Select a theme by id; `false` if unknown.
    pub fn select(&mut self, id: &str) -> bool {
        let ok = self.switcher.select(id);
        self.rebuild();
        ok
    }

    /// Select by index (clamped).
    pub fn select_index(&mut self, i: usize) {
        self.switcher.select_index(i);
        self.rebuild();
    }

    /// Override glow intensity for every theme (0 = off, 1 = as designed).
    pub fn set_glow_intensity(&mut self, intensity: f32) {
        self.glow = Some(intensity.max(0.0));
        self.rebuild();
    }

    /// Current glow intensity.
    pub fn glow_intensity(&self) -> f32 {
        self.theme.effects.glow_intensity
    }
}

impl Default for DodaiTheme {
    fn default() -> Self {
        Self::new()
    }
}
