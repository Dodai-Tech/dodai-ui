//! More hand-painted anime looks: clean cel-shaded cumulus, meadows, forests,
//! golden hour, a rainy night stop, sky lanterns, wheat fields, a snowy cabin,
//! cherry blossoms, plus watercolour, god-rays, petals, rain and cinema post
//! effects. Original code and palettes; only the mood is borrowed.
//!
//! GPU-friendly style: index loops, no early returns, no colour arrays.

#![allow(clippy::too_many_arguments, clippy::let_and_return)]

use crate::frame::{Ctx, Frame};
use crate::math::*;
#[allow(unused_imports)]
use crate::prelude::*;
use crate::{Family, ShaderInfo};

const F: Family = Family::Ghibli;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::generator(
        "gh_meadow",
        "Windy meadow",
        F,
        "Rolling hills with wind-swept grass, flowers, a lone tree and cumulus",
        gh_meadow,
    ),
    ShaderInfo::generator(
        "gh_clouds",
        "Cumulus sky",
        F,
        "Clean cel-shaded billowing clouds in three depths with drifting birds",
        gh_clouds,
    ),
    ShaderInfo::generator(
        "gh_forest",
        "Spirit forest",
        F,
        "Mossy trunks, scalloped canopies, god rays, mist and fireflies",
        gh_forest,
    ),
    ShaderInfo::generator(
        "gh_sunset",
        "Golden hour",
        F,
        "A low sun over layered mountains and a glittering lake",
        gh_sunset,
    ),
    ShaderInfo::generator(
        "gh_rain_stop",
        "Rainy night stop",
        F,
        "A lamp and a bus sign in the rain, puddle ripples and wet road glow",
        gh_rain_stop,
    ),
    ShaderInfo::generator(
        "gh_lanterns",
        "Sky lanterns",
        F,
        "Warm paper lanterns drifting up over a dark river",
        gh_lanterns,
    ),
    ShaderInfo::generator(
        "gh_wheat",
        "Wheat sea",
        F,
        "Gusts rolling over a golden field under big clouds",
        gh_wheat,
    ),
    ShaderInfo::generator(
        "gh_snow_cabin",
        "Snowy cabin",
        F,
        "A lit cabin, chimney smoke, pines and soft snowfall at dusk",
        gh_snow_cabin,
    ),
    ShaderInfo::generator(
        "gh_sakura",
        "Cherry blossoms",
        F,
        "A blossoming branch with petals drifting on the breeze",
        gh_sakura,
    ),
    ShaderInfo::post(
        "gh_watercolor",
        "Watercolour wash",
        F,
        "Wobbly edges, colour bleed, pooled pigment and paper fibres",
        gh_watercolor,
    ),
    ShaderInfo::post(
        "gh_godrays",
        "Golden rays",
        F,
        "Sunbeams streaming in from the upper left",
        gh_godrays,
    ),
    ShaderInfo::post(
        "gh_petals",
        "Falling petals",
        F,
        "Blossom petals tumbling over the picture",
        gh_petals,
    ),
    ShaderInfo::post(
        "gh_rain",
        "Rain",
        F,
        "Slanted rain streaks with a cool, damp grade",
        gh_rain,
    ),
    ShaderInfo::post(
        "gh_cinema",
        "Cinema frame",
        F,
        "Widescreen bars, warm highlights, teal shadows, halation and grain",
        gh_cinema,
    ),
];

// ------------------------------------------------------------- helpers --

const SKY_TOP: Vec3 = rgb8(64, 134, 214);
const SKY_MID: Vec3 = rgb8(128, 192, 238);
const SKY_HZ: Vec3 = rgb8(232, 243, 238);
const INK: Vec3 = rgb8(24, 32, 44);

/// One output pixel in centred units (anti-aliasing width).
fn pw(c: &Ctx) -> f32 {
    2.0 / c.h.max(1) as f32
}

fn sky(y: f32) -> Vec3 {
    SKY_HZ
        .mix(SKY_MID, smoothstep(0.0, 0.5, y))
        .mix(SKY_TOP, smoothstep(0.4, 1.0, y))
}

fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = sat(0.5 + 0.5 * (b - a) / k);
    mix(b, a, h) - k * h * (1.0 - h)
}

fn sun_dir() -> Vec2 {
    vec2(-0.55, 0.83)
}

/// Union of puffs on a flat base, about 1.6 wide; `d < 0` inside.
fn billow(p: Vec2, seed: f32) -> f32 {
    let mut d: f32 = 1000.0;
    for i in 0..7 {
        let fi = i as f32;
        let h = hash1(seed * 13.0 + fi * 3.7);
        let k = fi / 6.0 - 0.5;
        let r = 0.13 + 0.3 * (1.0 - (k * 2.0).abs()).powf(1.3) + 0.08 * h;
        let ctr = vec2(k * 1.35 + (h - 0.5) * 0.12, r * 0.78 + 0.05 * h);
        d = smin(d, (p - ctr).len() - r, 0.06);
    }
    d.max(-p.y)
}

/// Paints one cloud over `col`: cream sun-side rim, white body, cool underside.
/// Pixels far from the cloud skip the (costly) shape maths.
fn cloud(col: Vec3, q: Vec2, at: Vec2, s: f32, seed: f32, lit: Vec3, shade: Vec3, w: f32) -> Vec3 {
    let p = (q - at) / s;
    if p.x.abs() > 1.5 || p.y < -0.3 || p.y > 1.3 {
        col
    } else {
        cloud_full(col, q, at, s, seed, lit, shade, w)
    }
}

fn cloud_full(
    col: Vec3,
    q: Vec2,
    at: Vec2,
    s: f32,
    seed: f32,
    lit: Vec3,
    shade: Vec3,
    w: f32,
) -> Vec3 {
    let p = (q - at) / s;
    let d = billow(p, seed) * s;
    let l = sun_dir();
    let toward = billow(p + l * 0.16, seed);
    let away = billow(p - l * 0.2, seed);
    let body = fill_aa(d, w);
    let rim = smoothstep(-0.01, 0.01, toward);
    let sh = smoothstep(-0.01, 0.01, away);
    let under = smoothstep(0.34, 0.02, p.y);
    let mut c = Vec3::WHITE.mix(shade, sat(sh * 0.8 + under * 0.55));
    c = c.mix(lit, rim * (1.0 - sh * 0.4));
    col.mix(c, body)
}

