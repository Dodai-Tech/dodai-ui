//! Test-only themes: a rounded light theme and a square theme.

use dodai_core::themes::ThemeBuilder;
use dodai_core::{Corners, Mode, Shape, Theme, ThemeMeta};

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

/// A headless app with the component systems and their messages, for the constructor tests.
pub fn component_app() -> bevy::prelude::App {
    use bevy::prelude::*;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<crate::theme::DodaiTheme>();
    app.add_message::<crate::widgets::ControlEvent>();
    app.add_message::<bevy::input::mouse::MouseWheel>();
    crate::interact::add(&mut app);
    crate::nav::add(&mut app);
    crate::data::add(&mut app);
    app.add_systems(Update, crate::systems::input_keys);
    crate::form::add(&mut app);
    crate::time::add(&mut app);
    crate::themes::add(&mut app);
    crate::effects::add(&mut app);
    crate::overlay::add(&mut app);
    app
}
