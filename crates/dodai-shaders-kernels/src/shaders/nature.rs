//! Cosmic, water, fire and abstract shaders.

use crate::frame::Ctx;
use crate::math::*;
#[allow(unused_imports)]
use crate::prelude::*;
use crate::{Family, ShaderInfo};

const F: Family = Family::Nature;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::generator(
        "starfield",
        "Starfield warp",
        F,
        "Stars streaming past in a hyperspace tunnel",
        starfield,
    ),
    ShaderInfo::generator(
        "galaxy",
        "Spiral galaxy",
        F,
        "Rotating arms of dust and stars",
        galaxy,
    ),
    ShaderInfo::generator(
        "fireworks",
        "Fireworks",
        F,
        "Rockets bursting into fading sparks",
        fireworks,
    ),
    ShaderInfo::generator(
        "plasma",
        "Plasma",
        F,
        "Classic sine-sum demoscene plasma",
        plasma,
    ),
    ShaderInfo::generator(
        "caustics",
        "Water caustics",
        F,
        "Rippling light networks on a pool floor",
        caustics,
    ),
    ShaderInfo::generator(
        "ocean",
        "Ocean waves",
        F,
        "Layered waves under a moon path",
        ocean,
    ),
    ShaderInfo::generator(
        "aurora",
        "Aurora",
        F,
        "Curtains of light over a night sky",
        aurora,
    ),
    ShaderInfo::generator("snow", "Snowfall", F, "Parallax flakes drifting down", snow),
    ShaderInfo::generator(
        "lava_lamp",
        "Lava lamp",
        F,
        "Metaballs rising and merging",
        lava_lamp,
    ),
    ShaderInfo::generator(
        "embers",
        "Fire & embers",
        F,
        "Flame body with sparks floating up",
        embers,
    ),
    ShaderInfo::generator(
        "interference",
        "Sine interference",
        F,
        "Moiré rings from moving emitters",
        interference,
    ),
    ShaderInfo::generator(
        "gradient_flow",
        "Animated gradient",
        F,
        "Slowly flowing palette gradient",
        gradient_flow,
    ),
    ShaderInfo::generator(
        "spotlight",
        "Spotlight",
        F,
        "A roaming light cone over a dark floor",
        spotlight,
    ),
    ShaderInfo::generator("smoke", "Smoke", F, "Domain-warped noise plumes", smoke),
];

pub fn starfield(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let mut col = c.pal.bg;
    let layers = 6;
    for i in 0..layers {
        let depth = fract(i as f32 / layers as f32 + c.time * 0.12);
        let scale = mix(22.0, 0.6, depth);
        let fade = depth * smoothstep(1.0, 0.85, depth);
        let p = q * scale + vec2(i as f32 * 45.3, i as f32 * 17.7);
        let id = p.floor();
        let f = p.fract() - 0.5;
        let h = hash2(id.x as i32, id.y as i32);
        if h > 0.55 {
            let at = vec2(
                hash2(id.x as i32 + 9, id.y as i32) - 0.5,
                hash2(id.x as i32, id.y as i32 + 9) - 0.5,
            ) * 0.7;
            let d = (f - at).len();
            let tone = c.pal.fg.mix(c.pal.b, (h - 0.55) * 2.2);
            col = col + tone * smoothstep(0.1, 0.0, d) * fade * 1.4;
            col = col + c.pal.a * glow(d * 9.0, 1.0) * fade * 0.12 * h;
        }
    }
    col
}

pub fn galaxy(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv) * 1.3;
    let r = q.len();
    let a = q.y.atan2(q.x) + c.time * 0.12 / (r + 0.3);
    let arms = 0.5 + 0.5 * (a * 2.0 + r * 9.0).cos();
    let dust = fbm(q * 4.0 + vec2(c.time * 0.02, 0.0), 4);
    let dens = (arms.powf(2.0) * dust * (-r * 1.6).exp() * 2.4).min(1.5);
    let core = (-r * r * 14.0).exp();
    let mut col =
        c.pal.bg + c.pal.b * dens * 0.9 + c.pal.a * dens * dens * 0.8 + c.pal.fg * core * 1.4;
    let s = hash2((uv.x * c.w as f32) as i32, (uv.y * c.h as f32) as i32);
    if s > 0.995 {
        col = col + c.pal.fg * (0.3 + 0.7 * (c.time * 2.0 + s * 90.0).sin().abs());
    }
    col
}