/// Scalloped foliage edge: 1 below the curve (hanging leaf clumps).
fn scallop(x: f32, y: f32, top: f32, bump: f32, freq: f32, seed: f32, w: f32) -> f32 {
    let edge = top
        - bump * (x * freq + seed).sin().abs()
        - 0.5 * bump * (x * freq * 2.3 + seed * 3.0).sin().abs();
    fill_aa(y - edge, w)
}

fn drift(c: &Ctx, base: f32, speed: f32, t: f32, margin: f32) -> f32 {
    let span = c.aspect() * 2.0 + margin * 2.0;
    ((base + t * speed) % 1.0) * span - span * 0.5
}

fn bird(col: Vec3, q: Vec2, at: Vec2, s: f32, flap: f32, w: f32) -> Vec3 {
    let p = (q - at) / s;
    let wing = (p.y + p.x.abs() * (0.7 + flap)).abs() - 0.07;
    let b = fill_aa(wing * s, w) * step(p.x.abs(), 1.4) * step(p.y, 0.45) * step(-0.7, p.y);
    col.mix(INK, b * 0.8)
}

// ----------------------------------------------------------- generators --

pub fn gh_clouds(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let sun = vec2(0.75, 0.55);
    let mut col = sky(q.y * 0.5 + 0.5);
    col = col + rgb8(255, 232, 176) * (0.5 * (-(q - sun).len() * 2.6).exp());
    col = col.mix(rgb8(255, 244, 214), fill_aa((q - sun).len() - 0.11, w));
    for i in 0..9 {
        let fi = i as f32;
        let depth = fi % 3.0; // 0 far .. 2 near
        let h = hash1(fi * 7.3 + 1.0);
        let s = 0.34 + depth * 0.26 + h * 0.12;
        let x = drift(c, h, 0.006 + depth * 0.006, t, 1.4);
        let y = -0.3 + depth * 0.45 + hash1(fi * 3.1) * 0.4;
        let shade = rgb8(176, 198, 232).mix(rgb8(204, 186, 218), depth * 0.3);
        col = cloud(
            col,
            q,
            vec2(x, y),
            s,
            fi + 1.0,
            rgb8(255, 246, 224),
            shade,
            w,
        );
    }
    // a soft sea of cloud along the bottom
    let top = -0.62 + 0.07 * (q.x * 2.2 + t * 0.12).sin() + 0.04 * (q.x * 5.1 - t * 0.2).sin();
    let sea = fill_aa(q.y - top, w);
    let sea_col = Vec3::WHITE.mix(rgb8(180, 200, 232), smoothstep(-0.65, -1.0, q.y));
    col = col.mix(sea_col, sea);
    for i in 0..5 {
        let fi = i as f32;
        let bx = drift(c, fi * 0.21, 0.03 + 0.004 * fi, t, 0.4);
        let by = 0.6 + 0.1 * (fi * 1.7).sin() + 0.02 * (t * 2.0 + fi).sin();
        col = bird(
            col,
            q,
            vec2(bx, by),
            0.05,
            0.25 * (t * 7.0 + fi * 2.0).sin(),
            w,
        );
    }
    col
}

fn hill_y(x: f32, fi: f32) -> f32 {
    let freq = 0.7 + fi * 0.35;
    -0.02 - fi * 0.2 + 0.12 * (x * freq + fi * 2.3).sin() + 0.05 * (x * freq * 2.3 + fi * 5.1).sin()
}

pub fn gh_meadow(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let sun = vec2(0.85, 0.55);
    let mut col = sky(q.y * 0.5 + 0.35);
    col = col + rgb8(255, 226, 160) * (0.4 * (-(q - sun).len() * 2.8).exp());
    for i in 0..4 {
        let fi = i as f32;
        let x = drift(c, hash1(fi * 5.0 + 1.0), 0.008 + 0.004 * fi, t, 1.2);
        let y = 0.32 + 0.2 * hash1(fi * 9.0 + 2.0);
        col = cloud(
            col,
            q,
            vec2(x, y),
            0.42 + 0.18 * hash1(fi * 2.0),
            fi * 1.9 + 1.0,
            rgb8(255, 246, 224),
            rgb8(180, 200, 232),
            w,
        );
    }
    for i in 0..4 {
        let fi = i as f32;
        let ridge = hill_y(q.x, fi);
        let d = q.y - ridge;
        let far = 1.0 - fi / 3.0;
        let hi = rgb8(156, 212, 86).mix(rgb8(168, 210, 184), far);
        let lo = rgb8(50, 140, 74).mix(rgb8(110, 172, 152), far);
        let depth = sat(-d * 2.4);
        let mut hill = hi.mix(lo, smoothstep(0.0, 0.9, depth));
        // painted bands: lit rim, mid, shaded base
        hill = hill.mix(hi * 1.08, smoothstep(0.03, 0.0, -d) * 0.6);
        // wind: light gusts roll across, grass strokes lean with them
        let gust = smoothstep(
            0.55,
            1.0,
            (q.x * 1.3 - t * (0.7 + fi * 0.15) + fi * 2.0).sin(),
        );
        let stroke = smoothstep(
            0.8,
            0.95,
            (q.x * (30.0 + fi * 8.0) + q.y * 3.0 - t * 1.2 * gust).sin(),
        ) * smoothstep(0.3, 0.8, (q.y * (46.0 + fi * 10.0) + q.x * 5.0).sin());
        hill = hill
            + hi * (0.16 * stroke * sat(depth * 3.0) * (1.0 - far * 0.7)
                + 0.1 * gust * (1.0 - far));
        // flowers on the two near hills
        let fx = q.x * (9.0 + fi * 3.0);
        let fyc = d * (13.0 + fi * 4.0);
        let (ix, iy) = (fx.floor(), fyc.floor());
        let h = hash2(ix as i32 + i * 31, iy as i32);
        let off = vec2((hash1(h * 91.0) - 0.5) * 0.5, (hash1(h * 53.0) - 0.5) * 0.5);
        let sway = (t * 1.6 + h * 9.0).sin() * 0.06;
        let fp = vec2(fx - ix - 0.5 - sway, fyc - iy - 0.5) - off;
        let dist = fp.len() - 0.16;
        let on = step(0.8, h) * step(2.0, fi) * step(0.5, depth * 8.0 + 0.4) * step(d, -0.02);
        let fc = if h > 0.95 {
            rgb8(255, 252, 240)
        } else if h > 0.9 {
            rgb8(255, 214, 90)
        } else {
            rgb8(255, 160, 190)
        };
        hill = hill.mix(fc, fill_aa(dist, 0.08) * on);
        col = col.mix(hill, fill_aa(d, w));
        if i == 1 {
            // a lone tree on the second hill
            let tx = -0.95 + 0.04 * (t * 0.3).sin();
            let base = vec2(tx, hill_y(tx, 1.0) - 0.02);
            let trunk = sd_segment(q, base, base + vec2(0.0, 0.26)) - 0.016;
            col = col.mix(rgb8(78, 56, 52), fill_aa(trunk, w));
            let cc = base + vec2(0.0, 0.4);
            let mut cd: f32 = 1000.0;
            let mut lit: f32 = 1000.0;
            for k in 0..5 {
                let fk = k as f32;
                let o = vec2((fk - 2.0) * 0.07, ((fk * 1.7).sin()) * 0.05);
                let r = 0.12 - 0.012 * (fk - 2.0).abs();
                cd = cd.min((q - cc - o).len() - r);
                lit = lit.min((q - cc - o - sun_dir() * 0.05).len() - r);
            }
            let leaf =
                rgb8(44, 124, 84).mix(rgb8(120, 190, 96), smoothstep(0.0, 0.05, cd - lit + 0.02));
            col = col.mix(leaf, fill_aa(cd, w));
        }
    }
    col + rgb8(255, 236, 190) * (0.05 * smoothstep(0.7, 0.0, (q - sun).len()))
}

