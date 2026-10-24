//! Rainy neon megacities and Blade Runner-inspired scenes: layered skylines
//! with haze, flying cars, searchlights, wet streets, plus rain, grading and
//! anamorphic flares as post effects. Original code; the look is "inspired by"
//! the 1982 film's mood, nothing is copied from it.
//!
//! GPU-friendly style: index loops, no early returns, no arrays of colours.

use crate::frame::{Ctx, Frame};
use crate::math::*;
use crate::prelude::*;
use crate::{Family, ShaderInfo};
use std::f32::consts::TAU;

const F: Family = Family::Cyberpunk;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::generator(
        "neon_city",
        "Rain city",
        F,
        "Five-layer skyline in haze, neon signs, a flying car, wet street with ripples",
        neon_city,
    ),
    ShaderInfo::generator(
        "br_city",
        "Replicant city",
        F,
        "Orange smog, searchlights, billboards and a spinner over a rainy megacity",
        br_city,
    ),
    ShaderInfo::generator(
        "br_street",
        "Rainy alley",
        F,
        "Street-level canyon of neon signs, wet road, umbrellas and steam",
        br_street,
    ),
    ShaderInfo::generator(
        "br_tyrell",
        "Corporate pyramid",
        F,
        "Stepped ziggurat glowing through smog with searchlights",
        br_tyrell,
    ),
    ShaderInfo::generator(
        "br_off_world",
        "Refinery fires",
        F,
        "Smoggy orange sky over smokestacks venting flame",
        br_off_world,
    ),
    ShaderInfo::generator(
        "br_eye",
        "Empathy test",
        F,
        "A scanning iris with a breathing pupil and data read-outs",
        br_eye,
    ),
    ShaderInfo::post(
        "br_rain",
        "Heavy rain",
        F,
        "Three layers of slanted rain that refract the picture",
        br_rain,
    ),
    ShaderInfo::post(
        "br_grade",
        "Teal & orange",
        F,
        "Teal shadows, orange highlights, halation around lights",
        br_grade,
    ),
    ShaderInfo::post(
        "br_anamorphic",
        "Anamorphic flares",
        F,
        "Horizontal blue streaks from bright lights",
        br_anamorphic,
    ),
    ShaderInfo::post(
        "br_wet",
        "Wet pavement",
        F,
        "Rippled reflection of the picture in a rain-soaked street",
        br_wet,
    ),
    ShaderInfo::post(
        "br_smog",
        "Smog & shafts",
        F,
        "Drifting haze that glows around the bright parts",
        br_smog,
    ),
    ShaderInfo::post(
        "br_searchlights",
        "Searchlights",
        F,
        "Sweeping beams from below the horizon",
        br_searchlights,
    ),
];

// ---------------------------------------------------------------- shared --

/// Colours and switches of one city look.
#[derive(Clone, Copy)]
struct Look {
    fog: Vec3,
    sky: Vec3,
    warm: Vec3,
    cool: Vec3,
    n1: Vec3,
    n2: Vec3,
    street: f32,
    spinner: f32,
    beams: f32,
    billboards: f32,
}

fn neon_look(p: &crate::ctx::ShaderPalette) -> Look {
    Look {
        fog: p.c * 0.50 + p.b * 0.16,
        sky: p.bg * 1.2 + p.b * 0.015,
        warm: p.a,
        cool: p.b,
        n1: p.c,
        n2: p.b,
        street: 0.78,
        spinner: 1.0,
        beams: 0.0,
        billboards: 0.6,
    }
}

fn br_look(p: &crate::ctx::ShaderPalette) -> Look {
    Look {
        fog: p.a * 0.62 + p.c * 0.08,
        sky: p.bg * 1.1 + p.b * 0.05,
        warm: p.a,
        cool: p.b,
        n1: p.c,
        n2: p.b,
        street: 0.78,
        spinner: 1.0,
        beams: 1.0,
        billboards: 1.0,
    }
}

/// Vertical angle-based beam from `(ox, oy)` pointing up-ish at angle `ang`.
fn beam(x: f32, y: f32, ox: f32, oy: f32, ang: f32) -> f32 {
    let dx = x - ox;
    let dy = oy - y; // up is positive
    let up = if dy > 0.0 { 1.0 } else { 0.0 };
    let a = dx.atan2(dy.max(0.001));
    let d = a - ang;
    let r = (dx * dx + dy * dy).sqrt();
    let w = 0.04 + r * 0.05;
    up * (-(d * d) / (w * w)).exp() * (1.0 - smoothstep(0.3, 1.6, r)) * 0.9
}

fn sky(x: f32, y: f32, t: f32, k: &Look) -> Vec3 {
    let h = smoothstep(0.0, k.street, y);
    let mut col = k.sky.mix(k.fog, h * h * 0.95);
    // smog banks lit from below
    let cl = fbm(vec2(x * 1.3 + t * 0.012, y * 3.2 - t * 0.004), 4);
    let lit = smoothstep(0.1, k.street, y);
    col = col + k.fog * (cl - 0.32).max(0.0) * 0.9 * (0.25 + lit);
    // searchlights
    if k.beams > 0.0 {
        let s = (t * 0.35).sin();
        let b = beam(x, k.street, 0.45, k.street, -0.45 + 0.5 * s)
            + beam(x, k.street, 1.3, k.street, 0.35 - 0.5 * s)
            + beam(x, k.street, 0.9, k.street, 0.05 + 0.35 * (t * 0.23).sin());
        col = col + (k.warm * 0.5 + k.fog) * b * k.beams * (0.5 + cl);
    }
    col
}

