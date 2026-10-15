//! Cut-corner frame (bottom-right chamfer only, as on cyberpunk.net buttons).

use iced::{
    mouse,
    widget::canvas::{self, Canvas, Geometry, Path, Stroke},
    Color, Length, Point, Rectangle, Renderer, Theme,
};

/// Polygon points of a chamfered rectangle: only the bottom-right corner cut.
pub fn chamfer_points(w: f32, h: f32, cut: f32) -> [Point; 5] {
    let c = cut.min(w / 2.0).min(h / 2.0).max(0.0);
    [
        Point::new(0.0, 0.0),
        Point::new(w, 0.0),
        Point::new(w, h - c),
        Point::new(w - c, h),
        Point::new(0.0, h),
    ]
}

/// Canvas program drawing a filled, stroked chamfered rectangle.
/// Layer it behind content with `iced::widget::stack!`.
#[derive(Debug, Clone, Copy)]
pub struct ChamferFrame {
    pub fill: Color,
    pub stroke: Color,
    pub width: f32,
    pub cut: f32,
}

impl<Message> canvas::Program<Message> for ChamferFrame {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        // Inset by half the stroke so the line isn't clipped at the edges.
        let i = self.width / 2.0;
        let pts = chamfer_points(
            bounds.width - self.width,
            bounds.height - self.width,
            self.cut,
        );
        let path = Path::new(|b| {
            b.move_to(Point::new(pts[0].x + i, pts[0].y + i));
            for p in &pts[1..] {
                b.line_to(Point::new(p.x + i, p.y + i));
            }
            b.close();
        });
        frame.fill(&path, self.fill);
        frame.stroke(
            &path,
            Stroke::default()
                .with_color(self.stroke)
                .with_width(self.width),
        );
        vec![frame.into_geometry()]
    }
}

/// Ready-to-use canvas element that fills its parent.
pub fn chamfer_frame<'a, Message: 'a>(frame: ChamferFrame) -> Canvas<ChamferFrame, Message> {
    Canvas::new(frame).width(Length::Fill).height(Length::Fill)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_cut_bottom_right_only() {
        let p = chamfer_points(100.0, 50.0, 10.0);
        assert_eq!(p[0], Point::new(0.0, 0.0));
        assert_eq!(p[2], Point::new(100.0, 40.0));
        assert_eq!(p[3], Point::new(90.0, 50.0));
    }

    #[test]
    fn cut_is_clamped() {
        let p = chamfer_points(10.0, 10.0, 99.0);
        assert_eq!(p[3], Point::new(5.0, 10.0));
    }
}
