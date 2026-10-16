//! Yew 0.21 adapter for the Dodai UI design system.
//!
//! Components are thin typed wrappers that emit class names; all look-and-feel
//! (chamfer clip-paths from `Shape::outline`, glow, scanlines, uppercase
//! labels) comes from the token-derived CSS of `dodai-topcoat`, so there is
//! no per-theme branching. Wrap your app in [`ThemeProvider`].
//!
//! Render natively with `yew::ServerRenderer` (feature `ssr`) or in the browser.

pub mod classes;
pub mod components;
mod dom;
pub mod theme;

pub use components::*;
pub use theme::{theme_css, use_theme, ThemeContext, ThemeProvider, ThemeSwitcherBar};
