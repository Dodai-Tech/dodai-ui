//! Shaped frame (canvas) and the generic shaped box used by panels, tags, tabs…
//!
//! Rounded / square themes use a native `container` style; chamfered themes
//! stack a [`ShapedFrame`] canvas behind the content.

use dodai_core::{Color as LColor, Shape, ShapeKind};
use iced::{
    mouse,
    widget::{
        canvas::{self, Canvas, Geometry},
        container, Stack,
    },
    Background, Color, Element, Length, Padding, Rectangle, Renderer, Theme as IcedBase,
};

use crate::{
    color,
    geometry::{margin, paint_shape, Paint},
    IcedTheme,
};

/// Canvas program drawing a filled, stroked, optionally glowing shape per
/// [`Shape`]. Layer it behind content with `iced::widget::stack!`.
/// The shape is inset by [`ShapedFrame::margin`] so glow / shadow are not clipped.
#[derive(Debug, Clone)]
pub struct ShapedFrame {
    pub shape: Shape,
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub width: f32,
    pub glow: Option<Color>,
    pub glow_radius: f32,
    pub hard_shadow: f32,
}

impl ShapedFrame {
    /// Space reserved around the shape for glow / shadow.
    pub fn margin(&self) -> f32 {
        margin(
            if self.glow.is_some() {
                self.glow_radius
            } else {
                0.0
            },
            self.hard_shadow,
        )
    }
}

impl<Message> canvas::Program<Message> for ShapedFrame {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &IcedBase,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let m = self.margin();
        let rect = Rectangle {
            x: m,
            y: m,
            width: (bounds.width - 2.0 * m).max(0.0),
            height: (bounds.height - 2.0 * m).max(0.0),
        };
        paint_shape(
            &mut frame,
            &self.shape,
            rect,
            &Paint {
                fill: self.fill,
                stroke: self.stroke,
                width: self.width,
                glow: self.glow,
                glow_radius: self.glow_radius,
                shadow: self.hard_shadow,
            },
        );
        vec![frame.into_geometry()]
    }
}

/// Canvas element that fills its parent.
pub fn shaped_frame<'a, Message: 'a>(frame: ShapedFrame) -> Canvas<ShapedFrame, Message> {
    Canvas::new(frame).width(Length::Fill).height(Length::Fill)
}

/// Visual description of a shaped box (token colours, resolved by callers).
#[derive(Debug, Clone, Copy, Default)]
pub struct BoxStyle {
    pub fill: Option<LColor>,
    pub stroke: Option<LColor>,
    /// Draw the theme glow (no-op when the theme has none).
    pub glow: bool,
    /// Draw the theme's hard offset shadow (when `effects.hard_shadow > 0`).
    pub hard_shadow: bool,
    pub padding: Padding,
}

impl IcedTheme {
    /// Border radius for native widgets: the token radius for rounded themes,
    /// `0` for square and chamfered ones (chamfers need [`ShapedFrame`]).
    pub fn radius(&self) -> f32 {
        match self.theme.shape.kind() {
            ShapeKind::Rounded => self.theme.shape.radius,
            _ => 0.0,
        }
    }

    /// Native `container::Style` for a [`BoxStyle`] (rounded / square shapes).
    pub fn box_style(&self, s: &BoxStyle) -> container::Style {
        container::Style {
            text_color: None,
            background: s.fill.map(|c| Background::Color(color(c))),
            border: iced::Border {
                color: s.stroke.map_or(Color::TRANSPARENT, color),
                width: if s.stroke.is_some() {
                    self.theme.shape.border_width
                } else {
                    0.0
                },
                radius: self.radius().into(),
            },
            shadow: self.shadow(s.glow, s.hard_shadow),
            snap: true,
        }
    }

    /// Canvas frame equivalent of a [`BoxStyle`] (any shape).
    pub fn box_frame(&self, s: &BoxStyle) -> ShapedFrame {
        let e = &self.theme.effects;
        ShapedFrame {
            shape: self.theme.shape.clone(),
            fill: s.fill.map(color),
            stroke: s.stroke.map(color),
            width: self.theme.shape.border_width,
            glow: (s.glow && e.glow_enabled()).then(|| color(e.glow_color())),
            glow_radius: e.glow_radius(),
            hard_shadow: if s.hard_shadow { e.hard_shadow } else { 0.0 },
        }
    }

    /// Wrap `content` in a box shaped by the theme (chamfer / rounded / square).
    pub fn shaped_box<'a, Message: 'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        s: BoxStyle,
    ) -> Element<'a, Message> {
        if self.theme.shape.kind() == ShapeKind::Chamfered {
            let frame = self.box_frame(&s);
            let m = frame.margin();
            let pad = Padding {
                top: s.padding.top + m,
                right: s.padding.right + m,
                bottom: s.padding.bottom + m,
                left: s.padding.left + m,
            };
            // The content is the base layer: it sizes the stack, and the frame (which fills its
            // parent) follows. With the frame as the base, the box collapsed to nothing wherever
            // the height is unbounded (a scrollable page).
            Stack::new()
                .push(container(content).padding(pad))
                .push_under(shaped_frame(frame))
                .into()
        } else {
            let style = self.box_style(&s);
            container(content)
                .padding(s.padding)
                .style(move |_| style)
                .into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;
    use dodai_core::Theme;

    fn lt(t: Theme) -> IcedTheme {
        IcedTheme::new(t)
    }

    #[test]
    fn box_style_radius_and_border_from_tokens() {
        let s = BoxStyle {
            fill: Some(LColor::hex(0xffffff)),
            stroke: Some(LColor::hex(0)),
            ..BoxStyle::default()
        };
        let r = lt(rounded_light()).box_style(&s);
        assert_eq!(r.border.radius, 8.0.into());
        assert_eq!(r.border.width, 1.0);
        let q = lt(hard_square()).box_style(&s);
        assert_eq!(q.border.radius, 0.0.into());
        assert_eq!(q.border.width, 2.0);
        let c = lt(cyberpunk()).box_style(&s);
        assert_eq!(c.border.radius, 0.0.into());
    }

    #[test]
    fn hard_shadow_only_when_token_set() {
        let s = BoxStyle {
            hard_shadow: true,
            ..BoxStyle::default()
        };
        assert_eq!(lt(rounded_light()).box_style(&s).shadow.offset.x, 2.0);
        assert_eq!(lt(hard_square()).box_style(&s).shadow.blur_radius, 0.0);
        assert_eq!(lt(hard_square()).box_style(&s).shadow.offset.x, 0.0);
    }

    #[test]
    fn glow_follows_effects_and_intensity() {
        let s = BoxStyle {
            glow: true,
            ..BoxStyle::default()
        };
        assert!(lt(cyberpunk()).box_style(&s).shadow.blur_radius > 0.0);
        assert_eq!(
            lt(cyberpunk().with_glow_intensity(0.0))
                .box_style(&s)
                .shadow
                .blur_radius,
            0.0
        );
        assert_eq!(lt(rounded_light()).box_style(&s).shadow.blur_radius, 0.0);
        assert!(lt(cyberpunk()).box_frame(&s).glow.is_some());
        assert!(lt(rounded_light()).box_frame(&s).glow.is_none());
    }

    #[test]
    fn frame_margin_reserves_glow() {
        let s = BoxStyle {
            glow: true,
            ..BoxStyle::default()
        };
        let f = lt(cyberpunk()).box_frame(&s);
        assert_eq!(f.margin(), f.glow_radius.ceil());
        let plain = lt(cyberpunk()).box_frame(&BoxStyle::default());
        assert_eq!(plain.margin(), 0.0);
    }
}
