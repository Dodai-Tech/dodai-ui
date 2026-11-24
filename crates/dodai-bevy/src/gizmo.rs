//! Orientation axis gizmo: filled balls for +X, +Y and +Z, hollow rings for -X and -Y, a home
//! button on the circle line, drag anywhere inside to orbit.
//!
//! Geometry (placement, hit test, click-or-drag) is `dodai_core::orientation`;
//! this file spawns the nodes, colours them from the theme and turns picking into
//! [`GizmoEvent`]s. It is controlled: write the camera [`Basis`] into [`UiAxisGizmo::basis`]
//! each frame and move your camera when an event arrives.
//!
//! ```no_run
//! use bevy::prelude::*;
//! use dodai_bevy::prelude::*;
//!
//! fn setup(mut c: Commands) {
//!     c.spawn((
//!         axis_gizmo(130.0),
//!         Node { position_type: PositionType::Absolute, right: px(12), top: px(12), ..default() },
//!     ));
//! }
//! ```

use bevy::picking::events::{PointerClick, PointerDrag, PointerPress};
use bevy::picking::pointer::PointerButton;
use bevy::picking::Pickable;
use bevy::prelude::*;
use dodai_core::orientation::{place, DragTracker, GizmoLayout, Placed};
use dodai_core::{Color as Rgba, Palette};

pub use dodai_core::orientation::{orbit, AxisId, AxisTone, Basis};

use crate::theme::{color, DodaiTheme};
use crate::widgets::ControlPointer;

/// The gizmo root. Set [`basis`](Self::basis) from your camera every frame.
#[derive(Component, Debug, Clone, PartialEq)]
#[require(Node, GizmoDrag)]
pub struct UiAxisGizmo {
    /// The camera's axes in the Z-up world.
    pub basis: Basis,
    /// Sizes, derived from the side length.
    pub layout: GizmoLayout,
    /// Opacity of the whole gizmo, `0.0..=1.0`: backdrop, balls, stems and buttons.
    pub opacity: f32,
    /// Opacity of the round backdrop on its own, `0.0..=1.0` (multiplied by [`opacity`](Self::opacity)).
    pub backdrop: f32,
}

/// Default opacity of the round backdrop.
pub const DEFAULT_BACKDROP: f32 = 0.72;

impl UiAxisGizmo {
    /// Sets the opacity of the whole gizmo (clamped to `0.0..=1.0`).
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    /// Sets the opacity of the backdrop only (clamped to `0.0..=1.0`).
    pub fn set_backdrop(&mut self, backdrop: f32) {
        self.backdrop = backdrop.clamp(0.0, 1.0);
    }

    /// Whole-gizmo opacity builder.
    #[must_use]
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.set_opacity(opacity);
        self
    }

    /// Backdrop opacity builder.
    #[must_use]
    pub fn with_backdrop(mut self, backdrop: f32) -> Self {
        self.set_backdrop(backdrop);
        self
    }

    /// Points the gizmo like an orbit camera (yaw 0 looks north from the south).
    pub fn set_orbit(&mut self, yaw: f32, pitch: f32) {
        self.basis = Basis::from_orbit(yaw, pitch);
    }
}

/// Press-vs-drag bookkeeping of one gizmo.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct GizmoDrag(DragTracker);

/// A piece of a gizmo.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) enum GizmoPart {
    Stem(AxisId),
    Ball(AxisId),
    Letter(AxisId),
    Home,
    HomeInk,
    HomeCut,
    /// The plus (`true`) or minus (`false`) button.
    Zoom(bool),
    /// The bars of a plus or minus glyph.
    ZoomInk,
}

/// Something the gizmo reported.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct GizmoEvent {
    /// The gizmo root.
    pub entity: Entity,
    /// What happened.
    pub kind: GizmoKind,
}

/// Kind of [`GizmoEvent`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GizmoKind {
    /// An axis ball was clicked: show the view from that side ([`AxisId::view_angles`]).
    Select(AxisId),
    /// The home button was clicked.
    Home,
    /// The plus button was clicked: move the camera closer.
    ZoomIn,
    /// The minus button was clicked: move the camera away.
    ZoomOut,
    /// The pointer dragged inside the gizmo by this many pixels (feed it to [`orbit`]).
    Drag(Vec2),
}

fn abs(left: f32, top: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(left),
        top: Val::Px(top),
        width: Val::Px(w),
        height: Val::Px(h),
        ..default()
    }
}

