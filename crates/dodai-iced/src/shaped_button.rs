//! Shaped button: draws chamfered / rounded / square per [`Shape`] tokens.
//!
//! `iced::widget::button` can only draw rounded rectangles, so this is a small
//! canvas widget that paints the outline, label and hover/press states itself.
//! Prefer `IcedTheme::button_view`, which picks the native button when the
//! shape allows and this widget only for chamfered themes.

use dodai_core::Shape;
use iced::{
    alignment, mouse,
    widget::{
        canvas::{self, Action, Canvas, Geometry, Text},
        text::Alignment,
    },
    Color, Element, Event, Font, Length, Pixels, Point, Rectangle, Renderer, Theme,
};

use crate::geometry::{margin, paint_shape, Paint};

/// Colours for one interaction state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapedColors {
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub text: Color,
    pub glow: Option<Color>,
}

/// Canvas program for a shaped button. Build via `IcedTheme::shaped_button`.
#[derive(Debug, Clone)]
pub struct ShapedButton<Message> {
    pub label: String,
    pub font: Font,
    pub size: f32,
    pub shape: Shape,
    pub border_width: f32,
    pub idle: ShapedColors,
    pub hover: ShapedColors,
    pub press: ShapedColors,
    pub disabled: ShapedColors,
    /// Glow radius in px (already scaled by the theme's glow intensity).
    pub glow_radius: f32,
    pub on_press: Option<Message>,
    /// Backdrop under outline buttons (black @ 60%).
    pub backdrop: Color,
    /// Control height in px (excluding glow margin).
    pub height: f32,
    /// Horizontal padding in px each side of the label.
    pub padding_x: f32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ButtonState {
    pressed: bool,
    hovered: bool,
}

impl<Message> ShapedButton<Message> {
    /// Space reserved around the shape so the glow isn't clipped.
    pub fn margin(&self) -> f32 {
        margin(self.glow_radius, 0.0)
    }

    /// Size of the widget including glow margin: (width, height).
    pub fn outer_size(&self) -> (f32, f32) {
        let m = self.margin();
        // Rough text measure (no font metrics outside layout): ~0.62em/char.
        let w = self.label.chars().count() as f32 * self.size * 0.62 + 2.0 * self.padding_x;
        (w.max(self.height * 2.5) + 2.0 * m, self.height + 2.0 * m)
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

impl<Message: Clone> canvas::Program<Message> for ShapedButton<Message> {
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
        let m = self.margin();
        let local = Rectangle {
            x: m,
            y: m,
            width: rect.width,
            height: rect.height,
        };
        paint_shape(
            &mut frame,
            &self.shape,
            local,
            &Paint {
                fill: Some(c.fill.unwrap_or(self.backdrop)),
                stroke: c.stroke,
                width: self.border_width,
                glow: c.glow,
                glow_radius: self.glow_radius,
                shadow: 0.0,
            },
        );
        frame.fill_text(Text {
            content: self.label.clone(),
            position: Point::new(m + rect.width / 2.0, m + rect.height / 2.0),
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

impl<'a, Message: Clone + 'a> From<ShapedButton<Message>> for Element<'a, Message> {
    fn from(b: ShapedButton<Message>) -> Self {
        let (w, h) = b.outer_size();
        Canvas::new(b)
            .width(Length::Fixed(w))
            .height(Length::Fixed(h))
            .into()
    }
}