pub fn gh_forest(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let a = c.aspect();
    let mut col = rgb8(214, 232, 176).mix(rgb8(96, 156, 124), smoothstep(0.7, -0.9, q.y));
    // god rays from the upper left
    let ray = (((q.x + q.y * 0.7) * 6.0 + t * 0.12).sin() * 0.5 + 0.5)
        * (((q.x + q.y * 0.7) * 2.3 - t * 0.05).sin() * 0.5 + 0.5);
    col = col
        + rgb8(255, 240, 170) * (smoothstep(0.35, 0.95, ray) * 0.22 * smoothstep(-0.8, 0.8, q.y));
    let fog = rgb8(190, 220, 184);
    for l in 0..4 {
        let fl = l as f32;
        let depth = fl / 3.0;
        let sc = 1.1 + fl * 0.95;
        let x = q.x * sc + fl * 5.3;
        let cell = x.floor();
        let ci = cell as i32;
        let f = x - cell;
        let h1 = hash2(ci, l * 9 + 1);
        let h2 = hash2(ci, l * 9 + 2);
        let present = step(0.2, h1);
        let cx = 0.25 + 0.5 * h2;
        let sway = (t * 0.4 + h1 * 6.0).sin() * 0.008 * (q.y + 1.0);
        // trunk flares toward the roots
        let flare = 1.0 + 0.7 * smoothstep(-0.2, -1.0, q.y);
        let wd = (0.075 + 0.085 * h2) * flare;
        let dx = f - cx - sway;
        let trunk = fill_aa(dx.abs() - wd, w * sc) * present;
        let bark_lo = rgb8(46, 64, 64).mix(rgb8(30, 52, 52), depth);
        let bark_hi = rgb8(96, 112, 88);
        let grain = smoothstep(0.7, 1.0, (dx * 70.0 + (q.y * 3.0).sin() * 1.5).sin());
        let across = sat((dx / wd) * 0.5 + 0.5);
        let mut tc = bark_hi.mix(bark_lo, smoothstep(0.2, 0.9, across));
        tc = tc.mix(bark_lo * 0.8, grain * 0.35);
        tc = tc.mix(fog, 0.62 - 0.5 * depth);
        // moss on the roots
        let moss = smoothstep(-0.5, -0.9, q.y) * smoothstep(0.0, 0.5, (dx / wd).abs() - 0.2);
        tc = tc.mix(rgb8(104, 168, 80), moss * 0.6 * (1.0 - depth * 0.5));
        // scalloped canopy hanging from the top
        let cy = 0.55 + 0.25 * h2 - fl * 0.04;
        let spread = fill_aa(dx.abs() - (0.45 + 0.2 * h2), w * sc);
        let edge =
            cy - 0.05 * (x * 9.0 + fl).sin().abs() - 0.03 * (x * 21.0 + h1 * 7.0).sin().abs();
        let can = fill_aa(q.y - edge, w) * present * spread;
        let lc = rgb8(40, 120, 86)
            .mix(
                rgb8(150, 204, 100),
                smoothstep(edge - 0.18, edge, q.y) * 0.7 + 0.15 * (x * 4.0).sin(),
            )
            .mix(fog, 0.5 - 0.45 * depth);
        col = col.mix(lc, can);
        col = col.mix(tc, trunk);
        col = col.mix(rgb8(210, 234, 214), 0.1 + 0.04 * (t * 0.2 + fl).sin());
    }
    col = col.mix(rgb8(64, 124, 90), smoothstep(-0.6, -1.0, q.y) * 0.75);
    // fireflies
    for i in 0..16 {
        let fi = i as f32;
        let mx = (hash1(fi * 3.1) * 2.0 - 1.0) * a + (t * 0.3 + fi).sin() * 0.1;
        let my = ((t * (0.03 + 0.03 * hash1(fi)) + hash1(fi * 7.7)) % 1.0) * 2.0 - 1.0;
        let d = (q - vec2(mx, my)).len();
        let tw = 0.6 + 0.4 * (t * 2.0 + fi * 3.0).sin();
        col = col + rgb8(255, 244, 190) * (fill_aa(d - 0.012, w) * tw + (-d * 36.0).exp() * 0.12);
    }
    col
}

