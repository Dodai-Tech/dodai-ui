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
//! let sw = ThemeSwitcher::default(); // neutral fallback; real themes: the `cyberpunk`, `material`, `neobrutalism` and `dodai` features
//! let t = sw.current().clone();
//! let b = t.button(ButtonVariant::Primary, ControlState::Idle);
//! assert!(b.background.is_some());
//! ```

pub mod calendar;
pub mod chart_style;
pub mod clock_math;
mod color;
mod component;
mod crop;
pub mod export;
mod glass;
mod kit;
mod knob;
pub mod markdown;
pub mod math;
pub mod orientation;
pub mod paging;
mod palette;
mod panel_border;
mod shape_system;
pub mod slider_math;
mod spec;
pub mod spy;
pub mod stream;
mod surface_paint;
mod surface_style;
mod switcher;
pub mod syntax;
mod theme;
mod tokens;
pub mod tonal;
mod tune;
pub mod vehicle_look;

pub mod themes;

/// Cyberpunk: six chamfered themes (feature `cyberpunk`).
#[cfg(feature = "cyberpunk")]
pub mod cyberpunk;
/// Dodai: frosted glass and gradients (feature `dodai`).
#[cfg(feature = "dodai")]
pub mod dodai;
/// Material 3: a light/dark pair (feature `material`).
#[cfg(feature = "material")]
pub mod material;
/// Neo-Brutalism: seven accents, light and dark (feature `neobrutalism`).
#[cfg(feature = "neobrutalism")]
pub mod neobrutalism;

pub use chart_style::{ChartStyle, DialLook, NeedleResponse, ScopeLook};
pub use color::Color;
pub use component::{
    AppBarKind, AppBarTokens, ButtonTokens, ButtonVariant, CardVariant, ControlSize, ControlState,
    InputTokens, ItemState, ListItemTokens, NavigationTokens, PanelTokens, ProgressTokens,
    SizeMetrics, StepState, StepTokens, SurfaceTokens, TabTokens, TagTokens, ToggleKind,
    ToggleTokens, TypeRole, TypeStyle,
};
pub use dodai_fx::{
    EffectSet, PressStyle, ResolvedSurface, Surface, SurfaceClass, SurfaceTargets, SurfaceTuning,
    Texture,
};
pub use glass::{
    backdrop_mesh, design_picture, frost_scene, glass_picture, mesh_blobs, mesh_pixels,
    surface_glass, Glass, MeshBlob, SurfaceMode, MESH_SIZE,
};
pub use kit::{busy, Kit, KitStyle};
pub use knob::{
    arc_points, knob_angle, knob_value, KnobGeometry, KnobStyle, KNOB_DRAG_PX, KNOB_START_DEG,
    KNOB_SWEEP_DEG,
};
pub mod motion;
pub use crop::{ButtonCrop, Crop, CropEdge};
pub use motion::{Easing, EntryKind, EntrySet, Motion};
pub use palette::{Mode, Palette, Roles, SymbolState, Tone};
pub use panel_border::{AccentCorner, BorderStyle, HardShadow, Outline, PanelFrame};
pub use shape_system::{
    Corner, CornerSpec, CornerStyle, DefaultShape, RoleShape, ShapePreset, ShapeRole, ShapeScale,
    ShapeSize, ShapeSystem, FULL,
};
pub use spec::{
    DesignSpec, FadeStop, Fluid, Frost, GridSpec, IconSet, Look, ProgressStyle, ShadowLayer,
    SpecRef, SpinnerStyle,
};
pub use surface_paint::{surface_paint, surface_svg, swatch_paint};
pub use surface_style::SurfaceStyle;
pub use switcher::ThemeSwitcher;
pub use theme::{Theme, ThemeMeta};
pub use tokens::{
    size_entries, Clip, Corners, Effects, Scale, Shape, ShapeKind, Sizes, Spacing, StateLayer,
    Typography,
};
pub use tonal::TonalPalette;
pub use tune::{BorderTint, ControlColour, GradientMode, RoundnessGroup, ShadowDir, ShapeTune};

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

/// A [`ThemeSwitcher`] over [`themes::all`].
pub fn switcher() -> ThemeSwitcher {
    ThemeSwitcher::with_themes(themes::all())
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
