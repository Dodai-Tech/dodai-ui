//! Chamfered button: the real cyberpunk.net shape (square, bottom-right corner cut).
//!
//! `iced::widget::button` can only draw rounded rectangles, so this is a small
//! canvas widget that paints the polygon, label and hover/press states itself.

use iced::{
    alignment, mouse,
    widget::{
        canvas::{self, Action, Canvas, Geometry, Path, Stroke, Text},
        text::Alignment,
    },
    Color, Element, Event, Font, Length, Pixels, Point, Rectangle, Renderer, Theme,
};

use crate::chamfer::chamfer_points;

/// Colours for one interaction state.
#[derive(Debug, Clone, Copy)]
pub struct ShapedColors {
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub text: Color,
    pub glow: Option<Color>,
}

/// Canvas program for a chamfered button. Build via `IcedTheme::chamfer_button`.
#[derive(Debug, Clone)]
pub struct ChamferButton<Message> {
    pub label: String,
    pub font: Font,
    pub size: f32,
    pub cut: f32,
    pub border_width: f32,
    pub idle: ShapedColors,
    pub hover: ShapedColors,
    pub press: ShapedColors,
    pub disabled: ShapedColors,
    pub on_press: Option<Message>,
    /// Backdrop under outline buttons (cyberpunk.net uses black @ 60%).
    pub backdrop: Color,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ButtonState {
    pressed: bool,
    hovered: bool,
}

impl<Message: Clone> canvas::Program<Message> for ChamferButton<Message> {
    type State = ButtonState;

    fn update(
        &self,
        state: &mut ButtonState,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        let over = cursor.is_over(bounds);
        let enabled = self.on_press.is_some();
        let hover_changed = over != state.hovered;
        state.hovered = over;
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if over && enabled => {
                state.pressed = true;
                Some(Action::request_redraw().and_capture())
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.pressed => {
                state.pressed = false;
                if over {
                    self.on_press
                        .clone()
                        .map(|m| Action::publish(m).and_capture())
                } else {
                    Some(Action::request_redraw())
                }
            }
            _ if hover_changed => Some(Action::request_redraw()),
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &ButtonState,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let over = cursor.is_over(bounds);
        let c = if self.on_press.is_none() {
            self.disabled
        } else if state.pressed && over {
            self.press
        } else if over {
            self.hover
        } else {
            self.idle
        };

        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let w = self.border_width;
        let pts = chamfer_points(bounds.width - w, bounds.height - w, self.cut);
        let i = w / 2.0;
        let path = Path::new(|b| {
            b.move_to(Point::new(pts[0].x + i, pts[0].y + i));
            for p in &pts[1..] {
                b.line_to(Point::new(p.x + i, p.y + i));
            }
            b.close();
        });
        if let Some(g) = c.glow {
            // Cheap neon glow: widening translucent strokes under the shape.
            for (k, a) in [(6.0, 0.10), (4.0, 0.16), (2.0, 0.24)] {
                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_color(Color { a: g.a * a, ..g })
                        .with_width(w + k),
                );
            }
        }
        frame.fill(&path, c.fill.unwrap_or(self.backdrop));
        if let Some(s) = c.stroke {
            frame.stroke(&path, Stroke::default().with_color(s).with_width(w));
        }
        frame.fill_text(Text {
            content: self.label.clone(),
            position: Point::new((bounds.width - self.cut / 2.0) / 2.0, bounds.height / 2.0),
            color: c.text,
            size: Pixels(self.size),
            font: self.font,
            align_x: Alignment::Center,
            align_y: alignment::Vertical::Center,
            ..Text::default()
        });
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &ButtonState,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.on_press.is_some() && cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message: Clone + 'a> From<ChamferButton<Message>> for Element<'a, Message> {
    fn from(b: ChamferButton<Message>) -> Self {
        // Rough text measure (no font metrics available outside layout): ~0.62em/char + padding.
        let w = b.label.chars().count() as f32 * b.size * 0.62 + b.size * 3.0 + b.cut;
        let h = b.size * 3.0;
        Canvas::new(b)
            .width(Length::Fixed(w.max(120.0)))
            .height(Length::Fixed(h))
            .into()
    }
}
