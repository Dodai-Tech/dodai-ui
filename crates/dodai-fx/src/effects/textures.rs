//! Textures: the static patterns drawn over a surface.

use crate::canvas::Canvas;
use crate::geom::{fit, noise, MAX_QUADS};
use crate::Texture;
use crate::{scale_alpha, Rgba, SurfacePaint};

pub(crate) fn scanlines(cv: &mut Canvas, c: Rgba, pitch: f32) {
    let (w, h, bw) = (cv.w, cv.h, cv.shape.border_width);
    let pitch = pitch.max(2.0);
    let mut y = pitch - 1.0;
    while y < h {
        if y > bw && y < h - bw {
            if let Some((x0, x1)) = crate::geom::row_span(cv.shape, w, h, y) {
                let (x0, x1) = (x0.max(bw), x1.min(w - bw));
                if x1 > x0 {
                    cv.out.quad(x0, y, x1 - x0, 1.0, c);
                }
            }
        }
        y += pitch;
    }
}

/// Surfaces smaller than this (px²) cannot get near the primitive budget: no dry runs.
const DRY_RUN_MIN_AREA: f32 = 150_000.0;
/// How many times the cells are coarsened at most to fit the budget.
const MAX_DRY_RUNS: usize = 6;
/// A count this far over `MAX_QUADS` triggers coarsening.
const BUDGET_SLACK: f32 = 1.2;
/// Cells grow by `(count / budget)` to this power per dry run…
const COARSEN_EXPONENT: f32 = 0.75;
/// …and by at least this factor.
const MIN_COARSEN: f32 = 1.1;
/// The smallest texture scale (cells never shrink below a quarter).
const MIN_SCALE: f32 = 0.25;

/// Counts primitives instead of drawing them, to size a texture before it is painted.
struct Count(usize);

impl crate::Painter for Count {
    fn quad(&mut self, _: f32, _: f32, _: f32, _: f32, _: Rgba) {
        self.0 += 1;
    }
    fn dot(&mut self, _: f32, _: f32, _: f32, _: Rgba) {
        self.0 += 1;
    }
    fn stroke(&mut self, _: &[(f32, f32)], _: bool, _: f32, _: Rgba) {
        self.0 += 1;
    }
    fn fill(&mut self, _: &[(f32, f32)], _: Rgba) {
        self.0 += 1;
    }
}

/// Draw the texture, with its cells coarsened on big surfaces so one surface never costs more
/// than a few thousand primitives.
pub(crate) fn texture(cv: &mut Canvas, sp: &SurfacePaint, seed: i32) {
    let mut k = sp.fx.scale.max(MIN_SCALE) * sp.tune.cell;
    // Small surfaces cannot get near the budget: skip the dry runs.
    if cv.w * cv.h > DRY_RUN_MIN_AREA {
        for _ in 0..MAX_DRY_RUNS {
            let mut count = Count(0);
            let mut dry = Canvas {
                out: &mut count,
                shape: cv.shape,
                pts: cv.pts.clone(),
                w: cv.w,
                h: cv.h,
                inset: cv.inset,
                neon: cv.neon,
                neon_c: cv.neon_c,
                halo: cv.halo,
                clear: cv.clear,
                time: cv.time,
            };
            draw_texture(&mut dry, sp, seed, k);
            let n = count.0 as f32;
            if n <= crate::geom::MAX_QUADS * BUDGET_SLACK {
                break;
            }
            k *= (n / crate::geom::MAX_QUADS)
                .powf(COARSEN_EXPONENT)
                .max(MIN_COARSEN);
        }
    }
    draw_texture(cv, sp, seed, k);
}

