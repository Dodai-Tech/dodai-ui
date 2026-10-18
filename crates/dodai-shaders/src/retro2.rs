//! More retro terminal and monitor variations: phosphor colours, home
//! computers, vector displays, broadcast gear and analogue faults.

use crate::frame::{Ctx, Frame};
use crate::math::*;
use crate::{Family, ShaderInfo};
use std::f32::consts::{PI, TAU};

const RT: Family = Family::Retro;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::post(
        "amber_monitor",
        "Amber monitor",
        RT,
        "Curved amber-phosphor tube with scanlines and halo",
        amber_monitor,
    ),
    ShaderInfo::post(
        "green_phosphor",
        "Green phosphor (P1)",
        RT,
        "Long-persistence green glow and mild flicker",
        green_phosphor,
    ),
    ShaderInfo::post(
        "paper_white",
        "Paper-white mono",
        RT,
        "Black ink on a warm 1-bit paper screen",
        paper_white,
    ),
    ShaderInfo::post(
        "trinitron",
        "Trinitron",
        RT,
        "Aperture-grille stripes, damper wires and a bright tube",
        trinitron,
    ),
    ShaderInfo::post(
        "shadow_mask",
        "Shadow-mask tube",
        RT,
        "Staggered RGB triads like a colour TV",
        shadow_mask,
    ),
    ShaderInfo::post(
        "dot_matrix",
        "Dot matrix",
        RT,
        "Round dots on a dark grid",
        dot_matrix,
    ),
    ShaderInfo::post(
        "vector_glow",
        "Vector display",
        RT,
        "Only the outlines, drawn as bright phosphor lines",
        vector_glow,
    ),
    ShaderInfo::post(
        "c64",
        "Commodore 64",
        RT,
        "The 16 colours, dithered, inside a light-blue border",
        c64,
    ),
    ShaderInfo::post(
        "apple_ii",
        "Apple ][ hi-res",
        RT,
        "Six colours with artifact fringes",
        apple_ii,
    ),
    ShaderInfo::post(
        "zx_spectrum",
        "ZX Spectrum",
        RT,
        "Two colours per 8×8 attribute block",
        zx_spectrum,
    ),
    ShaderInfo::post(
        "ega",
        "EGA",
        RT,
        "16-colour EGA palette with ordered dither",
        ega,
    ),
    ShaderInfo::post(
        "composite_ntsc",
        "Composite NTSC",
        RT,
        "Chroma smear and dot crawl",
        composite_ntsc,
    ),
    ShaderInfo::post(
        "analog_tracking",
        "Tracking error",
        RT,
        "VCR tracking band with noise and shift",
        analog_tracking,
    ),
    ShaderInfo::post(
        "crt_degauss",
        "Degauss",
        RT,
        "A colourful wobble that settles every few seconds",
        crt_degauss,
    ),
    ShaderInfo::generator(
        "phosphor_radar",
        "Phosphor radar",
        RT,
        "Sweep with afterglow and fading blips",
        phosphor_radar,
    ),
    ShaderInfo::generator(
        "vga_text",
        "DOS text mode",
        RT,
        "Blue 80×25 screen with a dialog and blinking cursor",
        vga_text,
    ),
    ShaderInfo::generator(
        "c64_screen",
        "C64 READY.",
        RT,
        "Blue screen, light border, blinking cursor",
        c64_screen,
    ),
    ShaderInfo::generator(
        "zx_loading",
        "ZX loading",
        RT,
        "Red/cyan border stripes while the screen paints in",
        zx_loading,
    ),
    ShaderInfo::generator(
        "smpte_bars",
        "Colour bars",
        RT,
        "SMPTE test card",
        smpte_bars,
    ),
];

// ── shared helpers ──────────────────────────────────────────────────────

