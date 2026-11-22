//! Frosted glass: see-through fills and a saturated copy of the design's canvas mesh under them.
//!
//! egui cannot blur what it already drew, so the glass uses a picture it knows: the canvas mesh of
//! the design ([`dodai_ui_core::mesh_pixels`]), which [`paint_backdrop`] draws stretched over the
//! window. A pane finds the part of it that lies under itself from where it sits in that window.
//! Without a backdrop the fills still turn see-through, so paint one (or draw your own picture
//! under the UI). Turn the glass on with the surface ([`set_surface`](crate::set_surface):
//! `frost` reaches containers and input fields).

use crate::paint::color32;
use dodai_ui_core::{Color, Glass, SurfaceClass, Theme};
use egui::epaint::Mesh;
use egui::{pos2, Color32, Rect, Shape, Ui};
use std::cell::RefCell;
use std::rc::Rc;

/// Vertices across and down the mesh drawn as the backdrop (one per few pixels of picture).
const BACKDROP_GRID: (usize, usize) = (48, 30);
/// Largest grid of the frost under one pane.
const PANE_GRID: usize = 12;

struct Backdrop {
    /// The window rectangle the picture is stretched over.
    rect: Rect,
    id: String,
    pixels: Rc<Vec<u8>>,
}

thread_local! {
    static BACKDROP: RefCell<Option<Backdrop>> = const { RefCell::new(None) };
}

fn mesh_of(theme: &Theme) -> Option<Rc<Vec<u8>>> {
    let s = &theme.surface_style;
    if s.backdrop <= 0.0 {
        return None;
    }
    let (w, h) = dodai_ui_core::MESH_SIZE;
    Some(Rc::new(dodai_ui_core::mesh_pixels(
        theme.palette.background,
        s.wash,
        s.backdrop,
        w,
        h,
    )))
}

/// The app root: a central panel with the design's canvas mesh painted behind `add_contents`.
/// This is the default way to start a frame, so a design with a gradient canvas (Dodai) always
/// shows it.
pub fn root<R>(
    ui: &mut Ui,
    theme: &Theme,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    egui::CentralPanel::default().show(ui, |ui| {
        paint_backdrop(ui, theme, ui.ctx().content_rect());
        add_contents(ui)
    })
}

/// Draw the design's canvas mesh over `rect` (normally the whole window, as the first thing a
/// frame paints) and remember it as the picture panes frost. Does nothing for a design with a
/// flat canvas.
pub fn paint_backdrop(ui: &Ui, theme: &Theme, rect: Rect) {
    let pixels = BACKDROP.with(|b| {
        let mut b = b.borrow_mut();
        let keep = b.as_ref().filter(|k| k.id == theme.id());
        match keep {
            Some(k) => Some(k.pixels.clone()),
            None => mesh_of(theme).inspect(|p| {
                *b = Some(Backdrop {
                    rect,
                    id: theme.id().to_string(),
                    pixels: p.clone(),
                });
            }),
        }
    });
    let Some(pixels) = pixels else {
        BACKDROP.with(|b| *b.borrow_mut() = None);
        return;
    };
    BACKDROP.with(|b| {
        if let Some(b) = b.borrow_mut().as_mut() {
            b.rect = rect;
        }
    });
    let (gx, gy) = BACKDROP_GRID;
    let mut mesh = Mesh::default();
    for j in 0..=gy {
        for i in 0..=gx {
            let (u, v) = (i as f32 / gx as f32, j as f32 / gy as f32);
            let p = pos2(
                rect.min.x + rect.width() * u,
                rect.min.y + rect.height() * v,
            );
            mesh.colored_vertex(p, color32(sample(&pixels, u, v)));
        }
    }
    grid_triangles(&mut mesh, gx, gy);
    ui.painter().add(Shape::mesh(mesh));
}

fn grid_triangles(mesh: &mut Mesh, gx: usize, gy: usize) {
    let at = |i: usize, j: usize| (j * (gx + 1) + i) as u32;
    for j in 0..gy {
        for i in 0..gx {
            mesh.add_triangle(at(i, j), at(i + 1, j), at(i, j + 1));
            mesh.add_triangle(at(i + 1, j), at(i + 1, j + 1), at(i, j + 1));
        }
    }
}

