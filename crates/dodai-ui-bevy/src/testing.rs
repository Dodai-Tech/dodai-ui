//! Test-only themes: a rounded light theme and a square theme.

use dodai_ui_core::themes::ThemeBuilder;
use dodai_ui_core::{Corners, Mode, Shape, Theme, ThemeMeta};

fn spec(id: &'static str, shape: Shape) -> Theme {
    ThemeBuilder {
        shape,
        ..ThemeBuilder::neutral(ThemeMeta {
            id,
            name: id,
            family: "test",
            description: "",
            mode: Mode::Light,
        })
    }
    .build()
}

/// Rounded, light, no glow, labels not uppercase.
pub fn rounded_light() -> Theme {
    spec(
        "test-rounded",
        Shape {
            corners: Corners::NONE,
            radius: 8.0,
            chamfer: 0.0,
            scoop: false,
            border_width: 1.0,
            control_height: 40.0,
        },
    )
}

/// Square (no radius, no chamfer).
pub fn square_theme() -> Theme {
    spec(
        "test-square",
        Shape {
            corners: Corners::NONE,
            radius: 0.0,
            chamfer: 0.0,
            scoop: false,
            border_width: 2.0,
            control_height: 32.0,
        },
    )
}
