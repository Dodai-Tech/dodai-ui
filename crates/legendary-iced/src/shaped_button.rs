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
    /// Glow radius in px (already scaled by the theme's glow intensity).
    pub glow_radius: f32,
    pub on_press: Option<Message>,
    /// Backdrop under outline buttons (cyberpunk.net uses black @ 60%).
    pub backdrop: Color,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ButtonState {
    pressed: bool,
    hovered: bool,
}

impl<Message> ChamferButton<Message> {
    fn margin(&self) -> f32 {
        self.glow_radius.max(0.0).ceil()
    }

    /// The clickable shape inside the glow margin, in the same space as `bounds`.
    fn shape_rect(&self, bounds: Rectangle) -> Rectangle {
        let m = self.margin();
        Rectangle {
            x: bounds.x + m,
            y: bounds.y + m,
            width: (bounds.width - 2.0 * m).max(0.0),
            height: (bounds.height - 2.0 * m).max(0.0),
        }
    }
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
        let over = cursor.is_over(self.shape_rect(bounds));
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
        let rect = self.shape_rect(bounds);
        let over = cursor.is_over(rect);
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
        let m = self.margin();
        let pts = chamfer_points(rect.width - w, rect.height - w, self.cut);
        let o = w / 2.0 + m;
        let path = Path::new(|b| {
            b.move_to(Point::new(pts[0].x + o, pts[0].y + o));
            for p in &pts[1..] {
                b.line_to(Point::new(p.x + o, p.y + o));
            }
            b.close();
        });
        if let Some(g) = c.glow {
            // Soft neon glow: concentric translucent strokes fading outward.
            const STEPS: u32 = 10;
            for k in (1..=STEPS).rev() {
                let t = k as f32 / STEPS as f32; // 1 = outermost
                let fade = (1.0 - t).powi(2) * 0.35 + 0.02;
                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_color(Color {
                            a: (g.a * fade).min(1.0),
                            ..g
                        })
                        .with_width(w + 2.0 * self.glow_radius * t)
                        .with_line_join(canvas::LineJoin::Round),
                );
            }
        }
        frame.fill(&path, c.fill.unwrap_or(self.backdrop));
        if let Some(s) = c.stroke {
            frame.stroke(&path, Stroke::default().with_color(s).with_width(w));
        }
        frame.fill_text(Text {
            content: self.label.clone(),
            position: Point::new(
                m + (rect.width - self.cut / 2.0) / 2.0,
                m + rect.height / 2.0,
            ),
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
        if self.on_press.is_some() && cursor.is_over(self.shape_rect(bounds)) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message: Clone + 'a> From<ChamferButton<Message>> for Element<'a, Message> {
    fn from(b: ChamferButton<Message>) -> Self {
        // Rough text measure (no font metrics available outside layout): ~0.62em/char + padding.
        let m = b.margin();
        let w = (b.label.chars().count() as f32 * b.size * 0.62 + b.size * 3.0 + b.cut).max(120.0);
        let h = b.size * 3.0;
        Canvas::new(b)
            .width(Length::Fixed(w + 2.0 * m))
            .height(Length::Fixed(h + 2.0 * m))
            .into()
    }
}
