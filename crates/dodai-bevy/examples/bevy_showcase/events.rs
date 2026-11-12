//! Controls → model actions, theme sync, readouts, scrolling.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use dodai_bevy::prelude::*;
use dodai_showcase_model::{Action, Value};

use crate::chrome::{MenuList, RAIL_W, SIDEBAR_W};
use crate::render::{num_text, snap, Bound, Readout, TreeChevron, TreeRow};
use crate::{Dirty, Model, OpenMenu};
use bevy::ui::RelativeCursorPosition;

/// Which scrolling region a node is.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scroller {
    Sidebar,
    Page,
    Menu,
    /// A fixed-height scroll area inside the page.
    Area,
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

/// The sliders and knobs a [`Bound::Slide`] can write to.
#[derive(bevy::ecs::system::SystemParam)]
pub struct Dials<'w, 's> {
    sliders: Query<'w, 's, &'static mut UiSlider>,
    knobs: Query<'w, 's, &'static mut UiKnob>,
}

impl Dials<'_, '_> {
    /// Moves the slider or knob `e` to `v` (snapped to `step`); the value it ended up at.
    fn set(&mut self, e: Entity, v: f32, step: Option<f32>) -> Option<f32> {
        if let Ok(mut s) = self.sliders.get_mut(e) {
            let v = snap(v, s.min, s.max, step);
            if s.value != v {
                s.value = v;
            }
            Some(v)
        } else if let Ok(mut k) = self.knobs.get_mut(e) {
            let v = snap(v, k.min, k.max, step);
            if k.value != v {
                k.value = v;
            }
            Some(v)
        } else {
            None
        }
    }
}

/// Gizmo clicks and drags move the camera stored under its keys.
pub fn gizmo(
    mut ev: MessageReader<GizmoEvent>,
    bound: Query<&Bound>,
    mut model: ResMut<Model>,
    mut dirty: ResMut<Dirty>,
) {
    for e in ev.read() {
        let Ok(Bound::Gizmo { yaw, pitch }) = bound.get(e.entity) else {
            continue;
        };
        let home = crate::render::GIZMO_HOME;
        let vals = &model.0.values;
        let now = (
            vals.num(&dodai_showcase_model::Bind::new(yaw, home.0)),
            vals.num(&dodai_showcase_model::Bind::new(pitch, home.1)),
        );
        let (y, p) = match e.kind {
            GizmoKind::Select(a) => {
                let (y, p) = a.view_angles();
                (y.unwrap_or(now.0), p)
            }
            GizmoKind::Home => home,
            GizmoKind::Drag(d) => dodai_core::orientation::orbit(now.0, now.1, d.x, d.y),
        };
        model.0.update(Action::Set(yaw.clone(), Value::Num(y)));
        model.0.update(Action::Set(pitch.clone(), Value::Num(p)));
        dirty.content = true;
    }
}