/// One layer of towers (`l` = 0 far .. `n-1` near). Returns the new colour and
/// adds light that bleeds into the air around signs and screens.
fn tower_layer(x: f32, y: f32, t: f32, l: i32, n: i32, k: &Look, below: Vec3) -> Vec3 {
    let d = l as f32 / (n - 1).max(1) as f32;
    let scale = 2.3 + l as f32 * 1.25;
    let wx = x * scale + l as f32 * 13.7 + t * 0.004 * l as f32;
    let cell = wx.floor();
    let ci = cell as i32;
    let fx = wx - cell;
    let h1 = hash2(ci, l * 17 + 1);
    let h2 = hash2(ci, l * 17 + 2);
    let h3 = hash2(ci, l * 17 + 3);
    let h4 = hash2(ci, l * 17 + 4);
    let m = 0.03 + 0.2 * h2;
    let ground = k.street + 0.01;
    let height = (0.14 + 0.42 * h1 * h1 + 0.1 * h4) * (1.25 - 0.75 * d);
    let tier = if fx < 0.5 + (h3 - 0.5) * 0.5 {
        0.0
    } else {
        0.05 * h4
    };
    let top = ground - height + tier;
    let exists = hash2(ci + 77, l) > 0.35 * d * d + 0.05 * d;
    let inside = exists && y > top && y < ground && fx > m && fx < 1.0 - m;
    let fogamt = 0.88 - 0.8 * d;
    let mut col = below;

    if inside {
        let shade = 1.0 - 0.45 * smoothstep(top, ground, y);
        let mut body = k.sky * (0.7 + 0.6 * d) * shade + k.fog * 0.05 * d;
        // rim light on the lit side of the tower
        body = body + k.fog * smoothstep(m + 0.1, m, fx) * 0.35 * (1.0 - d * 0.4);
        let haze = k.fog * (0.55 + 0.7 * smoothstep(top, ground, y));
        body = body.mix(haze, fogamt);
        // windows
        let wcols = 3.0 + (h4 * 5.0).floor();
        let sp = 0.016 + 0.012 * d;
        let gx = (fx - m) / (1.0 - 2.0 * m) * wcols;
        let gy = (y - top) / sp;
        let (ix, iy) = (gx.floor() as i32, gy.floor() as i32);
        let (fxw, fyw) = (gx - gx.floor(), gy - gy.floor());
        let lit = hash3(ci + l * 101, ix, iy);
        let occ = 0.25 + 0.55 * h1;
        let slow = (t * 0.4 + lit * 60.0).sin();
        let on = step(lit, occ) * smoothstep(-0.97, -0.8, slow);
        let wm = smoothstep(0.1, 0.22, fxw)
            * smoothstep(0.9, 0.78, fxw)
            * smoothstep(0.18, 0.3, fyw)
            * smoothstep(0.88, 0.76, fyw);
        let pick = hash3(ci, ix + 7, iy + 3);
        let wc = if pick > 0.9 {
            k.n1
        } else if pick > 0.72 {
            k.cool
        } else {
            k.warm
        };
        let wglow = smoothstep(-0.25, 0.3, fxw)
            * smoothstep(1.25, 0.7, fxw)
            * smoothstep(-0.2, 0.3, fyw)
            * smoothstep(1.2, 0.7, fyw);
        let wcol = wc.mix(k.fog * 2.0, fogamt * 0.6);
        body = body + wcol * (wm + 0.28 * wglow) * on * (0.55 + 0.6 * lit) * (1.2 - 0.85 * fogamt);
        col = body;
    }

    // antenna + blinking aircraft light
    let ax = (fx - 0.5) / scale;
    let alen = 0.03 + 0.06 * h4;
    if h3 > 0.55 && y < top && y > top - alen && ax.abs() < 0.0013 {
        col = k.sky * 0.8 + k.fog * 0.3;
    }
    let blink = smoothstep(0.55, 0.75, fract(t * 0.2 + h2));
    let ad = vec2(ax, y - (top - alen)).len();
    col =
        col + vec3(1.0, 0.15, 0.1) * (0.0006 / (ad * ad + 0.00002)) * 0.02 * blink * step(0.55, h3);

    // neon sign strip
    let has_sign = step(0.7, h2) * step(0.25, d);
    let sx = m + 0.1;
    let sw = 0.07;
    let sy0 = top + 0.035;
    let slen = 0.1 + 0.2 * h1;
    let in_sign = has_sign * step(sx, fx) * step(fx, sx + sw) * step(sy0, y) * step(y, sy0 + slen);
    let seg = (y - sy0) * (60.0 - 20.0 * d);
    let glyph = step(0.2, fract(seg)) * step(0.35, hash2(ci + 5, seg.floor() as i32));
    let broken = if h4 > 0.8 {
        0.4 + 0.6 * smoothstep(0.2, 0.5, fract(t * 0.25 + h1 * 9.0))
    } else {
        1.0
    };
    let sc = if h1 > 0.5 { k.n1 } else { k.n2 };
    let sign_on = in_sign * broken;
    col = col.mix(sc * 1.7, sign_on * glyph);
    // halo of the sign spills into the air
    let hx = ((fx - (sx + sw * 0.5)) / scale).abs();
    let hy = ((y - (sy0 + slen * 0.5)).abs() - slen * 0.5).max(0.0);
    let halo = (-(hx * hx + hy * hy) / 0.0009).exp();
    col = col + sc * has_sign * broken * halo * 0.42 * (1.0 - 0.5 * d);

    // giant billboard on a few near towers
    let has_bb = step(0.9, h3) * step(0.45, d) * k.billboards;
    let bx0 = m + 0.12;
    let bx1 = 1.0 - m - 0.12;
    let by0 = top + 0.05;
    let in_bb = has_bb * step(bx0, fx) * step(fx, bx1) * step(by0, y) * step(y, by0 + 0.1);
    let u = (fx - bx0) / (bx1 - bx0).max(0.01);
    let band = fract(u * 3.0 - t * 0.25 + h1);
    let bb = k.n1.mix(k.warm, band).mix(k.n2, smoothstep(0.7, 1.0, band));
    let scan = 0.8 + 0.2 * (y * 500.0).sin();
    col = col.mix(bb * 1.5 * scan, in_bb);
    let bhx = ((fx - 0.5) / scale).abs();
    let bhy = ((y - (by0 + 0.05)).abs() - 0.05).max(0.0);
    col = col + bb * has_bb * (-(bhx * bhx + bhy * bhy) / 0.003).exp() * 0.3;
    col
}

