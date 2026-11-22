//! [`AxisGizmo`]: the corner orientation widget of a 3D view. Filled balls mark +X, +Y and +Z,
//! hollow rings the negative horizontal axes; click one to jump to that view, click the house
//! to go home, drag anywhere inside to orbit.
//!
//! The geometry (placement, hit test, click-or-drag) is `dodai_ui_core::orientation`;
//! this file only paints it with theme colours and forwards the pointer. The gizmo is controlled:
//! give it the camera [`Basis`] and change your camera in the callbacks.

use crate::{hsla, motion::mix, paint::scale_alpha, theme_of};
use dodai_ui_core::orientation::{hit, place, AxisId, AxisTone, DragTracker, GizmoLayout, Hit};
use dodai_ui_core::{Color, Palette};
use gpui::{
    canvas, div, point, px, App, Bounds, ElementId, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, PathBuilder, Pixels, Point, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

pub use dodai_ui_core::orientation::{orbit, Basis};

type AxisHandler = Rc<dyn Fn(AxisId, &mut Window, &mut App)>;
type DragHandler = Rc<dyn Fn(f32, f32, &mut Window, &mut App)>;
type Action = Rc<dyn Fn(&mut Window, &mut App)>;

struct GizmoState {
    /// Bounds of the gizmo square from the last paint, for pointer maths.
    bounds: Rc<Cell<Bounds<Pixels>>>,
    /// What the current press started on.
    pressed: Option<Hit>,
    last: (f32, f32),
    drag: DragTracker,
    hover: Hit,
}

impl GizmoState {
    fn new() -> Self {
        Self {
            bounds: Rc::new(Cell::new(Bounds::default())),
            pressed: None,
            last: (0.0, 0.0),
            drag: DragTracker::default(),
            hover: Hit::Outside,
        }
    }
}

/// Colour of an axis family, from the palette: X danger, Y success, Z primary.
fn tone_color(p: &Palette, tone: AxisTone) -> Color {
    match tone {
        AxisTone::X => p.danger,
        AxisTone::Y => p.success,
        AxisTone::Z => p.primary,
    }
}

/// Orientation gizmo for a 3D view (Z up). Controlled: pass the camera basis.
#[derive(IntoElement)]
pub struct AxisGizmo {
    id: ElementId,
    basis: Basis,
    size: f32,
    axes: Vec<AxisId>,
    on_select: Option<AxisHandler>,
    on_drag: Option<DragHandler>,
    on_home: Option<Action>,
}

impl AxisGizmo {
    /// Create a new [`AxisGizmo`] for a camera `basis` (see [`Basis::from_orbit`]).
    pub fn new(id: impl Into<ElementId>, basis: Basis) -> Self {
        Self {
            id: id.into(),
            basis,
            size: 130.0,
            axes: AxisId::DEFAULT.to_vec(),
            on_select: None,
            on_drag: None,
            on_home: None,
        }
    }
    /// Side of the round gizmo in px (default 130).
    pub fn size(mut self, px: f32) -> Self {
        self.size = px;
        self
    }
    /// Which axes to show (default: +X, +Y, +Z, -X, -Y).
    pub fn axes(mut self, axes: impl IntoIterator<Item = AxisId>) -> Self {
        self.axes = axes.into_iter().collect();
        self
    }
    crate::callback!(
        /// Called when an axis ball is clicked: show the view from that side
        /// ([`AxisId::view_angles`]).
        on_select,
        AxisId
    );
    crate::callback!(
        /// Called with the pointer movement in px while dragging inside the gizmo
        /// (feed it to [`orbit`]).
        on_drag,
        f32,
        f32
    );

    /// Called when the home button is clicked.
    pub fn on_home(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_home = Some(Rc::new(f));
        self
    }
}

fn local(pos: Point<Pixels>, b: Bounds<Pixels>) -> [f32; 2] {
    let (x, y): (f32, f32) = ((pos.x - b.origin.x).into(), (pos.y - b.origin.y).into());
    [x, y]
}

impl AxisGizmo {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let layout = GizmoLayout::new(self.size);
        let placed = place(&self.basis, &layout, &self.axes);
        let st = window.use_keyed_state(self.id.clone(), cx, |_, _| GizmoState::new());
        let hover = st.read(cx).hover;
        let bounds = st.read(cx).bounds.clone();
        let p = &t.palette;
        let border_w = t.shape.border_width.max(1.0);
        let ink = hsla(p.text);

        // Stems under everything, one stroke each, in depth order.
        let stems: Vec<([f32; 2], gpui::Hsla, f32)> = placed
            .iter()
            .map(|a| {
                let c = hsla(tone_color(p, a.axis.tone()));
                let c = if a.axis.positive() {
                    c
                } else {
                    scale_alpha(c, 0.55)
                };
                (a.centre, c, layout.stem)
            })
            .collect();
        let centre = layout.centre();
        let mut root = div()
            .id(self.id.clone())
            .relative()
            .w(px(self.size))
            .h(px(self.size))
            .rounded_full()
            .border(px(border_w))
            .border_color(scale_alpha(hsla(p.border_strong), 0.9))
            .bg(scale_alpha(hsla(p.surface), 0.72))
            .cursor_pointer()
            .child(
                canvas(
                    |_, _, _| (),
                    move |b: Bounds<Pixels>, _, window, _| {
                        bounds.set(b);
                        for (end, color, w) in &stems {
                            let at =
                                |q: [f32; 2]| point(b.origin.x + px(q[0]), b.origin.y + px(q[1]));
                            let mut pb = PathBuilder::stroke(px(*w));
                            pb.add_polygon(&[at(centre), at(*end)], false);
                            if let Ok(path) = pb.build() {
                                window.paint_path(path, *color);
                            }
                        }
                    },
                )
                .absolute()
                .size_full(),
            );

        for a in &placed {
            let base = hsla(tone_color(p, a.axis.tone()));
            let hot = hover == Hit::Axis(a.axis);
            let d = a.radius * 2.0;
            let mut ball = div()
                .absolute()
                .left(px(a.centre[0] - a.radius))
                .top(px(a.centre[1] - a.radius))
                .w(px(d))
                .h(px(d))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center();
            if a.axis.positive() {
                let fill = if hot { mix(base, ink, 0.3) } else { base };
                ball = ball
                    .bg(fill)
                    .text_color(hsla(t.on(tone_color(p, a.axis.tone()))))
                    .text_size(px(d * 0.55))
                    .font_weight(FontWeight::BOLD)
                    .child(a.axis.letter());
            } else {
                let w = (d * 0.17).max(2.0);
                ball = ball.border(px(w)).border_color(base);
                if hot {
                    ball = ball.bg(base);
                }
            }
            root = root.child(ball);
        }

        // Home: a house on the circle line, top left.
        let hc = layout.home_centre();
        let hr = layout.home / 2.0;
        let home_bg = if hover == Hit::Home {
            hsla(p.surface_raised)
        } else {
            scale_alpha(hsla(p.surface_raised), 0.85)
        };
        root = root.child(
            div()
                .absolute()
                .left(px(hc[0] - hr))
                .top(px(hc[1] - hr))
                .w(px(layout.home))
                .h(px(layout.home))
                .rounded_full()
                .bg(home_bg)
                .border(px(border_w))
                .border_color(hsla(p.border))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    crate::icons::Icon::new(crate::icons::IconName::Home)
                        .size(layout.home * 0.55)
                        .color(ink),
                ),
        );

        let (down, mv, up, up2) = (st.clone(), st.clone(), st.clone(), st.clone());
        let (placed_down, placed_move) = (placed.clone(), placed);
        let on_drag = self.on_drag.clone();
        let (on_select, on_home) = (self.on_select.clone(), self.on_home.clone());
        root.on_mouse_down(MouseButton::Left, move |e, _, cx| {
            down.update(cx, |s, _| {
                let q = local(e.position, s.bounds.get());
                s.pressed = Some(hit(&layout, &placed_down, q));
                s.last = (e.position.x.into(), e.position.y.into());
                s.drag.press();
            });
        })
        .on_mouse_move(move |e, w, cx| {
            let q = local(e.position, mv.read(cx).bounds.get());
            let (x, y): (f32, f32) = (e.position.x.into(), e.position.y.into());
            let pressed = mv.read(cx).pressed.is_some();
            if pressed {
                let (dx, dy, drag) = mv.update(cx, |s, _| {
                    let d = (x - s.last.0, y - s.last.1);
                    s.last = (x, y);
                    s.drag.add(d.0, d.1);
                    (d.0, d.1, s.drag.is_drag())
                });
                if drag {
                    if let Some(f) = &on_drag {
                        f(dx, dy, w, cx);
                    }
                }
            } else {
                let h = hit(&layout, &placed_move, q);
                mv.update(cx, |s, cx| {
                    if s.hover != h {
                        s.hover = h;
                        cx.notify();
                    }
                });
            }
        })
        .on_mouse_up(MouseButton::Left, move |_, w, cx| {
            let (target, drag) = up.update(cx, |s, _| (s.pressed.take(), s.drag.is_drag()));
            if drag {
                return;
            }
            match target {
                Some(Hit::Axis(a)) => {
                    if let Some(f) = &on_select {
                        f(a, w, cx);
                    }
                }
                Some(Hit::Home) => {
                    if let Some(f) = &on_home {
                        f(w, cx);
                    }
                }
                _ => {}
            }
        })
        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
            up2.update(cx, |s, _| s.pressed = None)
        })
    }
}

crate::entry_render!(AxisGizmo);
