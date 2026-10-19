//! Rust takes on the effects in the
//! [ghostty-shaders](https://github.com/0xhckr/ghostty-shaders) collection.
//!
//! These are independent re-implementations of the *looks* (CRT warp and
//! scanlines, Lottes-style beam and mask, in-game CRT, RGB-split glow, drunk
//! wobble, glitch loop, underwater rays, cubes, sparks, smoke, lava, gears,
//! the rune city). No GLSL is copied. Credit for the ideas goes to the
//! original authors: the Shadertoy "retro terminal" CRT, Timothy Lottes
//! (CRTS), qwerasd205, moni-dz (drunkard), kalgynirae (glow), Jan Mróz
//! (sparks), FabriceNeyret2 and otaviogood (runes), rymdlego (cubes),
//! unkn0wncode (gradients), Alex Sherwin, pkazmier and others.
//!
//! Most are *overlays*: they work on a terminal-like picture, so they are
//! post effects previewed over the terminal scene.

use crate::frame::{Ctx, Frame};
use crate::math::*;
#[allow(unused_imports)]
use crate::prelude::*;
use crate::retro2::{barrel, halo, inside, scan};
use crate::{Family, ShaderInfo};
use std::f32::consts::{PI, TAU};

const GT: Family = Family::Ghostty;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::post(
        "gt_retro_terminal",
        "Retro terminal",
        GT,
        "Warped glass, scanlines and a teal tint",
        gt_retro_terminal,
    ),
    ShaderInfo::post(
        "gt_better_crt",
        "Better CRT",
        GT,
        "The same curved tube with the tint removed",
        gt_better_crt,
    ),
    ShaderInfo::post(
        "gt_crt_lottes",
        "Lottes CRT",
        GT,
        "Beam-shaped scanlines, RGB mask, curvature, bloom",
        gt_crt_lottes,
    ),
    ShaderInfo::post(
        "gt_in_game_crt",
        "In-game CRT",
        GT,
        "Fringing, ghosting, grille, flicker, noise, vignette",
        gt_in_game_crt,
    ),
    ShaderInfo::post(
        "gt_glow_rgb",
        "Glow + RGB split",
        GT,
        "Twitchy channel split and a perceptual (Oklab) glow",
        gt_glow_rgb,
    ),
    ShaderInfo::post(
        "gt_drunkard",
        "Drunkard",
        GT,
        "Fractal noise wobble with chromatic aberration",
        gt_drunkard,
    ),
    ShaderInfo::post(
        "gt_glitchy",
        "Glitchy",
        GT,
        "A loop that calm-waits, then tears the picture",
        gt_glitchy,
    ),
    ShaderInfo::post(
        "gt_underwater",
        "Underwater",
        GT,
        "Sun rays through the dark parts of the screen",
        gt_underwater,
    ),
    ShaderInfo::post(
        "gt_cubes",
        "Cubes",
        GT,
        "A field of tumbling cubes behind the text",
        gt_cubes,
    ),
    ShaderInfo::post(
        "gt_sparks",
        "Sparks from fire",
        GT,
        "Glowing embers drifting up past the text",
        gt_sparks,
    ),
    ShaderInfo::post(
        "gt_smoke",
        "Smoke",
        GT,
        "Wisps rising from every glyph",
        gt_smoke,
    ),
    ShaderInfo::post(
        "gt_lava",
        "Cine lava",
        GT,
        "Slow colour-cycling lava blobs in the background",
        gt_lava,
    ),
    ShaderInfo::post(
        "gt_gradient_bg",
        "Gradient background",
        GT,
        "A diagonal gradient where the screen is dark",
        gt_gradient_bg,
    ),
    ShaderInfo::post(
        "gt_animated_gradient",
        "Animated gradient",
        GT,
        "The same, slowly cycling through three colours",
        gt_animated_gradient,
    ),
    ShaderInfo::generator(
        "gt_gears",
        "Gears and belts",
        GT,
        "Meshing gears of alternating spin",
        gt_gears,
    ),
    ShaderInfo::generator(
        "gt_inside_matrix",
        "Inside the Matrix",
        GT,
        "Flying through layers of falling runes",
        gt_inside_matrix,
    ),
    ShaderInfo::generator(
        "gt_noise_flow",
        "Noise flow",
        GT,
        "Domain-warped noise, tonal colours",
        gt_noise_flow,
    ),
];