pub fn gh_sunset(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let hz = -0.2;
    let sun = vec2(0.25, hz + 0.3);
    let k = sat((q.y - hz) / 1.2);
    let hzc = rgb8(255, 206, 128);
    let mut col = hzc
        .mix(rgb8(238, 128, 112), smoothstep(0.0, 0.45, k))
        .mix(rgb8(72, 66, 138), smoothstep(0.4, 1.0, k));
    let d = (q - sun).len();
    col = col + rgb8(255, 190, 120) * (0.55 * (-d * 2.4).exp() + 0.5 * (-d * 9.0).exp());
    col = col.mix(rgb8(255, 246, 214), fill_aa(d - 0.15, w));
    for i in 0..4 {
        let fi = i as f32;
        let x = drift(c, hash1(fi * 4.4), 0.006 + 0.003 * fi, t, 1.2);
        col = cloud(
            col,
            q,
            vec2(x, 0.15 + 0.17 * fi + 0.05 * hash1(fi)),
            0.4 + 0.1 * fi,
            fi * 2.3 + 0.5,
            rgb8(255, 214, 170),
            rgb8(150, 100, 150),
            w,
        );
    }
    // mountains, far to near, fading into the haze
    for i in 0..3 {
        let fi = i as f32;
        let ridge = hz
            + 0.1
            + 0.16 * fi * 0.0
            + (0.2 - 0.03 * fi) * (q.x * (1.1 + fi * 0.5) + fi * 2.0).sin().abs()
            + 0.05 * (q.x * (4.0 + fi * 2.0) + fi).sin();
        let top = rgb8(176, 112, 144).mix(rgb8(84, 58, 108), fi / 2.0);
        let bot = hzc.mix(top, 0.35 + fi * 0.3);
        let m = top.mix(bot, smoothstep(ridge, hz, q.y));
        col = col.mix(m, fill_aa(q.y - ridge, w) * step(hz, q.y));
    }
    // the lake mirrors the sky and glitters under the sun
    if q.y < hz {
        let r = hz - q.y;
        let mut lake = rgb8(214, 120, 120).mix(rgb8(64, 58, 112), smoothstep(0.0, 0.8, r));
        let band = (r * 150.0 / (0.25 + r) - t * 1.2 + (q.x * 6.0).sin()).sin();
        let streak = smoothstep(0.55, 1.0, band) * (-((q.x - sun.x).abs()) * (3.0 + r * 8.0)).exp();
        lake = lake + rgb8(255, 224, 160) * (streak * 0.9);
        lake = lake.mix(hzc, smoothstep(0.1, 0.0, r) * 0.6);
        col = lake;
    }
    for i in 0..5 {
        let fi = i as f32;
        let bx = drift(c, fi * 0.19, 0.025 + 0.004 * fi, t, 0.4);
        let by = 0.1 + 0.2 * (fi * 1.9).sin().abs() + 0.02 * (t * 2.0 + fi).sin();
        col = bird(col, q, vec2(bx, by), 0.04, 0.3 * (t * 6.0 + fi).sin(), w);
    }
    col
}

fn rain_layer(q: Vec2, t: f32, cols: f32, speed: f32, len: f32, slant: f32, seed: f32) -> f32 {
    let x = (q.x + q.y * slant) * cols;
    let ci = x.floor();
    let h = hash2(ci as i32, seed as i32);
    let fx = x - ci - 0.5;
    // the head falls: fract grows with time, so y shrinks
    let head = 1.3 - 2.8 * fract(h + t * speed);
    let along = (q.y - head) / len;
    let dash = step(0.0, along) * step(along, 1.0) * along;
    smoothstep(0.1, 0.0, fx.abs() - 0.05) * dash * step(0.35, hash1(h * 17.0 + seed))
}

pub fn gh_rain_stop(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let road = -0.42;
    let mut col = rgb8(70, 108, 116).mix(rgb8(22, 40, 58), smoothstep(-0.2, 0.9, q.y));
    // far trees, two scalloped rows
    for i in 0..2 {
        let fi = i as f32;
        let m = scallop(
            q.x,
            q.y,
            -0.02 + 0.18 * fi,
            0.1,
            3.0 + fi * 1.7,
            fi * 4.0,
            w,
        );
        let tc = rgb8(34, 62, 70).mix(rgb8(18, 36, 50), fi);
        col = col.mix(tc, m * step(road, q.y));
    }
    // wet road
    let lamp = vec2(0.55, 0.32);
    let rd = smoothstep(road, road - 0.003, q.y);
    let mut ground = rgb8(40, 62, 74).mix(rgb8(16, 28, 40), smoothstep(road, -1.0, q.y));
    let refl = (-(q.x - lamp.x).abs() * 5.0).exp() * smoothstep(road, -1.0, q.y);
    let rip = 0.6 + 0.4 * (q.y * 90.0 - t * 2.0 + (q.x * 8.0).sin()).sin();
    ground = ground + rgb8(255, 196, 110) * (refl * 0.5 * rip);
    // ripples of falling drops
    for i in 0..7 {
        let fi = i as f32;
        let ph = (t * 0.7 + hash1(fi * 3.3)) % 1.0;
        let cell = (t * 0.7 + hash1(fi * 3.3)).floor();
        let cx = (hash1(fi * 5.1 + cell) * 2.0 - 1.0) * c.aspect();
        let cy = road - 0.12 - 0.5 * hash1(fi * 8.8 + cell);
        let e = vec2((q.x - cx) * 1.0, (q.y - cy) * 3.2).len();
        let ring = smoothstep(0.012, 0.0, (e - ph * 0.22).abs()) * (1.0 - ph);
        ground = ground + rgb8(190, 220, 230) * (ring * 0.5);
    }
    col = col.mix(ground, rd);
    // lamp post and its cone of light
    let cone = smoothstep(0.0, 0.03, lamp.y - q.y)
        * smoothstep(
            (lamp.y - q.y) * 0.55 + 0.02,
            (lamp.y - q.y) * 0.55 - 0.05,
            (q.x - lamp.x).abs(),
        )
        * step(road, q.y);
    col = col + rgb8(255, 206, 130) * (cone * 0.2);
    let pole = sd_segment(q, vec2(lamp.x, road), lamp) - 0.011;
    let arm = sd_segment(q, lamp, lamp + vec2(-0.14, 0.02)) - 0.007;
    col = col.mix(rgb8(14, 24, 34), fill_aa(pole.min(arm), w));
    let head = lamp + vec2(-0.15, 0.0);
    let hd = (q - head).len();
    col = col + rgb8(255, 214, 140) * ((-hd * 7.0).exp() * 0.8);
    col = col.mix(rgb8(255, 246, 210), fill_aa(hd - 0.022, w));
    // bus sign
    let sp = vec2(-0.5, road);
    let post = sd_segment(q, sp, sp + vec2(0.0, 0.55)) - 0.01;
    col = col.mix(rgb8(20, 32, 44), fill_aa(post, w));
    let sign = sd_round_box(q - (sp + vec2(0.0, 0.6)), vec2(0.07, 0.09), 0.02);
    let lit = smoothstep(0.9, 0.0, (q - head).len());
    col = col.mix(rgb8(222, 170, 96) * (0.55 + 0.4 * lit), fill_aa(sign, w));
    col = col.mix(
        rgb8(40, 60, 70),
        fill_aa(
            sd_round_box(q - (sp + vec2(0.0, 0.6)), vec2(0.035, 0.03), 0.012),
            w,
        ),
    );
    // rain, three depths
    let mut r: f32 = 0.0;
    for l in 0..3 {
        let fl = l as f32;
        r += rain_layer(
            q,
            t,
            38.0 + fl * 22.0,
            0.9 + fl * 0.35,
            0.16 + fl * 0.05,
            0.18,
            3.0 + fl,
        ) * (0.25 + fl * 0.12);
    }
    col = col + rgb8(190, 214, 226) * (r * 0.9);
    col.mix(rgb8(52, 84, 96), 0.1)
}

