//! Controls → model actions, theme sync, readouts, scrolling.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use dodai_bevy::prelude::*;
use dodai_showcase_model::{Action, Value};

use crate::chrome::{MenuList, SIDEBAR_W};
use crate::render::{num_text, snap, Bound, Readout, TreeChevron, TreeRow};
use crate::{Dirty, Model, OpenMenu};
use bevy::ui::RelativeCursorPosition;

/// Which scrolling region a node is.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scroller {
    Sidebar,
    Page,
    Menu,
}

/// Counter range of the number stepper demo.
pub const STEPPER_RANGE: (i32, i32) = (0, 20);

pub fn hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<FocusedInput>,
    mut model: ResMut<Model>,
    mut dirty: ResMut<Dirty>,
) {
    if focus.0.is_some() || !keys.just_pressed(KeyCode::KeyT) {
        return;
    }
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    model.0.update(Action::CycleTheme(!shift));
    dirty.all();
}

#[allow(clippy::type_complexity)]
pub fn handle(
    mut ev: MessageReader<ControlEvent>,
    bound: Query<&Bound>,
    inputs: Query<&UiTextInput>,
    mut sliders: Query<&mut UiSlider>,
    mut model: ResMut<Model>,
    mut dirty: ResMut<Dirty>,
    mut open: ResMut<OpenMenu>,
) {
    for e in ev.read() {
        let Ok(b) = bound.get(e.entity) else { continue };
        match (b, e.kind) {
            (Bound::Menu(name), ControlKind::Pressed) => {
                open.0 = if open.0 == Some(*name) {
                    None
                } else {
                    Some(*name)
                };
                dirty.top = true;
            }
            (Bound::Act(a), ControlKind::Pressed | ControlKind::TabSelected) => {
                let filter = matches!(a, Action::Filter(_));
                if open.0.take().is_some() {
                    dirty.top = true;
                }
                model.0.update(a.clone());
                if filter {
                    dirty.nav = true;
                    dirty.content = true;
                } else {
                    dirty.all();
                }
            }
            (Bound::Toggle(k), ControlKind::Toggled(on)) => {
                model.0.update(Action::Set(k.clone(), Value::Bool(on)));
            }
            (Bound::Radio(g, i), ControlKind::Toggled(true)) => {
                model.0.update(Action::Set(g.clone(), Value::Index(*i)));
            }
            (Bound::Slide { key, step }, ControlKind::Slid(v)) => {
                if let Ok(mut s) = sliders.get_mut(e.entity) {
                    let v = snap(v, s.min, s.max, *step);
                    if s.value != v {
                        s.value = v;
                    }
                    model.0.update(Action::Set(key.clone(), Value::Num(v)));
                }
            }
            (Bound::Text(k), ControlKind::TextChanged) => {
                if let Ok(i) = inputs.get(e.entity) {
                    model
                        .0
                        .update(Action::Set(k.clone(), Value::Text(i.value.clone())));
                }
            }
            (Bound::Filter, ControlKind::TextChanged) => {
                if let Ok(i) = inputs.get(e.entity) {
                    model.0.update(Action::Filter(i.value.clone()));
                    dirty.nav = true;
                    dirty.content = true;
                }
            }
            (Bound::Step { key, by, default }, ControlKind::Pressed) => {
                let cur = model
                    .0
                    .values
                    .int(&dodai_showcase_model::Bind::new(key.clone(), *default));
                let next = (cur + by).clamp(STEPPER_RANGE.0, STEPPER_RANGE.1);
                model.0.update(Action::Set(key.clone(), Value::Int(next)));
            }
            _ => {}
        }
    }
}

/// A press on a tree row opens or closes the container that follows it. Leaves have no
/// container (the next sibling is another row), so they do nothing.
pub fn tree_toggle(
    mut ev: MessageReader<ControlEvent>,
    mut rows: Query<(&mut TreeRow, &ChildOf, &Children)>,
    marks: Query<(), With<TreeRow>>,
    siblings: Query<&Children>,
    mut nodes: Query<&mut Node>,
    mut chevrons: Query<&mut UiIcon, With<TreeChevron>>,
) {
    for e in ev.read() {
        if e.kind != ControlKind::Pressed {
            continue;
        }
        let Ok((mut row, parent, kids)) = rows.get_mut(e.entity) else {
            continue;
        };
        let Ok(all) = siblings.get(parent.parent()) else {
            continue;
        };
        let Some(at) = all.iter().position(|x| x == e.entity) else {
            continue;
        };
        let Some(next) = all.get(at + 1).copied() else {
            continue;
        };
        if marks.contains(next) {
            continue;
        }
        row.open = !row.open;
        if let Ok(mut n) = nodes.get_mut(next) {
            n.display = if row.open {
                Display::Flex
            } else {
                Display::None
            };
        }
        for k in kids.iter() {
            if let Ok(mut c) = chevrons.get_mut(k) {
                c.icon = if row.open {
                    dodai_icons::IconId::ChevronDown
                } else {
                    dodai_icons::IconId::ChevronRight
                };
            }
        }
    }
}

