//! Surface textures and light effects, drawn as egui shapes over a control's fill.
//!
//! The same mix as `dodai-gpui` ([`SurfaceFx`]): set it for the whole app with [`set_surface_fx`],
//! and each [`box_shapes_for`](crate::paint::box_shapes_for) call paints it on the surfaces it
//! targets. A design draws only the textures and effects it owns. Textures are procedural
//! lines, dots and quads kept inside the control's rectangle; the less common ones draw as
//! grain. Glitch, flicker and the lit lines of textures are not drawn here.

use crate::paint::color32;
use dodai_core::{EffectSet, SurfaceClass, SurfaceFx, Texture, Theme};
use egui::{pos2, vec2, Color32, Pos2, Rect, Shape, Stroke};
use std::cell::Cell;
use std::sync::OnceLock;
use std::time::Instant;

thread_local! {
    static FX: Cell<SurfaceFx> = const { Cell::new(SurfaceFx::NONE) };
    static ANIMATED: Cell<bool> = const { Cell::new(false) };
}

/// The surface mix applied to every shaped surface.
pub fn surface_fx() -> SurfaceFx {
    FX.with(Cell::get)
}

/// Apply `fx` to every shaped surface (repaint with `ctx.request_repaint()`).
pub fn set_surface_fx(fx: SurfaceFx) {
    FX.with(|c| c.set(fx));
}

/// Whether a surface painted since the last call animates, so the frame should be repainted.
pub fn take_animated() -> bool {
    ANIMATED.with(|a| a.replace(false))
}

/// Upper bound on shapes one surface may paint.
const MAX_SHAPES: f32 = 900.0;
/// Smallest side (px) of a surface that gets a texture.
const MIN_SIDE: f32 = 16.0;

fn clock() -> f32 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f32()
}

fn noise(a: i32, b: i32) -> f32 {
    let mut x = (a as u32).wrapping_mul(0x9E37_79B1) ^ (b as u32).wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xffff) as f32 / 65535.0
}

