//! The window around the sections: sidebar, control bar, scrolling page.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use dodai_bevy::kit::{kit, kit_pressable, Kit};
use dodai_bevy::prelude::*;
use dodai_bevy::theme::color;
use dodai_showcase_model::{Action, Showcase, CATEGORIES, SPEEDS};

use crate::events::Scroller;
use crate::render::{self, Bound, Cx};
use crate::{Dirty, Model, OpenMenu};

pub const SIDEBAR_W: f32 = 260.0;
/// Width of the sidebar folded to its icon rail.
pub const RAIL_W: f32 = 72.0;

/// The sidebar column (its width follows the rail).
#[derive(Component)]
pub struct SideBar;

/// Sidebar parts the rail hides (title, filter, keys hint).
#[derive(Component)]
pub struct FullOnly;

#[derive(Component)]
pub struct NavList;
#[derive(Component)]
pub struct TopBar;
#[derive(Component)]
pub struct Page;

/// The live-preview strip pinned above the Shape and Surface-effects pages.
#[derive(Component)]
pub struct Pinned;

pub fn setup(mut commands: Commands, model: Res<Model>, theme: Res<DodaiTheme>) {
    let _ = &model;
    commands.spawn(Camera2d);
    let t = theme.theme().clone();
    let sp = t.spacing.clone();
    commands
        .spawn(Node {
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            ..default()
        })
        .with_children(|root| {
            // The backdrop (the design's mesh or the frost demo's picture) behind everything.
            root.spawn((
                UiBackdrop,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
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
                SideBar,
            ))
            .with_children(|s| {
                s.spawn((
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(sp.sm),
                        ..default()
                    },
                    FullOnly,
                ))
                .with_children(|title| {
                    render::logo(title, t.typography.size_xl * 1.6);
                    title.spawn(heading("DODAI UI"));
                });
                s.spawn(text_input("Filter components…"))
                    .insert((Bound::Filter, FullOnly));
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
                s.spawn((
                    FullOnly,
                    text(dodai_showcase_model::catalog::KEYS_HINT, TextRole::Caption),
                ));
            });
            // Main column.
            root.spawn(Node {
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                height: Val::Percent(100.0),
                min_height: Val::Px(0.0),
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
                    Pinned,
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_shrink: 0.0,
                        ..default()
                    },
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

/// What a dropdown shows.
#[derive(Clone, Copy)]
pub struct Dropdown<'a> {
    pub name: &'static str,
    pub items: &'a [String],
    pub selected: Option<usize>,
    pub placeholder: &'a str,
    pub width: f32,
    pub open: bool,
}

/// A dropdown: the closed button (current choice) and, when open, the list below it.
pub fn dropdown(p: &mut ChildSpawnerCommands, d: &Dropdown, act: impl Fn(usize) -> Action) {
    let Dropdown {
        name,
        items,
        selected,
        placeholder,
        width,
        open,
    } = *d;
    p.spawn(Node {
        width: Val::Px(width),
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
                selected
                    .and_then(|i| items.get(i).cloned())
                    .unwrap_or_else(|| placeholder.to_string()),
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
                            selected: Some(i) == selected,
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
    let t = m.theme();
    let owned = |v: Vec<&str>| v.into_iter().map(String::from).collect::<Vec<_>>();
    field(p, "Design", |c| {
        dropdown(
            c,
            &Dropdown {
                name: "design",
                items: &owned(m.design_names()),
                selected: Some(m.design_index()),
                placeholder: "",
                width: 260.0,
                open: open == Some("design"),
            },
            Action::Design,
        );
    });
    field(p, "Theme", |c| {
        dropdown(
            c,
            &Dropdown {
                name: "theme",
                items: &owned(m.variants()),
                selected: Some(m.variant_index()),
                placeholder: "",
                width: 260.0,
                open: open == Some("theme"),
            },
            Action::Variant,
        );
    });
    let colours = m.colours();
    field(p, "Colour", |c| {
        let at = colours.iter().position(|x| *x == m.colour()).unwrap_or(0);
        let labels: Vec<String> = colours.iter().map(|x| x.label().to_string()).collect();
        crate::render::segmented(c, &t, &labels, at, |i| Action::Colour(colours[i]));
    });
    field(p, "Animate", |c| {
        crate::render::segmented(
            c,
            &t,
            &owned(OFF_ON.to_vec()),
            usize::from(m.animate),
            |i| Action::Animate(i == 1),
        );
    });
    field(p, "Speed", |c| {
        let labels: Vec<String> = SPEEDS.iter().map(|s| format!("{s}×")).collect();
        crate::render::segmented(c, &t, &labels, m.speed_index(), Action::Speed);
    });
    field(p, "Direction", |c| {
        crate::render::segmented(c, &t, &owned(vec!["LTR", "RTL"]), usize::from(m.rtl), |i| {
            Action::Rtl(i == 1)
        });
    });
    field(p, "Patterns", |c| {
        crate::render::segmented(
            c,
            &t,
            &owned(OFF_ON.to_vec()),
            usize::from(m.patterns),
            |i| Action::Patterns(i == 1),
        );
    });
}

pub fn spawn_nav(p: &mut ChildSpawnerCommands, m: &Showcase) {
    use dodai_icons::IconId;
    let rail = m.rail();
    // The fold button: right of the title, or on top of the rail.
    p.spawn(Node {
        justify_content: if rail {
            JustifyContent::Center
        } else {
            JustifyContent::FlexEnd
        },
        ..default()
    })
    .with_children(|r| {
        r.spawn((
            ButtonSpec::new(if rail { "»" } else { "«" }, ButtonVariant::Ghost)
                .size(dodai_core::ControlSize::Small)
                .bundle(),
            Bound::Act(Action::Press(Showcase::rail_toggle())),
        ));
    });
    if rail {
        let on_all = m.category().is_none();
        for (label, on, cat) in std::iter::once(("All", on_all, None)).chain(
            CATEGORIES
                .iter()
                .enumerate()
                .map(|(i, c)| (*c, m.category() == Some(i), Some(i))),
        ) {
            p.spawn((
                kit_pressable(Kit::Row { selected: on }),
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Bound::Act(Action::Category(cat)),
            ))
            .with_children(|r| {
                r.spawn(icon(
                    dodai_showcase_model::catalog::category_icon(label),
                    20,
                ));
            });
        }
        return;
    }
    let item =
        |p: &mut ChildSpawnerCommands, label: &str, count: usize, on: bool, cat: Option<usize>| {
            p.spawn((
                kit_pressable(Kit::Row { selected: on }),
                Node {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                Bound::Act(Action::Category(cat)),
            ))
            .with_children(|r| {
                r.spawn(icon(
                    dodai_showcase_model::catalog::category_icon(label),
                    16,
                ));
                r.spawn((
                    text(label.to_string(), TextRole::Control),
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                ));
                r.spawn(text(count.to_string(), TextRole::Caption));
                // A category shows whether its sections are listed under it.
                if let Some(c) = cat {
                    r.spawn(icon(
                        if m.lists(c) {
                            IconId::ChevronDown
                        } else {
                            IconId::ChevronRight
                        },
                        14,
                    ));
                }
            });
        };
    item(
        p,
        "All",
        m.count(None),
        m.current_category().is_none(),
        None,
    );
    for (i, c) in CATEGORIES.iter().enumerate() {
        let on = m.current_category() == Some(i);
        item(p, c, m.count(Some(i)), on, Some(i));
        if m.lists(i) {
            let active = m.active_section();
            for s in m.listed(i) {
                p.spawn((
                    kit_pressable(Kit::Row {
                        selected: active == Some(s.title),
                    }),
                    Node {
                        width: Val::Percent(100.0),
                        margin: UiRect::left(Val::Px(16.0)),
                        ..default()
                    },
                    Bound::Act(Action::Section(s.title)),
                ))
                .with_children(|r| {
                    r.spawn(text(s.title.to_string(), TextRole::ControlSmall));
                });
            }
        }
    }
}

pub fn spawn_page(
    p: &mut ChildSpawnerCommands,
    m: &Showcase,
    theme: &dodai_core::Theme,
    open: Option<&'static str>,
) {
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
    let cx = Cx {
        theme,
        model: m,
        open,
    };
    if let Some(o) = m.overview("bevy") {
        render::spawn(p, &o, &cx);
    }
    for s in sections {
        let node = (s.build)(&ctx);
        p.spawn((
            panel(s.heading),
            Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                ..default()
            },
            crate::events::SectionFx::default(),
            crate::events::SectionAnchor(s.title),
        ))
        .with_children(|c| {
            c.spawn(Node {
                height: Val::Px(theme.spacing.sm),
                ..default()
            });
            if s.live {
                // Rebuilt on its own as time passes (crate::events::live_refresh).
                c.spawn((
                    crate::events::LiveBody(s.title),
                    Node {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                ))
                .with_children(|c| render::spawn(c, &node, &cx));
            } else {
                render::spawn(c, &node, &cx);
            }
        });
    }
}

/// The live preview of the Shape and Surface-effects pages, with a rule under it.
fn spawn_pinned(
    p: &mut ChildSpawnerCommands,
    m: &Showcase,
    theme: &dodai_core::Theme,
    open: Option<&'static str>,
) {
    let Some(node) = m.pinned() else {
        return;
    };
    let sp = &theme.spacing;
    let cx = Cx {
        theme,
        model: m,
        open,
    };
    p.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(Val::Px(sp.lg), Val::Px(sp.md)),
            border: UiRect::bottom(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(color(theme.palette.border)),
        SidebarBorder,
    ))
    .with_children(|c| render::spawn(c, &node, &cx));
}

/// The three regions that are rebuilt from the model, and the borders that follow the theme.
#[derive(bevy::ecs::system::SystemParam)]
pub struct Regions<'w, 's> {
    nav: Query<'w, 's, Entity, With<NavList>>,
    top: Query<'w, 's, Entity, With<TopBar>>,
    page: Query<'w, 's, Entity, With<Page>>,
    pinned: Query<'w, 's, Entity, With<Pinned>>,
    side: Query<'w, 's, &'static mut Node, (With<SideBar>, Without<FullOnly>)>,
    full: Query<'w, 's, &'static mut Node, (With<FullOnly>, Without<SideBar>)>,
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
        let rail = m.rail();
        for mut n in &mut regions.side {
            n.width = Val::Px(if rail { RAIL_W } else { SIDEBAR_W });
            n.padding = UiRect::all(Val::Px(if rail { t.spacing.xs } else { t.spacing.lg }));
        }
        for mut n in &mut regions.full {
            n.display = if rail { Display::None } else { Display::Flex };
        }
    }
    if dirty.content {
        for e in &regions.page {
            commands
                .entity(e)
                .despawn_related::<Children>()
                .with_children(|p| spawn_page(p, m, &t, open.0));
        }
        for e in &regions.pinned {
            commands
                .entity(e)
                .despawn_related::<Children>()
                .with_children(|p| spawn_pinned(p, m, &t, open.0));
        }
    }
    *dirty = Dirty::default();
}
