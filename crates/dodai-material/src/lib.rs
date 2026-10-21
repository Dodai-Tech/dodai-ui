//! **Material 3** — a Material-3-inspired light/dark pair (baseline purple
//! tonal palette): rounded corners, mixed-case labels, no glow or scanlines,
//! flat tonal surfaces, 40 px controls.

use dodai_core::themes::ThemeBuilder;
use dodai_core::{Color, Corners, Mode, Roles, Shape, Spacing, Theme, ThemeMeta, Typography};

/// Material's emphasized motion: 300 ms on the M3 `emphasized` cubic-bezier.
fn motion() -> dodai_core::Motion {
    dodai_core::Motion::new(
        std::time::Duration::from_millis(300),
        dodai_core::Easing::CubicBezier(0.2, 0.0, 0.0, 1.0),
    )
    .personality(dodai_core::dodai_motion::Personality::Smooth)
}

pub mod fx;
mod scheme;
mod spec;
pub use scheme::{preset_themes, Preset, Scheme, SchemeStyle, PRESETS};
pub use spec::{shape_presets, shape_system, Material3, SHAPE_SCALE, SPEC};

fn typography() -> Typography {
    Typography {
        display: "Roboto, Inter, Segoe UI, Helvetica Neue, sans-serif",
        body: "Roboto, Inter, Segoe UI, Helvetica Neue, sans-serif",
        mono: "Roboto Mono, JetBrains Mono, Menlo, Consolas, monospace",
        size_xs: 11.0,
        size_sm: 13.0,
        size_md: 14.0,
        size_lg: 18.0,
        size_xl: 24.0,
        size_xxl: 34.0,
        uppercase_labels: false,
        label_tracking: 0.0,
    }
}

fn shape() -> Shape {
    Shape {
        corners: Corners::NONE,
        radius: 12.0,
        chamfer: 0.0,
        border_width: 1.0,
        control_height: 40.0,
    }
}

fn spacing() -> Spacing {
    Spacing {
        xs: 4.0,
        sm: 8.0,
        md: 16.0,
        lg: 24.0,
        xl: 32.0,
    }
}

/// The Material 3 baseline roles (light), straight from the spec's tonal palette.
fn light_roles() -> Roles {
    let h = Color::hex;
    Roles {
        surface_container_lowest: h(0xffffff),
        surface_container_low: h(0xf7f2fa),
        surface_container: h(0xf3edf7),
        surface_container_high: h(0xece6f0),
        surface_container_highest: h(0xe6e0e9),
        on_surface: h(0x1d1b20),
        on_surface_variant: h(0x49454f),
        outline: h(0x79747e),
        outline_variant: h(0xcac4d0),
        primary_container: h(0xeaddff),
        on_primary_container: h(0x21005d),
        secondary_container: h(0xe8def8),
        on_secondary_container: h(0x1d192b),
        tertiary: h(0x7d5260),
        on_tertiary: h(0xffffff),
        tertiary_container: h(0xffd8e4),
        on_tertiary_container: h(0x31111d),
        error_container: h(0xf9dedc),
        on_error: h(0xffffff),
        on_error_container: h(0x410e0b),
        inverse_surface: h(0x322f35),
        inverse_on_surface: h(0xf5eff7),
        inverse_primary: h(0xd0bcff),
        scrim: h(0x000000),
        shadow: h(0x000000),
    }
}

/// The Material 3 baseline roles (dark).
fn dark_roles() -> Roles {
    let h = Color::hex;
    Roles {
        surface_container_lowest: h(0x0f0d13),
        surface_container_low: h(0x1d1b20),
        surface_container: h(0x211f26),
        surface_container_high: h(0x2b2930),
        surface_container_highest: h(0x36343b),
        on_surface: h(0xe6e0e9),
        on_surface_variant: h(0xcac4d0),
        outline: h(0x938f99),
        outline_variant: h(0x49454f),
        primary_container: h(0x4f378b),
        on_primary_container: h(0xeaddff),
        secondary_container: h(0x4a4458),
        on_secondary_container: h(0xe8def8),
        tertiary: h(0xefb8c8),
        on_tertiary: h(0x492532),
        tertiary_container: h(0x633b48),
        on_tertiary_container: h(0xffd8e4),
        error_container: h(0x8c1d18),
        on_error: h(0x601410),
        on_error_container: h(0xf9dedc),
        inverse_surface: h(0xe6e0e9),
        inverse_on_surface: h(0x322f35),
        inverse_primary: h(0x6750a4),
        scrim: h(0x000000),
        shadow: h(0x000000),
    }
}

