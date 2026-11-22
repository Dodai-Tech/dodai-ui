//! Surface textures and effects, drawn by `dodai-ui-fx` through an iced canvas.

use dodai_ui_core::{ResolvedSurface, Surface, SurfaceClass};
use dodai_ui_fx::{Frame as FxFrame, Painter, Rgba, SurfacePaint};
use iced::{
    mouse,
    widget::canvas::{self, Canvas, Geometry, Path, Stroke},
    Color, Element, Length, Point, Rectangle, Renderer, Size, Theme as IcedBaseTheme,
};

use crate::IcedTheme;

fn color(c: Rgba) -> Color {
    Color::from_rgba(c.r, c.g, c.b, c.a)
}

/// The four `Painter` primitives on an iced canvas frame.
struct IcedPainter<'a> {
    frame: &'a mut canvas::Frame,
}

impl Painter for IcedPainter<'_> {
    fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba) {
        self.frame
            .fill_rectangle(Point::new(x, y), Size::new(w, h), color(c));
    }

    fn dot(&mut self, cx: f32, cy: f32, d: f32, c: Rgba) {
        self.frame
            .fill(&Path::circle(Point::new(cx, cy), d / 2.0), color(c));
    }

    fn stroke(&mut self, pts: &[(f32, f32)], closed: bool, width: f32, c: Rgba) {
        let path = Path::new(|b| {
            let mut it = pts.iter();
            if let Some(&(x, y)) = it.next() {
                b.move_to(Point::new(x, y));
                for &(x, y) in it {
                    b.line_to(Point::new(x, y));
                }
                if closed {
                    b.close();
                }
            }
        });
        self.frame.stroke(
            &path,
            Stroke::default().with_width(width).with_color(color(c)),
        );
    }

    fn fill(&mut self, pts: &[(f32, f32)], c: Rgba) {
        let path = Path::new(|b| {
            let mut it = pts.iter();
            if let Some(&(x, y)) = it.next() {
                b.move_to(Point::new(x, y));
                for &(x, y) in it {
                    b.line_to(Point::new(x, y));
                }
                b.close();
            }
        });
        self.frame.fill(&path, color(c));
    }
}

/// Canvas program that paints one resolved surface over its bounds.
#[derive(Debug, Clone)]
pub struct SurfaceFxProgram {
    paint: SurfacePaint,
    shape: dodai_ui_core::Shape,
}

impl<Message> canvas::Program<Message> for SurfaceFxProgram {
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
        let f = FxFrame {
            w: bounds.width,
            h: bounds.height,
            shape: &self.shape.clip(),
            seed: 1,
            time: 0.0,
            presses: &[],
        };
        dodai_ui_fx::paint(&mut IcedPainter { frame: &mut frame }, &f, &self.paint);
        vec![frame.into_geometry()]
    }
}

impl IcedTheme {
    /// The canvas program for `fx` on a panel, or `None` when this design draws nothing of it.
    pub fn surface_fx_program(&self, fx: ResolvedSurface) -> Option<SurfaceFxProgram> {
        let paint =
            dodai_ui_core::surface_paint(&self.theme, SurfaceClass::Panel, &Surface::INHERIT, fx)?;
        Some(SurfaceFxProgram {
            paint,
            shape: self.theme.shape.clone(),
        })
    }

    /// Full-size overlay for `fx` (put it last in a `stack!`); empty when nothing is drawn.
    /// Static only: no click ripples or animation yet.
    pub fn surface_fx<'a, Message: 'a>(&self, fx: ResolvedSurface) -> Element<'a, Message> {
        match self.surface_fx_program(fx) {
            Some(p) => Canvas::new(p)
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            None => iced::widget::Space::new().into(),
        }
    }
}
