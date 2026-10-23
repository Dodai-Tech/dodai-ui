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
    /// Colour of inner shadows.
    pub shade: Hsla,
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
            shade: hsla(p.roles.shadow),
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
        shade: hsla(p.roles.shadow),
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

/// The shape's outline polygon; rounded shapes get their corner arcs flattened too.
fn outline_of(shape: &Shape, w: f32, h: f32) -> Vec<(f32, f32)> {
    if shape.kind() != ShapeKind::Rounded {
        return shape.outline(w, h);
    }
    let r = shape.radius.min(w / 2.0).min(h / 2.0);
    let n = ((r / 2.0) as usize).clamp(3, 12);
    let mut p = Vec::with_capacity(4 * (n + 1));
    // Corner centres, clockwise from top-left, with the arc's start angle.
    for (cx, cy, a0) in [
        (r, r, 180.0_f32),
        (w - r, r, 270.0),
        (w - r, h - r, 0.0),
        (r, h - r, 90.0),
    ] {
        for i in 0..=n {
            let a = (a0 + 90.0 * i as f32 / n as f32).to_radians();
            p.push((cx + r * a.cos(), cy + r * a.sin()));
        }
    }
    p
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
    /// Texture fade towards the centre (`0` = even); set only while a texture is drawn.
    clear: f32,
}

