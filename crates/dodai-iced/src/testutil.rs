//! Test-only themes: proves components adapt to tokens, not theme ids.

use dodai_core::{themes::ThemeBuilder, Corners, Mode, Shape, Theme, ThemeMeta};

pub fn cyberpunk() -> Theme {
    dodai_core::themes::cyberpunk::yellow()
}

fn spec(id: &'static str, shape: Shape, glow: f32, glow_blur: f32, hard_shadow: f32) -> Theme {
    ThemeBuilder {
        shape,
        glow,
        glow_blur,
        hard_shadow,
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

/// Rounded light theme (radius 8, 1px border, no glow, soft 2px shadow).
pub fn rounded_light() -> Theme {
    spec(
        "test-light",
        Shape {
            corners: Corners::NONE,
            radius: 8.0,
            chamfer: 0.0,
            scoop: false,
            border_width: 1.0,
            control_height: 40.0,
        },
        0.0,
        0.0,
        2.0,
    )
}

/// Hard-square theme (no radius, no chamfer, 2px border).
pub fn hard_square() -> Theme {
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
        0.0,
        0.0,
        0.0,
    )
}
