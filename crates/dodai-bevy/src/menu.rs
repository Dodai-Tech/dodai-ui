//! A menu bar: titles across the top, a dropdown under each, items with a shortcut written on
//! them. Press a title to open it, move across the bar to open the next, press an item to choose
//! it (a [`MenuEvent`] says which), press anywhere else or `Esc` to close. A disabled item is
//! drawn as disabled and says nothing.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use dodai_bevy::prelude::*;
//!
//! fn setup(mut c: Commands) {
//!     c.spawn(menu_bar(vec![
//!         MenuSpec::new("File", vec![
//!             MenuItem::new("open", "Open").shortcut("Ctrl+O"),
//!             MenuItem::new("quit", "Quit").separator(),
//!         ]),
//!     ]));
//! }
//! ```
//!
//! The bar and its dropdowns are made of the design's own buttons and panels, so a theme switch
//! restyles them.

use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use dodai_core::{ButtonVariant, ControlSize};

use crate::widgets::{
    divider, text, ButtonSpec, ControlEvent, ControlKind, ControlPointer, TextRole,
};

/// One entry of a dropdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    /// What [`MenuEvent`] says when it is chosen.
    pub id: String,
    /// What it is called.
    pub label: String,
    /// The shortcut written at its right end (`"Ctrl+O"`); empty for none.
    pub shortcut: String,
    /// `false` draws it disabled and makes it say nothing.
    pub enabled: bool,
    /// A line above it.
    pub separator_before: bool,
}

impl MenuItem {
    /// An enabled item.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shortcut: String::new(),
            enabled: true,
            separator_before: false,
        }
    }

    /// The shortcut written on it.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = shortcut.into();
        self
    }

    /// Enabled or not.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// A line above it.
    pub fn separator(mut self) -> Self {
        self.separator_before = true;
        self
    }
}

/// One menu: its title and its items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuSpec {
    /// The title on the bar.
    pub label: String,
    /// The items of its dropdown.
    pub items: Vec<MenuItem>,
}

impl MenuSpec {
    /// A menu.
    pub fn new(label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        Self {
            label: label.into(),
            items,
        }
    }
}

/// The bar. Spawn it with [`menu_bar`]; its titles are made the frame after.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct UiMenuBar {
    /// The menus, left to right.
    pub menus: Vec<MenuSpec>,
    /// Which dropdown is open.
    pub open: Option<usize>,
}

/// A title on the bar.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiMenuTitle(pub usize);

/// An open dropdown.
#[derive(Component, Debug, Clone, Copy)]
pub struct UiMenuPanel;

/// An item of an open dropdown.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct UiMenuEntry {
    /// The item's id.
    pub id: String,
    /// The menu it is in.
    pub menu: usize,
}

/// An item was chosen.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct MenuEvent {
    /// The item's id.
    pub id: String,
    /// The index of the menu it is in.
    pub menu: usize,
}

/// A menu bar for `menus`.
pub fn menu_bar(menus: Vec<MenuSpec>) -> impl Bundle {
    (
        UiMenuBar { menus, open: None },
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(2.0),
            ..default()
        },
        FocusPolicy::Pass,
    )
}

/// What opens when `pressed` (a title just pressed) or `hovered` (a title under the pointer)
/// happens while `open` is open: pressing the open title closes it, pressing another opens it,
/// moving over another title switches only while one is open.
#[must_use]
pub fn next_open(
    open: Option<usize>,
    pressed: Option<usize>,
    hovered: Option<usize>,
) -> Option<usize> {
    match (pressed, hovered) {
        (Some(p), _) if open == Some(p) => None,
        (Some(p), _) => Some(p),
        (None, Some(h)) if open.is_some() => Some(h),
        _ => open,
    }
}

/// Spawns the titles of bars that were just added.
pub(crate) fn build_bars(
    mut commands: Commands,
    bars: Query<(Entity, &UiMenuBar), Added<UiMenuBar>>,
) {
    for (bar, spec) in &bars {
        for (i, m) in spec.menus.iter().enumerate() {
            let title = commands
                .spawn((
                    ButtonSpec::new(m.label.clone(), ButtonVariant::Ghost)
                        .square(true)
                        .size(ControlSize::XSmall)
                        .bundle(),
                    UiMenuTitle(i),
                ))
                .id();
            commands.entity(bar).add_child(title);
        }
    }
}

