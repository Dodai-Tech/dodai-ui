//! # dodai-ui-dioxus
//!
//! Renderer-agnostic [Dioxus](https://dioxuslabs.com) components for the Dodai UI
//! design system. Styling reuses the stylesheet from `dodai-ui-topcoat`
//! (CSS custom properties from theme tokens + `.dodai-*` component classes), so shapes
//! (chamfer via `clip-path`, rounded, square), glow and scanlines all derive from tokens.
//!
//! Wrap your app in [`ThemeProvider`] and use components such as [`Button`], [`Panel`],
//! [`Tag`], [`Tabs`], [`Toggle`], [`Checkbox`], [`Radio`], [`Progress`], [`Slider`],
//! [`TextInput`], [`ListItem`], [`Divider`], [`Tooltip`] and the text components.
//! Switch themes with [`use_theme`] (`next`/`prev`/`select`), which updates `data-theme`.

mod components;
mod content;
mod gizmo;
mod kit;
mod knob;
mod navigation;
mod provider;
mod surfaces;
mod time;
mod widgets;

pub use components::*;
pub use content::*;
/// A surface as a CSS `background-image` (see [`dodai_ui_topcoat::surface_background`]).
pub use dodai_ui_topcoat::surface_background;
pub use gizmo::*;
pub use kit::*;
pub use knob::*;
pub use navigation::*;
pub use provider::*;
pub use surfaces::*;
pub use time::*;
pub use widgets::*;