fn with_alpha(c: Color32, a: f32) -> Color32 {
    let a = (c.a() as f32 * a.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

/// Relative luminance in 0..1 of a token colour.
fn luma(c: dodai_core::Color) -> f32 {
    (0.2126 * c.r as f32 + 0.7152 * c.g as f32 + 0.0722 * c.b as f32) / 255.0
}

/// Clip the segment `a`-`b` to `r` (Liang–Barsky).
fn clip(a: Pos2, b: Pos2, r: Rect) -> Option<(Pos2, Pos2)> {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let (mut t0, mut t1) = (0.0f32, 1.0f32);
    for (p, q) in [
        (-dx, a.x - r.min.x),
        (dx, r.max.x - a.x),
        (-dy, a.y - r.min.y),
        (dy, r.max.y - a.y),
    ] {
        if p.abs() < 1e-6 {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
        }
    }
    (t0 < t1).then(|| {
        (
            pos2(a.x + t0 * dx, a.y + t0 * dy),
            pos2(a.x + t1 * dx, a.y + t1 * dy),
        )
    })
}

/// Sutherland–Hodgman: `poly` (convex) clipped to `r`.
fn clip_poly(poly: Vec<Pos2>, r: Rect) -> Vec<Pos2> {
    type Edge = (fn(Pos2, Rect) -> bool, fn(Pos2, Pos2, Rect) -> Pos2);
    let edges: [Edge; 4] = [
        (
            |p, r| p.x >= r.min.x,
            |a, b, r| {
                let t = (r.min.x - a.x) / (b.x - a.x);
                pos2(r.min.x, a.y + t * (b.y - a.y))
            },
        ),
        (
            |p, r| p.x <= r.max.x,
            |a, b, r| {
                let t = (r.max.x - a.x) / (b.x - a.x);
                pos2(r.max.x, a.y + t * (b.y - a.y))
            },
        ),
        (
            |p, r| p.y >= r.min.y,
            |a, b, r| {
                let t = (r.min.y - a.y) / (b.y - a.y);
                pos2(a.x + t * (b.x - a.x), r.min.y)
            },
        ),
        (
            |p, r| p.y <= r.max.y,
            |a, b, r| {
                let t = (r.max.y - a.y) / (b.y - a.y);
                pos2(a.x + t * (b.x - a.x), r.max.y)
            },
        ),
    ];
    let mut poly = poly;
    for (inside, cut) in edges {
        let input = std::mem::take(&mut poly);
        for i in 0..input.len() {
            let (a, b) = (input[i], input[(i + 1) % input.len()]);
            match (inside(a, r), inside(b, r)) {
                (true, true) => poly.push(b),
                (true, false) => poly.push(cut(a, b, r)),
                (false, true) => {
                    poly.push(cut(a, b, r));
                    poly.push(b);
                }
                (false, false) => {}
            }
        }
        if poly.is_empty() {
            break;
        }
    }
    poly
}

struct Pen {
    rect: Rect,
    out: Vec<Shape>,
}

impl Pen {
    fn line(&mut self, a: Pos2, b: Pos2, w: f32, c: Color32) {
        if let Some((a, b)) = clip(a, b, self.rect) {
            self.out
                .push(Shape::line_segment([a, b], Stroke::new(w, c)));
        }
    }
    fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, c: Color32) {
        let r = Rect::from_min_size(pos2(x, y), vec2(w, h)).intersect(self.rect);
        if r.width() > 0.0 && r.height() > 0.0 {
            self.out.push(Shape::rect_filled(r, 0.0, c));
        }
    }
    fn dot(&mut self, x: f32, y: f32, r: f32, c: Color32) {
        if self.rect.shrink(r).contains(pos2(x, y)) {
            self.out.push(Shape::circle_filled(pos2(x, y), r, c));
        }
    }
    /// Parallel bands at 45°, `bw` px wide every `p` px, clipped to the rectangle.
    fn bands(&mut self, p: f32, bw: f32, c: Color32) {
        let r = self.rect;
        let mut k = 0.0;
        while k < r.width() + r.height() {
            let poly = vec![
                pos2(r.min.x + k, r.min.y),
                pos2(r.min.x + k + bw, r.min.y),
                pos2(r.min.x + k + bw - r.height(), r.max.y),
                pos2(r.min.x + k - r.height(), r.max.y),
            ];
            let poly = clip_poly(poly, r);
            if poly.len() >= 3 {
                self.out.push(Shape::convex_polygon(poly, c, Stroke::NONE));
            }
            k += p;
        }
    }
    /// Parallel lines at 45° every `p` px, `down` = top-left to bottom-right.
    fn diagonals(&mut self, p: f32, w: f32, down: bool, c: Color32) {
        let r = self.rect;
        let span = r.width() + r.height();
        let mut k = 0.0;
        while k < span {
            let (a, b) = if down {
                (
                    pos2(r.min.x + k, r.min.y),
                    pos2(r.min.x + k - r.height(), r.max.y),
                )
            } else {
                (
                    pos2(r.min.x + k - r.height(), r.min.y),
                    pos2(r.min.x + k, r.max.y),
                )
            };
            self.line(a, b, w, c);
            k += p;
        }
    }
}

/// A cell pitch that keeps the shape count of `count(p)` within the budget.
fn fit(mut p: f32, count: impl Fn(f32) -> f32) -> f32 {
    while count(p) > MAX_SHAPES && p < 400.0 {
        p *= 1.25;
    }
    p
}

fn texture(pen: &mut Pen, tex: Texture, ink: Color32, fx: &SurfaceFx) {
    let r = pen.rect;
    let (w, h) = (r.width(), r.height());
    let s = fx.strength.clamp(0.0, 1.0);
    let sc = fx.scale.max(0.25);
    let area = |p: f32| w * h / (p * p);
    match tex {
        Texture::Halftone => {
            let p = fit(7.0 * sc, area);
            let (cols, rows) = ((w / p) as i32, (h / p) as i32);
            for j in 0..=rows {
                for i in 0..=cols {
                    let t = (i as f32 / cols.max(1) as f32 + j as f32 / rows.max(1) as f32) / 2.0;
                    pen.dot(
                        r.min.x + i as f32 * p,
                        r.min.y + j as f32 * p,
                        p * 0.42 * t,
                        with_alpha(ink, 0.55 * s),
                    );
                }
            }
        }
        Texture::DotGrid | Texture::Perforation | Texture::LedMatrix => {
            let big = tex == Texture::Perforation;
            let p = fit((if big { 9.0 } else { 8.0 }) * sc, area);
            let (cols, rows) = ((w / p) as i32, (h / p) as i32);
            for j in 0..=rows {
                for i in 0..=cols {
                    let off = if big && j % 2 == 1 { p / 2.0 } else { 0.0 };
                    pen.dot(
                        r.min.x + i as f32 * p + off,
                        r.min.y + j as f32 * p,
                        if big { p * 0.22 } else { 1.0 },
                        with_alpha(ink, 0.5 * s),
                    );
                }
            }
        }
        Texture::Hatch | Texture::Crosshatch => {
            let p = fit(6.0 * sc, |p| (w + h) / p * 2.0);
            pen.diagonals(p, 1.0, true, with_alpha(ink, 0.4 * s));
            if tex == Texture::Crosshatch {
                pen.diagonals(p, 1.0, false, with_alpha(ink, 0.4 * s));
            }
        }
        Texture::Stripes | Texture::Hazard => {
            let wide = tex == Texture::Hazard;
            let p = fit((if wide { 16.0 } else { 12.0 }) * sc, |p| (w + h) / p);
            pen.bands(
                p,
                p * 0.5,
                with_alpha(ink, (if wide { 0.5 } else { 0.35 }) * s),
            );
        }
        Texture::Grid | Texture::Blueprint => {
            let p = fit(12.0 * sc, |p| (w + h) / p);
            let (mut x, mut i) = (r.min.x, 0);
            while x <= r.max.x {
                let heavy = tex == Texture::Blueprint && i % 5 == 0;
                pen.line(
                    pos2(x, r.min.y),
                    pos2(x, r.max.y),
                    1.0,
                    with_alpha(ink, (if heavy { 0.5 } else { 0.25 }) * s),
                );
                x += p;
                i += 1;
            }
            let (mut y, mut j) = (r.min.y, 0);
            while y <= r.max.y {
                let heavy = tex == Texture::Blueprint && j % 5 == 0;
                pen.line(
                    pos2(r.min.x, y),
                    pos2(r.max.x, y),
                    1.0,
                    with_alpha(ink, (if heavy { 0.5 } else { 0.25 }) * s),
                );
                y += p;
                j += 1;
            }
        }
        Texture::Checker => {
            let p = fit(14.0 * sc, area);
            let (cols, rows) = ((w / p).ceil() as i32, (h / p).ceil() as i32);
            for j in 0..rows {
                for i in 0..cols {
                    if (i + j) % 2 == 0 {
                        pen.quad(
                            r.min.x + i as f32 * p,
                            r.min.y + j as f32 * p,
                            p,
                            p,
                            with_alpha(ink, 0.22 * s),
                        );
                    }
                }
            }
        }
        Texture::Carbon => {
            let p = fit(4.0 * sc, area);
            let (cols, rows) = ((w / p).ceil() as i32, (h / p).ceil() as i32);
            for j in 0..rows {
                for i in 0..cols {
                    let a = if (i + j) % 2 == 0 { 0.32 } else { 0.1 };
                    pen.quad(
                        r.min.x + i as f32 * p,
                        r.min.y + j as f32 * p,
                        p - 0.5,
                        p - 0.5,
                        with_alpha(ink, a * s),
                    );
                }
            }
        }
        Texture::Scanlines => {
            let p = fit(3.0 * sc, |p| h / p);
            let mut y = r.min.y;
            while y < r.max.y {
                pen.line(
                    pos2(r.min.x, y),
                    pos2(r.max.x, y),
                    1.0,
                    with_alpha(ink, 0.3 * s),
                );
                y += p;
            }
        }
        Texture::Brushed => {
            let p = fit(2.0 * sc, |p| h / p);
            let (mut y, mut j) = (r.min.y, 0);
            while y < r.max.y {
                let n = noise(j, 3);
                let x0 = r.min.x + n * w * 0.3;
                pen.line(
                    pos2(x0, y),
                    pos2(x0 + w * (0.4 + 0.6 * noise(j, 5)), y),
                    1.0,
                    with_alpha(ink, (0.12 + 0.3 * n) * s),
                );
                y += p;
                j += 1;
            }
        }
        // Everything else (grain, paper, dither, static, grime, ...) reads as fine grain.
        _ => {
            let p = fit(2.5 * sc, area);
            let (cols, rows) = ((w / p) as i32, (h / p) as i32);
            for j in 0..rows {
                for i in 0..cols {
                    let n = noise(i, j);
                    if n > 0.55 {
                        pen.quad(
                            r.min.x + i as f32 * p,
                            r.min.y + j as f32 * p,
                            p * 0.7,
                            p * 0.7,
                            with_alpha(ink, (n - 0.4) * 0.9 * s),
                        );
                    }
                }
            }
        }
    }
}

/// Light falling inward from the edges: `steps` fading lines per side.
fn edges(pen: &mut Pen, c: Color32, amount: f32, sides: [bool; 4]) {
    let r = pen.rect;
    let depth = (r.width().min(r.height()) * 0.18).clamp(3.0, 14.0);
    let steps = 6;
    for k in 0..steps {
        let f = k as f32 / steps as f32;
        let a = (1.0 - f).powi(2) * amount * 0.5;
        let d = f * depth;
        let col = with_alpha(c, a);
        if sides[0] {
            pen.line(
                pos2(r.min.x, r.min.y + d),
                pos2(r.max.x, r.min.y + d),
                1.0,
                col,
            );
        }
        if sides[1] {
            pen.line(
                pos2(r.max.x - d, r.min.y),
                pos2(r.max.x - d, r.max.y),
                1.0,
                col,
            );
        }
        if sides[2] {
            pen.line(
                pos2(r.min.x, r.max.y - d),
                pos2(r.max.x, r.max.y - d),
                1.0,
                col,
            );
        }
        if sides[3] {
            pen.line(
                pos2(r.min.x + d, r.min.y),
                pos2(r.min.x + d, r.max.y),
                1.0,
                col,
            );
        }
    }
}

/// The point at `t` (0..1) along the rectangle's border, clockwise from the top-left corner.
fn along(r: Rect, t: f32) -> Pos2 {
    let (w, h) = (r.width(), r.height());
    let mut d = t.rem_euclid(1.0) * 2.0 * (w + h);
    if d < w {
        return pos2(r.min.x + d, r.min.y);
    }
    d -= w;
    if d < h {
        return pos2(r.max.x, r.min.y + d);
    }
    d -= h;
    if d < w {
        return pos2(r.max.x - d, r.max.y);
    }
    d -= w;
    pos2(r.min.x, r.max.y - d)
}

/// The shapes of `fx` for a `class` surface over `rect` (empty when nothing applies).
pub fn surface_shapes(
    theme: &Theme,
    rect: Rect,
    class: SurfaceClass,
    fill: Option<dodai_core::Color>,
) -> Vec<Shape> {
    let owned: EffectSet = theme.surface_effects();
    let fx = surface_fx()
        .limited(theme.surface_textures(), owned)
        .for_class(class);
    if fx.is_none() || !fx.targets.contains(class) || rect.width().min(rect.height()) < MIN_SIDE {
        return Vec::new();
    }
    if fx.animated() {
        ANIMATED.with(|a| a.set(true));
    }
    let p = &theme.palette;
    // Ink that contrasts with the fill it sits on.
    let base = fill.unwrap_or(p.background);
    let ink = color32(if luma(base) > 0.5 {
        dark_of(theme)
    } else {
        light_of(theme)
    });
    let mut pen = Pen {
        rect,
        out: Vec::new(),
    };
    let t = clock();
    if fx.has_texture() {
        texture(&mut pen, fx.texture, ink, &fx);
    }
    if fx.depth > 0.0 {
        edges(
            &mut pen,
            color32(p.roles.shadow),
            fx.depth * 1.6,
            [true, false, false, true],
        );
    }
    if fx.edge > 0.0 {
        edges(&mut pen, color32(p.text), fx.edge, [true, true, true, true]);
    }
    if fx.sheen > 0.0 {
        let period = theme.surface_style.tuning.sheen_period.max(0.5);
        let phase = (t % period) / period;
        let band = rect.width() * 0.25;
        let x0 = rect.min.x - band + phase * (rect.width() + 2.0 * band);
        for k in 0..10 {
            let f = k as f32 / 10.0;
            let a = (1.0 - (f - 0.5).abs() * 2.0) * fx.sheen * 0.35;
            let x = x0 + f * band;
            pen.line(
                pos2(x + 4.0, rect.min.y),
                pos2(x - 4.0, rect.max.y),
                band / 10.0 + 0.5,
                with_alpha(color32(p.text), a),
            );
        }
    }
    if fx.pulse > 0.0 {
        let period = theme.surface_style.tuning.pulse_period.max(0.5);
        let a = (0.5 + 0.5 * (t * std::f32::consts::TAU / period).sin()) * fx.pulse;
        let c = color32(p.accent);
        for (i, k) in [0.0, 1.0].into_iter().enumerate() {
            let r = rect.shrink(k + 0.5);
            let q = [
                r.left_top(),
                r.right_top(),
                r.right_bottom(),
                r.left_bottom(),
                r.left_top(),
            ];
            pen.out.push(Shape::line(
                q.to_vec(),
                Stroke::new(1.0_f32, with_alpha(c, a * (1.0 - 0.5 * i as f32))),
            ));
        }
    }
    if fx.chase > 0.0 {
        let period = theme.surface_style.tuning.chase_period.max(0.5);
        let head = (t % period) / period;
        let c = color32(p.primary);
        let n = 14;
        for k in 0..n {
            let f = k as f32 / n as f32;
            let (a, b) = (
                along(rect.shrink(0.5), head - f * 0.12),
                along(rect.shrink(0.5), head - (f + 1.0 / n as f32) * 0.12),
            );
            pen.out.push(Shape::line_segment(
                [a, b],
                Stroke::new(1.5_f32, with_alpha(c, (1.0 - f) * fx.chase)),
            ));
        }
    }
    if fx.brackets > 0.0 {
        let l = (rect.width().min(rect.height()) * 0.28).clamp(4.0, 16.0);
        let c = with_alpha(color32(p.accent), fx.brackets);
        let r = rect.shrink(1.5);
        for (corner, dx, dy) in [
            (r.left_top(), 1.0, 1.0),
            (r.right_top(), -1.0, 1.0),
            (r.right_bottom(), -1.0, -1.0),
            (r.left_bottom(), 1.0, -1.0),
        ] {
            pen.out.push(Shape::line(
                vec![
                    pos2(corner.x + dx * l, corner.y),
                    corner,
                    pos2(corner.x, corner.y + dy * l),
                ],
                Stroke::new(1.5_f32, c),
            ));
        }
    }
    pen.out
}

fn dark_of(theme: &Theme) -> dodai_core::Color {
    if theme.is_dark() {
        theme.palette.background
    } else {
        theme.palette.text
    }
}

fn light_of(theme: &Theme) -> dodai_core::Color {
    if theme.is_dark() {
        theme.palette.text
    } else {
        theme.palette.background
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> Rect {
        Rect::from_min_size(pos2(10.0, 10.0), vec2(120.0, 48.0))
    }

    #[test]
    fn nothing_is_drawn_by_default() {
        set_surface_fx(SurfaceFx::NONE);
        let t = dodai::themes::cyberpunk::yellow();
        assert!(surface_shapes(&t, rect(), SurfaceClass::Button, None).is_empty());
    }

    #[test]
    fn a_texture_stays_inside_the_rect_and_within_the_budget() {
        let t = dodai::themes::cyberpunk::yellow();
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
            set_surface_fx(SurfaceFx {
                texture: tex,
                strength: 0.8,
                targets: dodai_core::SurfaceTargets::ALL,
                ..SurfaceFx::NONE
            });
            let shapes = surface_shapes(&t, rect(), SurfaceClass::Button, None);
            assert!(!shapes.is_empty(), "{tex:?}");
            assert!(
                shapes.len() as f32 <= MAX_SHAPES * 1.3,
                "{tex:?}: {}",
                shapes.len()
            );
            for s in &shapes {
                assert!(
                    rect().expand(2.0).contains_rect(s.visual_bounding_rect()),
                    "{tex:?} leaves the rect"
                );
            }
        }
        set_surface_fx(SurfaceFx::NONE);
    }

    #[test]
    fn a_design_draws_only_what_it_owns() {
        let m = dodai::themes::material::light();
        set_surface_fx(SurfaceFx {
            texture: Texture::Circuit,
            glitch: 1.0,
            targets: dodai_core::SurfaceTargets::ALL,
            ..SurfaceFx::NONE
        });
        assert!(surface_shapes(&m, rect(), SurfaceClass::Button, None).is_empty());
        set_surface_fx(SurfaceFx::NONE);
    }

    #[test]
    fn animated_effects_ask_for_frames() {
        let t = dodai::themes::cyberpunk::yellow();
        set_surface_fx(SurfaceFx {
            pulse: 0.8,
            targets: dodai_core::SurfaceTargets::ALL,
            ..SurfaceFx::NONE
        });
        let _ = take_animated();
        assert!(!surface_shapes(&t, rect(), SurfaceClass::Panel, None).is_empty());
        assert!(take_animated());
        set_surface_fx(SurfaceFx::NONE);
    }

    #[test]
    fn clipping_keeps_segments_inside() {
        let r = rect();
        let (a, b) = clip(pos2(0.0, 0.0), pos2(200.0, 100.0), r).unwrap();
        assert!(r.expand(0.01).contains(a) && r.expand(0.01).contains(b));
        assert!(clip(pos2(0.0, 0.0), pos2(5.0, 5.0), r).is_none());
    }
}
