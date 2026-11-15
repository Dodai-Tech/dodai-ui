//! Surface textures and effects painted over a control's fill, clipped to its shape.
//!
//! [`set_surface_fx`] switches them for the whole app (like [`crate::set_scanlines`]); each
//! shaped surface then paints them through [`paint_surface`]. The effects themselves live in
//! `dodai-fx`; this file is the gpui [`Painter`], the app-wide state and the click ripples.

use crate::paint::poly;
use dodai_core::{Shape, SurfaceClass, SurfaceFx, Texture, Theme};
use dodai_fx::{Painter, Rgba, Ripple};
use gpui::{point, px, size, App, Bounds, Hsla, PathBuilder, Pixels, Window};
use std::cell::Cell;
use std::sync::OnceLock;
use std::time::Instant;

thread_local! {
    static FX: Cell<SurfaceFx> = const { Cell::new(SurfaceFx::NONE) };
}

/// Smallest side (px) of a surface that gets a texture or effect.
const MIN_SIDE: f32 = 16.0;

/// The surface mix currently applied to every shaped surface.
pub fn surface_fx() -> SurfaceFx {
    FX.with(Cell::get)
}

/// Set the mix without an [`App`], for tests of other modules.
#[cfg(test)]
pub(crate) fn set_for_test(fx: SurfaceFx) {
    FX.with(|c| c.set(fx));
}

/// Apply `fx` to every shaped surface and repaint.
pub fn set_surface_fx(cx: &mut App, fx: SurfaceFx) {
    FX.with(|c| c.set(fx));
    cx.refresh_windows();
}

/// Apply the active design's suggested mix (see `Theme::surface_style`).
pub fn apply_surface_preset(cx: &mut App) {
    let fx = crate::theme_of(cx).surface_preset();
    set_surface_fx(cx, fx);
}

pub use dodai_fx::SurfacePaint;

/// Resolve the surface paint of a `class` surface under the app-wide mix.
pub fn surface_paint(theme: &Theme, class: SurfaceClass) -> Option<SurfacePaint> {
    surface_paint_with(theme, class, &dodai_core::Surface::INHERIT)
}

/// Like [`surface_paint`], with a component's own [`Surface`](dodai_core::Surface) on top.
pub fn surface_paint_with(
    theme: &Theme,
    class: SurfaceClass,
    over: &dodai_core::Surface,
) -> Option<SurfacePaint> {
    dodai_core::surface_paint(theme, class, over, surface_fx())
}

/// An overlay that paints `texture` over its parent (which must be `relative`), in the
/// theme's shape: a live preview of the texture for pickers.
pub fn texture_swatch(theme: &Theme, texture: Texture) -> gpui::Canvas<()> {
    use gpui::{canvas, Styled};
    let sp = dodai_core::swatch_paint(theme, texture, surface_fx().neon);
    let shape = theme.shape.clone();
    canvas(
        |_, _, _| (),
        move |b, _, window, _| paint_surface(window, b, &shape, &sp),
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

fn clock() -> f32 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f32()
}

fn hsla_of(c: Rgba) -> Hsla {
    gpui::Rgba {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
    .into()
}

/// Draws the effects with gpui quads and paths.
struct GpuiPainter<'a> {
    window: &'a mut Window,
    origin: gpui::Point<Pixels>,
}

impl Painter for GpuiPainter<'_> {
    fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba) {
        self.window.paint_quad(gpui::fill(
            Bounds::new(
                point(self.origin.x + px(x), self.origin.y + px(y)),
                size(px(w), px(h)),
            ),
            hsla_of(c),
        ));
    }

    fn dot(&mut self, cx: f32, cy: f32, d: f32, c: Rgba) {
        let r = d / 2.0;
        self.window.paint_quad(
            gpui::fill(
                Bounds::new(
                    point(self.origin.x + px(cx - r), self.origin.y + px(cy - r)),
                    size(px(d), px(d)),
                ),
                hsla_of(c),
            )
            .corner_radii(px(r)),
        );
    }

    fn stroke(&mut self, pts: &[(f32, f32)], closed: bool, width: f32, c: Rgba) {
        let mut b = PathBuilder::stroke(px(width));
        if closed {
            b.add_polygon(&poly(self.origin, pts, (0.0, 0.0)), true);
        } else {
            let mut it = pts.iter();
            let Some(&(x0, y0)) = it.next() else {
                return;
            };
            b.move_to(point(self.origin.x + px(x0), self.origin.y + px(y0)));
            for &(x, y) in it {
                b.line_to(point(self.origin.x + px(x), self.origin.y + px(y)));
            }
        }
        if let Ok(p) = b.build() {
            self.window.paint_path(p, hsla_of(c));
        }
    }

    fn fill(&mut self, pts: &[(f32, f32)], c: Rgba) {
        let mut b = PathBuilder::fill();
        b.add_polygon(&poly(self.origin, pts, (0.0, 0.0)), true);
        if let Ok(p) = b.build() {
            self.window.paint_path(p, hsla_of(c));
        }
    }
}