impl Canvas<'_> {
    fn span(&self, y: f32) -> Option<(f32, f32)> {
        match &self.pts {
            Some(p) => poly_span(p, y.clamp(0.0, self.h)),
            None => row_span(self.shape, self.w, self.h, y.clamp(0.0, self.h)),
        }
    }

    /// Alpha factor for the texture at `x, y`: 1 at the edges, down to `1 - clear` in the middle.
    fn keep(&self, x: f32, y: f32) -> f32 {
        if self.clear <= 0.0 {
            return 1.0;
        }
        let nx = ((x - self.w / 2.0) / (self.w / 2.0).max(1.0)).abs();
        let ny = ((y - self.h / 2.0) / (self.h / 2.0).max(1.0)).abs();
        let t = ((nx.max(ny) - 0.3) / 0.7).clamp(0.0, 1.0);
        let t = t * t * (3.0 - 2.0 * t);
        1.0 - self.clear * (1.0 - t)
    }

    /// Paint `x, y, w, h` clipped to the shape (and inside the border).
    fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, c: Hsla) {
        let c = scale_alpha(c, self.keep(x + w / 2.0, y + h / 2.0));
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
        let c = scale_alpha(c, self.keep(cx, cy));
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
            .unwrap_or_else(|| outline_of(self.shape, self.w, self.h));
        let mut b = PathBuilder::stroke(px(width));
        b.add_polygon(&poly(self.origin, &pts, (dx, 0.0)), true);
        if let Ok(p) = b.build() {
            self.window.paint_path(p, c);
        }
    }

    /// The shape's outline pulled in by `d` px, stroked `width` px wide (inner glow and shadow).
    fn outline_inset(&mut self, d: f32, width: f32, c: Hsla) {
        let (w, h) = (self.w - 2.0 * d, self.h - 2.0 * d);
        if w < 4.0 || h < 4.0 {
            return;
        }
        let mut sh = self.shape.clone();
        sh.radius = (sh.radius - d).max(0.0);
        let pts = outline_of(&sh, w, h);
        let mut b = PathBuilder::stroke(px(width));
        b.add_polygon(&poly(self.origin, &pts, (d, d)), true);
        if let Ok(p) = b.build() {
            self.window.paint_path(p, c);
        }
    }

    /// An open polyline through `pts` (shape-local px).
    fn polyline(&mut self, pts: &[(f32, f32)], width: f32, c: Hsla) {
        let mut it = pts.iter();
        let Some(&(x0, y0)) = it.next() else {
            return;
        };
        let mut b = PathBuilder::stroke(px(width));
        b.move_to(point(self.origin.x + px(x0), self.origin.y + px(y0)));
        for &(x, y) in it {
            b.line_to(point(self.origin.x + px(x), self.origin.y + px(y)));
        }
        if let Ok(p) = b.build() {
            self.window.paint_path(p, c);
        }
    }

    /// A circle outline.
    fn ring(&mut self, cx: f32, cy: f32, r: f32, width: f32, c: Hsla) {
        let n = 48;
        let pts: Vec<(f32, f32)> = (0..=n)
            .map(|i| {
                let a = std::f32::consts::TAU * i as f32 / n as f32;
                (cx + r * a.cos(), cy + r * a.sin())
            })
            .collect();
        self.polyline(&pts, width, c);
    }

    /// A filled polygon (shape-local px), for strips and tape.
    fn polygon(&mut self, pts: &[(f32, f32)], c: Hsla) {
        let mut b = PathBuilder::fill();
        b.add_polygon(&poly(self.origin, pts, (0.0, 0.0)), true);
        if let Ok(p) = b.build() {
            self.window.paint_path(p, c);
        }
    }

    /// A circle outline clipped to the surface rectangle (inside the border).
    fn ring_clipped(&mut self, cx: f32, cy: f32, r: f32, width: f32, c: Hsla) {
        let n = ((r * 0.6) as usize).clamp(24, 96);
        let (x0, y0) = (self.inset + 1.0, self.inset + 1.0);
        let (x1, y1) = (self.w - self.inset - 1.0, self.h - self.inset - 1.0);
        let mut run: Vec<(f32, f32)> = Vec::new();
        for i in 0..=n {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            let (x, y) = (cx + r * a.cos(), cy + r * a.sin());
            if x >= x0 && x <= x1 && y >= y0 && y <= y1 {
                run.push((x, y));
            } else if !run.is_empty() {
                self.polyline(&run, width, c);
                run.clear();
            }
        }
        if run.len() > 1 {
            self.polyline(&run, width, c);
        }
    }

    fn fill_shape(&mut self, c: Hsla) {
        let pts = self
            .pts
            .clone()
            .unwrap_or_else(|| outline_of(self.shape, self.w, self.h));
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
        clear: 0.0,
    };
    if fx.tint > 0.0 {
        tint(&mut cv, sp);
    }
    if fx.has_texture() {
        cv.clear = fx.clear.clamp(0.0, 1.0);
        texture(&mut cv, sp, seed);
        cv.clear = 0.0;
    }
    if fx.depth > 0.0 {
        depth(&mut cv, sp);
    }
    if fx.edge > 0.0 {
        edge_light(&mut cv, sp);
    }
    if fx.brackets > 0.0 {
        brackets(&mut cv, sp);
    }
    if fx.pop > 0.0 {
        pop(&mut cv, sp);
    }
    if fx.gloss > 0.0 {
        gloss(&mut cv, sp);
    }
    if fx.tape > 0.0 {
        tape(&mut cv, sp);
    }
    if fx.ripple > 0.0 {
        ripple(&mut cv, sp);
    }
    if fx.strip > 0.0 {
        strip(&mut cv, sp);
    }
    if fx.pulse > 0.0 {
        pulse(&mut cv, sp);
    }
    if fx.chase > 0.0 {
        chase(&mut cv, sp, seed);
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
            let a = scale_alpha(sp.ink, 0.36 * s);
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
                        cv.dot(x + 0.5, y + 0.5, 4.0, scale_alpha(sp.ink, 0.55 * s));
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
            // Fine lines over the whole width, with a few brighter streaks.
            let p = fit((2.0 * k).max(1.5), |p| h / p * 2.0);
            let mut y = 0.0;
            let mut i = 0;
            while y < h {
                cv.quad(
                    0.0,
                    y,
                    w,
                    1.0,
                    scale_alpha(sp.ink, (0.04 + 0.08 * noise(i, seed)) * s),
                );
                let len = w * (0.15 + 0.35 * noise(i, seed + 3));
                let xs = (w - len) * noise(i * 7, seed);
                cv.quad(
                    xs,
                    y,
                    len,
                    1.0,
                    scale_alpha(sp.ink, (0.08 + 0.14 * noise(i, seed + 9)) * s),
                );
                y += p;
                i += 1;
            }
        }
        Texture::Carbon => {
            // A 2/2 twill: diagonal bands of tows passing over and under.
            let p = fit((3.0 * k).max(2.0), |p| w * h / (p * p));
            let (nx, ny) = ((w / p) as i32 + 1, (h / p) as i32 + 1);
            for iy in 0..ny {
                for ix in 0..nx {
                    let over = (ix - iy).rem_euclid(4) < 2;
                    let a = if over { 0.24 } else { 0.07 };
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
        Texture::Ripples => {
            // Soft rings spreading from the top-right corner, fading with distance.
            let p = (24.0 * k).max(10.0);
            // The rings die out after a fixed number: a corner flourish, not a full-panel pattern.
            let n = 14;
            for i in 1..=n {
                let f = 1.0 - i as f32 / (n + 1) as f32;
                cv.ring_clipped(
                    w * 0.92,
                    h * 0.12,
                    i as f32 * p,
                    1.5,
                    scale_alpha(sp.ink, 0.55 * s * f * f),
                );
            }
        }
        Texture::Zigzag => {
            let p = (12.0 * k).max(6.0);
            let amp = p * 0.3;
            let rows = ((h - 2.0 * (cv.inset + amp)) / p) as i32;
            let teeth = (w / p).ceil() as i32 + 1;
            let (x0, x1) = (cv.inset + 1.0, w - cv.inset - 1.0);
            for r in 0..=rows.max(0) {
                let y = cv.inset + amp + 1.0 + r as f32 * p;
                let pts: Vec<(f32, f32)> = (0..=teeth * 2)
                    .map(|i| {
                        let x = (i as f32 * p / 2.0).clamp(x0, x1);
                        (x, y + if i % 2 == 0 { -amp } else { amp })
                    })
                    .collect();
                cv.polyline(&pts, 2.0, scale_alpha(sp.ink, 0.35 * s));
            }
        }
        Texture::Houndstooth => {
            // A 4×4 houndstooth tile: stepped diagonal teeth.
            const TILE: [[u8; 4]; 4] = [[1, 1, 0, 0], [1, 1, 1, 0], [0, 0, 1, 1], [0, 1, 1, 1]];
            let p = fit((0.8 * k).max(1.0), |p| w * h / (p * p) * 0.6);
            let (nx, ny) = ((w / p) as i32 + 1, (h / p) as i32 + 1);
            for iy in 0..ny {
                for ix in 0..nx {
                    if TILE[(iy % 4) as usize][(ix % 4) as usize] == 1 {
                        cv.quad(
                            ix as f32 * p,
                            iy as f32 * p,
                            p,
                            p,
                            scale_alpha(sp.ink, 0.2 * s),
                        );
                    }
                }
            }
        }
        Texture::Hazard => {
            // Warning tape along the top and bottom edges only, so the middle stays clean.
            let band = (h * 0.2).clamp(4.0, 9.0 * k);
            let p = (10.0 * k).max(5.0);
            let step = fit(1.0, |st| (band * 2.0 / st) * (w / p));
            for (y0, flip) in [(cv.inset, false), (h - cv.inset - band, true)] {
                let mut y = 0.0;
                while y < band {
                    let base = if flip { y } else { -y };
                    let mut x = base.rem_euclid(p) - p;
                    while x < w {
                        cv.quad(x, y0 + y, p / 2.0, step, scale_alpha(sp.ink, 0.32 * s));
                        x += p;
                    }
                    y += step;
                }
            }
        }
        Texture::HexMesh => {
            let a = (7.0 * k).max(4.0);
            let (cw, rh) = (a * 1.732, a * 1.5);
            let rows = fit(rh, |r| (h / r) * (w / cw) * 2.0);
            let rows_n = (h / rows) as i32 + 1;
            let cols_n = (w / cw) as i32 + 1;
            for iy in 0..rows_n {
                let off = if iy % 2 == 0 { 0.0 } else { cw / 2.0 };
                let y = iy as f32 * rows;
                for ix in 0..cols_n {
                    let x = ix as f32 * cw + off;
                    cv.line(x, y, 1.0, a * 0.5, scale_alpha(sp.ink, 0.42 * s));
                    cv.dot(x + 0.5, y, 2.0, scale_alpha(sp.ink, 0.6 * s));
                    cv.quad(
                        x + 0.5,
                        y + a * 0.5,
                        cw / 2.0,
                        1.0,
                        scale_alpha(sp.ink, 0.24 * s),
                    );
                }
            }
        }
        Texture::Rain => {
            let n = ((w * h / 650.0 / k) as i32).min(MAX_QUADS as i32 / 3);
            for i in 0..n {
                let (x, y) = (noise(i, seed) * w, noise(seed, i + 3) * h);
                let len = (6.0 + noise(i, 8) * 14.0) * k;
                let a = 0.1 + 0.3 * noise(i + 1, 2);
                cv.line(x, y, 1.0, len, scale_alpha(sp.ink, a * s * 1.5));
            }
        }
        Texture::Grime => {
            let clusters = ((w * h / 1400.0) as i32).min(60);
            for i in 0..clusters {
                let (cx, cy) = (noise(i, seed) * w, noise(seed, i + 9) * h);
                for j in 0..7 {
                    let (dx, dy) = (
                        (noise(i, j + 40) - 0.5) * 14.0 * k,
                        (noise(j, i + 70) - 0.5) * 8.0 * k,
                    );
                    let sz = 1.5 + noise(i + j, 6) * 3.5 * k;
                    let a = 0.05 + 0.13 * noise(j + 11, i);
                    cv.quad(
                        cx + dx,
                        cy + dy,
                        sz,
                        sz * 0.7,
                        scale_alpha(sp.ink, a * s * 2.0),
                    );
                }
            }
        }
        Texture::DataStream => {
            let (px_, py) = (fit(5.0 * k, |p| w * h / (p * p * 1.5)), 6.0 * k);
            let (nx, ny) = ((w / px_) as i32, (h / py) as i32);
            for iy in 0..=ny {
                for ix in 0..=nx {
                    let r = noise(ix + seed, iy + 17);
                    if r > 0.55 {
                        let lit = r > 0.93;
                        let c = if lit {
                            scale_alpha(sp.neon, 0.8 * s.max(0.4))
                        } else {
                            scale_alpha(sp.ink, 0.2 * s)
                        };
                        cv.quad(ix as f32 * px_, iy as f32 * py, px_ * 0.6, py * 0.3, c);
                    }
                }
            }
        }
        Texture::GlitchBlocks => {
            let n = ((w * h / 2600.0) as i32).clamp(3, 40);
            for i in 0..n {
                let (x, y) = (noise(i, seed) * w, noise(seed, i + 5) * h);
                let (bw, bh) = ((8.0 + noise(i, 3) * 26.0) * k, 2.0 + noise(i, 4) * 3.0);
                let a = 0.1 + 0.22 * noise(i, 12);
                cv.quad(x, y, bw, bh, scale_alpha(sp.ink, a * s * 1.6));
                cv.quad(x + 2.0, y, bw, bh, scale_alpha(sp.a, 0.16 * s));
            }
        }
        Texture::ChipDie => {
            let a = scale_alpha(sp.ink, 0.42 * s);
            let (mx, my) = (w * 0.2, h * 0.2);
            let (x0, y0, x1, y1) = (mx, my, w - mx, h - my);
            cv.line(x0, y0, x1 - x0, 1.0, a);
            cv.line(x0, y1, x1 - x0, 1.0, a);
            cv.line(x0, y0, 1.0, y1 - y0, a);
            cv.line(x1, y0, 1.0, y1 - y0 + 1.0, a);
            let q = scale_alpha(sp.ink, 0.28 * s);
            let (ix, iy) = (w * 0.32, h * 0.32);
            cv.quad(ix, iy, w - 2.0 * ix, 1.0, q);
            cv.quad(ix, h - iy, w - 2.0 * ix, 1.0, q);
            cv.quad(ix, iy, 1.0, h - 2.0 * iy, q);
            cv.quad(w - ix, iy, 1.0, h - 2.0 * iy, q);
            let pin = fit((6.0 * k).max(4.0), |p| (w + h) / p * 2.0);
            let mut x = x0 + pin;
            while x < x1 {
                cv.quad(x, y0 - 4.0, 1.0, 4.0, a);
                cv.quad(x, y1, 1.0, 4.0, a);
                x += pin;
            }
            let mut y = y0 + pin;
            while y < y1 {
                cv.quad(x0 - 4.0, y, 4.0, 1.0, a);
                cv.quad(x1, y, 4.0, 1.0, a);
                y += pin;
            }
        }
        Texture::Wiring => {
            let p = fit((6.0 * k).max(4.0), |p| h / p * 6.0);
            let rows = (h / p) as i32;
            let a = scale_alpha(sp.ink, 0.26 * s);
            for iy in 0..=rows {
                let y = iy as f32 * p;
                let mut x = noise(iy, seed) * 20.0;
                let mut seg = 0;
                while x < w {
                    let len = 18.0 + noise(iy, seg + 30) * 50.0 * k;
                    if noise(iy + 50, seg + seed) > 0.25 {
                        cv.line(x, y, len, 1.0, a);
                    }
                    if noise(seg, iy + 80) > 0.6 {
                        cv.line(x + len, y, 1.0, p, a);
                    }
                    x += len + 4.0 + noise(seg, iy) * 10.0;
                    seg += 1;
                }
            }
        }
        Texture::Radar => {
            let (cx, cy) = (w / 2.0, h / 2.0);
            let m = w.min(h) / 2.0 - cv.inset - 2.0;
            if m > 6.0 {
                for f in [0.34, 0.67, 1.0] {
                    cv.ring(cx, cy, m * f, 1.0, scale_alpha(sp.ink, 0.24 * s));
                }
                cv.quad(cx - m, cy, 2.0 * m, 1.0, scale_alpha(sp.ink, 0.2 * s));
                cv.quad(cx, cy - m, 1.0, 2.0 * m, scale_alpha(sp.ink, 0.2 * s));
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

/// Inner shadow: dark rings pulled in from the edge, fading inward.
fn depth(cv: &mut Canvas, sp: &SurfacePaint) {
    let d = sp.fx.depth.clamp(0.0, 1.0);
    let n = 6;
    for k in 0..n {
        let f = 1.0 - k as f32 / n as f32;
        cv.outline_inset(
            cv.inset + 1.0 + 2.0 * k as f32,
            2.0,
            scale_alpha(sp.shade, 0.55 * d * f * f),
        );
    }
}

/// Material: a tonal wash of the accent, strongest at the top (the elevation tint).
fn tint(cv: &mut Canvas, sp: &SurfacePaint) {
    let t = sp.fx.tint.clamp(0.0, 1.0);
    let (w, h) = (cv.w, cv.h);
    let step = 3.0;
    let mut y = 0.0;
    while y < h {
        let f = 1.0 - y / h;
        cv.quad(
            0.0,
            y,
            w,
            step,
            scale_alpha(sp.neon, (0.02 + 0.10 * f * f) * t),
        );
        y += step;
    }
}

/// Material: rings of the state layer spreading from the centre and fading out.
fn ripple(cv: &mut Canvas, sp: &SurfacePaint) {
    let r = sp.fx.ripple.clamp(0.0, 1.0);
    let period = sp.tune.pulse_period.max(0.8);
    let reach = cv.w.max(cv.h) * 0.6;
    for k in 0..3 {
        let t = (clock() / period + k as f32 / 3.0).fract();
        let f = (1.0 - t) * (1.0 - t);
        cv.ring_clipped(
            cv.w / 2.0,
            cv.h / 2.0,
            reach * t,
            2.0,
            scale_alpha(sp.neon, 0.4 * r * f),
        );
    }
}

/// Neo-brutalism: a hard offset block along the bottom and right inner edges.
fn pop(cv: &mut Canvas, sp: &SurfacePaint) {
    let k = sp.fx.pop.clamp(0.0, 1.0);
    // Thinner on small controls so the block never touches a label.
    let t = if cv.h < 64.0 {
        2.0 + 1.5 * k
    } else {
        3.0 + 4.0 * k
    };
    let (w, h, i) = (cv.w, cv.h, cv.inset);
    let c = scale_alpha(sp.ink, 0.9);
    cv.quad(i + t, h - i - t, w - 2.0 * i - t, t, c);
    cv.quad(w - i - t, i + t, t, h - 2.0 * i - 2.0 * t, c);
}

/// Neo-brutalism: translucent masking tape across the top-left and bottom-right corners.
fn tape(cv: &mut Canvas, sp: &SurfacePaint) {
    let k = sp.fx.tape.clamp(0.0, 1.0);
    let (w, h, i) = (cv.w, cv.h, cv.inset);
    // Tape is for containers; on a button it would sit over the label.
    if h < 64.0 {
        return;
    }
    let l = (w.min(h) * 0.22).clamp(14.0, 30.0);
    let t = l * 0.45;
    let c = scale_alpha(sp.neon, 0.75 * k);
    let edge = scale_alpha(sp.ink, 0.35 * k);
    let band = |cv: &mut Canvas, a: (f32, f32), b: (f32, f32), d: (f32, f32)| {
        let q = [a, b, (b.0 + d.0, b.1 + d.1), (a.0 + d.0, a.1 + d.1)];
        cv.polygon(&q, c);
        cv.polyline(&[q[0], q[1]], 1.0, edge);
        cv.polyline(&[q[2], q[3]], 1.0, edge);
    };
    band(cv, (i, i + l), (i + l, i), (-0.0, t));
    band(cv, (w - i - l, h - i), (w - i, h - i - l), (0.0, -t));
}

/// Aurora: a clear-coat reflection across the top, cut on a slant like a windscreen glint.
fn gloss(cv: &mut Canvas, sp: &SurfacePaint) {
    let g = sp.fx.gloss.clamp(0.0, 1.0);
    let (w, h) = (cv.w, cv.h);
    // A reflection is always lighter than the surface, so it brightens the background tone.
    let lit = Hsla {
        s: 0.0,
        l: 1.0,
        a: 1.0,
        ..sp.hole
    };
    let band = h * 0.5;
    let step = 1.0;
    let mut y = 0.0;
    while y < band {
        let t = y / band;
        let len = w * (1.0 - 0.45 * t);
        cv.quad(
            0.0,
            y,
            len,
            step,
            scale_alpha(lit, 0.45 * g * (1.0 - t) * (1.0 - t)),
        );
        y += step;
    }
    cv.quad(0.0, cv.inset, w, 1.0, scale_alpha(lit, 0.6 * g));
}

/// Aurora: a thin line of light all around the border, following every corner, with a
/// brighter spot that travels along it and a tail that fades out behind.
fn strip(cv: &mut Canvas, sp: &SurfacePaint) {
    let s = sp.fx.strip.clamp(0.0, 1.0);
    let d = cv.inset + 1.0;
    let (w, h) = (cv.w - 2.0 * d, cv.h - 2.0 * d);
    if w < 8.0 || h < 8.0 {
        return;
    }
    let mut sh = cv.shape.clone();
    sh.radius = (sh.radius - d).max(0.0);
    let mut pts: Vec<(f32, f32)> = outline_of(&sh, w, h)
        .into_iter()
        .map(|(x, y)| (x + d, y + d))
        .collect();
    pts.push(pts[0]);
    let mut cum = vec![0.0_f32];
    for k in 1..pts.len() {
        let (a, b) = (pts[k - 1], pts[k]);
        cum.push(cum[k - 1] + ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt());
    }
    let total = cum[cum.len() - 1];
    // Point at distance `t` along the closed outline.
    let at = |t: f32| -> (f32, f32) {
        let t = t.rem_euclid(total);
        let k = cum.partition_point(|&c| c <= t).clamp(1, cum.len() - 1);
        let f = (t - cum[k - 1]) / (cum[k] - cum[k - 1]).max(1e-3);
        (
            pts[k - 1].0 + (pts[k].0 - pts[k - 1].0) * f,
            pts[k - 1].1 + (pts[k].1 - pts[k - 1].1) * f,
        )
    };
    // The whole border carries a thin line of light, so it reads as one outline all around.
    cv.polyline(&pts, 1.5, scale_alpha(sp.neon, 0.4 * s));
    let len = (total * 0.28).clamp(40.0, 260.0);
    let head = (clock() / sp.tune.sheen_period.max(0.8)).fract() * total;
    let n = 24;
    let seg = len / n as f32;
    for j in 0..n {
        // The head is bright and the tail fades.
        let f = (j + 1) as f32 / n as f32;
        let t0 = head - len + j as f32 * seg;
        let run: Vec<(f32, f32)> = (0..=2).map(|q| at(t0 + seg * q as f32 / 2.0)).collect();
        cv.polyline(&run, 3.5, scale_alpha(sp.neon, 0.18 * s * f * f));
        cv.polyline(&run, 1.5, scale_alpha(sp.neon, 0.95 * s * f * f));
    }
}

/// Light falling inward from the edges, in the neon colour.
fn edge_light(cv: &mut Canvas, sp: &SurfacePaint) {
    let e = sp.fx.edge.clamp(0.0, 1.0);
    let n = 5;
    for k in 0..n {
        let f = 1.0 - k as f32 / n as f32;
        cv.outline_inset(
            cv.inset + 0.5 + 1.6 * k as f32,
            1.6,
            scale_alpha(sp.neon, 0.5 * e * f * f),
        );
    }
}

/// Bracket marks in the four corners (kept clear of cut corners).
fn brackets(cv: &mut Canvas, sp: &SurfacePaint) {
    let b = sp.fx.brackets.clamp(0.0, 1.0);
    let (w, h) = (cv.w, cv.h);
    let arm = (w.min(h) * 0.22).clamp(5.0, 12.0);
    let m = cv.inset + 3.0;
    let c = scale_alpha(sp.neon, 0.9 * b);
    let left = |cv: &Canvas, y: f32| cv.span(y).map_or(m, |s| s.0.max(0.0) + m);
    let right = |cv: &Canvas, y: f32| cv.span(y).map_or(w - m, |s| s.1.min(w) - m);
    let tl = left(cv, m).max(left(cv, m + arm));
    let tr = right(cv, m).min(right(cv, m + arm));
    let bl = left(cv, h - m).max(left(cv, h - m - arm));
    let br = right(cv, h - m).min(right(cv, h - m - arm));
    cv.quad(tl, m, arm, 1.5, c);
    cv.quad(tl, m, 1.5, arm, c);
    cv.quad(tr - arm, m, arm, 1.5, c);
    cv.quad(tr - 1.5, m, 1.5, arm, c);
    cv.quad(bl, h - m - 1.5, arm, 1.5, c);
    cv.quad(bl, h - m - arm, 1.5, arm, c);
    cv.quad(br - arm, h - m - 1.5, arm, 1.5, c);
    cv.quad(br - 1.5, h - m - arm, 1.5, arm, c);
}

/// Breathing light along the border.
fn pulse(cv: &mut Canvas, sp: &SurfacePaint) {
    let p = sp.fx.pulse.clamp(0.0, 1.0);
    let phase = clock() / sp.tune.pulse_period.max(0.4) * std::f32::consts::TAU;
    let a = 0.5 + 0.5 * phase.sin();
    let d = cv.inset * 0.5;
    cv.outline_inset(d, cv.inset + 4.0, scale_alpha(sp.neon, 0.12 * p * a));
    cv.outline_inset(
        d,
        cv.inset + 1.0,
        scale_alpha(sp.neon, (0.15 + 0.7 * a) * p),
    );
}

/// A bright segment (with a fading tail) running around the border.
fn chase(cv: &mut Canvas, sp: &SurfacePaint, seed: i32) {
    let k = sp.fx.chase.clamp(0.0, 1.0);
    let d = cv.inset * 0.5;
    let (w, h) = (cv.w - 2.0 * d, cv.h - 2.0 * d);
    if w < 4.0 || h < 4.0 {
        return;
    }
    let pts: Vec<(f32, f32)> = cv
        .shape
        .outline(w, h)
        .into_iter()
        .map(|(x, y)| (x + d, y + d))
        .collect();
    let n = pts.len();
    let mut cum = vec![0.0_f32];
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        cum.push(cum[i] + ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt());
    }
    let total = cum[n].max(1.0);
    let at = |s: f32| {
        let s = s.rem_euclid(total);
        let i = cum.partition_point(|c| *c <= s).clamp(1, n) - 1;
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        let t = (s - cum[i]) / (cum[i + 1] - cum[i]).max(1e-3);
        (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
    };
    let head = ((clock() / sp.tune.chase_period.max(0.5) + noise(seed, 5)).fract()) * total;
    let len = (total * 0.16).max(18.0);
    let parts = 3;
    for j in 0..parts {
        let (s0, s1) = (
            head - len * (j + 1) as f32 / parts as f32,
            head - len * j as f32 / parts as f32,
        );
        let seg: Vec<(f32, f32)> = (0..=6)
            .map(|i| at(s0 + (s1 - s0) * i as f32 / 6.0))
            .collect();
        let f = (parts - j) as f32 / parts as f32;
        cv.polyline(&seg, cv.inset + 3.0, scale_alpha(sp.neon, 0.15 * k * f));
        cv.polyline(&seg, cv.inset + 0.5, scale_alpha(sp.neon, 0.9 * k * f * f));
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
        let own = Surface::new().texture(Texture::Circuit);
        // Nothing app-wide, but this component asks for a texture (even on a Tag, which is
        // not under "Apply to").
        assert!(SurfacePaint::of_with(&t, SurfaceClass::Tag, &own).is_some());
        // A design draws only what it owns: Material has no circuit traces.
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
