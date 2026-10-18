//! General post effects usable on any generator.

use crate::frame::{Ctx, Frame};
use crate::math::*;
#[allow(unused_imports)]
use crate::prelude::*;
use crate::{Family, ShaderInfo};

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::post(
        "bloom",
        "Neon bloom",
        Family::Cyberpunk,
        "Wide additive glow from bright areas",
        bloom,
    ),
    ShaderInfo::post(
        "glitch",
        "Glitch",
        Family::Cyberpunk,
        "Block displacement, RGB split and flicker",
        glitch,
    ),
    ShaderInfo::post(
        "chromatic",
        "Chromatic aberration",
        Family::Cyberpunk,
        "Radial colour fringing",
        chromatic,
    ),
    ShaderInfo::post(
        "neon_edges",
        "Neon edges",
        Family::Cyberpunk,
        "Sobel outline glowing in the accent colour",
        neon_edges,
    ),
    ShaderInfo::post(
        "dither",
        "1-bit dither",
        Family::Retro,
        "Ordered dither between background and foreground",
        dither,
    ),
    ShaderInfo::post(
        "negative",
        "Negative",
        Family::Retro,
        "Inverts colours",
        negative,
    ),
    ShaderInfo::post(
        "m3_scrim",
        "Tonal scrim",
        Family::Material3,
        "Surface-tint wash and soft vignette like an M3 modal scrim",
        m3_scrim,
    ),
    ShaderInfo::post(
        "water_warp",
        "Water warp",
        Family::Nature,
        "Rippling refraction",
        water_warp,
    ),
    ShaderInfo::post(
        "heat_haze",
        "Heat haze",
        Family::Nature,
        "Rising shimmer",
        heat_haze,
    ),
    ShaderInfo::post(
        "kaleidoscope",
        "Kaleidoscope",
        Family::Nature,
        "Six-fold mirror",
        kaleidoscope,
    ),
];

pub fn bloom(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    // 12 nearest taps on two rings; the loss against a bilinear 24-tap
    // version is invisible once glowing areas overlap.
    const TAPS: [(f32, f32); 12] = [
        (3.0, 0.0),
        (0.0, 3.0),
        (-3.0, 0.0),
        (0.0, -3.0),
        (6.4, 6.4),
        (-6.4, 6.4),
        (-6.4, -6.4),
        (6.4, -6.4),
        (9.0, 0.0),
        (0.0, 9.0),
        (-9.0, 0.0),
        (0.0, -9.0),
    ];
    let s = src.sample(uv);
    let mut acc = Vec3::BLACK;
    for i in 0..TAPS.len() {
        let (dx, dy) = TAPS[i];
        let t = src.nearest(uv + vec2(dx / src.w as f32, dy / src.h as f32));
        acc = acc + t * (t.luma() - 0.35).max(0.0);
    }
    s + acc / 12.0 * 6.0 * c.intensity
}

pub fn glitch(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let t = (c.time * 8.0).floor() as i32;
    let band = (uv.y * 18.0).floor() as i32;
    let on = hash2(band, t) > 1.0 - 0.25 * c.intensity;
    let shift = if on {
        (hash2(band + 3, t) - 0.5) * 0.15 * c.intensity
    } else {
        0.0
    };
    let split = if on {
        0.01 * c.intensity
    } else {
        0.0015 * c.intensity
    };
    let u = vec2(uv.x + shift, uv.y);
    let col = vec3(
        src.sample(u + vec2(split, 0.0)).x,
        src.sample(u).y,
        src.sample(u - vec2(split, 0.0)).z,
    );
    let flick = 1.0 - 0.06 * c.intensity * hash1(t as f32);
    col * flick
}

pub fn chromatic(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let d = (uv - 0.5) * 0.012 * c.intensity;
    vec3(src.sample(uv + d).x, src.sample(uv).y, src.sample(uv - d).z)
}

pub fn neon_edges(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    // Sobel on the 8 integer neighbours: one luma read each, shared by gx/gy.
    let (x, y) = ((uv.x * src.w as f32) as i32, (uv.y * src.h as f32) as i32);
    let l = |dx: i32, dy: i32| src.get(x + dx, y + dy).luma();
    let (tl, t, tr) = (l(-1, -1), l(0, -1), l(1, -1));
    let (ml, mr) = (l(-1, 0), l(1, 0));
    let (bl, b, br) = (l(-1, 1), l(0, 1), l(1, 1));
    let gx = -tl - 2.0 * ml - bl + tr + 2.0 * mr + br;
    let gy = -tl - 2.0 * t - tr + bl + 2.0 * b + br;
    let e = (gx * gx + gy * gy).sqrt().min(1.0);
    let base = src.get(x, y) * 0.25;
    base + c.pal.b.mix(c.pal.a, e) * e * 2.2 * c.intensity
}

pub fn dither(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let (x, y) = ((uv.x * src.w as f32) as i32, (uv.y * src.h as f32) as i32);
    let l = src.sample(uv).luma().powf(0.8);
    if l > bayer4(x, y) { c.pal.fg } else { c.pal.bg }
}

pub fn negative(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    s.mix(Vec3::WHITE - s.clamp01(), c.intensity)
}

pub fn m3_scrim(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let d = (uv - 0.5).len();
    let tint = c.pal.bg.mix(c.pal.a, 0.14);
    s.mix(tint, 0.45 * c.intensity) * (1.0 - 0.35 * d * d * 2.0 * c.intensity)
}

pub fn water_warp(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let k = 0.012 * c.intensity;
    let o = vec2(
        (uv.y * 22.0 + c.time * 2.0).sin(),
        (uv.x * 18.0 + c.time * 1.6).cos(),
    ) * k;
    let s = src.sample(uv + o);
    s * vec3(0.9, 1.0, 1.1)
}

pub fn heat_haze(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let n = noise(vec2(uv.x * 30.0, uv.y * 14.0 - c.time * 3.0)) - 0.5;
    let k = 0.015 * c.intensity * (1.0 - uv.y * 0.5);
    src.sample(uv + vec2(n * k, n * k * 0.5))
}

pub fn kaleidoscope(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let q = (uv - 0.5) * vec2(c.aspect(), 1.0);
    let r = q.len();
    let seg = std::f32::consts::TAU / 6.0;
    let a = fmod(q.y.atan2(q.x) + c.time * 0.1, seg);
    let a = (a - seg / 2.0).abs();
    let m = vec2(a.cos(), a.sin()) * r;
    let sampled = src.sample(vec2(m.x / c.aspect() + 0.5, m.y + 0.5));
    src.sample(uv).mix(sampled, c.intensity)
}
