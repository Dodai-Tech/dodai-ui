//! Things that open over or inside the page: accordions, dropdowns, context menus, hover cards,
//! comboboxes, the command palette, speed-dial menus and dialogs.

#![allow(clippy::type_complexity)]

use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;
use dodai_core::{ButtonVariant, CardVariant};

use crate::interact::{with_selected, DisclosureBody, UiDisclosure, UiSelectGroup, UiSelectable};
use crate::kit::UiKit;
use crate::kit::{kit, kit_pressable, Kit};
use crate::nav::navigation_item;
use crate::parts::{column, glyph, glyph_slot, row, wide, Glyph};
use crate::systems::FocusedInput;
use crate::widgets::{
    button, heading, text, text_input_with, ControlEvent, ControlKind, ControlPointer, TextRole,
    UiTextInput, UiTooltip,
};
use bevy::input::keyboard::{Key, KeyboardInput};

type Spawner<'a> = ChildSpawner<'a>;

fn hidden_unless(open: bool) -> Node {
    Node {
        display: if open { Display::Flex } else { Display::None },
        ..default()
    }
}

/// Sections that fold: a header with a chevron over a body. Pressing a header opens or closes
/// its body. `open` is the section shown at the start.
pub fn accordion(items: Vec<(String, String)>, open: Option<usize>) -> impl Bundle {
    (
        wide(column()),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for (i, (title, body)) in items.into_iter().enumerate() {
                let is_open = open == Some(i);
                c.spawn(wide(column())).with_children(|s| {
                    s.spawn((
                        kit_pressable(Kit::Row { selected: false }),
                        UiDisclosure { open: is_open },
                        wide(row(8.0)),
                        Children::spawn(SpawnWith(move |h: &mut Spawner| {
                            h.spawn((
                                text(title, TextRole::Control),
                                Node {
                                    flex_grow: 1.0,
                                    ..default()
                                },
                            ));
                            h.spawn(glyph_slot(
                                if is_open {
                                    Glyph::ChevronDown
                                } else {
                                    Glyph::ChevronRight
                                },
                                16,
                            ));
                        })),
                    ));
                    s.spawn((
                        DisclosureBody,
                        Node {
                            padding: UiRect::all(Val::Px(12.0)),
                            ..hidden_unless(is_open)
                        },
                        children![text(body, TextRole::Muted)],
                    ));
                });
            }
        })),
    )
}

/// A popup list that closes itself when an entry is picked.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct CloseOnSelect;

fn popup_list(labels: Vec<String>, tweak: impl FnOnce(&mut Node)) -> impl Bundle {
    let mut node = Node {
        position_type: PositionType::Absolute,
        min_width: Val::Px(160.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        display: Display::None,
        ..default()
    };
    tweak(&mut node);
    (
        kit(Kit::Popup),
        UiSelectGroup::default(),
        CloseOnSelect,
        DisclosureBody,
        node,
        ZIndex(50),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for (i, l) in labels.into_iter().enumerate() {
                c.spawn((navigation_item(l, false), UiSelectable(i)));
            }
        })),
    )
}

fn owned(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

/// A button that opens a list below it; picking an entry closes it and reports
/// [`ControlKind::Selected`] from the entry.
pub fn dropdown_menu(label: impl Into<String>, items: &[&str]) -> impl Bundle {
    dropdown_with(label.into(), owned(items), ())
}

/// [`dropdown_menu`] whose list also carries `extra` (a marker a system reacts to).
pub(crate) fn dropdown_with(label: String, items: Vec<String>, extra: impl Bundle) -> impl Bundle {
    (
        Node {
            align_self: AlignSelf::FlexStart,
            ..column()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                kit_pressable(Kit::Select),
                UiDisclosure { open: false },
                row(8.0),
                Children::spawn(SpawnWith(move |h: &mut Spawner| {
                    h.spawn((
                        text(label, TextRole::Control),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                    ));
                    h.spawn(glyph_slot(Glyph::ChevronRight, 16));
                })),
            ));
            c.spawn((popup_list(items, |n| n.top = Val::Percent(100.0)), extra));
        })),
    )
}

