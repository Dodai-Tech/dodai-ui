//! Every Dodai UI component in Bevy, with a theme switcher and glow
//! controls. Keys: `t`/`T` next/previous theme, `+`/`-` glow, `Esc` quit
//! (when no text input has focus). Pass `--frames N` to exit after N frames.

use bevy::prelude::*;
use dodai_bevy::prelude::*;

#[derive(Component)]
struct ThemeButton(usize);
#[derive(Component)]
struct GlowButton(f32);
#[derive(Component)]
struct ThemeName;
#[derive(Component)]
struct GlowReadout;
#[derive(Component)]
struct ProgressDemo;
#[derive(Component)]
struct SliderValue;

fn main() {
    let frames: Option<u32> = std::env::args()
        .skip_while(|a| a != "--frames")
        .nth(1)
        .and_then(|n| n.parse().ok());
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Dodai UI - Bevy".into(),
            resolution: (1280, 900).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(DodaiUiPlugin::default())
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            theme_hotkeys,
            on_events,
            labels,
            animate_progress,
            slider_label,
        ),
    );
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

fn setup(mut commands: Commands, theme: Res<DodaiTheme>) {
    commands.spawn(Camera2d);
    let names: Vec<String> = theme
        .switcher()
        .themes()
        .iter()
        .map(|t| ascii(t.name()))
        .collect();
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|root| {
            // Header: title + theme switcher + glow controls.
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|h| {
                for n in ["home", "search", "settings", "favorite"] {
                    h.spawn(icon(n, 24));
                }
                h.spawn(heading("Dodai UI - Bevy"));
                h.spawn((ThemeName, muted("")));
                for (i, n) in names.iter().enumerate() {
                    h.spawn((ThemeButton(i), button(n.clone(), ButtonVariant::Outline)));
                }
                h.spawn((GlowButton(-0.25), button("Glow -", ButtonVariant::Ghost)));
                h.spawn((GlowButton(0.25), button("Glow +", ButtonVariant::Ghost)));
                h.spawn((GlowReadout, mono("")));
            });

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(16.0),
                row_gap: Val::Px(16.0),
                align_items: AlignItems::FlexStart,
                ..default()
            })
            .with_children(|cols| {
                cols.spawn((
                    panel("Buttons"),
                    Node {
                        width: Val::Px(300.0),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    for (l, v) in [
                        ("Primary", ButtonVariant::Primary),
                        ("Secondary", ButtonVariant::Secondary),
                        ("Outline", ButtonVariant::Outline),
                        ("Outline 2", ButtonVariant::OutlineSecondary),
                        ("Danger", ButtonVariant::Danger),
                        ("Ghost", ButtonVariant::Ghost),
                    ] {
                        p.spawn(button(l, v));
                    }
                    p.spawn(button_disabled("Disabled", ButtonVariant::Primary));
                    p.spawn((
                        button("Hover for tooltip", ButtonVariant::Outline),
                        UiTooltip("Tooltips take their look from tokens".into()),
                    ));
                });

                cols.spawn((
                    panel("Tags · tabs · list"),
                    Node {
                        width: Val::Px(340.0),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn(Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(6.0),
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|r| {
                        for (l, t) in [
                            ("Info", Tone::Info),
                            ("Success", Tone::Success),
                            ("Warning", Tone::Warning),
                            ("Danger", Tone::Danger),
                        ] {
                            r.spawn(tag(l, t, true));
                            r.spawn(tag(l, t, false));
                        }
                    });
                    p.spawn(divider());
                    p.spawn(tabs(&["Overview", "Specs", "Log"], 0));
                    p.spawn(divider());
                    for (i, l) in ["Yellow", "Blue", "Green"].iter().enumerate() {
                        p.spawn(list_item(*l, i == 0));
                    }
                });

                cols.spawn((
                    panel("Controls"),
                    Node {
                        width: Val::Px(340.0),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    row(p, |r| {
                        r.spawn(switch(true));
                        r.spawn(label("Switch"));
                        r.spawn(switch(false));
                    });
                    row(p, |r| {
                        r.spawn(checkbox(true));
                        r.spawn(label("Checkbox"));
                        r.spawn(checkbox(false));
                        r.spawn(toggle_disabled(ToggleKind::Checkbox, true));
                    });
                    row(p, |r| {
                        r.spawn(radio(true));
                        r.spawn(radio(false));
                        r.spawn(radio(false));
                        r.spawn(label("Radio group"));
                    });
                    p.spawn(label("Progress"));
                    p.spawn((ProgressDemo, progress(0.6, None)));
                    p.spawn(progress(0.8, Some(Tone::Success)));
                    p.spawn(progress(0.35, Some(Tone::Danger)));
                    p.spawn((SliderValue, muted("Slider")));
                    p.spawn(slider(40.0, 0.0, 100.0));
                    p.spawn(label("Text input"));
                    p.spawn(text_input("type here..."));
                });

                cols.spawn((
                    panel("Typography"),
                    Node {
                        width: Val::Px(300.0),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn(heading("Heading"));
                    p.spawn(label("Label text"));
                    p.spawn(body("Body text follows size_md."));
                    p.spawn(muted("Muted secondary text"));
                    p.spawn(mono("mono: 0xDEADBEEF"));
                    p.spawn(divider());
                    p.spawn(text("Caption", TextRole::Caption));
                });
            });
        });
}

/// The bundled default font has no `·`; swap it for ASCII.
fn ascii(s: &str) -> String {
    s.replace('·', "-")
}

fn row(p: &mut ChildSpawnerCommands, f: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(10.0),
        ..default()
    })
    .with_children(f);
}

fn on_events(
    mut ev: MessageReader<ControlEvent>,
    themes: Query<&ThemeButton>,
    glows: Query<&GlowButton>,
    mut theme: ResMut<DodaiTheme>,
) {
    for e in ev.read() {
        if e.kind != ControlKind::Pressed {
            continue;
        }
        if let Ok(t) = themes.get(e.entity) {
            theme.select_index(t.0);
        }
        if let Ok(g) = glows.get(e.entity) {
            let v = (theme.glow_intensity() + g.0).max(0.0);
            theme.set_glow_intensity(v);
        }
    }
}

fn labels(
    theme: Res<DodaiTheme>,
    mut name: Query<&mut UiText, (With<ThemeName>, Without<GlowReadout>)>,
    mut glow: Query<&mut UiText, With<GlowReadout>>,
) {
    if !theme.is_changed() {
        return;
    }
    for mut n in &mut name {
        n.raw = format!("theme: {}", ascii(theme.name()));
    }
    for mut g in &mut glow {
        g.raw = format!("glow {:.2}", theme.glow_intensity());
    }
}

fn animate_progress(time: Res<Time>, mut q: Query<&mut UiProgress, With<ProgressDemo>>) {
    for mut p in &mut q {
        p.value = 0.5 + 0.5 * (time.elapsed_secs() * 0.8).sin();
    }
}

fn slider_label(
    sliders: Query<&UiSlider, Changed<UiSlider>>,
    mut q: Query<&mut UiText, With<SliderValue>>,
) {
    for s in &sliders {
        for mut t in &mut q {
            t.raw = format!("Slider: {:.0}", s.value);
        }
    }
}
