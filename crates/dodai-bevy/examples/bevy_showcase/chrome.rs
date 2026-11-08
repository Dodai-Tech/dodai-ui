//! The window around the sections: sidebar, control bar, scrolling page.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use dodai_bevy::kit::{kit, kit_pressable, Kit};
use dodai_bevy::prelude::*;
use dodai_bevy::theme::color;
use dodai_core::ControlSize;
use dodai_showcase_model::{Action, Showcase, CATEGORIES, SPEEDS};

use crate::events::Scroller;
use crate::render::{self, Bound, Cx};
use crate::{Dirty, Model, OpenMenu};

pub const SIDEBAR_W: f32 = 260.0;

#[derive(Component)]
pub struct NavList;
#[derive(Component)]
pub struct TopBar;
#[derive(Component)]
pub struct Page;

pub fn setup(mut commands: Commands, model: Res<Model>, theme: Res<DodaiTheme>) {
    let _ = &model;
    commands.spawn(Camera2d);
    let t = theme.theme().clone();
    let sp = t.spacing.clone();
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        })
        .with_children(|root| {
            // Sidebar.
            root.spawn((
                Node {
                    width: Val::Px(SIDEBAR_W),
                    flex_shrink: 0.0,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(sp.lg)),
                    row_gap: Val::Px(sp.md),
                    border: UiRect::right(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(color(t.palette.border)),
                SidebarBorder,
            ))
            .with_children(|s| {
                s.spawn(heading("DODAI UI"));
                s.spawn(text_input("Filter components…"))
                    .insert(Bound::Filter);
                s.spawn((
                    NavList,
                    Node {
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::scroll_y(),
                        flex_grow: 1.0,
                        min_height: Val::Px(0.0),
                        ..default()
                    },
                    Scroller::Sidebar,
                ));
                s.spawn(text("t / T theme · click a page", TextRole::Caption));
            });
            // Main column.
            root.spawn(Node {
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|m| {
                m.spawn((
                    TopBar,
                    Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(sp.lg),
                        row_gap: Val::Px(sp.sm),
                        padding: UiRect::all(Val::Px(sp.md)),
                        border: UiRect::bottom(Val::Px(1.0)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(color(t.palette.border)),
                    SidebarBorder,
                ));
                m.spawn((
                    Page,
                    Node {
                        flex_grow: 1.0,
                        min_height: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::scroll_y(),
                        padding: UiRect::all(Val::Px(sp.xl)),
                        row_gap: Val::Px(sp.lg),
                        ..default()
                    },
                    Scroller::Page,
                ));
            });
        });
}

#[derive(Component)]
pub struct MenuList;

/// Borders that follow the theme's border colour.
#[derive(Component)]
pub struct SidebarBorder;

fn segmented(
    p: &mut ChildSpawnerCommands,
    labels: &[String],
    selected: usize,
    act: impl Fn(usize) -> Action,
) {
    p.spawn(Node {
        flex_wrap: FlexWrap::Wrap,
        column_gap: Val::Px(2.0),
        row_gap: Val::Px(2.0),
        ..default()
    })
    .with_children(|c| {
        for (i, l) in labels.iter().enumerate() {
            let v = if i == selected {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Ghost
            };
            c.spawn((
                ButtonSpec::new(l.clone(), v)
                    .size(ControlSize::Small)
                    .bundle(),
                Bound::Act(act(i)),
            ));
        }
    });
}

fn field(p: &mut ChildSpawnerCommands, name: &str, body: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(Node {
        align_items: AlignItems::Center,
        column_gap: Val::Px(8.0),
        ..default()
    })
    .with_children(|c| {
        c.spawn(text(name.to_string(), TextRole::Caption));
        body(c);
    });
}

/// Labels of the on/off pickers.
pub const OFF_ON: [&str; 2] = ["Off", "On"];

/// A dropdown: the closed button (current choice) and, when open, the list below it.
fn dropdown(
    p: &mut ChildSpawnerCommands,
    name: &'static str,
    items: &[String],
    selected: usize,
    open: bool,
    act: impl Fn(usize) -> Action,
) {
    p.spawn(Node {
        width: Val::Px(260.0),
        ..default()
    })
    .with_children(|c| {
        c.spawn((
            kit_pressable(Kit::Select),
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            Bound::Menu(name),
        ))
        .with_children(|b| {
            b.spawn(text(
                items.get(selected).cloned().unwrap_or_default(),
                TextRole::Content,
            ));
            b.spawn(dodai_bevy::widgets::icon(
                if open {
                    dodai_icons::IconId::ChevronUp
                } else {
                    dodai_icons::IconId::ChevronDown
                },
                16,
            ));
        });
        if open {
            c.spawn((
                MenuList,
                kit(Kit::Popup),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(46.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    max_height: Val::Px(420.0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                GlobalZIndex(50),
                RelativeCursorPosition::default(),
                ScrollPosition::default(),
                Scroller::Menu,
            ))
            .with_children(|l| {
                for (i, it) in items.iter().enumerate() {
                    l.spawn((
                        kit_pressable(Kit::Row {
                            selected: i == selected,
                        }),
                        Node {
                            width: Val::Percent(100.0),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        Bound::Act(act(i)),
                    ))
                    .with_children(|r| {
                        r.spawn(text(it.clone(), TextRole::Content));
                    });
                }
            });
        }
    });
}

pub fn spawn_top(p: &mut ChildSpawnerCommands, m: &Showcase, open: Option<&'static str>) {
    let owned = |v: Vec<&str>| v.into_iter().map(String::from).collect::<Vec<_>>();
    field(p, "Design", |c| {
        dropdown(
            c,
            "design",
            &owned(m.design_names()),
            m.design_index(),
            open == Some("design"),
            Action::Design,
        );
    });
    field(p, "Theme", |c| {
        dropdown(
            c,
            "theme",
            &owned(m.variants()),
            m.variant_index(),
            open == Some("theme"),
            Action::Variant,
        );
    });
    let colours = m.colours();
    field(p, "Colour", |c| {
        let at = colours.iter().position(|x| *x == m.colour()).unwrap_or(0);
        let labels: Vec<String> = colours.iter().map(|x| x.label().to_string()).collect();
        segmented(c, &labels, at, |i| Action::Colour(colours[i]));
    });
    field(p, "Animate", |c| {
        segmented(c, &owned(OFF_ON.to_vec()), usize::from(m.animate), |i| {
            Action::Animate(i == 1)
        });
    });
    field(p, "Speed", |c| {
        let labels: Vec<String> = SPEEDS.iter().map(|s| format!("{s}×")).collect();
        segmented(c, &labels, m.speed_index(), Action::Speed);
    });
    field(p, "Direction", |c| {
        segmented(c, &owned(vec!["LTR", "RTL"]), usize::from(m.rtl), |i| {
            Action::Rtl(i == 1)
        });
    });
    field(p, "Patterns", |c| {
        segmented(c, &owned(OFF_ON.to_vec()), usize::from(m.patterns), |i| {
            Action::Patterns(i == 1)
        });
    });
}

pub fn spawn_nav(p: &mut ChildSpawnerCommands, m: &Showcase) {
    let item =
        |p: &mut ChildSpawnerCommands, label: &str, count: usize, on: bool, cat: Option<usize>| {
            p.spawn((
                kit_pressable(Kit::Row { selected: on }),
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                Bound::Act(Action::Category(cat)),
            ))
            .with_children(|r| {
                r.spawn(text(label.to_string(), TextRole::Control));
                r.spawn(text(count.to_string(), TextRole::Caption));
            });
        };
    item(p, "All", m.count(None), m.category().is_none(), None);
    for (i, c) in CATEGORIES.iter().enumerate() {
        let on = m.category() == Some(i);
        item(p, c, m.count(Some(i)), on, Some(i));
        if on {
            for s in m.visible() {
                p.spawn(Node {
                    padding: UiRect::new(Val::Px(36.0), Val::Px(8.0), Val::Px(4.0), Val::Px(4.0)),
                    ..default()
                })
                .with_children(|r| {
                    r.spawn(text(s.title.to_string(), TextRole::Caption));
                });
            }
        }
    }
}

pub fn spawn_page(p: &mut ChildSpawnerCommands, m: &Showcase, theme: &dodai_core::Theme) {
    let title = m.category().map_or("All", |c| CATEGORIES[c]);
    p.spawn(text(title.to_string(), TextRole::Heading));
    let sections = m.visible();
    if sections.is_empty() {
        p.spawn(text("No components match", TextRole::Muted));
    }
    let ctx = dodai_showcase_model::Ctx {
        theme,
        values: &m.values,
    };
    let cx = Cx { theme, model: m };
    for s in sections {
        let node = (s.build)(&ctx);
        p.spawn((
            panel(s.title),
            Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .with_children(|c| {
            c.spawn(Node {
                height: Val::Px(theme.spacing.sm),
                ..default()
            });
            render::spawn(c, &node, &cx);
        });
    }
}

/// The three regions that are rebuilt from the model, and the borders that follow the theme.
#[derive(bevy::ecs::system::SystemParam)]
pub struct Regions<'w, 's> {
    nav: Query<'w, 's, Entity, With<NavList>>,
    top: Query<'w, 's, Entity, With<TopBar>>,
    page: Query<'w, 's, Entity, With<Page>>,
    borders: Query<'w, 's, &'static mut BorderColor, With<SidebarBorder>>,
}

pub fn rebuild(
    mut commands: Commands,
    model: Res<Model>,
    theme: Res<DodaiTheme>,
    open: Res<OpenMenu>,
    mut dirty: ResMut<Dirty>,
    mut regions: Regions,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    let (left, fire) = crate::settle_step(dirty.settle, mouse.pressed(MouseButton::Left));
    dirty.settle = left;
    if fire {
        dirty.content = true;
    }
    if !(dirty.nav || dirty.content || dirty.top) {
        return;
    }
    let m = &model.0;
    let t = theme.theme().clone();
    if dirty.top {
        for e in &regions.top {
            commands
                .entity(e)
                .despawn_related::<Children>()
                .with_children(|p| spawn_top(p, m, open.0));
        }
        for mut b in &mut regions.borders {
            *b = BorderColor::all(color(t.palette.border));
        }
    }
    if dirty.nav {
        for e in &regions.nav {
            commands
                .entity(e)
                .despawn_related::<Children>()
                .with_children(|p| spawn_nav(p, m));
        }
    }
    if dirty.content {
        for e in &regions.page {
            commands
                .entity(e)
                .despawn_related::<Children>()
                .with_children(|p| spawn_page(p, m, &t));
        }
    }
    *dirty = Dirty::default();
}