fn lantern_light(q: Vec2, t: f32, c: &Ctx, w: f32) -> Vec3 {
    let mut out = Vec3::BLACK;
    for i in 0..14 {
        let fi = i as f32;
        let h = hash1(fi * 4.7 + 1.0);
        let sp = 0.03 + 0.03 * hash1(fi * 2.9);
        let y = -0.9 + ((h * 3.0 + t * sp) % 1.0) * 2.5;
        let x = (hash1(fi * 8.3) * 2.0 - 1.0) * c.aspect() * 0.95 + (t * 0.5 + fi).sin() * 0.05;
        let s = 0.045 + 0.04 * hash1(fi * 6.1);
        let p = q - vec2(x, y);
        let body = sd_round_box(vec2(p.x, p.y * 0.85), vec2(s * 0.8, s), s * 0.45);
        let flick = 0.85 + 0.15 * (t * 6.0 + fi * 2.0).sin();
        let glow = (-body.max(0.0) * 9.0 / (s * 6.0 + 0.2)).exp() * 0.5;
        let inner = sat(1.0 - (p.len() / s) * 0.7);
        let paper = rgb8(255, 132, 52).mix(rgb8(255, 226, 150), inner);
        let fade = smoothstep(1.55, 1.0, y);
        out = out
            + paper * (fill_aa(body, w) * flick * fade)
            + rgb8(255, 150, 70) * (glow * flick * fade);
    }
    out
}

pub fn gh_lanterns(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let hz = -0.35;
    let mut col = rgb8(70, 54, 120).mix(rgb8(8, 12, 40), smoothstep(hz, 0.9, q.y));
    // stars
    for i in 0..40 {
        let fi = i as f32;
        let sx = (hash1(fi * 2.1) * 2.0 - 1.0) * c.aspect();
        let sy = hz + 0.15 + hash1(fi * 5.7) * (1.0 - hz - 0.15);
        let tw = 0.5 + 0.5 * (t * (1.0 + hash1(fi)) + fi).sin();
        col = col + Vec3::splat(0.8) * (fill_aa((q - vec2(sx, sy)).len() - 0.006, w) * tw);
    }
    // hills
    for i in 0..2 {
        let fi = i as f32;
        let ridge = hz
            + 0.06
            + 0.14 * (1.0 - fi * 0.5) * (q.x * (0.9 + fi * 0.7) + fi * 3.0).sin().abs()
            + 0.03 * (q.x * 5.0 + fi).sin();
        let hc = rgb8(30, 28, 70).mix(rgb8(12, 14, 40), fi);
        col = col.mix(hc, fill_aa(q.y - ridge, w));
    }
    // river with mirrored lanterns
    if q.y < hz {
        let r = hz - q.y;
        let mut riv = rgb8(24, 28, 64).mix(rgb8(8, 10, 30), smoothstep(0.0, 0.7, r));
        let wob = 0.006 * (q.y * 22.0 + t * 1.2).sin() * (1.0 + r * 2.0);
        let m = vec2(q.x + wob, 2.0 * hz - q.y);
        riv = riv + lantern_light(m, t, c, w * 3.0) * (0.55 * smoothstep(0.0, 0.1, r));
        col = riv;
    }
    col + lantern_light(q, t, c, w)
}

