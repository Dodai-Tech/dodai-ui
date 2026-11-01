//! One frame of an effect as an SVG document, for toolkits without a canvas (CSS backgrounds,
//! server-rendered pages, images): a [`Painter`] that writes elements, and [`surface_svg`] that
//! runs [`paint`](crate::paint) through it.

use crate::{paint, Frame, Painter, Rgba, SurfacePaint};
use std::fmt::Write;

/// Collects the primitives of a frame as SVG elements.
#[derive(Default)]
pub struct SvgPainter {
    body: String,
}

/// `#rrggbb` of `c`, and its opacity.
fn paint_of(c: Rgba) -> (String, f32) {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    (
        format!("#{:02x}{:02x}{:02x}", byte(c.r), byte(c.g), byte(c.b)),
        (c.a.clamp(0.0, 1.0) * 100.0).round() / 100.0,
    )
}

/// Coordinates to hundredths of a px, so documents stay small.
fn n(v: f32) -> f32 {
    (v * 100.0).round() / 100.0
}

fn points(pts: &[(f32, f32)]) -> String {
    pts.iter()
        .map(|&(x, y)| format!("{},{}", n(x), n(y)))
        .collect::<Vec<_>>()
        .join(" ")
}

impl SvgPainter {
    /// Whether anything was drawn.
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }

    /// The elements drawn so far.
    pub fn into_elements(self) -> String {
        self.body
    }
}

impl Painter for SvgPainter {
    fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba) {
        let (f, o) = paint_of(c);
        let _ = write!(
            self.body,
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{f}\" fill-opacity=\"{o}\"/>",
            n(x),
            n(y),
            n(w),
            n(h)
        );
    }

    fn dot(&mut self, cx: f32, cy: f32, d: f32, c: Rgba) {
        let (f, o) = paint_of(c);
        let _ = write!(
            self.body,
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{f}\" fill-opacity=\"{o}\"/>",
            n(cx),
            n(cy),
            n(d / 2.0)
        );
    }

    fn stroke(&mut self, pts: &[(f32, f32)], closed: bool, width: f32, c: Rgba) {
        let (f, o) = paint_of(c);
        let tag = if closed { "polygon" } else { "polyline" };
        let _ = write!(
            self.body,
            "<{tag} points=\"{}\" fill=\"none\" stroke=\"{f}\" stroke-opacity=\"{o}\" stroke-width=\"{}\" stroke-linejoin=\"round\"/>",
            points(pts),
            n(width)
        );
    }

    fn fill(&mut self, pts: &[(f32, f32)], c: Rgba) {
        let (f, o) = paint_of(c);
        let _ = write!(
            self.body,
            "<polygon points=\"{}\" fill=\"{f}\" fill-opacity=\"{o}\"/>",
            points(pts)
        );
    }
}

/// The frame `f` of `sp` as a `w`×`h` SVG document, or `None` when nothing is drawn (too small
/// a surface, or an empty mix). The second value says the effect animates, so a caller that
/// wants motion should ask again with a later `f.time`.
pub fn surface_svg(f: &Frame, sp: &SurfacePaint) -> Option<(String, bool)> {
    let mut pen = SvgPainter::default();
    let animated = paint(&mut pen, f, sp);
    if pen.is_empty() {
        return None;
    }
    let (w, h) = (n(f.w), n(f.h));
    Some((
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">{}</svg>",
            pen.into_elements()
        ),
        animated,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Clip, SurfaceFx, SurfaceTuning, Texture};

    fn paint_of_texture(texture: Texture) -> SurfacePaint {
        let ink = Rgba {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };
        SurfacePaint {
            fx: SurfaceFx {
                texture,
                strength: 0.8,
                ..SurfaceFx::NONE
            },
            ink,
            hole: Rgba::WHITE,
            a: ink,
            b: ink,
            light: Rgba::WHITE,
            neon: ink,
            shade: ink,
            wash: [ink; 3],
            tune: SurfaceTuning::DEFAULT,
            pressable: false,
            fill_lightness: 0.0,
        }
    }

    fn frame(shape: &Clip) -> Frame<'_> {
        Frame {
            w: 120.0,
            h: 48.0,
            shape,
            seed: 3,
            time: 0.0,
            presses: &[],
        }
    }

    #[test]
    fn a_texture_becomes_an_svg_of_the_surface_size() {
        let shape = Clip::default();
        let (svg, animated) =
            surface_svg(&frame(&shape), &paint_of_texture(Texture::Grid)).expect("grid draws");
        assert!(svg.starts_with("<svg ") && svg.ends_with("</svg>"));
        assert!(svg.contains("viewBox=\"0 0 120 48\""));
        assert!(svg.contains("<rect ") && svg.contains("fill=\"#1a334d\""));
        assert!(!animated);
    }

    #[test]
    fn nothing_to_draw_is_none() {
        let shape = Clip::default();
        let mut sp = paint_of_texture(Texture::Grid);
        sp.fx = SurfaceFx::NONE;
        assert!(surface_svg(&frame(&shape), &sp).is_none());
        let small = Frame {
            w: 4.0,
            ..frame(&shape)
        };
        assert!(surface_svg(&small, &paint_of_texture(Texture::Grid)).is_none());
    }

    #[test]
    fn dots_and_paths_use_their_own_elements() {
        let mut pen = SvgPainter::default();
        pen.dot(5.0, 6.0, 4.0, Rgba::WHITE);
        pen.stroke(&[(0.0, 0.0), (3.0, 4.0)], false, 1.5, Rgba::WHITE);
        pen.stroke(
            &[(0.0, 0.0), (3.0, 4.0), (0.0, 4.0)],
            true,
            1.0,
            Rgba::WHITE,
        );
        pen.fill(&[(0.0, 0.0), (3.0, 4.0), (0.0, 4.0)], Rgba::WHITE);
        let s = pen.into_elements();
        assert!(s.contains("<circle cx=\"5\" cy=\"6\" r=\"2\""));
        assert!(s.contains("<polyline points=\"0,0 3,4\""));
        assert!(s.contains("<polygon points=\"0,0 3,4 0,4\" fill=\"none\""));
        assert!(s.contains("<polygon points=\"0,0 3,4 0,4\" fill=\"#ffffff\""));
    }
}
