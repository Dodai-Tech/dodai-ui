//! Hand-painted anime looks, in the spirit of Studio Ghibli films: billowing
//! cumulus, rolling meadows, painted seas, misty forests, soot sprites, plus
//! cel-shaded 3D scenes with ink outlines. Original code and palettes, only the
//! mood is borrowed. Nothing here is taken from any film or other shader.
//!
//! GPU-friendly style: index loops, no early returns, no colour arrays.

#![allow(
    clippy::neg_multiply,
    clippy::approx_constant,
    clippy::needless_late_init,
    clippy::let_and_return
)]

use crate::frame::{Ctx, Frame};
use crate::math::*;
#[allow(unused_imports)]
use crate::prelude::*;
use crate::{Family, ShaderInfo};

const F: Family = Family::Ghibli;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::generator(
        "gh_sea",
        "Painted sea",
        F,
        "Rolling cel-shaded waves with foam, glints and a little sailboat",
        gh_sea,
    ),
    ShaderInfo::generator(
        "gh_soot",
        "Soot sprites",
        F,
        "Fuzzy black dust bunnies bouncing in a warm room",
        gh_soot,
    ),
    ShaderInfo::generator(
        "gh_night_train",
        "Night train",
        F,
        "A lit train gliding over a still moonlit sea",
        gh_night_train,
    ),
    ShaderInfo::generator(
        "gh_laputa",
        "Floating island",
        F,
        "3D cel-shaded sky island with castle, giant tree and ink outlines",
        gh_laputa,
    ),
    ShaderInfo::generator(
        "gh_pod",
        "Landed pod",
        F,
        "3D cel-shaded round capsule ship on a green island by the sea",
        gh_pod,
    ),
    ShaderInfo::post(
        "gh_painted",
        "Painted cel",
        F,
        "Paper grain, soft colour and ink lines on the edges",
        gh_painted,
    ),
    ShaderInfo::post(
        "gh_dream",
        "Dreamy glow",
        F,
        "Warm bloom and lifted, blue-ish shadows",
        gh_dream,
    ),
];

// ------------------------------------------------------------- colours --

const SKY_TOP: Vec3 = rgb8(70, 140, 214);
const SKY_MID: Vec3 = rgb8(128, 190, 236);
const SKY_HZ: Vec3 = rgb8(226, 240, 236);
const CLOUD_LIT: Vec3 = rgb8(255, 252, 244);
const CLOUD_SHADE: Vec3 = rgb8(178, 196, 226);
const INK: Vec3 = rgb8(22, 30, 42);

fn sky2d(y: f32) -> Vec3 {
    let a = SKY_HZ.mix(SKY_MID, smoothstep(0.0, 0.45, y));
    a.mix(SKY_TOP, smoothstep(0.35, 1.0, y))
}

/// Painted cumulus. Returns `(cover, light)`; `light` is 1 on sunny rims.
fn cumulus(q: Vec2, t: f32, seed: f32, base: f32, scale: f32) -> (f32, f32) {
    let p = vec2(q.x * scale + t * 0.03 + seed * 7.1, q.y * scale * 1.5);
    let f = |dx: f32, dy: f32| -> f32 {
        let n = fbm(vec2(p.x + dx, p.y + dy) * 0.6 + vec2(seed * 3.7, 0.0), 5);
        // flat bottoms, puffy tops
        n - 0.5 - (base - q.y) * 0.0 + (q.y - base) * 0.35 * (1.0 + 0.0 * seed)
            - smoothstep(base, base - 0.4, q.y) * 0.0
    };
    let n0 = f(0.0, 0.0);
    let n1 = f(-0.06, -0.08); // toward the sun (up-left)
    let flat_bottom = smoothstep(base - 0.05, base + 0.02, q.y);
    let cover = smoothstep(0.02, 0.07, n0 + 0.0)
        * flat_bottom
        * (1.0 - smoothstep(base + 0.45, base + 0.7, q.y));
    let light = smoothstep(0.0, 0.05, n1 - n0 + 0.02);
    (cover, light)
}

fn cloud_col(cover: f32, light: f32, sky: Vec3, shade: Vec3) -> Vec3 {
    let c = shade.mix(CLOUD_LIT, light);
    sky.mix(c, cover)
}

// -------------------------------------------------------------- 2D scenes --

