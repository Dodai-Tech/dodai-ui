//! Second wave of shaders: synthwave, glass, tonal art, oscilloscopes, ASCII,
//! forests, fractals, storms and general-purpose post effects.

use crate::frame::{Ctx, Frame};
use crate::math::*;
use crate::{Family, ShaderInfo};
use std::f32::consts::TAU;

const CY: Family = Family::Cyberpunk;
const M3: Family = Family::Material3;
const MX: Family = Family::Matrix;
const RT: Family = Family::Retro;
const PX: Family = Family::Pixel;
const NA: Family = Family::Nature;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::generator(
        "synthwave",
        "Synthwave sunset",
        CY,
        "Striped sun, mountains and a racing grid",
        synthwave,
    ),
    ShaderInfo::generator(
        "rain_glass",
        "Neon rain glass",
        CY,
        "Bokeh city lights behind a rain-streaked window",
        rain_glass,
    ),
    ShaderInfo::generator(
        "neon_signs",
        "Neon signs",
        CY,
        "Flickering rounded tubes with a halo",
        neon_signs,
    ),
    ShaderInfo::post(
        "hologram",
        "Hologram",
        CY,
        "Cyan projection: scan bands, jitter and fresnel edge",
        hologram,
    ),
    ShaderInfo::generator(
        "m3_mesh",
        "Mesh gradient",
        M3,
        "Four drifting colour blobs blended smoothly",
        m3_mesh,
    ),
    ShaderInfo::generator(
        "m3_dots",
        "Dot wave",
        M3,
        "A dot matrix swelling in rings",
        m3_dots,
    ),
    ShaderInfo::generator(
        "m3_loader",
        "Shape loader",
        M3,
        "A rotating, morphing loading indicator",
        m3_loader,
    ),
    ShaderInfo::generator(
        "m3_bars",
        "Tonal bars",
        M3,
        "Rounded equaliser bars in tonal steps",
        m3_bars,
    ),
    ShaderInfo::post(
        "m3_blur",
        "Frosted blur",
        M3,
        "Glass-like blur with a tonal tint",
        frosted_blur,
    ),
    ShaderInfo::generator(
        "matrix_depth",
        "Rain in depth",
        MX,
        "Three parallax layers of falling code",
        matrix_depth,
    ),
    ShaderInfo::post(
        "matrix_tint",
        "Green screen",
        MX,
        "Monochrome phosphor from any source",
        green_screen,
    ),
    ShaderInfo::generator(
        "oscilloscope",
        "Oscilloscope",
        RT,
        "Lissajous trace on a phosphor graticule",
        oscilloscope,
    ),
    ShaderInfo::generator(
        "teletext",
        "Teletext",
        RT,
        "Chunky mosaic blocks in the eight classic colours",
        teletext,
    ),
    ShaderInfo::post(
        "crt_boot",
        "CRT power-on",
        RT,
        "Dot, line, then the picture swells open, on a loop",
        crt_boot,
    ),
    ShaderInfo::post(
        "roll_bar",
        "Vertical roll",
        RT,
        "Slipping vertical hold and a bright bar",
        roll_bar,
    ),
    ShaderInfo::post(
        "halftone",
        "Halftone",
        RT,
        "Print dots sized by brightness",
        halftone,
    ),
    ShaderInfo::post(
        "ascii_art",
        "ASCII art",
        RT,
        "Brightness mapped to character-like cells",
        ascii_art,
    ),
    ShaderInfo::post(
        "cga",
        "CGA",
        RT,
        "Cyan, magenta, white and black with dithering",
        cga,
    ),
    ShaderInfo::post("vignette", "Vignette", RT, "Darkened corners", vignette),
    ShaderInfo::post("film_grain", "Film grain", RT, "Animated grain", film_grain),
    ShaderInfo::post("sepia", "Sepia", RT, "Warm monochrome", sepia),
    ShaderInfo::generator(
        "forest_parallax",
        "Pine forest",
        PX,
        "Layered silhouettes drifting past at dusk",
        forest_parallax,
    ),
    ShaderInfo::generator(
        "pixel_sea",
        "Sunset sea",
        PX,
        "Banded water and a broken sun reflection",
        pixel_sea,
    ),
    ShaderInfo::generator(
        "mario_sky",
        "Block sky",
        PX,
        "Clouds, hills, bricks and floating ? blocks",
        mario_sky,
    ),
    ShaderInfo::generator(
        "invaders",
        "Invaders",
        PX,
        "A marching grid of two-frame sprites",
        invaders,
    ),
    ShaderInfo::post(
        "posterize",
        "Posterize",
        PX,
        "Few tone levels per channel",
        posterize,
    ),
    ShaderInfo::generator(
        "voronoi",
        "Voronoi cells",
        NA,
        "Moving cells with glowing borders",
        voronoi,
    ),
    ShaderInfo::generator("clouds", "Clouds", NA, "Drifting fractal clouds", clouds),
    ShaderInfo::generator(
        "tunnel",
        "Tunnel",
        NA,
        "Flying down a patterned tube",
        tunnel,
    ),
    ShaderInfo::generator(
        "metaballs",
        "Metaballs",
        NA,
        "Blobs that merge and split",
        metaballs,
    ),
    ShaderInfo::generator(
        "julia",
        "Julia set",
        NA,
        "Fractal with an orbiting parameter",
        julia,
    ),
    ShaderInfo::generator(
        "lightning",
        "Lightning storm",
        NA,
        "Forked bolts and sky flashes",
        lightning,
    ),
    ShaderInfo::generator(
        "rain_drops",
        "Rain on water",
        NA,
        "Expanding ripples",
        rain_drops,
    ),
    ShaderInfo::generator(
        "sine_waves",
        "Sine waves",
        NA,
        "Layered glowing waves",
        sine_waves,
    ),
];

