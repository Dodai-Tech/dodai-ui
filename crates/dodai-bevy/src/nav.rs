//! Navigation: items, bars, rails, drawers, sheets, app bars, the tree navigation and search.
//!
//! The item lists are a [`UiSelectGroup`]: pressing an item selects it, clears the others and
//! reports [`ControlKind::Selected`](crate::widgets::ControlKind::Selected).

#![allow(clippy::type_complexity)]

use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;
use dodai_core::CardVariant;

use crate::interact::{UiSelectGroup, UiSelectable};
use crate::kit::{kit, kit_pressable, Kit};
use crate::parts::{column, glyph, row, wide, Glyph};
use crate::widgets::{heading, text, TextRole};

type Spawner<'a> = ChildSpawner<'a>;

/// One navigation entry (a row; selected = the filled indicator).
pub fn navigation_item(label: impl Into<String>, selected: bool) -> impl Bundle {
    let label = label.into();
    (
        kit_pressable(Kit::Row { selected }),
        Node {
            width: Val::Percent(100.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(text(label, TextRole::Control));
        })),
    )
}

fn items(labels: Vec<String>, selected: usize) -> impl Bundle {
    Children::spawn(SpawnWith(move |c: &mut Spawner| {
        for (i, l) in labels.into_iter().enumerate() {
            c.spawn((navigation_item(l, i == selected), UiSelectable(i)));
        }
    }))
}

fn owned(labels: &[&str]) -> Vec<String> {
    labels.iter().map(|s| (*s).to_string()).collect()
}

/// A bottom navigation bar: the items side by side in a docked bar.
pub fn navigation_bar(labels: &[&str], selected: usize) -> impl Bundle {
    (
        kit(Kit::Toolbar {
            floating: false,
            vibrant: false,
        }),
        UiSelectGroup {
            selected: Some(selected),
        },
        Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceEvenly,
            ..default()
        },
        items(owned(labels), selected),
    )
}

/// A navigation rail: the items stacked in a narrow column.
pub fn navigation_rail(labels: &[&str], selected: usize) -> impl Bundle {
    (
        kit(Kit::Card(CardVariant::Filled)),
        UiSelectGroup {
            selected: Some(selected),
        },
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Px(96.0),
            row_gap: Val::Px(4.0),
            ..default()
        },
        items(owned(labels), selected),
    )
}

/// A navigation drawer: a title above the items.
pub fn navigation_drawer(title: &str, labels: &[&str], selected: usize) -> impl Bundle {
    let (title, labels) = (title.to_string(), owned(labels));
    (
        kit(Kit::Card(CardVariant::Filled)),
        UiSelectGroup {
            selected: Some(selected),
        },
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Px(280.0),
            row_gap: Val::Px(4.0),
            align_items: AlignItems::Stretch,
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(heading(title));
            for (i, l) in labels.into_iter().enumerate() {
                c.spawn((navigation_item(l, i == selected), UiSelectable(i)));
            }
        })),
    )
}

/// A drawer that sits over the page (a raised card); add a scrim yourself behind it.
pub fn modal_drawer(title: &str, labels: &[&str], selected: usize) -> impl Bundle {
    (
        navigation_drawer(title, labels, selected),
        kit(Kit::Popup),
        ZIndex(100),
    )
}

/// A top app bar: a menu button, the title, then room for actions (add children).
pub fn top_app_bar(title: impl Into<String>) -> impl Bundle {
    let title = title.into();
    (
        kit(Kit::Toolbar {
            floating: false,
            vibrant: false,
        }),
        Node {
            width: Val::Percent(100.0),
            column_gap: Val::Px(12.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(glyph(Glyph::Menu, 20));
            c.spawn((
                heading(title),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
        })),
    )
}

/// A bottom app bar: a docked bar of actions; add children (and a [`fab`](crate::components::fab)).
pub fn bottom_app_bar() -> impl Bundle {
    (
        kit(Kit::Toolbar {
            floating: false,
            vibrant: false,
        }),
        Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
    )
}

/// A side sheet: a card docked at the edge with a title; add content with `.with_children`.
pub fn side_sheet(title: impl Into<String>) -> impl Bundle {
    let title = title.into();
    (
        kit(Kit::Card(CardVariant::Elevated)),
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Px(320.0),
            height: Val::Percent(100.0),
            row_gap: Val::Px(8.0),
            align_items: AlignItems::Stretch,
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(heading(title));
        })),
    )
}

/// A bottom sheet: a card across the bottom with a drag handle; add content with `.with_children`.
pub fn bottom_sheet() -> impl Bundle {
    (
        kit(Kit::Card(CardVariant::Elevated)),
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            row_gap: Val::Px(8.0),
            align_items: AlignItems::Stretch,
            ..default()
        },
        Children::spawn(SpawnWith(|c: &mut Spawner| {
            c.spawn((
                kit(Kit::Skeleton),
                Node {
                    width: Val::Px(32.0),
                    height: Val::Px(4.0),
                    align_self: AlignSelf::Center,
                    ..default()
                },
            ));
        })),
    )
}