/// Light scheme (baseline purple).
pub fn light() -> Theme {
    let mut t = ThemeBuilder {
        meta: ThemeMeta {
            id: "material",
            name: "Material 3 · Light",
            family: "material",
            description: "Tonal purple on a soft white surface",
            mode: Mode::Light,
        },
        mode: Mode::Light,
        background: 0xfef7ff,
        surface: 0xf7f2fa,
        raised: 0xece6f0,
        border: 0xcac4d0,
        border_strong: 0x79747e,
        text: 0x1d1b20,
        muted: 0x49454f,
        disabled: 0x938f99,
        primary: 0x6750a4,
        primary_hover: 0x5b4599,
        on_primary: 0xffffff,
        secondary: 0x625b71,
        on_secondary: 0xffffff,
        accent: 0x7d5260,
        info: 0x0061a4,
        success: 0x146c2e,
        warning: 0x8a5100,
        danger: 0xb3261e,
        series: [
            0x6750a4, 0x0061a4, 0x146c2e, 0x8a5100, 0xb3261e, 0x7d5260, 0x006874, 0x625b71,
        ],
        typography: typography(),
        spacing: spacing(),
        shape: shape(),
        glow: 0.0,
        glow_blur: 0.0,
        scanlines: false,
        hard_shadow: 0.0,
    }
    .build();
    t.palette.roles = light_roles();
    t.motion = motion();
    t.shape_system = Some(shape_system());
    t.spec = dodai_core::SpecRef::new(&SPEC);
    t
}

/// Dark scheme.
pub fn dark() -> Theme {
    let mut t = ThemeBuilder {
        meta: ThemeMeta {
            id: "material-dark",
            name: "Material 3 · Dark",
            family: "material",
            description: "Lavender on a deep violet-grey surface",
            mode: Mode::Dark,
        },
        mode: Mode::Dark,
        background: 0x141218,
        surface: 0x1d1b20,
        raised: 0x2b2930,
        border: 0x49454f,
        border_strong: 0x938f99,
        text: 0xe6e0e9,
        muted: 0xcac4d0,
        disabled: 0x79747e,
        primary: 0xd0bcff,
        primary_hover: 0xe0d2ff,
        on_primary: 0x381e72,
        secondary: 0xccc2dc,
        on_secondary: 0x332d41,
        accent: 0xefb8c8,
        info: 0x9ecaff,
        success: 0x7adb8f,
        warning: 0xffb870,
        danger: 0xf2b8b5,
        series: [
            0xd0bcff, 0x9ecaff, 0x7adb8f, 0xffb870, 0xf2b8b5, 0xefb8c8, 0x4fd8eb, 0xccc2dc,
        ],
        typography: typography(),
        spacing: spacing(),
        shape: shape(),
        glow: 0.0,
        glow_blur: 0.0,
        scanlines: false,
        hard_shadow: 0.0,
    }
    .build();
    t.palette.roles = dark_roles();
    t.motion = motion();
    t.shape_system = Some(shape_system());
    t.spec = dodai_core::SpecRef::new(&SPEC);
    t
}

/// Both Material 3 schemes.
pub fn variants() -> Vec<Theme> {
    let mut v = vec![light(), dark()];
    v.extend(preset_themes());
    v
}

/// The Material Symbols catalogue (`icons` feature).
#[cfg(feature = "icons")]
pub use dodai_material_icons as icons;
