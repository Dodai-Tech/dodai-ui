//! A rotary knob: drag up or right to increase, the wheel turns it too. The arc is a ring of
//! short rotated segments (lit up to the value), the body a round node, the indicator a thin
//! rotated bar. Geometry, drag maths and colours come from `dodai-ui-core`.

use bevy::ecs::spawn::SpawnWith;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use dodai_ui_core::{knob_angle, knob_value, KnobGeometry, KNOB_SWEEP_DEG};

use crate::theme::{color, DodaiTheme};
use crate::widgets::{ControlEvent, ControlKind, ControlPointer};

/// Segments the arc is made of.
pub const KNOB_SEGMENTS: usize = 54;

/// Pixels of wheel travel one notch counts for.
const WHEEL_NOTCH: f32 = 8.0;

/// A rotary knob. Build it with [`knob`].
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[require(ControlPointer)]
pub struct UiKnob {
    /// Current value.
    pub value: f32,
    /// Minimum.
    pub min: f32,
    /// Maximum.
    pub max: f32,
    /// Snap to multiples of this from `min`.
    pub step: Option<f32>,
    /// Diameter in px.
    pub size: f32,
    /// Dimmed and not draggable.
    pub disabled: bool,
}

impl UiKnob {
    /// Value as a fraction of the range, `0..=1`.
    pub fn fraction(&self) -> f32 {
        if self.max > self.min {
            ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Where a drag started: the pointer and the value then.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct KnobDrag(pub Option<(Vec2, f32)>);

#[derive(Component)]
pub(crate) struct KnobSegment(usize);
#[derive(Component)]
pub(crate) struct KnobBody;
#[derive(Component)]
pub(crate) struct KnobTick;

/// Top-left corner, size and rotation (radians) of the node that draws arc segment `i` of `n`
/// of a knob `size` px across. The node is `len` long along the arc and `ring` thick.
pub fn segment_box(size: f32, i: usize, n: usize) -> (Vec2, Vec2, f32) {
    let g = KnobGeometry::new(size);
    let c = size / 2.0;
    let mid = (i as f32 + 0.5) / n as f32;
    let th = knob_angle(mid);
    let len = g.r_ring * (KNOB_SWEEP_DEG / n as f32).to_radians() * 1.25;
    let centre = Vec2::new(c + g.r_ring * th.cos(), c + g.r_ring * th.sin());
    let dims = Vec2::new(len, g.ring);
    (centre - dims / 2.0, dims, th + std::f32::consts::FRAC_PI_2)
}

/// Whether segment `i` of `n` is inside the lit part for fraction `frac`.
pub fn segment_lit(i: usize, n: usize, frac: f32) -> bool {
    (i as f32 + 0.5) / n as f32 <= frac
}

/// A knob `size` px across over `min..=max`.
pub fn knob(value: f32, min: f32, max: f32, size: f32) -> impl Bundle {
    let g = KnobGeometry::new(size);
    let c = size / 2.0;
    (
        UiKnob {
            value,
            min,
            max,
            step: None,
            size,
            disabled: false,
        },
        KnobDrag::default(),
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            flex_shrink: 0.0,
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            for i in 0..KNOB_SEGMENTS {
                let (at, dims, rot) = segment_box(size, i, KNOB_SEGMENTS);
                parent.spawn((
                    KnobSegment(i),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(at.x),
                        top: Val::Px(at.y),
                        width: Val::Px(dims.x),
                        height: Val::Px(dims.y),
                        ..default()
                    },
                    UiTransform::from_rotation(Rot2::radians(rot)),
                    BackgroundColor::default(),
                ));
            }
            parent.spawn((
                KnobBody,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(c - g.r_body),
                    top: Val::Px(c - g.r_body),
                    width: Val::Px(g.r_body * 2.0),
                    height: Val::Px(g.r_body * 2.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor::default(),
                BorderColor::default(),
            ));
            parent.spawn((
                KnobTick,
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                UiTransform::default(),
                BackgroundColor::default(),
            ));
        })),
    )
}

/// The tick's box and rotation for `frac`.
pub fn tick_box(size: f32, frac: f32) -> (Vec2, Vec2, f32) {
    let g = KnobGeometry::new(size);
    let c = size / 2.0;
    let th = knob_angle(frac);
    let (a, b) = (g.r_body * 0.35, g.r_body * 0.9);
    let mid = Vec2::new(c + th.cos() * (a + b) / 2.0, c + th.sin() * (a + b) / 2.0);
    let dims = Vec2::new(b - a, g.tick);
    (mid - dims / 2.0, dims, th)
}