/// A search field with its magnifier.
pub fn search_bar(placeholder: impl Into<String>) -> impl Bundle {
    let placeholder = placeholder.into();
    (
        kit(Kit::Select),
        wide(row(8.0)),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(glyph(Glyph::Search, 18));
            c.spawn(crate::widgets::text_input_with(placeholder, |n| {
                n.flex_grow = 1.0;
            }));
        })),
    )
}

/// A search bar with its result list opened below it.
pub fn search_view(placeholder: impl Into<String>, results: &[&str]) -> impl Bundle {
    let (placeholder, results) = (placeholder.into(), owned(results));
    (
        wide(column()),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(search_bar(placeholder));
            c.spawn((
                kit(Kit::Popup),
                UiSelectGroup::default(),
                wide(column()),
                Children::spawn(SpawnWith(move |c: &mut Spawner| {
                    for (i, r) in results.into_iter().enumerate() {
                        c.spawn((navigation_item(r, false), UiSelectable(i)));
                    }
                })),
            ));
        })),
    )
}

/// A responsive app frame: the sidebar beside the content; the narrow layout hides the sidebar
/// (`UiResponsive::breakpoint`). Spawn the two panes as children in that order.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiResponsive {
    /// Window width below which the first child (the sidebar) is hidden.
    pub breakpoint: f32,
}

/// See [`UiResponsive`].
pub fn responsive_shell(breakpoint: f32) -> impl Bundle {
    (
        UiResponsive { breakpoint },
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    )
}

pub(crate) fn responsive_shells(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    shells: Query<(&UiResponsive, &Children)>,
    mut panes: Query<&mut Node>,
) {
    let Ok(w) = windows.single() else { return };
    for (r, kids) in &shells {
        let wide = w.width() >= r.breakpoint;
        if let Some(first) = kids.first() {
            if let Ok(mut n) = panes.get_mut(*first) {
                let d = if wide { Display::Flex } else { Display::None };
                if n.display != d {
                    n.display = d;
                }
            }
        }
    }
}

/// One item of a tree.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TreeItem {
    /// Label.
    pub label: String,
    /// Children; a node with some is a branch.
    pub children: Vec<TreeItem>,
    /// A branch starts open.
    pub open: bool,
    /// Starts selected.
    pub selected: bool,
}

impl TreeItem {
    /// A leaf.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..default()
        }
    }

    /// A branch holding `children`.
    pub fn branch(label: impl Into<String>, children: Vec<TreeItem>) -> Self {
        Self {
            label: label.into(),
            children,
            ..default()
        }
    }

    /// Starts open.
    #[must_use]
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Starts selected.
    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// How many items there are in this subtree, itself included.
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(TreeItem::count).sum::<usize>()
    }
}

/// A leaf row of a tree navigation: pressing it selects it in the whole tree.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UiTreeLeaf;

/// The root of a tree navigation: one selected leaf for the whole tree.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UiTreeNav;

/// One row of a tree at `depth`: a chevron for a branch (`open`), the label. Pressing a branch
/// row opens or closes the [`DisclosureBody`](crate::interact::DisclosureBody) next to it.
pub fn tree_row(
    label: impl Into<String>,
    depth: usize,
    open: Option<bool>,
    selected: bool,
) -> impl Bundle {
    let label = label.into();
    (
        kit_pressable(Kit::Row { selected }),
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::left(Val::Px(8.0 + 18.0 * depth as f32)),
            column_gap: Val::Px(6.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            match open {
                Some(o) => {
                    c.spawn(crate::parts::glyph_slot(
                        if o {
                            Glyph::ChevronDown
                        } else {
                            Glyph::ChevronRight
                        },
                        14,
                    ));
                }
                None => {
                    c.spawn(Node {
                        width: Val::Px(14.0),
                        ..default()
                    });
                }
            }
            c.spawn(text(label, TextRole::Content));
        })),
    )
}

fn tree_branch(c: &mut Spawner, item: TreeItem, depth: usize) {
    if item.children.is_empty() {
        c.spawn((tree_row(item.label, depth, None, item.selected), UiTreeLeaf));
        return;
    }
    let (label, open, kids) = (item.label, item.open, item.children);
    c.spawn(wide(column())).with_children(|s| {
        s.spawn((
            tree_row(label, depth, Some(open), false),
            crate::interact::UiDisclosure { open },
        ));
        s.spawn((
            crate::interact::DisclosureBody,
            Node {
                display: if open { Display::Flex } else { Display::None },
                ..wide(column())
            },
        ))
        .with_children(|b| {
            for k in kids {
                tree_branch(b, k, depth + 1);
            }
        });
    });
}