/// 1 where the picture is dark enough to draw behind it, with a soft edge.
fn dark_mask(s: Vec3, thr: f32) -> f32 {
    1.0 - smoothstep(thr * 0.5, thr, s.luma())
}

// ── CRT looks ───────────────────────────────────────────────────────────

fn warp_scan(src: &Frame, uv: Vec2, c: &Ctx, tint: Option<Vec3>) -> Vec3 {
    let warp = 0.25 * c.intensity.max(0.1);
    let dc = (vec2(0.5, 0.5) - uv).abs();
    let (dx, dy) = (dc.x * dc.x, dc.y * dc.y);
    let u = vec2(
        (uv.x - 0.5) * (1.0 + dy * 0.3 * warp) + 0.5,
        (uv.y - 0.5) * (1.0 + dx * 0.4 * warp) + 0.5,
    );
    if !inside(u) {
        Vec3::BLACK
    } else {
        let s = src.sample(u) * scan(u.y * src.h as f32, 3.0, 0.5 * c.intensity);
        match tint {
            Some(t) => vec3(s.x * t.x, s.y * t.y, s.z * t.z) * 1.35 + t * s.luma() * 0.15,
            None => s,
        }
    }
}

pub fn gt_retro_terminal(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    warp_scan(src, uv, c, Some(vec3(0.0, 0.8, 0.6)))
}

pub fn gt_better_crt(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    warp_scan(src, uv, c, None)
}

pub fn gt_crt_lottes(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let u = barrel(uv, 0.12);
    if !inside(u) {
        Vec3::BLACK
    } else {
        let (px, py) = (u.x * src.w as f32, u.y * src.h as f32);
        let s = src.sample(u);
        // Beam: a gaussian across the scanline, wider where the pixel is bright.
        let y = (py.fract() - 0.5).abs() * 2.0;
        let sigma = 0.45 + 0.7 * s.luma();
        let beam = (-(y * y) / (2.0 * sigma * sigma * 0.35)).exp();
        // Shadow mask: RGB triads, 3 px wide.
        let t = (px as i32).rem_euclid(3);
        let mask = [
            vec3(1.0, 0.7, 0.7),
            vec3(0.7, 1.0, 0.7),
            vec3(0.7, 0.7, 1.0),
        ][t as usize];
        let col = vec3(s.x * mask.x, s.y * mask.y, s.z * mask.z) * beam * 1.5;
        let bloom = halo(src, u, 4.0) * 0.35;
        let vig = 1.0 - ((u - 0.5).dot(u - 0.5) * 1.3).min(0.8);
        (col + bloom).mix(s, 1.0 - c.intensity) * vig
    }
}

pub fn gt_in_game_crt(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let u = barrel(uv, 0.18);
    if !inside(u) {
        Vec3::BLACK
    } else {
        let k = c.intensity;
        let tx = 1.0 / src.w as f32;
        // Colour fringing and ghosting.
        let spread = 1.2 * tx * k;
        let mut s = vec3(
            src.sample(u + vec2(spread, 0.0)).x,
            src.sample(u).y,
            src.sample(u - vec2(spread, 0.0)).z,
        );
        s = s + src.sample(u - vec2(6.0 * tx, 0.0)) * 0.1 * k;
        s = s.mix(s * 0.6, 0.4);
        let (px, py) = (u.x * src.w as f32, u.y * src.h as f32);
        let sl = 1.0 - 0.2 * k * (0.65 + 0.35 * (py * TAU / 4.0).cos());
        let grille = 1.0 - 0.3 * k * (0.5 + 0.5 * (px * PI).cos());
        let flick = 1.0 + 0.04 * k * (c.time * 15.0 * TAU).sin();
        let n = hash3(px as i32, py as i32, (c.time * 30.0) as i32) - 0.5;
        let vig = 1.0 - smoothstep(0.2, 0.75, (u - 0.5).len() * 1.4) * 0.9;
        let tint = vec3(0.93, 1.0, 0.96);
        let g = vec3(s.x * tint.x, s.y * tint.y, s.z * tint.z) * sl * grille * flick;
        (g + halo(src, u, 6.0) * 0.18 * k + Vec3::splat(n * 0.12 * k)) * vig * 1.9
    }
}