/// Barrel distortion; `k` ≈ 0.1–0.3.
pub(crate) fn barrel(uv: Vec2, k: f32) -> Vec2 {
    let c = uv - 0.5;
    c * (1.0 + k * c.dot(c) * 2.0) + 0.5
}

pub(crate) fn inside(uv: Vec2) -> bool {
    (0.0..=1.0).contains(&uv.x) && (0.0..=1.0).contains(&uv.y)
}

/// Dark gaps between scanlines, `period` pixels apart.
pub(crate) fn scan(py: f32, period: f32, depth: f32) -> f32 {
    1.0 - depth * (0.5 + 0.5 * (py * TAU / period).cos())
}

/// A cheap glow: average of 8 taps around `uv`, `r` pixels out.
pub(crate) fn halo(src: &Frame, uv: Vec2, r: f32) -> Vec3 {
    let (tx, ty) = (r / src.w as f32, r / src.h as f32);
    let mut acc = Vec3::BLACK;
    for i in 0..8 {
        let a = i as f32 * TAU / 8.0;
        acc = acc + src.sample(uv + vec2(a.cos() * tx, a.sin() * ty));
    }
    acc / 8.0
}

fn mono(src: &Frame, uv: Vec2, c: &Ctx, tint: Vec3, k: f32, glow_amt: f32, flick: f32) -> Vec3 {
    let u = barrel(uv, k);
    if !inside(u) {
        return Vec3::BLACK;
    }
    let py = u.y * src.h as f32;
    let l = src.sample(u).luma();
    let h = halo(src, u, 2.5).luma();
    let f = 1.0 - flick * (0.5 + 0.5 * (c.time * 47.0).sin()) * c.intensity;
    let v = (l + h * glow_amt) * scan(py, 3.0, 0.35 * c.intensity) * f;
    let edge = ((u - 0.5).dot(u - 0.5) * 1.6).min(1.0);
    tint * v * (1.0 - edge * 0.5) * 1.25
}

fn nearest_in(pal: &[[u8; 3]], c: Vec3) -> Vec3 {
    let mut best = (f32::MAX, Vec3::BLACK);
    for p in pal {
        let v = Vec3::from_u8(*p);
        let d = v - c;
        let dist = d.dot(d);
        if dist < best.0 {
            best = (dist, v);
        }
    }
    best.1
}

fn dithered(src: &Frame, uv: Vec2, amount: f32) -> Vec3 {
    let (x, y) = ((uv.x * src.w as f32) as i32, (uv.y * src.h as f32) as i32);
    (src.get(x, y) + Vec3::splat((bayer4(x, y) - 0.5) * amount)).clamp01()
}

// ── monitors ────────────────────────────────────────────────────────────

pub fn amber_monitor(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    mono(src, uv, c, vec3(1.0, 0.69, 0.0), 0.16, 1.4, 0.04)
}

pub fn green_phosphor(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let u = barrel(uv, 0.1);
    let wide = halo(src, u, 5.0).luma();
    let m = mono(src, uv, c, vec3(0.2, 1.0, 0.35), 0.1, 1.0, 0.06);
    m + vec3(0.05, 0.3, 0.1) * wide * 1.2 * c.intensity
}

pub fn paper_white(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let (x, y) = ((uv.x * src.w as f32) as i32, (uv.y * src.h as f32) as i32);
    let l = src.sample(uv).luma().powf(0.7);
    let on = l + (bayer4(x, y) - 0.5) * 0.5 * c.intensity > 0.5;
    let paper = vec3(0.93, 0.91, 0.84);
    let ink = vec3(0.08, 0.08, 0.1);
    let shade = 1.0 - ((uv - 0.5).dot(uv - 0.5) * 0.5);
    if on {
        paper * shade
    } else {
        ink
    }
}