/// Spawn an orientation gizmo `size` px wide. Add your own `Node` next to it for the position.
pub fn axis_gizmo(size: f32) -> impl Bundle {
    let layout = GizmoLayout::new(size);
    let axes = AxisId::DEFAULT;
    let stems = axes.into_iter().map(move |a| {
        (
            GizmoPart::Stem(a),
            Node {
                position_type: PositionType::Absolute,
                height: Val::Px(layout.stem),
                border_radius: BorderRadius::all(Val::Px(layout.stem / 2.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            UiTransform::default(),
            ZIndex(0),
            Pickable::IGNORE,
        )
    });
    let balls = axes.into_iter().map(move |a| {
        let d = if a.positive() {
            layout.ball
        } else {
            layout.ring
        };
        let mut node = abs(0.0, 0.0, d, d);
        node.align_items = AlignItems::Center;
        node.justify_content = JustifyContent::Center;
        node.border_radius = BorderRadius::all(Val::Px(d / 2.0));
        if !a.positive() {
            node.border = UiRect::all(Val::Px((d * 0.17).max(2.0)));
        }
        let letter = if a.positive() { a.letter() } else { "" };
        (
            GizmoPart::Ball(a),
            ControlPointer::default(),
            node,
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            ZIndex(100),
            children![(
                GizmoPart::Letter(a),
                Text::new(letter),
                TextFont {
                    font_size: bevy::text::FontSize::Px(d * 0.55),
                    ..default()
                },
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            )],
        )
    });
    let hc = layout.home_centre();
    let h = layout.home;
    let mut home_node = abs(hc[0] - h / 2.0, hc[1] - h / 2.0, h, h);
    home_node.border = UiRect::all(Val::Px(1.0));
    home_node.border_radius = BorderRadius::all(Val::Px(h / 2.0));
    let glyph = |part: GizmoPart, left: f32, top: f32, w: f32, hh: f32, turn: f32| {
        let mut n = abs(left * h, top * h, w * h, hh * h);
        if turn != 0.0 {
            n.border_radius = BorderRadius::ZERO;
        }
        (
            part,
            n,
            BackgroundColor(Color::NONE),
            UiTransform::from_rotation(Rot2::degrees(turn)),
            Pickable::IGNORE,
        )
    };
    let home = (
        GizmoPart::Home,
        ControlPointer::default(),
        home_node,
        BackgroundColor(Color::NONE),
        BorderColor::all(Color::NONE),
        ZIndex(1000),
        children![
            glyph(GizmoPart::HomeInk, 0.27, 0.17, 0.46, 0.46, 45.0),
            glyph(GizmoPart::HomeInk, 0.27, 0.45, 0.46, 0.34, 0.0),
            glyph(GizmoPart::HomeCut, 0.44, 0.55, 0.12, 0.24, 0.0),
        ],
    );
    let zoom_button = move |zoom_in: bool| {
        let c = if zoom_in {
            layout.zoom_in_centre()
        } else {
            layout.zoom_out_centre()
        };
        let d = layout.zoom;
        let mut node = abs(c[0] - d / 2.0, c[1] - d / 2.0, d, d);
        node.border = UiRect::all(Val::Px(1.0));
        node.border_radius = BorderRadius::all(Val::Px(d / 2.0));
        let bar = move |left: f32, top: f32, w: f32, hh: f32| {
            (
                GizmoPart::ZoomInk,
                abs(left * d, top * d, w * d, hh * d),
                BackgroundColor(Color::NONE),
                Pickable::IGNORE,
            )
        };
        (
            GizmoPart::Zoom(zoom_in),
            ControlPointer::default(),
            node,
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            ZIndex(1000),
            Children::spawn(SpawnWith(move |c: &mut ChildSpawner| {
                c.spawn(bar(0.28, 0.46, 0.44, 0.08));
                if zoom_in {
                    c.spawn(bar(0.46, 0.28, 0.08, 0.44));
                }
            })),
        )
    };
    (
        UiAxisGizmo {
            basis: Basis::from_orbit(0.6, 0.45),
            layout,
            opacity: 1.0,
            backdrop: DEFAULT_BACKDROP,
        },
        Children::spawn((
            SpawnIter(stems),
            SpawnIter(balls),
            Spawn(home),
            Spawn(zoom_button(false)),
            Spawn(zoom_button(true)),
        )),
    )
}

fn upd<T: PartialEq>(slot: &mut T, v: T) {
    if *slot != v {
        *slot = v;
    }
}

/// `c` with its alpha scaled by `opacity`.
fn fade(c: Rgba, opacity: f32) -> Rgba {
    c.with_alpha(f32::from(c.a) / 255.0 * opacity.clamp(0.0, 1.0))
}

fn lighten(c: Rgba, towards: Rgba, t: f32) -> Rgba {
    c.mix(towards, t)
}

/// Backdrop, size and ring of every gizmo root.
pub(crate) fn style_gizmo_roots(
    theme: Res<DodaiTheme>,
    mut q: Query<(
        &UiAxisGizmo,
        &mut Node,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    let p = &theme.palette;
    let bw = theme.shape.border_width.max(1.0);
    for (g, mut n, mut bg, mut bc) in &mut q {
        let s = g.layout.size;
        upd(&mut n.width, Val::Px(s));
        upd(&mut n.height, Val::Px(s));
        upd(&mut n.border, UiRect::all(Val::Px(bw)));
        upd(&mut n.border_radius, BorderRadius::all(Val::Px(s / 2.0)));
        upd(
            &mut bg.0,
            color(fade(p.surface.with_alpha(g.backdrop), g.opacity)),
        );
        upd(
            &mut *bc,
            BorderColor::all(color(fade(p.border_strong, g.opacity))),
        );
    }
}

fn tone(p: &Palette, t: AxisTone) -> Rgba {
    match t {
        AxisTone::X => p.danger,
        AxisTone::Y => p.success,
        AxisTone::Z => p.primary,
    }
}

fn find_root(
    mut e: Entity,
    parents: &Query<&ChildOf>,
    roots: &Query<&UiAxisGizmo>,
) -> Option<Entity> {
    for _ in 0..3 {
        e = parents.get(e).ok()?.parent();
        if roots.contains(e) {
            return Some(e);
        }
    }
    None
}

/// Everything a gizmo part may carry; the style system fills in what each part has.
type Parts<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GizmoPart,
        Option<&'static ControlPointer>,
        &'static mut Node,
        Option<&'static mut BackgroundColor>,
        Option<&'static mut BorderColor>,
        Option<&'static mut ZIndex>,
        Option<&'static mut UiTransform>,
    ),
>;

/// Positions and colours the balls, stems and home button from the camera basis and theme.
pub(crate) fn style_gizmo_parts(
    theme: Res<DodaiTheme>,
    roots: Query<&UiAxisGizmo>,
    parents: Query<&ChildOf>,
    mut parts: Parts,
    mut letters: Query<(Entity, &GizmoPart, &mut TextColor)>,
) {
    let opacity_of = |e: Entity| {
        find_root(e, &parents, &roots)
            .and_then(|r| roots.get(r).ok())
            .map_or(1.0, |g| g.opacity)
    };
    let p = &theme.palette;
    let bw = theme.shape.border_width.max(1.0);
    // Absolute children sit inside the root's border: shift by it.
    let placed_for = |root: &UiAxisGizmo| place(&root.basis, &root.layout, &AxisId::DEFAULT);
    let find = |placed: &[Placed], a: AxisId| placed.iter().copied().find(|x| x.axis == a);
    for (e, part, pointer, mut node, bg, bc, z, tf) in &mut parts {
        let Some(root_e) = find_root(e, &parents, &roots) else {
            continue;
        };
        let Ok(root) = roots.get(root_e) else {
            continue;
        };
        let layout = root.layout;
        let hovered = pointer.is_some_and(|p| *p != ControlPointer::None);
        let c = layout.centre();
        match *part {
            GizmoPart::Stem(a) => {
                let placed = placed_for(root);
                let Some(pl) = find(&placed, a) else { continue };
                let d = [pl.centre[0] - c[0], pl.centre[1] - c[1]];
                let len = d[0].hypot(d[1]);
                let mid = [(c[0] + pl.centre[0]) / 2.0, (c[1] + pl.centre[1]) / 2.0];
                upd(&mut node.width, Val::Px(len));
                upd(&mut node.left, Val::Px(mid[0] - len / 2.0 - bw));
                upd(&mut node.top, Val::Px(mid[1] - layout.stem / 2.0 - bw));
                if let Some(mut tf) = tf {
                    upd(&mut tf.rotation, Rot2::radians(d[1].atan2(d[0])));
                }
                if let Some(mut bg) = bg {
                    let col = tone(p, a.tone());
                    let col = if a.positive() {
                        col
                    } else {
                        col.with_alpha(0.55)
                    };
                    upd(&mut bg.0, color(fade(col, root.opacity)));
                }
                if let Some(mut z) = z {
                    upd(&mut z.0, (pl.depth * 40.0) as i32);
                }
            }
            GizmoPart::Ball(a) => {
                let placed = placed_for(root);
                let Some(pl) = find(&placed, a) else { continue };
                upd(&mut node.left, Val::Px(pl.centre[0] - pl.radius - bw));
                upd(&mut node.top, Val::Px(pl.centre[1] - pl.radius - bw));
                let base = tone(p, a.tone());
                if let Some(mut bg) = bg {
                    let fill = if a.positive() {
                        if hovered {
                            lighten(base, p.text, 0.3)
                        } else {
                            base
                        }
                    } else if hovered {
                        base
                    } else {
                        Rgba::rgba(0, 0, 0, 0)
                    };
                    upd(&mut bg.0, color(fade(fill, root.opacity)));
                }
                if let Some(mut bc) = bc {
                    upd(&mut *bc, BorderColor::all(color(fade(base, root.opacity))));
                }
                if let Some(mut z) = z {
                    upd(&mut z.0, 100 + (pl.depth * 40.0) as i32);
                }
            }
            GizmoPart::Home => {
                let hc = layout.home_centre();
                upd(&mut node.left, Val::Px(hc[0] - layout.home / 2.0 - bw));
                upd(&mut node.top, Val::Px(hc[1] - layout.home / 2.0 - bw));
                if let Some(mut bg) = bg {
                    let fill = if hovered {
                        p.surface_raised
                    } else {
                        p.surface_raised.with_alpha(0.85)
                    };
                    upd(&mut bg.0, color(fade(fill, root.opacity)));
                }
                if let Some(mut bc) = bc {
                    upd(
                        &mut *bc,
                        BorderColor::all(color(fade(p.border, root.opacity))),
                    );
                }
            }
            GizmoPart::Zoom(zoom_in) => {
                let hc = if zoom_in {
                    layout.zoom_in_centre()
                } else {
                    layout.zoom_out_centre()
                };
                upd(&mut node.left, Val::Px(hc[0] - layout.zoom / 2.0 - bw));
                upd(&mut node.top, Val::Px(hc[1] - layout.zoom / 2.0 - bw));
                if let Some(mut bg) = bg {
                    let fill = if hovered {
                        p.surface_raised
                    } else {
                        p.surface_raised.with_alpha(0.85)
                    };
                    upd(&mut bg.0, color(fade(fill, root.opacity)));
                }
                if let Some(mut bc) = bc {
                    upd(
                        &mut *bc,
                        BorderColor::all(color(fade(p.border, root.opacity))),
                    );
                }
            }
            GizmoPart::HomeInk | GizmoPart::ZoomInk => {
                if let Some(mut bg) = bg {
                    upd(&mut bg.0, color(fade(p.text, root.opacity)));
                }
            }
            GizmoPart::HomeCut => {
                if let Some(mut bg) = bg {
                    upd(&mut bg.0, color(fade(p.surface_raised, root.opacity)));
                }
            }
            GizmoPart::Letter(_) => {}
        }
    }
    for (e, part, mut tc) in &mut letters {
        if let GizmoPart::Letter(a) = *part {
            upd(
                &mut tc.0,
                color(fade(theme.on(tone(p, a.tone())), opacity_of(e))),
            );
        }
    }
}

/// A press inside a gizmo starts a new click-or-drag.
pub(crate) fn gizmo_press(press: On<PointerPress>, mut q: Query<&mut GizmoDrag>) {
    if press.button != PointerButton::Primary {
        return;
    }
    if let Ok(mut d) = q.get_mut(press.entity) {
        d.0.press();
    }
}

/// Dragging inside a gizmo reports the movement once it passes the click slop.
pub(crate) fn gizmo_drag(
    drag: On<PointerDrag>,
    mut q: Query<&mut GizmoDrag>,
    mut out: MessageWriter<GizmoEvent>,
) {
    if drag.button != PointerButton::Primary {
        return;
    }
    if let Ok(mut d) = q.get_mut(drag.entity) {
        d.0.add(drag.delta.x, drag.delta.y);
        if d.0.is_drag() {
            out.write(GizmoEvent {
                entity: drag.entity,
                kind: GizmoKind::Drag(drag.delta),
            });
        }
    }
}

/// What a click on `part` reports (`None`: the part is not clickable).
fn click_kind(part: GizmoPart) -> Option<GizmoKind> {
    match part {
        GizmoPart::Ball(a) => Some(GizmoKind::Select(a)),
        GizmoPart::Home => Some(GizmoKind::Home),
        GizmoPart::Zoom(true) => Some(GizmoKind::ZoomIn),
        GizmoPart::Zoom(false) => Some(GizmoKind::ZoomOut),
        _ => None,
    }
}

/// A click that was not a drag on a ball or the home button.
pub(crate) fn gizmo_click(
    click: On<PointerClick>,
    parts: Query<(&GizmoPart, &ChildOf)>,
    drags: Query<&GizmoDrag>,
    mut out: MessageWriter<GizmoEvent>,
) {
    if click.button != PointerButton::Primary {
        return;
    }
    let Ok((part, parent)) = parts.get(click.entity) else {
        return;
    };
    let root = parent.parent();
    if drags.get(root).is_ok_and(|d| d.0.is_drag()) {
        return;
    }
    let Some(kind) = click_kind(*part) else {
        return;
    };
    out.write(GizmoEvent { entity: root, kind });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(DodaiTheme::new())
            .add_systems(Update, (style_gizmo_roots, style_gizmo_parts));
        app
    }

    fn ball(app: &mut App, a: AxisId) -> (Val, Val, i32) {
        let mut q = app.world_mut().query::<(&GizmoPart, &Node, &ZIndex)>();
        q.iter(app.world())
            .find(|(p, _, _)| **p == GizmoPart::Ball(a))
            .map(|(_, n, z)| (n.left, n.top, z.0))
            .expect("ball")
    }

    #[test]
    fn up_ball_sits_above_the_centre_and_the_near_one_is_on_top() {
        let mut app = app();
        app.world_mut().spawn(axis_gizmo(130.0));
        app.update();
        let (_, top, _) = ball(&mut app, AxisId::PosZ);
        let Val::Px(top) = top else { panic!("px") };
        assert!(top < 65.0, "+Z is above the centre: {top}");
        // From the south the -Y ring points at the viewer, +Y away.
        let (_, _, near) = ball(&mut app, AxisId::NegY);
        let (_, _, far) = ball(&mut app, AxisId::PosY);
        assert!(near > far);
    }

    #[test]
    fn the_basis_moves_the_balls() {
        let mut app = app();
        let e = app.world_mut().spawn(axis_gizmo(130.0)).id();
        app.update();
        let (Val::Px(x0), ..) = ball(&mut app, AxisId::PosX) else {
            panic!("px")
        };
        app.world_mut()
            .get_mut::<UiAxisGizmo>(e)
            .expect("gizmo")
            .set_orbit(std::f32::consts::FRAC_PI_2, 0.0);
        app.update();
        let (Val::Px(x1), ..) = ball(&mut app, AxisId::PosX) else {
            panic!("px")
        };
        assert!(
            x1 < x0,
            "from the east +X points at the viewer, near the centre: {x0} {x1}"
        );
    }

    fn part_node(app: &mut App, part: GizmoPart) -> Node {
        let mut q = app.world_mut().query::<(&GizmoPart, &Node)>();
        q.iter(app.world())
            .find(|(p, _)| **p == part)
            .map(|(_, n)| n.clone())
            .expect("part")
    }

    fn part_fill(app: &mut App, part: GizmoPart) -> Color {
        let mut q = app.world_mut().query::<(&GizmoPart, &BackgroundColor)>();
        q.iter(app.world())
            .find(|(p, _)| **p == part)
            .map(|(_, b)| b.0)
            .expect("part")
    }

    #[test]
    fn plus_and_minus_sit_at_the_bottom_corners_with_their_bars() {
        let mut app = app();
        app.world_mut().spawn(axis_gizmo(130.0));
        app.update();
        let px = |v: Val| match v {
            Val::Px(x) => x,
            _ => panic!("px"),
        };
        let minus = part_node(&mut app, GizmoPart::Zoom(false));
        let plus = part_node(&mut app, GizmoPart::Zoom(true));
        let home = part_node(&mut app, GizmoPart::Home);
        assert!(px(minus.left) < 65.0 && px(plus.left) > 65.0, "left/right");
        assert!(
            px(minus.top) > 65.0 && px(plus.top) > 65.0,
            "both below the centre"
        );
        assert!(px(home.top) < 65.0, "home stays on top");
        // minus has one bar, plus has two
        let kids = |app: &mut App, e: Entity| {
            let mut q = app.world_mut().query::<(&GizmoPart, &ChildOf)>();
            q.iter(app.world())
                .filter(|(p, c)| **p == GizmoPart::ZoomInk && c.parent() == e)
                .count()
        };
        let mut q2 = app.world_mut().query::<(Entity, &GizmoPart)>();
        let ents: Vec<_> = q2
            .iter(app.world())
            .filter_map(|(e, p)| match p {
                GizmoPart::Zoom(z) => Some((*z, e)),
                _ => None,
            })
            .collect();
        for (zoom_in, e) in ents {
            assert_eq!(kids(&mut app, e), if zoom_in { 2 } else { 1 });
        }
    }

    #[test]
    fn opacity_scales_every_colour_and_clamps() {
        let mut app = app();
        let e = app.world_mut().spawn(axis_gizmo(130.0)).id();
        app.update();
        let alpha = |c: Color| c.to_srgba().alpha;
        let full_ball = alpha(part_fill(&mut app, GizmoPart::Ball(AxisId::PosZ)));
        let full_ink = alpha(part_fill(&mut app, GizmoPart::ZoomInk));
        assert!((full_ball - 1.0).abs() < 0.01 && (full_ink - 1.0).abs() < 0.01);
        app.world_mut()
            .get_mut::<UiAxisGizmo>(e)
            .expect("gizmo")
            .set_opacity(0.5);
        app.update();
        assert!((alpha(part_fill(&mut app, GizmoPart::Ball(AxisId::PosZ))) - 0.5).abs() < 0.01);
        assert!((alpha(part_fill(&mut app, GizmoPart::ZoomInk)) - 0.5).abs() < 0.01);
        assert!((alpha(part_fill(&mut app, GizmoPart::HomeInk)) - 0.5).abs() < 0.01);
        let mut g = app.world_mut().get_mut::<UiAxisGizmo>(e).expect("gizmo");
        g.set_opacity(7.0);
        assert_eq!(g.opacity, 1.0);
        g.set_opacity(-1.0);
        assert_eq!(g.opacity, 0.0);
    }

    #[test]
    fn backdrop_opacity_only_changes_the_disc() {
        let mut app = app();
        let e = app.world_mut().spawn(axis_gizmo(130.0)).id();
        app.update();
        let disc = |app: &mut App| {
            app.world()
                .get::<BackgroundColor>(e)
                .expect("disc")
                .0
                .to_srgba()
                .alpha
        };
        let ball = |app: &mut App| {
            part_fill(app, GizmoPart::Ball(AxisId::PosX))
                .to_srgba()
                .alpha
        };
        let (d0, b0) = (disc(&mut app), ball(&mut app));
        assert!(d0 < 1.0, "the disc is translucent by default: {d0}");
        app.world_mut()
            .get_mut::<UiAxisGizmo>(e)
            .expect("gizmo")
            .set_backdrop(0.2);
        app.update();
        assert!(disc(&mut app) < d0);
        assert!((ball(&mut app) - b0).abs() < 1e-4);
        let g = UiAxisGizmo {
            basis: Basis::from_orbit(0.0, 0.0),
            layout: GizmoLayout::new(100.0),
            opacity: 1.0,
            backdrop: DEFAULT_BACKDROP,
        }
        .with_opacity(2.0)
        .with_backdrop(-3.0);
        assert_eq!((g.opacity, g.backdrop), (1.0, 0.0));
    }

    #[test]
    fn clicking_reports_the_kind_of_each_button() {
        assert_eq!(click_kind(GizmoPart::Zoom(true)), Some(GizmoKind::ZoomIn));
        assert_eq!(click_kind(GizmoPart::Zoom(false)), Some(GizmoKind::ZoomOut));
        assert_eq!(click_kind(GizmoPart::Home), Some(GizmoKind::Home));
        assert_eq!(
            click_kind(GizmoPart::Ball(AxisId::PosX)),
            Some(GizmoKind::Select(AxisId::PosX))
        );
        // glyph bars, stems and letters are not buttons
        assert_eq!(click_kind(GizmoPart::ZoomInk), None);
        assert_eq!(click_kind(GizmoPart::HomeInk), None);
        assert_eq!(click_kind(GizmoPart::Stem(AxisId::PosX)), None);
    }
}
