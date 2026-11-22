//! The orientation gizmo of a 3D view on a canvas: stems and balls of the world axes for an orbit
//! camera, a home button, click a ball to look along that axis, drag to orbit. Geometry and hit
//! testing come from `dodai_ui_core::orientation`.

use dodai_ui_core::orientation::{
    hit, orbit, place, AxisId, AxisTone, Basis, DragTracker, GizmoLayout, Hit,
};
use dodai_ui_core::Palette;
use iced::{
    mouse,
    widget::canvas::{self, Canvas, Frame, Path, Stroke, Text},
    Element, Event, Point, Rectangle, Renderer, Theme as IcedBase,
};

use crate::{color, IcedTheme};

/// The camera a gizmo starts at and goes home to: yaw and pitch in radians.
pub const GIZMO_HOME: (f32, f32) = (0.6, 0.45);

/// Pointer state of a gizmo.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GizmoState {
    pressed: Option<Point>,
    drag: DragTracker,
}

struct Gizmo<Message> {
    lt: IcedTheme,
    yaw: f32,
    pitch: f32,
    size: f32,
    on_change: Box<dyn Fn(f32, f32) -> Message>,
}

/// Colour of an axis family: X danger, Y success, Z primary.
fn tone(p: &Palette, t: AxisTone) -> dodai_ui_core::Color {
    match t {
        AxisTone::X => p.danger,
        AxisTone::Y => p.success,
        AxisTone::Z => p.primary,
    }
}

impl<Message> Gizmo<Message> {
    fn layout(&self) -> GizmoLayout {
        GizmoLayout::new(self.size)
    }
}

impl<Message> canvas::Program<Message> for Gizmo<Message> {
    type State = GizmoState;

