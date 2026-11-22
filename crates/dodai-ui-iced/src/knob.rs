//! A rotary knob on a canvas: drag up or right to increase, the wheel turns it too.
//! Geometry, drag maths and colours come from `dodai-ui-core`.

use dodai_ui_core::{arc_points, knob_angle, knob_value, KnobGeometry, KnobStyle};
use iced::{
    mouse,
    widget::canvas::{self, Canvas, Frame, Path, Stroke},
    Element, Event, Point, Rectangle, Renderer, Size, Theme as IcedBase,
};

use crate::{color, IcedTheme};

/// Pixels of wheel travel one notch counts for.
const WHEEL_NOTCH: f32 = 8.0;

/// The drag state of a knob and what each input does to its value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KnobRange {
    /// Lowest value.
    pub min: f32,
    /// Highest value.
    pub max: f32,
    /// Snap to multiples of this from `min`.
    pub step: Option<f32>,
}

/// Where a drag started: the pointer and the value then.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct KnobDrag {
    anchor: Option<(Point, f32)>,
}

impl KnobDrag {
    /// The pointer went down at `at` while the value was `value`.
    pub fn press(&mut self, at: Point, value: f32) {
        self.anchor = Some((at, value));
    }

    /// The new value when the pointer moved to `at` during a drag.
    pub fn moved(&self, at: Point, r: KnobRange) -> Option<f32> {
        let (a, v0) = self.anchor?;
        Some(knob_value(v0, at.x - a.x, at.y - a.y, r.min, r.max, r.step))
    }

    /// The pointer went up.
    pub fn release(&mut self) {
        self.anchor = None;
    }

    /// Whether a drag is under way.
    pub fn dragging(&self) -> bool {
        self.anchor.is_some()
    }
}

/// The new value after a wheel turn of `dy` (positive = up = increase).
pub fn wheel_value(value: f32, dy: f32, r: KnobRange) -> f32 {
    knob_value(value, 0.0, -dy.signum() * WHEEL_NOTCH, r.min, r.max, r.step)
}

struct Knob<Message> {
    style: KnobStyle,
    range: KnobRange,
    value: f32,
    disabled: bool,
    on_change: Box<dyn Fn(f32) -> Message>,
}

fn stroke(path: &Path, c: dodai_ui_core::Color, w: f32) -> (Path, Stroke<'static>) {
    (
        path.clone(),
        Stroke::default().with_color(color(c)).with_width(w),
    )
}

fn polyline(pts: &[(f32, f32)]) -> Path {
    Path::new(|b| {
        let mut it = pts.iter().map(|&(x, y)| Point::new(x, y));
        if let Some(p) = it.next() {
            b.move_to(p);
            it.for_each(|p| b.line_to(p));
        }
    })
}

impl<Message> canvas::Program<Message> for Knob<Message> {
    type State = KnobDrag;

    fn draw(
        &self,
        _state: &KnobDrag,
        renderer: &Renderer,
        _theme: &IcedBase,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let size = bounds.width.min(bounds.height);
        let g = KnobGeometry::new(size);
        let st = &self.style;
        let c = size / 2.0;
        let span = (self.range.max - self.range.min).max(f32::EPSILON);
        let frac = ((self.value - self.range.min) / span).clamp(0.0, 1.0);
        let mut frame = Frame::new(renderer, Size::new(size, size));
        let (p, s) = stroke(
            &polyline(&arc_points(c, c, g.r_ring, 0.0, 1.0)),
            st.track,
            g.ring,
        );
        frame.stroke(&p, s);
        if frac > 0.0 {
            let arc = polyline(&arc_points(c, c, g.r_ring, 0.0, frac));
            if let Some((glow, r)) = st.glow {
                let (p, s) = stroke(
                    &arc,
                    glow.with_alpha(glow.a as f32 / 255.0 * 0.4),
                    g.ring + r * 0.6,
                );
                frame.stroke(&p, s);
            }
            let (p, s) = stroke(&arc, st.fill, g.ring);
            frame.stroke(&p, s);
        }
        let disc = Path::circle(Point::new(c, c), g.r_body);
        frame.fill(&disc, color(st.body));
        let (p, s) = stroke(&disc, st.body_edge, st.border_width);
        frame.stroke(&p, s);
        let th = knob_angle(frac);
        let (co, si) = (th.cos(), th.sin());
        let tick = Path::line(
            Point::new(c + co * g.r_body * 0.35, c + si * g.r_body * 0.35),
            Point::new(c + co * g.r_body * 0.9, c + si * g.r_body * 0.9),
        );
        let (p, s) = stroke(&tick, st.tick, g.tick);
        frame.stroke(&p, s);
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut KnobDrag,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if self.disabled {
            return None;
        }
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let at = cursor.position_in(bounds)?;
                state.press(at, self.value);
                Some(canvas::Action::capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging() => {
                let at = cursor.position()?;
                let v = state.moved(Point::new(at.x - bounds.x, at.y - bounds.y), self.range)?;
                Some(canvas::Action::publish((self.on_change)(v)).and_capture())
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.dragging() => {
                state.release();
                Some(canvas::Action::capture())
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let dy = match delta {
                    mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => {
                        *y
                    }
                };
                (dy != 0.0).then(|| {
                    canvas::Action::publish((self.on_change)(wheel_value(
                        self.value, dy, self.range,
                    )))
                    .and_capture()
                })
            }
            _ => None,
        }
    }
}

impl IcedTheme {
    /// A rotary knob `size` px across over `min..=max`, calling `on_change` with the new value.
    pub fn knob<'a, Message: 'a>(
        &self,
        value: f32,
        range: KnobRange,
        size: f32,
        disabled: bool,
        on_change: impl Fn(f32) -> Message + 'static,
    ) -> Element<'a, Message> {
        Canvas::new(Knob {
            style: self.theme.knob(disabled),
            range,
            value,
            disabled,
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

    const R: KnobRange = KnobRange {
        min: 0.0,
        max: 1.0,
        step: None,
    };

    #[test]
    fn a_drag_moves_the_value_from_where_it_started() {
        let mut d = KnobDrag::default();
        assert_eq!(d.moved(Point::new(5.0, 5.0), R), None);
        d.press(Point::new(10.0, 10.0), 0.5);
        assert!(d.dragging());
        assert!(d.moved(Point::new(10.0, -10.0), R).unwrap() > 0.5);
        assert!(d.moved(Point::new(10.0, 30.0), R).unwrap() < 0.5);
        assert_eq!(d.moved(Point::new(10.0, 10.0), R), Some(0.5));
        d.release();
        assert!(!d.dragging());
        assert_eq!(d.moved(Point::new(0.0, 0.0), R), None);
    }

    #[test]
    fn the_wheel_turns_up_to_increase_and_stays_in_range() {
        assert!(wheel_value(0.5, 1.0, R) > 0.5);
        assert!(wheel_value(0.5, -1.0, R) < 0.5);
        assert_eq!(wheel_value(1.0, 1.0, R), 1.0);
        assert_eq!(wheel_value(0.0, -1.0, R), 0.0);
    }

    #[test]
    fn it_builds_for_every_design() {
        for t in [cyberpunk(), rounded_light(), hard_square()]
            .into_iter()
            .chain(dodai_ui_core::themes::all())
        {
            let lt = IcedTheme::new(t);
            let _: Element<'_, f32> = lt.knob(0.4, R, 72.0, false, |v| v);
            let _: Element<'_, f32> = lt.knob(0.4, R, 72.0, true, |v| v);
        }
    }
}
