//! # legendary-bevy
//!
//! [Bevy](https://bevy.org) `bevy_ui` adapter for the Legendary UI design
//! system. Everything derives from `legendary-core` tokens — no theme-id
//! branching: shapes follow `ShapeKind`, glow follows `Effects`, case follows
//! `Typography::uppercase_labels`.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use legendary_bevy::prelude::*;
//!
//! App::new()
//!     .add_plugins((DefaultPlugins, LegendaryUiPlugin::default()))
//!     .add_systems(Startup, |mut c: Commands| {
//!         c.spawn(Camera2d);
//!         c.spawn(button("Jack in", ButtonVariant::Primary));
//!     })
//!     .run();
//! ```
//!
//! Modules: [`theme`] (resource), [`surface`] (shape/glow painting),
//! [`widgets`] (components + spawn helpers), [`style`] (pure token mapping),
//! [`material`] (shader materials), [`plugin`].

pub mod material;
pub mod plugin;
pub mod style;
pub mod surface;
mod systems;
pub mod theme;
pub mod widgets;

#[cfg(test)]
mod testing;

pub use plugin::{theme_hotkeys, LegendarySet, LegendaryUiPlugin, ScanlineOverlay};
pub use systems::FocusedInput;
pub use theme::{color, LegendaryTheme};

/// Everything needed to build a themed UI.
pub mod prelude {
    pub use crate::plugin::{theme_hotkeys, LegendarySet, LegendaryUiPlugin, ScanlineOverlay};
    pub use crate::surface::Surface;
    pub use crate::systems::FocusedInput;
    pub use crate::theme::{color, LegendaryTheme};
    pub use crate::widgets::*;
    pub use legendary_core::{themes, ButtonVariant, Theme, ThemeSwitcher, Tone};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_resource_switches_and_overrides_glow() {
        let mut t = LegendaryTheme::new();
        let first = t.theme().id();
        t.cycle_next();
        assert_ne!(t.theme().id(), first);
        t.set_glow_intensity(2.0);
        assert_eq!(t.glow_intensity(), 2.0);
        // glow override survives a theme switch
        t.cycle_next();
        assert_eq!(t.glow_intensity(), 2.0);
        assert!(t.select(first) && !t.select("nope"));
        assert_eq!(t.theme().id(), first);
    }

    #[test]
    fn colour_conversion_is_lossless_for_srgb8() {
        let c = color(legendary_core::Color::rgba(10, 20, 30, 255)).to_srgba();
        assert!((c.red - 10.0 / 255.0).abs() < 1e-3 && (c.blue - 30.0 / 255.0).abs() < 1e-3);
    }

    /// Everything derives from tokens: no source file may branch on a theme
    /// identity (ids are built at runtime so this file does not match itself).
    #[test]
    fn no_theme_id_branching() {
        let banned = [
            format!("{}.id()", "theme"),
            format!("meta.{}", "id"),
            format!("\"{}\"", "cyberpunk"),
            format!("{}::{}", "cyberpunk", "night_city"),
        ];
        for (name, src) in [
            ("surface", include_str!("surface.rs")),
            ("style", include_str!("style.rs")),
            ("systems", include_str!("systems.rs")),
            ("widgets", include_str!("widgets.rs")),
            ("plugin", include_str!("plugin.rs")),
            ("material", include_str!("material.rs")),
        ] {
            let prod = src.split("#[cfg(test)]").next().unwrap();
            for b in &banned {
                assert!(!prod.contains(b.as_str()), "{name} contains {b}");
            }
        }
    }
}