fn bayer(uv: Vec2, f: &Frame) -> f32 {
    bayer4((uv.x * f.w as f32) as i32, (uv.y * f.h as f32) as i32)
}

// ───────────────────────────── Cyberpunk ─────────────────────────────

pub fn synthwave(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let horizon = -0.05;
    let p = &c.pal;
    if q.y > horizon {
        // sky
        let k = ((q.y - horizon) / (1.0 - horizon)).clamp(0.0, 1.0);
        let mut col = p.c.mix(p.bg, 0.4).mix(p.bg, k.powf(0.6));
        // sun with cut-out stripes climbing toward the horizon
        let sc = vec2(0.0, 0.32);
        let d = (q - sc).len();
        let h = (q.y - horizon) / 0.7;
        let cut = if h < 0.55 {
            step(
                0.0,
                (h * 22.0 - c.time * 0.8).sin() * (1.0 - h) + h * 0.9 - 0.35,
            )
        } else {
            1.0
        };
        let sun = fill_aa(d - 0.5, 0.01) * cut;
        col = col.mix(
            p.a.mix(p.c, 1.0 - ((q.y - horizon) / 0.7).clamp(0.0, 1.0)),
            sun,
        );
        col = col + p.c * glow(d - 0.5, 5.0) * 0.18;
        // mountain silhouette
        let m =
            horizon + 0.05 + 0.16 * fbm(vec2(q.x * 2.2 + 3.0, 1.0), 4) * (0.6 + q.x.abs() * 0.8);
        if q.y < m {
            col = p.bg * 0.6;
            let ridge = glow((q.y - m) * 90.0, 1.0);
            col = col + p.b * ridge * 0.9;
        }
        col
    } else {
        // floor grid in perspective
        let depth = 1.0 / (horizon - q.y + 0.02);
        let gx = q.x * depth * 0.8;
        let gz = depth * 0.6 + c.time * 1.3;
        let lx = (gx - gx.round()).abs();
        let lz = (gz - gz.round()).abs();
        let w = 0.04 + 0.02 / depth.max(0.5);
        let line = sat(1.0 - lx / w).max(sat(1.0 - lz / w));
        let fade = sat(1.0 - depth / 14.0);
        let floor = p.bg.mix(p.c, 0.08);
        floor + p.b.mix(p.c, (q.x * 0.5 + 0.5).clamp(0.0, 1.0)) * line * fade * 1.4
    }
}

pub fn rain_glass(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let mut col = p.bg * (0.8 + 0.4 * uv.y);
    // bokeh: hashed discs on a grid, blurred
    for layer in 0..3 {
        let sc = 3.0 + layer as f32 * 2.5;
        let g = q * sc + vec2(c.time * 0.05 * (layer as f32 + 1.0), 0.0);
        let id = g.floor();
        let f = g.fract() - 0.5;
        let h = hash2(id.x as i32 + layer * 31, id.y as i32);
        let off = vec2(
            hash2(id.x as i32, id.y as i32 + 7) - 0.5,
            hash2(id.x as i32 + 9, id.y as i32) - 0.5,
        ) * 0.5;
        let r = 0.18 + 0.15 * h;
        let d = (f - off).len();
        let tone = [p.a, p.b, p.c][(h * 3.0) as usize % 3];
        let on = if h > 0.35 { 1.0 } else { 0.0 };
        col = col + tone * smoothstep(r, r * 0.55, d) * on * (0.22 / (layer as f32 + 1.0) + 0.05);
    }
    // drops and streaks
    let cols = 26.0;
    let cx = (uv.x * cols).floor();
    let hx = hash1(cx);
    let speed = 0.15 + hx * 0.35;
    let ty = fract(uv.y * 0.6 - c.time * speed + hx * 9.0);
    let fx = (uv.x * cols).fract() - 0.5 + (hash1(cx + 3.0) - 0.5) * 0.5;
    let drop =
        smoothstep(0.1, 0.0, fx.abs()) * smoothstep(0.0, 0.05, ty) * smoothstep(0.35, 0.0, ty);
    let head = smoothstep(0.14, 0.0, (vec2(fx * 0.8, (ty - 0.04) * 1.1)).len());
    col + p.fg * (drop * 0.12 + head * 0.35)
}

