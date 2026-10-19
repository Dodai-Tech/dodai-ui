//! Retro terminal shaders: CRT, phosphor, VHS, TFT, snow and the screens
//! they are meant to be applied to.

use crate::frame::{Ctx, Frame};
use crate::math::*;
use crate::matrix::glyph;
#[allow(unused_imports)]
use crate::prelude::*;
use crate::{Family, ShaderInfo};

const F: Family = Family::Retro;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::generator(
        "terminal_scene",
        "Terminal screen",
        F,
        "Text, prompt and a blinking cursor: a source for the effects below",
        terminal_scene,
    ),
    ShaderInfo::generator(
        "test_pattern",
        "Test pattern",
        F,
        "SMPTE-style colour bars",
        test_pattern,
    ),
    ShaderInfo::generator(
        "tv_snow",
        "TV snow",
        F,
        "Analogue static with a rolling bar",
        tv_snow,
    ),
    ShaderInfo::post(
        "crt",
        "CRT",
        F,
        "Curvature, scanlines, shadow mask, bloom, vignette, flicker",
        crt,
    ),
    ShaderInfo::post(
        "phosphor",
        "Phosphor monitor",
        F,
        "Mono phosphor tube in the palette colour with glow and persistence streaks",
        phosphor,
    ),
    ShaderInfo::post(
        "vhs",
        "VHS",
        F,
        "Tracking noise, line jitter, chroma bleed and tape grain",
        vhs,
    ),
    ShaderInfo::post(
        "tft",
        "TFT subpixels",
        F,
        "LCD RGB stripes and pixel gaps",
        tft,
    ),
    ShaderInfo::post(
        "static_noise",
        "Static",
        F,
        "Mixes analogue snow over the frame",
        static_noise,
    ),
];

/// Terminal text as fixed-width byte rows (0 = blank), built at compile time.
const LINES: [&str; 15] = [
    "DODAI-OS 2.0  (c) 2077 ACME SYSTEMS",
    "",
    "login: operator",
    "last login: tue jan 07 03:14:15 on tty1",
    "",
    "$ ls /sys/neural",
    "core  cortex  daemon  grid  mesh  uplink",
    "$ ./boot --theme cyberpunk --glow 0.8",
    "[ ok ] mounting shaders",
    "[ ok ] calibrating phosphor",
    "[warn] flux capacitor at 88%",
    "[ ok ] starting display manager",
    "",
    "$ cat /etc/motd",
    "wake up, operator. the grid has you.",
];
const COLS: usize = 48;
const WORDS: usize = COLS / 4;
/// Four characters per word (little-endian), so one line is only 12 words.
const fn pack(s: &str) -> [u32; WORDS] {
    let b = s.as_bytes();
    let mut out = [0u32; WORDS];
    let mut i = 0;
    while i < b.len() && i < COLS {
        out[i / 4] |= (b[i] as u32) << ((i % 4) * 8);
        i += 1;
    }
    out
}
const fn build() -> [[u32; WORDS]; 15] {
    let mut out = [[0u32; WORDS]; 15];
    let mut i = 0;
    while i < 15 {
        out[i] = pack(LINES[i]);
        i += 1;
    }
    out
}
const TEXT: [[u32; WORDS]; 15] = build();

/// Character at (`row`, `col`), 0 outside. A `match` on the row keeps the GPU
/// build from copying the whole table for every pixel.
fn text_char(row: i32, col: i32) -> u32 {
    if row < 0 || row >= 15 || col < 0 || col >= COLS as i32 {
        return 0;
    }
    let w = (col / 4) as usize;
    let word = match row {
        0 => TEXT[0][w],
        1 => TEXT[1][w],
        2 => TEXT[2][w],
        3 => TEXT[3][w],
        4 => TEXT[4][w],
        5 => TEXT[5][w],
        6 => TEXT[6][w],
        7 => TEXT[7][w],
        8 => TEXT[8][w],
        9 => TEXT[9][w],
        10 => TEXT[10][w],
        11 => TEXT[11][w],
        12 => TEXT[12][w],
        13 => TEXT[13][w],
        _ => TEXT[14][w],
    };
    (word >> ((col % 4) as u32 * 8)) & 0xff
}