/// A flying car: headlights, strobes and a light cone onto the street.
fn spinner(x: f32, y: f32, t: f32, k: &Look, aspect: f32, seed: f32) -> Vec3 {
    let u = fract(t * (0.045 + 0.02 * seed) + seed * 0.37);
    let dir = if seed > 0.5 { -1.0 } else { 1.0 };
    let px = if dir > 0.0 {
        mix(-0.15, aspect + 0.15, u)
    } else {
        mix(aspect + 0.15, -0.15, u)
    };
    let py = 0.26 + 0.07 * (u * 9.0 + seed * 5.0).sin() + 0.1 * seed;
    let dx = x - px;
    let dy = y - py;
    let head = vec2(dx - dir * 0.022, dy).len();
    let tail = vec2(dx + dir * 0.022, dy).len();
    let body = smoothstep(0.03, 0.01, vec2(dx, dy * 2.2).len());
    let strobe = 0.8 + 0.2 * (t * 1.5 + seed * 10.0).sin();
    let mut c = Vec3::WHITE * (0.00045 / (head * head + 0.00001)) * 0.05;
    c = c + k.n1 * (0.0004 / (tail * tail + 0.00001)) * 0.05 * strobe;
    c = c + k.warm * body * 0.25;
    let down = dy.max(0.0);
    let width = 0.012 + down * 0.3;
    let cone = smoothstep(width, 0.0, dx.abs() - dir * 0.0) * (-down * 5.0).exp() * step(0.0, dy);
    c + k.warm * cone * 0.22 * k.spinner
}

fn city_scene(x: f32, y: f32, t: f32, aspect: f32, k: &Look) -> Vec3 {
    let mut col = sky(x, y, t, k);
    for l in 0..5 {
        col = tower_layer(x, y, t, l, 5, k, col);
    }
    if k.spinner > 0.0 {
        col = col + spinner(x, y, t, k, aspect, 0.2) + spinner(x, y, t, k, aspect, 0.8);
    }
    col
}

/// Slanted rain streaks; `n` columns, brightness per streak.
fn rain_layer(uv: Vec2, aspect: f32, t: f32, n: f32, speed: f32, len: f32, seed: f32) -> f32 {
    let sx = (uv.x + uv.y * 0.12) * aspect * n;
    let id = sx.floor();
    let fx = sx - id;
    let hs = hash1(id + seed);
    let ph = fract(uv.y * 1.4 - t * speed * (0.8 + 0.5 * hs) + hs * 9.0);
    // bright head at the lower (leading) end, trail above it
    let q = 1.0 - ph;
    let head = smoothstep(len, 0.0, q) * smoothstep(0.0, 0.03, q);
    let w = smoothstep(0.5, 0.0, (fx - 0.5).abs() * 2.0 - 0.7 + 0.4 * hs);
    head * w * step(0.35, hash1(id + seed + 5.0))
}

fn rain(uv: Vec2, aspect: f32, t: f32, tint: Vec3) -> Vec3 {
    let r = rain_layer(uv, aspect, t, 60.0, 1.5, 0.25, 1.0) * 0.30
        + rain_layer(uv, aspect, t, 110.0, 1.9, 0.18, 2.0) * 0.22
        + rain_layer(uv, aspect, t, 200.0, 2.4, 0.12, 3.0) * 0.16;
    (tint * 0.8 + Vec3::WHITE * 0.2) * r
}

/// Wet street: mirrored scene, puddle patches, ripples and car lights.
fn street(x: f32, y: f32, t: f32, aspect: f32, k: &Look) -> Vec3 {
    let depth = (y - k.street) / (1.0 - k.street);
    let pud = smoothstep(0.42, 0.62, noise(vec2(x * 3.0, depth * 7.0 + 3.0)));
    let wob = (noise(vec2(x * 42.0, y * 110.0 + t * 0.7)) - 0.5) * 0.012 * (0.4 + depth);
    // ripples from falling drops
    let g = vec2(x * 9.0, depth * 22.0);
    let id = g.floor();
    let f = g.fract() - 0.5;
    let hh = hash2(id.x as i32, id.y as i32);
    let ph = fract(t * 0.8 + hh * 5.0);
    let rd = vec2(f.x, f.y * 2.4).len();
    let ring = smoothstep(0.03, 0.0, (rd - ph * 0.5).abs()) * (1.0 - ph) * step(0.45, hh);
    let ry = k.street - (y - k.street) * 1.15 + ring * 0.01;
    let refl = (city_scene(x + wob + ring * 0.01, ry, t, aspect, k)
        + city_scene(x + wob * 1.7, ry - 0.012 - 0.02 * depth, t, aspect, k) * 0.8
        + city_scene(x + wob * 0.6, ry + 0.01, t, aspect, k) * 0.6)
        / 2.4;
    let base = k.sky * (0.8 - 0.4 * depth) + k.fog * 0.12 * (1.0 - depth);
    let mut col = base + refl * (0.55 + 0.6 * pud) * (1.0 - depth * 0.5);
    col = col + Vec3::WHITE * ring * 0.35 * (0.5 + pud);
    // headlight streaks in two lanes
    for lane in 0..3 {
        let ly = k.street + 0.045 + 0.06 * lane as f32;
        let dir = if lane == 1 { -1.0 } else { 1.0 };
        let bx = fract(t * (0.12 + 0.05 * lane as f32) * dir + 0.31 * lane as f32) * (aspect + 0.4)
            - 0.2;
        let dy = (y - ly) / 0.006;
        let dx = (x - bx) / 0.07;
        let lc = if dir > 0.0 { k.warm } else { k.n1 };
        col = col + lc * (-(dy * dy + dx * dx)).exp() * 0.6;
        col = col + lc * (-(dy * dy) * 0.3).exp() * (-(dx * dx) * 0.05).exp() * 0.04;
    }
    col
}