pub fn neon_signs(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let mut col = p.bg;
    let signs = [
        (vec2(-0.75, 0.35), vec2(0.5, 0.22), p.a, 0.0),
        (vec2(0.55, -0.05), vec2(0.6, 0.3), p.b, 1.7),
        (vec2(-0.2, -0.5), vec2(0.45, 0.14), p.c, 3.1),
    ];
    for (ctr, half, tone, seed) in signs {
        let d = sd_round_box(q - ctr, half, 0.12).abs();
        let phase = c.time * 3.0 + seed;
        let flick = if (phase.sin() * (phase * 2.7).sin()) > 0.93 {
            0.25
        } else {
            1.0
        };
        let tube = fill_aa(d - 0.012, 0.012);
        col = col + tone * (tube * 1.2 + glow(d, 9.0) * 0.35) * flick;
        // inner slash
        let s = sd_segment(q, ctr - half * 0.5, ctr + half * 0.5).abs();
        col = col + tone * (fill_aa(s - 0.008, 0.012) + glow(s, 14.0) * 0.2) * flick * 0.8;
    }
    col
}

pub fn hologram(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let band = (uv.y * 3.0 - c.time * 0.4).fract();
    let jitter = if hash1((c.time * 6.0).floor()) > 0.93 {
        (hash1(uv.y * 40.0) - 0.5) * 0.03 * c.intensity
    } else {
        0.0
    };
    let s = src.sample(vec2(uv.x + jitter, uv.y));
    let l = s.luma();
    let scan = 0.75 + 0.25 * ((uv.y * src.h as f32 * 1.5).sin() * 0.5 + 0.5);
    let edge = ((uv.x - 0.5).abs().max((uv.y - 0.5).abs()) * 2.0).powi(3);
    let tint = p.b.mix(p.a, l * 0.5);
    let holo = tint * (l * 1.3 + 0.08) * scan
        + p.b * edge * 0.25
        + p.fg * smoothstep(0.9, 1.0, band) * 0.15;
    s.mix(holo, 0.5 + 0.5 * c.intensity) * (0.9 + 0.1 * (c.time * 40.0).sin())
}

// ───────────────────────────── Material 3 ─────────────────────────────

pub fn m3_mesh(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let t = c.time * 0.35;
    let pts = [
        (vec2((t).sin() * 0.9, (t * 1.3).cos() * 0.5), p.a),
        (
            vec2((t * 0.8 + 2.0).cos() * 0.9, (t * 0.6 + 1.0).sin() * 0.6),
            p.b,
        ),
        (
            vec2((t * 1.1 + 4.0).sin() * 0.7, (t * 0.9 + 3.0).cos() * 0.6),
            p.c,
        ),
        (
            vec2((t * 0.5 + 1.0).cos() * 0.6, (t * 1.2).sin() * 0.7),
            p.fg,
        ),
        (
            vec2((t * 0.7 + 5.0).sin() * 0.9, (t * 0.4).cos() * 0.5),
            p.bg,
        ),
        (
            vec2((t * 0.9 + 2.0).cos() * 0.9, (t * 0.5 + 4.0).sin() * 0.5),
            p.bg * 2.0,
        ),
    ];
    let mut acc = Vec3::BLACK;
    let mut wsum = 0.0;
    for (pos, col) in pts {
        let w = 1.0 / ((q - pos).dot(q - pos) + 0.04).powi(3);
        acc = acc + col * w;
        wsum += w;
    }
    acc / wsum
}

pub fn m3_dots(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv) * 9.0;
    let id = q.floor();
    let f = q.fract() - 0.5;
    let wave = ((id.len()) * 0.8 - c.time * 2.2).sin() * 0.5 + 0.5;
    let r = 0.12 + 0.34 * wave;
    let d = f.len() - r;
    let col = p.b.mix(p.a, wave).mix(p.c, smoothstep(0.85, 1.0, wave));
    p.bg.mix(col, fill_aa(d, 0.08))
}

pub fn m3_loader(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv).rot(-c.time * 1.6);
    let a = q.y.atan2(q.x);
    let morph = (c.time * 1.1).sin() * 0.5 + 0.5;
    let n = 5.0 + (morph * 3.0).floor();
    let r = 0.45 + 0.07 * (a * n + c.time * 2.0).cos() * (0.4 + morph);
    let d = q.len() - r;
    let ring = fill_aa(d.abs() - 0.045, 0.012);
    let fillc = fill_aa(d + 0.1, 0.012) * 0.12;
    let arc_t = (a / TAU + 0.5 + c.time * 0.0).fract();
    let head = smoothstep(0.0, 0.7, arc_t);
    p.bg.mix(p.a, fillc).mix(p.b.mix(p.a, head), ring)
}

pub fn m3_bars(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let n = 18.0;
    let i = (uv.x * n).floor();
    let fx = (uv.x * n).fract() - 0.5;
    let h = 0.18
        + 0.7
            * (0.5 + 0.5 * (c.time * (1.5 + hash1(i) * 2.0) + i * 0.7).sin())
            * (0.4 + 0.6 * hash1(i + 1.0));
    let y = 1.0 - uv.y;
    let aspect_fix = n / c.aspect();
    let d = sd_round_box(
        vec2(fx, (y - h * 0.5) * aspect_fix),
        vec2(0.34, h * 0.5 * aspect_fix),
        0.34,
    );
    let tone =
        p.b.mix(p.a, y / h.max(0.01))
            .mix(p.c, smoothstep(0.7, 1.0, h));
    p.bg.mix(tone, fill_aa(d, 0.06))
}

