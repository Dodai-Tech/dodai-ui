//! # dodai-bevy
//!
//! [Bevy](https://bevy.org) `bevy_ui` adapter for the Dodai UI design
//! system. Everything derives from `dodai-core` tokens — no theme-id
//! branching: shapes follow `ShapeKind`, glow follows `Effects`, case follows
//! `Typography::uppercase_labels`.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use dodai_bevy::prelude::*;
//!
//! App::new()
//!     .add_plugins((DefaultPlugins, DodaiUiPlugin::default()))
//!     .add_systems(Startup, |mut c: Commands| {
//!         c.spawn(Camera2d);
//!         c.spawn(button("Continue", ButtonVariant::Primary));
//!     })
//!     .run();
//! ```
//!
//! Modules: [`theme`] (resource), [`surface`] (shape/glow painting),
//! [`widgets`] (components + spawn helpers), [`gizmo`] (orientation axis gizmo), [`style`] (pure token mapping),
//! [`material`] (shader materials), [`plugin`].

pub mod gizmo;
pub mod material;
pub mod plugin;
pub mod style;
pub mod surface;
pub mod surface_fx;
mod systems;
pub mod theme;
pub mod widgets;

#[cfg(test)]
mod testing;

pub use plugin::{theme_hotkeys, DodaiSet, DodaiUiPlugin, ScanlineOverlay};
pub use systems::FocusedInput;
pub use theme::{color, DodaiTheme};

/// Everything needed to build a themed UI.
pub mod prelude {
    pub use crate::gizmo::{axis_gizmo, AxisId, Basis, GizmoEvent, GizmoKind, UiAxisGizmo};
    pub use crate::plugin::{theme_hotkeys, DodaiSet, DodaiUiPlugin, ScanlineOverlay};
    pub use crate::surface::Surface;
    pub use crate::systems::FocusedInput;
    pub use crate::theme::{color, DodaiTheme};
    pub use crate::widgets::*;
    pub use dodai_core::{themes, ButtonVariant, SymbolState, Theme, ThemeSwitcher, Tone};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_resource_switches_and_overrides_glow() {
        let mut t = DodaiTheme::new();
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
        let c = color(dodai_core::Color::rgba(10, 20, 30, 255)).to_srgba();
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
            format!("{}::{}", "cyberpunk", "yellow"),
        ];
        for (name, src) in [
            ("surface", include_str!("surface.rs")),
            ("style", include_str!("style.rs")),
            ("systems", include_str!("systems.rs")),
            ("widgets", include_str!("widgets.rs")),
            ("gizmo", include_str!("gizmo.rs")),
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