fn draw_texture(cv: &mut Canvas, sp: &SurfacePaint, seed: i32, k: f32) {
    let fx = sp.fx;
    let s = fx.strength.clamp(0.0, 1.0);
    match fx.texture {
        Texture::None => {}
        Texture::Grain => grain(cv, sp, k, s, seed),
        Texture::Halftone => halftone(cv, sp, k, s),
        Texture::Dither => dither(cv, sp, k, s),
        Texture::Hatch => hatch(cv, sp, 8.0 * k, 1.0, 0.3 * s),
        Texture::Crosshatch => crosshatch(cv, sp, k, s),
        Texture::Grid => grid(cv, sp, 12.0 * k, 5, 0.18 * s, 0.4 * s),
        Texture::Blueprint => blueprint(cv, sp, k, s),
        Texture::DotGrid => dot_grid(cv, sp, k, s),
        Texture::Stripes => stripes(cv, sp, k, s),
        Texture::Checker => checker(cv, sp, k, s),
        Texture::Circuit => circuit(cv, sp, k, s, seed),
        Texture::DiamondPlate => diamond_plate(cv, sp, k, s),
        Texture::Paper => paper(cv, sp, k, s, seed),
        Texture::LedMatrix => led_matrix(cv, sp, k, s, seed),
        Texture::Barcode => barcode(cv, sp, k, s, seed),
        Texture::Static => noise_static(cv, sp, k, s, seed),
        Texture::Brushed => brushed(cv, sp, k, s, seed),
        Texture::Carbon => carbon(cv, sp, k, s),
        Texture::Perforation => perforation(cv, sp, k, s),
        Texture::Ripples => ripples(cv, sp, k, s),
        Texture::Zigzag => zigzag(cv, sp, k, s),
        Texture::Houndstooth => houndstooth(cv, sp, k, s),
        Texture::Hazard => hazard(cv, sp, k, s),
        Texture::HexMesh => hex_mesh(cv, sp, k, s),
        Texture::Rain => rain(cv, sp, k, s, seed),
        Texture::Grime => grime(cv, sp, k, s, seed),
        Texture::DataStream => data_stream(cv, sp, k, s, seed),
        Texture::GlitchBlocks => glitch_blocks(cv, sp, k, s, seed),
        Texture::ChipDie => chip_die(cv, sp, k, s),
        Texture::Wiring => wiring(cv, sp, k, s, seed),
        Texture::Radar => radar(cv, sp, k, s),
        Texture::Scanlines => scanlines_texture(cv, sp, k, s),
    }
}