pub fn trinitron(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let u = barrel(uv, 0.06);
    if !inside(u) {
        return Vec3::BLACK;
    }
    let px = u.x * src.w as f32;
    let s = src.sample(u);
    let stripe = (px % 3.0).floor() as i32;
    let m = match stripe {
        0 => vec3(1.0, 0.55, 0.55),
        1 => vec3(0.55, 1.0, 0.55),
        _ => vec3(0.55, 0.55, 1.0),
    };
    let mut col = vec3(s.x * m.x, s.y * m.y, s.z * m.z) * 1.7 + halo(src, u, 3.0) * 0.25;
    for w in [1.0 / 3.0, 2.0 / 3.0] {
        col = col * (1.0 - 0.45 * smoothstep(0.012, 0.0, (u.y - w).abs()) * c.intensity);
    }
    col
}

pub fn shadow_mask(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let u = barrel(uv, 0.1);
    if !inside(u) {
        return Vec3::BLACK;
    }
    let (px, py) = (u.x * src.w as f32, u.y * src.h as f32);
    let stagger = if (py as i32) % 2 == 0 { 0.0 } else { 1.5 };
    let t = ((px + stagger) % 3.0).floor() as i32;
    let m = [
        vec3(1.0, 0.3, 0.3),
        vec3(0.3, 1.0, 0.3),
        vec3(0.3, 0.3, 1.0),
    ][t as usize % 3];
    let s = src.sample(u);
    let k = c.intensity;
    let strength = |v: f32, mm: f32| v * (1.0 - k + k * mm) * (1.0 + 0.6 * k);
    vec3(strength(s.x, m.x), strength(s.y, m.y), strength(s.z, m.z)) * scan(py, 3.0, 0.2)
}

pub fn dot_matrix(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let cell = (src.h as f32 / 64.0).max(3.0);
    let (px, py) = (uv.x * src.w as f32, uv.y * src.h as f32);
    let (cx, cy) = ((px / cell).floor(), (py / cell).floor());
    let f = vec2((px / cell).fract() - 0.5, (py / cell).fract() - 0.5);
    let s = src.nearest(vec2(
        (cx + 0.5) * cell / src.w as f32,
        (cy + 0.5) * cell / src.h as f32,
    ));
    let r = 0.2 + 0.3 * s.luma().sqrt();
    let on = smoothstep(r, r - 0.18, f.len());
    s * on * (1.0 + 0.4 * c.intensity) + c.pal.bg * 0.5
}

pub fn vector_glow(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let (x, y) = ((uv.x * src.w as f32) as i32, (uv.y * src.h as f32) as i32);
    let l = |dx: i32, dy: i32| src.get(x + dx, y + dy).luma();
    let gx = -l(-1, -1) - 2.0 * l(-1, 0) - l(-1, 1) + l(1, -1) + 2.0 * l(1, 0) + l(1, 1);
    let gy = -l(-1, -1) - 2.0 * l(0, -1) - l(1, -1) + l(-1, 1) + 2.0 * l(0, 1) + l(1, 1);
    let e = ((gx * gx + gy * gy).sqrt() * 1.4).min(1.0);
    let tint = vec3(0.7, 1.0, 0.85);
    tint * e * (0.8 + 0.4 * c.intensity) + c.pal.bg * 0.6
}

// ── home computers ──────────────────────────────────────────────────────

const C64: [[u8; 3]; 16] = [
    [0, 0, 0],
    [255, 255, 255],
    [136, 0, 0],
    [170, 255, 238],
    [204, 68, 204],
    [0, 204, 85],
    [0, 0, 170],
    [238, 238, 119],
    [221, 136, 85],
    [102, 68, 0],
    [255, 119, 119],
    [51, 51, 51],
    [119, 119, 119],
    [170, 255, 102],
    [0, 136, 255],
    [187, 187, 187],
];
const EGA: [[u8; 3]; 16] = [
    [0, 0, 0],
    [0, 0, 170],
    [0, 170, 0],
    [0, 170, 170],
    [170, 0, 0],
    [170, 0, 170],
    [170, 85, 0],
    [170, 170, 170],
    [85, 85, 85],
    [85, 85, 255],
    [85, 255, 85],
    [85, 255, 255],
    [255, 85, 85],
    [255, 85, 255],
    [255, 255, 85],
    [255, 255, 255],
];
const ZX: [[u8; 3]; 8] = [
    [0, 0, 0],
    [0, 0, 215],
    [215, 0, 0],
    [215, 0, 215],
    [0, 215, 0],
    [0, 215, 215],
    [215, 215, 0],
    [215, 215, 215],
];

