//! Theme switching and glow controls.

use crate::{install, Button};
use dodai_core::{ButtonVariant, Theme, ThemeSwitcher};
use egui::{Context, Key, Ui};

/// Glow intensity step for [`glow_controls`].
pub const GLOW_STEP: f32 = 0.25;
/// Max glow intensity for [`glow_controls`].
pub const GLOW_MAX: f32 = 3.0;

/// Button per theme; returns `true` if the selection changed.
pub fn theme_switcher_bar(ui: &mut Ui, switcher: &mut ThemeSwitcher) -> bool {
    let names: Vec<_> = switcher.themes().iter().map(|t| t.name()).collect();
    let cur = switcher.index();
    let mut pick = None;
    ui.horizontal_wrapped(|ui| {
        for (i, n) in names.iter().enumerate() {
            let v = if i == cur {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Outline
            };
            if ui.add(Button::new(*n).variant(v)).clicked() {
                pick = Some(i);
            }
        }
    });
    match pick {
        Some(i) if i != cur => {
            switcher.select_index(i);
            true
        }
        _ => false,
    }
}

/// `−` / `+` buttons adjusting glow intensity; returns `true` on change.
pub fn glow_controls(ui: &mut Ui, intensity: &mut f32) -> bool {
    let before = *intensity;
    ui.horizontal(|ui| {
        if ui
            .add(
                Button::new("−")
                    .variant(ButtonVariant::Ghost)
                    .min_width(40.0),
            )
            .clicked()
        {
            *intensity = step_glow(*intensity, -GLOW_STEP);
        }
        ui.label(format!("glow {:.2}", *intensity));
        if ui
            .add(
                Button::new("+")
                    .variant(ButtonVariant::Ghost)
                    .min_width(40.0),
            )
            .clicked()
        {
            *intensity = step_glow(*intensity, GLOW_STEP);
        }
    });
    *intensity != before
}

/// Clamped intensity step.
pub fn step_glow(current: f32, delta: f32) -> f32 {
    (current + delta).clamp(0.0, GLOW_MAX)
}

/// Switcher + glow intensity; install the result each frame.
pub struct ThemeController {
    /// Theme list and active index.
    pub switcher: ThemeSwitcher,
    /// Glow intensity applied on top of the active theme.
    pub glow: f32,
}

impl Default for ThemeController {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeController {
    /// All built-in themes, glow as designed.
    pub fn new() -> Self {
        Self {
            switcher: ThemeSwitcher::new(),
            glow: 1.0,
        }
    }
    /// Active theme with the glow intensity applied.
    pub fn theme(&self) -> Theme {
        self.switcher
            .current()
            .clone()
            .with_glow_intensity(self.glow)
    }
    /// Install the active theme (cheap when unchanged).
    pub fn install(&self, ctx: &Context) {
        install(ctx, &self.theme());
    }
    /// `t` next theme, `Shift+T` previous, `+`/`-` glow. Returns `true` on change.
    pub fn handle_keys(&mut self, ctx: &Context) -> bool {
        let (t, shift, plus, minus) = ctx.input(|i| {
            (
                i.key_pressed(Key::T),
                i.modifiers.shift,
                i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals),
                i.key_pressed(Key::Minus),
            )
        });
        if ctx.wants_keyboard_input() {
            return false;
        }
        if t {
            if shift {
                self.switcher.cycle_prev();
            } else {
                self.switcher.cycle_next();
            }
        }
        if plus {
            self.glow = step_glow(self.glow, GLOW_STEP);
        }
        if minus {
            self.glow = step_glow(self.glow, -GLOW_STEP);
        }
        t || plus || minus
    }
    /// Theme buttons + glow controls; returns `true` on change.
    pub fn show(&mut self, ui: &mut Ui) -> bool {
        let a = theme_switcher_bar(ui, &mut self.switcher);
        let b = glow_controls(ui, &mut self.glow);
        a || b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glow_step_clamps() {
        assert_eq!(step_glow(0.0, -GLOW_STEP), 0.0);
        assert_eq!(step_glow(GLOW_MAX, GLOW_STEP), GLOW_MAX);
        assert_eq!(step_glow(1.0, GLOW_STEP), 1.25);
    }

    #[test]
    fn controller_applies_glow_to_current() {
        let mut c = ThemeController::new();
        c.glow = 2.0;
        assert_eq!(c.theme().effects.glow_intensity, 2.0);
        let id = c.theme().id();
        c.switcher.cycle_next();
        assert_ne!(c.theme().id(), id);
        assert_eq!(c.theme().effects.glow_intensity, 2.0);
    }
}
