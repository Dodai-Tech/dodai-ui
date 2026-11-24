//! Named constructors for the token-styled boxes: [`card`], [`alert`], [`chip`], [`avatar`],
//! [`tooltip`], [`toolbar`], [`breadcrumb`], [`pagination`], [`stepper`] and the rest.
//!
//! Each is a [`kit`](crate::kit::kit) box (so the theme restyles it) with its label children
//! already spawned. Give it a [`Node`] of your own for size; extra children go through
//! `.with_children`.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use dodai_bevy::prelude::*;
//!
//! fn setup(mut c: Commands) {
//!     c.spawn(Camera2d);
//!     c.spawn((alert(Tone::Warning, "Battery low"), Node::default()));
//!     c.spawn((breadcrumb(&["Home", "Garage", "911"]), Node::default()));
//! }
//! ```

use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;
use dodai_core::{CardVariant, StepState, Tone};

use crate::kit::{kit, kit_pressable, Kit};
use crate::widgets::{text, TextRole};

type Spawner<'a> = ChildSpawner<'a>;

fn labelled(k: Kit, label: impl Into<String>, role: TextRole) -> impl Bundle {
    let label = label.into();
    (
        kit(k),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(text(label, role));
        })),
    )
}

/// A card; add content with `.with_children`.
pub fn card(variant: CardVariant) -> impl Bundle {
    (
        kit(Kit::Card(variant)),
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )
}

/// A message bar in `tone`.
pub fn alert(tone: Tone, message: impl Into<String>) -> impl Bundle {
    labelled(
        Kit::Alert {
            tone,
            banner: false,
        },
        message,
        TextRole::Content,
    )
}

/// A solid full-width banner in `tone`.
pub fn banner(tone: Tone, message: impl Into<String>) -> impl Bundle {
    labelled(
        Kit::Alert { tone, banner: true },
        message,
        TextRole::Content,
    )
}

/// A snackbar.
pub fn snackbar(message: impl Into<String>) -> impl Bundle {
    labelled(Kit::Snackbar, message, TextRole::Content)
}

/// A toast: a floating message, drawn like a snackbar.
pub fn toast(message: impl Into<String>) -> impl Bundle {
    snackbar(message)
}

/// A filter chip.
pub fn chip(label: impl Into<String>, selected: bool) -> impl Bundle {
    (
        kit_pressable(Kit::Chip { selected }),
        Children::spawn(SpawnWith({
            let label = label.into();
            move |c: &mut Spawner| {
                c.spawn(text(label, TextRole::ControlSmall));
            }
        })),
    )
}

/// A count badge.
pub fn badge(count: impl Into<String>, tone: Tone) -> impl Bundle {
    labelled(Kit::Count(tone), count, TextRole::ControlSmall)
}

/// A small status dot; `None` is neutral.
pub fn dot(tone: Option<Tone>) -> impl Bundle {
    (
        kit(Kit::Dot(tone)),
        Node {
            width: Val::Px(8.0),
            height: Val::Px(8.0),
            ..default()
        },
    )
}

/// A round avatar with initials.
pub fn avatar(initials: impl Into<String>, size: f32) -> impl Bundle {
    (
        labelled(Kit::Avatar, initials, TextRole::ControlSmall),
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            justify_content: JustifyContent::Center,
            ..default()
        },
    )
}

/// A code listing box (monospace).
pub fn code_block(source: impl Into<String>) -> impl Bundle {
    labelled(Kit::Code, source, TextRole::Mono)
}

/// A plain tooltip bubble.
pub fn tooltip(label: impl Into<String>) -> impl Bundle {
    labelled(Kit::Tooltip { rich: false }, label, TextRole::Caption)
}

/// A tooltip with a title and a body.
pub fn rich_tooltip(title: impl Into<String>, body: impl Into<String>) -> impl Bundle {
    let (title, body) = (title.into(), body.into());
    (
        kit(Kit::Tooltip { rich: true }),
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(text(title, TextRole::Control));
            c.spawn(text(body, TextRole::Muted));
        })),
    )
}

/// A floating panel anchored by you (popover, dropdown list, menu).
pub fn popover() -> impl Bundle {
    (
        kit(Kit::Popup),
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )
}

/// A dialog: an elevated card with a title; add body and actions with `.with_children`.
pub fn dialog(title: impl Into<String>) -> impl Bundle {
    let title = title.into();
    (
        kit(Kit::Card(CardVariant::Elevated)),
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(text(title, TextRole::Heading));
        })),
    )
}

/// A key cap (`Ctrl`, `K`).
pub fn kbd(key: impl Into<String>) -> impl Bundle {
    labelled(Kit::Count(Tone::Info), key, TextRole::Mono)
}

/// A loading placeholder of a given size.
pub fn skeleton(width: f32, height: f32) -> impl Bundle {
    (
        kit(Kit::Skeleton),
        Node {
            width: Val::Px(width),
            height: Val::Px(height),
            ..default()
        },
    )
}

/// A floating action button; add an icon or label as a child.
pub fn fab(extended: bool) -> impl Bundle {
    (
        kit_pressable(Kit::Fab { extended }),
        Node {
            justify_content: JustifyContent::Center,
            ..default()
        },
    )
}

