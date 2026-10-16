//! [`Knob`]: rotary control. Drag up/right to increase, down/left to decrease;
//! the mouse wheel also turns it. Sweep is 270° starting bottom-left.
//!
//! Colours come from the toggle/progress tokens, the glow from `Effects`, so
//! it follows every theme without knowing which one is active.

use crate::{controls::slider_fraction, hsla, theme_of};
use dodai_core::ControlState;
use gpui::{
    canvas, div, point, px, App, Bounds, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, PathBuilder, Pixels, RenderOnce, ScrollDelta, Styled, Window,
};
use std::rc::Rc;

/// Start of the sweep, degrees clockwise from +x (screen space): bottom-left.
pub const KNOB_START_DEG: f32 = 135.0;
/// Total sweep in degrees.
pub const KNOB_SWEEP_DEG: f32 = 270.0;
/// Pixels of drag that cover the whole range.
pub const KNOB_DRAG_PX: f32 = 160.0;

/// Angle in radians (screen space, clockwise from +x) for a 0..1 fraction.
pub fn knob_angle(frac: f32) -> f32 {
    (KNOB_START_DEG + frac.clamp(0.0, 1.0) * KNOB_SWEEP_DEG).to_radians()
}

/// New value after dragging `dx`/`dy` px (screen space; up and right increase)
/// from `start`. `step` snaps to multiples from `min`.
pub fn knob_value(start: f32, dx: f32, dy: f32, min: f32, max: f32, step: Option<f32>) -> f32 {
    let mut v = start + (dx - dy) / KNOB_DRAG_PX * (max - min);
    if let Some(s) = step.filter(|s| *s > 0.0) {
        v = min + ((v - min) / s).round() * s;
    }
    v.clamp(min.min(max), max.max(min))
}

/// Points of the arc from fraction `a` to `b` on a circle.
pub fn arc_points(cx: f32, cy: f32, r: f32, a: f32, b: f32) -> Vec<(f32, f32)> {
    let n = (((b - a).abs() * KNOB_SWEEP_DEG / 6.0).ceil() as usize).max(2);
    (0..=n)
        .map(|i| {
            let th = knob_angle(a + (b - a) * i as f32 / n as f32);
            (cx + r * th.cos(), cy + r * th.sin())
        })
        .collect()
}

type FloatHandler = Rc<dyn Fn(f32, &mut Window, &mut App)>;

#[derive(Default)]
struct KnobState {
    /// (pointer x, pointer y, value) at mouse down.
    anchor: Option<(f32, f32, f32)>,
}

/// Rotary knob over `min..=max` (default `0..=1`). Controlled.
#[derive(IntoElement)]
pub struct Knob {
    id: ElementId,
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    size: f32,
    disabled: bool,
    on_change: Option<FloatHandler>,
}