fn grain(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32, seed: i32) {
    let (w, h) = (cv.w, cv.h);
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

fn halftone(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn dither(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn crosshatch(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    hatch_share(cv, sp, 9.0 * k, 1.0, 0.24 * s, 0.5);
    hatch_share(cv, sp, 9.0 * k, -1.0, 0.24 * s, 0.5);
}

fn blueprint(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
    grid(cv, sp, 8.0 * k, 4, 0.12 * s, 0.32 * s);
    // registration crosses in the corners
    let m = 8.0;
    for (x, y) in [(m, m), (w - m, m), (m, h - m), (w - m, h - m)] {
        cv.line(x - 4.0, y, 9.0, 1.0, scale_alpha(sp.ink, 0.5 * s));
        cv.line(x, y - 4.0, 1.0, 9.0, scale_alpha(sp.ink, 0.5 * s));
    }
}

fn dot_grid(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn stripes(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn checker(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn circuit(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32, seed: i32) {
    let (w, h) = (cv.w, cv.h);
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

fn diamond_plate(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn paper(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32, seed: i32) {
    let (w, h) = (cv.w, cv.h);
    let n = ((w * h / 90.0) as i32).min(MAX_QUADS as i32 / 2);
    for i in 0..n {
        let (x, y) = (noise(i, seed) * w, noise(seed, i + 7) * h);
        let len = 2.0 + noise(i, 5) * 7.0 * k;
        let a = 0.06 + 0.18 * noise(i + 3, 11);
        cv.quad(x, y, len, 1.0, scale_alpha(sp.ink, a * s * 2.0));
    }
}

fn led_matrix(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32, seed: i32) {
    let (w, h) = (cv.w, cv.h);
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

fn barcode(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32, seed: i32) {
    let (w, h) = (cv.w, cv.h);
    let bar_h = (10.0 * k).max(6.0).min(h * 0.3);
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

fn noise_static(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32, seed: i32) {
    let (w, h) = (cv.w, cv.h);
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

fn brushed(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32, seed: i32) {
    let (w, h) = (cv.w, cv.h);
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

fn carbon(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn perforation(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn ripples(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn zigzag(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn houndstooth(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn hazard(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
    // Warning tape along the top and bottom edges only, so the middle stays clean.
    let band = (h * 0.2).max(4.0).min(9.0 * k);
    let p = (10.0 * k).max(5.0);
    // Count falls with the dash length alone here, so it grows linearly with the overshoot.
    let step = ((band * 2.0) * (w / p) / crate::geom::MAX_QUADS).max(1.0);
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

fn hex_mesh(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn rain(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32, seed: i32) {
    let (w, h) = (cv.w, cv.h);
    let n = ((w * h / 650.0 / k) as i32).min(MAX_QUADS as i32 / 3);
    for i in 0..n {
        let (x, y) = (noise(i, seed) * w, noise(seed, i + 3) * h);
        let len = (6.0 + noise(i, 8) * 14.0) * k;
        let a = 0.1 + 0.3 * noise(i + 1, 2);
        cv.line(x, y, 1.0, len, scale_alpha(sp.ink, a * s * 1.5));
    }
}

fn grime(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32, seed: i32) {
    let (w, h) = (cv.w, cv.h);
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

fn data_stream(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32, seed: i32) {
    let (w, h) = (cv.w, cv.h);
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

fn glitch_blocks(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32, seed: i32) {
    let (w, h) = (cv.w, cv.h);
    let n = ((w * h / 2600.0) as i32).clamp(3, 40);
    for i in 0..n {
        let (x, y) = (noise(i, seed) * w, noise(seed, i + 5) * h);
        let (bw, bh) = ((8.0 + noise(i, 3) * 26.0) * k, 2.0 + noise(i, 4) * 3.0);
        let a = 0.1 + 0.22 * noise(i, 12);
        cv.quad(x, y, bw, bh, scale_alpha(sp.ink, a * s * 1.6));
        cv.quad(x + 2.0, y, bw, bh, scale_alpha(sp.a, 0.16 * s));
    }
}

fn chip_die(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn wiring(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32, seed: i32) {
    let (w, h) = (cv.w, cv.h);
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

fn radar(cv: &mut Canvas, sp: &SurfacePaint, _k: f32, s: f32) {
    let (w, h) = (cv.w, cv.h);
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

fn scanlines_texture(cv: &mut Canvas, sp: &SurfacePaint, k: f32, s: f32) {
    scanlines(cv, scale_alpha(sp.ink, 0.22 * s), crate::SCANLINE_PITCH * k);
}

pub(crate) fn hatch(cv: &mut Canvas, sp: &SurfacePaint, pitch: f32, dir: f32, alpha: f32) {
    hatch_share(cv, sp, pitch, dir, alpha, 1.0);
}

/// [`hatch`] using only `share` of the primitive budget (two crossing hatches take half each).
fn hatch_share(cv: &mut Canvas, sp: &SurfacePaint, pitch: f32, dir: f32, alpha: f32, share: f32) {
    let (w, h) = (cv.w, cv.h);
    // Both the line spacing and the dash length grow on huge surfaces to stay under the budget.
    let n = (h / 2.0) * (w / pitch.max(3.0));
    let grow = (n / (crate::geom::MAX_QUADS * share)).max(1.0).sqrt();
    let p = pitch.max(3.0) * grow;
    let step = 2.0 * grow;
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

pub(crate) fn grid(
    cv: &mut Canvas,
    sp: &SurfacePaint,
    pitch: f32,
    major: i32,
    minor_a: f32,
    major_a: f32,
) {
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