pub fn gh_wheat(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let hz = -0.08;
    let mut col = sky(q.y * 0.55 + 0.3);
    for i in 0..5 {
        let fi = i as f32;
        let x = drift(c, hash1(fi * 5.5 + 3.0), 0.007 + 0.003 * (fi % 3.0), t, 1.2);
        col = cloud(
            col,
            q,
            vec2(x, 0.12 + 0.55 * hash1(fi * 2.2) - 0.0 * fi),
            0.4 + 0.25 * hash1(fi * 8.1),
            fi * 1.3 + 2.0,
            rgb8(255, 248, 226),
            rgb8(184, 202, 232),
            w,
        );
    }
    // a far farmhouse and tree line
    let farm = vec2(-0.6, hz + 0.02);
    let wall = sd_box(q - farm - vec2(0.0, 0.03), vec2(0.06, 0.03));
    let roof = (q.y - farm.y - 0.06)
        .max((q.x - farm.x).abs() * 0.8 - (0.075 - (q.y - farm.y - 0.06)) * 0.8 + 0.0);
    col = col.mix(rgb8(236, 226, 206), fill_aa(wall, w));
    col = col.mix(
        rgb8(196, 80, 64),
        fill_aa(
            roof.max(farm.y + 0.06 - q.y - 0.001)
                .max(q.y - farm.y - 0.11),
            w,
        ),
    );
    let trees = scallop(q.x, q.y, hz + 0.04, 0.035, 9.0, 1.0, w);
    col = col.mix(
        rgb8(86, 128, 84),
        trees * step(hz - 0.01, q.y) * step(0.45, 1.0 - fill_aa(wall, w)),
    );
    // the field in perspective
    if q.y < hz {
        let d = hz - q.y;
        let z = 1.0 / (0.12 + d * 1.4);
        let x = q.x * z;
        let wave = (x * 0.9 - t * 1.3 + z * 1.6).sin() * 0.5 + 0.5;
        let gust = smoothstep(0.55, 1.0, wave);
        let base = rgb8(214, 160, 56).mix(rgb8(238, 196, 84), smoothstep(0.0, 0.6, d));
        let mut f = base.mix(rgb8(255, 232, 140), gust * 0.65);
        let blades = smoothstep(
            0.6,
            1.0,
            (x * 40.0 + (z * 3.0).sin() * 2.0 + gust * 2.0 - t * 0.6 * gust).sin(),
        );
        f = f.mix(rgb8(184, 128, 44), blades * 0.1 * sat(d * 3.0));
        let dark = smoothstep(0.4, 0.0, wave) * 0.1;
        col = (f * (1.0 - dark)).mix(rgb8(252, 232, 190), smoothstep(0.1, 0.0, d) * 0.6);
    }
    // tall stalks in the foreground, swaying with the gusts
    for i in 0..9 {
        let fi = i as f32;
        let bx = (hash1(fi * 3.7) * 2.0 - 1.0) * c.aspect();
        let sway = (t * 1.1 + fi * 1.3 + bx * 0.8).sin() * 0.06;
        let base = vec2(bx, -1.05);
        let mid = vec2(bx + sway * 0.4, -0.78);
        let tip = vec2(bx + sway, -0.52 - 0.12 * hash1(fi * 9.1));
        let stem = sd_segment(q, base, mid).min(sd_segment(q, mid, tip)) - 0.006;
        let head = sd_round_box(
            (q - tip - vec2(sway * 0.1, 0.05)).rot(sway * 1.5),
            vec2(0.012, 0.05),
            0.012,
        );
        col = col.mix(rgb8(120, 82, 30), fill_aa(stem, w));
        col = col.mix(rgb8(232, 184, 70), fill_aa(head, w));
    }
    col
}

fn snow_layer(q: Vec2, t: f32, scale: f32, speed: f32, seed: f32, w: f32) -> f32 {
    let p = vec2(
        q.x * scale + (t * 0.3 + q.y).sin() * 0.3,
        q.y * scale + t * speed,
    );
    let ix = p.x.floor();
    let iy = p.y.floor();
    let h = hash2(ix as i32 + seed as i32, iy as i32);
    let off = vec2(0.25 + 0.5 * hash1(h * 33.0), 0.25 + 0.5 * hash1(h * 71.0));
    let d = (vec2(p.x - ix, p.y - iy) - off).len();
    smoothstep(0.09 + w * scale, 0.09 - w * scale, d) * step(0.8, h)
}

pub fn gh_snow_cabin(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let mut col = rgb8(236, 178, 150)
        .mix(rgb8(104, 112, 176), smoothstep(-0.3, 0.5, q.y))
        .mix(rgb8(36, 44, 100), smoothstep(0.4, 1.0, q.y));
    for i in 0..30 {
        let fi = i as f32;
        let sx = (hash1(fi * 2.3) * 2.0 - 1.0) * c.aspect();
        let sy = 0.35 + hash1(fi * 6.9) * 0.65;
        col = col
            + Vec3::splat(0.7)
                * (fill_aa((q - vec2(sx, sy)).len() - 0.005, w)
                    * smoothstep(0.35, 0.7, sy)
                    * (0.5 + 0.5 * (t + fi).sin()));
    }
    // snowy ridges
    for i in 0..3 {
        let fi = i as f32;
        let ridge = -0.15 - fi * 0.22
            + 0.1 * (q.x * (0.8 + fi * 0.4) + fi * 2.0).sin()
            + 0.03 * (q.x * 3.0 + fi).sin();
        let top = rgb8(222, 228, 248).mix(rgb8(172, 186, 226), fi / 2.0);
        let bot = rgb8(150, 164, 214).mix(rgb8(112, 126, 188), fi / 2.0);
        col = col.mix(
            top.mix(bot, smoothstep(ridge, ridge - 0.4, q.y)),
            fill_aa(q.y - ridge, w),
        );
    }
    // pines on the right
    for i in 0..6 {
        let fi = i as f32;
        let px = 0.9 + fi * 0.2 + 0.05 * hash1(fi * 3.3);
        let base = -0.38 - 0.07 * hash1(fi * 8.0) + 0.04 * (fi % 2.0);
        let h = 0.5 + 0.15 * hash1(fi * 2.0);
        let y = (q.y - base) / h;
        let width = (1.0 - y) * 0.3 * (0.7 + 0.3 * (y * 14.0).sin().abs());
        let tree = fill_aa((q.x - px).abs() * 2.2 - width * h * 2.0, w * 2.0)
            * step(0.0, y)
            * step(y, 1.0);
        col = col.mix(rgb8(30, 58, 70).mix(rgb8(20, 36, 56), fi / 5.0), tree);
    }
    // cabin
    let cab = vec2(-0.35, -0.33);
    let walls = sd_box(q - cab - vec2(0.0, 0.14), vec2(0.28, 0.14));
    col = col.mix(rgb8(96, 62, 54), fill_aa(walls, w));
    let logs = smoothstep(0.8, 1.0, (q.y * 70.0).sin()) * 0.2;
    col = col * (1.0 - logs * fill_aa(walls, w));
    let roof = ((q.y - cab.y - 0.28) * 1.0)
        .max(-(q.y - cab.y - 0.28) + (q.x - cab.x).abs() * 0.6 - 0.2)
        .max((q.x - cab.x).abs() - 0.36);
    let roof = roof.max(cab.y + 0.27 - q.y);
    col = col.mix(rgb8(92, 52, 54), fill_aa(roof, w));
    let snowcap = roof.max(-(q.y - cab.y - 0.38 + (q.x - cab.x).abs() * 0.2) * 1.0);
    col = col.mix(rgb8(244, 244, 252), fill_aa(snowcap, w));
    // window with warm light spilling on the snow
    let win = vec2(cab.x + 0.05, cab.y + 0.15);
    let flick = 0.92 + 0.08 * (t * 5.0).sin() * (t * 3.1).sin();
    let spill = (-((q.x - win.x) * 3.0).abs().powf(1.6) - ((q.y - (cab.y - 0.04)) * 7.0).abs())
        .exp()
        * step(q.y, cab.y);
    col = col + rgb8(255, 190, 100) * (spill * 0.4 * flick);
    let wd = sd_round_box(q - win, vec2(0.07, 0.065), 0.012);
    col = col + rgb8(255, 170, 80) * ((-wd.max(0.0) * 10.0).exp() * 0.35 * flick);
    col = col.mix(rgb8(255, 214, 130) * flick, fill_aa(wd, w));
    let mull = sd_box(q - win, vec2(0.004, 0.065)).min(sd_box(q - win, vec2(0.07, 0.004)));
    col = col.mix(rgb8(110, 70, 50), fill_aa(mull, w) * fill_aa(wd, w));
    // chimney smoke
    let ch = vec2(cab.x + 0.18, cab.y + 0.42);
    col = col.mix(
        rgb8(84, 54, 54),
        fill_aa(sd_box(q - ch, vec2(0.025, 0.05)), w),
    );
    for i in 0..7 {
        let fi = i as f32;
        let ph = (t * 0.12 + fi / 7.0) % 1.0;
        let p = ch
            + vec2(
                0.04 * ph * 6.0 * 0.4 + 0.03 * (ph * 8.0 + fi).sin(),
                0.07 + ph * 0.5,
            );
        let r = 0.025 + ph * 0.07;
        col = col.mix(
            rgb8(236, 232, 244),
            fill_aa((q - p).len() - r, w * 2.0) * (1.0 - ph) * 0.55,
        );
    }
    // snowfall, three depths, falling down
    let mut sn: f32 = 0.0;
    for l in 0..3 {
        let fl = l as f32;
        sn += snow_layer(q, t, 14.0 + fl * 9.0, 0.35 + fl * 0.2, fl * 17.0, w) * (0.5 + fl * 0.2);
    }
    col + Vec3::splat(sn)
}

