//! Test-only themes: chamfered (built-in), rounded light and square.

use dodai_core::themes::ThemeBuilder;
use dodai_core::{Corners, Mode, Shape, Spacing, Theme, ThemeMeta, Typography};

/// Built-in chamfered theme.
pub fn chamfered() -> Theme {
    dodai_core::themes::cyberpunk::yellow()
}

/// Rounded light theme (no glow).
pub fn rounded() -> Theme {
    ThemeBuilder {
        meta: ThemeMeta {
            id: "test-light",
            name: "Test Light",
            family: "test",
            description: "",
            mode: Mode::Light,
        },
        mode: Mode::Light,
        background: 0xf6f7f9,
        surface: 0xffffff,
        raised: 0xeceef2,
        border: 0xd5d9e0,
        border_strong: 0xaab1bd,
        text: 0x14171c,
        muted: 0x4a5260,
        disabled: 0x8a92a0,
        primary: 0x1f5fd6,
        primary_hover: 0x1a50b5,
        on_primary: 0xffffff,
        secondary: 0x0a7d6e,
        on_secondary: 0xffffff,
        accent: 0xb45309,
        info: 0x1f6fd6,
        success: 0x15803d,
        warning: 0xb45309,
        danger: 0xc81e3a,
        series: [
            0x1f5fd6, 0x0a7d6e, 0xb45309, 0xc81e3a, 0x7c3aed, 0x0e7490, 0xbe185d, 0x4a5260,
        ],
        typography: Typography {
            display: "Inter",
            body: "Inter",
            mono: "Menlo",
            size_xs: 11.0,
            size_sm: 13.0,
            size_md: 15.0,
            size_lg: 18.0,
            size_xl: 24.0,
            size_xxl: 36.0,
            uppercase_labels: false,
            label_tracking: 0.0,
        },
        spacing: Spacing {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 20.0,
            xl: 32.0,
        },
        shape: Shape {
            corners: Corners::NONE,
            radius: 8.0,
            chamfer: 0.0,
            scoop: false,
            border_width: 1.0,
            control_height: 40.0,
        },
        glow: 0.0,
        glow_blur: 0.0,
        scanlines: false,
        hard_shadow: 2.0,
    }
    .build()
}

/// Square, uppercase-label variant of the light theme.
pub fn square() -> Theme {
    let mut t = rounded();
    t.meta.id = "test-square";
    t.shape.radius = 0.0;
    t.typography.uppercase_labels = true;
    t
}
