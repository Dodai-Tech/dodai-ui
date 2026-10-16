//! # legendary-egui
//!
//! Turns a [`legendary_core::Theme`] into [egui](https://docs.rs/egui) styling
//! plus painter-drawn widgets. Call [`install`] once per theme change, then use
//! the widgets. Nothing here looks at a theme id: shape (square / rounded /
//! chamfered), glow, hard shadow, scanlines and uppercase labels all come from
//! tokens, so new themes work automatically.
//!
//! ## Components
//!
//! [`Button`] (every `ButtonVariant`, disabled), [`Panel`], [`Tag`], [`Tabs`],
//! [`Toggle`], [`Checkbox`], [`Radio`], [`ProgressBar`], [`Slider`],
//! [`TextInput`], [`ListItem`], [`Divider`], [`tooltip`], text helpers
//! ([`LegendaryUi`]), [`paint_scanlines`] and [`ThemeController`].
//!
//! ```no_run
//! use legendary_core::Theme;
//! fn frame(ctx: &egui::Context) {
//!     legendary_egui::install(ctx, &Theme::default());
//!     egui::CentralPanel::default().show(ctx, |ui| {
//!         ui.add(legendary_egui::Button::new("Jack in"));
//!     });
//! }
//! ```

pub mod button;
pub mod controls;
pub mod install;
pub mod list;
pub mod paint;
pub mod panel;
pub mod scanlines;
pub mod switcher;
pub mod tabs;
pub mod tag;
pub mod text;

#[cfg(test)]
mod testing;

pub use button::Button;
pub use controls::{Checkbox, ProgressBar, Radio, Slider, Toggle};
pub use install::{active_theme, apply_to_style, install, visuals};
pub use list::{Divider, ListItem};
pub use paint::{box_shapes, color32, glow_shapes, outline_points, outline_shape};
pub use panel::{tooltip, Panel, TextInput};
pub use scanlines::{paint_scanlines, scanline_ys};
pub use switcher::{glow_controls, theme_switcher_bar, ThemeController};
pub use tabs::Tabs;
pub use tag::Tag;
pub use text::{display_text, LegendaryUi};