// ── Colour and distortion ───────────────────────────────────────────────

fn srgb_to_lin(x: f32) -> f32 {
    if x >= 0.04045 {
        ((x + 0.055) / 1.055).powf(2.4)
    } else {
        x / 12.92
    }
}
fn lin_to_srgb(x: f32) -> f32 {
    if x >= 0.003_130_8 {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    } else {
        12.92 * x
    }
}
/// sRGB → Oklab (Björn Ottosson's published matrices).
fn to_oklab(c: Vec3) -> Vec3 {
    let (r, g, b) = (srgb_to_lin(c.x), srgb_to_lin(c.y), srgb_to_lin(c.z));
    let l = (0.412_221_47 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    vec3(
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    )
}
fn from_oklab(c: Vec3) -> Vec3 {
    let l = (c.x + 0.396_337_78 * c.y + 0.215_803_76 * c.z).powi(3);
    let m = (c.x - 0.105_561_346 * c.y - 0.063_854_17 * c.z).powi(3);
    let s = (c.x - 0.089_484_18 * c.y - 1.291_485_5 * c.z).powi(3);
    vec3(
        lin_to_srgb(4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_93 * s).clamp(0.0, 1.0),
        lin_to_srgb(-1.268_438 * l + 2.609_757_4 * m - 0.341_319_4 * s).clamp(0.0, 1.0),
        lin_to_srgb(-0.004_196_086 * l - 0.703_418_6 * m + 1.707_614_7 * s).clamp(0.0, 1.0),
    )
}

pub fn gt_glow_rgb(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let k = c.intensity;
    let amt = (c.time * 3.0).sin() * (c.time * 7.3 + 1.0).sin() * 0.012 * k;
    let s = vec3(
        src.sample(uv + vec2(amt, 0.0)).x,
        src.sample(uv).y,
        src.sample(uv - vec2(amt, 0.0)).z,
    );
    // Glow in a perceptual space: blur lightness only, keep the hue.
    let mut lab = to_oklab(s);
    let mut acc = 0.0;
    let (tx, ty) = (4.0 / src.w as f32, 4.0 / src.h as f32);
    for i in 0..8 {
        let a = i as f32 * TAU / 8.0;
        // Oklab lightness ≈ cbrt of linear luma; the cheap form is enough here.
        acc += srgb_to_lin(src.sample(uv + vec2(a.cos() * tx, a.sin() * ty)).luma()).cbrt();
    }
    lab.x += (acc / 8.0) * 0.45 * k;
    from_oklab(lab)
}

pub fn gt_drunkard(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let p = uv * 1.4 + vec2(c.time * 0.4, c.time * 0.3);
    let n = (fbm(p, 6) - 0.5) * 0.14 * c.intensity;
    let n2 = (fbm(p + vec2(7.3, 1.9), 6) - 0.5) * 0.14 * c.intensity;
    vec3(
        src.sample(uv + vec2(n * 1.1, n2 * 1.1)).x,
        src.sample(uv + vec2(n, n2)).y,
        src.sample(uv + vec2(n * 0.9, n2 * 0.9)).z,
    )
}

pub fn gt_glitchy(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let loop_t = (c.time % 10.0) / 10.0;
    let active = smoothstep(0.0, 0.01, loop_t) * smoothstep(0.1, 0.09, loop_t) * c.intensity;
    if active <= 0.0 {
        return src.sample(uv);
    }
    let frame = (c.time * 24.0).floor();
    let band = (uv.y * 14.0).floor();
    let tear = (hash2(band as i32, frame as i32) - 0.5)
        * 0.25
        * active
        * if hash2(band as i32 + 50, frame as i32) > 0.55 {
            1.0
        } else {
            0.0
        };
    let split = 0.012 * active;
    let u = vec2(uv.x + tear, uv.y);
    let s = vec3(
        src.sample(u + vec2(split, 0.0)).x,
        src.sample(u).y,
        src.sample(u - vec2(split, 0.0)).z,
    );
    let block = hash2((uv.x * 9.0) as i32 + frame as i32, (uv.y * 6.0) as i32) > 0.97;
    if block && active > 0.3 {
        Vec3::splat(1.0) - s
    } else {
        s
    }
}

// ── Overlays on the dark parts of the screen ────────────────────────────

fn ray(src_xy: Vec2, dir: Vec2, p: Vec2, seed: (f32, f32), speed: f32, t: f32) -> f32 {
    let d = p - src_xy;
    let cosa = d.dot(dir) / d.len().max(1e-4);
    let r = (0.45 + 0.15 * (cosa * seed.0 + t * speed).sin())
        + (0.3 + 0.2 * (-cosa * seed.1 + t * speed).cos());
    sat(r) * mix(0.5, 1.0, sat(1.0 - d.len() / 1.8))
}

pub fn gt_underwater(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let p = vec2(uv.x, 1.0 - uv.y);
    let a = ray(
        vec2(0.7, 1.1),
        vec2(1.0, 0.116),
        p,
        (36.22, 21.11),
        1.1,
        c.time,
    );
    let b = ray(
        vec2(0.8, 1.2),
        vec2(1.0, -0.241),
        p,
        (22.4, 18.02),
        0.9,
        c.time,
    );
    let rays = a * 0.5 + b * 0.4;
    let depth = 1.0 - p.y;
    let col = vec3(
        rays * (0.05 + depth * 0.8),
        rays * (0.15 + depth * 0.6),
        rays * (0.3 + depth * 0.5),
    ) * 0.55;
    s.mix(
        col * c.intensity + s * (1.0 - c.intensity),
        dark_mask(s, 0.4),
    )
}

fn rot3(v: [f32; 3], axis: [f32; 3], a: f32) -> [f32; 3] {
    let n = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    let k = [axis[0] / n, axis[1] / n, axis[2] / n];
    let (s, co) = a.sin_cos();
    let d = k[0] * v[0] + k[1] * v[1] + k[2] * v[2];
    let cr = [
        k[1] * v[2] - k[2] * v[1],
        k[2] * v[0] - k[0] * v[2],
        k[0] * v[1] - k[1] * v[0],
    ];
    [
        v[0] * co + cr[0] * s + k[0] * d * (1.0 - co),
        v[1] * co + cr[1] * s + k[1] * d * (1.0 - co),
        v[2] * co + cr[2] * s + k[2] * d * (1.0 - co),
    ]
}

pub fn gt_cubes(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let m = dark_mask(s, 0.4);
    if m < 0.01 {
        return s;
    }
    let t = c.time * 0.2;
    let q = c.centered(uv);
    let origin = [t * 1.0, 0.0, 0.0];
    let fwd = [1.0, 0.0, 0.0];
    let dir_raw = [fwd[0], q.y * 0.6, q.x * 0.6];
    let dn = (dir_raw[0] * dir_raw[0] + dir_raw[1] * dir_raw[1] + dir_raw[2] * dir_raw[2]).sqrt();
    let dir = rot3(
        [dir_raw[0] / dn, dir_raw[1] / dn, dir_raw[2] / dn],
        [0.0, 0.0, 1.0],
        t * 0.5,
    );
    let size = 0.32 + 0.1 * (c.time * 0.05).sin().abs();
    let axis = [(t / 4.0).sin() * 10.0, (t / 4.0).cos() * 12.0, 2.7];
    let ang = t * 0.6 * 2.8;
    let dist = |p: [f32; 3]| {
        let cell = |v: f32| (v % 5.0 + 5.0) % 5.0 - 2.5;
        let r = rot3([cell(p[0]), cell(p[1]), cell(p[2])], axis, ang);
        let d = [r[0].abs() - size, r[1].abs() - size, r[2].abs() - size];
        (d[0].max(0.0).powi(2) + d[1].max(0.0).powi(2) + d[2].max(0.0).powi(2)).sqrt()
    };
    let (mut p, mut total) = (origin, 0.0);
    let mut hit = false;
    for _ in 0..32 {
        let d = dist(p);
        if d < 0.01 {
            hit = true;
            break;
        }
        total += d;
        if total > 24.0 {
            break;
        }
        p = [
            origin[0] + dir[0] * total,
            origin[1] + dir[1] * total,
            origin[2] + dir[2] * total,
        ];
    }
    if !hit {
        return s;
    }
    let shade = (1.0 - total / 24.0).max(0.0);
    let col = vec3(
        q.x.abs() * 0.8,
        q.y.abs() * 0.8,
        0.5 + 0.5 * (c.time * 2.0).sin(),
    ) * shade
        * 0.5
        + Vec3::splat(shade * 0.15);
    s + col * (m * c.intensity)
}

pub fn gt_sparks(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let m = dark_mask(s, 0.5);
    let p = vec2(uv.x * c.aspect(), 1.0 - uv.y);
    let mut e = 0.0;
    for layer in 0..6 {
        let lf = layer as f32;
        let sc = 6.0 + lf * 4.0;
        let g = vec2(
            p.x * sc + (c.time * 0.5 + lf).sin() * 0.4,
            p.y * sc - c.time * (0.6 + lf * 0.12),
        );
        let id = g.floor();
        let f = g.fract() - 0.5;
        let h = hash2(id.x as i32 + layer * 17, id.y as i32);
        if h > 0.7 {
            let o = vec2(
                hash2(id.x as i32, id.y as i32 + 5) - 0.5,
                hash2(id.x as i32 + 3, id.y as i32) - 0.5,
            ) * 0.6;
            let d = (f - o).len();
            let life = fract(h * 13.0 + c.time * 0.8);
            e += smoothstep(0.1 - lf * 0.01, 0.0, d) * (1.0 - life) * (1.0 - lf * 0.1);
        }
    }
    let haze = fbm(p * 3.0 + vec2(0.0, -c.time * 0.4), 4) * (1.0 - p.y) * 0.18;
    let fire = vec3(1.0, 0.4, 0.05);
    s + fire * ((e * 1.5 + haze) * m * c.intensity)
}

pub fn gt_smoke(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let mut acc = 0.0;
    for i in 1..=10 {
        let h = i as f32 * 0.012;
        let wob = (fbm(vec2(uv.x * 12.0 + i as f32, uv.y * 9.0 + c.time * 0.5), 3) - 0.5)
            * 0.08
            * (0.3 + h * 8.0);
        let t = src.sample(vec2(uv.x + wob, uv.y + h));
        acc += t.luma() * (1.0 - i as f32 / 11.0);
    }
    let smoke = Vec3::splat(acc * 0.1 * c.intensity);
    s + smoke * (1.0 - s.luma())
}

pub fn gt_lava(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let m = dark_mask(s, 0.4);
    if m < 0.01 {
        return s;
    }
    let q = c.centered(uv);
    let field = |p: Vec2| {
        let mut f = 0.0;
        for i in 0..10 {
            let fi = i as f32;
            let t = c.time * 0.1 * (fract(fi * 0.412 + 0.513) - 0.5) * 2.0;
            let pos = vec2((t + fi * 5.2).sin() * 1.3, (t * 1.3 + fi * 6.4).cos() * 0.8);
            let r = 0.25 + 0.15 * fract(fi * 0.731);
            f += r * r / ((p - pos).dot(p - pos) + 0.01);
        }
        f
    };
    let f = field(q);
    let e = 0.01;
    let n = vec2(
        field(q + vec2(e, 0.0)) - field(q - vec2(e, 0.0)),
        field(q + vec2(0.0, e)) - field(q - vec2(0.0, e)),
    );
    let light = sat(0.5 + n.dot(vec2(-0.5, 0.8)) * 0.02);
    let hue = (c.time * 0.03 + f * 0.05).fract();
    let lava = hsv(hue, 0.75, 0.2 + 0.7 * light) * smoothstep(0.9, 1.3, f);
    let bg = hsv(hue + 0.05, 0.6, 0.05);
    s + (lava + bg) * (0.45 * m * c.intensity)
}

pub fn gt_gradient_bg(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let g = vec3(0.1, 0.1, 0.5).mix(vec3(0.5, 0.1, 0.1), (uv.x + uv.y) * 0.5);
    s.mix(g, dark_mask(s, 0.5) * c.intensity)
}

pub fn gt_animated_gradient(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let a = c.time * 0.2;
    let k = |x: f32| smoothstep(0.0, 1.0, x.sin() * 0.5 + 0.5);
    let (c1, c2, c3) = (
        vec3(0.1, 0.1, 0.5),
        vec3(0.5, 0.1, 0.1),
        vec3(0.1, 0.5, 0.1),
    );
    let start = c1.mix(c2, k(a)).mix(c3, k(a + 2.0));
    let end = c2.mix(c3, k(a + 1.0)).mix(c1, k(a + 3.0));
    let f = smoothstep(0.0, 1.0, (uv.x + uv.y) * 0.5);
    s.mix(start.mix(end, f), dark_mask(s, 0.5) * c.intensity)
}

// ── Generators ──────────────────────────────────────────────────────────

pub fn gt_gears(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv) * 2.2 + vec2(c.time * 0.05, 0.0);
    let id = q.floor();
    let f = q.fract() - 0.5;
    let odd = ((id.x + id.y) as i32).rem_euclid(2);
    let dir = if odd == 0 { 1.0 } else { -1.0 };
    let teeth = 12.0;
    let a = f.y.atan2(f.x);
    let spin = dir * c.time * 0.5 + odd as f32 * PI / teeth;
    let tooth = ((a - spin) * teeth).cos().clamp(-0.4, 0.4) * 0.12;
    let r = f.len();
    let body = r - (0.40 + tooth);
    let hole = r - 0.1;
    let spokes = (((a - spin) * 3.0).cos() * 0.5 + 0.5) > 0.4 && r > 0.1 && r < 0.3;
    let mut col = p.bg;
    let metal = [p.a, p.b, p.c][((id.x * 3.0 + id.y) as i32).rem_euclid(3) as usize];
    if body < 0.0 && hole > 0.0 && !spokes {
        let rim = smoothstep(-0.05, 0.0, body);
        col = metal * (0.3 + 0.25 * (a - spin).cos()) + p.fg * rim * 0.25;
    }
    if hole < 0.0 && hole > -0.03 {
        col = p.fg * 0.6;
    }
    col
}