#[allow(clippy::type_complexity)]
pub fn handle(
    mut ev: MessageReader<ControlEvent>,
    bound: Query<&Bound>,
    inputs: Query<&UiTextInput>,
    mut dials: Dials,
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
                // A select inside the page redraws the page too.
                dirty.content |= name.starts_with("sel:");
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
                let v = dials.set(e.entity, v, *step);
                if let Some(v) = v {
                    model.0.update(Action::Set(key.clone(), Value::Num(v)));
                    dirty.settle = crate::SETTLE_FRAMES;
                }
            }
            (
                Bound::Range {
                    lo,
                    hi,
                    step,
                    span: (min, max),
                },
                ControlKind::Ranged(a, b),
            ) => {
                let snap = |v| crate::render::snap(v, *min, *max, *step);
                model.0.update(Action::Set(lo.clone(), Value::Num(snap(a))));
                model.0.update(Action::Set(hi.clone(), Value::Num(snap(b))));
                dirty.settle = crate::SETTLE_FRAMES;
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

/// Hand the Surface effects page's mix and the frost demo's picture to the glass.
pub fn sync_glass(
    model: Res<Model>,
    mut glass: ResMut<DodaiGlass>,
    mut seen: Local<Option<(dodai_core::SurfaceFx, Option<&'static str>)>>,
) {
    let theme = model.0.theme();
    let now = (
        model.0.surface_fx(),
        model.0.wallpaper().then(|| theme.id()),
    );
    if *seen == Some(now) {
        return;
    }
    let picture_changed = seen.map(|s| s.1) != Some(now.1);
    *seen = Some(now);
    glass.set_mix(now.0);
    if picture_changed {
        glass.set_picture(now.1.map(|_| {
            let p = dodai_showcase_model::media::wallpaper(&theme);
            (p.rgba.to_vec(), p.width as usize, p.height as usize)
        }));
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

/// The region the wheel moves: an open menu, then a scroll area under the cursor, then the
/// sidebar or the page by the cursor's x.
pub fn scroll_target(menu_over: bool, area_over: bool, x: f32, side_w: f32) -> Scroller {
    if menu_over {
        Scroller::Menu
    } else if area_over {
        Scroller::Area
    } else if x < side_w {
        Scroller::Sidebar
    } else {
        Scroller::Page
    }
}

/// How far (logical px) below the page's top a section still counts as reached.
const SPY_SLACK: f32 = 60.0;

/// What `section_nav` keeps between frames: the section it is scrolling to, the frames to let
/// the layout settle after that, and the frames seen.
#[derive(Default)]
pub struct NavState {
    pending: Option<&'static str>,
    settle: u32,
    frames: u32,
}

/// A section panel of the page, found by its title to scroll to it and to follow the scroll.
#[derive(Component, Debug, Clone, Copy)]
pub struct SectionAnchor(pub &'static str);

/// Sidebar picks scroll the page to that section; scrolling the page moves the sidebar's
/// highlight to the section at the top (the scroll-spy).
pub fn section_nav(
    mut model: ResMut<Model>,
    mut dirty: ResMut<Dirty>,
    anchors: Query<(&SectionAnchor, &ComputedNode, &UiGlobalTransform)>,
    mut pages: Query<(
        &Scroller,
        &mut ScrollPosition,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    wheel: Res<AccumulatedMouseScroll>,
    mut nav: Local<NavState>,
) {
    let NavState {
        pending,
        settle,
        frames,
    } = &mut *nav;
    *frames += 1;
    if *frames == 20 {
        // `DODAI_SECTION=<title>`: pick that section, as a click in the sidebar would.
        let wanted = std::env::var("DODAI_SECTION").ok().and_then(|t| {
            model
                .0
                .visible()
                .iter()
                .find(|s| s.title == t)
                .map(|s| s.title)
        });
        if let Some(t) = wanted {
            model.0.update(Action::Section(t));
            dirty.nav = true;
            dirty.content = true;
        }
    }
    if wheel.delta.y != 0.0 {
        // The user scrolls: a picked section stops holding the highlight.
        model.bypass_change_detection().0.update(Action::Unpin);
    }
    if let Some(t) = model.bypass_change_detection().0.take_jump() {
        *pending = Some(t);
    }
    let Some((_, mut pos, page, page_at)) = pages.iter_mut().find(|p| *p.0 == Scroller::Page)
    else {
        return;
    };
    let top = |n: &ComputedNode, at: &UiGlobalTransform| at.translation.y - n.size().y / 2.0;
    let page_top = top(page, page_at);
    let scale = page.inverse_scale_factor();
    if let Some(t) = *pending {
        // The page may still be rebuilding: wait until the panel has been laid out.
        if let Some((_, n, at)) = anchors
            .iter()
            .find(|(a, n, _)| a.0 == t && n.size().y > 0.0)
        {
            let max = (page.content_size().y - page.size().y).max(0.0) * scale;
            pos.y = (pos.y + (top(n, at) - page_top) * scale).clamp(0.0, max);
            *pending = None;
            *settle = 20;
        }
        return;
    }
    if *settle > 0 {
        *settle -= 1;
        return;
    }
    let mut tops: Vec<(&'static str, f32)> = anchors
        .iter()
        .map(|(a, n, at)| (a.0, (top(n, at) - page_top) * scale))
        .collect();
    tops.sort_by(|a, b| a.1.total_cmp(&b.1));
    let max = (page.content_size().y - page.size().y).max(0.0) * scale;
    let at_end = max > 1.0 && pos.y >= max - 1.0;
    let before = model.0.active_section();
    model
        .bypass_change_detection()
        .0
        .follow_scroll(&tops, 0.0, at_end, SPY_SLACK);
    if model.0.active_section() != before {
        dirty.nav = true;
    }
}

/// Wheel scrolling: the region under the cursor moves.
pub fn scroll(
    wheel: Res<AccumulatedMouseScroll>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(
        &Scroller,
        &mut ScrollPosition,
        &ComputedNode,
        Option<&RelativeCursorPosition>,
    )>,
    menus: Query<&RelativeCursorPosition, With<MenuList>>,
    model: Res<Model>,
    mut first: Local<bool>,
) {
    let side_w = if model.0.rail() { RAIL_W } else { SIDEBAR_W };
    let x = windows
        .single()
        .ok()
        .and_then(Window::cursor_position)
        .map_or(f32::MAX, |p| p.x);
    let dy = match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y * 40.0,
        MouseScrollUnit::Pixel => wheel.delta.y,
    };
    let range =
        |n: &ComputedNode| (n.content_size().y - n.size().y).max(0.0) * n.inverse_scale_factor();
    // A scroll area under the cursor takes the wheel until it can't move that way; then the
    // page goes on (as the gpui showcase's nested areas do).
    let over_area = q.iter().any(|(s, pos, n, r)| {
        *s == Scroller::Area
            && r.is_some_and(|r| r.cursor_over())
            && ((dy > 0.0 && pos.y > 0.0) || (dy < 0.0 && pos.y < range(n)) || dy == 0.0)
    });
    let target = scroll_target(
        menus.iter().any(RelativeCursorPosition::cursor_over),
        over_area,
        x,
        side_w,
    );
    if !*first {
        *first = true;
        if let Some(px) = std::env::var("DODAI_SCROLL")
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
        {
            for (s, mut pos, _, _) in &mut q {
                if *s == Scroller::Page {
                    pos.y = px;
                }
            }
        }
    }
    if dy == 0.0 {
        return;
    }
    for (s, mut pos, node, rel) in &mut q {
        // Of the scroll areas only the one under the cursor moves.
        let under = *s != Scroller::Area || rel.is_some_and(|r| r.cursor_over());
        if *s == target && under {
            pos.y = (pos.y - dy).max(0.0);
            let scale = node.inverse_scale_factor();
            let max = (node.content_size().y - node.size().y).max(0.0) * scale;
            pos.y = pos.y.min(max);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_live_bodies_in_the_window_are_rebuilt() {
        use super::on_screen;
        assert!(on_screen(100.0, 300.0, 900.0));
        assert!(on_screen(-50.0, 20.0, 900.0), "cut by the top edge");
        assert!(on_screen(880.0, 1000.0, 900.0), "cut by the bottom edge");
        assert!(!on_screen(-400.0, -10.0, 900.0), "scrolled out above");
        assert!(!on_screen(950.0, 1200.0, 900.0), "below the window");
        assert!(on_screen(0.0, 0.0, 900.0), "not laid out yet");
    }

    #[test]
    fn turning_a_knob_stores_its_value_and_schedules_a_rebuild() {
        let mut a = app();
        let e = a
            .world_mut()
            .spawn((
                Bound::Slide {
                    key: "k".into(),
                    step: None,
                },
                UiKnob {
                    value: 0.0,
                    min: 0.0,
                    max: 1.0,
                    step: None,
                    size: 72.0,
                    disabled: false,
                },
            ))
            .id();
        send(&mut a, e, ControlKind::Slid(0.4));
        assert_eq!(model(&a).values.num(&Bind::new("k", 0.0)), 0.4);
        assert_eq!(a.world().get::<UiKnob>(e).unwrap().value, 0.4);
        assert_eq!(a.world().resource::<Dirty>().settle, crate::SETTLE_FRAMES);
    }

    #[test]
    fn the_wheel_goes_to_the_menu_then_a_scroll_area_then_sidebar_or_page() {
        use super::{scroll_target, Scroller};
        use crate::chrome::SIDEBAR_W;
        assert_eq!(scroll_target(true, true, 0.0, SIDEBAR_W), Scroller::Menu);
        assert_eq!(scroll_target(false, true, 0.0, SIDEBAR_W), Scroller::Area);
        assert_eq!(
            scroll_target(false, false, SIDEBAR_W - 1.0, SIDEBAR_W),
            Scroller::Sidebar
        );
        assert_eq!(
            scroll_target(false, false, SIDEBAR_W + 1.0, SIDEBAR_W),
            Scroller::Page
        );
    }

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
                    vertical: false,
                    lo: None,
                },
            ))
            .id();
        send(&mut a, e, ControlKind::Slid(6.6));
        assert_eq!(model(&a).values.num(&Bind::new("s", 0.0)), 7.0);
        assert_eq!(a.world().get::<UiSlider>(e).unwrap().value, 7.0);
        // Sliding does not rebuild the page at once (a rebuild would drop the drag); it is
        // scheduled for when the slider rests.
        let d = a.world().resource::<Dirty>();
        assert!(!d.content && d.settle == crate::SETTLE_FRAMES);
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

/// A section panel the Surface effects page's mix paints (texture, scanlines, bevel...), redrawn
/// when its size or the mix changes.
#[derive(Component, Default)]
pub struct SectionFx {
    drawn: Option<(Vec2, dodai_core::SurfaceFx, Entity)>,
}

/// Paint the app-wide surface mix over every section panel, as the other showcases do.
pub fn section_fx(
    mut commands: Commands,
    model: Res<Model>,
    theme: Res<DodaiTheme>,
    mut q: Query<(Entity, &ComputedNode, &mut SectionFx)>,
) {
    let fx = model.0.surface_fx();
    for (e, node, mut s) in &mut q {
        let size = (node.size() * node.inverse_scale_factor()).round();
        if size.x < 1.0 || size.y < 1.0 {
            continue;
        }
        if s.drawn.is_some_and(|(sz, f, _)| sz == size && f == fx) {
            continue;
        }
        if let Some((_, _, old)) = s.drawn.take() {
            commands.entity(old).try_despawn();
        }
        let mut layer = Entity::PLACEHOLDER;
        commands.entity(e).with_children(|p| {
            layer = p
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Px(size.x),
                        height: Val::Px(size.y),
                        ..default()
                    },
                    bevy::picking::Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ))
                .with_children(|l| {
                    dodai_bevy::surface_fx::spawn_surface_fx(l, theme.theme(), fx, size)
                })
                .id();
        });
        s.drawn = Some((size, fx, layer));
    }
}

/// The body of a section that shows the time (a clock, a running timer).
#[derive(Component)]
pub struct LiveBody(pub &'static str);

/// Whether a node spanning `top..bottom` (physical px) shows in a window `height` px high. A
/// node not laid out yet (`top == bottom`) counts as shown, so it gets its first build.
pub fn on_screen(top: f32, bottom: f32, height: f32) -> bool {
    top == bottom || (bottom >= 0.0 && top <= height)
}

/// Rebuild the live section bodies as often as the model asks
/// ([`Showcase::redraw_every`](dodai_showcase_model::Showcase::redraw_every)); bodies scrolled
/// out of the window wait until they come back (the Animation page asks for every frame).
pub fn live_refresh(
    mut commands: Commands,
    model: Res<Model>,
    (theme, open): (Res<DodaiTheme>, Res<OpenMenu>),
    time: Res<Time>,
    mut last: Local<f32>,
    q: Query<(Entity, &LiveBody, &ComputedNode, &UiGlobalTransform)>,
    windows: Query<&Window>,
) {
    let Some(every) = model.0.redraw_every() else {
        return;
    };
    let now = time.elapsed_secs();
    if now - *last < every.as_secs_f32() {
        return;
    }
    *last = now;
    let m = &model.0;
    let t = theme.theme().clone();
    let ctx = dodai_showcase_model::Ctx {
        theme: &t,
        values: &m.values,
    };
    let cx = crate::render::Cx {
        theme: &t,
        model: m,
        open: open.0,
    };
    let height = windows
        .iter()
        .next()
        .map_or(f32::MAX, |w| w.physical_height() as f32);
    for (e, body, n, at) in &q {
        let half = n.size().y / 2.0;
        if !on_screen(at.translation.y - half, at.translation.y + half, height) {
            continue;
        }
        let Some(s) = dodai_showcase_model::sections()
            .iter()
            .find(|s| s.title == body.0)
        else {
            continue;
        };
        let node = (s.build)(&ctx);
        commands
            .entity(e)
            .despawn_related::<Children>()
            .with_children(|c| crate::render::spawn(c, &node, &cx));
    }
}
