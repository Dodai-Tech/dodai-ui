//! Surface textures and effects, drawn by `dodai-surface` as absolutely placed UI nodes.
//!
//! Bevy UI has no immediate-mode canvas, so the [`Painter`] records primitives
//! ([`Prim`]) and [`spawn_surface_fx`] turns them into child nodes: quads and dots become
//! coloured nodes, strokes become thin rotated nodes. Filled polygons are not drawn yet.
//! Static only (no click ripples, no animation).

use bevy::prelude::*;
use dodai_core::{Shape, Surface, SurfaceClass, SurfaceFx, Theme};
use dodai_surface::{Frame, Painter, Rgba, SurfacePaint};

use crate::theme::color;

/// One recorded drawing primitive, in px relative to the surface's top-left corner.
#[derive(Debug, Clone, PartialEq)]
pub enum Prim {
    /// Filled rectangle `x, y, w, h`.
    Quad(f32, f32, f32, f32, Rgba),
    /// Filled circle: centre `x, y` and diameter.
    Dot(f32, f32, f32, Rgba),
    /// One straight stroke segment `x0, y0, x1, y1`, width.
    Line(f32, f32, f32, f32, f32, Rgba),
}

/// A [`Painter`] that records [`Prim`]s.
#[derive(Debug, Default)]
pub struct Recorder {
    /// What was drawn.
    pub prims: Vec<Prim>,
    /// Filled polygons skipped (no polygon fill in Bevy UI yet).
    pub skipped_fills: usize,
}

impl Painter for Recorder {
    fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba) {
        self.prims.push(Prim::Quad(x, y, w, h, c));
    }

    fn dot(&mut self, cx: f32, cy: f32, d: f32, c: Rgba) {
        self.prims.push(Prim::Dot(cx, cy, d, c));
    }

    fn stroke(&mut self, pts: &[(f32, f32)], closed: bool, width: f32, c: Rgba) {
        for w in pts.windows(2) {
            self.prims
                .push(Prim::Line(w[0].0, w[0].1, w[1].0, w[1].1, width, c));
        }
        if let (true, Some(a), Some(b)) = (closed, pts.last(), pts.first()) {
            self.prims.push(Prim::Line(a.0, a.1, b.0, b.1, width, c));
        }
    }

    fn fill(&mut self, _pts: &[(f32, f32)], _c: Rgba) {
        self.skipped_fills += 1;
    }
}

/// Record what `fx` draws on a `size` panel of `theme`'s shape; `None` when nothing is drawn.
pub fn record_surface_fx(theme: &Theme, fx: SurfaceFx, size: Vec2) -> Option<Recorder> {
    let paint = SurfacePaint::resolve(theme, SurfaceClass::Panel, &Surface::INHERIT, fx)?;
    let shape: &Shape = &theme.shape;
    let frame = Frame {
        w: size.x,
        h: size.y,
        shape,
        seed: 1,
        time: 0.0,
        presses: &[],
    };
    let mut rec = Recorder::default();
    dodai_surface::paint(&mut rec, &frame, &paint);
    Some(rec)
}

/// Spawn the effects of `fx` as children of `parent`, which must be `size` px and
/// `PositionType`-agnostic (the nodes are absolute).
pub fn spawn_surface_fx(
    parent: &mut ChildSpawnerCommands,
    theme: &Theme,
    fx: SurfaceFx,
    size: Vec2,
) {
    let Some(rec) = record_surface_fx(theme, fx, size) else {
        return;
    };
    for p in rec.prims {
        match p {
            Prim::Quad(x, y, w, h, c) => {
                parent.spawn((rect(x, y, w, h), BackgroundColor(color(to_core(c)))));
            }
            Prim::Dot(cx, cy, d, c) => {
                let mut n = rect(cx - d / 2.0, cy - d / 2.0, d, d);
                n.border_radius = BorderRadius::all(Val::Px(d / 2.0));
                parent.spawn((n, BackgroundColor(color(to_core(c)))));
            }
            Prim::Line(x0, y0, x1, y1, width, c) => {
                let (dx, dy) = (x1 - x0, y1 - y0);
                let len = dx.hypot(dy);
                if len < 0.01 {
                    continue;
                }
                // A centred bar of `len x width`, rotated about its centre.
                let (mx, my) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
                parent.spawn((
                    rect(mx - len / 2.0, my - width / 2.0, len, width),
                    BackgroundColor(color(to_core(c))),
                    UiTransform::from_rotation(Rot2::radians(dy.atan2(dx))),
                ));
            }
        }
    }
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(x),
        top: Val::Px(y),
        width: Val::Px(w),
        height: Val::Px(h),
        ..default()
    }
}

fn to_core(c: Rgba) -> dodai_core::Color {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    dodai_core::Color {
        r: b(c.r),
        g: b(c.g),
        b: b(c.b),
        a: b(c.a),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_core::Texture;

    #[test]
    fn a_grid_records_primitives_and_nothing_records_nothing() {
        let t = dodai::themes::cyberpunk::yellow();
        let fx = SurfaceFx {
            texture: Texture::Circuit,
            strength: 0.8,
            scale: 1.0,
            ..SurfaceFx::NONE
        };
        let rec = record_surface_fx(&t, fx, Vec2::new(200.0, 100.0)).unwrap();
        assert!(!rec.prims.is_empty());
        assert!(record_surface_fx(&t, SurfaceFx::NONE, Vec2::new(200.0, 100.0)).is_none());
    }
}