/// Paint the texture and effects of `sp` into `bounds`, clipped to `shape`.
pub fn paint_surface(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    shape: &Shape,
    sp: &SurfacePaint,
) {
    let w: f32 = bounds.size.width.into();
    let h: f32 = bounds.size.height.into();
    if w.min(h) < MIN_SIDE {
        return;
    }
    let seed = {
        let (x, y): (f32, f32) = (bounds.origin.x.into(), bounds.origin.y.into());
        (x * 0.37) as i32 ^ ((y * 0.61) as i32).wrapping_mul(31)
    };
    let presses = if sp.fx.ripple > 0.0 && sp.pressable {
        presses(window, bounds, (w, h))
    } else {
        Vec::new()
    };
    let frame = dodai_fx::Frame {
        w,
        h,
        shape: &shape.clip(),
        seed,
        time: clock(),
        presses: &presses,
    };
    let mut out = GpuiPainter {
        window,
        origin: bounds.origin,
    };
    if dodai_fx::paint(&mut out, &frame, sp) {
        crate::motion::next_frame(window);
    }
}

/// One click on a surface, kept while its ripple plays.
struct Press {
    /// Size of the surface, to find it again on later frames.
    size: (f32, f32),
    /// Window position of the surface's top left corner when last painted.
    origin: (f32, f32),
    /// Touch point, relative to the surface.
    at: (f32, f32),
    /// `clock()` when pressed.
    t0: f32,
}

thread_local! {
    static PRESSES: std::cell::RefCell<Vec<Press>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// How long a click ripple plays, in seconds.
const PRESS_SECS: f32 = 0.7;

/// Material: listen for presses on this surface and play the click ripple of every live one.
fn presses(window: &mut Window, bounds: Bounds<Pixels>, (w, h): (f32, f32)) -> Vec<Ripple> {
    let (ox, oy): (f32, f32) = (bounds.origin.x.into(), bounds.origin.y.into());
    let size_key = (w, h);
    window.on_mouse_event(
        move |e: &gpui::MouseDownEvent, phase, window: &mut Window, _| {
            if phase == gpui::DispatchPhase::Bubble
                && e.button == gpui::MouseButton::Left
                && bounds.contains(&e.position)
            {
                let (px_, py_): (f32, f32) = (e.position.x.into(), e.position.y.into());
                PRESSES.with(|p| {
                    p.borrow_mut().push(Press {
                        size: size_key,
                        origin: (ox, oy),
                        at: (px_ - ox, py_ - oy),
                        t0: clock(),
                    });
                });
                window.refresh();
            }
        },
    );
    let now = clock();
    let live: Vec<(f32, f32, f32)> = PRESSES.with(|p| {
        let mut p = p.borrow_mut();
        p.retain(|r| now - r.t0 < PRESS_SECS);
        p.iter_mut()
            .filter(|r| {
                (r.size.0 - size_key.0).abs() < 1.0
                    && (r.size.1 - size_key.1).abs() < 1.0
                    // Controls lift a few px on hover, so allow a little drift.
                    && (r.origin.0 - ox).abs() < 8.0
                    && (r.origin.1 - oy).abs() < 8.0
            })
            .map(|r| {
                r.origin = (ox, oy);
                (r.at.0, r.at.1, (now - r.t0) / PRESS_SECS)
            })
            .collect()
    });
    // Debug: DODAI_PRESS_DEMO freezes a ripple mid-play, since screenshots cannot click.
    let live = if live.is_empty() && std::env::var_os("DODAI_PRESS_DEMO").is_some() {
        vec![(w * 0.3, h * 0.5, 0.35)]
    } else {
        live
    };
    live.into_iter()
        .map(|(x, y, t)| Ripple { at: (x, y), t })
        .collect()
}

/// Number of click ripples still playing (for tests).
#[doc(hidden)]
pub fn live_press_count() -> usize {
    PRESSES.with(|p| p.borrow().len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_drawn_for_the_default_mix() {
        set_surface_fx_for_test(SurfaceFx::NONE);
        assert!(surface_paint(
            &dodai_core::themes::cyberpunk::yellow(),
            SurfaceClass::Button
        )
        .is_none());
        set_surface_fx_for_test(SurfaceFx {
            glitch: 0.5,
            ..SurfaceFx::NONE
        });
        assert!(surface_paint(
            &dodai_core::themes::cyberpunk::yellow(),
            SurfaceClass::Button
        )
        .is_some());
        set_surface_fx_for_test(SurfaceFx::NONE);
    }

    #[test]
    fn a_component_surface_overrides_the_app_wide_mix() {
        use dodai_core::Surface;
        let t = dodai_core::themes::cyberpunk::yellow();
        let m = dodai_core::themes::material::light();
        set_surface_fx_for_test(SurfaceFx::NONE);
        let own = Surface::new().texture(Texture::Circuit);
        // Nothing app-wide, but this component asks for a texture (even on a Tag, which is
        // not under "Apply to").
        assert!(surface_paint_with(&t, SurfaceClass::Tag, &own).is_some());
        // A design draws only what it owns: Material has no circuit traces.
        assert!(surface_paint_with(&m, SurfaceClass::Tag, &own).is_none());
        // `Surface::NONE` opts one component out of an app-wide mix.
        set_surface_fx_for_test(SurfaceFx {
            texture: Texture::Grid,
            targets: dodai_core::SurfaceTargets::ALL,
            ..SurfaceFx::NONE
        });
        assert!(surface_paint(&t, SurfaceClass::Button).is_some());
        assert!(surface_paint_with(&t, SurfaceClass::Button, &Surface::NONE).is_none());
        // Setting one field keeps the rest of the app-wide mix.
        let p =
            surface_paint_with(&t, SurfaceClass::Button, &Surface::new().strength(0.9)).unwrap();
        assert_eq!((p.fx.texture, p.fx.strength), (Texture::Grid, 0.9));
        set_surface_fx_for_test(SurfaceFx::NONE);
    }

    fn set_surface_fx_for_test(fx: SurfaceFx) {
        FX.with(|c| c.set(fx));
    }
}
