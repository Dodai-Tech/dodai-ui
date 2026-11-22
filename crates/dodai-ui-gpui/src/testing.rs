//! Test-only themes proving components adapt purely from tokens.

use dodai_ui_core::themes::ThemeBuilder;
use dodai_ui_core::{Corners, Mode, Shape, Theme, ThemeMeta, Typography};

fn base(id: &'static str, shape: Shape, glow: f32, scan: bool, hard: f32, upper: bool) -> Theme {
    ThemeBuilder {
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
            uppercase_labels: upper,
            label_tracking: 0.0,
        },
        shape,
        glow,
        glow_blur: if glow > 0.0 { 10.0 } else { 0.0 },
        scanlines: scan,
        hard_shadow: hard,
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

/// The Material 3 light theme (rounded corners, tonal surfaces, its own spec).
pub fn light_rounded() -> Theme {
    dodai_ui_core::themes::material::light()
}

/// Square theme with scanlines, uppercase labels, a hard shadow and the Cyberpunk spec.
pub fn square() -> Theme {
    let t = base(
        "test-square",
        Shape {
            corners: Corners::NONE,
            radius: 0.0,
            chamfer: 0.0,
            scoop: false,
            border_width: 2.0,
            control_height: 36.0,
        },
        0.0,
        true,
        3.0,
        true,
    );
    t.with_spec(&dodai_ui_core::cyberpunk::SPEC)
}