fn finish(col: Vec3, uv: Vec2, t: f32) -> Vec3 {
    let v = uv - 0.5;
    let vig = 1.0 - 0.55 * (v.x * v.x * 0.9 + v.y * v.y * 1.3);
    let grain = (hash3(
        (uv.x * 997.0) as i32,
        (uv.y * 991.0) as i32,
        (t * 6.0) as i32,
    ) - 0.5)
        * 0.012;
    (col * vig + grain).clamp01()
}

fn city_main(uv: Vec2, c: &Ctx, k: &Look) -> Vec3 {
    let a = c.aspect();
    let x = uv.x * a;
    let t = c.time;
    let col = if uv.y < k.street {
        city_scene(x, uv.y, t, a, k)
    } else {
        street(x, uv.y, t, a, k)
    };
    let rn = rain(uv, a, t, k.fog * 2.0 + k.cool * 0.3);
    finish(col + rn * 0.45, uv, t)
}

/// Rainy cyberpunk skyline.
pub fn neon_city(uv: Vec2, c: &Ctx) -> Vec3 {
    let k = neon_look(&c.pal);
    city_main(uv, c, &k)
}

/// Smoggy orange megacity with searchlights, billboards and a spinner.
pub fn br_city(uv: Vec2, c: &Ctx) -> Vec3 {
    let k = br_look(&c.pal);
    city_main(uv, c, &k)
}

// ---------------------------------------------------------------- street --

/// Facade at depth `z` (along the street) and height `y` above the road.
fn wall(z: f32, y: f32, side: f32, t: f32, k: &Look) -> Vec3 {
    let u = z * 14.0;
    let tile = u.floor();
    let ti = tile as i32 + (side * 50.0) as i32;
    let fu = u - tile;
    let v = y * 24.0;
    let h1 = hash2(ti, 11);
    let h2 = hash2(ti, 12);
    let mut col = k.sky * (1.2 + 1.5 * h1) + k.fog * 0.06 * (0.5 + h2);
    // window grid
    let gx = fu * 4.0;
    let gy = v * 1.4;
    let lit = hash3(ti, gx.floor() as i32, gy.floor() as i32);
    let wm = smoothstep(0.12, 0.24, fract(gx))
        * smoothstep(0.88, 0.76, fract(gx))
        * smoothstep(0.15, 0.3, fract(gy))
        * smoothstep(0.9, 0.75, fract(gy));
    let wc = if lit > 0.86 {
        k.n1
    } else if lit > 0.7 {
        k.cool
    } else {
        k.warm
    };
    col = col + wc * wm * step(lit, 0.3 + 0.3 * h1) * 0.9 * step(1.4, v);
    // vertical neon sign hanging off the facade
    let sgn = step(0.5, h2);
    let sx = smoothstep(0.08, 0.14, fu) * smoothstep(0.3, 0.24, fu);
    let sy = smoothstep(1.4, 1.6, v) * smoothstep(7.0, 6.8, v);
    let seg = (v * 1.6).floor();
    let glyph = step(0.22, fract(v * 1.6)) * step(0.3, hash2(ti + 9, seg as i32));
    let flick = if h1 > 0.8 {
        0.45 + 0.55 * smoothstep(0.15, 0.45, fract(t * 0.25 + h2 * 7.0))
    } else {
        1.0
    };
    let sc = if h1 > 0.5 { k.n1 } else { k.n2 };
    col = col.mix(sc * 1.9, sgn * sx * sy * glyph * flick);
    col = col + sc * sgn * flick * (-((fu - 0.19) * (fu - 0.19)) / 0.02).exp() * 0.3 * sy;
    // shopfronts at street level
    let shop = smoothstep(1.4, 1.0, v);
    let gl = smoothstep(0.08, 0.2, fract(u * 0.5)) * smoothstep(0.95, 0.85, fract(u * 0.5));
    col = col + k.warm.mix(k.n2, h2) * shop * (0.1 + 0.3 * gl);
    col
}