pub fn fireworks(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let mut col = c.pal.bg * (1.0 - 0.3 * uv.y);
    for k in 0..4 {
        let kf = k as f32;
        let period = 2.8;
        let t0 = c.time + kf * 0.7;
        let n = (t0 / period).floor();
        let t = (t0 / period).fract() * period;
        let seed = n * 7.0 + kf * 13.0;
        let ctr = vec2(
            (hash1(seed) - 0.5) * 1.5 * c.aspect(),
            hash1(seed + 1.0) * 0.8 - 0.1,
        );
        let tone = hsv(hash1(seed + 2.0), 0.7, 1.0).mix(c.pal.a, 0.35);
        if t < 0.7 {
            // rocket
            let y = -1.0 + (ctr.y + 1.0) * (t / 0.7);
            col = col + c.pal.fg * glow((q - vec2(ctr.x, y)).len() * 90.0, 1.0) * 0.8;
        } else {
            let bt = t - 0.7;
            // Sparks stay within this radius: skip far pixels entirely.
            let reach = 0.9 * 0.7 * bt.sqrt() + 0.18 * bt * bt + 0.12;
            if (q - ctr).len() > reach + 0.1 {
                continue;
            }
            let sparks = 36;
            for i in 0..sparks {
                let a = hash1(seed + i as f32 * 1.37) * std::f32::consts::TAU;
                let sp = 0.35 + hash1(seed + i as f32 * 5.1) * 0.35;
                let pos =
                    ctr + vec2(a.cos(), a.sin()) * sp * bt.sqrt() * 0.9 - vec2(0.0, 0.18 * bt * bt);
                let life = (1.0 - bt / 1.9).max(0.0);
                col = col + tone * glow((q - pos).len() * 85.0, 1.0) * life * life * 0.9;
            }
        }
    }
    col
}

pub fn plasma(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv) * 3.0;
    let t = c.time;
    let v = (q.x + t).sin()
        + ((q.y * 1.3 + t * 0.7).sin())
        + ((q.x + q.y + t).sin())
        + (((q.x * q.x + q.y * q.y).sqrt() * 1.5 - t * 1.1).sin());
    let k = v * 0.25 + 0.5;
    palette(
        k,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(1.0, 1.0, 1.0),
        vec3(0.0, 0.33, 0.67),
    )
    .mix(c.pal.a.mix(c.pal.c, k), 0.55)
}

fn caustic(p: Vec2, t: f32) -> f32 {
    let mut i = p;
    let mut cc = 1.0;
    let inten = 0.005;
    for n in 0..5 {
        let tt = t * (1.0 - 3.5 / (n + 1) as f32);
        i = p + vec2(
            (tt - i.x).cos() + (tt + i.y).sin(),
            (tt - i.y).sin() + (tt + i.x).cos(),
        );
        cc += 1.0
            / vec2(
                p.x / ((i.x + tt).sin() / inten),
                p.y / ((i.y + tt).cos() / inten),
            )
            .len();
    }
    cc /= 5.0;
    1.17 - cc.powf(1.4)
}

pub fn caustics(uv: Vec2, c: &Ctx) -> Vec3 {
    let tau = std::f32::consts::TAU;
    let p = vec2(
        fmod(uv.x * c.aspect() * tau * 2.0, tau),
        fmod(uv.y * tau * 2.0, tau),
    ) - 250.0;
    let v = sat(caustic(p, c.time * 0.5 + 23.0).abs().powf(8.0) * 3.0);
    // lines are where light bunches up: dark net on a lit pool floor
    let deep = c.pal.bg.mix(c.pal.b * 0.35, 0.6);
    deep.mix(c.pal.b * 0.9 + c.pal.fg * 0.15, v)
}