/// Push the model's theme into the Bevy resource when it changed.
pub fn sync_theme(
    model: Res<Model>,
    mut theme: ResMut<DodaiTheme>,
    mut seen: Local<Option<u64>>,
    mut dirty: ResMut<Dirty>,
) {
    let rev = model.0.revision();
    if *seen != Some(rev) {
        *seen = Some(rev);
        theme.set_theme(model.0.theme());
        dirty.all();
    }
}

/// Keep numbers that show a stored value in step with it.
pub fn readouts(model: Res<Model>, mut q: Query<(&Readout, &mut UiText)>) {
    if !model.is_changed() {
        return;
    }
    let v = &model.0.values;
    for (r, mut t) in &mut q {
        let shown = match r {
            Readout::Num { key, default, step } => num_text(
                v.num(&dodai_showcase_model::Bind::new(key.clone(), *default)),
                *step,
            ),
            Readout::Int { key, default } => v
                .int(&dodai_showcase_model::Bind::new(key.clone(), *default))
                .to_string(),
        };
        if t.raw != shown {
            t.raw = shown;
        }
    }
}

/// Wheel scrolling: the region under the cursor moves.
pub fn scroll(
    wheel: Res<AccumulatedMouseScroll>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&Scroller, &mut ScrollPosition, &ComputedNode)>,
    menus: Query<&RelativeCursorPosition, With<MenuList>>,
    mut first: Local<bool>,
) {
    let x = windows
        .single()
        .ok()
        .and_then(Window::cursor_position)
        .map_or(f32::MAX, |p| p.x);
    let target = if menus.iter().any(RelativeCursorPosition::cursor_over) {
        Scroller::Menu
    } else if x < SIDEBAR_W {
        Scroller::Sidebar
    } else {
        Scroller::Page
    };
    let dy = match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y * 40.0,
        MouseScrollUnit::Pixel => wheel.delta.y,
    };
    if !*first {
        *first = true;
        if let Some(px) = std::env::var("DODAI_SCROLL")
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
        {
            for (s, mut pos, _) in &mut q {
                if *s == Scroller::Page {
                    pos.y = px;
                }
            }
        }
    }
    if dy == 0.0 {
        return;
    }
    for (s, mut pos, node) in &mut q {
        if *s == target {
            pos.y = (pos.y - dy).max(0.0);
            let scale = node.inverse_scale_factor();
            let max = (node.content_size().y - node.size().y).max(0.0) * scale;
            pos.y = pos.y.min(max);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_showcase_model::{Bind, Showcase};

    fn app() -> App {
        let mut app = App::new();
        app.add_message::<ControlEvent>()
            .insert_resource(Model(Showcase::new()))
            .insert_resource(Dirty::default())
            .insert_resource(OpenMenu::default())
            .add_systems(Update, handle);
        app
    }

    fn send(app: &mut App, entity: Entity, kind: ControlKind) {
        app.world_mut().write_message(ControlEvent { entity, kind });
        app.update();
    }

    fn model(app: &App) -> &Showcase {
        &app.world().resource::<Model>().0
    }

    #[test]
    fn pressing_a_category_selects_it_and_rebuilds_everything() {
        let mut a = app();
        let e = a
            .world_mut()
            .spawn(Bound::Act(Action::Category(Some(2))))
            .id();
        send(&mut a, e, ControlKind::Pressed);
        assert_eq!(model(&a).category(), Some(2));
        let d = a.world().resource::<Dirty>();
        assert!(d.nav && d.content && d.top);
    }

    #[test]
    fn typing_in_the_filter_rebuilds_the_list_but_not_the_field_or_top_bar() {
        let mut a = app();
        let e = a
            .world_mut()
            .spawn((
                Bound::Filter,
                UiTextInput {
                    value: "tabs".into(),
                    ..default()
                },
            ))
            .id();
        send(&mut a, e, ControlKind::TextChanged);
        assert_eq!(model(&a).filter(), "tabs");
        let d = a.world().resource::<Dirty>();
        assert!(d.nav && d.content && !d.top);
    }

    #[test]
    fn a_slider_snaps_to_its_step_and_stores_the_snapped_value() {
        let mut a = app();
        let e = a
            .world_mut()
            .spawn((
                Bound::Slide {
                    key: "s".into(),
                    step: Some(1.0),
                },
                UiSlider {
                    value: 0.0,
                    min: 0.0,
                    max: 10.0,
                },
            ))
            .id();
        send(&mut a, e, ControlKind::Slid(6.6));
        assert_eq!(model(&a).values.num(&Bind::new("s", 0.0)), 7.0);
        assert_eq!(a.world().get::<UiSlider>(e).unwrap().value, 7.0);
        // Sliding does not rebuild the page (a rebuild would drop the drag).
        assert!(!a.world().resource::<Dirty>().content);
    }

    #[test]
    fn toggles_radios_and_text_store_their_values() {
        let mut a = app();
        let t = a.world_mut().spawn(Bound::Toggle("sw".into())).id();
        send(&mut a, t, ControlKind::Toggled(true));
        assert!(model(&a).values.flag(&Bind::new("sw", false)));
        let r = a.world_mut().spawn(Bound::Radio("g".into(), 2)).id();
        send(&mut a, r, ControlKind::Toggled(true));
        assert_eq!(model(&a).values.index_of("g", 0), 2);
        // A radio switching off stores nothing.
        let r0 = a.world_mut().spawn(Bound::Radio("g".into(), 0)).id();
        send(&mut a, r0, ControlKind::Toggled(false));
        assert_eq!(model(&a).values.index_of("g", 9), 2);
        let i = a
            .world_mut()
            .spawn((
                Bound::Text("t".into()),
                UiTextInput {
                    value: "hi".into(),
                    ..default()
                },
            ))
            .id();
        send(&mut a, i, ControlKind::TextChanged);
        assert_eq!(model(&a).values.text(&Bind::new("t", String::new())), "hi");
    }

    #[test]
    fn the_stepper_stays_inside_its_range() {
        let mut a = app();
        let down = a
            .world_mut()
            .spawn(Bound::Step {
                key: "n".into(),
                by: -1,
                default: 0,
            })
            .id();
        send(&mut a, down, ControlKind::Pressed);
        assert_eq!(model(&a).values.int(&Bind::new("n", 0)), STEPPER_RANGE.0);
        let up = a
            .world_mut()
            .spawn(Bound::Step {
                key: "n".into(),
                by: 1,
                default: 0,
            })
            .id();
        for _ in 0..(STEPPER_RANGE.1 + 5) {
            send(&mut a, up, ControlKind::Pressed);
        }
        assert_eq!(model(&a).values.int(&Bind::new("n", 0)), STEPPER_RANGE.1);
    }

    #[test]
    fn a_menu_opens_closes_and_closes_when_an_item_is_picked() {
        let mut a = app();
        let m = a.world_mut().spawn(Bound::Menu("theme")).id();
        send(&mut a, m, ControlKind::Pressed);
        assert_eq!(a.world().resource::<OpenMenu>().0, Some("theme"));
        send(&mut a, m, ControlKind::Pressed);
        assert_eq!(a.world().resource::<OpenMenu>().0, None);
        send(&mut a, m, ControlKind::Pressed);
        let item = a.world_mut().spawn(Bound::Act(Action::Variant(1))).id();
        send(&mut a, item, ControlKind::Pressed);
        assert_eq!(a.world().resource::<OpenMenu>().0, None);
        assert_eq!(model(&a).variant_index(), 1);
    }

    #[test]
    fn unbound_and_mismatched_events_change_nothing() {
        let mut a = app();
        let plain = a.world_mut().spawn_empty().id();
        send(&mut a, plain, ControlKind::Pressed);
        let tog = a.world_mut().spawn(Bound::Toggle("x".into())).id();
        send(&mut a, tog, ControlKind::Pressed);
        assert!(!model(&a).values.flag(&Bind::new("x", false)));
        assert!(!a.world().resource::<Dirty>().content);
    }
}