pub fn c64(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let (bx, by) = (0.06, 0.09);
    let border = uv.x < bx || uv.x > 1.0 - bx || uv.y < by || uv.y > 1.0 - by;
    if border {
        return Vec3::from_u8([136, 126, 203]);
    }
    let u = vec2(
        (uv.x - bx) / (1.0 - 2.0 * bx),
        (uv.y - by) / (1.0 - 2.0 * by),
    );
    nearest_in(&C64, dithered(src, u, 0.25 * c.intensity))
}

pub fn ega(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    nearest_in(&EGA, dithered(src, uv, 0.22 * c.intensity))
}

pub fn apple_ii(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let (x, y) = ((uv.x * src.w as f32) as i32, (uv.y * src.h as f32) as i32);
    let s = src.get(x, y);
    let l = (s.luma() + (bayer4(x, y) - 0.5) * 0.3 * c.intensity).clamp(0.0, 1.0);
    let odd = x % 2 == 1;
    let warm = s.x >= s.z;
    if l > 0.8 {
        vec3(1.0, 1.0, 1.0)
    } else if l > 0.2 {
        match (odd, warm) {
            (false, true) => Vec3::from_u8([255, 68, 253]),
            (true, true) => Vec3::from_u8([20, 245, 60]),
            (false, false) => Vec3::from_u8([20, 207, 253]),
            (true, false) => Vec3::from_u8([255, 106, 60]),
        }
    } else {
        Vec3::BLACK
    }
}

pub fn zx_spectrum(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let (bw, bh) = (src.w as f32 / 32.0, src.h as f32 / 24.0);
    let (px, py) = (uv.x * src.w as f32, uv.y * src.h as f32);
    let (x0, y0) = ((px / bw).floor() * bw, (py / bh).floor() * bh);
    let (mut lo, mut hi) = (
        src.nearest(vec2(x0 / src.w as f32, y0 / src.h as f32)),
        Vec3::BLACK,
    );
    let mut lo_l = lo.luma();
    let mut hi_l = -1.0;
    for i in 0..4 {
        let u = vec2(
            (x0 + bw * ((i % 2) as f32 * 0.8 + 0.1)) / src.w as f32,
            (y0 + bh * ((i / 2) as f32 * 0.8 + 0.1)) / src.h as f32,
        );
        let s = src.nearest(u);
        if s.luma() > hi_l {
            hi_l = s.luma();
            hi = s;
        }
        if s.luma() < lo_l {
            lo_l = s.luma();
            lo = s;
        }
    }
    let bright = hi_l > 0.55;
    let k = if bright { 255.0 / 215.0 } else { 1.0 };
    let ink = nearest_in(&ZX, hi) * k;
    let paper = nearest_in(&ZX, lo * 0.7);
    let (x, y) = (px as i32, py as i32);
    let l = src.get(x, y).luma() + (bayer4(x, y) - 0.5) * 0.3 * c.intensity;
    if l > (hi_l + lo_l) * 0.5 {
        ink.clamp01()
    } else {
        paper
    }
}

// ── broadcast / faults ──────────────────────────────────────────────────