/// Street level: a canyon of towers, wet road, umbrellas and steam.
pub fn br_street(uv: Vec2, c: &Ctx) -> Vec3 {
    let k = br_look(&c.pal);
    let a = c.aspect();
    let t = c.time;
    let hcam = 0.05;
    let vp = vec2(0.5 * a, 0.42);
    let p = vec2(uv.x * a, uv.y) - vp;
    let ax = p.x.abs().max(0.0005);
    let side = if p.x < 0.0 { 0.0 } else { 1.0 };
    let on_floor = p.y > 0.5 * ax;
    let in_slot = p.y < -12.0 * ax;
    let mut col;
    if on_floor {
        let z = hcam / p.y;
        let xl = p.x * z / 0.1; // -1..1 between the walls
        let pud = smoothstep(0.35, 0.6, noise(vec2(xl * 1.6, z * 30.0)));
        let wob = (noise(vec2(xl * 14.0, z * 160.0 + t * 0.7)) - 0.5) * 0.25;
        // reflection of the facade nearest to this side, smeared downward
        let ex = xl.abs();
        let wl = wall(z, 0.01 + 0.04 * (1.0 - ex) + wob * 0.02, side, t, &k);
        let wl2 = wall(z, 0.05 + 0.1 * (1.0 - ex) + wob * 0.03, side, t, &k);
        let refl = (wl + wl2) * 0.5;
        let edge = smoothstep(0.1, 1.0, ex);
        let fogz = 1.0 - (-z * 1.4).exp();
        col = k.sky * 0.9 + refl * (0.25 + 0.6 * pud) * (0.25 + 0.9 * edge);
        // headlight streaks down the middle
        let lane =
            (-((xl - 0.1) * (xl - 0.1)) / 0.02).exp() * (0.5 + 0.5 * (z * 40.0 - t * 2.0).sin());
        col = col + k.warm * lane * 0.12 * (1.0 - fogz);
        // ripples
        let g = vec2(xl * 6.0, z * 70.0);
        let id = g.floor();
        let f = g.fract() - 0.5;
        let hh = hash2(id.x as i32, id.y as i32 + 40);
        let ph = fract(t * 0.8 + hh * 5.0);
        let ring = smoothstep(0.04, 0.0, (vec2(f.x, f.y * 0.7).len() - ph * 0.5).abs())
            * (1.0 - ph)
            * step(0.5, hh);
        col = col + Vec3::WHITE * ring * 0.25;
        col = col.mix(k.fog * 0.9, fogz);
    } else if in_slot {
        col = sky(uv.x * a, 0.3 + uv.y * 0.3, t, &k) + k.fog * 0.3;
    } else {
        let z = 0.1 / ax;
        let yh = -p.y * z + hcam; // height above the road
        col = wall(z, yh, side, t, &k);
        let fogz = 1.0 - (-z * 1.4).exp();
        col = col.mix(k.fog * 0.9, fogz);
        // darker towards the top of the canyon
        col = col * (0.6 + 0.4 * smoothstep(0.35, 0.0, yh));
    }
    // walkers with umbrellas
    for i in 0..5 {
        let fi = i as f32;
        let wz = fract(t * 0.03 * (0.6 + 0.2 * fi) + fi * 0.21);
        let s = 0.04 + 0.28 * wz * wz;
        let wy = vp.y + 0.02 + 0.45 * wz * wz;
        let wx = vp.x + (hash1(fi + 3.0) - 0.5) * 1.5 * wz * wz;
        let q = (vec2(uv.x * a, uv.y) - vec2(wx, wy)) / s;
        let body = sd_round_box(q - vec2(0.0, -0.9), vec2(0.12, 0.45), 0.1);
        let head = sd_circle(q - vec2(0.0, -1.5), 0.1);
        let umb = (q - vec2(0.0, -1.62)).len() - 0.5;
        let umb = umb.max(q.y + 1.62);
        let aa = 0.01 / s.max(0.04);
        let sil = fill_aa(body.min(head), aa).max(fill_aa(umb, aa));
        let ucol = if i % 2 == 0 { k.n1 } else { k.n2 };
        col = col.mix(k.sky * 0.2, sil * 0.95);
        col = col + ucol * glow(umb, 60.0) * step(-1.7, q.y) * wz * 0.35 * (1.0 - sil);
    }
    // steam rising from a grate
    let st = fbm(vec2(p.x * 7.0, p.y * 3.0 - t * 0.5), 4);
    let pl = smoothstep(0.3, 0.0, (uv.x * a - (vp.x - 0.2)).abs())
        * smoothstep(0.2, 0.7, uv.y)
        * smoothstep(1.0, 0.7, uv.y);
    col = col + k.fog * 1.6 * st * pl * 0.5;
    col = col + rain(uv, a, t, k.fog * 2.0 + k.cool * 0.3) * 0.5;
    finish(col, uv, t)
}

// --------------------------------------------------------------- pyramid --