pub fn ocean(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let horizon = 0.42;
    if uv.y < horizon {
        let t = uv.y / horizon;
        let mut col = p.bg.mix(p.a * 0.5, t * t);
        let m = (uv - vec2(0.7, 0.2)).len();
        col = col + p.fg * smoothstep(0.07, 0.06, m) + p.fg * glow(m * 8.0, 1.0) * 0.15;
        return col;
    }
    let y = (uv.y - horizon) / (1.0 - horizon);
    let depth = 1.0 / (y + 0.05);
    let x = (uv.x - 0.5) * c.aspect() * depth;
    let wave = (x * 1.6 + c.time * 0.8 + (depth * 3.0 + c.time).sin() * 0.5).sin() * 0.5
        + (x * 3.7 - c.time * 1.2 + depth * 5.0).sin() * 0.25;
    let base = p.bg.mix(p.b * 0.6, 0.3 + 0.35 * y);
    let crest = smoothstep(
        0.55,
        0.85,
        wave + 0.25 * noise(vec2(x * 2.0, depth * 3.0 - c.time)),
    );
    // moon path: bright band under the moon
    let path = (-((uv.x - 0.7) * 9.0 / (0.3 + y)).powi(2)).exp() * (0.5 + 0.5 * wave);
    base + p.fg * crest * 0.25 / depth.max(1.0).sqrt() + p.fg * path * 0.5 * (1.0 - y * 0.4)
}

pub fn aurora(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let mut col = p.bg.mix(p.a * 0.12, 1.0 - uv.y);
    let s = hash2((uv.x * c.w as f32) as i32, (uv.y * c.h as f32) as i32);
    if s > 0.996 && uv.y < 0.7 {
        col = col + p.fg * (0.3 + 0.5 * (c.time * 2.0 + s * 50.0).sin().abs());
    }
    for i in 0..3 {
        let f = i as f32;
        let sway = fbm(vec2(uv.x * 2.5 + f * 4.0, c.time * 0.12 + f), 4);
        let base = 0.32 + 0.16 * f + (sway - 0.5) * 0.35;
        let d = uv.y - base;
        let curtain = smoothstep(0.0, 0.02, d)
            * (-d * 5.0).exp()
            * (0.6 + 0.8 * noise(vec2(uv.x * 28.0 + f * 9.0, c.time * 0.3)));
        let tone = c
            .pal
            .b
            .mix(c.pal.a, f / 2.0)
            .mix(c.pal.c, smoothstep(0.1, 0.4, d) * 0.6);
        col = col + tone * curtain * 0.65;
    }
    col
}

pub fn snow(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let mut col = p.bg.mix(p.b * 0.25, uv.y);
    for layer in 0..4 {
        let l = layer as f32;
        let scale = 14.0 + l * 12.0;
        let q = vec2(
            uv.x * scale * c.aspect() + (c.time * 0.6 + l).sin() * 0.7,
            uv.y * scale - c.time * (0.8 + l * 0.5),
        );
        let cell = q.floor();
        let f = q.fract() - 0.5;
        let h = hash2(cell.x as i32, cell.y as i32 + layer * 91);
        if h > 0.55 {
            let off = vec2(h * 0.4 - 0.2, (h * 7.0).fract() * 0.4 - 0.2);
            let r = 0.12 + 0.08 * (h * 13.0).fract();
            col = col + p.fg * smoothstep(r, r * 0.4, (f - off).len()) * (0.5 + 0.12 * l);
        }
    }
    col
}

pub fn lava_lamp(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let mut field = 0.0;
    for i in 0..6 {
        let f = i as f32;
        let ctr = vec2(
            (c.time * 0.23 + f * 1.9).sin() * 0.5 * c.aspect() * 0.5,
            ((c.time * (0.18 + 0.05 * f) + f * 2.7).sin()) * 0.7,
        );
        let r = 0.22 + 0.06 * (f * 1.3).sin();
        field += r * r / (q - ctr).dot(q - ctr).max(1e-4);
    }
    let edge = smoothstep(0.9, 1.1, field);
    let rim = glow((field - 1.0) * 3.0, 3.0) * 0.5;
    let bg = p.bg.mix(p.c * 0.35, uv.y * 0.8);
    bg.mix(p.a.mix(p.c, (field * 0.2).min(1.0)), edge) + p.fg * rim * 0.1
}

