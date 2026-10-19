//! **Material 3** — a Material-3-inspired light/dark pair (baseline purple
//! tonal palette): rounded corners, mixed-case labels, no glow or scanlines,
//! flat tonal surfaces, 40 px controls.

use dodai_core::themes::Spec;
use dodai_core::{Corners, Mode, Shape, Spacing, Theme, ThemeMeta, Typography};

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

/// Light scheme (baseline purple).
pub fn light() -> Theme {
    Spec {
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
    .build()
}

/// Dark scheme.
pub fn dark() -> Theme {
    Spec {
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
    .build()
}

/// Both Material 3 schemes.
pub fn variants() -> Vec<Theme> {
    vec![light(), dark()]
}

/// The Material Symbols catalogue (`icons` feature).
#[cfg(feature = "icons")]
pub use dodai_material_icons as icons;
