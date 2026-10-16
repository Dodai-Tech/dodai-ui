//! Shape geometry shared by every canvas-drawn widget.
//!
//! Everything is derived from [`legendary_core::Shape`]; there is no
//! theme-id branching. [`ShapeKind::Chamfered`] uses `Shape::outline`,
//! [`ShapeKind::Rounded`] a rounded rectangle, [`ShapeKind::Square`] a rectangle.

use iced::{
    widget::canvas::{self, Path, Stroke},
    Color, Point, Rectangle, Size,
};
use legendary_core::{Shape, ShapeKind};

/// Resolved outline for a box of a given size.
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeGeom {
    /// Plain rectangle.
    Rect,
    /// Rounded rectangle with the (clamped) radius.
    Rounded(f32),
    /// Chamfered polygon, points relative to the box origin.
    Polygon(Vec<(f32, f32)>),
}

impl ShapeGeom {
    /// Outline of `shape` for a `w`×`h` box.
    pub fn of(shape: &Shape, w: f32, h: f32) -> Self {
        match shape.kind() {
            ShapeKind::Square => Self::Rect,
            ShapeKind::Rounded => Self::Rounded(shape.radius.min(w.min(h) / 2.0).max(0.0)),
            ShapeKind::Chamfered => Self::Polygon(shape.outline(w, h)),
        }
    }

    /// Canvas path of this outline for a `w`×`h` box whose top-left is `origin`.
    pub fn path(&self, origin: Point, w: f32, h: f32) -> Path {
        match self {
            Self::Rect => Path::rectangle(origin, Size::new(w, h)),
            Self::Rounded(r) => Path::rounded_rectangle(origin, Size::new(w, h), (*r).into()),
            Self::Polygon(pts) => Path::new(|b| {
                for (i, (x, y)) in pts.iter().enumerate() {
                    let p = Point::new(origin.x + x, origin.y + y);
                    if i == 0 {
                        b.move_to(p);
                    } else {
                        b.line_to(p);
                    }
                }
                b.close();
            }),
        }
    }
}

/// Paint description for [`paint_shape`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct Paint {
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub width: f32,
    pub glow: Option<Color>,
    pub glow_radius: f32,
    /// Hard offset shadow in px (0 = none).
    pub shadow: f32,
}

/// Extra space needed around a shape so glow / shadow are not clipped.
pub(crate) fn margin(glow_radius: f32, shadow: f32) -> f32 {
    glow_radius.max(shadow).max(0.0).ceil()
}

/// Draw shadow, glow, fill and border for `shape` inside `rect` (the shape's box).
pub(crate) fn paint_shape(frame: &mut canvas::Frame, shape: &Shape, rect: Rectangle, p: &Paint) {
    let w = p.width;
    let geom = ShapeGeom::of(shape, rect.width - w, rect.height - w);
    let o = Point::new(rect.x + w / 2.0, rect.y + w / 2.0);
    let (iw, ih) = (rect.width - w, rect.height - w);
    let path = geom.path(o, iw, ih);
    if p.shadow > 0.0 {
        let so = Point::new(o.x + p.shadow, o.y + p.shadow);
        frame.fill(&geom.path(so, iw, ih), Color::from_rgba(0.0, 0.0, 0.0, 0.5));
    }
    if let Some(g) = p.glow {
        // Soft glow: concentric translucent strokes fading outward.
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
                    .with_width(w + 2.0 * p.glow_radius * t)
                    .with_line_join(canvas::LineJoin::Round),
            );
        }
    }
    if let Some(f) = p.fill {
        frame.fill(&path, f);
    }
    if let Some(s) = p.stroke {
        frame.stroke(&path, Stroke::default().with_color(s).with_width(w));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn geometry_follows_shape_kind() {
        let cp = cyberpunk();
        assert!(matches!(
            ShapeGeom::of(&cp.shape, 100.0, 40.0),
            ShapeGeom::Polygon(_)
        ));
        assert!(matches!(
            ShapeGeom::of(&rounded_light().shape, 100.0, 40.0),
            ShapeGeom::Rounded(r) if r == 8.0
        ));
        assert_eq!(
            ShapeGeom::of(&hard_square().shape, 100.0, 40.0),
            ShapeGeom::Rect
        );
    }

    #[test]
    fn polygon_is_core_outline() {
        let cp = cyberpunk();
        let ShapeGeom::Polygon(p) = ShapeGeom::of(&cp.shape, 100.0, 50.0) else {
            panic!("expected polygon");
        };
        assert_eq!(p, cp.shape.outline(100.0, 50.0));
        assert!(p.len() > 4);
    }

    #[test]
    fn rounded_radius_is_clamped() {
        let s = rounded_light().shape;
        assert_eq!(ShapeGeom::of(&s, 100.0, 10.0), ShapeGeom::Rounded(5.0));
    }

    #[test]
    fn margin_covers_glow_and_shadow() {
        assert_eq!(margin(9.2, 0.0), 10.0);
        assert_eq!(margin(0.0, 4.0), 4.0);
    }
}