fn spawn_panel(commands: &mut Commands, title: Entity, menu: usize, items: &[MenuItem]) {
    let panel = commands
        .spawn((
            crate::widgets::panel_untitled(),
            UiMenuPanel,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(100.0),
                left: Val::Px(0.0),
                min_width: Val::Px(240.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(4.0)),
                row_gap: Val::Px(1.0),
                ..default()
            },
            GlobalZIndex(1000),
            FocusPolicy::Block,
        ))
        .id();
    commands.entity(title).add_child(panel);
    for item in items {
        if item.separator_before {
            let d = commands.spawn(divider()).id();
            commands.entity(panel).add_child(d);
        }
        let row = Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(10.0), Val::Px(0.0)),
            column_gap: Val::Px(24.0),
            ..default()
        };
        let spec = ButtonSpec::new(item.label.clone(), ButtonVariant::Ghost)
            .square(true)
            .size(ControlSize::XSmall)
            .disabled(!item.enabled);
        let entry = commands
            .spawn(spec.bundle())
            .insert((
                UiMenuEntry {
                    id: item.id.clone(),
                    menu,
                },
                row,
            ))
            .id();
        if !item.shortcut.is_empty() {
            let s = commands
                .spawn(text(item.shortcut.clone(), TextRole::Caption))
                .id();
            commands.entity(entry).add_child(s);
        }
        commands.entity(panel).add_child(entry);
    }
}