/// An option row of a select list: selected ones are filled; `index` is its place in the
/// [`UiSelectGroup`] it sits in.
pub fn select_option(label: impl Into<String>, selected: bool, index: usize) -> impl Bundle {
    (navigation_item(label, selected), UiSelectable(index))
}

/// A menu list shown as it is (a popup panel of entries).
pub fn menu(items: &[&str]) -> impl Bundle {
    menu_with(items, |_| {})
}

fn menu_with(items: &[&str], tweak: impl FnOnce(&mut Node)) -> impl Bundle {
    let items = owned(items);
    let mut node = Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        min_width: Val::Px(160.0),
        ..default()
    };
    tweak(&mut node);
    (
        kit(Kit::Popup),
        UiSelectGroup::default(),
        node,
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for (i, l) in items.into_iter().enumerate() {
                c.spawn((navigation_item(l, false), UiSelectable(i)));
            }
        })),
    )
}

/// An area that shows its [`context_menu`] child on a right click.
#[derive(Component, Debug, Clone, Copy, Default)]
#[require(bevy::ui::RelativeCursorPosition)]
pub struct UiContextArea;

/// The menu of a [`UiContextArea`]: hidden until the area is right-clicked, then placed at the
/// cursor; it closes when an entry is picked, on Escape or on a click elsewhere.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UiContextMenu;

/// A context menu: spawn it as a child of an area (`UiContextArea`).
pub fn context_menu(items: &[&str]) -> impl Bundle {
    (
        UiContextMenu,
        CloseOnSelect,
        menu_with(items, |n| {
            n.position_type = PositionType::Absolute;
            n.display = Display::None;
        }),
        ZIndex(60),
    )
}

pub(crate) fn context_menus(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    areas: Query<
        (&bevy::ui::RelativeCursorPosition, &ComputedNode, &Children),
        With<UiContextArea>,
    >,
    mut menus: Query<&mut Node, With<UiContextMenu>>,
) {
    let esc = keys.just_pressed(KeyCode::Escape);
    let left = mouse.just_pressed(MouseButton::Left);
    let right = mouse.just_pressed(MouseButton::Right);
    for (rel, node, kids) in &areas {
        for k in kids.iter() {
            let Ok(mut n) = menus.get_mut(k) else {
                continue;
            };
            if right && rel.cursor_over() {
                let at = rel.normalized.unwrap_or(Vec2::ZERO) + Vec2::splat(0.5);
                let size = node.size() * node.inverse_scale_factor();
                n.left = Val::Px(at.x * size.x);
                n.top = Val::Px(at.y * size.y);
                n.display = Display::Flex;
            } else if esc || (left && n.display == Display::Flex) {
                // A press on an entry is handled by CloseOnSelect; anything else closes.
                if esc {
                    n.display = Display::None;
                }
            }
        }
    }
}

pub(crate) fn close_on_select(
    groups: Query<(Entity, &ChildOf), (With<CloseOnSelect>, Changed<UiSelectGroup>)>,
    mut nodes: Query<&mut Node>,
    kids: Query<&Children>,
    mut headers: Query<&mut UiDisclosure>,
) {
    for (e, parent) in &groups {
        if let Ok(mut n) = nodes.get_mut(e) {
            if n.display != Display::None {
                n.display = Display::None;
            }
        }
        if let Ok(siblings) = kids.get(parent.parent()) {
            for s in siblings.iter() {
                if let Ok(mut h) = headers.get_mut(s) {
                    h.open = false;
                }
            }
        }
    }
}

/// A trigger that shows the [`DisclosureBody`] next to it while the pointer is over it.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct HoverReveal;

/// A chip that shows a card with `title` and `body` while hovered.
pub fn hover_card(
    trigger: impl Into<String>,
    title: impl Into<String>,
    body: impl Into<String>,
) -> impl Bundle {
    let (trigger, title, body) = (trigger.into(), title.into(), body.into());
    (
        Node {
            align_self: AlignSelf::FlexStart,
            ..column()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                kit_pressable(Kit::Chip { selected: false }),
                HoverReveal,
                children![text(trigger, TextRole::ControlSmall)],
            ));
            c.spawn((
                DisclosureBody,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    display: Display::None,
                    ..default()
                },
                ZIndex(50),
                children![crate::components::rich_tooltip(title, body)],
            ));
        })),
    )
}