pub fn frosted_blur(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let r = 2.5 * (0.3 + 0.7 * c.intensity);
    let (tx, ty) = (r / src.w as f32, r / src.h as f32);
    let mut acc = Vec3::BLACK;
    let mut n = 0.0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let w = if dx == 0 && dy == 0 {
                4.0
            } else if dx == 0 || dy == 0 {
                2.0
            } else {
                1.0
            };
            acc = acc + src.sample(uv + vec2(dx as f32 * tx, dy as f32 * ty)) * w;
            n += w;
        }
    }
    (acc / n).mix(c.pal.bg.mix(c.pal.fg, 0.2), 0.12)
}

// ───────────────────────────── Matrix ─────────────────────────────

pub fn matrix_depth(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let mut col = p.bg;
    for layer in 0..3 {
        let lf = layer as f32;
        let cols = 14.0 + lf * 16.0;
        let rows = cols * 1.8 / c.aspect().max(0.5);
        let cx = (uv.x * cols).floor();
        let hx = hash1(cx + lf * 97.0);
        let speed = (0.25 + hx * 0.5) * (1.0 + lf * 0.7);
        let y = uv.y * rows + c.time * speed * rows * 0.35 + hx * 100.0;
        let row = y.floor();
        let head = fract(-(y / rows) * 2.0 + hx * 5.0);
        let trail = (1.0 - head).powf(2.0 + (2.0 - lf));
        let gid = (hash2(cx as i32, row as i32) * 400.0
            + (c.time * 6.0 * hash2(row as i32, 3)).floor()) as i32;
        let g = crate::matrix::glyph(gid, vec2((uv.x * cols).fract(), y.fract()));
        let bright = [0.35, 0.65, 1.0][layer as usize];
        let hot = smoothstep(0.93, 1.0, 1.0 - head);
        col = col + (p.a * trail + p.fg * hot) * g * bright;
    }
    col
}

pub fn green_screen(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let l = s.luma();
    let scan = 0.88 + 0.12 * (uv.y * src.h as f32 * TAU * 0.5).sin();
    c.pal
        .bg
        .mix(c.pal.a, l.powf(0.8) * 1.1)
        .mix(c.pal.fg, (l - 0.7).max(0.0) * 1.5)
        * scan
}

// ───────────────────────────── Retro ─────────────────────────────

pub fn oscilloscope(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let g = q * 4.0;
    let grid = {
        let l = (g - g.floor() - 0.5).abs();
        let m = (0.5 - l.x).min(0.5 - l.y);
        smoothstep(0.02, 0.0, m) * 0.18
    };
    let mut col = p.bg + p.a * grid;
    let mut dmin: f32 = 9.0;
    let (fa, fb) = (3.0, 2.0 + (c.time * 0.1).sin() * 0.5);
    let pt = |i: i32| {
        let t = i as f32 / 48.0 * TAU;
        vec2((t * fa + c.time).sin() * 1.1, (t * fb).sin() * 0.7)
    };
    let mut prev = pt(0);
    for i in 1..=48 {
        let cur = pt(i);
        dmin = dmin.min(sd_segment(q, prev, cur));
        prev = cur;
    }
    col = col + p.a * (glow(dmin, 55.0) * 0.9 + glow(dmin, 12.0) * 0.18);
    col + p.fg * smoothstep(0.012, 0.0, dmin) * 0.6
}

pub fn teletext(uv: Vec2, c: &Ctx) -> Vec3 {
    let cols = 40.0;
    let rows = 25.0;
    let (gx, gy) = ((uv.x * cols).floor(), (uv.y * rows).floor());
    let fx = (uv.x * cols).fract();
    let fy = (uv.y * rows).fract();
    let tick = (c.time * 1.5).floor();
    let h = hash2(gx as i32 / 2 + tick as i32 % 5, gy as i32 / 3);
    let cols8 = [
        Vec3::BLACK,
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(1.0, 1.0, 0.0),
        vec3(0.0, 0.0, 1.0),
        vec3(1.0, 0.0, 1.0),
        vec3(0.0, 1.0, 1.0),
        vec3(1.0, 1.0, 1.0),
    ];
    let col = cols8[((h * 8.0) as usize).min(7)];
    // 2x3 sextant mosaic
    let sx = (fx * 2.0) as i32;
    let sy = (fy * 3.0) as i32;
    let on = hash2(gx as i32 * 2 + sx, gy as i32 * 3 + sy + tick as i32) > 0.45;
    let gap = (fx * 2.0).fract().min((fy * 3.0).fract());
    let m = if on && gap > 0.08 { 1.0 } else { 0.0 };
    let bar = if gy < 1.0 { 1.0 } else { 0.0 };
    (col * m).mix(vec3(0.0, 0.0, 0.6), bar * 0.8) * 0.9 + c.pal.bg * 0.1
}