fn blossom(q: Vec2, p: Vec2, r: f32, seed: f32, w: f32) -> Vec3 {
    if (q - p).len() > r * 1.5 {
        Vec3::BLACK
    } else {
        blossom_full(q, p, r, seed, w)
    }
}

fn blossom_full(q: Vec2, p: Vec2, r: f32, seed: f32, w: f32) -> Vec3 {
    // returns (alpha, light, core) packed
    let mut d: f32 = 1000.0;
    let mut dl: f32 = 1000.0;
    for k in 0..5 {
        let fk = k as f32;
        let a = fk * 1.2566 + seed * 3.0;
        let o = vec2(a.cos(), a.sin()) * r * 0.55;
        d = d.min((q - p - o).len() - r * 0.55);
        dl = dl.min((q - p - o - sun_dir() * r * 0.2).len() - r * 0.55);
    }
    let core = fill_aa((q - p).len() - r * 0.15, w);
    vec3(
        fill_aa(d, w),
        smoothstep(0.0, r * 0.3, d - dl + r * 0.08),
        core,
    )
}

pub fn gh_sakura(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let mut col = rgb8(204, 226, 246).mix(rgb8(252, 226, 232), smoothstep(0.6, -0.8, q.y));
    let sun = vec2(0.6, 0.5);
    col = col + rgb8(255, 240, 210) * (0.5 * (-(q - sun).len() * 2.4).exp());
    // soft hills far away
    let hr = -0.45 + 0.1 * (q.x * 1.1).sin() + 0.04 * (q.x * 3.3).sin();
    col = col.mix(rgb8(188, 214, 196), fill_aa(q.y - hr, w));
    let a = c.aspect();
    // main branch sweeping in from the upper left
    let p0 = vec2(-a - 0.1, 0.55);
    let p1 = vec2(-a * 0.3, 0.7 + 0.02 * (t * 0.6).sin());
    let p2 = vec2(a * 0.35, 0.28 + 0.03 * (t * 0.7).sin());
    let p3 = vec2(a * 0.95, 0.4 + 0.03 * (t * 0.5).sin());
    let b1 = sd_segment(q, p0, p1) - 0.035;
    let b2 = sd_segment(q, p1, p2) - 0.028;
    let b3 = sd_segment(q, p2, p3) - 0.018;
    let tw1 = sd_segment(q, p1, p1 + vec2(0.1, -0.35)) - 0.014;
    let br = b1.min(b2).min(b3).min(tw1);
    let bark = rgb8(86, 58, 62).mix(rgb8(138, 96, 92), smoothstep(0.0, -0.03, br) * 0.0 + 0.3);
    col = col.mix(bark, fill_aa(br, w));
    // blossom clumps along the branch
    for i in 0..44 {
        let fi = i as f32;
        let k = hash1(fi * 1.7 + 4.0);
        let seg = (fi % 3.0).floor();
        let (a0, a1) = if seg < 0.5 {
            (p0, p1)
        } else if seg < 1.5 {
            (p1, p2)
        } else {
            (p2, p3)
        };
        let base = a0 + (a1 - a0) * (hash1(fi * 5.3) * 0.95);
        let j = vec2(hash1(fi * 9.1) - 0.5, hash1(fi * 2.9) - 0.5) * 0.45;
        let r = 0.07 + 0.06 * k;
        let p = base + j + vec2((t * 0.7 + fi).sin() * 0.006, 0.0);
        let bl = blossom(q, p, r, fi, w);
        let petal = rgb8(246, 168, 192).mix(rgb8(255, 226, 232), bl.y);
        col = col.mix(petal, bl.x);
        col = col.mix(rgb8(236, 120, 150), bl.z * 0.7);
    }
    // petals on the breeze
    for i in 0..36 {
        let fi = i as f32;
        let h = hash1(fi * 3.9);
        let sp = 0.05 + 0.05 * hash1(fi * 6.1);
        let y = 1.2 - ((h * 5.0 + t * sp) % 1.0) * 2.5;
        let x = (hash1(fi * 8.2) * 2.0 - 1.0) * a
            + (t * 0.8 + fi * 2.0).sin() * 0.12
            + (1.2 - y) * 0.12;
        let rot = t * (1.0 + h) + fi;
        let pe = (q - vec2(x, y)).rot(rot);
        let d = vec2(pe.x / 1.6, pe.y).len() - 0.018 * (0.6 + h * 0.8);
        col = col.mix(rgb8(255, 196, 212), fill_aa(d, w) * 0.9);
    }
    col
}

