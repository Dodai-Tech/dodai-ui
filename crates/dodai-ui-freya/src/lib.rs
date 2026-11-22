//! # dodai-ui-freya
//!
//! Freya adapter for the Dodai UI design system.
//!
//! * [`style`] — pure token → paint mapping (colours, border / radius / shadow
//!   numbers, chamfered-polygon SVG, text roles). No Freya types: always
//!   compiled, unit-tested without a window.
//! * `paint`, `theme`, `components` (feature `ui`, default) — token → Freya
//!   values, theme install ([`use_init_dodai`]),
//!   [`ThemeSwitcher`](dodai_ui_core::ThemeSwitcher) integration and the component
//!   set. Targets freya 0.5 (builder API: `rect()`, `label()`, `Component`).
//!
//! Shapes come from tokens only: `Rounded` → `corner_radius`, `Square` → plain
//! rect, `Chamfered` → an absolutely positioned `SvgViewer` polygon built from
//! [`dodai_ui_core::Shape::outline`] behind the content (needs explicit
//! width/height). Glow is a Freya `Shadow` (rect path) or a blurred SVG
//! polygon (chamfered path), driven by `Effects`. No code branches on a theme id.
//!
//! ```ignore
//! use freya::prelude::*;
//! use dodai_ui_freya::*;
//!
//! fn app() -> impl IntoElement {
//!     use_init_dodai(dodai_ui_core::Theme::default());
//!     DodaiButton::new("Continue").variant(ButtonVariant::Primary)
//! }
//! ```

pub mod style;

#[cfg(feature = "icons")]
mod icon;
#[cfg(feature = "icons")]
pub use icon::icon_svg;

pub use dodai_ui_core as core;
pub use style::{
    button_style, chamfer_svg, input_style, list_item_style, panel_style, progress_style,
    resolved_surface, set_surface, surface, tab_style, tag_style, text_style, toggle_style,
    ShapeStyle, TextRole, TextStyle,
};

#[cfg(feature = "ui")]
pub mod components;
#[cfg(feature = "ui")]
pub mod paint;
#[cfg(feature = "ui")]
pub mod theme;

#[cfg(feature = "ui")]
pub use components::*;
#[cfg(feature = "ui")]
pub use dodai_ui_core::{ButtonVariant, ControlState, Tone};
#[cfg(all(feature = "ui", feature = "icons"))]
pub use icon::DodaiIcon;
#[cfg(feature = "ui")]
pub use theme::{use_dodai, use_dodai_theme, use_init_dodai, Dodai, DodaiState};
