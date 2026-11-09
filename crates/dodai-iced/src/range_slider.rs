//! A two-thumb range slider on a canvas, horizontal or vertical: the nearer thumb follows the
//! pointer and the thumbs never cross. Maths from `dodai_core::slider_math`, colours from the
//! slider tokens (the rail of `progress`, thumbs of a switched-on `toggle`).

use dodai_core::slider_math::{
    nearest_thumb, range_update, slider_fraction, slider_value, tick_fractions,
};
use dodai_core::{ControlState, ShapeKind};
use iced::{
    mouse,
    widget::canvas::{self, Canvas, Frame, Path, Stroke},
    Element, Event, Length, Point, Rectangle, Renderer, Size, Theme as IcedBase,
};

use crate::{color, IcedTheme};

/// The span a range slider covers and how it moves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RangeSpec {
    /// Lowest value.
    pub min: f32,
    /// Highest value.
    pub max: f32,
    /// Snap to multiples of this from `min`.
    pub step: Option<f32>,
    /// Marks at every step.
    pub ticks: bool,
    /// Runs bottom to top, this many px long (else as wide as it is given).
    pub vertical: Option<f32>,
}

/// Across the track px (the thumb's long side).
const GIRTH: f32 = 20.0;

/// Which thumb is being dragged.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RangeDrag {
    thumb: Option<usize>,
}

struct Range<Message> {
    lt: IcedTheme,
    lo: f32,
    hi: f32,
    spec: RangeSpec,
    on_change: Box<dyn Fn(f32, f32) -> Message>,
}

impl<Message> Range<Message> {
    /// Thumb length along the track.
    fn thumb(&self) -> f32 {
        if self.lt.theme.shape.kind() == ShapeKind::Rounded {
            16.0
        } else {
            10.0
        }
    }

    /// The value under `at` (inside `size`).
    fn value_at(&self, at: Point, size: Size) -> f32 {
        let s = &self.spec;
        let (pos, len) = match s.vertical {
            Some(_) => (size.height - at.y, size.height),
            None => (at.x, size.width),
        };
        slider_value(pos, 0.0, len, self.thumb(), s.min, s.max, s.step)
    }
}

impl<Message> canvas::Program<Message> for Range<Message> {
    type State = RangeDrag;

    fn draw(
        &self,
        state: &RangeDrag,
        renderer: &Renderer,
        _theme: &IcedBase,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let t = &self.lt.theme;
        let rail = t.progress(None);
        let mut frame = Frame::new(renderer, bounds.size());
        let s = &self.spec;
        let vertical = s.vertical.is_some();
        let len = if vertical {
            bounds.height
        } else {
            bounds.width
        };
        let thumb = self.thumb();
        let travel = (len - thumb).max(1.0);
        // Along-track position of a fraction, from the start (bottom when vertical).
        let at = |f: f32| thumb / 2.0 + f * travel;
        // A rectangle `a..b` along the track, `w` across, centred.
        let bar = |a: f32, b: f32, w: f32| {
            let (a, b) = (a.min(b), a.max(b));
            if vertical {
                Path::rectangle(
                    Point::new((GIRTH - w) / 2.0, bounds.height - b),
                    Size::new(w, b - a),
                )
            } else {
                Path::rectangle(Point::new(a, (GIRTH - w) / 2.0), Size::new(b - a, w))
            }
        };
        frame.fill(&bar(0.0, len, 4.0), color(rail.track));
        if s.ticks {
            for f in tick_fractions(s.min, s.max, s.step) {
                let x = at(f);
                frame.fill(&bar(x - 0.5, x + 0.5, 10.0), color(rail.track));
            }
        }
        let (fa, fb) = (
            slider_fraction(self.lo, s.min, s.max),
            slider_fraction(self.hi, s.min, s.max),
        );
        frame.fill(&bar(at(fa), at(fb), 4.0), color(rail.fill));
        for (i, f) in [fa, fb].into_iter().enumerate() {
            let st = if state.thumb == Some(i) {
                ControlState::Pressed
            } else {
                ControlState::Idle
            };
            let k = t.toggle(true, st);
            let c = at(f);
            let path = if t.shape.kind() == ShapeKind::Rounded {
                let centre = if vertical {
                    Point::new(GIRTH / 2.0, bounds.height - c)
                } else {
                    Point::new(c, GIRTH / 2.0)
                };
                Path::circle(centre, thumb / 2.0)
            } else {
                bar(c - thumb / 2.0, c + thumb / 2.0, GIRTH)
            };
            frame.fill(&path, color(k.track));
            if t.shape.border_width > 0.0 {
                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_color(color(k.border))
                        .with_width(t.shape.border_width),
                );
            }
        }
        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut RangeDrag,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let at = cursor.position_in(bounds)?;
                let v = self.value_at(at, bounds.size());
                let thumb = nearest_thumb(self.lo, self.hi, v);
                state.thumb = Some(thumb);
                let (lo, hi) = range_update(thumb, v, self.lo, self.hi);
                Some(canvas::Action::publish((self.on_change)(lo, hi)).and_capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let thumb = state.thumb?;
                let p = cursor.position()?;
                let v = self.value_at(Point::new(p.x - bounds.x, p.y - bounds.y), bounds.size());
                let (lo, hi) = range_update(thumb, v, self.lo, self.hi);
                Some(canvas::Action::publish((self.on_change)(lo, hi)).and_capture())
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.thumb.take().map(|_| canvas::Action::capture())
            }
            _ => None,
        }
    }
}

impl IcedTheme {
    /// A two-thumb slider over `spec`, thumbs at `lo` and `hi`, calling `on_change` with the
    /// new `(lo, hi)` (always `lo <= hi`).
    pub fn range_slider<'a, Message: 'a>(
        &self,
        (lo, hi): (f32, f32),
        spec: RangeSpec,
        on_change: impl Fn(f32, f32) -> Message + 'static,
    ) -> Element<'a, Message> {
        let c = Canvas::new(Range {
            lt: self.clone(),
            lo: lo.min(hi),
            hi: hi.max(lo),
            spec,
            on_change: Box::new(on_change),
        });
        match spec.vertical {
            Some(len) => c.width(GIRTH).height(len),
            None => c.width(Length::Fill).height(GIRTH),
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn it_builds_for_every_design_both_ways() {
        let spec = RangeSpec {
            min: 0.0,
            max: 1.0,
            step: Some(0.1),
            ticks: true,
            vertical: None,
        };
        for t in [cyberpunk(), rounded_light(), hard_square()] {
            let lt = IcedTheme::new(t);
            let _: Element<'_, (f32, f32)> = lt.range_slider((0.2, 0.7), spec, |a, b| (a, b));
            let v = RangeSpec {
                vertical: Some(120.0),
                ..spec
            };
            let _: Element<'_, (f32, f32)> = lt.range_slider((0.7, 0.2), v, |a, b| (a, b));
        }
    }

    #[test]
    fn the_pointer_maps_to_a_value_along_the_track() {
        let r = Range {
            lt: IcedTheme::new(hard_square()),
            lo: 0.2,
            hi: 0.7,
            spec: RangeSpec {
                min: 0.0,
                max: 1.0,
                step: None,
                ticks: false,
                vertical: Some(110.0),
            },
            on_change: Box::new(|a, b| (a, b)),
        };
        let size = Size::new(GIRTH, 110.0);
        assert_eq!(
            r.value_at(Point::new(10.0, 110.0), size),
            0.0,
            "bottom is min"
        );
        assert_eq!(r.value_at(Point::new(10.0, 0.0), size), 1.0, "top is max");
    }
}