/// A tree: branches fold and unfold.
pub fn tree_view(items: Vec<TreeItem>) -> impl Bundle {
    (
        wide(column()),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for i in items {
                tree_branch(c, i, 0);
            }
        })),
    )
}

/// A tree that also keeps one selected leaf: a click on a leaf selects it.
pub fn tree_navigation(items: Vec<TreeItem>) -> impl Bundle {
    (UiTreeNav, tree_view(items))
}

pub(crate) fn tree_selection(
    pressed: Query<
        (Entity, &crate::widgets::ControlPointer),
        (Changed<crate::widgets::ControlPointer>, With<UiTreeLeaf>),
    >,
    parents: Query<&ChildOf>,
    roots: Query<(), With<UiTreeNav>>,
    mut leaves: Query<(Entity, &mut crate::kit::UiKit), With<UiTreeLeaf>>,
    mut out: MessageWriter<crate::widgets::ControlEvent>,
) {
    for (entity, p) in &pressed {
        if *p != crate::widgets::ControlPointer::Pressed {
            continue;
        }
        let root_of = |mut e: Entity| {
            while let Ok(c) = parents.get(e) {
                e = c.parent();
                if roots.contains(e) {
                    return Some(e);
                }
            }
            None
        };
        let Some(root) = root_of(entity) else {
            continue;
        };
        for (e, mut k) in &mut leaves {
            if root_of(e) == Some(root) {
                let next = Kit::Row {
                    selected: e == entity,
                };
                if k.0 != next {
                    k.0 = next;
                }
            }
        }
        out.write(crate::widgets::ControlEvent {
            entity,
            kind: crate::widgets::ControlKind::Selected,
        });
    }
}

/// Registers the systems above.
pub(crate) fn add(app: &mut App) {
    app.add_systems(
        Update,
        (responsive_shells, tree_selection).after(crate::kit::press_input),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interact::UiSelectGroup;
    use crate::kit::UiKit;
    use crate::widgets::ControlPointer;

    fn app() -> App {
        crate::testing::component_app()
    }

    #[test]
    fn pressing_a_navigation_item_selects_only_it() {
        let mut app = app();
        let bar = app
            .world_mut()
            .spawn(navigation_bar(&["A", "B", "C"], 0))
            .id();
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(bar).unwrap().iter().collect();
        assert_eq!(kids.len(), 3);
        *app.world_mut().get_mut::<ControlPointer>(kids[2]).unwrap() = ControlPointer::Pressed;
        app.update();
        assert_eq!(
            app.world().get::<UiSelectGroup>(bar).unwrap().selected,
            Some(2)
        );
        let sel: Vec<bool> = kids
            .iter()
            .map(|k| {
                matches!(
                    app.world().get::<UiKit>(*k).unwrap().0,
                    Kit::Row { selected: true }
                )
            })
            .collect();
        assert_eq!(sel, vec![false, false, true]);
    }

    #[test]
    fn tree_counts_and_folds() {
        let t = TreeItem::branch(
            "root",
            vec![
                TreeItem::new("a"),
                TreeItem::branch("b", vec![TreeItem::new("c")]),
            ],
        );
        assert_eq!(t.count(), 4);
        let mut app = app();
        let tree = app.world_mut().spawn(tree_view(vec![t])).id();
        app.update();
        // root section: header + body (closed)
        let section = app.world().get::<Children>(tree).unwrap()[0];
        let parts: Vec<Entity> = app
            .world()
            .get::<Children>(section)
            .unwrap()
            .iter()
            .collect();
        let header = parts[0];
        let body = parts[1];
        assert_eq!(
            app.world().get::<Node>(body).unwrap().display,
            Display::None
        );
        *app.world_mut().get_mut::<ControlPointer>(header).unwrap() = ControlPointer::Pressed;
        app.update();
        assert_eq!(
            app.world().get::<Node>(body).unwrap().display,
            Display::Flex
        );
        *app.world_mut().get_mut::<ControlPointer>(header).unwrap() = ControlPointer::Hovered;
        *app.world_mut().get_mut::<ControlPointer>(header).unwrap() = ControlPointer::Pressed;
        app.update();
        assert_eq!(
            app.world().get::<Node>(body).unwrap().display,
            Display::None
        );
    }

    #[test]
    fn tree_navigation_keeps_one_selected_leaf() {
        let mut app = app();
        let tree = app
            .world_mut()
            .spawn(tree_navigation(vec![
                TreeItem::new("x").selected(true),
                TreeItem::new("y"),
            ]))
            .id();
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(tree).unwrap().iter().collect();
        *app.world_mut().get_mut::<ControlPointer>(kids[1]).unwrap() = ControlPointer::Pressed;
        app.update();
        let on = |app: &App, e: Entity| {
            matches!(
                app.world().get::<UiKit>(e).unwrap().0,
                Kit::Row { selected: true }
            )
        };
        assert!(!on(&app, kids[0]) && on(&app, kids[1]));
    }
}
