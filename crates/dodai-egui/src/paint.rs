//! Token-driven geometry: outlines, glow layers and hard shadows as egui shapes.

use dodai_core::{Color, ControlState, Shape as ShapeTokens, ShapeKind, Theme};
use egui::{epaint::PathShape, pos2, vec2, Color32, CornerRadius, Rect, Response, Sense, Stroke};
use egui::{epaint::RectShape, Pos2, Shape, StrokeKind};

/// Token colour → egui colour (unmultiplied alpha input).
pub fn color32(c: Color) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r, c.g, c.b, c.a)
}

/// Outline polygon of `shape` placed at `rect` (from [`ShapeTokens::outline`]).
pub fn outline_points(shape: &ShapeTokens, rect: Rect) -> Vec<Pos2> {
    shape
        .outline(rect.width(), rect.height())
        .into_iter()
        .map(|(x, y)| pos2(rect.min.x + x, rect.min.y + y))
        .collect()
}

/// Corner radius for rounded shapes (clamped to half the shorter side), else zero.
pub fn corner_radius(shape: &ShapeTokens, rect: Rect) -> CornerRadius {
    if shape.kind() == ShapeKind::Rounded {
        let r = shape.radius.min(rect.width().min(rect.height()) / 2.0);
        CornerRadius::same(r.round().clamp(0.0, 255.0) as u8)
    } else {
        CornerRadius::ZERO
    }
}

/// One filled+stroked outline: polygon when chamfered, rect otherwise.
pub fn outline_shape(shape: &ShapeTokens, rect: Rect, fill: Color32, stroke: Stroke) -> Shape {
    match shape.kind() {
        ShapeKind::Chamfered if shape.scoop => {
            scooped(outline_points(shape, rect), rect, fill, stroke)
        }
        ShapeKind::Chamfered => {
            PathShape::convex_polygon(outline_points(shape, rect), fill, stroke).into()
        }
        _ => RectShape::new(
            rect,
            corner_radius(shape, rect),
            fill,
            stroke,
            StrokeKind::Inside,
        )
        .into(),
    }
}

/// A scooped outline is concave, which egui's convex fill cannot take: the body is a triangle
/// fan from the centre (every point of a scooped outline is in view of it) and the edge a closed
/// line on top.
fn scooped(points: Vec<Pos2>, rect: Rect, fill: Color32, stroke: Stroke) -> Shape {
    let mut parts = Vec::with_capacity(2);
    if fill.a() > 0 && points.len() > 2 {
        let mut mesh = egui::epaint::Mesh::default();
        mesh.colored_vertex(rect.center(), fill);
        for p in &points {
            mesh.colored_vertex(*p, fill);
        }
        let n = points.len() as u32;
        for i in 0..n {
            mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
        }
        parts.push(Shape::mesh(mesh));
    }
    if stroke.width > 0.0 && stroke.color.a() > 0 {
        parts.push(PathShape::closed_line(points, stroke).into());
    }
    Shape::Vec(parts)
}

/// Soft glow: stacked, fading outlines around `rect`. Empty when the theme's
/// glow is disabled (see `Effects::glow_enabled`, intensity via
/// `Theme::with_glow_intensity`).
pub fn glow_shapes(theme: &Theme, rect: Rect) -> Vec<Shape> {
    if !theme.effects.glow_enabled() {
        return Vec::new();
    }
    const LAYERS: usize = 5;
    let r = theme.effects.glow_radius();
    let c = theme.effects.glow_color();
    (1..=LAYERS)
        .rev()
        .map(|i| {
            let k = i as f32 / LAYERS as f32;
            let fall = (1.0 - (k - 1.0 / LAYERS as f32)).powi(2);
            let a = (c.a as f32 * fall * 0.5).round().clamp(0.0, 255.0) as u8;
            let col = Color32::from_rgba_unmultiplied(c.r, c.g, c.b, a);
            outline_shape(
                &theme.shape,
                rect.expand(r * k),
                Color32::TRANSPARENT,
                Stroke::new(r / LAYERS as f32 + 0.5, col),
            )
        })
        .collect()
}

/// Full box: glow (optional), hard shadow (optional, when the theme has one),
/// then the filled/bordered body.
pub fn box_shapes(
    theme: &Theme,
    rect: Rect,
    fill: Option<Color>,
    border: Option<Color>,
    glow: bool,
    shadow: bool,
) -> Vec<Shape> {
    box_shapes_for(theme, rect, fill, border, glow, shadow, None)
}

