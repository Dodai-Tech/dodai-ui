//! The effect painters: textures and animated effects, drawn through a [`Canvas`].

use crate::canvas::Canvas;
use crate::geom::{fit, noise, outline_of, MAX_QUADS};
use crate::Texture;
use crate::{scale_alpha, Rgba, SurfacePaint};

/// Horizontal scanline rows every `pitch` px, kept inside the shape and its border.
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

pub(crate) fn texture(cv: &mut Canvas, sp: &SurfacePaint, seed: i32) {
    let fx = sp.fx;
    let k = fx.scale.max(0.25) * sp.tune.cell;
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
    hatch(cv, sp, 9.0 * k, 1.0, 0.24 * s);
    hatch(cv, sp, 9.0 * k, -1.0, 0.24 * s);
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
    let (w, h) = (cv.w, cv.h);
    // Both the line spacing and the dash length grow on huge surfaces to stay under the budget.
    let n = (h / 2.0) * (w / pitch.max(3.0));
    let grow = (n / crate::geom::MAX_QUADS).max(1.0).sqrt();
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

/// Inner shadow: dark rings pulled in from the edge, fading inward.
pub(crate) fn depth(cv: &mut Canvas, sp: &SurfacePaint) {
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
pub(crate) fn tint(cv: &mut Canvas, sp: &SurfacePaint) {
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

/// Draw one frame (`t` in `0..1`) of a click ripple of the given style.
pub(crate) fn press_frame(
    cv: &mut Canvas,
    style: crate::PressStyle,
    accent: Rgba,
    ink: Rgba,
    k: f32,
    at: (f32, f32),
    t: f32,
) {
    use crate::PressStyle as P;
    let (w, h) = (cv.w, cv.h);
    // Ease-out spread, and a fade that holds for the first third.
    let spread = 1.0 - (1.0 - t).powi(3);
    let fade = 1.0 - ((t - 0.3).max(0.0) / 0.7).powi(2);
    let far = |x: f32, y: f32| {
        let dx = x.max(w - x);
        let dy = y.max(h - y);
        (dx * dx + dy * dy).sqrt()
    };
    let (cx, cy) = (w / 2.0, h / 2.0);
    match style {
        P::Ink | P::Centre => {
            let (px0, py0) = if style == P::Ink { at } else { (cx, cy) };
            let r = far(px0, py0) * spread;
            let c = scale_alpha(ink, 0.18 * k * fade);
            let mut y = cv.inset;
            while y < h - cv.inset {
                let dy = y + 0.5 - py0;
                if dy.abs() < r {
                    let half = (r * r - dy * dy).sqrt();
                    cv.quad(px0 - half, y, 2.0 * half, 1.0, c);
                }
                y += 1.0;
            }
        }
        P::Ring | P::Echo => {
            let waves = if style == P::Ring { 1 } else { 3 };
            for i in 0..waves {
                let ti = (t - 0.12 * i as f32).clamp(0.0, 1.0);
                if ti <= 0.0 {
                    continue;
                }
                let r = far(at.0, at.1) * (1.0 - (1.0 - ti).powi(3));
                let f = (1.0 - ti) * (1.0 - ti);
                cv.ring_clipped(at.0, at.1, r, 2.0, scale_alpha(accent, 0.6 * k * f));
            }
        }
        P::Sweep => {
            // A soft vertical band moving away from the touch point towards the far side.
            let dir = if at.0 < cx { 1.0 } else { -1.0 };
            let travel = w * spread;
            let x = at.0 + dir * travel;
            let band = (w * 0.25).max(12.0);
            let n = 10;
            for j in 0..n {
                let f = (j as f32 + 0.5) / n as f32;
                let a = (std::f32::consts::PI * f).sin();
                let seg = band / n as f32;
                cv.quad(
                    x - band / 2.0 + j as f32 * seg,
                    0.0,
                    seg + 0.5,
                    h,
                    scale_alpha(accent, 0.4 * k * a * fade),
                );
            }
        }
        P::Glow => {
            // Stacked discs make a soft radial glow that only fades.
            let r0 = h.max(24.0);
            let layers = 6;
            for j in (0..layers).rev() {
                let r = r0 * (0.35 + 0.65 * (j + 1) as f32 / layers as f32);
                let c = scale_alpha(accent, 0.14 * k * (1.0 - t));
                let mut y = cv.inset;
                while y < h - cv.inset {
                    let dy = y + 0.5 - at.1;
                    if dy.abs() < r {
                        let half = (r * r - dy * dy).sqrt();
                        cv.quad(at.0 - half, y, 2.0 * half, 1.0, c);
                    }
                    y += 1.0;
                }
            }
        }
    }
}

/// Neo-brutalism: a hard offset block along the bottom and right inner edges.
pub(crate) fn pop(cv: &mut Canvas, sp: &SurfacePaint) {
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
pub(crate) fn tape(cv: &mut Canvas, sp: &SurfacePaint) {
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

/// A three-colour gradient laid over the surface at 135°, in coarse cells so it stays cheap.
/// It is faint on purpose: frosted glass is tinted by it, not covered.
pub(crate) fn wash(cv: &mut Canvas, sp: &SurfacePaint) {
    let luma = |c: Rgba| 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
    let wash_luma = sp.wash.iter().map(|c| luma(*c)).sum::<f32>() / 3.0;
    // A wash darker than a bright surface (a pale primary button) would only grey it, so it
    // fades out in proportion to how much it would darken.
    let hole = luma(sp.hole);
    let darken = if hole > 0.6 {
        (hole - wash_luma).max(0.0)
    } else {
        0.0
    };
    let k = sp.fx.wash.clamp(0.0, 1.0) * (1.0 - 0.8 * darken.min(1.0));
    let (w, h) = (cv.w, cv.h);
    let cols = (w / 10.0).clamp(2.0, 20.0) as usize;
    // Rows are coarse in the middle but 2 px tall through the rounded corners, where each cell
    // is clipped to the curve; coarse rows there would turn it into steps.
    let corner = cv.shape.radius.min(h / 2.0).ceil();
    let mut ys = vec![0.0_f32];
    while *ys.last().unwrap() < h - 0.5 {
        let y = *ys.last().unwrap();
        let step = if y < corner || y >= h - corner {
            2.0
        } else {
            6.0
        };
        ys.push((y + step).min(h));
    }
    let rows = ys.len() - 1;
    let edge = |n: usize, len: f32, of: usize| (len * n as f32 / of as f32).round();
    let at = |t: f32| {
        let s = &sp.wash;
        let (a, b, t) = if t < 0.5 {
            (s[0], s[1], t * 2.0)
        } else {
            (s[1], s[2], t * 2.0 - 1.0)
        };
        Rgba {
            r: a.r + (b.r - a.r) * t,
            g: a.g + (b.g - a.g) * t,
            b: a.b + (b.b - a.b) * t,
            a: 1.0,
        }
    };
    for j in 0..rows {
        for i in 0..cols {
            let t = ((i as f32 + 0.5) / cols as f32 + ((ys[j] + ys[j + 1]) / 2.0) / h) / 2.0;
            let (x0, x1) = (edge(i, w, cols), edge(i + 1, w, cols));
            let (y0, y1) = (ys[j], ys[j + 1]);
            cv.quad(x0, y0, x1 - x0, y1 - y0, scale_alpha(at(t), 0.24 * k));
        }
    }
}

/// A clear-coat reflection across the top, cut on a slant like a windscreen glint.
pub(crate) fn gloss(cv: &mut Canvas, sp: &SurfacePaint) {
    let g = sp.fx.gloss.clamp(0.0, 1.0);
    let (w, h) = (cv.w, cv.h);
    // A reflection is always lighter than the surface, so it brightens the background tone.
    let lit = Rgba::WHITE;
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

/// A thin line of light all around the border, following every corner, with a
/// brighter spot that travels along it and a tail that fades out behind.
pub(crate) fn strip(cv: &mut Canvas, sp: &SurfacePaint) {
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
    let head = (cv.time / sp.tune.sheen_period.max(0.8)).fract() * total;
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
pub(crate) fn edge_light(cv: &mut Canvas, sp: &SurfacePaint) {
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
pub(crate) fn brackets(cv: &mut Canvas, sp: &SurfacePaint) {
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
pub(crate) fn pulse(cv: &mut Canvas, sp: &SurfacePaint) {
    let p = sp.fx.pulse.clamp(0.0, 1.0);
    let phase = cv.time / sp.tune.pulse_period.max(0.4) * std::f32::consts::TAU;
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
pub(crate) fn chase(cv: &mut Canvas, sp: &SurfacePaint, seed: i32) {
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
    let head = ((cv.time / sp.tune.chase_period.max(0.5) + noise(seed, 5)).fract()) * total;
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

pub(crate) fn sheen(cv: &mut Canvas, sp: &SurfacePaint) {
    let phase = (cv.time / sp.tune.sheen_period.max(0.5)).fract();
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

pub(crate) fn glitch(cv: &mut Canvas, sp: &SurfacePaint, seed: i32) {
    let g = sp.fx.glitch.clamp(0.0, 1.0);
    let period = sp.tune.glitch_period.max(0.5);
    let burst = sp.tune.glitch_burst + 0.25 * g;
    let t = (cv.time + noise(seed, 3) * period) % period;
    if t > burst {
        return;
    }
    let frame = (cv.time * 24.0) as i32;
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