pub fn composite_ntsc(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let tx = 1.0 / src.w as f32;
    let yiq = |s: Vec3| {
        (
            0.299 * s.x + 0.587 * s.y + 0.114 * s.z,
            0.596 * s.x - 0.274 * s.y - 0.322 * s.z,
            0.211 * s.x - 0.523 * s.y + 0.312 * s.z,
        )
    };
    let (y0, _, _) = yiq(src.sample(uv));
    let (mut i, mut q, mut n) = (0.0, 0.0, 0.0);
    for k in -4..=4 {
        let (_, ii, qq) =
            yiq(src.sample(uv + vec2(k as f32 * tx * 1.3 * c.intensity.max(0.2), 0.0)));
        let w = 1.0 - (k as f32).abs() / 5.0;
        i += ii * w;
        q += qq * w;
        n += w;
    }
    let (px, py) = (uv.x * src.w as f32, uv.y * src.h as f32);
    let crawl = ((px * PI * 0.5 + py * PI + c.time * 20.0).sin()) * 0.025 * c.intensity;
    let y = y0 + crawl;
    let (i, q) = (i / n, q / n);
    vec3(
        y + 0.956 * i + 0.621 * q,
        y - 0.272 * i - 0.647 * q,
        y - 1.106 * i + 1.703 * q,
    )
    .clamp01()
}

pub fn analog_tracking(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let k = c.intensity;
    let band_c = 1.0 - (c.time * 0.12).fract();
    let d = uv.y - band_c;
    let in_band = smoothstep(0.1, 0.0, d.abs());
    let row = (uv.y * src.h as f32) as i32;
    let shift = (hash2(row, (c.time * 20.0) as i32) - 0.5) * 0.12 * in_band * k
        + (uv.y * 8.0 + c.time * 3.0).sin() * 0.004 * k;
    let s = src.sample(vec2(uv.x + shift, uv.y));
    let snow = hash3((uv.x * src.w as f32) as i32, row, (c.time * 30.0) as i32);
    let tint = vec3(s.x, s.y, s.z).mix(Vec3::splat(s.luma()), in_band * 0.6);
    tint.mix(Vec3::splat(snow), in_band * 0.45 * k)
}

pub fn crt_degauss(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let t = c.time % 9.0;
    let decay = (-t * 1.6).exp() * c.intensity;
    let r = (uv - 0.5).len();
    let w = (r * 30.0 - t * 25.0).sin() * 0.03 * decay;
    let a = (uv.y - 0.5).atan2(uv.x - 0.5) + t * 4.0;
    let off = vec2(a.cos(), a.sin()) * w;
    let s = vec3(
        src.sample(uv + off * 1.4).x,
        src.sample(uv + off * 0.4).y,
        src.sample(uv - off).z,
    );
    s + hsv((r * 2.0 - t).fract(), 0.8, 1.0) * (decay * 0.12 * smoothstep(0.0, 0.5, r))
}

// ── generators ──────────────────────────────────────────────────────────

pub fn phosphor_radar(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let g = vec3(0.25, 1.0, 0.4);
    let r = q.len();
    let sweep = c.time * 1.3;
    let mut a = (q.y.atan2(q.x) - sweep) % TAU;
    if a < 0.0 {
        a += TAU;
    }
    let behind = TAU - a; // how long ago the beam passed
    let trail = (-(behind) * 0.9).exp();
    let disk = smoothstep(0.92, 0.9, r);
    let mut v = trail * 0.55 * disk;
    for ring in 1..=4 {
        v += glow((r - ring as f32 * 0.22).abs() * 6.0, 1.0) * 0.18 * disk;
    }
    v += (glow(q.x.abs() * 40.0, 1.0) + glow(q.y.abs() * 40.0, 1.0)) * 0.1 * disk;
    for i in 0..7 {
        let (ba, br) = (hash1(i as f32) * TAU, 0.2 + hash1(i as f32 + 9.0) * 0.65);
        let bp = vec2(ba.cos(), ba.sin()) * br;
        let mut since = (sweep - ba) % TAU;
        if since < 0.0 {
            since += TAU;
        }
        v += (-since * 0.7).exp() * smoothstep(0.05, 0.0, (q - bp).len()) * 1.3;
    }
    g * v + c.pal.bg * 0.5 + g * glow((r - 0.9).abs() * 12.0, 1.0) * 0.4
}