pub(crate) fn knob_input(
    mut knobs: Query<(Entity, &ControlPointer, &mut UiKnob, &mut KnobDrag)>,
    window: Query<&Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<MouseWheel>,
    mut out: MessageWriter<ControlEvent>,
) {
    let cursor = window.single().ok().and_then(|w| w.cursor_position());
    let turns: Vec<f32> = wheel
        .read()
        .map(|e| match e.unit {
            MouseScrollUnit::Line | MouseScrollUnit::Pixel => e.y,
        })
        .collect();
    for (entity, pointer, mut k, mut drag) in &mut knobs {
        if k.disabled {
            continue;
        }
        let mut set = |k: &mut UiKnob, v: f32| {
            if (v - k.value).abs() > f32::EPSILON {
                k.value = v;
                out.write(ControlEvent {
                    entity,
                    kind: ControlKind::Slid(v),
                });
            }
        };
        if !mouse.pressed(MouseButton::Left) {
            if drag.0.is_some() {
                drag.0 = None;
            }
        } else if let Some(at) = cursor {
            match drag.0 {
                None if *pointer == ControlPointer::Pressed => drag.0 = Some((at, k.value)),
                Some((a, v0)) => {
                    let v = knob_value(v0, at.x - a.x, at.y - a.y, k.min, k.max, k.step);
                    set(&mut k, v);
                }
                None => {}
            }
        }
        if *pointer != ControlPointer::None {
            for dy in &turns {
                if *dy != 0.0 {
                    let v = knob_value(
                        k.value,
                        0.0,
                        -dy.signum() * WHEEL_NOTCH,
                        k.min,
                        k.max,
                        k.step,
                    );
                    set(&mut k, v);
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn style_knobs(
    theme: Res<DodaiTheme>,
    knobs: Query<(&UiKnob, &Children)>,
    mut segs: Query<(&KnobSegment, &mut BackgroundColor), (Without<KnobBody>, Without<KnobTick>)>,
    mut bodies: Query<
        (&mut BackgroundColor, &mut BorderColor),
        (With<KnobBody>, Without<KnobSegment>, Without<KnobTick>),
    >,
    mut ticks: Query<
        (&mut Node, &mut UiTransform, &mut BackgroundColor),
        (With<KnobTick>, Without<KnobSegment>, Without<KnobBody>),
    >,
) {
    for (k, children) in &knobs {
        let st = theme.knob(k.disabled);
        let frac = k.fraction();
        for child in children.iter() {
            if let Ok((seg, mut bg)) = segs.get_mut(child) {
                let want = color(if segment_lit(seg.0, KNOB_SEGMENTS, frac) {
                    st.fill
                } else {
                    st.track
                });
                if bg.0 != want {
                    bg.0 = want;
                }
            } else if let Ok((mut bg, mut bc)) = bodies.get_mut(child) {
                let (fill, edge) = (color(st.body), BorderColor::all(color(st.body_edge)));
                if bg.0 != fill {
                    bg.0 = fill;
                }
                if *bc != edge {
                    *bc = edge;
                }
            } else if let Ok((mut n, mut tf, mut bg)) = ticks.get_mut(child) {
                let (at, dims, rot) = tick_box(k.size, frac);
                let (l, t, w, h) = (
                    Val::Px(at.x),
                    Val::Px(at.y),
                    Val::Px(dims.x),
                    Val::Px(dims.y),
                );
                if n.left != l || n.top != t || n.width != w || n.height != h {
                    n.left = l;
                    n.top = t;
                    n.width = w;
                    n.height = h;
                }
                let r = Rot2::radians(rot);
                if tf.rotation != r {
                    tf.rotation = r;
                }
                let c = color(st.tick);
                if bg.0 != c {
                    bg.0 = c;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_lie_on_the_ring_and_run_clockwise() {
        let size = 72.0;
        let g = KnobGeometry::new(size);
        let mut last = -1.0_f32;
        for i in 0..KNOB_SEGMENTS {
            let (at, dims, rot) = segment_box(size, i, KNOB_SEGMENTS);
            let centre = at + dims / 2.0;
            let d = (centre - Vec2::splat(size / 2.0)).length();
            assert!((d - g.r_ring).abs() < 1e-3, "segment {i} is off the ring");
            assert!(rot > last, "segments must advance clockwise");
            last = rot;
        }
    }

    #[test]
    fn the_lit_part_grows_with_the_value() {
        let lit = |f: f32| {
            (0..KNOB_SEGMENTS)
                .filter(|i| segment_lit(*i, KNOB_SEGMENTS, f))
                .count()
        };
        assert_eq!(lit(0.0), 0);
        assert_eq!(lit(1.0), KNOB_SEGMENTS);
        assert!((lit(0.5) as i32 - KNOB_SEGMENTS as i32 / 2).abs() <= 1);
        assert!(lit(0.25) < lit(0.75));
    }

    #[test]
    fn the_tick_points_along_the_value_angle_and_stays_inside_the_body() {
        let g = KnobGeometry::new(72.0);
        for f in [0.0, 0.3, 0.5, 1.0] {
            let (at, dims, rot) = tick_box(72.0, f);
            assert!((rot - knob_angle(f)).abs() < 1e-6);
            let d = (at + dims / 2.0 - Vec2::splat(36.0)).length();
            assert!(d > 0.0 && d < g.r_body);
        }
    }

    #[test]
    fn the_fraction_follows_the_range() {
        let k = |v| UiKnob {
            value: v,
            min: 0.0,
            max: 4.0,
            step: None,
            size: 72.0,
            disabled: false,
        };
        assert_eq!(k(2.0).fraction(), 0.5);
        assert_eq!(k(9.0).fraction(), 1.0);
        assert_eq!(k(-1.0).fraction(), 0.0);
    }

    #[test]
    fn styling_lights_the_segments_up_to_the_value() {
        let mut app = App::new();
        app.insert_resource(DodaiTheme::new())
            .add_systems(Update, style_knobs);
        app.world_mut().spawn(knob(0.5, 0.0, 1.0, 72.0));
        app.update();
        let w = app.world_mut();
        let st = w.resource::<DodaiTheme>().knob(false);
        let (fill, track) = (color(st.fill), color(st.track));
        let mut q = w.query::<(&KnobSegment, &BackgroundColor)>();
        let lit = q.iter(w).filter(|(_, bg)| bg.0 == fill).count();
        let dim = q.iter(w).filter(|(_, bg)| bg.0 == track).count();
        assert!(fill != track);
        assert_eq!(lit + dim, KNOB_SEGMENTS);
        assert!((lit as i32 - KNOB_SEGMENTS as i32 / 2).abs() <= 1);
    }

    fn input_app() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.init_resource::<ButtonInput<MouseButton>>()
            .add_message::<MouseWheel>()
            .add_message::<ControlEvent>()
            .add_systems(Update, knob_input);
        let win = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        let k = app.world_mut().spawn(knob(0.5, 0.0, 1.0, 72.0)).id();
        (app, win, k)
    }

    fn events(app: &mut App) -> Vec<f32> {
        let mut r = app.world_mut().resource_mut::<Messages<ControlEvent>>();
        r.drain()
            .filter_map(|e| match e.kind {
                ControlKind::Slid(v) => Some(v),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn dragging_up_raises_the_value_and_letting_go_ends_the_drag() {
        let (mut app, win, k) = input_app();
        app.world_mut()
            .get_mut::<Window>(win)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(100.0, 100.0)));
        *app.world_mut().get_mut::<ControlPointer>(k).unwrap() = ControlPointer::Pressed;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert!(app.world().get::<KnobDrag>(k).unwrap().0.is_some());
        app.world_mut()
            .get_mut::<Window>(win)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(100.0, 60.0)));
        // The pointer left the knob: the drag goes on while the button is held.
        *app.world_mut().get_mut::<ControlPointer>(k).unwrap() = ControlPointer::None;
        app.update();
        let v = app.world().get::<UiKnob>(k).unwrap().value;
        assert!(v > 0.5, "{v}");
        assert_eq!(events(&mut app), vec![v]);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        assert!(app.world().get::<KnobDrag>(k).unwrap().0.is_none());
        assert_eq!(app.world().get::<UiKnob>(k).unwrap().value, v);
    }

    #[test]
    fn a_disabled_knob_ignores_the_pointer() {
        let (mut app, win, k) = input_app();
        app.world_mut().get_mut::<UiKnob>(k).unwrap().disabled = true;
        app.world_mut()
            .get_mut::<Window>(win)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(100.0, 100.0)));
        *app.world_mut().get_mut::<ControlPointer>(k).unwrap() = ControlPointer::Pressed;
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert!(app.world().get::<KnobDrag>(k).unwrap().0.is_none());
    }
}
