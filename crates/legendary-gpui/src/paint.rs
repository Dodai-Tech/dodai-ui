//! Shaped painting: turns [`Shape`] tokens into gpui boxes.
//!
//! * `Square` / `Rounded` themes use plain `div` styling (border, radius, box shadow).
//! * `Chamfered` themes paint the [`Shape::outline`] polygon on an absolutely
//!   positioned [`canvas`] (fill, hard shadow, glow layers, stroke) behind the
//!   children. The surrounding `div` keeps normal layout and hit-testing.
//!
//! No function here looks at a theme id; everything derives from tokens.

use crate::hsla;
use gpui::{
    canvas, point, px, Bounds, BoxShadow, Div, Hsla, InteractiveElement, ParentElement,
    PathBuilder, Pixels, Styled, Window,
};
use legendary_core::{Color, Shape, ShapeKind, Theme};

/// Everything needed to paint one shaped box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeStyle {
    /// Fill colour (`None` = transparent).
    pub fill: Option<Hsla>,
    /// Border colour (`None` = no border).
    pub stroke: Option<Hsla>,
    /// Border width in px (ignored when `stroke` is `None`).
    pub stroke_width: f32,
    /// Glow colour + blur radius in px.
    pub glow: Option<(Hsla, f32)>,
    /// Hard offset shadow colour + offset in px.
    pub shadow: Option<(Hsla, f32)>,
}

impl ShapeStyle {
    /// Build from token colours. `glow` adds the theme glow when enabled;
    /// the hard shadow is added whenever `effects.hard_shadow > 0` and `shadow` is set.
    pub fn from_tokens(
        theme: &Theme,
        fill: Option<Color>,
        stroke: Option<Color>,
        glow: bool,
        shadow: bool,
    ) -> Self {
        let fx = &theme.effects;
        Self {
            fill: fill.map(hsla),
            stroke: stroke.map(hsla),
            stroke_width: if stroke.is_some() {
                theme.shape.border_width
            } else {
                0.0
            },
            glow: (glow && fx.glow_enabled()).then(|| (hsla(fx.glow_color()), fx.glow_radius())),
            shadow: (shadow && fx.hard_shadow > 0.0)
                .then(|| (hsla(theme.palette.primary_active), fx.hard_shadow)),
        }
    }
}

/// One layer of the layered-stroke glow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlowLayer {
    /// Stroke width in px (centred on the outline).
    pub width: f32,
    /// Multiplier applied to the glow colour alpha.
    pub alpha: f32,
}

/// Layers for a glow of `radius` px around a `border`-px outline, outermost first.
/// Layers overlap, so alpha accumulates towards the edge (never above 0.6 total).
pub fn glow_layers(radius: f32, border: f32) -> Vec<GlowLayer> {
    if radius <= 0.0 {
        return Vec::new();
    }
    let n = (radius / 2.0).ceil().clamp(2.0, 8.0) as usize;
    (1..=n)
        .rev()
        .map(|k| GlowLayer {
            width: border + 2.0 * radius * k as f32 / n as f32,
            alpha: 0.6 / n as f32,
        })
        .collect()
}

/// Outline of a `w`×`h` box inset by `inset` px on every side (for centred strokes).
pub fn inset_outline(shape: &Shape, w: f32, h: f32, inset: f32) -> Vec<(f32, f32)> {
    let iw = (w - 2.0 * inset).max(0.0);
    let ih = (h - 2.0 * inset).max(0.0);
    shape
        .outline(iw, ih)
        .into_iter()
        .map(|(x, y)| (x + inset, y + inset))
        .collect()
}

fn scale_alpha(c: Hsla, k: f32) -> Hsla {
    Hsla {
        a: (c.a * k).clamp(0.0, 1.0),
        ..c
    }
}

fn poly(origin: gpui::Point<Pixels>, pts: &[(f32, f32)], dx: f32) -> Vec<gpui::Point<Pixels>> {
    pts.iter()
        .map(|&(x, y)| point(origin.x + px(x + dx), origin.y + px(y + dx)))
        .collect()
}

/// Paint a [`ShapeStyle`] into `bounds` following `shape`'s outline polygon.
pub fn paint_polygon(window: &mut Window, bounds: Bounds<Pixels>, shape: &Shape, s: &ShapeStyle) {
    let w: f32 = bounds.size.width.into();
    let h: f32 = bounds.size.height.into();
    let full = shape.outline(w, h);
    let fill_path = |window: &mut Window, pts: Vec<gpui::Point<Pixels>>, c: Hsla| {
        let mut b = PathBuilder::fill();
        b.add_polygon(&pts, true);
        if let Ok(p) = b.build() {
            window.paint_path(p, c);
        }
    };
    if let Some((c, off)) = s.shadow {
        fill_path(window, poly(bounds.origin, &full, off), c);
    }
    if let Some((c, r)) = s.glow {
        for l in glow_layers(r, s.stroke_width) {
            let mut b = PathBuilder::stroke(px(l.width));
            b.add_polygon(&poly(bounds.origin, &full, 0.0), true);
            if let Ok(p) = b.build() {
                window.paint_path(p, scale_alpha(c, l.alpha));
            }
        }
    }
    if let Some(c) = s.fill {
        fill_path(window, poly(bounds.origin, &full, 0.0), c);
    }
    if let (Some(c), bw) = (s.stroke, s.stroke_width) {
        if bw > 0.0 {
            let pts = inset_outline(shape, w, h, bw / 2.0);
            let mut b = PathBuilder::stroke(px(bw));
            b.add_polygon(&poly(bounds.origin, &pts, 0.0), true);
            if let Ok(p) = b.build() {
                window.paint_path(p, c);
            }
        }
    }
}