pub fn crt_boot(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let cycle = 7.0;
    let t = (c.time % cycle) / cycle;
    // 0..0.15 dot, .15..0.35 line, .35..0.6 open, rest hold
    let k = c.intensity;
    let _ = k;
    let wx = if t < 0.15 {
        0.002
    } else if t < 0.35 {
        mix(0.002, 1.0, ((t - 0.15) / 0.2).powi(3))
    } else {
        1.0
    };
    let hy = if t < 0.35 {
        0.004
    } else if t < 0.6 {
        mix(0.004, 1.0, smoothstep(0.0, 1.0, (t - 0.35) / 0.25))
    } else {
        1.0
    };
    let dx = (uv.x - 0.5).abs() * 2.0;
    let dy = (uv.y - 0.5).abs() * 2.0;
    let inside = dx < wx && dy < hy;
    let flash = glow(dy * 60.0, 1.0) * glow(dx * 3.0, 1.0);
    let base = if inside {
        src.sample(uv) * (1.0 + (1.0 - hy) * 3.0)
    } else {
        Vec3::BLACK
    };
    let off = if t > 0.93 {
        sat(1.0 - (t - 0.93) / 0.07)
    } else {
        1.0
    };
    base * off + c.pal.fg * flash * 0.6 * if hy < 0.99 { 1.0 } else { 0.0 }
}

pub fn roll_bar(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let k = c.intensity;
    let slip = (c.time * 0.35).fract();
    let slipping = smoothstep(0.7, 0.8, slip) * smoothstep(1.0, 0.9, slip);
    let y = (uv.y + slipping * (slip - 0.7) * 3.0 * k).fract();
    let wob = (uv.y * 30.0 + c.time * 10.0).sin() * 0.002 * k * (1.0 + slipping * 6.0);
    let s = src.sample(vec2(uv.x + wob, y));
    let bar = glow((uv.y - (c.time * 0.2).fract()) * 6.0, 1.0);
    s * (1.0 + bar * 0.35 * k) * (0.9 + 0.1 * (c.time * 55.0).sin())
}

pub fn halftone(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let cell = (src.h as f32 / 28.0).max(3.0);
    let (px, py) = (uv.x * src.w as f32, uv.y * src.h as f32);
    let ang = 0.6;
    let r = vec2(px, py).rot(ang) / cell;
    let id = r.floor() + 0.5;
    let f = r - id;
    let centre = (id * cell).rot(-ang);
    let s = src.sample(vec2(centre.x / src.w as f32, centre.y / src.h as f32));
    let rad = s.luma().sqrt() * 0.72;
    let d = f.len() - rad;
    let paper = c.pal.bg.mix(c.pal.fg, 0.05);
    paper.mix(s * 1.15, fill_aa(d, 0.12 * cell.recip() * 8.0))
}

pub fn ascii_art(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let cw = 6.0;
    let ch = 8.0;
    let (px, py) = (uv.x * src.w as f32, uv.y * src.h as f32);
    let (cx, cy) = ((px / cw).floor(), (py / ch).floor());
    let f = vec2((px / cw).fract(), (py / ch).fract());
    let s = src.sample(vec2(
        (cx + 0.5) * cw / src.w as f32,
        (cy + 0.5) * ch / src.h as f32,
    ));
    let l = s.luma().powf(0.8);
    let g = vec2((f.x - 0.5).abs(), (f.y - 0.5).abs());
    let ink = if l < 0.12 {
        0.0
    } else if l < 0.25 {
        // .
        sat(1.0 - ((f - vec2(0.5, 0.82)).len() * 7.0))
    } else if l < 0.4 {
        // :
        sat(1.0 - ((f - vec2(0.5, 0.3)).len() * 7.0))
            .max(sat(1.0 - ((f - vec2(0.5, 0.75)).len() * 7.0)))
    } else if l < 0.55 {
        // +
        step(g.x, 0.1).max(step(g.y, 0.08)) * step(g.x.max(g.y), 0.32)
    } else if l < 0.72 {
        // x
        step(((f.x - 0.5) - (f.y - 0.5)).abs(), 0.13)
            .max(step(((f.x - 0.5) + (f.y - 0.5)).abs(), 0.13))
            * step(g.x.max(g.y), 0.36)
    } else {
        // #
        step(g.x.min(g.y), 0.09).max(step((g.x - 0.2).abs(), 0.07)) * step(g.x.max(g.y), 0.4)
    };
    c.pal.bg.mix(s.mix(c.pal.fg, 0.35) * 1.3, ink)
}

pub fn cga(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let pal = [
        vec3(0.0, 0.0, 0.0),
        vec3(0.33, 1.0, 1.0),
        vec3(1.0, 0.33, 1.0),
        vec3(1.0, 1.0, 1.0),
    ];
    let s = src.sample(uv);
    let l = (s.luma() + (bayer(uv, src) - 0.5) * 0.35 * c.intensity).clamp(0.0, 0.999);
    let hue = (s.x - s.z) * 0.5 + 0.5;
    let idx = if l < 0.22 {
        0
    } else if l > 0.8 {
        3
    } else if hue > 0.5 {
        2
    } else {
        1
    };
    pal[idx]
}