pub fn embers(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let t = c.time;
    // flame body
    let fy = (uv.y - 0.55).max(0.0) * 2.2;
    let turb = fbm(vec2(q.x * 3.0, q.y * 2.0 - t * 1.8), 5);
    let width = (1.0 - fy).max(0.0) * 0.55;
    let body = smoothstep(
        width,
        width * 0.3,
        q.x.abs() + (turb - 0.5) * 0.5 * (0.3 + fy),
    );
    let heat = body * (1.0 - fy * 0.9).max(0.0) * (0.5 + turb);
    let flame = (p.c * heat * 1.6).screen(p.a * heat * heat * 1.2);
    let mut col = p.bg + flame;
    // sparks
    for layer in 0..3 {
        let l = layer as f32;
        let scale = 18.0 + l * 10.0;
        let s = vec2(
            uv.x * scale * c.aspect(),
            (uv.y + t * (0.25 + 0.1 * l)) * scale,
        );
        let cell = s.floor();
        let h = hash2(cell.x as i32, cell.y as i32 + layer * 77);
        if h > 0.9 {
            let f = s.fract() - 0.5;
            let wob = (t * 3.0 + h * 20.0).sin() * 0.15;
            let d = (f - vec2(wob, 0.0)).len();
            let life = smoothstep(0.1, 0.5, uv.y) * (1.0 - smoothstep(0.7, 1.0, 1.0 - uv.y));
            col = col
                + p.a.mix(p.fg, 0.4)
                    * smoothstep(0.1, 0.0, d)
                    * life
                    * (0.6 + 0.4 * (t * 8.0 + h * 40.0).sin());
        }
    }
    col
}

pub fn interference(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let mut v = 0.0;
    for i in 0..4 {
        let f = i as f32;
        let e = vec2(
            (c.time * 0.4 + f * 1.6).sin() * 0.9,
            (c.time * 0.33 + f * 2.1).cos() * 0.6,
        );
        v += ((q - e).len() * 26.0 - c.time * 3.0).sin();
    }
    let k = v * 0.125 + 0.5;
    c.pal
        .bg
        .mix(c.pal.a, smoothstep(0.45, 0.55, k))
        .mix(c.pal.c, smoothstep(0.7, 0.8, k) * 0.7)
        + c.pal.b * 0.15 * k
}

pub fn gradient_flow(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let warp = fbm(q * 1.2 + vec2(c.time * 0.07, 0.0), 3);
    let t = (q.x * 0.35 + q.y * 0.5 + warp * 1.6 + c.time * 0.08).sin() * 0.5 + 0.5;
    let col = c
        .pal
        .a
        .mix(c.pal.b, t)
        .mix(c.pal.c, smoothstep(0.55, 1.0, (warp + t) * 0.6));
    c.pal.bg.mix(col, 0.8)
}

pub fn spotlight(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let ctr = vec2(
        (c.time * 0.5).sin() * 0.9 * c.aspect() * 0.6,
        (c.time * 0.37).cos() * 0.4,
    );
    let d = (q - ctr).len();
    let cone = smoothstep(0.7, 0.05, d);
    let floor = ((q.x * 4.0).floor() + (q.y * 4.0).floor()) as i32 % 2 == 0;
    let base = if floor { c.pal.bg * 1.5 } else { c.pal.bg };
    base * 0.35 + c.pal.fg * cone * cone * 0.9 + c.pal.a * glow(d * 4.0, 1.0) * 0.2
}

pub fn smoke(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv) * 1.4;
    let t = c.time * 0.15;
    let w1 = fbm(q + vec2(t, -t * 2.0), 4);
    let w2 = fbm(q * 1.7 + vec2(w1 * 2.0, w1 * 2.0 + t), 5);
    let dens = smoothstep(0.35, 0.8, w2) * (1.0 - uv.y * 0.5);
    c.pal.bg.mix(c.pal.fg * 0.7, dens * 0.8) + c.pal.a * dens * dens * 0.15
}