pub fn gh_sea(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let hz = 0.1;
    let mut col = sky2d((q.y - hz) * 0.9 + 0.2);
    let (cv, lt) = cumulus(q, t, 3.0, 0.3, 0.7);
    col = cloud_col(cv, lt, col, CLOUD_SHADE);
    if q.y < hz {
        let d = (hz - q.y) / (1.0 + hz); // 0 at horizon .. 1 bottom
        let persp = 1.0 / (0.15 + d * 1.6);
        let x = q.x * persp;
        let z = persp;
        // layered wave bands that roll toward the viewer
        let wv = (z * 5.5 - t * 1.1
            + (x * 0.6).sin() * 0.7
            + noise(vec2(x * 0.7, z * 1.2 + t * 0.1)) * 1.8)
            .sin();
        let band = smoothstep(-0.1, 0.1, wv);
        let deep = rgb8(30, 98, 170);
        let mid = rgb8(56, 150, 204);
        let light = rgb8(120, 206, 224);
        let mut sea = deep.mix(mid, smoothstep(0.0, 0.8, d));
        sea = sea.mix(light, band * 0.55 * (1.0 - d * 0.3));
        // foam crest on the wave edge
        let crest = smoothstep(0.12, 0.0, (wv - 0.55).abs())
            * (0.5 + 0.5 * noise(vec2(x * 3.0, z * 4.0 - t)));
        sea = sea.mix(rgb8(250, 252, 248), crest * smoothstep(0.0, 0.06, d));
        // sun glints
        let g = smoothstep(0.82, 0.9, noise(vec2(x * 4.0 + t * 0.4, z * 8.0 - t * 0.7)))
            * smoothstep(1.4, 0.0, (q.x - 0.7).abs() * 1.5 + d * 0.4);
        sea = sea + rgb8(255, 240, 190) * g * 0.7;
        // horizon haze
        sea = sea.mix(SKY_HZ, 1.0 - smoothstep(0.0, 0.18, d));
        col = sea;
    }
    // sailboat
    let bob = (t * 1.1).sin() * 0.012;
    let bp = (q - vec2(-0.45, hz - 0.3 + bob)) * 0.8;
    let rot = (t * 0.8).sin() * 0.04;
    let bpr = bp.rot(rot);
    let hull = sd_round_box(bpr - vec2(0.0, -0.01), vec2(0.17, 0.03), 0.025);
    let sail1 = sd_segment(bpr, vec2(0.0, 0.0), vec2(0.0, 0.34)) - 0.004;
    let tri = (bpr.x + 0.01).max(-(bpr.x + 0.01) * 0.0 + (bpr.y - 0.03) * 0.28 - 0.0);
    let s1 = (bpr.x - 0.012 * 1.0).max(0.0) * 0.0
        + (bpr.x - (0.15 - bpr.y * 0.42))
            .max(-(bpr.x - 0.012))
            .max(0.03 - bpr.y);
    let sail = s1.max(bpr.y - 0.33);
    let _ = tri;
    col = col.mix(rgb8(250, 244, 230), fill_aa(sail, 0.004));
    col = col.mix(rgb8(220, 90, 70), fill_aa(hull, 0.004));
    col = col.mix(INK, fill_aa(sail1, 0.003));
    col
}

pub fn gh_soot(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    // warm plaster wall, a window of light and a wooden floor
    let mut col = rgb8(236, 214, 170).mix(rgb8(214, 176, 128), smoothstep(0.6, -0.6, q.y));
    col = col * (0.95 + 0.08 * noise(q * 14.0));
    let win = sd_round_box(q - vec2(0.55, 0.3), vec2(0.32, 0.38), 0.03);
    col = col.mix(rgb8(255, 246, 200), fill_aa(win, 0.01));
    let mullion = sd_box(q - vec2(0.55, 0.3), vec2(0.008, 0.38))
        .min(sd_box(q - vec2(0.55, 0.3), vec2(0.32, 0.008)));
    col = col.mix(
        rgb8(150, 108, 70),
        fill_aa(mullion, 0.004) * fill_aa(win, 0.01),
    );
    let floor_y = -0.55;
    let boards = (q.x * 3.0 + (q.y * 0.0)).sin().abs();
    let wood = rgb8(150, 100, 64).mix(
        rgb8(120, 78, 52),
        smoothstep(0.9, 1.0, boards) + noise(vec2(q.x * 2.0, q.y * 20.0)) * 0.3,
    );
    col = col.mix(wood, step(q.y, floor_y));
    col = col
        + rgb8(255, 244, 190)
            * 0.18
            * smoothstep(0.9, 0.0, (q - vec2(0.55, -0.7)).len())
            * step(q.y, floor_y);
    // sprites
    for i in 0..9 {
        let fi = i as f32;
        let sp = 0.7 + 0.5 * hash1(fi * 5.1);
        let ph = t * sp + fi * 1.9;
        let bx = (hash1(fi * 2.3) * 2.0 - 1.0) * (c.aspect() * 0.9) + 0.1 * (ph * 0.5).sin();
        let hop = (ph % 3.14159).sin().abs();
        let r = 0.07 + 0.04 * hash1(fi * 9.9);
        let by = floor_y + r + 0.25 * hop * hop.sqrt() * 0.8 + 0.02 * (fi % 3.0)
            - 0.03 * (fi % 2.0) * 3.0;
        let squash = 1.0 - 0.18 * smoothstep(0.15, 0.0, hop);
        let p = (q - vec2(bx, by)) / r;
        let p2 = vec2(p.x / (2.0 - squash), p.y / squash);
        let ang = p2.y.atan2(p2.x);
        let fur = 1.0
            + 0.28
                * (ang * 13.0 + fi * 3.0 + (t * 3.0).sin() * 0.3)
                    .sin()
                    .abs()
                    .powf(3.0)
            + 0.1 * noise(vec2(ang * 3.0, fi));
        let d = p2.len() - fur;
        let body = smoothstep(0.1, -0.1, d);
        // eyes
        let blink = 1.0 - 0.9 * smoothstep(0.96, 1.0, (t * 0.5 + fi * 0.31) % 1.0);
        let e1 = vec2((p2.x - 0.32) / 0.2, (p2.y - 0.1) / (0.26 * blink));
        let e2 = vec2((p2.x + 0.32) / 0.2, (p2.y - 0.1) / (0.26 * blink));
        let eyes = smoothstep(1.0, 0.85, e1.len().min(e2.len()));
        let pup = smoothstep(
            0.55,
            0.4,
            vec2(e1.x * 1.6 - 0.1, e1.y * 1.6)
                .len()
                .min(vec2(e2.x * 1.6 - 0.1, e2.y * 1.6).len()),
        );
        let shade = rgb8(30, 24, 34).mix(rgb8(54, 44, 58), sat(0.5 - p2.y * 0.3));
        col = col.mix(
            col * 0.55,
            smoothstep(0.35, -0.1, d) * 0.0
                + smoothstep(0.4, 0.0, (vec2(p.x, p.y + 1.1)).len() - 0.9) * 0.25 * step(0.0, 1.0),
        );
        col = col.mix(shade, body);
        col = col.mix(Vec3::WHITE, eyes * body);
        col = col.mix(INK, pup * body * eyes);
    }
    col
}

