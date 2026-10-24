//! Material 3 shaders: dynamic-colour blobs, state-layer ripples, tonal
//! waves, tonal elevation and shape morphing.

use crate::frame::Ctx;
use crate::math::*;
use crate::prelude::*;
use crate::{Family, ShaderInfo};

const F: Family = Family::Material3;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::generator(
        "m3_blobs",
        "Dynamic colour",
        F,
        "Soft tonal blobs drifting over a surface, like Material You wallpapers",
        m3_blobs,
    ),
    ShaderInfo::generator(
        "m3_ripple",
        "Touch ripple",
        F,
        "Ink ripples with a state-layer wash",
        m3_ripple,
    ),
    ShaderInfo::generator(
        "m3_waves",
        "Tonal waves",
        F,
        "Layered sine bands in primary tones",
        m3_waves,
    ),
    ShaderInfo::generator(
        "m3_elevation",
        "Tonal elevation",
        F,
        "Five elevation levels: shadow plus tonal overlay",
        m3_elevation,
    ),
    ShaderInfo::generator(
        "m3_morph",
        "Shape morph",
        F,
        "Expressive shape morphing: circle, squircle, flower",
        m3_morph,
    ),
    ShaderInfo::generator(
        "m3_stripes",
        "Loading indicator",
        F,
        "Wavy determinate progress with a stop indicator",
        m3_progress,
    ),
];

/// Tonal surface: `bg` lifted toward `tint` by elevation `e` (0..5), as M3
/// surface-tint overlays do.
pub fn tonal(bg: Vec3, tint: Vec3, e: f32) -> Vec3 {
    const OVER: [f32; 6] = [0.0, 0.05, 0.08, 0.11, 0.12, 0.14];
    let i = e.clamp(0.0, 4.999);
    let k = i.floor() as usize;
    let a = mix(OVER[k], OVER[k + 1], i - i.floor());
    bg.mix(tint, a)
}

pub fn m3_blobs(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let mut col = tonal(p.bg, p.a, 1.0);
    let tones = [p.a, p.c, p.b, p.a.mix(p.c, 0.5)];
    for i in 0..4 {
        let tone = &tones[i];
        let f = i as f32;
        let ctr = vec2(
            (c.time * 0.21 + f * 1.7).sin() * 0.9 * c.aspect() * 0.7,
            (c.time * 0.17 + f * 2.3).cos() * 0.6,
        );
        let r = 0.55 + 0.12 * (c.time * 0.3 + f).sin();
        let w = (-((q - ctr).len() / r).powi(2) * 1.6).exp();
        col = col.mix(*tone * 0.75 + p.bg * 0.25, w * 0.55);
    }
    // faint noise grain keeps gradients from banding
    col + Vec3::splat((hash2((uv.x * c.w as f32) as i32, (uv.y * c.h as f32) as i32) - 0.5) * 0.012)
}

pub fn m3_ripple(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let mut col = tonal(p.bg, p.a, 1.0);
    let period = 2.4;
    for k in 0..3 {
        let t0 = c.time - k as f32 * 0.8;
        let n = (t0 / period).floor();
        let age = (t0 - n * period) / period; // 0..1
        let seed = n * 3.0 + k as f32;
        let ctr = vec2(
            (hash1(seed) - 0.5) * 1.4 * c.aspect(),
            (hash1(seed + 1.0) - 0.5) * 1.2,
        );
        let r = age.sqrt() * 1.5;
        let d = (q - ctr).len();
        let inside = smoothstep(r, r - 0.02, d);
        let fade = (1.0 - age).powi(2);
        col = col.mix(p.a, inside * 0.28 * fade);
        col = col + p.a * glow((d - r) * 60.0, 1.0) * 0.18 * fade;
    }
    col
}

pub fn m3_waves(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let mut col = tonal(p.bg, p.a, 0.5);
    for i in 0..5 {
        let f = i as f32;
        let base = 0.35 + f * 0.13;
        let y = base
            + 0.05 * (uv.x * (3.0 + f) * 2.0 + c.time * (0.35 + f * 0.1) + f * 1.9).sin()
            + 0.025 * (uv.x * 9.0 - c.time * 0.5 + f).sin();
        if uv.y > y {
            let tone = p.a.mix(p.bg, 0.78 - f * 0.12).mix(p.c, 0.15 * f / 4.0);
            col = tone;
        }
    }
    col
}

