//! [`ShaderSurface`]: a panel whose fill is a live shader, drawn in the design's own shape.
//!
//! gpui clips to rectangles only, so the cut corners of a chamfered design are covered by
//! triangles in the page colour; rounded designs use the shader's own corner radius.

use crate::paint::{poly, scale_alpha, shaped, ShapeStyle};
use crate::{hsla, theme_of, ShaderView};
use dodai_core::{Shape, ShapeKind};
use gpui::{
    canvas, div, px, AnyElement, App, Hsla, IntoElement, ParentElement, PathBuilder, RenderOnce,
    SharedString, Styled, Window,
};

/// The corner triangles a chamfered `shape` cuts from its `w`×`h` box.
pub fn cut_triangles(shape: &Shape, w: f32, h: f32) -> Vec<[(f32, f32); 3]> {
    let pts = shape.outline(w, h);
    let on_box = |p: (f32, f32)| {
        (p.0.abs() < 0.5 || (p.0 - w).abs() < 0.5) && (p.1.abs() < 0.5 || (p.1 - h).abs() < 0.5)
    };
    let mut out = Vec::new();
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        if (a.0 - b.0).abs() < 0.5 || (a.1 - b.1).abs() < 0.5 {
            continue;
        }
        if let Some(c) = [(a.0, b.1), (b.0, a.1)].into_iter().find(|c| on_box(*c)) {
            out.push([a, c, b]);
        }
    }
    out
}

/// A shader-filled panel. Content goes on top; `scrim` darkens the shader behind it so text
/// stays legible.
#[derive(IntoElement)]
pub struct ShaderSurface {
    id: SharedString,
    preset: &'static dodai_shaders::Preset,
    size: (f32, f32),
    scrim: f32,
    children: Vec<AnyElement>,
}

impl ShaderSurface {
    /// A `width`×`height` px surface filled by `preset`.
    pub fn new(
        id: impl Into<SharedString>,
        preset: &'static dodai_shaders::Preset,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            id: id.into(),
            preset,
            size: (width, height),
            scrim: 0.35,
            children: Vec::new(),
        }
    }

    /// How strongly the page colour dims the shader behind the content (`0.0..=1.0`).
    pub fn scrim(mut self, s: f32) -> Self {
        self.scrim = s.clamp(0.0, 1.0);
        self
    }
}

impl ParentElement for ShaderSurface {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for ShaderSurface {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let shape = t.shape.clone();
        let page: Hsla = hsla(t.palette.background);
        let (w, h) = self.size;
        let mut view = ShaderView::preset(self.id.clone(), self.preset)
            .aspect(w / h)
            .pixel(3.0);
        if shape.kind() == ShapeKind::Rounded {
            view = view.rounded(shape.radius);
        }
        let mask_shape = shape.clone();
        let scrim = self.scrim;
        let frame = ShapeStyle::from_tokens(&t, None, Some(t.palette.border), false, false);
        shaped(&shape, frame)
            .w(px(w))
            .h(px(h))
            .overflow_hidden()
            .child(div().absolute().top_0().left_0().size_full().child(view))
            .child(
                canvas(
                    |_, _, _| (),
                    move |b, _, window, _| {
                        let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                        // Dim the shader, then hide what the cut corners remove.
                        let full = mask_shape.outline(bw, bh);
                        let mut pb = PathBuilder::fill();
                        pb.add_polygon(&poly(b.origin, &full, (0.0, 0.0)), true);
                        if let Ok(p) = pb.build() {
                            window.paint_path(p, scale_alpha(page, scrim));
                        }
                        for tri in cut_triangles(&mask_shape, bw, bh) {
                            let mut pb = PathBuilder::fill();
                            pb.add_polygon(&poly(b.origin, &tri, (0.0, 0.0)), true);
                            if let Ok(p) = pb.build() {
                                window.paint_path(p, page);
                            }
                        }
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
            .child(
                div()
                    .relative()
                    .size_full()
                    .p(px(t.spacing.md))
                    .children(self.children),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cut_triangles_cover_exactly_the_cut_corners() {
        let mut sh = dodai::themes::cyberpunk::yellow().shape;
        sh.radius = 0.0;
        sh.chamfer = 10.0;
        sh.corners = dodai_core::Corners::BOTTOM_RIGHT;
        let tris = cut_triangles(&sh, 100.0, 40.0);
        assert_eq!(tris.len(), 1);
        assert!(tris[0].contains(&(100.0, 40.0)));
        sh.corners = dodai_core::Corners::NONE;
        sh.chamfer = 0.0;
        assert!(cut_triangles(&sh, 100.0, 40.0).is_empty());
    }
}
