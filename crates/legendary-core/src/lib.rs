//! # legendary-core
//!
//! Framework-agnostic **design tokens** for the Legendary Themes design system:
//! colours (semantic roles), typography, spacing, shape, effects, resolved
//! component styles and a [`ThemeSwitcher`]. Adapter crates
//! (`legendary-ratatui`, `legendary-iced`, `legendary-gpui`, `legendary-topcoat`)
//! turn a [`Theme`] into native widgets/styles.
//!
//! ```
//! use legendary_core::{ThemeSwitcher, ButtonVariant, ControlState};
//! let mut sw = ThemeSwitcher::new();
//! let t = sw.cycle_next().clone();
//! let b = t.button(ButtonVariant::Primary, ControlState::Idle);
//! assert!(b.background.is_some());
//! ```

mod color;
mod component;
mod palette;
mod switcher;
mod theme;
mod tokens;

pub mod themes;

pub use color::Color;
pub use component::{ButtonTokens, ButtonVariant, ControlState, InputTokens, PanelTokens};
pub use palette::{Mode, Palette, Tone};
pub use switcher::ThemeSwitcher;
pub use theme::{Theme, ThemeMeta};
pub use tokens::{Effects, Shape, Spacing, Typography};
