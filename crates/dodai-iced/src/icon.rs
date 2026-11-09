//! Icons: the shared vocabulary of `dodai-icons` drawn on a canvas in the design's own style.

use dodai_core::Color as LColor;
use dodai_icons::IconId;
use iced::{
    mouse,
    widget::canvas::{self, fill, Canvas, Fill, Geometry, Path},
    Element, Point, Rectangle, Renderer, Theme as IcedBase,
};

use crate::{color, IcedTheme};

/// Side of the grid every glyph is drawn on.
const GRID: f32 = 24.0;

struct IconCanvas {
    polygons: Vec<Vec<(f32, f32)>>,
    ink: iced::Color,
}

impl<Message> canvas::Program<Message> for IconCanvas {
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
        let k = bounds.width.min(bounds.height) / GRID;
        let path = Path::new(|b| {
            for poly in &self.polygons {
                let mut pts = poly.iter().map(|&(x, y)| Point::new(x * k, y * k));
                if let Some(first) = pts.next() {
                    b.move_to(first);
                    pts.for_each(|p| b.line_to(p));
                    b.close();
                }
            }
        });
        frame.fill(
            &path,
            Fill {
                style: canvas::Style::Solid(self.ink),
                rule: fill::Rule::NonZero,
            },
        );
        vec![frame.into_geometry()]
    }
}

impl IcedTheme {
    /// The polygons of `id` in this design's icon style (empty when the design has no such
    /// icon).
    pub fn icon_polygons(&self, id: IconId, filled: bool) -> Vec<Vec<(f32, f32)>> {
        dodai_icons::glyph_for_theme(id, &self.theme, filled)
            .map(|g| g.polygons())
            .unwrap_or_default()
    }

    /// `id` at `size` px in `ink` (the text colour when `None`).
    pub fn icon<'a, Message: 'a>(
        &self,
        id: IconId,
        size: f32,
        ink: Option<LColor>,
    ) -> Element<'a, Message> {
        self.icon_styled(id, size, ink, false)
    }

    /// A drawing of `dodai-icons` (a catalogue icon or a symbol) at `size` px in `ink`; nothing
    /// for `None`.
    pub fn glyph<'a, Message: 'a>(
        &self,
        glyph: Option<dodai_icons::Glyph>,
        size: f32,
        ink: LColor,
    ) -> Element<'a, Message> {
        Canvas::new(IconCanvas {
            polygons: glyph.map(|g| g.polygons()).unwrap_or_default(),
            ink: color(ink),
        })
        .width(size)
        .height(size)
        .into()
    }

    /// Like [`icon`](Self::icon), in the filled form where the design has one.
    pub fn icon_styled<'a, Message: 'a>(
        &self,
        id: IconId,
        size: f32,
        ink: Option<LColor>,
        filled: bool,
    ) -> Element<'a, Message> {
        let ink = color(ink.unwrap_or(self.theme.palette.text));
        Canvas::new(IconCanvas {
            polygons: self.icon_polygons(id, filled),
            ink,
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
    fn every_design_draws_the_common_icons_inside_the_grid() {
        let themes = [cyberpunk(), rounded_light(), hard_square()]
            .into_iter()
            .chain(dodai_core::themes::all());
        for t in themes {
            let lt = IcedTheme::new(t);
            for id in [IconId::Home, IconId::Search, IconId::Check, IconId::Star] {
                let polys = lt.icon_polygons(id, false);
                assert!(!polys.is_empty(), "{} {id:?}", lt.theme().id());
                for (x, y) in polys.iter().flatten() {
                    assert!(
                        (-1.0..=GRID + 1.0).contains(x) && (-1.0..=GRID + 1.0).contains(y),
                        "{id:?} off the grid at {x},{y}"
                    );
                }
                assert!(
                    !lt.icon_polygons(id, true).is_empty(),
                    "{} {id:?} filled",
                    lt.theme().id()
                );
            }
        }
    }
}