/// A screen of terminal text. Characters are pseudo-glyphs per letter.
pub fn terminal_scene(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let rows = 20.0;
    let cols = (rows * c.aspect() / 0.55).floor();
    let q = vec2(uv.x * cols, uv.y * rows);
    let (cx, cy) = (q.x.floor() as i32, q.y.floor() as i32);
    let f = q.fract();
    let mut col = p.bg;
    let row = cy - 1;
    let ch = match text_char(row, cx - 1) {
        0 => b' ' as u32,
        c => c,
    };
    let (l0, l1, l2) = (text_char(row, 0), text_char(row, 1), text_char(row, 2));
    let tinted = (l0 == b'[' as u32 && l2 == b'o' as u32) || l0 == b'$' as u32;
    let mut fg = p.fg;
    if l0 == b'[' as u32 && l1 == b'w' as u32 {
        fg = p.c;
    } else if tinted {
        fg = p.a;
    }
    if ch != b' ' as u32 && cy >= 1 {
        col = col.mix(fg, glyph(ch as i32 * 31 + 7, f * vec2(1.0, 1.0)));
    }
    // title bar
    if cy == 0 {
        col = p.a * 0.85;
        if ch != b' ' as u32 || (cx > 0 && cx < 30) {
            let t = glyph(hash2(cx, 5) as i32 + cx * 13, f);
            col = col.mix(p.bg, t * step(1.0, cx as f32));
        }
    }
    // prompt + blinking cursor on the line after the text
    let last = (TEXT.len() + 2) as i32;
    if cy == last {
        if cx == 1 {
            col = col.mix(p.a, glyph(36 * 31 + 7, f));
        }
        if cx == 3 && ((c.time * 1.6) as i32 % 2 == 0) {
            col = p.fg;
        }
    }
    col
}

pub fn test_pattern(uv: Vec2, c: &Ctx) -> Vec3 {
    let bars = [
        vec3(0.75, 0.75, 0.75),
        vec3(0.75, 0.75, 0.0),
        vec3(0.0, 0.75, 0.75),
        vec3(0.0, 0.75, 0.0),
        vec3(0.75, 0.0, 0.75),
        vec3(0.75, 0.0, 0.0),
        vec3(0.0, 0.0, 0.75),
    ];
    let mut col = if uv.y < 0.66 {
        bars[((uv.x * 7.0) as usize).min(6)]
    } else if uv.y < 0.75 {
        let rev = [
            bars[6],
            Vec3::BLACK,
            bars[4],
            Vec3::BLACK,
            bars[2],
            Vec3::BLACK,
            bars[0],
        ];
        rev[((uv.x * 7.0) as usize).min(6)]
    } else {
        // -I, white, +Q, black, pluge
        let x = uv.x;
        if x < 0.17 {
            vec3(0.0, 0.13, 0.3)
        } else if x < 0.34 {
            Vec3::WHITE
        } else if x < 0.5 {
            vec3(0.2, 0.0, 0.4)
        } else if x < 0.7 {
            Vec3::BLACK
        } else {
            Vec3::splat(0.05 + 0.1 * ((x - 0.7) / 0.3 * 3.0).floor() / 2.0)
        }
    };
    // moving marker so the animation is visible
    let m = (c.time * 0.3) % 1.0;
    col = col + Vec3::splat(smoothstep(0.01, 0.0, (uv.x - m).abs()) * 0.25 * step(0.9, uv.y));
    col
}

pub fn tv_snow(uv: Vec2, c: &Ctx) -> Vec3 {
    let t = (c.time * 30.0).floor() as i32;
    let n = hash3((uv.x * c.w as f32) as i32, (uv.y * c.h as f32) as i32, t);
    let roll = fract(uv.y - c.time * 0.2);
    let bar = smoothstep(0.0, 0.08, roll) * smoothstep(0.2, 0.08, roll);
    Vec3::splat(n * (0.55 + 0.35 * bar)).mix(c.pal.a * n, 0.25)
}

fn curve(uv: Vec2, k: f32) -> Vec2 {
    let cc = uv * 2.0 - 1.0;
    let cc = cc * (1.0 + cc.dot(cc) * 0.08 * k);
    cc * 0.5 + 0.5
}

fn off_screen(uv: Vec2) -> bool {
    uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0
}

fn blur(src: &Frame, uv: Vec2, r: f32) -> Vec3 {
    let t = vec2(r / src.w as f32, r / src.h as f32);
    (src.sample(uv)
        + src.sample(uv + vec2(t.x, 0.0))
        + src.sample(uv - vec2(t.x, 0.0))
        + src.sample(uv + vec2(0.0, t.y))
        + src.sample(uv - vec2(0.0, t.y)))
        / 5.0
}