pub(crate) fn hover_reveal(
    triggers: Query<(&ControlPointer, &ChildOf), (Changed<ControlPointer>, With<HoverReveal>)>,
    kids: Query<&Children>,
    mut bodies: Query<&mut Node, With<DisclosureBody>>,
) {
    for (p, parent) in &triggers {
        let show = *p != ControlPointer::None;
        if let Ok(siblings) = kids.get(parent.parent()) {
            for s in siblings.iter() {
                if let Ok(mut n) = bodies.get_mut(s) {
                    let d = if show { Display::Flex } else { Display::None };
                    if n.display != d {
                        n.display = d;
                    }
                }
            }
        }
    }
}

/// A search field with a list of options that narrows as you type; picking one fills the field.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UiCombobox {
    /// The list is always shown (a command list) instead of opening on focus.
    pub always_open: bool,
    /// Whether the list is shown now.
    pub open: bool,
    /// The option the arrow keys are on, if any.
    pub highlight: Option<Entity>,
}

/// An option of a [`UiCombobox`].
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct ComboOption(pub String);

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ComboInput;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ComboList;

fn combo(options: Vec<String>, placeholder: String, always_open: bool) -> impl Bundle {
    (
        UiCombobox {
            always_open,
            open: always_open,
            highlight: None,
        },
        Node {
            align_self: AlignSelf::Stretch,
            ..column()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                ComboInput,
                text_input_with(placeholder, |n| n.width = Val::Percent(100.0)),
            ));
            c.spawn((
                ComboList,
                kit(Kit::Popup),
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    display: if always_open {
                        Display::Flex
                    } else {
                        Display::None
                    },
                    ..default()
                },
                Children::spawn(SpawnWith(move |l: &mut Spawner| {
                    for o in options {
                        l.spawn((ComboOption(o.clone()), navigation_item(o, false)));
                    }
                })),
            ));
        })),
    )
}

/// An input with a dropdown of options filtered by what is typed.
pub fn combobox(options: &[&str], placeholder: impl Into<String>) -> impl Bundle {
    combo(owned(options), placeholder.into(), false)
}

/// A filtered command list: always open, with its search field on top.
pub fn command(items: &[&str]) -> impl Bundle {
    combo(owned(items), "Type a command…".into(), true)
}

/// A command list in a centred floating panel.
pub fn command_palette(items: &[&str]) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(15.0),
            width: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        ZIndex(200),
        children![(
            kit(Kit::Popup),
            Node {
                width: Val::Px(480.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..column()
            },
            children![command(items)],
        )],
    )
}

fn matches(label: &str, query: &str) -> bool {
    query.is_empty() || label.to_lowercase().contains(&query.to_lowercase())
}

pub(crate) fn combobox_filter(
    mut roots: Query<(&mut UiCombobox, &Children)>,
    inputs: Query<(Entity, &UiTextInput), With<ComboInput>>,
    lists: Query<&Children, With<ComboList>>,
    mut nodes: Query<&mut Node, Without<ComboList>>,
    mut list_nodes: Query<&mut Node, With<ComboList>>,
    options: Query<&ComboOption>,
    focus: Res<FocusedInput>,
) {
    for (mut root, kids) in &mut roots {
        let mut query = String::new();
        let mut focused = false;
        for k in kids.iter() {
            if let Ok((e, i)) = inputs.get(k) {
                query = i.value.clone();
                focused = focus.0 == Some(e);
            }
        }
        if focused && !root.open {
            root.open = true;
        }
        for k in kids.iter() {
            if let Ok(items) = lists.get(k) {
                if let Ok(mut ln) = list_nodes.get_mut(k) {
                    let d = if root.open || root.always_open {
                        Display::Flex
                    } else {
                        Display::None
                    };
                    if ln.display != d {
                        ln.display = d;
                    }
                }
                for item in items.iter() {
                    if let (Ok(o), Ok(mut n)) = (options.get(item), nodes.get_mut(item)) {
                        let d = if matches(&o.0, &query) {
                            Display::Flex
                        } else {
                            Display::None
                        };
                        if n.display != d {
                            n.display = d;
                        }
                    }
                }
            }
        }
    }
}

