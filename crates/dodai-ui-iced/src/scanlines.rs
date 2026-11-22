//! CRT scanline overlay (when `effects.scanlines`).

use iced::{
    mouse,
    widget::{
        canvas::{self, Canvas, Geometry, Path},
        Space,
    },
    Color, Element, Length, Point, Rectangle, Renderer, Size, Theme as IcedBaseTheme,
};

use crate::IcedTheme;

/// Canvas program drawing 1px dark lines every `gap` px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scanlines {
    pub color: Color,
    pub gap: f32,
}

impl Scanlines {
    /// Y offsets of the lines for a surface `height` px tall.
    pub fn line_ys(&self, height: f32) -> Vec<f32> {
        let gap = self.gap.max(2.0);
        let mut ys = Vec::new();
        let mut y = gap - 1.0;
        while y < height {
            ys.push(y);
            y += gap;
        }
        ys
    }
}

impl<Message> canvas::Program<Message> for Scanlines {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &IcedBaseTheme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        for y in self.line_ys(bounds.height) {
            frame.fill(
                &Path::rectangle(Point::new(0.0, y), Size::new(bounds.width, 1.0)),
                self.color,
            );
        }
        vec![frame.into_geometry()]
    }
}

impl IcedTheme {
    /// Scanline program for this theme, or `None` when `effects.scanlines` is off.
    pub fn scanlines_program(&self) -> Option<Scanlines> {
        self.theme.effects.scanlines.then(|| Scanlines {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.18),
            gap: 3.0,
        })
    }

    /// Full-size overlay: put last in a `stack!`. Renders nothing (zero-size
    /// space) when the theme has no scanlines. Ignores mouse input.
    pub fn scanlines<'a, Message: 'a>(&self) -> Element<'a, Message> {
        match self.scanlines_program() {
            Some(p) => Canvas::new(p)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            None => Space::new().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn follows_effect_flag() {
        let mut t = cyberpunk();
        t.effects.scanlines = true;
        assert!(IcedTheme::new(t).scanlines_program().is_some());
        assert!(IcedTheme::new(rounded_light())
            .scanlines_program()
            .is_none());
    }

    #[test]
    fn line_positions() {
        let s = Scanlines {
            color: Color::BLACK,
            gap: 3.0,
        };
        assert_eq!(s.line_ys(10.0), vec![2.0, 5.0, 8.0]);
    }
}