// ----------------------------------------------------------------- post --

pub fn gh_watercolor(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let px = c.texel();
    let s0 = src.sample(uv);
    // wobbly edges: displace the lookup with low-frequency noise
    let n = vec2(
        noise(vec2(uv.x * 38.0, uv.y * 24.0)) - 0.5,
        noise(vec2(uv.x * 24.0 + 9.0, uv.y * 38.0)) - 0.5,
    );
    let u = uv + vec2(n.x * px.x * 5.0, n.y * px.y * 5.0);
    let mut blur = Vec3::BLACK;
    for i in 0..8 {
        let a = i as f32 * 0.8;
        blur = blur + src.sample(u + vec2(a.cos() * px.x * 2.2, a.sin() * px.y * 2.2));
    }
    let wash = (blur + src.sample(u) * 2.0) / 10.0;
    // pigment pools where the colour changes
    let edge = (wash.luma() - s0.luma()).abs() * 8.0 + (wash - s0).len() * 3.0;
    let pool = sat(edge);
    let soft = Vec3::splat(wash.luma()).mix(wash, 1.1);
    let mut col = soft * (1.0 - 0.28 * pool);
    // flat washes: gentle posterise
    let qz = |v: f32| (v * 9.0 + 0.5).floor() / 9.0;
    let q = vec3(qz(col.x), qz(col.y), qz(col.z));
    col = col.mix(q, 0.35);
    let paper = 0.95
        + 0.05 * (noise(vec2(uv.x * c.w as f32 * 0.7, uv.y * c.h as f32 * 0.7)) - 0.5)
        + 0.03 * (noise(vec2(uv.x * 60.0, uv.y * 4.0)) - 0.5);
    let out = col.mix(Vec3::WHITE, 0.06) * paper;
    s0.mix(out, c.intensity)
}

pub fn gh_godrays(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let sun = vec2(0.18, -0.08);
    let dir = (sun - uv) / 18.0;
    let mut acc = Vec3::BLACK;
    let mut p = uv;
    let mut wgt: f32 = 1.0;
    for _ in 0..18 {
        p = p + dir;
        let v = src.sample(p);
        acc = acc + v * ((v.luma() - 0.55).max(0.0) * wgt);
        wgt *= 0.93;
    }
    let beams = 0.7 + 0.3 * ((uv.x - sun.x).atan2(uv.y - sun.y) * 14.0 + c.time * 0.4).sin();
    let out = s + acc * rgb8(255, 232, 170) * (0.09 * beams);
    s.mix(out, c.intensity)
}

fn petal_layer(q: Vec2, t: f32, a: f32, count: i32, size: f32, seed: f32, w: f32) -> f32 {
    let mut m: f32 = 0.0;
    for i in 0..count {
        let fi = i as f32 + seed;
        let h = hash1(fi * 3.9);
        let y = 1.2 - ((h * 5.0 + t * (0.07 + 0.05 * hash1(fi * 6.1))) % 1.0) * 2.5;
        let x = (hash1(fi * 8.2) * 2.0 - 1.0) * a
            + (t * 0.8 + fi * 2.0).sin() * 0.14
            + (1.2 - y) * 0.14;
        let pe = (q - vec2(x, y)).rot(t * (1.0 + h) + fi);
        let d = vec2(pe.x / 1.7, pe.y).len() - size * (0.6 + h * 0.8);
        m = m.max(fill_aa(d, w));
    }
    m
}

pub fn gh_petals(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let q = c.centered(uv);
    let w = pw(c);
    let far = petal_layer(q, c.time, c.aspect(), 26, 0.012, 0.0, w * 2.0);
    let near = petal_layer(q, c.time * 1.3, c.aspect(), 12, 0.026, 40.0, w * 2.0);
    let out = s
        .mix(rgb8(255, 206, 220), far * 0.7)
        .mix(rgb8(255, 226, 234), near * 0.9);
    s.mix(out, c.intensity)
}

pub fn gh_rain(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let q = c.centered(uv);
    let mut r: f32 = 0.0;
    for l in 0..3 {
        let fl = l as f32;
        r += rain_layer(
            q,
            c.time,
            40.0 + fl * 24.0,
            0.9 + fl * 0.35,
            0.16 + fl * 0.05,
            0.2,
            7.0 + fl,
        ) * (0.28 + fl * 0.12);
    }
    let damp = Vec3::splat(s.luma()).mix(s, 0.8) * rgb8(226, 238, 246) * 0.94;
    let out = damp + rgb8(206, 224, 236) * (r * 0.8);
    s.mix(out, c.intensity)
}

pub fn gh_cinema(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let px = c.texel();
    let s = src.sample(uv);
    // halation: warm bloom from the highlights
    let mut hal = Vec3::BLACK;
    for i in 0..10 {
        let a = i as f32 * 0.6283;
        let v = src.sample(uv + vec2(a.cos() * px.x * 9.0, a.sin() * px.y * 9.0));
        hal = hal + v * (v.luma() - 0.72).max(0.0);
    }
    let l = s.luma();
    let graded = s * rgb8(255, 246, 232)
        + rgb8(0, 14, 22) * (1.0 - l) * 0.35
        + hal * rgb8(255, 170, 110) * 0.09;
    let q = c.centered(uv);
    let vig = 1.0 - 0.28 * smoothstep(0.5, 1.7, q.len());
    let grain = (hash2(
        (uv.x * c.w as f32) as i32,
        (uv.y * c.h as f32) as i32 + (c.time * 24.0) as i32,
    ) - 0.5)
        * 0.05;
    let mut out = graded * vig + Vec3::splat(grain);
    // widescreen bars
    let half = (c.aspect() / 2.39).min(1.0);
    out = out * step(q.y.abs(), half);
    s.mix(out, c.intensity)
}
