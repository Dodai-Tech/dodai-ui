//! Layers and behaviours: frost, glow and scanline layers, the travelling selection marker,
//! entry animations, overscroll, lazy culling and pointer state.

#![allow(clippy::type_complexity)]

use std::time::Duration;

use bevy::ecs::spawn::SpawnWith;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use dodai_core::motion::{EntrySet, Motion, Stretch};
use dodai_core::overscroll::{Overscroll, OverscrollFeel};
use dodai_core::SurfaceClass;

use crate::glass::UiGlass;
use crate::interact::{UiSelectGroup, UiSelectable};
use crate::material::ScanlineMaterial;
use crate::surface::Surface;
use crate::theme::{color, DodaiTheme};
use crate::widgets::{text, ControlPointer, TextRole};

type Spawner<'a> = ChildSpawner<'a>;

/// What the pointer is doing to a control: none, over, pressed.
pub type Interact = ControlPointer;

fn fill_parent() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        top: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

/// Frosted glass under a fill: a layer that covers its parent and shows the blurred backdrop
/// (the app's [`DodaiGlass`](crate::glass::DodaiGlass) picture or the design's mesh).
pub fn frost_layer() -> impl Bundle {
    (
        fill_parent(),
        Surface::default(),
        UiGlass::of(SurfaceClass::Panel).with(dodai_core::Surface::frosted()),
        Pickable::IGNORE,
    )
}

/// A box that holds a picture-sized scene; add [`frosted_panel`]s (and anything else) as
/// children. The picture itself is the app backdrop ([`UiBackdrop`](crate::glass::UiBackdrop)).
pub fn frosted_scene(w: f32, h: f32) -> impl Bundle {
    Node {
        width: Val::Px(w),
        height: Val::Px(h),
        flex_shrink: 0.0,
        overflow: Overflow::clip(),
        ..default()
    }
}

/// A frosted panel at `(x, y)` of size `(w, h)` inside a [`frosted_scene`]: padding for its
/// content and the design's frost edge.
pub fn frosted_panel(x: f32, y: f32, w: f32, h: f32) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(x),
            top: Val::Px(y),
            width: Val::Px(w),
            height: Val::Px(h),
            padding: UiRect::all(Val::Px(12.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        Surface::default(),
        UiGlass::of(SurfaceClass::Panel).with(dodai_core::Surface::frosted()),
    )
}

/// One layer of a glow: a soft shadow of `colour` around its parent, `blur` px wide.
pub fn glow_layer(colour: Color, spread: f32, blur: f32) -> impl Bundle {
    (
        fill_parent(),
        BoxShadow::new(
            colour,
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(spread),
            Val::Px(blur),
        ),
        Pickable::IGNORE,
    )
}

/// The CRT scanline overlay; place it last under the root. The plugin shows or hides it with the
/// design ([`Effects::scanlines`](dodai_core::Effects)).
pub fn scanlines(mats: &mut Assets<ScanlineMaterial>) -> impl Bundle {
    (
        crate::plugin::ScanlineOverlay,
        Node {
            display: Display::None,
            ..fill_parent()
        },
        MaterialNode(mats.add(ScanlineMaterial {
            color: LinearRgba::BLACK,
            params: Vec4::new(3.0, 0.34, 0.0, 0.0),
        })),
        GlobalZIndex(1000),
        Pickable::IGNORE,
    )
}

// ------------------------------------------------------------------ selection track

/// What the travelling indicator looks like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionMarker {
    /// A filled block behind the chosen item.
    Fill,
    /// The design's underline band at the foot of the chosen item.
    Underline,
}

/// A row of choices with an indicator that glides to the chosen item (and follows the pointer
/// while it is over one). The items are children with [`UiSelectable`]; the marker is the first
/// child.
#[derive(Component, Clone, Debug)]
pub struct UiSelectionTrack {
    /// The indicator's look.
    pub marker: SelectionMarker,
    glide: Option<Stretch>,
    target: Option<usize>,
}

/// The indicator of a [`UiSelectionTrack`].
#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct TrackMarker;

/// The item a [`UiSelectionTrack`]'s indicator heads for changed: the hovered one, else the
/// selected one; `None` when nothing is.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Follow {
    /// The track.
    pub entity: Entity,
    /// The item index.
    pub index: Option<usize>,
}