pub fn gt_inside_matrix(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let mut col = p.bg;
    for layer in 0..8 {
        let z = (layer as f32 + (c.time * 0.7) % 1.0) / 8.0; // 0 near … 1 far
        let depth = 1.0 - z;
        let s = 0.25 + z * 2.4; // farther = bigger query scale (smaller on screen)
        let g = q * s * 4.0;
        let cell = g.floor();
        let h = hash2(
            cell.x as i32 + layer * 19,
            (c.time * 0.7).floor() as i32 - layer,
        );
        if h < 0.5 {
            continue;
        }
        let drop = fract(
            hash1(cell.x + layer as f32 * 3.0) * 9.0 - c.time * 0.8 * (0.6 + hash1(cell.x) * 0.7),
        );
        let yy = fract(g.y * 0.1);
        let trail = (1.0 - ((yy - drop).abs() * 3.0).min(1.0)).powi(2);
        let local = g.fract();
        let gid = (hash2(cell.x as i32, cell.y as i32) * 400.0 + (c.time * 5.0).floor()) as i32;
        let gl = crate::matrix::glyph(gid, local);
        let fade = depth.powf(1.4);
        col = col + (p.a * trail + p.fg * trail * trail * 0.5) * gl * fade * 1.4;
    }
    col * (1.0 - (q.len() * 0.35).min(0.6))
}

pub fn gt_noise_flow(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = vec2(uv.x * c.aspect(), uv.y) * 2.0;
    let w = vec2(
        fbm(q + c.time * 0.12, 4),
        fbm(q + vec2(5.2, 1.3) - c.time * 0.1, 4),
    );
    let n = fbm(q + w * 2.2, 5);
    p.bg.mix(p.b, sat(n * 1.6))
        .mix(p.a, sat((n - 0.45) * 3.0))
        .mix(p.c, sat((w.x - 0.55) * 2.5) * 0.7)
}