pub fn m3_elevation(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let mut col = tonal(p.bg, p.a, 0.0);
    let n = 5;
    let aspect = c.aspect();
    let w = (aspect * 2.0 - 0.3) / n as f32;
    for i in 0..n {
        let e = i as f32;
        let cx = -aspect + 0.15 + w * (i as f32 + 0.5);
        let lift = 0.04 * (c.time * 1.2 + e).sin();
        let ctr = vec2(cx, lift * (e / 4.0));
        let half = vec2(w * 0.42, 0.32);
        let d = sd_round_box(q - ctr, half, 0.12);
        // key + ambient shadow grows with elevation
        let sd = sd_round_box(q - ctr - vec2(0.0, -0.02 - 0.012 * e), half, 0.12);
        let blur = 0.02 + 0.03 * e;
        let shadow = smoothstep(blur, -blur * 0.3, sd) * (0.12 + 0.05 * e);
        col = col * (1.0 - shadow);
        let surf = tonal(p.bg, p.a, e + 1.0) + p.fg * 0.05;
        col = col.mix(surf, fill_aa(d, 2.0 / c.h as f32));
    }
    col
}

/// Rounded-polygon radius function: morphs `circle → squircle → flower`.
fn shape_r(a: f32, k: f32) -> f32 {
    let circle = 1.0;
    let squircle = 1.0 / (a.cos().abs().powi(4) + a.sin().abs().powi(4)).powf(0.25);
    let flower = 0.82 + 0.18 * (a * 6.0).cos();
    if k < 1.0 {
        mix(circle, squircle * 0.92, k)
    } else {
        mix(squircle * 0.92, flower, k - 1.0)
    }
}

pub fn m3_morph(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let ph = (c.time * 0.45) % 3.0;
    let k = if ph < 1.0 {
        smoothstep(0.0, 1.0, ph)
    } else if ph < 2.0 {
        1.0 + smoothstep(1.0, 2.0, ph) * 0.999
    } else {
        2.0 - smoothstep(2.0, 3.0, ph) * 2.0
    };
    let ang = q.y.atan2(q.x) - c.time * 0.3;
    let rot = q.rot(c.time * 0.3);
    let _ = rot;
    let r = q.len();
    let edge = 0.55 * shape_r(ang, k);
    let inside = fill_aa(r - edge, 2.0 / c.h as f32);
    let mut col = tonal(p.bg, p.a, 1.0);
    // container / on-container tones
    col = col.mix(p.a * 0.85, inside);
    col = col.mix(p.bg, fill_aa(r - edge * 0.45, 2.0 / c.h as f32) * 0.6);
    col + p.fg * glow(r - edge, 25.0) * 0.04
}

pub fn m3_progress(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let mut col = tonal(p.bg, p.a, 0.5);
    let a = c.aspect();
    let q = vec2((uv.x - 0.5) * 2.0 * a, (0.5 - uv.y) * 2.0);
    let rows = 3;
    for i in 0..rows {
        let y = 0.45 - i as f32 * 0.45;
        let progress = ((c.time * 0.25 + i as f32 * 0.31) % 1.0) * 1.1;
        let half = a * 0.78;
        let x0 = -half;
        let x1 = x0 + 2.0 * half * progress.min(1.0);
        let wave = 0.045 * (q.x * (14.0 + 4.0 * i as f32) - c.time * 6.0).sin();
        // active track (wavy), inactive track (flat), stop dot
        let in_active = step(x0, q.x) * step(q.x, x1);
        let da = (q.y - y - wave * in_active).abs() - 0.03;
        let active = fill_aa(da, 2.0 / c.h as f32) * in_active;
        let inactive = fill_aa((q.y - y).abs() - 0.018, 2.0 / c.h as f32)
            * step(x1 + 0.05, q.x)
            * step(q.x, half);
        col = col.mix(p.a, active).mix(p.b * 0.45, inactive * 0.9);
        let stop = fill_aa(sd_circle(q - vec2(half, y), 0.03), 2.0 / c.h as f32);
        col = col.mix(p.a, stop);
    }
    col
}