/// A row of `labels` with a travelling `marker` under or behind the chosen one.
pub fn selection_track(labels: &[&str], selected: usize, marker: SelectionMarker) -> impl Bundle {
    let labels: Vec<String> = labels.iter().map(|s| (*s).to_string()).collect();
    (
        UiSelectionTrack {
            marker,
            glide: None,
            target: Some(selected),
        },
        UiSelectGroup {
            selected: Some(selected),
        },
        Node {
            column_gap: Val::Px(4.0),
            align_items: AlignItems::Center,
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                TrackMarker,
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor::default(),
                ZIndex(-1),
                Pickable::IGNORE,
            ));
            for (i, l) in labels.into_iter().enumerate() {
                c.spawn((
                    UiSelectable(i),
                    ControlPointer::default(),
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        ..default()
                    },
                    children![text(l, TextRole::Control)],
                ));
            }
        })),
    )
}

/// Which item the indicator heads for: the hovered one, else the selected one.
pub fn follow_target(hovered: Option<usize>, selected: Option<usize>) -> Option<usize> {
    hovered.or(selected)
}

/// `(left, width)` in logical px of an item inside its track, from physical centres and widths.
pub fn rel_span(
    item_centre: f32,
    item_w: f32,
    track_centre: f32,
    track_w: f32,
    inv: f32,
) -> (f32, f32) {
    let left = (item_centre - item_w / 2.0) - (track_centre - track_w / 2.0);
    (left * inv, item_w * inv)
}

#[allow(clippy::type_complexity)]
pub(crate) fn track_follow(
    time: Res<Time>,
    theme: Res<DodaiTheme>,
    mut tracks: Query<(
        Entity,
        &mut UiSelectionTrack,
        &UiSelectGroup,
        &Children,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    items: Query<
        (
            &UiSelectable,
            &ControlPointer,
            &ComputedNode,
            &UiGlobalTransform,
        ),
        Without<TrackMarker>,
    >,
    mut markers: Query<(&mut Node, &mut BackgroundColor), With<TrackMarker>>,
    mut out: MessageWriter<Follow>,
) {
    let now = time.elapsed_secs();
    for (e, mut track, group, kids, tn, tg) in &mut tracks {
        let hovered = kids
            .iter()
            .filter_map(|k| items.get(k).ok())
            .find(|(_, p, _, _)| **p != ControlPointer::None)
            .map(|(s, _, _, _)| s.0);
        let target = follow_target(hovered, group.selected);
        if track.target != target {
            track.target = target;
            out.write(Follow {
                entity: e,
                index: target,
            });
        }
        let Some(idx) = target else { continue };
        let Some((_, _, cn, gt)) = kids
            .iter()
            .filter_map(|k| items.get(k).ok())
            .find(|(s, _, _, _)| s.0 == idx)
        else {
            continue;
        };
        let inv = tn.inverse_scale_factor();
        let (x, w) = rel_span(
            gt.translation.x,
            cn.size().x,
            tg.translation.x,
            tn.size().x,
            inv,
        );
        let m = Motion::default();
        match &mut track.glide {
            Some(g) => g.to((x, w), &m, now),
            g @ None => *g = Some(Stretch::new((x, w), &m)),
        }
        let (gx, gw) = track.glide.as_ref().map_or((x, w), |g| g.value(now));
        let h = cn.size().y * inv;
        for k in kids.iter() {
            if let Ok((mut n, mut bg)) = markers.get_mut(k) {
                let (top, height, fill) = match track.marker {
                    SelectionMarker::Fill => (0.0, h, theme.palette.primary.with_alpha(0.25)),
                    SelectionMarker::Underline => (h - 2.0, 2.0, theme.palette.primary),
                };
                n.left = Val::Px(gx);
                n.width = Val::Px(gw);
                n.top = Val::Px(top);
                n.height = Val::Px(height);
                let c = color(fill);
                if bg.0 != c {
                    bg.0 = c;
                }
            }
        }
    }
}

// ------------------------------------------------------------------ entry animation

/// Gives every child of this node an entry animation of `set` instead of appearing at once.
/// Offsets animate (rise, drop, slide); bevy_ui 0.20 has no node opacity, so a plain fade
/// shows no motion.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiEntryScope(pub EntrySet);

/// The animation of one child, from its start.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct EntryAnim {
    set: EntrySet,
    t: f32,
}

/// A node whose children enter with `set`.
pub fn entry_scope(set: impl Into<EntrySet>) -> impl Bundle {
    (
        UiEntryScope(set.into()),
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )
}