/// The mesh colour at unit position `(u, v)`, bilinear.
fn sample(pixels: &[u8], u: f32, v: f32) -> Color {
    let (w, h) = dodai_ui_core::MESH_SIZE;
    let x = (u.clamp(0.0, 1.0) * (w - 1) as f32).max(0.0);
    let y = (v.clamp(0.0, 1.0) * (h - 1) as f32).max(0.0);
    let (x0, y0) = (x.floor() as usize, y.floor() as usize);
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let px = |x: usize, y: usize, c: usize| f32::from(pixels[(y * w + x) * 4 + c]);
    let ch = |c: usize| {
        let top = px(x0, y0, c) + (px(x1, y0, c) - px(x0, y0, c)) * fx;
        let bottom = px(x0, y1, c) + (px(x1, y1, c) - px(x0, y1, c)) * fx;
        (top + (bottom - top) * fy).round().clamp(0.0, 255.0) as u8
    };
    Color::rgba(ch(0), ch(1), ch(2), 255)
}

fn saturate(c: Color, s: f32) -> Color {
    let l = 0.2126 * f32::from(c.r) + 0.7152 * f32::from(c.g) + 0.0722 * f32::from(c.b);
    let f = |v: u8| (l + (f32::from(v) - l) * s).round().clamp(0.0, 255.0) as u8;
    Color::rgba(f(c.r), f(c.g), f(c.b), c.a)
}

/// The glass of a `class` surface under the app-wide surface.
pub fn glass_of(theme: &Theme, class: SurfaceClass) -> Glass {
    dodai_ui_core::surface_glass(
        theme,
        class,
        &dodai_ui_core::Surface::INHERIT,
        crate::resolved_surface(theme),
    )
}

/// The frosted picture under a pane at `rect`: the part of the backdrop that lies there,
/// saturated, at the glass's amount. Empty without glass or a backdrop.
pub fn frost_shapes(glass: &Glass, rect: Rect) -> Vec<Shape> {
    if !glass.frosted() || rect.width() < 1.0 || rect.height() < 1.0 {
        return Vec::new();
    }
    BACKDROP.with(|b| {
        let b = b.borrow();
        let Some(b) = b
            .as_ref()
            .filter(|b| b.rect.width() > 0.0 && b.rect.height() > 0.0)
        else {
            return Vec::new();
        };
        let n = |len: f32| ((len / 24.0).ceil() as usize).clamp(1, PANE_GRID);
        let (gx, gy) = (n(rect.width()), n(rect.height()));
        let mut mesh = Mesh::default();
        let alpha = (glass.amount * 255.0).round().clamp(0.0, 255.0) as u8;
        for j in 0..=gy {
            for i in 0..=gx {
                let p = pos2(
                    rect.min.x + rect.width() * i as f32 / gx as f32,
                    rect.min.y + rect.height() * j as f32 / gy as f32,
                );
                let u = (p.x - b.rect.min.x) / b.rect.width();
                let v = (p.y - b.rect.min.y) / b.rect.height();
                let c = saturate(sample(&b.pixels, u, v), glass.saturation);
                let c = Color32::from_rgba_unmultiplied(c.r, c.g, c.b, alpha);
                mesh.colored_vertex(p, c);
            }
        }
        grid_triangles(&mut mesh, gx, gy);
        vec![Shape::mesh(mesh)]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::rounded;
    use dodai_ui_core::{ResolvedSurface, SurfaceTargets};
    use egui::vec2;

    #[test]
    fn solid_without_the_mix_and_glass_with_it() {
        let t = rounded();
        crate::set_surface(ResolvedSurface::NONE);
        assert!(glass_of(&t, SurfaceClass::Panel).is_solid());
        crate::set_surface(ResolvedSurface {
            frost: 1.0,
            targets: SurfaceTargets::ALL,
            ..ResolvedSurface::NONE
        });
        let g = glass_of(&t, SurfaceClass::Panel);
        assert!(g.frosted());
        crate::set_surface(ResolvedSurface::NONE);
    }

    #[test]
    fn no_backdrop_no_frost_picture() {
        BACKDROP.with(|b| *b.borrow_mut() = None);
        let g = Glass {
            amount: 1.0,
            ..Glass::SOLID
        };
        let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 50.0));
        assert!(frost_shapes(&g, r).is_empty());
    }

    #[test]
    fn saturation_one_keeps_the_colour() {
        let c = Color::rgba(40, 90, 200, 255);
        assert_eq!(saturate(c, 1.0), c);
        let grey = saturate(c, 0.0);
        assert_eq!(grey.r, grey.g);
    }
}