/// Arrow keys move a highlight over the options the typed text leaves; Enter picks it, Escape
/// closes the list. Acts on the combobox whose field has focus (or whose list is open).
#[allow(clippy::too_many_arguments)]
pub(crate) fn combobox_keys(
    mut keys: MessageReader<KeyboardInput>,
    focus: Res<FocusedInput>,
    mut roots: Query<(&mut UiCombobox, &Children)>,
    inputs: Query<(), With<ComboInput>>,
    lists: Query<&Children, With<ComboList>>,
    nodes: Query<&Node>,
    options: Query<&ComboOption>,
    mut kits: Query<&mut UiKit>,
    mut fields: Query<&mut UiTextInput, With<ComboInput>>,
    mut out: MessageWriter<ControlEvent>,
) {
    for k in keys.read() {
        if !k.state.is_pressed() {
            continue;
        }
        for (mut root, kids) in &mut roots {
            let field = kids.iter().find(|c| inputs.contains(*c));
            let active =
                field.is_some_and(|f| focus.0 == Some(f)) || (root.open && !root.always_open);
            if !active {
                continue;
            }
            let shown: Vec<Entity> = kids
                .iter()
                .filter_map(|c| lists.get(c).ok())
                .flat_map(|l| l.iter())
                .filter(|o| {
                    options.contains(*o) && nodes.get(*o).is_ok_and(|n| n.display != Display::None)
                })
                .collect();
            let at = root
                .highlight
                .and_then(|h| shown.iter().position(|o| *o == h));
            match &k.logical_key {
                Key::ArrowDown | Key::ArrowUp if !shown.is_empty() => {
                    let to = match (&k.logical_key, at) {
                        (Key::ArrowDown, Some(i)) => (i + 1).min(shown.len() - 1),
                        (Key::ArrowDown, None) => 0,
                        (_, Some(i)) => i.saturating_sub(1),
                        (_, None) => shown.len() - 1,
                    };
                    root.open = true;
                    root.highlight = Some(shown[to]);
                    for (i, o) in shown.iter().enumerate() {
                        if let Ok(mut kit) = kits.get_mut(*o) {
                            let next = with_selected(kit.0, i == to);
                            if kit.0 != next {
                                kit.0 = next;
                            }
                        }
                    }
                }
                Key::Enter => {
                    let Some(o) = at.map(|i| shown[i]) else {
                        continue;
                    };
                    if let (Some(f), Ok(opt)) = (field, options.get(o)) {
                        if let Ok(mut input) = fields.get_mut(f) {
                            input.value = opt.0.clone();
                        }
                    }
                    if !root.always_open {
                        root.open = false;
                    }
                    root.highlight = None;
                    if let Ok(mut kit) = kits.get_mut(o) {
                        kit.0 = with_selected(kit.0, false);
                    }
                    out.write(ControlEvent {
                        entity: o,
                        kind: ControlKind::Selected,
                    });
                }
                Key::Escape if !root.always_open => root.open = false,
                _ => {}
            }
        }
    }
}

pub(crate) fn combobox_pick(
    pressed: Query<(Entity, &ComboOption, &ControlPointer, &ChildOf), Changed<ControlPointer>>,
    parents: Query<&ChildOf>,
    kids: Query<&Children>,
    mut roots: Query<&mut UiCombobox>,
    mut inputs: Query<&mut UiTextInput, With<ComboInput>>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (e, o, p, list) in &pressed {
        if *p != ControlPointer::Pressed {
            continue;
        }
        let Ok(root) = parents.get(list.parent()) else {
            continue;
        };
        if let Ok(mut r) = roots.get_mut(root.parent()) {
            if !r.always_open {
                r.open = false;
            }
        }
        if let Ok(siblings) = kids.get(root.parent()) {
            for s in siblings.iter() {
                if let Ok(mut i) = inputs.get_mut(s) {
                    i.value = o.0.clone();
                }
            }
        }
        out.write(ControlEvent {
            entity: e,
            kind: ControlKind::Selected,
        });
    }
}