    fn draw(
        &self,
        _state: &GizmoState,
        renderer: &Renderer,
        _theme: &IcedBase,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let t = &self.lt.theme;
        let p = &t.palette;
        let layout = self.layout();
        let placed = place(
            &Basis::from_orbit(self.yaw, self.pitch),
            &layout,
            &AxisId::DEFAULT,
        );
        let over = cursor
            .position_in(bounds)
            .map(|q| hit(&layout, &placed, [q.x, q.y]));
        let mut frame = Frame::new(renderer, bounds.size());
        let [cx, cy] = layout.centre();
        let disc = Path::circle(Point::new(cx, cy), layout.size / 2.0 - 1.0);
        let mut back = color(p.surface);
        back.a *= 0.72;
        frame.fill(&disc, back);
        let mut rim = color(p.border_strong);
        rim.a *= 0.9;
        frame.stroke(
            &disc,
            Stroke::default()
                .with_color(rim)
                .with_width(t.shape.border_width.max(1.0)),
        );
        // Stems under everything, back to front.
        for a in &placed {
            let mut c = color(tone(p, a.axis.tone()));
            if !a.axis.positive() {
                c.a *= 0.55;
            }
            frame.stroke(
                &Path::line(Point::new(cx, cy), Point::new(a.centre[0], a.centre[1])),
                Stroke::default().with_color(c).with_width(layout.stem),
            );
        }
        for a in &placed {
            let base = tone(p, a.axis.tone());
            let hot = over == Some(Hit::Axis(a.axis));
            let at = Point::new(a.centre[0], a.centre[1]);
            let ball = Path::circle(at, a.radius);
            if a.axis.positive() {
                let fill = if hot { base.mix(p.text, 0.3) } else { base };
                frame.fill(&ball, color(fill));
                frame.fill_text(Text {
                    content: a.axis.letter().into(),
                    position: at,
                    color: color(t.on(base)),
                    size: (a.radius * 2.0 * 0.55).into(),
                    font: iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..iced::Font::DEFAULT
                    },
                    align_x: iced::widget::text::Alignment::Center,
                    align_y: iced::alignment::Vertical::Center,
                    ..Text::default()
                });
            } else {
                if hot {
                    frame.fill(&ball, color(base));
                }
                let w = (a.radius * 2.0 * 0.17).max(2.0);
                frame.stroke(
                    &Path::circle(at, a.radius - w / 2.0),
                    Stroke::default().with_color(color(base)).with_width(w),
                );
            }
        }
        // Home: a house on the circle line, top left.
        let [hx, hy] = layout.home_centre();
        let hr = layout.home / 2.0;
        let home = Path::circle(Point::new(hx, hy), hr);
        let mut hb = color(p.surface_raised);
        if over != Some(Hit::Home) {
            hb.a *= 0.85;
        }
        frame.fill(&home, hb);
        frame.stroke(
            &home,
            Stroke::default()
                .with_color(color(p.border))
                .with_width(t.shape.border_width.max(1.0)),
        );
        let s = hr * 0.55;
        let house = Path::new(|b| {
            b.move_to(Point::new(hx - s, hy));
            b.line_to(Point::new(hx, hy - s));
            b.line_to(Point::new(hx + s, hy));
            b.move_to(Point::new(hx - s * 0.7, hy - s * 0.2));
            b.line_to(Point::new(hx - s * 0.7, hy + s * 0.8));
            b.line_to(Point::new(hx + s * 0.7, hy + s * 0.8));
            b.line_to(Point::new(hx + s * 0.7, hy - s * 0.2));
        });
        frame.stroke(
            &house,
            Stroke::default().with_color(color(p.text)).with_width(1.5),
        );
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut GizmoState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let at = cursor.position_in(bounds)?;
                let layout = self.layout();
                if hit(&layout, &[], [at.x, at.y]) == Hit::Outside {
                    return None;
                }
                state.pressed = Some(at);
                state.drag.press();
                Some(canvas::Action::capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let from = state.pressed?;
                let p = cursor.position()?;
                let at = Point::new(p.x - bounds.x, p.y - bounds.y);
                let (dx, dy) = (at.x - from.x, at.y - from.y);
                state.drag.add(dx, dy);
                state.pressed = Some(at);
                if !state.drag.is_drag() {
                    return Some(canvas::Action::request_redraw());
                }
                let (yaw, pitch) = orbit(self.yaw, self.pitch, dx, dy);
                Some(canvas::Action::publish((self.on_change)(yaw, pitch)).and_capture())
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                let at = state.pressed.take()?;
                if state.drag.is_drag() {
                    return Some(canvas::Action::capture());
                }
                let layout = self.layout();
                let placed = place(
                    &Basis::from_orbit(self.yaw, self.pitch),
                    &layout,
                    &AxisId::DEFAULT,
                );
                let (yaw, pitch) = match hit(&layout, &placed, [at.x, at.y]) {
                    Hit::Axis(a) => {
                        let (y, p) = a.view_angles();
                        (y.unwrap_or(self.yaw), p)
                    }
                    Hit::Home => GIZMO_HOME,
                    _ => return Some(canvas::Action::capture()),
                };
                Some(canvas::Action::publish((self.on_change)(yaw, pitch)).and_capture())
            }
            _ => None,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &GizmoState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        match cursor.position_in(bounds) {
            Some(q) if hit(&self.layout(), &[], [q.x, q.y]) != Hit::Outside => {
                mouse::Interaction::Pointer
            }
            _ => mouse::Interaction::default(),
        }
    }
}

impl IcedTheme {
    /// An orientation gizmo `size` px wide for an orbit camera at `yaw` / `pitch` (radians,
    /// Z up). `on_change` gets the new camera: a clicked ball's view, home ([`GIZMO_HOME`]) or
    /// the orbit of a drag.
    pub fn axis_gizmo<'a, Message: 'a>(
        &self,
        (yaw, pitch): (f32, f32),
        size: f32,
        on_change: impl Fn(f32, f32) -> Message + 'static,
    ) -> Element<'a, Message> {
        Canvas::new(Gizmo {
            lt: self.clone(),
            yaw,
            pitch,
            size,
            on_change: Box::new(on_change),
        })
        .width(size)
        .height(size)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn it_builds_for_every_design() {
        for t in [cyberpunk(), rounded_light(), hard_square()] {
            let lt = IcedTheme::new(t);
            let _: Element<'_, (f32, f32)> = lt.axis_gizmo(GIZMO_HOME, 130.0, |y, p| (y, p));
        }
    }
}