pub(crate) fn entry_start(
    scopes: Query<(&UiEntryScope, &Children)>,
    started: Query<(), With<EntryAnim>>,
    mut commands: Commands,
) {
    for (s, kids) in &scopes {
        for k in kids.iter() {
            if !started.contains(k) {
                commands.entity(k).insert(EntryAnim { set: s.0, t: 0.0 });
            }
        }
    }
}

const ENTRY_DEFAULT: Duration = Duration::from_millis(220);

pub(crate) fn entry_step(time: Res<Time>, mut q: Query<(&mut EntryAnim, &mut UiTransform)>) {
    for (mut a, mut tf) in &mut q {
        let total = a.set.duration(ENTRY_DEFAULT).as_secs_f32();
        if a.t >= total {
            continue;
        }
        a.t = (a.t + time.delta_secs()).min(total);
        let (_, dx, dy) = a.set.frame(a.t / total, ENTRY_DEFAULT);
        tf.translation = Val2::px(dx, dy);
    }
}

// ------------------------------------------------------------------ overscroll

/// Rubber-band stretch when a [`UiScrollArea`](crate::data::UiScrollArea) is pushed past an end.
#[derive(Component, Clone, Copy, Debug)]
pub struct UiOverscroll(pub Overscroll);

/// The feel a theme gets: soft for rounded designs, snappy for the others.
pub fn overscroll_feel(theme: &dodai_core::Theme) -> OverscrollFeel {
    Overscroll::for_theme(theme)
}

/// Overscroll with `feel` (add it to a scroll area).
pub fn overscroll(feel: OverscrollFeel) -> UiOverscroll {
    UiOverscroll(Overscroll::new(feel))
}

pub(crate) fn overscroll_wheel(
    time: Res<Time>,
    mut wheel: MessageReader<MouseWheel>,
    mut q: Query<(
        &mut UiOverscroll,
        &bevy::ui::RelativeCursorPosition,
        &ScrollPosition,
        &ComputedNode,
        &mut UiTransform,
    )>,
) {
    let now = std::time::Instant::now();
    let dy: f32 = wheel
        .read()
        .map(|w| match w.unit {
            MouseScrollUnit::Line => (w.y * 8.0).clamp(-24.0, 24.0),
            MouseScrollUnit::Pixel => w.y,
        })
        .sum();
    for (mut o, rel, pos, node, mut tf) in &mut q {
        if dy != 0.0 && rel.cursor_over() {
            let max =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
            o.0.on_wheel(dy, pos.y <= 0.5, pos.y >= max - 0.5, now);
        }
        if o.0.active() {
            o.0.settle(now, time.delta_secs());
        }
        let want = Val2::px(0.0, o.0.offset());
        if tf.translation != want {
            tf.translation = want;
        }
    }
}

// ------------------------------------------------------------------ lazy

/// Whether a block at `top..top + height` (window coordinates) is within `margin` of a window
/// `viewport` tall.
pub fn near(top: f32, height: f32, viewport: f32, margin: f32) -> bool {
    top + height > -margin && top < viewport + margin
}

/// Hides this node (keeping its layout) while it is far outside the window, so a long page
/// only draws what is near the screen.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct UiLazy {
    /// How far outside the window it still counts as near, px.
    pub margin: f32,
}

/// A culled node with a margin of 500 px.
pub fn lazy() -> UiLazy {
    UiLazy { margin: 500.0 }
}

pub(crate) fn lazy_cull(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut q: Query<(&UiLazy, &ComputedNode, &UiGlobalTransform, &mut Visibility)>,
) {
    let Ok(w) = windows.single() else { return };
    for (l, n, g, mut v) in &mut q {
        let inv = n.inverse_scale_factor();
        let h = n.size().y * inv;
        let top = g.translation.y * inv - h / 2.0;
        let want = if near(top, h, w.height(), l.margin) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *v != want {
            *v = want;
        }
    }
}

