//! # dodai-freya
//!
//! Freya adapter for the Dodai UI design system.
//!
//! * [`style`] — pure token → Freya-attribute mapping (colour strings,
//!   `border`/`corner_radius`/`shadow` values, chamfered-polygon SVG, text
//!   roles). No Freya types: always compiled, unit-tested without a window.
//! * `theme`, `components` (feature `ui`, default) — theme install
//!   ([`use_init_dodai`]), [`ThemeSwitcher`](dodai_core::ThemeSwitcher)
//!   integration and the component set.
//!
//! Shapes come from tokens only: `Rounded` → `corner_radius`, `Square` → plain
//! rect, `Chamfered` → an absolutely positioned `svg` polygon built from
//! [`dodai_core::Shape::outline`] behind the content (needs explicit
//! `width`/`height`). Glow is a Freya `shadow` (rect path) or a blurred SVG
//! polygon (chamfered path), driven by `Effects`. No code branches on a theme id.
//!
//! ```ignore
//! use freya::prelude::*;
//! use dodai_freya::*;
//!
//! fn app() -> Element {
//!     use_init_dodai(dodai_core::Theme::default());
//!     rsx!(Button { label: "Jack in", variant: ButtonVariant::Primary })
//! }
//! ```

pub mod style;

pub use dodai_core as core;
pub use style::{
    button_style, chamfer_svg, css, input_style, list_item_style, panel_style, progress_style,
    tab_style, tag_style, text_style, toggle_style, ShapeStyle, TextRole, TextStyle,
};

#[cfg(feature = "ui")]
pub mod components;
#[cfg(feature = "ui")]
pub mod theme;

#[cfg(feature = "ui")]
pub use components::*;
#[cfg(feature = "ui")]
pub use dodai_core::{ButtonVariant, ControlState, Tone};
#[cfg(feature = "ui")]
pub use theme::{
    use_init_dodai, use_dodai, use_dodai_theme, Dodai, DodaiState,
};
