//! The Dodai showcase in Bevy: the same pages, sections and controls as the gpui showcase,
//! drawn from `dodai-showcase-model` with dodai-bevy widgets.
//!
//! Keys: `t` / `T` next / previous theme. Env: `DODAI_THEME` (theme id), `DODAI_CAT` (0 = all,
//! 1.. = category), `DODAI_OPEN` (`design` or `theme`: start with that dropdown open), `DODAI_SCROLL` (px), `DODAI_SHOT` (screenshot path), `DODAI_SHOT_FRAMES` (frames to wait, default 90), `--frames N`.

mod chrome;
mod events;
mod render;

use bevy::prelude::*;
use dodai_bevy::prelude::*;
use dodai_showcase_model::Showcase;

/// The showcase state.
#[derive(Resource)]
pub struct Model(pub Showcase);

/// The dropdown that is open, if any (`design` or `theme`).
#[derive(Resource, Default, PartialEq, Eq)]
pub struct OpenMenu(pub Option<&'static str>);

/// Which regions must be rebuilt from the model.
#[derive(Resource, Default)]
pub struct Dirty {
    pub nav: bool,
    pub content: bool,
    pub top: bool,
}

impl Dirty {
    pub fn all(&mut self) {
        self.nav = true;
        self.content = true;
        self.top = true;
    }
}

fn main() {
    let frames: Option<u32> = std::env::args()
        .skip_while(|a| a != "--frames")
        .nth(1)
        .and_then(|n| n.parse().ok());
    let model = Showcase::from_env();
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Dodai UI - Bevy".into(),
            resolution: (1400, 900).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(DodaiUiPlugin::default().with_scanlines(false))
    .insert_resource(Model(model))
    .insert_resource(OpenMenu(match std::env::var("DODAI_OPEN").as_deref() {
        Ok("design") => Some("design"),
        Ok("theme") => Some("theme"),
        _ => None,
    }))
    .insert_resource(Dirty {
        nav: true,
        content: true,
        top: true,
    })
    .add_systems(Startup, chrome::setup)
    .add_systems(
        Update,
        (
            events::hotkeys,
            events::handle,
            events::tree_toggle,
            events::sync_theme,
            events::readouts,
            chrome::rebuild,
            events::scroll,
        )
            .chain(),
    );
    if let Ok(path) = std::env::var("DODAI_SHOT") {
        let shot_frame: u32 = std::env::var("DODAI_SHOT_FRAMES")
            .ok()
            .and_then(|n| n.parse().ok())
            .unwrap_or(90);
        app.add_systems(Update, move |mut c: Commands, mut n: Local<u32>| {
            *n += 1;
            if *n == shot_frame {
                c.spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                    .observe(bevy::render::view::screenshot::save_to_disk(path.clone()));
            }
        });
    }
    if let Some(n) = frames {
        app.add_systems(
            Update,
            move |mut c: Local<u32>, mut exit: MessageWriter<AppExit>| {
                *c += 1;
                if *c >= n {
                    exit.write(AppExit::Success);
                }
            },
        );
    }
    app.run();
}
