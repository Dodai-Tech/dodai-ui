//! Surface textures and light effects, drawn as egui shapes over a control's fill.
//!
//! The same mix and the same geometry as every other toolkit: `dodai-fx` works out each frame
//! (textures, lit lines, glitch, flicker, sheen, pulse, chase, brackets, wash, ...) and this
//! module only turns its four primitives into egui shapes. Set the mix for the whole app with
//! [`set_surface`]; each [`box_shapes_for`](crate::paint::box_shapes_for) call paints it on
//! the surfaces it targets. A design draws only the textures and effects it owns.

use dodai_core::{ResolvedSurface, Surface, SurfaceClass, Theme};
use dodai_fx::{Frame, Painter, Rgba};
use egui::epaint::{Mesh, PathShape};
use egui::{pos2, vec2, Color32, Pos2, Rect, Shape, Stroke};
use std::cell::Cell;
use std::sync::OnceLock;
use std::time::Instant;

thread_local! {
    static MIX: Cell<Surface> = const { Cell::new(Surface::INHERIT) };
    static ANIMATED: Cell<bool> = const { Cell::new(false) };
}

/// The surface mix the app set ([`Surface::INHERIT`] until it sets one).
pub fn surface() -> Surface {
    MIX.with(Cell::get)
}

/// The surface mix applied to every shaped surface of `theme`: the app's, or until it sets one
/// the design's own glass ([`Theme::default_surface`]).
pub fn resolved_surface(theme: &Theme) -> ResolvedSurface {
    theme.resolve_surface(&surface())
}

/// Apply `mix` to every shaped surface (repaint with `ctx.request_repaint()`). A [`ResolvedSurface`]
/// converts into a custom [`Surface`]; [`Surface::INHERIT`] goes back to the design's own.
pub fn set_surface(mix: impl Into<Surface>) {
    MIX.with(|c| c.set(mix.into()));
}

/// Whether a surface painted since the last call animates, so the frame should be repainted.
pub fn take_animated() -> bool {
    ANIMATED.with(|a| a.replace(false))
}

fn clock() -> f32 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f32()
}

fn colour(c: Rgba) -> Color32 {
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(byte(c.r), byte(c.g), byte(c.b), byte(c.a))
}

/// `dodai-fx`'s four primitives as egui shapes, moved to the surface's corner.
struct EguiPainter {
    origin: Pos2,
    out: Vec<Shape>,
}

impl EguiPainter {
    fn at(&self, x: f32, y: f32) -> Pos2 {
        pos2(self.origin.x + x, self.origin.y + y)
    }

    fn points(&self, pts: &[(f32, f32)]) -> Vec<Pos2> {
        pts.iter().map(|&(x, y)| self.at(x, y)).collect()
    }
}

impl Painter for EguiPainter {
    fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba) {
        let r = Rect::from_min_size(self.at(x, y), vec2(w, h));
        self.out.push(Shape::rect_filled(r, 0.0, colour(c)));
    }

    fn dot(&mut self, cx: f32, cy: f32, d: f32, c: Rgba) {
        self.out
            .push(Shape::circle_filled(self.at(cx, cy), d / 2.0, colour(c)));
    }

    fn stroke(&mut self, pts: &[(f32, f32)], closed: bool, width: f32, c: Rgba) {
        let (pts, stroke) = (self.points(pts), Stroke::new(width, colour(c)));
        self.out.push(if closed {
            PathShape::closed_line(pts, stroke).into()
        } else {
            Shape::line(pts, stroke)
        });
    }

    /// A fan from the centroid: right for convex and for scooped (star-shaped) outlines alike,
    /// where egui's own polygon fill takes convex shapes only.
    fn fill(&mut self, pts: &[(f32, f32)], c: Rgba) {
        if pts.len() < 3 {
            return;
        }
        let col = colour(c);
        let n = pts.len() as f32;
        let centre = pts
            .iter()
            .fold((0.0, 0.0), |(sx, sy), &(x, y)| (sx + x / n, sy + y / n));
        let mut mesh = Mesh::default();
        mesh.colored_vertex(self.at(centre.0, centre.1), col);
        for &(x, y) in pts {
            mesh.colored_vertex(self.at(x, y), col);
        }
        let n = pts.len() as u32;
        for i in 0..n {
            mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
        }
        self.out.push(Shape::mesh(mesh));
    }
}

