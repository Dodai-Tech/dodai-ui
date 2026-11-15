//! Leptos adapter for the Dodai UI design system.
//!
//! Components render semantic HTML with `dodai-*` classes; all look & feel comes
//! from the CSS custom properties generated from a theme's tokens
//! (see [`dodai_topcoat`]). Shape (square / rounded / chamfered) and glow
//! therefore follow the active theme without any theme-id branching.
//!
//! * [`ThemeProvider`] sets `data-theme` and emits the stylesheet.
//! * [`install_css`] / [`install_css_with`] build the stylesheet for themes.
//! * [`components`] holds the typed components.
//!
//! Built with the `ssr` feature so it renders natively (`RenderHtml::to_html`).

pub mod class;
pub mod components;
mod glass;
pub mod theme;

pub use components::*;
pub use dodai_core::{ButtonVariant, Theme, ThemeSwitcher, Tone};
/// A surface mix as a CSS `background-image` (see [`dodai_topcoat::surface_background`]).
pub use dodai_topcoat::surface_background;
pub use glass::{glass, GlassContext};
pub use theme::{
    install_css, install_css_with, scanlines_css, ThemeProvider, GLOW_LEVELS, SCANLINES_CSS,
};