pub fn crt(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let k = c.intensity;
    let u = curve(uv, k);
    if off_screen(u) {
        Vec3::BLACK
    } else {
        let ca = 0.0007 * k;
        let base = vec3(
            src.sample(u + vec2(ca, 0.0)).x,
            src.sample(u).y,
            src.sample(u - vec2(ca, 0.0)).z,
        );
        let glowc = blur(src, u, 2.0);
        let mut col = base + glowc * 0.35 * k;
        // scanlines
        let sl = 0.8 + 0.2 * (u.y * src.h as f32 * std::f32::consts::PI).sin().abs();
        col = col * mix(1.0, sl, k);
        // aperture grille
        let px = ((u.x * src.w as f32) as i32).rem_euclid(3);
        let mask = match px {
            0 => vec3(1.0, 0.82, 0.82),
            1 => vec3(0.82, 1.0, 0.82),
            _ => vec3(0.82, 0.82, 1.0),
        };
        col = col * Vec3::WHITE.mix(mask, 0.6 * k);
        // vignette + flicker
        let d = (uv - 0.5) * vec2(1.0, 1.0);
        col = col * (1.0 - 0.9 * d.dot(d) * k) * (1.0 + 0.015 * (c.time * 55.0).sin() * k);
        col * 1.12
    }
}

pub fn phosphor(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let u = curve(uv, c.intensity);
    if off_screen(u) {
        Vec3::BLACK
    } else {
        let l = src.sample(u).luma();
        let halo = blur(src, u, 3.0).luma();
        let tint = c.pal.a;
        let lit = (l * 1.4).min(1.2) + halo * 0.55;
        let sl = 0.78 + 0.22 * (u.y * src.h as f32 * std::f32::consts::PI).sin().abs();
        // slow vertical refresh band
        let band = 0.06 * smoothstep(0.0, 0.2, fract(u.y - c.time * 0.12));
        let col = c.pal.bg + tint * lit * sl + tint * band * 0.4;
        let d = uv - 0.5;
        col * (1.0 - 0.8 * d.dot(d))
    }
}

pub fn vhs(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let k = c.intensity;
    let t = c.time;
    let row = (uv.y * src.h as f32).floor();
    let line_t = (t * 24.0).floor();
    // per-line jitter, stronger near the tracking band
    let band_y = fract(1.0 - t * 0.17);
    let near = smoothstep(0.12, 0.0, (uv.y - band_y).abs());
    let jitter = (hash2(row as i32, line_t as i32) - 0.5) * 0.006
        + near * 0.03 * (hash2(row as i32, 3) - 0.5) * 4.0;
    let u = vec2(uv.x + jitter * k, uv.y);
    let bleed = 0.004 * k;
    let col = vec3(
        src.sample(u + vec2(bleed, 0.0)).x,
        src.sample(u).y,
        src.sample(u - vec2(bleed, 0.0)).z,
    );
    // soft luma blur horizontally
    let soft = (src.sample(u + vec2(0.004, 0.0)) + src.sample(u - vec2(0.004, 0.0))) * 0.5;
    let mut col = col.mix(soft, 0.35 * k);
    // desaturate + warm cast
    let l = col.luma();
    col = Vec3::splat(l).mix(col, 0.8) * vec3(1.05, 1.0, 0.92);
    // grain, tracking noise, head switching at the bottom
    let n = hash3((uv.x * src.w as f32) as i32, row as i32, line_t as i32);
    col = col + Vec3::splat((n - 0.5) * 0.12 * k);
    col = col + Vec3::splat(near * n * 0.5 * k);
    let head = smoothstep(0.94, 1.0, uv.y);
    col = col.mix(Vec3::splat(n), head * 0.5 * k);
    let d = uv - 0.5;
    col * (1.0 - 0.5 * d.dot(d))
}

pub fn tft(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let cell = 3.0;
    let px = uv.x * src.w as f32;
    let py = uv.y * src.h as f32;
    let (cx, cy) = ((px / cell).floor(), (py / cell).floor());
    let center = vec2(
        (cx + 0.5) * cell / src.w as f32,
        (cy + 0.5) * cell / src.h as f32,
    );
    let s = src.sample(center);
    let sub = (px % cell).floor() as i32;
    let m = match sub {
        0 => vec3(1.0, 0.0, 0.0),
        1 => vec3(0.0, 1.0, 0.0),
        _ => vec3(0.0, 0.0, 1.0),
    };
    let lit = s * (m * 2.2 + Vec3::splat(0.15));
    let gap = if (py % cell) < 1.0 { 0.55 } else { 1.0 };
    s.mix(lit * gap, c.intensity)
}

pub fn static_noise(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let t = (c.time * 30.0).floor() as i32;
    let n = hash3(
        (uv.x * src.w as f32) as i32,
        (uv.y * src.h as f32) as i32,
        t,
    );
    src.sample(uv).mix(Vec3::splat(n), 0.35 * c.intensity)
}