/// The shapes of the app-wide mix for a `class` surface over `rect` (empty when nothing
/// applies). `fill` is the colour underneath, so patterns stay visible on bright and dark fills.
pub fn surface_shapes(
    theme: &Theme,
    rect: Rect,
    class: SurfaceClass,
    fill: Option<dodai_core::Color>,
) -> Vec<Shape> {
    let Some(mut paint) =
        dodai_core::surface_paint(theme, class, &Surface::INHERIT, resolved_surface(theme))
    else {
        return Vec::new();
    };
    if let Some(f) = fill {
        paint = paint.contrasting(Rgba::from(f).lightness());
    }
    let shape = theme.shape.clip();
    let frame = Frame {
        w: rect.width(),
        h: rect.height(),
        shape: &shape,
        // Stable per size, so a surface keeps its pattern while the page scrolls.
        seed: (rect.width() as i32).wrapping_mul(31) ^ rect.height() as i32,
        time: clock(),
        presses: &[],
    };
    let mut pen = EguiPainter {
        origin: rect.min,
        out: Vec::new(),
    };
    if dodai_fx::paint(&mut pen, &frame, &paint) {
        ANIMATED.with(|a| a.set(true));
    }
    pen.out
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_core::{SurfaceTargets, Texture};

    fn rect() -> Rect {
        Rect::from_min_size(pos2(10.0, 10.0), vec2(120.0, 48.0))
    }

    fn all(fx: ResolvedSurface) {
        set_surface(ResolvedSurface {
            targets: SurfaceTargets::ALL,
            ..fx
        });
    }

    #[test]
    fn nothing_is_drawn_by_default() {
        set_surface(ResolvedSurface::NONE);
        let t = dodai_core::themes::cyberpunk::yellow();
        assert!(surface_shapes(&t, rect(), SurfaceClass::Button, None).is_empty());
    }

    #[test]
    fn a_texture_stays_inside_the_rect() {
        let t = dodai_core::themes::cyberpunk::yellow();
        for tex in [
            Texture::Grain,
            Texture::Halftone,
            Texture::Hatch,
            Texture::Grid,
            Texture::Checker,
            Texture::Carbon,
            Texture::Brushed,
            Texture::Stripes,
        ] {
            all(ResolvedSurface {
                texture: tex,
                strength: 0.8,
                ..ResolvedSurface::NONE
            });
            let shapes = surface_shapes(&t, rect(), SurfaceClass::Button, None);
            assert!(!shapes.is_empty(), "{tex:?}");
            for s in &shapes {
                assert!(
                    rect().expand(2.0).contains_rect(s.visual_bounding_rect()),
                    "{tex:?} leaves the rect"
                );
            }
        }
        set_surface(ResolvedSurface::NONE);
    }

    #[test]
    fn a_design_draws_only_what_it_owns() {
        let m = dodai_core::themes::material::light();
        all(ResolvedSurface {
            texture: Texture::Circuit,
            glitch: 1.0,
            ..ResolvedSurface::NONE
        });
        assert!(surface_shapes(&m, rect(), SurfaceClass::Button, None).is_empty());
        set_surface(ResolvedSurface::NONE);
    }

    #[test]
    fn animated_effects_ask_for_frames() {
        let t = dodai_core::themes::cyberpunk::yellow();
        all(ResolvedSurface {
            pulse: 0.8,
            ..ResolvedSurface::NONE
        });
        let _ = take_animated();
        assert!(!surface_shapes(&t, rect(), SurfaceClass::Panel, None).is_empty());
        assert!(take_animated());
        set_surface(ResolvedSurface::NONE);
    }

    #[test]
    fn lit_lines_add_a_glow_to_line_textures() {
        let t = dodai_core::themes::cyberpunk::yellow();
        let count = |neon: f32| {
            all(ResolvedSurface {
                texture: Texture::Grid,
                strength: 0.8,
                neon,
                ..ResolvedSurface::NONE
            });
            surface_shapes(&t, rect(), SurfaceClass::Button, None).len()
        };
        let (flat, lit) = (count(0.0), count(1.0));
        assert!(lit > flat, "{lit} shapes lit, {flat} flat");
        set_surface(ResolvedSurface::NONE);
    }

    #[test]
    fn glitch_and_flicker_animate() {
        let t = dodai_core::themes::cyberpunk::yellow();
        all(ResolvedSurface {
            glitch: 1.0,
            flicker: 1.0,
            ..ResolvedSurface::NONE
        });
        let _ = take_animated();
        let _ = surface_shapes(&t, rect(), SurfaceClass::Panel, None);
        assert!(take_animated());
        set_surface(ResolvedSurface::NONE);
    }
}

#[cfg(test)]
mod surface_api_tests {
    use super::*;

    #[test]
    fn set_surface_is_what_resolved_surface_and_surface_report() {
        let t = dodai_core::themes::cyberpunk::yellow();
        set_surface(Surface::INHERIT);
        assert_eq!(resolved_surface(&t), t.default_surface());
        set_surface(Surface::frosted().glitch(0.5));
        let fx = resolved_surface(&t);
        assert!(fx.frost > 0.0 && fx.glitch == 0.5);
        assert_eq!(surface(), Surface::frosted().glitch(0.5));
        set_surface(Surface::NONE);
        assert_eq!(resolved_surface(&t), ResolvedSurface::NONE);
        set_surface(Surface::INHERIT);
    }
}
