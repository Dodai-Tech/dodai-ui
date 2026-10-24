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
mod spec;
mod surface;
mod switcher;
mod theme;
mod tokens;
pub mod tonal;
mod tune;

pub mod themes;

pub use color::Color;
pub use component::{
    AppBarKind, AppBarTokens, ButtonTokens, ButtonVariant, CardVariant, ControlSize, ControlState,
    InputTokens, ListItemTokens, NavigationTokens, PanelTokens, ProgressTokens, SizeMetrics,
    StepState, StepTokens, SurfaceTokens, TabTokens, TagTokens, ToggleKind, ToggleTokens, TypeRole,
    TypeStyle,
};
pub use dodai_motion;
pub use dodai_motion::{Easing, EntryKind, EntrySet, Motion};
pub use palette::{Mode, Palette, Roles, Tone};
pub use shape_system::{
    Corner, CornerSpec, CornerStyle, DefaultShape, RoleShape, ShapePreset, ShapeRole, ShapeScale,
    ShapeSize, ShapeSystem, FULL,
};
pub use spec::{DesignSpec, FadeStop, Fluid, Frost, GridSpec, IconSet, Look, ShadowLayer, SpecRef};
pub use surface::{
    EffectSet, PressStyle, Surface, SurfaceClass, SurfaceFx, SurfaceStyle, SurfaceTargets,
    SurfaceTuning, Texture,
};
pub use switcher::ThemeSwitcher;
pub use theme::{Theme, ThemeMeta};
pub use tokens::{
    Corners, Effects, Scale, Shape, ShapeKind, Sizes, Spacing, StateLayer, Typography,
};
pub use tonal::TonalPalette;
pub use tune::{BorderTint, RoundnessGroup, ShadowDir, ShapeTune};

/// Turn a generated string into a `&'static str`, once per distinct value.
///
/// Theme ids and names are `&'static str`, but presets build them with `format!`. Leaking a new
/// copy on every theme build grows without bound (a theme switcher rebuilds them all), so equal
/// strings share one allocation, kept in a table that stays reachable.
pub fn intern(s: String) -> &'static str {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock, PoisonError};
    static TABLE: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut table = TABLE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some(&hit) = table.get(s.as_str()) {
        return hit;
    }
    let leaked: &'static str = Box::leak(s.into_boxed_str());
    table.insert(leaked);
    leaked
}

#[cfg(test)]
mod intern_tests {
    #[test]
    fn equal_strings_share_one_allocation() {
        let a = super::intern(format!("theme-{}", 7));
        let b = super::intern(format!("theme-{}", 7));
        assert!(std::ptr::eq(a, b));
        assert_ne!(a, super::intern("other".into()));
    }
}