/// Opens, switches, chooses and closes.
#[allow(clippy::too_many_arguments)]
pub(crate) fn drive_menus(
    mut commands: Commands,
    mut events: MessageReader<ControlEvent>,
    mut chosen: MessageWriter<MenuEvent>,
    mut bars: Query<(Entity, &mut UiMenuBar)>,
    titles: Query<(Entity, &UiMenuTitle, &ControlPointer, &ChildOf)>,
    entries: Query<(&UiMenuEntry, &ControlPointer)>,
    panels: Query<Entity, With<UiMenuPanel>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
) {
    let mut pressed_title: Option<(Entity, usize)> = None;
    let mut closing = false;
    for e in events.read() {
        if e.kind != ControlKind::Pressed {
            continue;
        }
        if let Ok((_, t, _, _)) = titles.get(e.entity) {
            pressed_title = Some((e.entity, t.0));
        } else if let Ok((entry, _)) = entries.get(e.entity) {
            chosen.write(MenuEvent {
                id: entry.id.clone(),
                menu: entry.menu,
            });
            closing = true;
        }
    }
    for (bar, mut spec) in &mut bars {
        let in_bar = |e: Entity| titles.get(e).is_ok_and(|(_, _, _, p)| p.parent() == bar);
        let hovered = titles
            .iter()
            .find(|(e, _, p, _)| in_bar(*e) && **p == ControlPointer::Hovered)
            .map(|(_, t, _, _)| t.0);
        let over_menu = titles
            .iter()
            .any(|(e, _, p, _)| in_bar(e) && *p != ControlPointer::None)
            || entries.iter().any(|(_, p)| *p != ControlPointer::None);
        let outside = mouse
            .as_ref()
            .is_some_and(|m| m.just_pressed(MouseButton::Left))
            && !over_menu;
        let escape = keys
            .as_ref()
            .is_some_and(|k| k.just_pressed(KeyCode::Escape));
        let mut next = next_open(
            spec.open,
            pressed_title.filter(|(e, _)| in_bar(*e)).map(|(_, i)| i),
            hovered,
        );
        if closing || outside || escape {
            next = None;
        }
        if next != spec.open {
            spec.open = next;
            for p in &panels {
                commands.entity(p).despawn();
            }
            if let Some(i) = next {
                if let Some((title, _, _, _)) =
                    titles.iter().find(|(e, t, _, _)| in_bar(*e) && t.0 == i)
                {
                    spawn_panel(&mut commands, title, i, &spec.menus[i].items);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::DodaiTheme;

    #[test]
    fn pressing_opens_pressing_again_closes_hovering_switches_only_when_open() {
        assert_eq!(next_open(None, Some(1), None), Some(1));
        assert_eq!(next_open(Some(1), Some(1), None), None);
        assert_eq!(next_open(Some(1), Some(2), None), Some(2));
        assert_eq!(next_open(None, None, Some(2)), None);
        assert_eq!(next_open(Some(1), None, Some(2)), Some(2));
        assert_eq!(next_open(Some(1), None, None), Some(1));
    }

    fn app() -> App {
        let mut app = App::new();
        app.insert_resource(DodaiTheme::new())
            .add_message::<ControlEvent>()
            .add_message::<MenuEvent>()
            .add_systems(Update, (build_bars, drive_menus).chain());
        app
    }

    fn bar() -> impl Bundle {
        menu_bar(vec![
            MenuSpec::new(
                "File",
                vec![
                    MenuItem::new("open", "Open").shortcut("Ctrl+O"),
                    MenuItem::new("quit", "Quit").separator().enabled(false),
                ],
            ),
            MenuSpec::new("View", vec![MenuItem::new("home", "Home")]),
        ])
    }

    fn press(app: &mut App, e: Entity) {
        app.world_mut().write_message(ControlEvent {
            entity: e,
            kind: ControlKind::Pressed,
        });
        app.update();
    }

    fn title(app: &mut App, i: usize) -> Entity {
        app.world_mut()
            .query::<(Entity, &UiMenuTitle)>()
            .iter(app.world())
            .find(|(_, t)| t.0 == i)
            .expect("title")
            .0
    }

    fn entry(app: &mut App, id: &str) -> Entity {
        app.world_mut()
            .query::<(Entity, &UiMenuEntry)>()
            .iter(app.world())
            .find(|(_, e)| e.id == id)
            .expect("entry")
            .0
    }

    fn messages(app: &mut App) -> Vec<MenuEvent> {
        let mut v = Vec::new();
        let mut cursor = app.world_mut().resource_mut::<Messages<MenuEvent>>();
        v.extend(cursor.drain());
        v
    }

    #[test]
    fn a_bar_makes_its_titles_and_a_press_opens_a_dropdown_with_the_items() {
        let mut app = app();
        app.world_mut().spawn(bar());
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&UiMenuTitle>()
                .iter(app.world())
                .count(),
            2
        );
        assert_eq!(
            app.world_mut()
                .query::<&UiMenuPanel>()
                .iter(app.world())
                .count(),
            0
        );
        let file = title(&mut app, 0);
        press(&mut app, file);
        assert_eq!(
            app.world_mut()
                .query::<&UiMenuPanel>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&UiMenuEntry>()
                .iter(app.world())
                .count(),
            2
        );
        // the other title opens its own, and only one dropdown is ever open
        let view = title(&mut app, 1);
        press(&mut app, view);
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&UiMenuPanel>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&UiMenuEntry>()
                .iter(app.world())
                .count(),
            1
        );
        // pressing the open title again closes it
        let view = title(&mut app, 1);
        press(&mut app, view);
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&UiMenuPanel>()
                .iter(app.world())
                .count(),
            0
        );
    }

    #[test]
    fn choosing_an_item_says_which_and_closes_the_menu() {
        let mut app = app();
        app.world_mut().spawn(bar());
        app.update();
        let file = title(&mut app, 0);
        press(&mut app, file);
        let open = entry(&mut app, "open");
        press(&mut app, open);
        assert_eq!(
            messages(&mut app),
            vec![MenuEvent {
                id: "open".into(),
                menu: 0
            }]
        );
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&UiMenuPanel>()
                .iter(app.world())
                .count(),
            0
        );
    }

    #[test]
    fn escape_closes_an_open_menu() {
        let mut app = app();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.world_mut().spawn(bar());
        app.update();
        let file = title(&mut app, 0);
        press(&mut app, file);
        assert_eq!(
            app.world_mut()
                .query::<&UiMenuPanel>()
                .iter(app.world())
                .count(),
            1
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&UiMenuPanel>()
                .iter(app.world())
                .count(),
            0
        );
    }
}