/// A bar of actions.
pub fn toolbar(floating: bool, vibrant: bool) -> impl Bundle {
    (
        kit(Kit::Toolbar { floating, vibrant }),
        Node {
            align_self: if floating {
                AlignSelf::FlexStart
            } else {
                AlignSelf::Auto
            },
            ..default()
        },
    )
}

/// A segmented control: one strip with a segment per label; `selected` is filled.
pub fn segmented(labels: &[&str], selected: usize) -> impl Bundle {
    let labels: Vec<String> = labels.iter().map(|s| s.to_string()).collect();
    (
        kit(Kit::Segmented),
        Node {
            column_gap: Val::Px(0.0),
            align_self: AlignSelf::FlexStart,
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for (i, l) in labels.into_iter().enumerate() {
                c.spawn((
                    kit_pressable(Kit::Segment {
                        selected: i == selected,
                    }),
                    children![text(l, TextRole::Control)],
                ));
            }
        })),
    )
}

/// A breadcrumb trail: the last crumb is the current page.
pub fn breadcrumb(crumbs: &[&str]) -> impl Bundle {
    let crumbs: Vec<String> = crumbs.iter().map(|s| s.to_string()).collect();
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            let last = crumbs.len().saturating_sub(1);
            for (i, crumb) in crumbs.into_iter().enumerate() {
                if i > 0 {
                    c.spawn(text("/", TextRole::Muted));
                }
                let role = if i == last {
                    TextRole::Content
                } else {
                    TextRole::Muted
                };
                c.spawn(text(crumb, role));
            }
        })),
    )
}

/// A page strip `1 … n` with `current` (zero-based) selected.
pub fn pagination(pages: usize, current: usize) -> impl Bundle {
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for p in 0..pages {
                c.spawn((
                    kit_pressable(Kit::Page {
                        selected: p == current,
                    }),
                    Node {
                        min_width: Val::Px(32.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![text((p + 1).to_string(), TextRole::Control)],
                ));
            }
        })),
    )
}

/// A horizontal stepper: numbered dots joined by the step labels; steps before `current` are done.
pub fn stepper(labels: &[&str], current: usize) -> impl Bundle {
    let labels: Vec<String> = labels.iter().map(|s| s.to_string()).collect();
    (
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for (i, l) in labels.into_iter().enumerate() {
                let state = match i.cmp(&current) {
                    std::cmp::Ordering::Less => StepState::Done,
                    std::cmp::Ordering::Equal => StepState::Current,
                    std::cmp::Ordering::Greater => StepState::Todo,
                };
                c.spawn((
                    kit(Kit::StepDot(state)),
                    Node {
                        width: Val::Px(28.0),
                        height: Val::Px(28.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![text((i + 1).to_string(), TextRole::ControlSmall)],
                ));
                c.spawn(text(l, TextRole::Content));
            }
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::UiKit;

    fn kit_of(world: &World, e: Entity) -> Kit {
        world.get::<UiKit>(e).expect("a kit box").0
    }

    #[test]
    fn named_boxes_carry_their_kit_and_label() {
        let mut w = World::new();
        let a = w.spawn(alert(Tone::Danger, "Boom")).id();
        assert_eq!(
            kit_of(&w, a),
            Kit::Alert {
                tone: Tone::Danger,
                banner: false
            }
        );
        assert_eq!(w.get::<Children>(a).map(|c| c.len()), Some(1));
        let b = w.spawn(banner(Tone::Info, "Hi")).id();
        assert!(matches!(kit_of(&w, b), Kit::Alert { banner: true, .. }));
        let t = w.spawn(rich_tooltip("T", "body")).id();
        assert_eq!(w.get::<Children>(t).map(|c| c.len()), Some(2));
        let c = w.spawn(chip("x", true)).id();
        assert_eq!(kit_of(&w, c), Kit::Chip { selected: true });
    }

    #[test]
    fn strips_spawn_one_child_per_item() {
        let mut w = World::new();
        let s = w.spawn(segmented(&["a", "b", "c"], 1)).id();
        let kids = w.get::<Children>(s).unwrap();
        assert_eq!(kids.len(), 3);
        assert_eq!(kit_of(&w, kids[1]), Kit::Segment { selected: true });
        assert_eq!(kit_of(&w, kids[0]), Kit::Segment { selected: false });
        let p = w.spawn(pagination(5, 2)).id();
        assert_eq!(w.get::<Children>(p).unwrap().len(), 5);
        // crumbs: 3 labels + 2 separators
        let b = w.spawn(breadcrumb(&["a", "b", "c"])).id();
        assert_eq!(w.get::<Children>(b).unwrap().len(), 5);
        // stepper: dot + label per step
        let st = w.spawn(stepper(&["one", "two"], 1)).id();
        let kids = w.get::<Children>(st).unwrap();
        assert_eq!(kids.len(), 4);
        assert_eq!(kit_of(&w, kids[0]), Kit::StepDot(StepState::Done));
        assert_eq!(kit_of(&w, kids[2]), Kit::StepDot(StepState::Current));
    }
}