/// A speed dial: a floating button that opens a column of labelled actions above it.
pub fn fab_menu(actions: &[&str]) -> impl Bundle {
    let actions = owned(actions);
    (
        Node {
            flex_direction: FlexDirection::ColumnReverse,
            align_items: AlignItems::FlexEnd,
            row_gap: Val::Px(8.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                kit_pressable(Kit::Fab { extended: false }),
                UiDisclosure { open: false },
                Node {
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![glyph(Glyph::Plus, 24)],
            ));
            c.spawn((
                DisclosureBody,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: Val::Px(8.0),
                    display: Display::None,
                    ..default()
                },
                Children::spawn(SpawnWith(move |l: &mut Spawner| {
                    for a in actions {
                        l.spawn((
                            kit_pressable(Kit::Chip { selected: false }),
                            children![text(a, TextRole::ControlSmall)],
                        ));
                    }
                })),
            ));
        })),
    )
}

/// A button with a second part that opens a list of related actions.
pub fn split_button(label: impl Into<String>, items: &[&str]) -> impl Bundle {
    let (label, items) = (label.into(), owned(items));
    (
        Node {
            align_self: AlignSelf::FlexStart,
            flex_wrap: FlexWrap::Wrap,
            ..row(2.0)
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(button(label, ButtonVariant::Primary));
            c.spawn((
                kit_pressable(Kit::Elevated),
                UiDisclosure { open: false },
                children![glyph(Glyph::ChevronDown, 16)],
            ));
            c.spawn(popup_list(items, |n| {
                n.top = Val::Percent(100.0);
                n.right = Val::Px(0.0);
            }));
        })),
    )
}

/// A dialog card: a title, a message and a row of action buttons (the last is the primary one).
pub fn dialog_card(
    title: impl Into<String>,
    message: impl Into<String>,
    actions: &[&str],
) -> impl Bundle {
    let (title, message, actions) = (title.into(), message.into(), owned(actions));
    let last = actions.len().saturating_sub(1);
    (
        kit(Kit::Card(CardVariant::Elevated)),
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            min_width: Val::Px(280.0),
            max_width: Val::Px(480.0),
            align_items: AlignItems::Stretch,
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(heading(title));
            c.spawn(text(message, TextRole::Body));
            c.spawn((
                Node {
                    justify_content: JustifyContent::FlexEnd,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                Children::spawn(SpawnWith(move |r: &mut Spawner| {
                    for (i, a) in actions.into_iter().enumerate() {
                        r.spawn(button(
                            a,
                            if i == last {
                                ButtonVariant::Primary
                            } else {
                                ButtonVariant::Ghost
                            },
                        ));
                    }
                })),
            ));
        })),
    )
}

/// A dialog that fills the window: a bar with a close button, the title and a save button, and
/// room below for content (add it with `.with_children`).
pub fn full_screen_dialog(title: impl Into<String>) -> impl Bundle {
    let title = title.into();
    (
        kit(Kit::Card(CardVariant::Filled)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(12.0),
            ..default()
        },
        ZIndex(300),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                row(12.0),
                Children::spawn(SpawnWith(move |h: &mut Spawner| {
                    h.spawn(glyph(Glyph::Close, 20));
                    h.spawn((
                        heading(title),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                    ));
                    h.spawn(button("Save", ButtonVariant::Primary));
                })),
            ));
        })),
    )
}

/// Shows `label` near the cursor while this entity is hovered.
pub fn dodai_tooltip(label: impl Into<String>) -> impl Bundle {
    UiTooltip(label.into())
}

