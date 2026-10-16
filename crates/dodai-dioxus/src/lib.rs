//! # dodai-dioxus
//!
//! Renderer-agnostic [Dioxus](https://dioxuslabs.com) components for the Dodai UI
//! design system. Styling reuses the stylesheet from `dodai-topcoat`
//! (CSS custom properties from theme tokens + `.dodai-*` component classes), so shapes
//! (chamfer via `clip-path`, rounded, square), glow and scanlines all derive from tokens.
//!
//! Wrap your app in [`ThemeProvider`] and use components such as [`Button`], [`Panel`],
//! [`Tag`], [`Tabs`], [`Toggle`], [`Checkbox`], [`Radio`], [`Progress`], [`Slider`],
//! [`TextInput`], [`ListItem`], [`Divider`], [`Tooltip`] and the text components.
//! Switch themes with [`use_theme`] (`next`/`prev`/`select`), which updates `data-theme`.

mod components;
mod provider;

pub use components::*;
pub use provider::*;
