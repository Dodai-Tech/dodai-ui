//! Surface textures and effects painted over a control's fill, clipped to its shape.
//!
//! [`set_surface_fx`] switches them for the whole app (like [`crate::set_scanlines`]); each
//! shaped surface then paints them through [`paint_surface`]. Everything is drawn from quads
//! and strokes, so it follows cut corners; nothing here looks at a theme id.

use crate::hsla;
use crate::paint::{poly, poly_span, row_span, scale_alpha};
use dodai_core::{Shape, ShapeKind, SurfaceClass, SurfaceFx, Texture, Theme};
use gpui::{point, px, size, App, Bounds, Hsla, PathBuilder, Pixels, Window};
use std::cell::Cell;
use std::sync::OnceLock;
use std::time::Instant;

thread_local! {
    static FX: Cell<SurfaceFx> = const { Cell::new(SurfaceFx::NONE) };
}

/// Upper bound on quads one surface may paint per frame.
const MAX_QUADS: f32 = 1800.0;
/// Smallest side (px) of a surface that gets a texture or effect.
const MIN_SIDE: f32 = 16.0;

/// The surface mix currently applied to every shaped surface.
pub fn surface_fx() -> SurfaceFx {
    FX.with(Cell::get)
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

/// The mix plus the colours resolved from a theme, carried by a `ShapeStyle`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfacePaint {
    /// What to draw.
    pub fx: SurfaceFx,
    /// Pattern colour (the text colour).
    pub ink: Hsla,
    /// Surface colour, for holes and dimming.
    pub hole: Hsla,
    /// First glitch ghost.
    pub a: Hsla,
    /// Second glitch ghost.
    pub b: Hsla,
    /// Sheen band.
    pub light: Hsla,
    /// Colour of lit lines and lamps.
    pub neon: Hsla,
    /// Timing and sizes from the theme.
    pub tune: dodai_core::SurfaceTuning,
}

impl SurfacePaint {
    /// Resolve colours for `theme`; `None` when nothing would be drawn on a `class` surface.
    pub fn of(theme: &Theme, class: SurfaceClass) -> Option<Self> {
        Self::of_with(theme, class, &dodai_core::Surface::INHERIT)
    }

    /// Like [`of`](Self::of), with a component's own [`Surface`](dodai_core::Surface) on top.
    pub fn of_with(theme: &Theme, class: SurfaceClass, over: &dodai_core::Surface) -> Option<Self> {
        if over.is_off() {
            return None;
        }
        let owned = (theme.surface_textures(), theme.surface_effects());
        let base = surface_fx().limited(owned.0, owned.1).for_class(class);
        let fx = over.over(base).limited(owned.0, owned.1);
        if fx.is_none() || !(fx.targets.contains(class) || over.forces_on()) {
            return None;
        }
        let p = &theme.palette;
        Some(Self {
            fx,
            ink: hsla(p.text),
            hole: hsla(p.background),
            a: hsla(p.link),
            b: hsla(p.danger),
            light: hsla(p.text),
            neon: hsla(p.accent),
            tune: theme.surface_style.tuning,
        })
    }
}

/// An overlay that paints `texture` over its parent (which must be `relative`), in the
/// theme's shape: a live preview of the texture for pickers.
pub fn texture_swatch(theme: &Theme, texture: Texture) -> gpui::Canvas<()> {
    use gpui::{canvas, Styled};
    let p = &theme.palette;
    let sp = SurfacePaint {
        fx: SurfaceFx {
            texture,
            strength: 0.8,
            scale: 1.0,
            neon: surface_fx().neon,
            ..SurfaceFx::NONE
        },
        ink: hsla(p.text),
        hole: hsla(p.background),
        a: hsla(p.link),
        b: hsla(p.danger),
        light: hsla(p.text),
        neon: hsla(p.accent),
        tune: theme.surface_style.tuning,
    };
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

/// Deterministic noise in `0..1` from two integers.
fn noise(a: i32, b: i32) -> f32 {
    let mut x = (a as u32).wrapping_mul(0x9E37_79B1) ^ (b as u32).wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    (x & 0xFFFF) as f32 / 65535.0
}

/// Pitch grown so that `count(pitch)` stays under [`MAX_QUADS`].
fn fit(pitch: f32, count: impl Fn(f32) -> f32) -> f32 {
    let n = count(pitch);
    if n <= MAX_QUADS {
        pitch
    } else {
        pitch * (n / MAX_QUADS).sqrt()
    }
}

struct Canvas<'a> {
    window: &'a mut Window,
    origin: gpui::Point<Pixels>,
    shape: &'a Shape,
    pts: Option<Vec<(f32, f32)>>,
    w: f32,
    h: f32,
    inset: f32,
    neon: f32,
    neon_c: Hsla,
    halo: f32,
}