/// A stepped corporate ziggurat, lit from within, under searchlights.
pub fn br_tyrell(uv: Vec2, c: &Ctx) -> Vec3 {
    let k = br_look(&c.pal);
    let a = c.aspect();
    let t = c.time;
    let x = uv.x * a;
    let cx = 0.5 * a;
    let base_y = 0.82;
    let mut col = sky(x, uv.y, t, &k);
    // distant towers
    for l in 0..1 {
        col = tower_layer(x, uv.y, t, l, 5, &k, col);
    }
    // pyramid: 7 setback tiers
    let dx = (x - cx).abs();
    let ty = (base_y - uv.y) / 0.62; // 0 base .. 1 apex
    let mut tier_i = 0.0;
    let mut hit = 0.0;
    if ty > 0.0 && ty < 1.0 {
        tier_i = (ty * 7.0).floor();
        let half = 0.46 * (1.0 - tier_i / 7.0) * a * 0.45 + 0.01;
        hit = step(dx, half);
        let _ = half;
    }
    let tf = if ty > 0.0 { fract(ty * 7.0) } else { 0.0 };
    let face = k.sky * 0.8 + k.fog * (0.35 + 0.4 * tf);
    // lit window rows glowing from inside
    let gx = (x - cx) * 60.0;
    let lit = hash3(gx.floor() as i32, tier_i as i32, (tf * 5.0) as i32);
    let rows = smoothstep(0.2, 0.3, fract(tf * 5.0)) * smoothstep(0.85, 0.75, fract(tf * 5.0));
    let cols = smoothstep(0.1, 0.25, fract(gx)) * smoothstep(0.9, 0.75, fract(gx));
    let win = k.warm * rows * cols * step(lit, 0.8) * (0.9 + 0.08 * (t * 0.8 + lit * 40.0).sin());
    let pyr = face + win * 1.3 + k.warm * smoothstep(0.1, 0.0, tf) * 0.4;
    col = col.mix(pyr, hit);
    // apex beacon
    let ap = vec2(x - cx, uv.y - (base_y - 0.62)).len();
    col = col + k.warm * (0.0006 / (ap * ap + 0.00004)) * 0.1;
    // haze at the foot and a streetline
    col = col + k.fog * smoothstep(0.55, base_y, uv.y) * 0.4 * (1.0 - hit * 0.3);
    if uv.y > base_y {
        let depth = (uv.y - base_y) / (1.0 - base_y);
        let wob = (noise(vec2(x * 40.0, uv.y * 100.0 + t * 0.6)) - 0.5) * 0.012;
        let ry = base_y - (uv.y - base_y) * 1.1;
        let rdx = (x + wob - cx).abs();
        let rty = (base_y - ry) / 0.62;
        let rh = step(
            rdx,
            0.46 * (1.0 - (rty * 7.0).floor() / 7.0) * a * 0.45 + 0.01,
        ) * step(0.0, rty)
            * step(rty, 1.0);
        // lit windows mirrored in the wet street
        let rtf = fract(rty * 7.0);
        let rgx = (x + wob - cx) * 60.0;
        let rrows =
            smoothstep(0.2, 0.3, fract(rtf * 5.0)) * smoothstep(0.85, 0.75, fract(rtf * 5.0));
        let rcols = smoothstep(0.1, 0.25, fract(rgx)) * smoothstep(0.9, 0.75, fract(rgx));
        let rlit = step(
            hash3(
                rgx.floor() as i32,
                (rty * 7.0).floor() as i32,
                (rtf * 5.0) as i32,
            ),
            0.8,
        );
        let refl = (k.fog * 0.8 + k.warm * 0.25 * rtf + k.warm * rrows * rcols * rlit * 0.9) * rh
            + sky(x + wob, ry, t, &k) * (1.0 - rh);
        col = k.sky * 0.7 + refl * (0.85 - 0.45 * depth);
    }
    finish(col + rain(uv, a, t, k.fog * 2.0) * 0.45, uv, t)
}

// ----------------------------------------------------------- refinery ----

/// Smokestacks venting flame under a smoggy orange sky.
pub fn br_off_world(uv: Vec2, c: &Ctx) -> Vec3 {
    let k = br_look(&c.pal);
    let a = c.aspect();
    let t = c.time;
    let x = uv.x * a;
    let h = smoothstep(0.0, 0.85, uv.y);
    let mut col = k.sky.mix(k.fog * 1.6, h * h);
    let cl = fbm(vec2(x * 1.6 + t * 0.03, uv.y * 4.0), 5);
    col = col + k.warm * (cl - 0.3).max(0.0) * 0.7 * (0.3 + h);
    // glowing core of smog near the horizon
    col = col + k.warm * 0.35 * (-((uv.y - 0.78) * (uv.y - 0.78)) / 0.02).exp();
    // stacks: 4 columns with flame
    for i in 0..4 {
        let fi = i as f32;
        let sx = a * (0.14 + 0.24 * fi) + (hash1(fi + 1.0) - 0.5) * 0.08;
        let sh = 0.33 + 0.18 * hash1(fi + 7.0);
        let top = 0.9 - sh;
        let w = 0.018 + 0.01 * hash1(fi + 4.0);
        let d = (x - sx).abs();
        let body = step(d, w * (1.0 + 0.8 * (uv.y - top))) * step(top, uv.y);
        // flame plume
        let fy = top - uv.y; // above the top
        let wob = fbm(vec2(x * 18.0 + fi * 5.0, uv.y * 12.0 - t * 3.0), 3);
        let fw = 0.002 + 0.03 * (1.0 - fy / 0.2).max(0.0) * (0.6 + wob);
        let flame = smoothstep(fw, 0.0, (x - sx + (wob - 0.5) * 0.05).abs())
            * step(0.0, fy)
            * smoothstep(0.22, 0.0, fy);
        let fc = Vec3::WHITE
            .mix(k.warm, sat(fy * 6.0))
            .mix(k.n1, sat(fy * 3.0 - 0.5));
        col = col + fc * flame * (1.4 + 0.15 * (t * 2.5 + fi * 3.0).sin());
        col = col + k.warm * (-(d * d + fy * fy * 0.4) / 0.02).exp() * 0.3;
        col = col.mix(k.sky * 0.35, body);
        // lattice + red lights on the stack
        let lamp = vec2(
            x - sx,
            uv.y - (top + 0.08 * (t * 0.5 + fi).sin().abs() + 0.12),
        )
        .len();
        col = col
            + vec3(1.0, 0.2, 0.1) * 0.00025 / (lamp * lamp + 0.00001)
                * 0.1
                * smoothstep(0.5, 0.7, fract(t * 0.2 + fi * 0.3));
    }
    // refinery silhouette along the ground
    let ground = 0.9 + 0.01 * noise(vec2(x * 30.0, 1.0));
    let pipes = step(0.86, uv.y) * step(hash2((x * 18.0) as i32, 3), 0.7);
    let hz = if uv.y > ground || pipes > 0.0 {
        1.0
    } else {
        0.0
    };
    col = col.mix(k.sky * 0.25, hz);
    finish(col, uv, t)
}