/// A hashed 5×7 "letter" in cell-local `f` (0..1), stable per `id`.
fn pseudo_char(id: i32, f: Vec2) -> f32 {
    if f.x < 0.15 || f.x > 0.85 || f.y < 0.1 || f.y > 0.9 {
        return 0.0;
    }
    let (bx, by) = (
        ((f.x - 0.15) / 0.7 * 5.0) as i32,
        ((f.y - 0.1) / 0.8 * 7.0) as i32,
    );
    let h = hash_u(id as u32 ^ 0x9e37_79b9);
    let bits =
        (h * 4_294_967_296.0) as u64 ^ ((hash_u(id as u32 + 77) * 4_294_967_296.0) as u64) << 20;
    if (bits >> ((by * 5 + bx) % 35)) & 1 == 1 && hash2(id, by) > 0.18 {
        1.0
    } else {
        0.0
    }
}

pub fn vga_text(uv: Vec2, c: &Ctx) -> Vec3 {
    let (cols, rows) = (80.0, 25.0);
    let (gx, gy) = ((uv.x * cols).floor(), (uv.y * rows).floor());
    let f = vec2((uv.x * cols).fract(), (uv.y * rows).fract());
    let bg = vec3(0.0, 0.0, 0.67);
    let grey = vec3(0.67, 0.67, 0.67);
    // dialog
    let (dx0, dx1, dy0, dy1) = (22.0, 58.0, 6.0, 18.0);
    let in_dlg = gx >= dx0 && gx < dx1 && gy >= dy0 && gy < dy1;
    if gy < 1.0 {
        let on = pseudo_char(gx as i32 + 3, f) > 0.0 && gx < 40.0;
        return if on { Vec3::BLACK } else { grey };
    }
    if in_dlg {
        let border = gx == dx0 || gx == dx1 - 1.0 || gy == dy0 || gy == dy1 - 1.0;
        if border {
            return vec3(0.33, 1.0, 1.0).mix(
                bg,
                1.0 - sat(1.0 - ((f.x - 0.5).abs().min((f.y - 0.5).abs())) * 8.0),
            );
        }
        let line = gy as i32 % 2 == 0 && gx > dx0 + 1.0 && gx < dx1 - 2.0 - hash1(gy) * 12.0;
        let on = line && pseudo_char(gx as i32 * 31 + gy as i32 * 7, f) > 0.0;
        let dlg = vec3(0.0, 0.67, 0.67);
        return if on { Vec3::BLACK } else { dlg };
    }
    let text_line = hash1(gy * 3.1) > 0.35 && gx < 8.0 + hash1(gy) * 55.0 && gx > 1.0;
    let on = text_line && pseudo_char(gx as i32 + gy as i32 * 91, f) > 0.0;
    let cur = gy == 23.0 && gx == 2.0 && (c.time * 2.0) as i32 % 2 == 0 && f.y > 0.8;
    if on || cur {
        grey
    } else {
        bg
    }
}

