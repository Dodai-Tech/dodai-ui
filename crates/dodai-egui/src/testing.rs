//! Test-only themes: chamfered (built-in), rounded light and square.

use dodai_core::themes::ThemeBuilder;
use dodai_core::{Mode, Theme, ThemeMeta};

/// Built-in chamfered theme.
pub fn chamfered() -> Theme {
    dodai_core::themes::cyberpunk::yellow()
}

/// Rounded light theme (no glow).
pub fn rounded() -> Theme {
    ThemeBuilder::neutral(ThemeMeta {
        id: "test-light",
        name: "Test Light",
        family: "test",
        description: "",
        mode: Mode::Light,
    })
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