pub fn gh_night_train(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let hz = -0.05;
    let mut col = rgb8(14, 22, 58).mix(rgb8(58, 86, 140), smoothstep(0.95, hz, q.y));
    // stars
    let g = (q * 60.0).floor();
    let hs = hash2(g.x as i32, g.y as i32);
    col = col
        + Vec3::WHITE
            * step(0.985, hs)
            * (0.5 + 0.5 * (t * 2.0 + hs * 90.0).sin())
            * smoothstep(0.0, 0.4, q.y);
    // moon
    let moon = vec2(-0.7, 0.55);
    let md = (q - moon).len();
    col = col
        + rgb8(255, 248, 214) * smoothstep(0.17, 0.15, md)
        + rgb8(200, 220, 255) * (-md * 4.0).exp() * 0.25;
    let ph = if q.y > hz { 1.0 } else { -1.0 };
    // mirror for the water so the scene above is reflected below
    let sy = if q.y > hz { q.y } else { hz + (hz - q.y) * 1.0 };
    let wob = if q.y < hz {
        (noise(vec2(q.x * 16.0 + t * 0.5, q.y * 90.0)) - 0.5) * 0.02 * (hz - q.y) * 6.0
    } else {
        0.0
    };
    let x = q.x + wob;
    // rails on a causeway slightly above the water
    let rail_y = 0.06;
    let train_h = 0.2;
    let tx = x * 1.0 - t * 0.35;
    let carlen = 0.62;
    let cell = (tx / carlen).floor();
    let fc = tx / carlen - cell;
    let body = step(rail_y, sy) * step(sy, rail_y + train_h) * step(0.06, fc) * step(fc, 0.94);
    let win = step(rail_y + 0.08, sy)
        * step(sy, rail_y + 0.17)
        * step(0.12, fc % 0.2 * 5.0 * 0.2 + 0.1)
        * step(0.1, fc)
        * step(fc, 0.9)
        * step(0.15, (fc * 5.0) % 1.0)
        * step((fc * 5.0) % 1.0, 0.85);
    let train_col = rgb8(24, 34, 48).mix(rgb8(255, 214, 120), win);
    let mut scene = col;
    scene = scene.mix(train_col, body);
    // bridge posts
    scene = scene.mix(rgb8(18, 24, 40), step(rail_y - 0.03, sy) * step(sy, rail_y));
    if ph < 0.0 {
        let d = hz - q.y;
        let wcol = rgb8(10, 20, 52).mix(scene, 0.55 * (-d * 2.0).exp() + 0.1);
        let ripple = smoothstep(
            0.82,
            0.95,
            noise(vec2(q.x * 10.0 + t * 0.2, d * 80.0 + t * 0.3)),
        );
        col = wcol + rgb8(180, 210, 255) * ripple * 0.18 * (1.0 - d);
    } else {
        col = scene;
    }
    // steam
    let st = fbm(vec2(x * 2.0 - t * 0.3, sy * 4.0 - t * 0.1), 4);
    col = col
        + rgb8(220, 230, 255)
            * smoothstep(0.55, 0.75, st)
            * 0.12
            * step(0.0, ph)
            * smoothstep(rail_y + train_h, rail_y + 0.4, sy);
    col
}