pub fn c64_screen(uv: Vec2, c: &Ctx) -> Vec3 {
    let (bx, by) = (0.06, 0.09);
    let border = Vec3::from_u8([136, 126, 203]);
    if uv.x < bx || uv.x > 1.0 - bx || uv.y < by || uv.y > 1.0 - by {
        return border;
    }
    let u = vec2(
        (uv.x - bx) / (1.0 - 2.0 * bx),
        (uv.y - by) / (1.0 - 2.0 * by),
    );
    let (gx, gy) = ((u.x * 40.0).floor(), (u.y * 25.0).floor());
    let f = vec2((u.x * 40.0).fract(), (u.y * 25.0).fract());
    let bg = Vec3::from_u8([53, 40, 121]);
    let fg = border;
    let lines: [(f32, f32); 5] = [
        (1.0, 38.0),
        (3.0, 31.0),
        (4.0, 38.0),
        (6.0, 6.0),
        (9.0, 6.0),
    ];
    let on = lines
        .iter()
        .any(|(row, len)| gy == *row && gx >= 1.0 && gx < 1.0 + *len)
        && pseudo_char(gx as i32 * 13 + gy as i32 * 5, f) > 0.0;
    let cursor = gy == 10.0 && gx == 0.0 && (c.time * 1.6) as i32 % 2 == 0;
    if on || cursor {
        fg
    } else {
        bg
    }
}

pub fn zx_loading(uv: Vec2, c: &Ctx) -> Vec3 {
    let t = c.time % 8.0;
    let (bx, by) = (0.1, 0.12);
    let inner = uv.x > bx && uv.x < 1.0 - bx && uv.y > by && uv.y < 1.0 - by;
    if !inner {
        let band = ((uv.y * 90.0 + t * 400.0 * hash1((c.time * 40.0).floor())) as i32 % 6).abs()
            as f32
            / 6.0
            + hash1((uv.y * 40.0).floor() + (c.time * 25.0).floor()) * 0.5;
        return if band > 0.5 {
            vec3(0.85, 0.0, 0.0)
        } else {
            vec3(0.0, 0.85, 0.85)
        }
        .mix(vec3(0.85, 0.85, 0.0), if band > 0.97 { 1.0 } else { 0.0 });
    }
    let u = vec2(
        (uv.x - bx) / (1.0 - 2.0 * bx),
        (uv.y - by) / (1.0 - 2.0 * by),
    );
    let progress = (t / 6.0).min(1.0);
    if u.y > progress {
        return vec3(0.75, 0.75, 0.75);
    }
    let (cx, cy) = ((u.x * 32.0).floor(), (u.y * 24.0).floor());
    let ink = ZX[1 + (hash2(cx as i32, cy as i32) * 7.0) as usize % 7];
    let on =
        hash2(((u.x * 256.0) as i32) / 2, ((u.y * 192.0) as i32) / 2) > 0.5 + 0.25 * (u.y - 0.5);
    if on {
        Vec3::from_u8(ink)
    } else {
        Vec3::BLACK
    }
}

pub fn smpte_bars(uv: Vec2, _c: &Ctx) -> Vec3 {
    let top = [
        vec3(0.75, 0.75, 0.75),
        vec3(0.75, 0.75, 0.0),
        vec3(0.0, 0.75, 0.75),
        vec3(0.0, 0.75, 0.0),
        vec3(0.75, 0.0, 0.75),
        vec3(0.75, 0.0, 0.0),
        vec3(0.0, 0.0, 0.75),
    ];
    let mid = [
        vec3(0.0, 0.0, 0.75),
        Vec3::BLACK,
        vec3(0.75, 0.0, 0.75),
        Vec3::BLACK,
        vec3(0.0, 0.75, 0.75),
        Vec3::BLACK,
        vec3(0.75, 0.75, 0.75),
    ];
    let i = ((uv.x * 7.0) as usize).min(6);
    if uv.y < 0.67 {
        top[i]
    } else if uv.y < 0.75 {
        mid[i]
    } else {
        let u = uv.x * 6.0;
        if u < 1.0 {
            vec3(0.0, 0.2, 0.4)
        } else if u < 2.0 {
            Vec3::WHITE
        } else if u < 3.0 {
            vec3(0.2, 0.0, 0.4)
        } else if u < 4.5 {
            Vec3::splat(0.07)
        } else if u < 5.0 {
            Vec3::BLACK
        } else if u < 5.33 {
            Vec3::splat(0.03)
        } else {
            Vec3::splat(0.11)
        }
    }
}