/// [`box_shapes`] for a surface of `class`: the app-wide texture and effects of the design
/// ([`crate::set_surface`]) are painted over the body when the class is targeted.
pub fn box_shapes_for(
    theme: &Theme,
    rect: Rect,
    fill: Option<Color>,
    border: Option<Color>,
    glow: bool,
    shadow: bool,
    class: Option<dodai_core::SurfaceClass>,
) -> Vec<Shape> {
    let tune = crate::shape_tune();
    let p = &theme.palette;
    let tint = match tune.tint {
        dodai_core::BorderTint::Design => None,
        dodai_core::BorderTint::Strong => Some(p.border_strong),
        dodai_core::BorderTint::Accent => Some(p.primary),
        dodai_core::BorderTint::Text => Some(p.text),
    };
    let border = match (border, tune.outline_all) {
        (Some(c), _) => Some(tint.unwrap_or(c)),
        (None, true) if fill.is_some() => Some(tint.unwrap_or(p.border)),
        (None, _) => None,
    };
    let bw = if border.is_some() && tune.outline_all && tune.border_width.is_none() {
        theme.shape.border_width.max(1.0)
    } else {
        theme.shape.border_width
    };
    let mut out = Vec::new();
    if glow {
        out.extend(glow_shapes(theme, rect));
    }
    let s = theme.effects.hard_shadow;
    if shadow && s > 0.0 {
        let (dx, dy) = tune.shadow_dir.dir();
        out.push(outline_shape(
            &theme.shape,
            rect.translate(vec2(s * dx, s * dy)),
            color32(theme.palette.border),
            Stroke::NONE,
        ));
    }
    let stroke = border.map_or(Stroke::NONE, |c| Stroke::new(bw, color32(c)));
    // The glass the surface gives this class: a see-through fill over the frosted backdrop.
    let glass = class.map(|c| crate::glass::glass_of(theme, c));
    let fill = match (&glass, fill) {
        (Some(g), Some(f)) => Some(g.color(f)),
        (_, f) => f,
    };
    if let Some(g) = &glass {
        out.extend(crate::glass::frost_shapes(g, rect));
    }
    out.push(outline_shape(
        &theme.shape,
        rect,
        fill.map_or(Color32::TRANSPARENT, color32),
        stroke,
    ));
    if let Some(class) = class {
        out.extend(crate::surface::surface_shapes(theme, rect, class, fill));
    }
    out
}

/// Interaction state of a response.
pub fn control_state(resp: &Response, enabled: bool) -> ControlState {
    if !enabled {
        ControlState::Disabled
    } else if resp.is_pointer_button_down_on() {
        ControlState::Pressed
    } else if resp.has_focus() {
        ControlState::Focused
    } else if resp.hovered() {
        ControlState::Hovered
    } else {
        ControlState::Idle
    }
}

/// `Sense` for an enabled/disabled clickable control.
pub fn sense(enabled: bool) -> Sense {
    if enabled {
        Sense::click()
    } else {
        Sense::hover()
    }
}

/// Lay out single-line text (colour decided at paint time).
pub fn layout(ui: &egui::Ui, text: String, size: f32) -> std::sync::Arc<egui::Galley> {
    ui.painter()
        .layout_no_wrap(text, egui::FontId::proportional(size), Color32::PLACEHOLDER)
}

/// Paint `shapes` on `ui`.
pub fn add_all(ui: &egui::Ui, shapes: Vec<Shape>) {
    ui.painter().extend(shapes);
    if crate::surface::take_animated() {
        ui.ctx().request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{chamfered, rounded, square};

    fn rect() -> Rect {
        Rect::from_min_size(pos2(10.0, 20.0), vec2(100.0, 40.0))
    }

    #[test]
    fn chamfered_is_polygon_with_core_outline() {
        let t = chamfered();
        match outline_shape(&t.shape, rect(), Color32::RED, Stroke::NONE) {
            Shape::Path(p) => {
                assert_eq!(p.points.len(), t.shape.outline(100.0, 40.0).len());
                assert!(p.points.len() > 4);
                assert!(p.points.iter().all(|q| rect().expand(0.01).contains(*q)));
            }
            other => panic!("expected path, got {other:?}"),
        }
    }

    #[test]
    fn scooped_fills_a_fan_from_the_centre_and_strokes_the_outline() {
        let mut t = chamfered();
        t.shape.scoop = true;
        let n = t.shape.outline(100.0, 40.0).len();
        let stroke = Stroke::new(1.0, Color32::WHITE);
        let Shape::Vec(parts) = outline_shape(&t.shape, rect(), Color32::RED, stroke) else {
            panic!("a scooped outline is a fill and a line");
        };
        let [Shape::Mesh(m), Shape::Path(p)] = parts.as_slice() else {
            panic!("expected a mesh then a path, got {parts:?}");
        };
        assert_eq!(m.indices.len(), 3 * n, "one triangle per outline edge");
        assert!(p.closed && p.points.len() == n);
        // Every vertex stays inside the box.
        assert!(m
            .vertices
            .iter()
            .all(|v| rect().expand(0.01).contains(v.pos)));
    }

    #[test]
    fn rounded_and_square_are_rects() {
        match outline_shape(&rounded().shape, rect(), Color32::RED, Stroke::NONE) {
            Shape::Rect(r) => assert!(r.corner_radius.nw > 0),
            o => panic!("{o:?}"),
        }
        match outline_shape(&square().shape, rect(), Color32::RED, Stroke::NONE) {
            Shape::Rect(r) => assert_eq!(r.corner_radius, CornerRadius::ZERO),
            o => panic!("{o:?}"),
        }
    }

    #[test]
    fn glow_follows_effects() {
        let c = chamfered();
        assert!(c.effects.glow_enabled());
        assert!(!glow_shapes(&c, rect()).is_empty());
        assert!(glow_shapes(&c.clone().with_glow_intensity(0.0), rect()).is_empty());
        assert!(glow_shapes(&rounded(), rect()).is_empty());
    }

    #[test]
    fn stronger_glow_reaches_further() {
        let reach = |t: &Theme| {
            glow_shapes(t, rect())
                .iter()
                .map(|s| s.visual_bounding_rect().width())
                .fold(0.0, f32::max)
        };
        let c = chamfered();
        assert!(reach(&c.clone().with_glow_intensity(2.0)) > reach(&c));
    }

    #[test]
    fn hard_shadow_only_when_token_set() {
        let mut t = square();
        t.effects.hard_shadow = 0.0;
        assert_eq!(box_shapes(&t, rect(), None, None, false, true).len(), 1);
        t.effects.hard_shadow = 3.0;
        assert_eq!(box_shapes(&t, rect(), None, None, false, true).len(), 2);
    }
}
