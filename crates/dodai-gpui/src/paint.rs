//! Shaped painting: turns [`Shape`] tokens into gpui boxes.
//!
//! * `Square` / `Rounded` themes use plain `div` styling (border, radius, box shadow).
//! * `Chamfered` themes paint the [`Shape::outline`] polygon on an absolutely
//!   positioned [`canvas`] (fill, hard shadow, glow layers, stroke) behind the
//!   children. The surrounding `div` keeps normal layout and hit-testing.
//!
//! No function here looks at a theme id; everything derives from tokens.

use crate::hsla;
use dodai_core::{Color, Shape, ShapeKind, Theme};
use gpui::{
    canvas, point, px, Bounds, BoxShadow, Div, Hsla, InteractiveElement, ParentElement,
    PathBuilder, Pixels, Styled, Window,
};

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
    /// Fine horizontal scanline texture over the fill (from `effects.scanlines`).
    pub scanlines: bool,
    /// Animated highlight sweep, clipped to the shape (see [`Shimmer`]).
    pub shimmer: Option<Shimmer>,
}

/// Shimmer sweep painted by [`shaped`] over any shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shimmer {
    /// Peak highlight colour (include the alpha).
    pub color: Hsla,
    /// Duration of one sweep in ms.
    pub ms: u32,
    pub playback: crate::feedback::Playback,
}

impl ShapeStyle {
    /// Add a shimmer sweep that follows whatever shape this style is drawn in.
    pub fn with_shimmer(mut self, s: Shimmer) -> Self {
        self.shimmer = Some(s);
        self
    }
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
            scanlines: fx.scanlines && fill.is_some(),
            shimmer: None,
        }
    }
}

/// Scanline pitch in px (one dark 1px line per pitch).
pub const SCANLINE_PITCH: f32 = 3.0;
/// Scanline alpha (black).
pub const SCANLINE_ALPHA: f32 = 0.18;

/// Horizontal extent `(x0, x1)` of `shape` at row `y` in a `w`×`h` box, or
/// `None` when the row is outside. Follows chamfers and rounded corners.
pub fn row_span(shape: &Shape, w: f32, h: f32, y: f32) -> Option<(f32, f32)> {
    if y < 0.0 || y > h || w <= 0.0 {
        return None;
    }
    match shape.kind() {
        ShapeKind::Chamfered => {
            let pts = shape.outline(w, h);
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for i in 0..pts.len() {
                let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
                let (y0, y1) = (a.1.min(b.1), a.1.max(b.1));
                if y < y0 || y > y1 {
                    continue;
                }
                let x = if (b.1 - a.1).abs() < 1e-6 {
                    // horizontal edge: both ends count
                    lo = lo.min(a.0.min(b.0));
                    hi = hi.max(a.0.max(b.0));
                    continue;
                } else {
                    a.0 + (b.0 - a.0) * (y - a.1) / (b.1 - a.1)
                };
                lo = lo.min(x);
                hi = hi.max(x);
            }
            (lo <= hi).then_some((lo, hi))
        }
        ShapeKind::Rounded => {
            let r = shape.radius.min(w / 2.0).min(h / 2.0);
            let dy = if y < r {
                r - y
            } else if y > h - r {
                y - (h - r)
            } else {
                0.0
            };
            let inset = if dy > 0.0 && r > 0.0 {
                r - (r * r - dy * dy).max(0.0).sqrt()
            } else {
                0.0
            };
            Some((inset, w - inset))
        }
        ShapeKind::Square => Some((0.0, w)),
    }
}

/// Polygon of the bottom `band` px of `shape` in a `w`×`h` box (follows cut corners).
pub fn bottom_band(shape: &Shape, w: f32, h: f32, band: f32) -> Vec<(f32, f32)> {
    let top = (h - band).max(0.0);
    let (a0, a1) = row_span(shape, w, h, top).unwrap_or((0.0, w));
    let (b0, b1) = row_span(shape, w, h, h).unwrap_or((0.0, w));
    vec![(a0, top), (a1, top), (b1, h), (b0, h)]
}