/// A `relative()` [`Div`] drawn in `shape` with `style`; add children afterwards.
///
/// For chamfered shapes the polygon is painted by a background canvas, so do
/// not set `.bg()`/`.border_*()` on the returned div yourself.
pub fn shaped(shape: &Shape, style: ShapeStyle) -> Div {
    let base = gpui::div().relative();
    match shape.kind() {
        ShapeKind::Chamfered => {
            let shape = shape.clone();
            base.child(
                canvas(
                    |_, _, _| (),
                    move |b, _, window, _| paint_polygon(window, b, &shape, &style),
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
        }
        kind => {
            let mut d = base;
            if kind == ShapeKind::Rounded {
                d = d.rounded(px(shape.radius));
            }
            if let Some(f) = style.fill {
                d = d.bg(f);
            }
            if let Some(c) = style.stroke {
                let w = px(style.stroke_width);
                let bw = &mut d.style().border_widths;
                bw.top = Some(w.into());
                bw.right = Some(w.into());
                bw.bottom = Some(w.into());
                bw.left = Some(w.into());
                d = d.border_color(c);
            }
            let mut shadows = Vec::new();
            if let Some((c, off)) = style.shadow {
                shadows.push(BoxShadow {
                    color: c,
                    offset: point(px(off), px(off)),
                    blur_radius: px(0.0),
                    spread_radius: px(0.0),
                });
            }
            if let Some((c, r)) = style.glow {
                shadows.push(BoxShadow {
                    color: c,
                    offset: point(px(0.0), px(0.0)),
                    blur_radius: px(r),
                    spread_radius: px(0.0),
                });
            }
            if shadows.is_empty() {
                d
            } else {
                d.shadow(shadows)
            }
        }
    }
}

/// Convenience: [`shaped`] with an element id so it can take click/hover listeners.
pub fn shaped_id(
    id: impl Into<gpui::ElementId>,
    shape: &Shape,
    style: ShapeStyle,
) -> gpui::Stateful<Div> {
    shaped(shape, style).id(id)
}

/// Shape for round controls (switch, radio, slider thumb) of `side` px height:
/// fully round (pill/circle) when the theme is rounded, otherwise the theme's own shape.
pub fn round_shape(shape: &Shape, side: f32) -> Shape {
    let mut s = shape.clone();
    if s.kind() == ShapeKind::Rounded {
        s.radius = side / 2.0;
    }
    s
}

/// Shape adapted to a small control of `side` px: radius capped at a quarter
/// of the side so small rounded controls do not turn into circles.
pub fn scaled_shape(shape: &Shape, side: f32) -> Shape {
    let mut s = shape.clone();
    s.radius = s.radius.min(side / 4.0);
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{light_rounded, square};
    use legendary_core::themes;

    #[test]
    fn glow_layers_are_ordered_and_bounded() {
        let l = glow_layers(12.0, 2.0);
        assert!(l.len() >= 2);
        assert!(l.windows(2).all(|w| w[0].width > w[1].width));
        let total: f32 = l.iter().map(|x| x.alpha).sum();
        assert!((total - 0.6).abs() < 1e-5);
        assert!(glow_layers(0.0, 2.0).is_empty());
    }

    #[test]
    fn style_glow_follows_intensity_not_id() {
        let t = themes::cyberpunk::night_city();
        let on = ShapeStyle::from_tokens(&t, None, None, true, false);
        assert!(on.glow.is_some());
        let off =
            ShapeStyle::from_tokens(&t.clone().with_glow_intensity(0.0), None, None, true, false);
        assert!(off.glow.is_none());
        let (_, r1) = on.glow.unwrap();
        let hi = ShapeStyle::from_tokens(&t.with_glow_intensity(2.0), None, None, true, false);
        assert!(hi.glow.unwrap().1 > r1);
    }

    #[test]
    fn rounded_and_square_follow_effect_tokens() {
        let l = light_rounded();
        let s = ShapeStyle::from_tokens(&l, Some(l.palette.surface), None, true, true);
        assert!(s.glow.is_none() && s.shadow.is_none());
        let q = square();
        let s = ShapeStyle::from_tokens(&q, None, Some(q.palette.border), true, true);
        assert!(s.glow.is_none());
        assert_eq!(s.shadow.map(|x| x.1), Some(3.0));
        assert_eq!(s.stroke_width, 2.0);
    }

    #[test]
    fn inset_outline_stays_inside() {
        for t in themes::all() {
            let pts = inset_outline(&t.shape, 100.0, 40.0, 1.0);
            assert!(pts
                .iter()
                .all(|&(x, y)| (1.0..=99.0).contains(&x) && (1.0..=39.0).contains(&y)));
        }
    }

    #[test]
    fn polygon_vertex_count_per_kind() {
        let n = |t: &legendary_core::Theme| t.shape.outline(80.0, 40.0).len();
        assert_eq!(n(&light_rounded()), 4);
        assert_eq!(n(&square()), 4);
        assert!(n(&themes::cyberpunk::night_city()) > 4);
    }

    #[test]
    fn scaled_shape_caps_radius() {
        let l = light_rounded();
        assert_eq!(scaled_shape(&l.shape, 16.0).radius, 4.0);
        assert_eq!(scaled_shape(&l.shape, 100.0).radius, l.shape.radius);
    }

    #[test]
    fn round_shape_only_changes_rounded() {
        let l = light_rounded();
        assert_eq!(round_shape(&l.shape, 20.0).radius, 10.0);
        let q = square();
        assert_eq!(round_shape(&q.shape, 20.0), q.shape);
    }
}