/// Registers the systems above.
pub(crate) fn add(app: &mut App) {
    app.add_systems(
        Update,
        (
            close_on_select,
            context_menus,
            hover_reveal,
            combobox_filter,
            combobox_keys,
            combobox_pick,
        )
            .chain()
            .after(crate::kit::press_input),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        crate::testing::component_app()
    }

    fn press(app: &mut App, e: Entity) {
        *app.world_mut().get_mut::<ControlPointer>(e).unwrap() = ControlPointer::Hovered;
        *app.world_mut().get_mut::<ControlPointer>(e).unwrap() = ControlPointer::Pressed;
    }

    #[test]
    fn accordion_opens_and_closes_its_sections() {
        let mut app = app();
        let a = app
            .world_mut()
            .spawn(accordion(
                vec![("One".into(), "a".into()), ("Two".into(), "b".into())],
                Some(0),
            ))
            .id();
        app.update();
        let sections: Vec<Entity> = app.world().get::<Children>(a).unwrap().iter().collect();
        let part = |app: &App, s: Entity, i: usize| app.world().get::<Children>(s).unwrap()[i];
        let body = |app: &App, s: Entity| app.world().get::<Node>(part(app, s, 1)).unwrap().display;
        assert_eq!(body(&app, sections[0]), Display::Flex);
        assert_eq!(body(&app, sections[1]), Display::None);
        let h1 = part(&app, sections[1], 0);
        press(&mut app, h1);
        app.update();
        assert_eq!(body(&app, sections[1]), Display::Flex);
    }

    #[test]
    fn dropdown_opens_then_closes_when_an_entry_is_picked() {
        let mut app = app();
        let d = app
            .world_mut()
            .spawn(dropdown_menu("Pick", &["a", "b"]))
            .id();
        app.update();
        let parts: Vec<Entity> = app.world().get::<Children>(d).unwrap().iter().collect();
        let (trigger, list) = (parts[0], parts[1]);
        press(&mut app, trigger);
        app.update();
        assert_eq!(
            app.world().get::<Node>(list).unwrap().display,
            Display::Flex
        );
        let item = app.world().get::<Children>(list).unwrap()[1];
        press(&mut app, item);
        app.update();
        app.update();
        assert_eq!(
            app.world().get::<Node>(list).unwrap().display,
            Display::None
        );
        assert!(!app.world().get::<UiDisclosure>(trigger).unwrap().open);
        assert_eq!(
            app.world().get::<UiSelectGroup>(list).unwrap().selected,
            Some(1)
        );
    }

    #[test]
    fn combobox_filters_and_picks() {
        let mut app = app();
        let c = app
            .world_mut()
            .spawn(combobox(&["Apple", "Banana", "Apricot"], "Fruit"))
            .id();
        app.update();
        let parts: Vec<Entity> = app.world().get::<Children>(c).unwrap().iter().collect();
        let (input, list) = (parts[0], parts[1]);
        app.world_mut().resource_mut::<FocusedInput>().0 = Some(input);
        app.world_mut().get_mut::<UiTextInput>(input).unwrap().value = "ap".into();
        app.update();
        let opts: Vec<Entity> = app.world().get::<Children>(list).unwrap().iter().collect();
        let shown: Vec<bool> = opts
            .iter()
            .map(|o| app.world().get::<Node>(*o).unwrap().display == Display::Flex)
            .collect();
        assert_eq!(shown, vec![true, false, true]);
        assert_eq!(
            app.world().get::<Node>(list).unwrap().display,
            Display::Flex
        );
        press(&mut app, opts[2]);
        app.update();
        assert_eq!(
            app.world().get::<UiTextInput>(input).unwrap().value,
            "Apricot"
        );
        assert!(!app.world().get::<UiCombobox>(c).unwrap().open);
    }

    #[test]
    fn combobox_arrows_move_and_enter_picks() {
        use bevy::input::keyboard::KeyboardInput;
        use bevy::input::ButtonState;
        let mut app = app();
        let c = app
            .world_mut()
            .spawn(combobox(&["Apple", "Banana", "Apricot"], "Fruit"))
            .id();
        app.update();
        let parts: Vec<Entity> = app.world().get::<Children>(c).unwrap().iter().collect();
        let (input, list) = (parts[0], parts[1]);
        app.world_mut().resource_mut::<FocusedInput>().0 = Some(input);
        app.world_mut().get_mut::<UiTextInput>(input).unwrap().value = "ap".into();
        app.update();
        let key = |app: &mut App, k: Key| {
            app.world_mut().write_message(KeyboardInput {
                key_code: KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified),
                logical_key: k,
                state: ButtonState::Pressed,
                text: None,
                repeat: false,
                window: Entity::PLACEHOLDER,
            });
            app.update();
        };
        key(&mut app, Key::ArrowDown);
        key(&mut app, Key::ArrowDown);
        let opts: Vec<Entity> = app.world().get::<Children>(list).unwrap().iter().collect();
        assert_eq!(
            app.world().get::<UiCombobox>(c).unwrap().highlight,
            Some(opts[2])
        );
        key(&mut app, Key::Enter);
        assert_eq!(
            app.world().get::<UiTextInput>(input).unwrap().value,
            "Apricot"
        );
        assert!(!app.world().get::<UiCombobox>(c).unwrap().open);
    }

    #[test]
    fn a_command_list_is_always_open() {
        let mut app = app();
        let c = app.world_mut().spawn(command(&["Save", "Open"])).id();
        app.update();
        assert!(app.world().get::<UiCombobox>(c).unwrap().open);
        let list = app.world().get::<Children>(c).unwrap()[1];
        assert_eq!(
            app.world().get::<Node>(list).unwrap().display,
            Display::Flex
        );
    }

    #[test]
    fn hover_card_shows_while_hovered() {
        let mut app = app();
        let h = app.world_mut().spawn(hover_card("?", "T", "B")).id();
        app.update();
        let parts: Vec<Entity> = app.world().get::<Children>(h).unwrap().iter().collect();
        *app.world_mut().get_mut::<ControlPointer>(parts[0]).unwrap() = ControlPointer::Hovered;
        app.update();
        assert_eq!(
            app.world().get::<Node>(parts[1]).unwrap().display,
            Display::Flex
        );
        *app.world_mut().get_mut::<ControlPointer>(parts[0]).unwrap() = ControlPointer::None;
        app.update();
        assert_eq!(
            app.world().get::<Node>(parts[1]).unwrap().display,
            Display::None
        );
    }

    #[test]
    fn dialogs_and_speed_dial_build() {
        let mut app = app();
        let d = app
            .world_mut()
            .spawn(dialog_card("T", "m", &["Cancel", "OK"]))
            .id();
        let f = app.world_mut().spawn(full_screen_dialog("Edit")).id();
        let m = app.world_mut().spawn(fab_menu(&["a", "b"])).id();
        let s = app.world_mut().spawn(split_button("Save", &["x"])).id();
        app.update();
        assert_eq!(app.world().get::<Children>(d).unwrap().len(), 3);
        assert_eq!(app.world().get::<Children>(f).unwrap().len(), 1);
        let body = app.world().get::<Children>(m).unwrap()[1];
        assert_eq!(app.world().get::<Children>(body).unwrap().len(), 2);
        let trigger = app.world().get::<Children>(m).unwrap()[0];
        press(&mut app, trigger);
        app.update();
        assert_eq!(
            app.world().get::<Node>(body).unwrap().display,
            Display::Flex
        );
        assert_eq!(app.world().get::<Children>(s).unwrap().len(), 3);
    }

    #[test]
    fn context_menu_opens_on_right_click_and_closes_on_escape() {
        let mut app = app();
        let area = app.world_mut().spawn((UiContextArea, Node::default())).id();
        let m = app.world_mut().spawn(context_menu(&["Copy"])).id();
        app.world_mut().entity_mut(area).add_children(&[m]);
        app.update();
        {
            let mut rel = app
                .world_mut()
                .get_mut::<bevy::ui::RelativeCursorPosition>(area)
                .unwrap();
            rel.cursor_over = true;
            rel.normalized = Some(Vec2::new(0.0, 0.0));
        }
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        app.update();
        assert_eq!(app.world().get::<Node>(m).unwrap().display, Display::Flex);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert_eq!(app.world().get::<Node>(m).unwrap().display, Display::None);
    }
}