/// Fill the bottom `band` px of `shape` with `color` (absolute canvas child).
pub fn bottom_band_canvas(shape: &Shape, band: f32, color: Hsla) -> impl gpui::IntoElement {
    let shape = shape.clone();
    canvas(
        |_, _, _| (),
        move |b, _, window, _| {
            let w: f32 = b.size.width.into();
            let h: f32 = b.size.height.into();
            let pts = poly(b.origin, &bottom_band(&shape, w, h, band), 0.0);
            let mut pb = PathBuilder::fill();
            pb.add_polygon(&pts, true);
            if let Ok(p) = pb.build() {
                window.paint_path(p, color);
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

/// Paint scanline rows inside `bounds`, clipped to `shape`.
pub fn paint_scanlines(window: &mut Window, bounds: Bounds<Pixels>, shape: &Shape) {
    let w: f32 = bounds.size.width.into();
    let h: f32 = bounds.size.height.into();
    let bw = shape.border_width;
    let mut y = SCANLINE_PITCH - 1.0;
    while y < h {
        if y > bw && y < h - bw {
            if let Some((x0, x1)) = row_span(shape, w, h, y) {
                let (x0, x1) = (x0.max(bw), x1.min(w - bw));
                if x1 > x0 {
                    window.paint_quad(gpui::fill(
                        Bounds::new(
                            point(bounds.origin.x + px(x0), bounds.origin.y + px(y)),
                            gpui::size(px(x1 - x0), px(1.0)),
                        ),
                        gpui::hsla(0.0, 0.0, 0.0, SCANLINE_ALPHA),
                    ));
                }
            }
        }
        y += SCANLINE_PITCH;
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

pub(crate) fn scale_alpha(c: Hsla, k: f32) -> Hsla {
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
    if s.scanlines {
        paint_scanlines(window, bounds, shape);
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
    let d = shaped_base(shape, style);
    match style.shimmer {
        Some(sh) => d.child(shimmer_overlay(shape, sh)),
        None => d,
    }
}

fn shaped_base(shape: &Shape, style: ShapeStyle) -> Div {
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
            if style.scanlines {
                let sh = shape.clone();
                d = d.child(
                    canvas(
                        |_, _, _| (),
                        move |b, _, window, _| paint_scanlines(window, b, &sh),
                    )
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full(),
                );
            }
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

/// Highlight band at `x..x+w` (fractions of the width) painted row by row so
/// it stays inside `shape` (rounded / chamfered corners, circles).
fn shimmer_band(shape: Shape, x: f32, w: f32, color: Hsla) -> impl gpui::IntoElement {
    canvas(
        |_, _, _| (),
        move |b, _, window, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let (x0, x1) = (x * bw, (x + w) * bw);
            let at = |p: f32| {
                let f = ((p - x0) / (x1 - x0).max(1e-3)).clamp(0.0, 1.0);
                hsla_a(color, color.a * f)
            };
            let mut y = 0.0;
            while y < bh {
                let step = (bh - y).min(1.0);
                if let Some((a, c)) = row_span(&shape, bw, bh, y + step / 2.0) {
                    let (l, r) = (x0.max(a), x1.min(c));
                    if r > l {
                        let bounds = gpui::Bounds {
                            origin: point(b.origin.x + px(l), b.origin.y + px(y)),
                            size: gpui::size(px(r - l), px(step)),
                        };
                        window.paint_quad(gpui::fill(
                            bounds,
                            gpui::linear_gradient(
                                90.0,
                                gpui::linear_color_stop(at(l), 0.0),
                                gpui::linear_color_stop(at(r), 1.0),
                            ),
                        ));
                    }
                }
                y += step;
            }
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

fn hsla_a(c: Hsla, a: f32) -> Hsla {
    Hsla { a, ..c }
}

/// Sweep overlay clipped to `shape`; the animation is keyed by the parent
/// element's id, so give the shaped element an id (e.g. [`shaped_id`]).
fn shimmer_overlay(shape: &Shape, sh: Shimmer) -> impl gpui::IntoElement {
    use crate::feedback::Playback;
    use gpui::AnimationExt;
    let dur = std::time::Duration::from_millis(sh.ms.max(1) as u64);
    let anim = match sh.playback {
        Playback::Loop => gpui::Animation::new(dur).repeat(),
        Playback::Once => gpui::Animation::new(dur),
        Playback::Stopped => {
            gpui::Animation::new(std::time::Duration::from_millis(1)).with_easing(|_| 0.35)
        }
    };
    let shape = shape.clone();
    gpui::div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .with_animation("shimmer", anim, move |el, delta| {
            let w = 0.4;
            el.child(shimmer_band(
                shape.clone(),
                -w + delta * (1.0 + w),
                w,
                sh.color,
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bottom_band_stops_at_cut_corner() {
        let mut sh = dodai_core::themes::cyberpunk::night_city().shape;
        sh.chamfer = 10.0;
        sh.corners = dodai_core::Corners::BOTTOM_RIGHT;
        let b = bottom_band(&sh, 100.0, 40.0, 2.0);
        assert_eq!(b[0], (0.0, 38.0));
        assert!(b[2].0 < 91.0, "bottom-right edge must be cut: {:?}", b[2]);
        assert_eq!(b[3], (0.0, 40.0));
    }

    #[test]
    fn row_span_follows_shape() {
        let mut sh = dodai_core::themes::cyberpunk::night_city().shape;
        sh.chamfer = 10.0;
        sh.radius = 0.0;
        sh.corners = dodai_core::Corners::BOTTOM_RIGHT;
        assert_eq!(sh.kind(), ShapeKind::Chamfered);
        assert_eq!(row_span(&sh, 100.0, 40.0, 5.0), Some((0.0, 100.0)));
        let (_, x1) = row_span(&sh, 100.0, 40.0, 39.0).unwrap();
        assert!(x1 < 95.0, "cut corner narrows the last row, got {x1}");
        assert_eq!(row_span(&sh, 100.0, 40.0, 41.0), None);
        let mut r = crate::testing::light_rounded().shape;
        r.radius = 8.0;
        r.chamfer = 0.0;
        let (x0, _) = row_span(&r, 100.0, 40.0, 1.0).unwrap();
        assert!(x0 > 1.0);
        assert_eq!(row_span(&r, 100.0, 40.0, 20.0), Some((0.0, 100.0)));
    }
    use crate::testing::{light_rounded, square};
    use dodai_core::themes;

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
        let n = |t: &dodai_core::Theme| t.shape.outline(80.0, 40.0).len();
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