/// Registers the systems above.
pub(crate) fn add(app: &mut App) {
    app.add_message::<Follow>()
        .add_message::<MouseWheel>()
        .add_systems(
            Update,
            (
                track_follow,
                entry_start,
                entry_step,
                overscroll_wheel,
                lazy_cull,
            )
                .chain()
                .after(crate::kit::press_input),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follow_prefers_the_hovered_item() {
        assert_eq!(follow_target(Some(2), Some(0)), Some(2));
        assert_eq!(follow_target(None, Some(0)), Some(0));
        assert_eq!(follow_target(None, None), None);
    }

    #[test]
    fn rel_span_is_relative_to_the_track() {
        // track centre 100, width 200 (left edge 0); item centre 60 width 40 (left edge 40)
        assert_eq!(rel_span(60.0, 40.0, 100.0, 200.0, 1.0), (40.0, 40.0));
        assert_eq!(rel_span(60.0, 40.0, 100.0, 200.0, 0.5), (20.0, 20.0));
    }

    #[test]
    fn near_covers_the_window_and_its_margin() {
        assert!(near(0.0, 100.0, 800.0, 0.0));
        assert!(!near(900.0, 100.0, 800.0, 0.0));
        assert!(near(900.0, 100.0, 800.0, 200.0));
        assert!(!near(-700.0, 100.0, 800.0, 500.0));
        assert!(near(-550.0, 100.0, 800.0, 500.0));
    }

    #[test]
    fn a_selection_track_reports_what_it_follows() {
        let mut app = crate::testing::component_app();
        let t = app
            .world_mut()
            .spawn(selection_track(
                &["A", "B", "C"],
                0,
                SelectionMarker::Underline,
            ))
            .id();
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(t).unwrap().iter().collect();
        assert_eq!(kids.len(), 4, "marker and three items");
        *app.world_mut().get_mut::<ControlPointer>(kids[3]).unwrap() = ControlPointer::Hovered;
        app.update();
        let msgs = app.world().resource::<Messages<Follow>>();
        let got: Vec<_> = msgs.get_cursor().read(msgs).copied().collect();
        assert_eq!(
            got,
            vec![Follow {
                entity: t,
                index: Some(2)
            }]
        );
        // pressing selects it
        *app.world_mut().get_mut::<ControlPointer>(kids[3]).unwrap() = ControlPointer::Pressed;
        app.update();
        assert_eq!(
            app.world().get::<UiSelectGroup>(t).unwrap().selected,
            Some(2)
        );
    }

    #[test]
    fn an_entry_scope_animates_its_children() {
        use dodai_core::motion::EntryKind;
        let mut app = crate::testing::component_app();
        let s = app.world_mut().spawn(entry_scope(EntryKind::Rise)).id();
        let c = app.world_mut().spawn(Node::default()).id();
        app.world_mut().entity_mut(s).add_children(&[c]);
        app.update();
        std::thread::sleep(Duration::from_millis(20));
        app.update();
        app.update();
        let tf = app.world().get::<UiTransform>(c).unwrap();
        assert!(tf.translation.y != Val::ZERO, "{:?}", tf.translation);
        // settles
        for _ in 0..40 {
            std::thread::sleep(Duration::from_millis(10));
            app.update();
        }
        assert_eq!(
            app.world().get::<UiTransform>(c).unwrap().translation,
            Val2::px(0.0, 0.0)
        );
    }

    #[test]
    fn overscroll_stretches_past_the_end_and_comes_back() {
        let mut o = overscroll(OverscrollFeel::SNAPPY);
        let t = std::time::Instant::now();
        assert!(o.0.on_wheel(30.0, true, false, t));
        assert!(o.0.offset() > 0.0);
        let mut now = t + Duration::from_millis(100);
        for _ in 0..200 {
            o.0.settle(now, 1.0 / 60.0);
            now += Duration::from_millis(16);
        }
        assert_eq!(o.0.offset(), 0.0);
    }

    #[test]
    fn layers_are_pickable_through_and_fill_their_parent() {
        let mut app = crate::testing::component_app();
        let f = app.world_mut().spawn(frost_layer()).id();
        let g = app
            .world_mut()
            .spawn(glow_layer(Color::WHITE, 2.0, 12.0))
            .id();
        let p = app
            .world_mut()
            .spawn(frosted_panel(1.0, 2.0, 30.0, 40.0))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Node>(f).unwrap().width,
            Val::Percent(100.0)
        );
        assert!(app.world().get::<BoxShadow>(g).is_some());
        assert_eq!(app.world().get::<Node>(p).unwrap().left, Val::Px(1.0));
        assert!(app.world().get::<Pickable>(f).is_some());
    }

    #[test]
    fn interact_is_the_pointer_state() {
        let i: Interact = ControlPointer::Hovered;
        assert_eq!(i, ControlPointer::Hovered);
    }
}