pub fn vignette(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let d = (uv - 0.5) * vec2(1.0, 0.9);
    src.sample(uv) * (1.0 - smoothstep(0.25, 0.85, d.len() * 1.5) * c.intensity)
}

pub fn film_grain(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let (x, y) = ((uv.x * src.w as f32) as i32, (uv.y * src.h as f32) as i32);
    let n = hash3(x, y, (c.time * 24.0) as i32) - 0.5;
    src.sample(uv) + Vec3::splat(n * 0.28 * c.intensity)
}

pub fn sepia(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let l = s.luma();
    s.mix(vec3(l * 1.07, l * 0.86, l * 0.62), c.intensity)
}

// ───────────────────────────── Pixel ─────────────────────────────

fn ridge(x: f32, seed: f32, scale: f32, amp: f32) -> f32 {
    fbm(vec2(x * scale + seed, seed), 3) * amp
}

pub fn forest_parallax(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let sky = p.c.mix(p.bg, uv.y.powf(0.4) * 1.1).mix(p.a, 0.12);
    let mut col = sky;
    // moon
    col = col.mix(p.fg, fill_aa((q - vec2(0.7, 0.55)).len() - 0.12, 0.02));
    for layer in 0..4 {
        let lf = layer as f32;
        let speed = 0.05 + lf * 0.12;
        let x = q.x + c.time * speed;
        let base = -0.15 - lf * 0.2 + 0.25;
        let hill = base + ridge(x, lf * 11.0, 1.2, 0.35);
        // pine silhouettes: triangular wave on top of hills
        let freq = 5.0 + lf * 3.0;
        let tri = (fract(x * freq) - 0.5).abs();
        let tree_h = hill
            + (0.5 - tri)
                * (0.16 + lf * 0.03)
                * if hash1((x * freq).floor() + lf) > 0.25 {
                    1.0
                } else {
                    0.35
                };
        let tone =
            p.bg.mix(p.b, 0.22 - lf * 0.05)
                .mix(p.c, (1.0 - lf / 3.0) * 0.12);
        let shade = tone * (0.35 + lf * 0.22);
        if q.y < tree_h {
            col = shade;
        }
    }
    // fog
    col.mix(p.fg, (1.0 - uv.y).powi(3) * 0.08)
}

pub fn pixel_sea(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let horizon = 0.45;
    if uv.y < horizon {
        let k = uv.y / horizon;
        let sky = p.c.mix(p.a, (1.0 - k).powf(1.5)).mix(p.bg, k * 0.55);
        let sun = fill_aa(
            ((uv - vec2(0.5, 0.36)) * vec2(c.aspect(), 1.0)).len() - 0.14,
            0.01,
        );
        // quantise to bands for the 16-bit look
        let band = (k * 6.0).floor() / 6.0;
        sky.mix(p.a.mix(p.fg, 0.4), sun) * (0.85 + 0.15 * band)
    } else {
        let d = (uv.y - horizon) / (1.0 - horizon);
        let row = (d * 18.0).floor();
        let wave = (row * 1.7 + uv.x * (6.0 + row) * 0.5 + c.time * (0.8 + row * 0.05)).sin();
        let base = p.b.mix(p.bg, d * 0.8);
        let sun_col = (uv.x - 0.5).abs() < 0.14 * (1.0 - d * 0.5) && wave > 0.2 - (1.0 - d) * 0.5;
        let c0 = base * (0.7 + 0.15 * wave.signum());
        if sun_col && d < 0.8 {
            p.a.mix(p.fg, 0.35)
        } else {
            c0
        }
    }
}

pub fn mario_sky(uv: Vec2, c: &Ctx) -> Vec3 {
    let x = uv.x * c.aspect() + c.time * 0.12; // world x, in screen heights
    let mut col = vec3(0.36, 0.58, 0.99);
    // clouds: three parallax lanes of circle unions
    for i in 0..3 {
        let f = i as f32;
        let period = 1.2 + f * 0.4;
        let lane = x * (0.45 + f * 0.2) + f * 0.37;
        let cell = (lane / period).floor();
        let local = vec2(
            (lane / period).fract() * period - period * 0.5,
            uv.y - (0.14 + f * 0.1 + hash1(cell + f) * 0.06),
        );
        let r = |o: f32, rr: f32| (local - vec2(o, 0.0)).len() - rr;
        let d = r(0.0, 0.06)
            .min(r(0.08, 0.075))
            .min(r(-0.08, 0.05))
            .min(r(0.16, 0.05));
        if d < 0.0 && local.y < 0.05 && hash1(cell * 1.7 + f) > 0.2 {
            col = Vec3::WHITE;
        }
    }
    // hills
    if uv.y > 0.74 + 0.07 * (x * 4.0).sin().abs() {
        col = vec3(0.0, 0.66, 0.16);
    }
    // question blocks on a row at y 0.5
    let cell = 0.08;
    let bxn = x * 1.3 / cell;
    if hash1(bxn.floor() * 3.7) > 0.88 && uv.y > 0.5 && uv.y < 0.5 + cell {
        let f = vec2(bxn.fract(), (uv.y - 0.5) / cell);
        let border = f.x < 0.1 || f.x > 0.9 || f.y < 0.1 || f.y > 0.9;
        let q = (f - 0.5).abs();
        let mark =
            (q.x < 0.1 && f.y > 0.3 && f.y < 0.62) || (f.y > 0.72 && f.y < 0.82 && q.x < 0.1);
        col = if border {
            vec3(0.4, 0.2, 0.0)
        } else if mark {
            vec3(1.0, 0.95, 0.7)
        } else {
            vec3(0.98, 0.65, 0.1)
        };
    }
    // brick ground
    let gy = 0.86;
    if uv.y > gy {
        let row = ((uv.y - gy) / 0.07).floor();
        let bx = (x * 14.0 + row * 0.5).fract();
        let fy = ((uv.y - gy) / 0.07).fract();
        col = if bx < 0.07 || fy < 0.1 {
            Vec3::BLACK
        } else {
            vec3(0.78, 0.30, 0.06)
        };
    }
    col
}