// ---------------------------------------------------------------- 3D kit --

fn nrm(v: Vec3) -> Vec3 {
    v * (1.0 / v.len().max(1e-6))
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    vec3(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}
fn len2(x: f32, y: f32) -> f32 {
    (x * x + y * y).sqrt()
}
fn sd_cyl_y(p: Vec3, r: f32, y0: f32, y1: f32) -> f32 {
    let a = len2(p.x, p.z) - r;
    let b = (p.y - 0.5 * (y0 + y1)).abs() - 0.5 * (y1 - y0);
    a.max(b).min(0.0) + len2(a.max(0.0), b.max(0.0))
}
fn sd_capsule(p: Vec3, a: Vec3, b: Vec3, r: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = sat(pa.dot(ba) / ba.dot(ba));
    (pa - ba * h).len() - r
}
fn sphere_hit(ro: Vec3, rd: Vec3, c: Vec3, r: f32) -> f32 {
    let oc = ro - c;
    let b = oc.dot(rd);
    let h = b * b - oc.dot(oc) + r * r;
    step(0.0, h) * step(0.0, -b + h.max(0.0).sqrt())
}
fn band(x: f32) -> f32 {
    // three-step cel lighting
    0.62 + 0.2 * step(0.0, x) + 0.2 * step(0.35, x)
}

/// Camera ray for `uv` looking from `ro` at `ta`.
fn cam_ray(ro: Vec3, ta: Vec3, q: Vec2, fl: f32) -> Vec3 {
    let f = nrm(ta - ro);
    let r = nrm(vec3(-f.z, 0.0, f.x));
    let u = cross(r, f);
    nrm(r * q.x + u * q.y + f * fl)
}

/// Sky seen along `rd`: gradient, sun and cel-painted cumulus.
fn sky3d(rd: Vec3, t: f32, top: Vec3, mid: Vec3, hz: Vec3, sun: Vec3) -> Vec3 {
    let y = rd.y.max(0.0);
    let mut col = hz.mix(mid, smoothstep(0.0, 0.25, y));
    col = col.mix(top, smoothstep(0.15, 0.7, y));
    let az = rd.x.atan2(rd.z);
    let p = vec2(az * 1.6, rd.y * 3.0 + 0.2);
    let (cv, lt) = cumulus(p, t, 5.0, 0.12, 1.0);
    col = cloud_col(cv * smoothstep(-0.05, 0.1, rd.y), lt, col, CLOUD_SHADE);
    let sd = rd.dot(sun);
    col + rgb8(255, 232, 170)
        * (0.35 * (sd - 0.9).max(0.0) * 10.0 + 0.7 * smoothstep(0.9985, 0.9995, sd))
}

const PX_ANGLE: f32 = 1.0; // multiplied by 1/(h*fl) at call sites

/// Sphere trace. Returns `(t, id, edge)`; `id < 0` means a miss. Ids below 9.5
/// are inked when the ray skims their outline.
fn march3(ro: Vec3, rd: Vec3, scene: u32, tmax: f32, w: f32, time: f32) -> Vec3 {
    let mut t = 0.4;
    let mut id = -1.0;
    let mut prev = 1e9;
    let mut edge: f32 = 0.0;
    for _ in 0..110 {
        let p = ro + rd * t;
        let h = map3(p, scene, time);
        if h.x < 0.0006 * t {
            id = h.y;
            break;
        }
        let ink = h.y < 9.5;
        if ink && h.x > prev && prev < w * t {
            edge = edge.max(sat((w * t - prev) / (w * t * 0.55)));
        }
        prev = if ink { h.x } else { 1e9 };
        t += h.x * 0.9;
        if t > tmax {
            break;
        }
    }
    vec3(t, id, edge)
}

fn normal3(p: Vec3, scene: u32, time: f32) -> Vec3 {
    let e = 0.0012;
    let a = vec3(1.0, -1.0, -1.0);
    let b = vec3(-1.0, -1.0, 1.0);
    let c = vec3(-1.0, 1.0, -1.0);
    let d = vec3(1.0, 1.0, 1.0);
    nrm(a * map3(p + a * e, scene, time).x
        + b * map3(p + b * e, scene, time).x
        + c * map3(p + c * e, scene, time).x
        + d * map3(p + d * e, scene, time).x)
}

fn lump(p: Vec3, f: f32, a: f32) -> f32 {
    a * (p.x * f).sin() * (p.y * f * 1.3).sin() * (p.z * f).sin()
}

// ---- scene 0: floating island

fn map_island(p0: Vec3, time: f32) -> Vec2 {
    let p = p0 - vec3(0.0, 0.08 * (time * 0.7).sin(), 0.0);
    let r = len2(p.x, p.z);
    // grass top: rounded disc
    let a = r - 1.44 + 0.07 * (p.x * 3.0).sin() * (p.z * 2.5).sin();
    let b = p.y.abs() - 0.04;
    let top = a.max(b).min(0.0) + len2(a.max(0.0), b.max(0.0)) - 0.06;
    // rock cone beneath
    let cone = ((r - 1.45 * (p.y + 2.3) / 2.2) * 0.82).max(p.y - 0.0) + lump(p, 7.0, 0.05);
    let mut res = vec2(top, 1.0);
    if cone < res.x {
        res = vec2(cone, 0.0);
    }
    // castle tower
    let q = p - vec3(0.45, 0.0, 0.15);
    let wall = sd_cyl_y(q, 0.16, 0.05, 0.72);
    let ring = sd_cyl_y(q, 0.2, 0.7, 0.76);
    let roof_r = 0.2 * (0.98 - q.y) / 0.24;
    let roof = ((len2(q.x, q.z) - roof_r) * 0.84)
        .max(q.y - 0.98)
        .max(0.74 - q.y);
    let side = sd_cyl_y(q - vec3(-0.28, 0.0, 0.12), 0.1, 0.05, 0.45);
    let side_roof = ((len2(q.x + 0.28, q.z - 0.12) - 0.12 * (0.65 - q.y) / 0.2) * 0.84)
        .max(q.y - 0.65)
        .max(0.43 - q.y);
    let stone = wall.min(ring).min(side);
    if stone < res.x {
        res = vec2(stone, 2.0);
    }
    let roofs = roof.min(side_roof);
    if roofs < res.x {
        res = vec2(roofs, 3.0);
    }
    // giant tree
    let tq = p - vec3(-0.7, 0.0, -0.2);
    let trunk = sd_capsule(tq, vec3(0.0, 0.0, 0.0), vec3(0.05, 0.55, 0.0), 0.07 - 0.0);
    if trunk < res.x {
        res = vec2(trunk, 4.0);
    }
    let leaf1 = (tq - vec3(0.05, 0.85, 0.0)).len() - 0.42;
    let leaf2 = (tq - vec3(-0.25, 0.68, 0.1)).len() - 0.28;
    let leaf3 = (tq - vec3(0.3, 0.7, -0.1)).len() - 0.27;
    let leaves = (leaf1.min(leaf2).min(leaf3) + lump(tq, 11.0, 0.03)) * 0.9;
    if leaves < res.x {
        res = vec2(leaves, 5.0);
    }
    // little rocks drifting around the island
    for i in 0..3 {
        let fi = i as f32;
        let a = time * (0.18 + 0.05 * fi) + fi * 2.1;
        let rp = vec3(
            a.cos() * (2.0 + 0.2 * fi),
            -0.4 + 0.5 * fi + 0.06 * (time + fi).sin(),
            a.sin() * (2.0 + 0.2 * fi),
        );
        let rk = ((p - rp).len() - 0.14 - 0.04 * fi) + lump(p, 9.0, 0.02);
        if rk < res.x {
            res = vec2(rk, 0.0);
        }
    }
    res
}

fn shade_island(p0: Vec3, n: Vec3, id: f32, time: f32, l: Vec3) -> Vec3 {
    let p = p0 - vec3(0.0, 0.08 * (time * 0.7).sin(), 0.0);
    let lam = n.dot(l);
    let k = band(lam);
    let mut base;
    if id < 0.5 {
        // striated rock, mossy near the top
        let s = (p.y * 9.0 + noise(vec2(p.x * 2.0 + p.z, p.y * 3.0)) * 2.0).sin();
        base = rgb8(150, 130, 116).mix(rgb8(104, 94, 96), step(0.3, s) * 0.8);
        base = base.mix(rgb8(86, 140, 70), smoothstep(-0.55, -0.1, p.y) * 0.7);
    } else if id < 1.5 {
        let g = noise(vec2(p.x * 5.0, p.z * 5.0));
        base = rgb8(126, 190, 72).mix(rgb8(170, 214, 90), step(0.55, g));
        // flowers
        let c = vec2(p.x * 9.0, p.z * 9.0);
        let h = hash2(c.x.floor() as i32, c.y.floor() as i32);
        base = base.mix(
            rgb8(255, 236, 190),
            step(0.97, h) * step(len2(c.x.fract() - 0.5, c.y.fract() - 0.5), 0.25),
        );
    } else if id < 2.5 {
        let course = step(0.06, (p.y * 12.0).fract());
        base = rgb8(236, 230, 214).mix(rgb8(214, 206, 192), 1.0 - course);
    } else if id < 3.5 {
        base = rgb8(52, 150, 150);
    } else if id < 4.5 {
        base = rgb8(110, 80, 60);
    } else {
        base = rgb8(44, 140, 78).mix(
            rgb8(120, 196, 90),
            step(
                0.62,
                n.dot(l) + 0.15 * (p.x * 12.0).sin() * (p.y * 12.0).sin(),
            ),
        );
    }
    let shade = rgb8(120, 130, 180);
    let lit = Vec3::WHITE;
    base * shade.mix(lit, sat(k * 1.6 - 0.6)) * (0.9 + 0.1 * k)
}

pub fn gh_laputa(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let az = 0.7 + t * 0.12;
    let ro = vec3(az.sin() * 4.3, 0.45 + 0.1 * (t * 0.3).sin(), az.cos() * 4.3);
    let fl = 2.6;
    let rd = cam_ray(ro, vec3(0.0, -0.3, 0.0), q, fl);
    let l = nrm(vec3(-0.5, 0.75, 0.45));
    let w = 1.7 * PX_ANGLE / (c.h as f32 * 0.5 * fl);
    let mut col = sky3d(rd, t, SKY_TOP, SKY_MID, SKY_HZ, l);
    // sea of clouds far below
    let r = march3(ro, rd, 0, 14.0, w, t);
    if r.y >= 0.0 {
        let p = ro + rd * r.x;
        let n = normal3(p, 0, t);
        col = shade_island(p, n, r.y, t, l);
    }
    // cloud floor below the island
    if r.y < 0.0 && rd.y < -0.02 {
        let tp = (-3.2 - ro.y) / rd.y;
        let p = ro + rd * tp;
        let n = fbm(vec2(p.x, p.z) * 0.35 + vec2(t * 0.04, 0.0), 5);
        let cc = CLOUD_SHADE.mix(CLOUD_LIT, smoothstep(0.45, 0.65, n));
        col = col.mix(cc, smoothstep(0.0, 0.25, -rd.y) * 0.95);
    }
    col.mix(INK, r.z * 0.9)
}

// ---- scene 1: round pod on an island

const GROUND: f32 = -1.04;

fn map_pod(p: Vec3, _time: f32) -> Vec2 {
    let mut res = vec2(p.len() - 1.0, 6.0);
    // landing legs and feet
    for i in 0..3 {
        let a = 0.78 + i as f32 * 2.094;
        let (s, co) = (a.sin(), a.cos());
        let top = vec3(s * 0.78, -0.62, co * 0.78);
        let foot = vec3(s * 1.22, GROUND + 0.04, co * 1.22);
        let strut = sd_capsule(p, top, foot, 0.04);
        let fp = p - vec3(s * 1.22, 0.0, co * 1.22);
        let pad = sd_cyl_y(fp, 0.11, GROUND, GROUND + 0.05) - 0.01;
        let l = strut.min(pad);
        if l < res.x {
            res = vec2(l, 7.0);
        }
    }
    // small belly cylinder
    let belly = sd_cyl_y(p, 0.14, -1.12, -0.85);
    if belly < res.x {
        res = vec2(belly, 7.0);
    }
    // trees
    for i in 0..2 {
        let (tx, tz, th, tr): (f32, f32, f32, f32) = if i == 0 {
            (1.9, 0.9, 0.7, 0.16)
        } else {
            (-3.4, -2.6, 1.7, 0.42)
        };
        let q = p - vec3(tx, GROUND, tz);
        let trunk = sd_capsule(
            q,
            vec3(0.0, -0.1, 0.0),
            vec3(0.0, th, 0.0),
            0.02 + 0.01 * th,
        );
        if trunk < res.x {
            res = vec2(trunk, 8.0);
        }
        let lv = ((q - vec3(0.0, th, 0.0)).len() - tr + lump(q, 9.0, 0.03)) * 0.85;
        if lv < res.x {
            res = vec2(lv, 9.0);
        }
    }
    res
}

fn map3(p: Vec3, scene: u32, time: f32) -> Vec2 {
    match scene {
        0 => map_island(p, time),
        _ => map_pod(p, time),
    }
}

fn arch(p: Vec2, hw: f32, y0: f32, y1: f32) -> f32 {
    let yy = p.y - p.y.clamp(y0 - hw, y1 - hw);
    let d = len2(p.x, yy) - hw;
    d.max(y0 - p.y)
}

fn shade_pod(p: Vec3, n: Vec3, id: f32, l: Vec3, w_px: f32) -> Vec3 {
    let lam = n.dot(l);
    let sh = rgb8(150, 176, 214);
    let col: Vec3;
    if id < 6.5 {
        // hull
        let lon = p.x.atan2(p.z);
        let rxz = len2(p.x, p.z);
        let belt = step(-0.52, p.y) * step(p.y, 0.08);
        let cap = step(0.76, p.y);
        let dark = belt.max(cap);
        let white = rgb8(236, 242, 232);
        let black = rgb8(18, 28, 36);
        let hi = rgb8(120, 172, 172);
        let a = (lon - 0.0 / 1.0) * rxz;
        let _ = a;
        // porthole windows around the belt, none where the door is
        let seg = 6.2831855 / 12.0;
        let k = ((lon / seg) + 0.5).floor();
        let lc = lon - k * seg;
        let win = (len2(lc * rxz * 1.0, p.y + 0.22) - 0.1) * 1.0;
        let on_win = step(0.5, k.abs()) * smoothstep(0.012, -0.012, win);
        let glass = rgb8(14, 86, 128).mix(
            rgb8(92, 178, 204),
            smoothstep(
                0.0,
                0.1,
                (vec2(lc * rxz + 0.035, p.y + 0.22 - 0.04)).len() - 0.04,
            ),
        );
        // door arch outline
        let da = arch(vec2(lon * rxz, p.y), 0.17, -0.6, 0.21);
        let door_ink =
            smoothstep(0.012 + w_px, 0.004, da.abs()) * step(0.0, p.z) * step(lon.abs(), 0.4);
        let mut paint = white.mix(black, dark);
        let sheen = noise(vec2(p.x * 6.0 + p.y, p.y * 6.0)) * 0.0
            + smoothstep(
                0.35,
                0.5,
                n.dot(nrm(vec3(-0.5, 0.6, 0.6))) * 1.0 - 0.0 + 0.0,
            );
        paint = paint.mix(hi, dark * sheen * 0.9);
        paint = paint.mix(glass, on_win);
        paint = paint.mix(INK, door_ink);
        col = paint * sh.mix(Vec3::WHITE, sat(band(lam) * 1.6 - 0.6));
    } else if id < 7.5 {
        col = rgb8(190, 196, 204) * sh.mix(Vec3::WHITE, sat(band(lam) * 1.6 - 0.6));
    } else if id < 8.5 {
        col = rgb8(226, 228, 200) * sh.mix(Vec3::WHITE, sat(band(lam) * 1.6 - 0.6));
    } else {
        let k = n.dot(l) + 0.12 * (p.x * 14.0).sin() * (p.y * 14.0).sin();
        let base = rgb8(32, 112, 150).mix(rgb8(90, 186, 196), step(0.55, k));
        col = base * rgb8(190, 200, 230).mix(Vec3::WHITE, step(0.0, k));
    }
    col
}

fn pod_ground(p: Vec3, ro: Vec3, l: Vec3, time: f32) -> Vec3 {
    let r = len2(p.x, p.z);
    let warp = noise(vec2(p.x, p.z) * 0.22) - 0.5;
    let coast = r - 9.5 - 5.0 * warp;
    let mut col = rgb8(56, 150, 150);
    col = col.mix(
        rgb8(36, 118, 128),
        smoothstep(0.52, 0.56, fbm(vec2(p.x, p.z) * 0.35 + vec2(3.0, 1.0), 4)),
    );
    // sandy patch with ink dashes
    let dirt = fbm(vec2(p.x, p.z) * 0.25 + vec2(-13.0, 31.0), 4) - 0.6;
    col = col.mix(rgb8(220, 204, 156), smoothstep(-0.01, 0.01, dirt));
    col = col.mix(INK, smoothstep(0.012, 0.0, dirt.abs()) * 0.8);
    // grass ticks
    let g = vec2(p.x * 2.5, p.z * 2.5);
    let gc = g.floor();
    let gh = hash2(gc.x as i32, gc.y as i32);
    let gf = g.fract();
    col = col.mix(
        INK,
        step(0.6, gh)
            * step(0.0, -dirt)
            * smoothstep(0.1, 0.06, (gf.x - 0.5).abs())
            * step(gf.y, 0.28)
            * step(coast, -0.8)
            * 0.7,
    );
    // beach and sea
    col = col.mix(rgb8(204, 128, 98), smoothstep(-0.75, -0.7, coast));
    col = col.mix(rgb8(222, 206, 160), smoothstep(-0.6, -0.55, coast));
    let lap = 0.5 + 0.5 * (1.3 * time - 6.0 * noise(vec2(p.x, p.z) * 0.4)).sin();
    let shore = -0.35 * lap;
    let wave = noise(vec2(
        3.0 * r + 3.0 * noise(vec2(p.x, p.z) * 0.25) + 1.2 * time,
        0.5,
    )) * noise(vec2(p.x, p.z) * 0.6 + vec2(2.0, 2.0));
    let mut sea = rgb8(111, 169, 126).mix(rgb8(140, 196, 158), 1.0 - smoothstep(0.9, 1.0, coast));
    let foam = (1.0 - smoothstep(shore + 0.15, shore + 0.2, coast))
        .max(smoothstep(0.5, 0.53, wave) * (1.0 - smoothstep(20.0, 45.0, (p - ro).len())));
    sea = sea.mix(rgb8(204, 232, 204), foam);
    col = col.mix(sea, smoothstep(shore - 0.05, shore, coast));
    // cast shadows from the ship and the trees
    let o = p + vec3(0.0, 0.01, 0.0);
    let sh = sphere_hit(o, l, vec3(0.0, 0.0, 0.0), 1.0)
        .max(sphere_hit(o, l, vec3(1.9, GROUND + 0.7, 0.9), 0.17))
        .max(sphere_hit(o, l, vec3(-3.4, GROUND + 1.7, -2.6), 0.42));
    col = col * rgb8(176, 198, 224).mix(Vec3::WHITE, 1.0 - sh);
    // distance haze
    let d = (p - ro).len();
    col.mix(rgb8(158, 204, 190), 1.0 - (-0.012 * d).exp())
}

pub fn gh_pod(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let az = 0.42 + t * 0.105;
    let el = 0.18 + 0.03 * (t * 0.21).sin();
    let ro = vec3(
        az.sin() * el.cos() * 5.6,
        -0.25 + el.sin() * 5.6,
        az.cos() * el.cos() * 5.6,
    );
    let fl = 3.0;
    let rd = cam_ray(ro, vec3(0.0, -0.25, 0.0), q, fl);
    let l = nrm(vec3(-0.59, 0.36, 0.73));
    let w = 1.8 * PX_ANGLE / (c.h as f32 * 0.5 * fl);
    let tp: f32 = if rd.y < 0.0 {
        (GROUND - ro.y) / rd.y
    } else {
        1e4
    };
    let r = march3(ro, rd, 1, tp.min(40.0), w, t);
    let mut col = sky3d(
        rd,
        t,
        rgb8(76, 170, 94),
        rgb8(148, 199, 128),
        rgb8(189, 222, 178),
        l,
    );
    let mut t_front: f32 = 1e9;
    if r.y >= 0.0 {
        let p = ro + rd * r.x;
        let n = normal3(p, 1, t);
        col = shade_pod(p, n, r.y, l, w * r.x);
        // hull shadow on the trees and legs
        let o = p + n * 0.03;
        let sh = sphere_hit(o, l, vec3(0.0, 0.0, 0.0), 1.0) * step(6.5, r.y);
        col = col * rgb8(170, 196, 224).mix(Vec3::WHITE, 1.0 - sh);
        t_front = r.x;
    } else if tp < 1e3 {
        col = pod_ground(ro + rd * tp, ro, l, t);
    }
    let _ = t_front;
    let col = col.mix(INK, r.z * 0.9);
    // mild retro grade and grain
    let q2 = uv - 0.5;
    let vig = 1.0 - 0.45 * (q2.x * q2.x + q2.y * q2.y);
    (col * vig * vec3(0.96, 1.0, 0.97) + 0.02).clamp01()
}

// ---------------------------------------------------------------- post --

pub fn gh_painted(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let px = c.texel();
    let l = |dx: f32, dy: f32| src.sample(uv + vec2(dx * px.x, dy * px.y)).luma();
    let gx = -l(-1.0, -1.0) - 2.0 * l(-1.0, 0.0) - l(-1.0, 1.0)
        + l(1.0, -1.0)
        + 2.0 * l(1.0, 0.0)
        + l(1.0, 1.0);
    let gy = -l(-1.0, -1.0) - 2.0 * l(0.0, -1.0) - l(1.0, -1.0)
        + l(-1.0, 1.0)
        + 2.0 * l(0.0, 1.0)
        + l(1.0, 1.0);
    let e = smoothstep(0.35, 0.8, (gx * gx + gy * gy).sqrt());
    // paper
    let g = noise(vec2(uv.x * c.w as f32 * 0.5, uv.y * c.h as f32 * 0.5));
    let fibre = noise(vec2(uv.x * 40.0, uv.y * 3.0)) - 0.5;
    let paper = 0.96 + 0.06 * (g - 0.5) + 0.02 * fibre;
    // soften and quantise toward gouache colour
    let sat_ = Vec3::splat(s.luma()).mix(s, 1.12);
    let soft = (sat_ * 5.0 + 0.5).clamp01();
    let col = (sat_ * 0.98 + soft * 0.0).mix(INK, e * 0.55) * paper;
    s.mix(col, c.intensity)
}

pub fn gh_dream(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let px = c.texel();
    let mut glow = Vec3::BLACK;
    for i in 0..12 {
        let a = i as f32 * 0.5236;
        let o = vec2(a.cos() * px.x, a.sin() * px.y);
        let t1 = src.sample(uv + o * 7.0);
        let t2 = src.sample(uv + o * 18.0);
        glow = glow + t1 * (t1.luma() - 0.75).max(0.0) + t2 * (t2.luma() - 0.7).max(0.0) * 0.6;
    }
    let lifted = s + rgb8(18, 22, 36) * (1.0 - s.luma()) * 0.5;
    let out = lifted + (glow / 12.0) * rgb8(255, 236, 200) * 0.9;
    s.mix(out, c.intensity)
}