// -------------------------------------------------------------------- eye --

/// Close-up of an eye being scanned.
pub fn br_eye(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let t = c.time;
    let q = c.centered(uv);
    let mut col = p.bg * 1.5 + p.b * 0.03 * (1.0 - q.len() * 0.5);
    // almond shape
    let lid = (q.y.abs() - 0.42 * (1.0 - (q.x * 0.62) * (q.x * 0.62)).max(0.0)).max(-1.0);
    let inside = smoothstep(0.02, -0.02, lid);
    let r = q.len();
    let ang = q.y.atan2(q.x);
    let pupil_r = 0.13 + 0.035 * (t * 1.3).sin() + 0.02 * (t * 3.1).sin();
    let iris_r = 0.3;
    // iris: radial fibres + crypts
    let fib = fbm(vec2(ang * 5.0 + 3.0, r * 14.0), 4);
    let fib2 = noise(vec2(ang * 22.0, r * 4.0));
    let iris_base = p.a * 0.35 + p.b * 0.12;
    let mut iris = iris_base * (0.5 + fib * 1.1) + p.a * fib2 * 0.25;
    iris = iris + p.b * smoothstep(iris_r * 0.7, iris_r, r) * 0.25;
    iris = iris * (0.55 + 0.6 * smoothstep(pupil_r, pupil_r + 0.08, r));
    // collarette ring and limbal darkening
    iris = iris + p.a * smoothstep(0.02, 0.0, (r - pupil_r * 1.7).abs()) * 0.3;
    iris = iris * (1.0 - 0.7 * smoothstep(iris_r - 0.05, iris_r, r));
    let mut eye = Vec3::WHITE * 0.75 * (0.7 + 0.3 * (1.0 - smoothstep(0.3, 0.9, r)));
    eye = eye + p.c * 0.12 * noise(vec2(q.x * 20.0, q.y * 20.0)) * smoothstep(0.3, 0.9, r); // veins
    eye = eye * (0.65 + 0.35 * smoothstep(0.95, 0.3, q.x.abs()));
    eye = eye.mix(iris, smoothstep(iris_r, iris_r - 0.015, r));
    eye = eye.mix(Vec3::BLACK, smoothstep(pupil_r, pupil_r - 0.01, r));
    // reflections of window light
    let sp = (q - vec2(-0.1, 0.1)).len();
    eye = eye + Vec3::WHITE * smoothstep(0.05, 0.0, sp) * 0.9;
    let sp2 = (q - vec2(0.12, -0.08)).len();
    eye = eye + p.b * smoothstep(0.03, 0.0, sp2) * 0.5;
    // lid shadow
    eye = eye * (0.35 + 0.65 * smoothstep(0.0, -0.18, lid));
    col = col.mix(eye, inside);
    col = col + p.fg * smoothstep(0.02, 0.0, lid.abs()) * 0.12;
    // scan reticle
    let scan_y = fract(t * 0.35) * 2.0 - 1.0;
    col = col + p.b * smoothstep(0.012, 0.0, (q.y - scan_y).abs()) * 0.55 * inside;
    col = col + p.b * smoothstep(0.08, 0.0, (q.y - scan_y).abs()) * 0.08;
    let rd = (r - 0.55).abs();
    col = col + p.b * smoothstep(0.006, 0.0, rd) * step(0.5, fract(ang * 1.6 + t * 0.3)) * 0.5;
    col = col + p.a * smoothstep(0.004, 0.0, (r - iris_r - 0.04).abs()) * 0.4;
    // tick marks
    let tick = smoothstep(0.012, 0.0, ((ang * 36.0 / TAU).fract() - 0.5).abs() * 0.1)
        * smoothstep(0.012, 0.0, (r - 0.62).abs() - 0.02);
    col = col + p.b * tick * 0.6;
    // read-out bars
    for i in 0..6 {
        let fi = i as f32;
        let by = 0.7 - fi * 0.22;
        let val = 0.3 + 0.7 * hash2((t * 0.8 + fi) as i32, i + 5);
        let in_row = step((q.y - by).abs(), 0.025);
        let lenx = val * 0.5;
        let left = step(-q.x - 1.0, 0.0) * step(0.0, -q.x - 1.0 + lenx) * in_row;
        let _ = left;
        let bx = (q.x + c.aspect() * 0.95) / 0.5;
        let bar = in_row * step(0.0, bx) * step(bx, val);
        col = col + p.a * bar * 0.7 + p.b * in_row * step(0.0, bx) * step(bx, 1.0) * 0.12;
    }
    let vig = 1.0 - 0.5 * q.dot(q) * 0.5;
    (col * vig).clamp01()
}

// ------------------------------------------------------------------ post --

pub fn br_rain(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let a = c.aspect();
    let t = c.time;
    let mut off = 0.0;
    let r1 = rain_layer(uv, a, t, 60.0, 1.5, 0.25, 11.0);
    let r2 = rain_layer(uv, a, t, 110.0, 1.9, 0.18, 12.0);
    let r3 = rain_layer(uv, a, t, 200.0, 2.4, 0.12, 13.0);
    off += r1 * 0.006 + r2 * 0.003;
    let base = src.sample(uv + vec2(off * c.intensity, -off * c.intensity * 0.5));
    let streak = (r1 * 0.5 + r2 * 0.35 + r3 * 0.25) * c.intensity;
    let tint = base * 0.5 + Vec3::WHITE * 0.4;
    // damp the image a little
    base * (1.0 - 0.12 * c.intensity) + tint * streak * 0.55
}