impl Canvas<'_> {
    fn span(&self, y: f32) -> Option<(f32, f32)> {
        match &self.pts {
            Some(p) => poly_span(p, y.clamp(0.0, self.h)),
            None => row_span(self.shape, self.w, self.h, y.clamp(0.0, self.h)),
        }
    }

    /// Paint `x, y, w, h` clipped to the shape (and inside the border).
    fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, c: Hsla) {
        let (y0, y1) = (y.max(self.inset), (y + h).min(self.h - self.inset));
        if y1 <= y0 {
            return;
        }
        let (Some(a), Some(b)) = (self.span(y0), self.span(y1)) else {
            return;
        };
        let lo = a.0.max(b.0).max(self.inset);
        let hi = a.1.min(b.1).min(self.w - self.inset);
        let (x0, x1) = (x.max(lo), (x + w).min(hi));
        if x1 > x0 {
            self.window.paint_quad(gpui::fill(
                Bounds::new(
                    point(self.origin.x + px(x0), self.origin.y + px(y0)),
                    size(px(x1 - x0), px(y1 - y0)),
                ),
                c,
            ));
        }
    }

    /// A thin line segment: flat ink, or lit (halo plus bright core) when neon is on.
    fn line(&mut self, x: f32, y: f32, w: f32, h: f32, ink: Hsla) {
        if self.neon <= 0.0 {
            self.quad(x, y, w, h, ink);
            return;
        }
        let (n, c) = (self.neon, self.neon_c);
        let o = self.halo;
        self.quad(
            x - o,
            y - o,
            w + 2.0 * o,
            h + 2.0 * o,
            scale_alpha(c, 0.10 * n),
        );
        let i = o / 2.0;
        self.quad(
            x - i,
            y - i,
            w + 2.0 * i,
            h + 2.0 * i,
            scale_alpha(c, 0.22 * n),
        );
        self.quad(
            x,
            y,
            w,
            h,
            scale_alpha(c, (ink.a * 2.0).min(1.0) * (0.5 + 0.5 * n)),
        );
    }

    /// A round dot of diameter `d` centred at `cx, cy`, kept fully inside the shape.
    fn dot(&mut self, cx: f32, cy: f32, d: f32, c: Hsla) {
        let r = d / 2.0;
        let (Some(a), Some(b)) = (self.span(cy - r), self.span(cy + r)) else {
            return;
        };
        if cx - r < a.0.max(b.0).max(self.inset) || cx + r > a.1.min(b.1).min(self.w - self.inset) {
            return;
        }
        if cy - r < self.inset || cy + r > self.h - self.inset {
            return;
        }
        self.window.paint_quad(
            gpui::fill(
                Bounds::new(
                    point(self.origin.x + px(cx - r), self.origin.y + px(cy - r)),
                    size(px(d), px(d)),
                ),
                c,
            )
            .corner_radii(px(r)),
        );
    }

    fn outline(&mut self, dx: f32, width: f32, c: Hsla) {
        let pts = self
            .pts
            .clone()
            .unwrap_or_else(|| self.shape.outline(self.w, self.h));
        let mut b = PathBuilder::stroke(px(width));
        b.add_polygon(&poly(self.origin, &pts, (dx, 0.0)), true);
        if let Ok(p) = b.build() {
            self.window.paint_path(p, c);
        }
    }

    fn fill_shape(&mut self, c: Hsla) {
        let pts = self
            .pts
            .clone()
            .unwrap_or_else(|| self.shape.outline(self.w, self.h));
        let mut b = PathBuilder::fill();
        b.add_polygon(&poly(self.origin, &pts, (0.0, 0.0)), true);
        if let Ok(p) = b.build() {
            self.window.paint_path(p, c);
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
    let fx = sp.fx;
    let seed = {
        let (x, y): (f32, f32) = (bounds.origin.x.into(), bounds.origin.y.into());
        (x * 0.37) as i32 ^ ((y * 0.61) as i32).wrapping_mul(31)
    };
    let mut cv = Canvas {
        window,
        origin: bounds.origin,
        shape,
        pts: (shape.kind() == ShapeKind::Chamfered).then(|| shape.outline(w, h)),
        w,
        h,
        inset: shape.border_width,
        neon: fx.neon.clamp(0.0, 1.0),
        neon_c: sp.neon,
        halo: sp.tune.halo,
    };
    if fx.has_texture() {
        texture(&mut cv, sp, seed);
    }
    if fx.bloom > 0.0 {
        let n = 4;
        for k in (1..=n).rev() {
            let width = cv.inset + 2.0 * fx.bloom * sp.tune.bloom_spread * k as f32 / n as f32;
            cv.outline(0.0, width, scale_alpha(sp.a, 0.5 * fx.bloom / n as f32));
        }
    }
    if fx.sheen > 0.0 {
        sheen(&mut cv, sp);
    }
    if fx.flicker > 0.0 {
        let frame = (clock() * sp.tune.flicker_rate) as i32;
        if noise(frame, seed) < fx.flicker * sp.tune.flicker_chance {
            cv.fill_shape(scale_alpha(sp.hole, 0.28 * fx.flicker.max(0.3)));
        }
    }
    if fx.glitch > 0.0 {
        glitch(&mut cv, sp, seed);
    }
    if fx.animated() {
        cv.window.request_animation_frame();
    }
}

fn texture(cv: &mut Canvas, sp: &SurfacePaint, seed: i32) {
    let fx = sp.fx;
    let (w, h) = (cv.w, cv.h);
    let k = fx.scale.max(0.25) * sp.tune.cell;
    let s = fx.strength.clamp(0.0, 1.0);
    match fx.texture {
        Texture::None => {}
        Texture::Grain => {
            let c = fit((2.0 * k).max(2.0), |c| w * h / (c * c) * 0.25 * s.max(0.2));
            let (nx, ny) = ((w / c) as i32, (h / c) as i32);
            for iy in 0..ny {
                for ix in 0..nx {
                    let r = noise(ix + seed, iy);
                    if r < 0.25 * s.max(0.2) {
                        let a = 0.15 + 0.6 * noise(iy, ix + seed);
                        cv.quad(
                            ix as f32 * c,
                            iy as f32 * c,
                            c,
                            c,
                            scale_alpha(sp.ink, a * s),
                        );
                    }
                }
            }
        }
        Texture::Halftone => {
            let p = fit(6.0 * k, |p| w * h / (p * p));
            let rows = (h / p) as i32 + 1;
            let cols = (w / p) as i32 + 1;
            for iy in 0..rows {
                let t = iy as f32 / rows.max(1) as f32;
                let d = p * (0.12 + 0.62 * t * s.max(0.3));
                for ix in 0..cols {
                    let off = if iy % 2 == 0 { 0.0 } else { p / 2.0 };
                    cv.dot(
                        ix as f32 * p + off + p / 2.0,
                        iy as f32 * p + p / 2.0,
                        d,
                        scale_alpha(sp.ink, 0.5 * s + 0.1),
                    );
                }
            }
        }
        Texture::Dither => {
            let p = fit((2.0 * k).max(1.0), |p| w * h / (2.0 * p * p));
            let (nx, ny) = ((w / p) as i32 + 1, (h / p) as i32 + 1);
            for iy in 0..ny {
                for ix in 0..nx {
                    if (ix + iy) % 2 == 0 {
                        cv.quad(
                            ix as f32 * p,
                            iy as f32 * p,
                            p,
                            p,
                            scale_alpha(sp.ink, 0.22 * s),
                        );
                    }
                }
            }
        }
        Texture::Hatch => hatch(cv, sp, 8.0 * k, 1.0, 0.3 * s),
        Texture::Crosshatch => {
            hatch(cv, sp, 9.0 * k, 1.0, 0.24 * s);
            hatch(cv, sp, 9.0 * k, -1.0, 0.24 * s);
        }
        Texture::Grid => grid(cv, sp, 12.0 * k, 5, 0.18 * s, 0.4 * s),
        Texture::Blueprint => {
            grid(cv, sp, 8.0 * k, 4, 0.12 * s, 0.32 * s);
            // registration crosses in the corners
            let m = 8.0;
            for (x, y) in [(m, m), (w - m, m), (m, h - m), (w - m, h - m)] {
                cv.line(x - 4.0, y, 9.0, 1.0, scale_alpha(sp.ink, 0.5 * s));
                cv.line(x, y - 4.0, 1.0, 9.0, scale_alpha(sp.ink, 0.5 * s));
            }
        }
        Texture::DotGrid => {
            let p = fit(8.0 * k, |p| w * h / (p * p));
            let (nx, ny) = ((w / p) as i32, (h / p) as i32);
            for iy in 0..=ny {
                for ix in 0..=nx {
                    cv.dot(
                        ix as f32 * p + p / 2.0,
                        iy as f32 * p + p / 2.0,
                        1.5 + 0.5 * k.min(2.0),
                        scale_alpha(sp.ink, 0.55 * s),
                    );
                }
            }
        }
        Texture::Stripes => {
            let p = (18.0 * k).max(6.0);
            let step = fit(2.0, |st| (h / st) * (w / p));
            let mut y = 0.0;
            while y < h {
                let mut x = (-y).rem_euclid(p) - p;
                while x < w {
                    cv.quad(x, y, p / 2.0, step, scale_alpha(sp.ink, 0.14 * s));
                    x += p;
                }
                y += step;
            }
        }
        Texture::Checker => {
            let p = fit((10.0 * k).max(4.0), |p| w * h / (2.0 * p * p));
            let (nx, ny) = ((w / p) as i32 + 1, (h / p) as i32 + 1);
            for iy in 0..ny {
                for ix in 0..nx {
                    if (ix + iy) % 2 == 0 {
                        cv.quad(
                            ix as f32 * p,
                            iy as f32 * p,
                            p,
                            p,
                            scale_alpha(sp.ink, 0.1 * s),
                        );
                    }
                }
            }
        }
        Texture::Circuit => {
            let p = fit((14.0 * k).max(8.0), |p| w * h / (p * p) * 2.0);
            let (nx, ny) = ((w / p) as i32, (h / p) as i32);
            let a = scale_alpha(sp.ink, 0.28 * s);
            for iy in 0..=ny {
                for ix in 0..=nx {
                    let (x, y) = (ix as f32 * p, iy as f32 * p);
                    let r = noise(ix + seed, iy);
                    if r < 0.35 {
                        cv.line(x, y, p, 1.0, a);
                    } else if r < 0.7 {
                        cv.line(x, y, 1.0, p, a);
                    }
                    if noise(iy + 91, ix + seed) > 0.82 {
                        cv.dot(x + 0.5, y + 0.5, 4.0, scale_alpha(sp.ink, 0.45 * s));
                    }
                }
            }
        }
        Texture::DiamondPlate => {
            let p = fit((9.0 * k).max(5.0), |p| w * h / (p * p));
            let (nx, ny) = ((w / p) as i32 + 1, (h / p) as i32 + 1);
            for iy in 0..ny {
                for ix in 0..nx {
                    let off = if iy % 2 == 0 { 0.0 } else { p / 2.0 };
                    let (cx, cy) = (ix as f32 * p + off, iy as f32 * p);
                    let (a, b) = (scale_alpha(sp.ink, 0.22 * s), scale_alpha(sp.ink, 0.1 * s));
                    let u = p / 8.0;
                    cv.quad(cx - u, cy, 2.0 * u, u, a);
                    cv.quad(cx - 3.0 * u, cy + u, 6.0 * u, u, a);
                    cv.quad(cx - 3.0 * u, cy + 2.0 * u, 6.0 * u, u, b);
                    cv.quad(cx - u, cy + 3.0 * u, 2.0 * u, u, b);
                }
            }
        }
        Texture::Paper => {
            let n = ((w * h / 90.0) as i32).min(MAX_QUADS as i32 / 2);
            for i in 0..n {
                let (x, y) = (noise(i, seed) * w, noise(seed, i + 7) * h);
                let len = 2.0 + noise(i, 5) * 7.0 * k;
                let a = 0.06 + 0.18 * noise(i + 3, 11);
                cv.quad(x, y, len, 1.0, scale_alpha(sp.ink, a * s * 2.0));
            }
        }
        Texture::LedMatrix => {
            let p = fit((7.0 * k).max(4.0), |p| w * h / (p * p));
            let (nx, ny) = ((w / p) as i32, (h / p) as i32);
            for iy in 0..=ny {
                for ix in 0..=nx {
                    let lit = noise(ix + seed, iy + 13) > 0.72;
                    let c = if lit {
                        scale_alpha(sp.neon, 0.85 * s.max(0.4))
                    } else {
                        scale_alpha(sp.ink, 0.12 * s)
                    };
                    cv.dot(ix as f32 * p + p / 2.0, iy as f32 * p + p / 2.0, p * 0.5, c);
                }
            }
        }
        Texture::Barcode => {
            let bar_h = (10.0 * k).clamp(6.0, h * 0.3);
            let y = h - cv.inset - bar_h - 6.0;
            let mut x = 8.0;
            let end = (w * 0.45).min(w - 16.0);
            let mut i = 0;
            while x < end {
                let bw = 1.0 + (noise(i, seed) * 3.0).floor();
                cv.quad(x, y, bw, bar_h, scale_alpha(sp.ink, 0.5 * s));
                x += bw + 1.0 + (noise(seed, i + 1) * 2.0).floor();
                i += 1;
            }
        }
        Texture::Static => {
            let mut y = cv.inset;
            let mut i = 0;
            while y < h {
                let bh = 1.0 + (noise(i, seed) * 4.0 * k).floor();
                if noise(i + 5, seed) < 0.3 {
                    let len = w * (0.15 + 0.6 * noise(i, 9));
                    let xs = (w - len) * noise(i, 21);
                    cv.quad(
                        xs,
                        y,
                        len,
                        bh,
                        scale_alpha(sp.ink, (0.08 + 0.18 * noise(i, 4)) * s * 2.0),
                    );
                }
                y += bh + 2.0;
                i += 1;
            }
        }
        Texture::Brushed => {
            let p = fit((2.0 * k).max(1.5), |p| h / p * 2.0);
            let mut y = 0.0;
            let mut i = 0;
            while y < h {
                for seg in 0..3 {
                    let r = noise(i * 7 + seg + seed, i);
                    let len = w * (0.25 + 0.6 * noise(i, seg * 5 + seed));
                    let xs = (w - len) * r;
                    cv.quad(
                        xs,
                        y,
                        len,
                        1.0,
                        scale_alpha(sp.ink, (0.08 + 0.2 * noise(seg, i)) * s * 2.0),
                    );
                }
                y += p;
                i += 1;
            }
        }
        Texture::Carbon => {
            let p = fit((3.0 * k).max(2.0), |p| w * h / (p * p) / 2.0);
            let (nx, ny) = ((w / p) as i32 + 1, (h / p) as i32 + 1);
            for iy in 0..ny {
                for ix in 0..nx {
                    if (ix + iy) % 2 == 0 {
                        let a = if (ix / 2 + iy) % 2 == 0 { 0.26 } else { 0.12 };
                        cv.quad(
                            ix as f32 * p,
                            iy as f32 * p,
                            p,
                            p,
                            scale_alpha(sp.ink, a * s),
                        );
                    }
                }
            }
        }
        Texture::Perforation => {
            let p = fit(8.0 * k, |p| w * h / (p * p));
            let rows = (h / p) as i32 + 1;
            let cols = (w / p) as i32 + 1;
            for iy in 0..rows {
                for ix in 0..cols {
                    let off = if iy % 2 == 0 { 0.0 } else { p / 2.0 };
                    cv.dot(
                        ix as f32 * p + off + p / 2.0,
                        iy as f32 * p + p / 2.0,
                        p * 0.38,
                        scale_alpha(sp.hole, 0.35 + 0.5 * s),
                    );
                }
            }
        }
        Texture::Scanlines => {
            crate::paint::paint_scanlines(
                cv.window,
                Bounds::new(cv.origin, size(px(w), px(h))),
                cv.shape,
                scale_alpha(sp.ink, 0.22 * s),
                dodai_core::Effects::SCANLINE_PITCH * k,
            );
        }
    }
}

fn hatch(cv: &mut Canvas, sp: &SurfacePaint, pitch: f32, dir: f32, alpha: f32) {
    let (w, h) = (cv.w, cv.h);
    let p = pitch.max(3.0);
    let step = fit(2.0, |st| (h / st) * (w / p));
    let mut y = 0.0;
    while y < h {
        let base = if dir > 0.0 { -y } else { y };
        let mut x = base.rem_euclid(p);
        while x < w {
            cv.line(x, y, 1.5, step, scale_alpha(sp.ink, alpha));
            x += p;
        }
        y += step;
    }
}

fn grid(cv: &mut Canvas, sp: &SurfacePaint, pitch: f32, major: i32, minor_a: f32, major_a: f32) {
    let (w, h) = (cv.w, cv.h);
    let p = fit(pitch.max(4.0), |p| (w + h) / p * 3.0);
    let alpha = |i: i32| {
        scale_alpha(
            sp.ink,
            if i % major == major - 1 {
                major_a
            } else {
                minor_a
            },
        )
    };
    let (mut i, mut y) = (0, p);
    while y < h {
        cv.line(0.0, y, w, 1.0, alpha(i));
        y += p;
        i += 1;
    }
    let (mut i, mut x) = (0, p);
    while x < w {
        // vertical extent of the shape at this column
        let (mut top, mut bot, mut yy) = (None, 0.0, 0.0);
        while yy <= h {
            if cv.span(yy).is_some_and(|(lo, hi)| x >= lo && x <= hi) {
                top.get_or_insert(yy);
                bot = yy;
            }
            yy += 2.0;
        }
        if let Some(t) = top {
            cv.line(x, t, 1.0, bot - t + 2.0, alpha(i));
        }
        x += p;
        i += 1;
    }
}

fn sheen(cv: &mut Canvas, sp: &SurfacePaint) {
    let phase = (clock() / sp.tune.sheen_period.max(0.5)).fract();
    if phase > 0.45 {
        return;
    }
    let t = phase / 0.45;
    let (w, h) = (cv.w, cv.h);
    let band = (w * 0.16).max(12.0);
    let skew = 0.6;
    let cx = -band + (w + h * skew + 2.0 * band) * t;
    let mut y = 0.0;
    while y < h {
        let x = cx - (y - h / 2.0) * skew;
        cv.quad(x, y, band, 2.0, scale_alpha(sp.light, 0.14 * sp.fx.sheen));
        y += 2.0;
    }
}

fn glitch(cv: &mut Canvas, sp: &SurfacePaint, seed: i32) {
    let g = sp.fx.glitch.clamp(0.0, 1.0);
    let period = sp.tune.glitch_period.max(0.5);
    let burst = sp.tune.glitch_burst + 0.25 * g;
    let t = (clock() + noise(seed, 3) * period) % period;
    if t > burst {
        return;
    }
    let frame = (clock() * 24.0) as i32;
    let dx = 1.0 + 3.0 * g;
    cv.outline(-dx, 1.0, scale_alpha(sp.a, 0.7));
    cv.outline(dx, 1.0, scale_alpha(sp.b, 0.7));
    let (w, h) = (cv.w, cv.h);
    let slices = 2 + (g * 2.0) as i32;
    for i in 0..slices {
        let y = noise(frame, seed + i) * (h - 4.0);
        let th = 2.0 + noise(i, frame) * (4.0 + 6.0 * g);
        let off = (noise(frame + 1, seed + i) - 0.5) * 2.0 * (4.0 + 12.0 * g);
        let c = if i % 2 == 0 { sp.a } else { sp.b };
        cv.quad(off, y, w, th, scale_alpha(c, 0.35));
        cv.quad(-off, y + th, w * 0.4, 1.0, scale_alpha(sp.light, 0.4));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_stays_in_unit_range_and_varies() {
        let v: Vec<f32> = (0..200).map(|i| noise(i, 7)).collect();
        assert!(v.iter().all(|x| (0.0..=1.0).contains(x)));
        assert!(v.iter().any(|x| *x < 0.3) && v.iter().any(|x| *x > 0.7));
        assert_eq!(noise(3, 4), noise(3, 4));
    }

    #[test]
    fn fit_keeps_the_quad_count_bounded() {
        let count = |p: f32| 400.0 * 300.0 / (p * p);
        let p = fit(2.0, count);
        assert!(count(p) <= MAX_QUADS * 1.01, "{}", count(p));
        assert_eq!(fit(50.0, count), 50.0);
    }

    #[test]
    fn nothing_is_drawn_for_the_default_mix() {
        set_surface_fx_for_test(SurfaceFx::NONE);
        assert!(
            SurfacePaint::of(&dodai::themes::cyberpunk::yellow(), SurfaceClass::Button).is_none()
        );
        set_surface_fx_for_test(SurfaceFx {
            glitch: 0.5,
            ..SurfaceFx::NONE
        });
        assert!(
            SurfacePaint::of(&dodai::themes::cyberpunk::yellow(), SurfaceClass::Button).is_some()
        );
        set_surface_fx_for_test(SurfaceFx::NONE);
    }

    #[test]
    fn a_component_surface_overrides_the_app_wide_mix() {
        use dodai_core::Surface;
        let t = dodai::themes::cyberpunk::yellow();
        let m = dodai::themes::material::light();
        set_surface_fx_for_test(SurfaceFx::NONE);
        let own = Surface::new().texture(Texture::Grid);
        // Nothing app-wide, but this component asks for a texture (even on a Tag, which is
        // not under "Apply to").
        assert!(SurfacePaint::of_with(&t, SurfaceClass::Tag, &own).is_some());
        // A design draws only what it owns: Material owns no textures.
        assert!(SurfacePaint::of_with(&m, SurfaceClass::Tag, &own).is_none());
        // `Surface::NONE` opts one component out of an app-wide mix.
        set_surface_fx_for_test(SurfaceFx {
            texture: Texture::Grid,
            targets: dodai_core::SurfaceTargets::ALL,
            ..SurfaceFx::NONE
        });
        assert!(SurfacePaint::of(&t, SurfaceClass::Button).is_some());
        assert!(SurfacePaint::of_with(&t, SurfaceClass::Button, &Surface::NONE).is_none());
        // Setting one field keeps the rest of the app-wide mix.
        let p =
            SurfacePaint::of_with(&t, SurfaceClass::Button, &Surface::new().strength(0.9)).unwrap();
        assert_eq!((p.fx.texture, p.fx.strength), (Texture::Grid, 0.9));
        set_surface_fx_for_test(SurfaceFx::NONE);
    }

    fn set_surface_fx_for_test(fx: SurfaceFx) {
        FX.with(|c| c.set(fx));
    }
}