impl Knob {
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            id: id.into(),
            value,
            min: 0.0,
            max: 1.0,
            step: None,
            size: 72.0,
            disabled: false,
            on_change: None,
        }
    }
    /// Value range.
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    /// Snap to multiples of `step` from `min`.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
    /// Diameter in px (default 72).
    pub fn size(mut self, px: f32) -> Self {
        self.size = px;
        self
    }
    /// Disable interaction and dim.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Called with the new value while dragging or scrolling.
    pub fn on_change(mut self, f: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

fn pts(origin: gpui::Point<Pixels>, p: &[(f32, f32)]) -> Vec<gpui::Point<Pixels>> {
    p.iter()
        .map(|&(x, y)| point(origin.x + px(x), origin.y + px(y)))
        .collect()
}

impl Knob {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = window.use_keyed_state(self.id.clone(), cx, |_, _| KnobState::default());
        let frac = slider_fraction(self.value, self.min, self.max);
        let state = if self.disabled {
            ControlState::Disabled
        } else {
            ControlState::Idle
        };
        let on = t.toggle(true, state);
        let off = t.toggle(false, state);
        let prog = t.progress(None);
        let (body, body_edge, track_c, fill_c, tick_c) = (
            hsla(off.track),
            hsla(off.border),
            hsla(prog.track),
            hsla(prog.fill),
            hsla(on.thumb),
        );
        let glow = (!self.disabled && t.effects.glow_enabled())
            .then(|| (hsla(t.effects.glow_color()), t.effects.glow_radius()));
        let bw = t.shape.border_width.max(1.0);
        let size = self.size;
        let ring = (size * 0.09).clamp(3.0, 8.0);
        let (min, max, step) = (self.min, self.max, self.step);
        let on_change = self.on_change.clone();
        let disabled = self.disabled;

        let (down, mv, up, up2) = (st.clone(), st.clone(), st.clone(), st.clone());
        let (f1, f2) = (on_change.clone(), on_change.clone());
        let value = self.value;
        div()
            .id(self.id)
            .relative()
            .w(px(size))
            .h(px(size))
            .when_enabled(disabled)
            .child(
                canvas(
                    |_, _, _| (),
                    move |b: Bounds<Pixels>, _, window, _| {
                        let (cx0, cy0) = (size / 2.0, size / 2.0);
                        let r_ring = size / 2.0 - ring / 2.0 - 1.0;
                        let r_body = r_ring - ring / 2.0 - size * 0.06;
                        // Track arc.
                        let mut pb = PathBuilder::stroke(px(ring));
                        pb.add_polygon(
                            &pts(b.origin, &arc_points(cx0, cy0, r_ring, 0.0, 1.0)),
                            false,
                        );
                        if let Ok(p) = pb.build() {
                            window.paint_path(p, track_c);
                        }
                        // Value arc (+ glow layers).
                        if frac > 0.0 {
                            let arc = pts(b.origin, &arc_points(cx0, cy0, r_ring, 0.0, frac));
                            if let Some((gc, gr)) = glow {
                                for l in crate::paint::glow_layers(gr, 0.0) {
                                    let mut pb = PathBuilder::stroke(px(ring + l.width));
                                    pb.add_polygon(&arc, false);
                                    if let Ok(p) = pb.build() {
                                        window
                                            .paint_path(p, crate::paint::scale_alpha(gc, l.alpha));
                                    }
                                }
                            }
                            let mut pb = PathBuilder::stroke(px(ring));
                            pb.add_polygon(&arc, false);
                            if let Ok(p) = pb.build() {
                                window.paint_path(p, fill_c);
                            }
                        }
                        // Body disc + edge.
                        let disc = pts(b.origin, &full_circle(cx0, cy0, r_body));
                        let mut pb = PathBuilder::fill();
                        pb.add_polygon(&disc, true);
                        if let Ok(p) = pb.build() {
                            window.paint_path(p, body);
                        }
                        let mut pb = PathBuilder::stroke(px(bw));
                        pb.add_polygon(&disc, true);
                        if let Ok(p) = pb.build() {
                            window.paint_path(p, body_edge);
                        }
                        // Indicator tick.
                        let th = knob_angle(frac);
                        let (c, s) = (th.cos(), th.sin());
                        let tick = [
                            (cx0 + c * r_body * 0.35, cy0 + s * r_body * 0.35),
                            (cx0 + c * r_body * 0.9, cy0 + s * r_body * 0.9),
                        ];
                        let mut pb = PathBuilder::stroke(px((size * 0.05).max(2.0)));
                        pb.add_polygon(&pts(b.origin, &tick), false);
                        if let Ok(p) = pb.build() {
                            window.paint_path(p, tick_c);
                        }
                    },
                )
                .absolute()
                .size_full(),
            )
            .on_mouse_down(MouseButton::Left, move |e, _, cx| {
                if !disabled {
                    down.update(cx, |s, _| {
                        s.anchor = Some((e.position.x.into(), e.position.y.into(), value))
                    });
                }
            })
            .on_mouse_move(move |e, w, cx| {
                if let (Some((ax, ay, v0)), Some(f)) = (mv.read(cx).anchor, &f1) {
                    let (x, y): (f32, f32) = (e.position.x.into(), e.position.y.into());
                    f(knob_value(v0, x - ax, y - ay, min, max, step), w, cx);
                }
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                up.update(cx, |s, _| s.anchor = None)
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                up2.update(cx, |s, _| s.anchor = None)
            })
            .on_scroll_wheel(move |e, w, cx| {
                if disabled {
                    return;
                }
                let dy: f32 = match e.delta {
                    ScrollDelta::Pixels(p) => p.y.into(),
                    ScrollDelta::Lines(l) => l.y * 16.0,
                };
                if let Some(f) = &f2 {
                    f(knob_value(value, 0.0, -dy, min, max, step), w, cx);
                }
            })
    }
}

impl RenderOnce for Knob {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "Knob", motion)
    }
}

fn full_circle(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    (0..64)
        .map(|i| {
            let th = i as f32 / 64.0 * std::f32::consts::TAU;
            (cx + r * th.cos(), cy + r * th.sin())
        })
        .collect()
}

trait WhenEnabled: Sized {
    fn when_enabled(self, disabled: bool) -> Self;
}
impl WhenEnabled for gpui::Stateful<gpui::Div> {
    fn when_enabled(self, disabled: bool) -> Self {
        if disabled {
            self.opacity(0.5)
        } else {
            self.cursor_pointer()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_sweeps_270() {
        let a0 = knob_angle(0.0).to_degrees();
        let a1 = knob_angle(1.0).to_degrees();
        assert!((a0 - 135.0).abs() < 1e-3);
        assert!((a1 - a0 - 270.0).abs() < 1e-3);
        assert_eq!(knob_angle(-3.0), knob_angle(0.0));
    }

    #[test]
    fn drag_up_or_right_increases() {
        assert!(knob_value(0.5, 0.0, -40.0, 0.0, 1.0, None) > 0.5);
        assert!(knob_value(0.5, 40.0, 0.0, 0.0, 1.0, None) > 0.5);
        assert!(knob_value(0.5, 0.0, 40.0, 0.0, 1.0, None) < 0.5);
        assert_eq!(knob_value(0.5, 0.0, -9999.0, 0.0, 1.0, None), 1.0);
        assert_eq!(knob_value(0.5, 0.0, 9999.0, 0.0, 1.0, None), 0.0);
        assert_eq!(knob_value(0.0, 0.0, -48.0, 0.0, 10.0, Some(2.0)), 4.0);
    }

    #[test]
    fn arc_endpoints() {
        let p = arc_points(50.0, 50.0, 10.0, 0.0, 1.0);
        let (x0, y0) = p[0];
        let (x1, y1) = *p.last().unwrap();
        // bottom-left start, bottom-right end, symmetric
        assert!(x0 < 50.0 && y0 > 50.0);
        assert!(x1 > 50.0 && (y1 - y0).abs() < 1e-3);
    }
}