const INV: [[u8; 8]; 2] = [
    [0x18, 0x3C, 0x7E, 0xDB, 0xFF, 0x24, 0x5A, 0xA5],
    [0x18, 0x3C, 0x7E, 0xDB, 0xFF, 0x5A, 0x81, 0x42],
];

pub fn invaders(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let frame = ((c.time * 2.0) as usize) % 2;
    let cols = 9.0;
    let rows = 5.0;
    let cell = vec2(1.0 / cols, 0.7 / rows);
    let sway = (c.time * 0.7).sin() * 0.05 + (c.time * 0.35).sin() * 0.03;
    let step_y = ((c.time * 0.25) % 1.0) * 0.06;
    let q = vec2(uv.x - 0.02 - sway, uv.y - 0.05 - step_y);
    let mut col = p.bg;
    // stars
    let sx = (uv.x * c.w as f32 * 0.25).floor();
    let sy = (uv.y * c.h as f32 * 0.25).floor();
    if hash2(sx as i32, sy as i32) > 0.985 {
        col = col + p.fg * 0.4;
    }
    if q.x >= 0.0 && q.y >= 0.0 && q.x < 1.0 && q.y < 0.7 {
        let cx = (q.x / cell.x).floor();
        let cy = (q.y / cell.y).floor();
        let f = vec2((q.x / cell.x).fract(), (q.y / cell.y).fract());
        if f.x > 0.12 && f.x < 0.88 && f.y > 0.12 && f.y < 0.88 {
            let sp = vec2((f.x - 0.12) / 0.76, (f.y - 0.12) / 0.76);
            let (bx, by) = ((sp.x * 8.0) as usize, (sp.y * 8.0) as usize);
            let bits = INV[frame][by.min(7)];
            if bits & (0x80 >> bx.min(7)) != 0 {
                let tone = [p.c, p.a, p.b, p.fg, p.a][cy as usize % 5];
                col = tone.mix(p.fg, hash2(cx as i32, 0) * 0.2);
            }
        }
    }
    col
}

pub fn posterize(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let levels = mix(8.0, 3.0, c.intensity).round();
    let s = src.sample(uv);
    let q = |v: f32| (v * (levels - 1.0)).round() / (levels - 1.0);
    vec3(q(s.x), q(s.y), q(s.z))
}

// ───────────────────────────── Nature ─────────────────────────────

pub fn voronoi(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv) * 3.2;
    let id = q.floor();
    let mut best = (9.0, 9.0, 0.0);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let g = id + vec2(dx as f32, dy as f32);
            let h = hash2(g.x as i32, g.y as i32);
            let o = vec2(
                0.5 + 0.45 * (c.time * 0.8 + h * TAU).sin(),
                0.5 + 0.45 * (c.time * 0.7 + h * 17.0).cos(),
            );
            let d = (g + o - q).len();
            if d < best.0 {
                best = (d, best.0, h);
            } else if d < best.1 {
                best.1 = d;
            }
        }
    }
    let edge = best.1 - best.0;
    let fillc =
        p.b.mix(p.a, best.2).mix(p.c, (best.2 * 7.0).fract() * 0.4) * (0.25 + 0.5 * (1.0 - best.0));
    p.bg.mix(fillc, 0.8) + p.fg * smoothstep(0.08, 0.0, edge) * 0.9
}

pub fn clouds(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = vec2(uv.x * c.aspect(), uv.y) * 2.2 + vec2(c.time * 0.06, 0.0);
    let n = fbm(q + fbm(q * 1.7 + c.time * 0.03, 3) * 0.8, 5);
    let cover = smoothstep(0.38, 0.75, n);
    let sky = p.b.mix(p.bg, uv.y).mix(p.a, (1.0 - uv.y) * 0.25);
    let shade = fbm(q + vec2(0.1, -0.08), 4);
    let cloud = p.fg.mix(p.b * 0.6, (n - shade).max(0.0) * 3.0 + 0.1);
    sky.mix(cloud, cover)
}