pub fn br_grade(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let s = src.sample(uv);
    // halation: warm glow from bright areas
    let px = c.texel();
    let mut hal = Vec3::BLACK;
    for i in 0..8 {
        let a = i as f32 * TAU / 8.0;
        let o = vec2(a.cos(), a.sin());
        let t1 = src.sample(uv + vec2(o.x * px.x, o.y * px.y) * 6.0);
        let t2 = src.sample(uv + vec2(o.x * px.x, o.y * px.y) * 14.0);
        hal = hal + t1 * (t1.luma() - 0.3).max(0.0) + t2 * (t2.luma() - 0.3).max(0.0) * 0.8;
    }
    let l = s.luma();
    let teal = p.b * 0.5 + p.bg * 0.5;
    let shadow_tint = Vec3::WHITE.mix(teal * 2.0 + 0.3, 0.55 * (1.0 - smoothstep(0.0, 0.5, l)));
    let hi_tint = Vec3::WHITE.mix(p.a * 1.2 + 0.2, 0.5 * smoothstep(0.25, 0.9, l));
    let mut g = s * shadow_tint * hi_tint;
    g = g + (hal / 8.0) * (p.a * 0.8 + p.c * 0.15) * 0.9;
    // S-curve
    let g = vec3(
        g.x * g.x * (3.0 - 2.0 * g.x.min(1.0)),
        g.y * g.y * (3.0 - 2.0 * g.y.min(1.0)),
        g.z * g.z * (3.0 - 2.0 * g.z.min(1.0)),
    );
    s.mix(g * 1.12, c.intensity)
}

pub fn br_anamorphic(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let s = src.sample(uv);
    let px = c.texel();
    let mut streak = Vec3::BLACK;
    for i in 1..16 {
        let d = i as f32;
        let w = (-d * 0.14).exp();
        let l = src.sample(uv + vec2(px.x * d * 6.0, 0.0));
        let r = src.sample(uv - vec2(px.x * d * 6.0, 0.0));
        let lt = (l.luma() - 0.55).max(0.0);
        let rt = (r.luma() - 0.55).max(0.0);
        streak = streak + (l * lt + r * rt) * w;
    }
    let tint = Vec3::WHITE * 0.4 + p.b * 0.9;
    s + streak * tint * 0.55 * c.intensity
}

pub fn br_wet(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let hz = 0.62;
    let t = c.time;
    let a = c.aspect();
    let depth = ((uv.y - hz) / (1.0 - hz)).max(0.0);
    let wob = (noise(vec2(uv.x * a * 40.0, uv.y * 120.0 + t * 0.7)) - 0.5) * 0.015 * (0.4 + depth);
    let g = vec2(uv.x * a * 9.0, depth * 20.0);
    let id = g.floor();
    let f = g.fract() - 0.5;
    let hh = hash2(id.x as i32, id.y as i32);
    let ph = fract(t * 0.8 + hh * 5.0);
    let ring = smoothstep(0.03, 0.0, (vec2(f.x, f.y * 2.4).len() - ph * 0.5).abs())
        * (1.0 - ph)
        * step(0.45, hh);
    let ry = (hz - (uv.y - hz) * 1.05).clamp(0.0, 1.0);
    let refl = src.sample(vec2(uv.x + wob + ring * 0.01, ry));
    let pud = smoothstep(0.4, 0.62, noise(vec2(uv.x * a * 3.0, depth * 7.0)));
    let below = src.sample(uv) * (0.55 - 0.2 * depth)
        + refl * (0.3 + 0.5 * pud) * (1.0 - depth * 0.5)
        + Vec3::WHITE * ring * 0.25;
    let on = step(hz, uv.y);
    let s = src.sample(uv);
    s.mix(s.mix(below, c.intensity), on)
}

pub fn br_smog(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let t = c.time;
    let a = c.aspect();
    let s = src.sample(uv);
    let px = c.texel();
    let mut lit = 0.0;
    for i in 0..6 {
        let ang = i as f32 * TAU / 6.0;
        let o = vec2(ang.cos(), ang.sin()) * 18.0;
        lit += src.sample(uv + vec2(o.x * px.x, o.y * px.y)).luma();
    }
    lit /= 6.0;
    let n = fbm(vec2(uv.x * a * 2.2 + t * 0.03, uv.y * 3.5 - t * 0.01), 5);
    let haze = (p.a * 0.6 + p.c * 0.1) * (n - 0.25).max(0.0) * (0.15 + 1.1 * lit);
    let flat = s * (1.0 - 0.2 * c.intensity) + p.bg * 0.2 * c.intensity;
    flat + haze * 1.2 * c.intensity
}

pub fn br_searchlights(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let t = c.time;
    let a = c.aspect();
    let x = uv.x * a;
    let y = uv.y;
    let hz = 0.8;
    let s = (t * 0.35).sin();
    let b = beam(x, hz, 0.45 * a, hz, -0.45 + 0.5 * s)
        + beam(x, hz, 0.8 * a, hz, 0.35 - 0.5 * s)
        + beam(x, hz, 0.62 * a, hz, 0.05 + 0.35 * (t * 0.23).sin());
    let base = src.sample(uv);
    base + (p.a * 0.6 + p.fg * 0.45)
        * b
        * (0.6 + 0.4 * noise(vec2(x * 8.0, y * 8.0 + t)))
        * c.intensity
}
