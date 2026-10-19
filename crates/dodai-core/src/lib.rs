//! # dodai-core
//!
//! Framework-agnostic **design tokens** for the Dodai UI design system:
//! colours (semantic roles), typography, spacing, shape, effects, resolved
//! component styles and a [`ThemeSwitcher`]. Adapter crates
//! (`dodai-ratatui`, `dodai-iced`, `dodai-gpui`, `dodai-topcoat`)
//! turn a [`Theme`] into native widgets/styles.
//!
//! ```
//! use dodai_core::{ThemeSwitcher, ButtonVariant, ControlState};
//! let sw = ThemeSwitcher::default(); // neutral fallback; real themes: `dodai-cyberpunk`, `dodai-material`
//! let t = sw.current().clone();
//! let b = t.button(ButtonVariant::Primary, ControlState::Idle);
//! assert!(b.background.is_some());
//! ```

mod color;
mod component;
pub mod export;
mod palette;
mod shape_system;
mod switcher;
mod theme;
mod tokens;

pub mod themes;

pub use color::Color;
pub use component::{
    ButtonTokens, ButtonVariant, ControlState, InputTokens, ListItemTokens, PanelTokens,
    ProgressTokens, TabTokens, TagTokens, ToggleTokens,
};
pub use palette::{Mode, Palette, Tone};
pub use shape_system::{
    Corner, CornerSpec, CornerStyle, DefaultShape, RoleShape, ShapePreset, ShapeRole, ShapeScale,
    ShapeSize, ShapeSystem, FULL,
};
pub use switcher::ThemeSwitcher;
pub use theme::{Theme, ThemeMeta};
pub use tokens::{Corners, Effects, Shape, ShapeKind, Spacing, Typography};