pub fn tunnel(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv) + vec2((c.time * 0.4).sin() * 0.15, (c.time * 0.3).cos() * 0.1);
    let r = q.len().max(0.001);
    let a = q.y.atan2(q.x);
    let z = 0.35 / r + c.time * 0.9;
    let u = a / TAU * 12.0;
    let check = ((u.floor() + z.floor()) % 2.0).abs();
    let ring = glow((z - z.round()) * 3.0, 3.0);
    let col = p.b.mix(p.a, check) * (0.25 + 0.5 * check) + p.c * ring * 0.4;
    col * smoothstep(0.0, 0.5, r)
}

pub fn metaballs(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let mut f = 0.0;
    for i in 0..6 {
        let t = c.time * (0.4 + i as f32 * 0.13) + i as f32 * 1.9;
        let pos = vec2(t.sin() * 0.8, (t * 1.3 + i as f32).cos() * 0.55);
        let d = (q - pos).dot(q - pos);
        f += 0.045 / (d + 0.01);
    }
    let edge = smoothstep(0.9, 1.1, f);
    let rim = glow((f - 1.0) * 6.0, 1.0);
    p.bg.mix(p.b.mix(p.a, (f * 0.3).fract() + 0.3), edge) + p.c * rim * 0.25
}

pub fn julia(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let mut z = c.centered(uv) * 1.35;
    let k = vec2(
        -0.74 + 0.1 * (c.time * 0.3).cos(),
        0.15 + 0.1 * (c.time * 0.23).sin(),
    );
    let mut i = 0;
    let n = 48;
    while i < n && z.dot(z) < 16.0 {
        z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + k;
        i += 1;
    }
    if i == n {
        return p.bg;
    }
    let sm = i as f32 - (z.dot(z).ln() / 2.0).ln().max(0.0) / std::f32::consts::LN_2;
    let t = (sm / n as f32).sqrt();
    p.bg.mix(p.b, sat(t * 2.0))
        .mix(p.a, sat(t * 2.0 - 0.6))
        .mix(p.c, sat(t * 3.0 - 2.0))
}

pub fn lightning(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let strike = (c.time * 0.9).floor();
    let ph = fract(c.time * 0.9);
    let on = smoothstep(0.0, 0.03, ph)
        * smoothstep(0.4, 0.1, ph)
        * if hash1(strike) > 0.2 { 1.0 } else { 0.0 };
    let x0 = (hash1(strike + 3.0) - 0.5) * 1.4 * c.aspect();
    let mut col = p.bg.mix(p.b, 0.25 * (1.0 - uv.y)) + p.fg * on * 0.12 * (1.0 - uv.y);
    // clouds
    col = col.mix(
        p.bg * 0.5,
        fbm(vec2(uv.x * 3.0 * c.aspect() + c.time * 0.1, uv.y * 3.0), 4)
            * smoothstep(0.55, 0.0, uv.y)
            * 0.8,
    );
    for b in 0..3 {
        let bf = b as f32;
        let jag = (fbm(vec2(q.y * 6.0 + strike * 10.0 + bf * 5.0, bf), 4) - 0.5)
            * 0.7
            * (1.0 - bf * 0.15);
        let x = x0 + jag + bf * (q.y + 1.0) * 0.12 * if b % 2 == 0 { 1.0 } else { -1.0 };
        let d = (q.x - x).abs();
        let lit = if b == 0 {
            1.0
        } else {
            0.4 * smoothstep(0.1, -0.7, q.y)
        };
        col = col + (p.fg * glow(d * 90.0, 1.0) + p.b * glow(d, 10.0) * 0.4) * on * lit;
    }
    col
}

pub fn rain_drops(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = vec2(uv.x * c.aspect(), uv.y) * 4.0;
    let mut h = 0.0;
    for layer in 0..2 {
        let g = q * (1.0 + layer as f32 * 0.7) + vec2(layer as f32 * 13.0, 0.0);
        let id = g.floor();
        let f = g.fract() - 0.5;
        let seed = hash2(id.x as i32, id.y as i32 + layer * 5);
        let t = fract(c.time * 0.35 + seed);
        let r = t * 0.7;
        let d = (f - vec2(seed - 0.5, hash2(id.x as i32 + 4, id.y as i32) - 0.5) * 0.4).len();
        h += (d - r).abs().min(0.2).recip().min(60.0) * 0.0
            + (1.0 - ((d - r) * 14.0).abs()).max(0.0)
                * (1.0 - t).powi(2)
                * (((d - r) * 30.0).cos() * 0.5 + 0.5);
    }
    let base = p.bg.mix(p.b, (1.0 - uv.y) * 0.5 + 0.1);
    base + p.fg * h * 0.55 + p.a * h * h * 0.2
}

pub fn sine_waves(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let mut col = p.bg;
    for i in 0..7 {
        let f = i as f32;
        let y = (q.x * (1.2 + f * 0.35) + c.time * (0.6 + f * 0.12) + f).sin() * (0.25 - f * 0.018)
            + (f - 3.0) * 0.14;
        let d = (q.y - y).abs();
        let tone = p.a.mix(p.c, f / 6.0).mix(p.b, 0.25);
        col = col + tone * (glow(d * 40.0, 1.0) * 0.8 + glow(d, 9.0) * 0.12);
    }
    col
}
