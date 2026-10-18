//! DMT Hyperspace: sacred geometry, rotating wireframes, mandalas and
//! kaleidoscopic tunnels, a few in the spirit of visionary art (layered
//! auras, chakra glows, radiant eyes), plus effects that turn any picture
//! into a trip. Also holds the Mandelbrot zoom for the cosmic family.
//! Original code and palettes; only the mood is borrowed.
//!
//! GPU-friendly style: index loops, `match`, no arrays, no strings.

#![allow(clippy::too_many_arguments, clippy::let_and_return)]

use crate::frame::{Ctx, Frame};
use crate::math::*;
#[allow(unused_imports)]
use crate::prelude::*;
use crate::{Family, ShaderInfo};

const F: Family = Family::Dmt;
const TAU: f32 = std::f32::consts::TAU;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::generator(
        "mandelbrot_zoom",
        "Mandelbrot zoom",
        Family::Nature,
        "An endless dive into the seahorse valley, smooth-coloured, looping",
        mandelbrot_zoom,
    ),
    ShaderInfo::generator(
        "dmt_breakthrough",
        "Breakthrough",
        F,
        "A twelve-fold geometric tunnel opening onto a white core",
        dmt_breakthrough,
    ),
    ShaderInfo::generator(
        "dmt_machine_elves",
        "Machine elves",
        F,
        "Folded lattices of neon light, endlessly unfolding",
        dmt_machine_elves,
    ),
    ShaderInfo::generator(
        "dmt_flower_of_life",
        "Flower of life",
        F,
        "The overlapping-circle lattice, breathing and slowly turning",
        dmt_flower_of_life,
    ),
    ShaderInfo::generator(
        "dmt_metatron",
        "Metatron's cube",
        F,
        "Thirteen circles joined by every line, rotating in light",
        dmt_metatron,
    ),
    ShaderInfo::generator(
        "dmt_sri_yantra",
        "Yantra",
        F,
        "Interlocking triangles inside lotus petals, a gate and a bindu",
        dmt_sri_yantra,
    ),
    ShaderInfo::generator(
        "dmt_merkaba",
        "Merkaba",
        F,
        "Two counter-rotating tetrahedra in 3D, a star tetrahedron",
        dmt_merkaba,
    ),
    ShaderInfo::generator(
        "dmt_platonic",
        "Platonic solids",
        F,
        "A cube nested in an octahedron inside a sphere, all turning",
        dmt_platonic,
    ),
    ShaderInfo::generator(
        "dmt_lotus_mandala",
        "Lotus mandala",
        F,
        "Seven layers of counter-turning petals around a glowing seed",
        dmt_lotus_mandala,
    ),
    ShaderInfo::generator(
        "dmt_energy_body",
        "Energy body",
        F,
        "A seated figure in layered auras, with a spine and seven chakras",
        dmt_energy_body,
    ),
    ShaderInfo::generator(
        "dmt_third_eye",
        "Third eye",
        F,
        "A radiant eye: fractal iris, halo contours and rays",
        dmt_third_eye,
    ),
    ShaderInfo::post(
        "dmt_kaleidoscope",
        "Kaleidoscope",
        F,
        "Mirrors the picture into a turning six-fold symmetry",
        dmt_kaleidoscope,
    ),
    ShaderInfo::post(
        "dmt_breath",
        "Breathing",
        F,
        "The image swells and ebbs with a prismatic colour split",
        dmt_breath,
    ),
    ShaderInfo::post(
        "dmt_tracers",
        "Tracers",
        F,
        "Spectral echoes trail every bright shape",
        dmt_tracers,
    ),
    ShaderInfo::post(
        "dmt_aura",
        "Aura",
        F,
        "Outlines every edge in drifting rainbow light and a soft halo",
        dmt_aura,
    ),
    ShaderInfo::post(
        "dmt_liquid",
        "Liquid drift",
        F,
        "Slow domain-warped flow, as if the picture were underwater",
        dmt_liquid,
    ),
    ShaderInfo::post(
        "dmt_sacred_overlay",
        "Sacred overlay",
        F,
        "A glowing flower-of-life lattice laid over the picture",
        dmt_sacred_overlay,
    ),
    ShaderInfo::post(
        "dmt_hue_flow",
        "Hue flow",
        F,
        "Colours slide round the wheel, with boosted saturation",
        dmt_hue_flow,
    ),
];

// ------------------------------------------------------------- helpers --

/// One output pixel in centred units.
fn pw(c: &Ctx) -> f32 {
    2.0 / c.h.max(1) as f32
}

fn spectrum(h: f32) -> Vec3 {
    hsv(fract(h), 0.72, 1.0)
}

/// Crisp neon line with a soft halo around a distance field.
fn line(d: f32, w: f32, k: f32) -> f32 {
    fill_aa(d.abs() - w, w) + 0.28 * (-d.abs() * k).exp()
}

fn vignette(q: Vec2) -> f32 {
    1.0 - 0.35 * smoothstep(0.6, 1.9, q.len())
}

/// Rotate around y by `a`, then around x by `b`.
fn rot3(p: Vec3, a: f32, b: f32) -> Vec3 {
    let (sa, ca) = a.sin_cos();
    let x = p.x * ca + p.z * sa;
    let z = -p.x * sa + p.z * ca;
    let (sb, cb) = b.sin_cos();
    vec3(x, p.y * cb - z * sb, p.y * sb + z * cb)
}

fn proj(p: Vec3) -> Vec2 {
    let k = 2.8 / (2.8 + p.z);
    vec2(p.x * k, p.y * k)
}

/// Hue rotation about the grey axis.
fn hue_rot(col: Vec3, a: f32) -> Vec3 {
    let k = 0.57735027;
    let (s, co) = a.sin_cos();
    let cr = vec3(
        k * (col.z - col.y),
        k * (col.x - col.z),
        k * (col.y - col.x),
    );
    let dp = k * (col.x + col.y + col.z);
    col * co + cr * s + vec3(k, k, k) * (dp * (1.0 - co))
}

fn deep_void(q: Vec2, t: f32, c: &Ctx) -> Vec3 {
    let r = q.len();
    c.pal
        .bg
        .mix(c.pal.a * 0.18, 0.5 + 0.5 * (r * 2.2 - t * 0.3).sin())
        * (1.0 - 0.5 * smoothstep(0.2, 1.6, r))
}

// ----------------------------------------------------------- mandelbrot --

pub fn mandelbrot_zoom(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let ph = fract(c.time / 30.0);
    let scale = 1.6 * (5.0e-4f32).powf(ph);
    let target = vec2(-0.743_643_9, 0.131_825_9);
    let start = vec2(-0.5, 0.0);
    let cen = target + (start - target) * (scale / 1.6).powf(1.3);
    let c0 = cen + q * scale;
    let mut z = vec2(0.0, 0.0);
    let mut i: i32 = 0;
    let n: i32 = 150 + (ph * 220.0) as i32;
    while i < n && z.dot(z) < 256.0 {
        z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c0;
        i += 1;
    }
    let fade = smoothstep(0.0, 0.025, ph) * smoothstep(1.0, 0.965, ph);
    if i >= n {
        return c.pal.bg * fade;
    }
    let sm = i as f32 + 1.0 - (z.dot(z).ln() * 0.5).ln() / std::f32::consts::LN_2;
    let col = palette(
        sm * 0.022 + 0.55,
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(1.0, 1.0, 1.0),
        vec3(0.0, 0.18, 0.32),
    );
    let lift = smoothstep(2.0, 14.0, sm);
    c.pal.bg.mix(col, lift) * fade
}

// ----------------------------------------------------------- generators --

pub fn dmt_breakthrough(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let r = q.len().max(1e-4);
    let a = q.y.atan2(q.x) + 0.15 * (t * 0.4).sin();
    let depth = 0.55 / (r + 0.04) + t * 1.1;
    let tx = fract(a / TAU * 12.0) - 0.5;
    let ty = fract(depth) - 0.5;
    let band = (r * 6.0).min(1.0);
    let lw = 0.03 + 0.05 * (1.0 - band);
    let diamond = (tx.abs() + ty.abs() - 0.34).abs();
    let ring = (vec2(tx, ty).len() - 0.18).abs();
    let cross = tx.abs().min(ty.abs());
    let hue = depth * 0.06 + t * 0.04 + r * 0.25;
    let mut col = spectrum(hue) * (fill_aa(diamond - lw, lw * 0.7) * 1.1)
        + spectrum(hue + 0.33) * (fill_aa(ring - lw * 0.8, lw * 0.6) * 0.9)
        + spectrum(hue + 0.6) * (fill_aa(cross - lw * 0.5, lw * 0.5) * 0.35);
    // facets glow between the lines
    col = col + spectrum(hue + 0.15) * (0.18 * (1.0 - (tx.abs() + ty.abs()) * 1.4).max(0.0));
    col = col * smoothstep(0.0, 0.3, r) * (0.55 + 0.45 * band);
    let core = (-r * 7.0).exp() * (0.7 + 0.3 * (t * 2.0).sin());
    col + Vec3::splat(1.0) * core * 0.9 + c.pal.a * ((-r * 2.0).exp() * 0.15)
}

pub fn dmt_machine_elves(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let mut p = q * 1.15;
    let mut acc = Vec3::BLACK;
    for i in 0..8 {
        let fi = i as f32;
        p = p.abs()
            - vec2(
                0.5 + 0.12 * (t * 0.21 + fi).sin(),
                0.28 + 0.06 * (t * 0.17).cos(),
            );
        p = p.rot(t * 0.08 + fi * 0.32) * 1.2;
        let d = p.x.abs().min(p.y.abs()).min((p.len() - 0.35).abs());
        let g = 0.011 / (d + 0.011);
        acc = acc + spectrum(fi / 8.0 + t * 0.05 + p.len() * 0.1) * (g * 0.2);
    }
    let v = vignette(q);
    deep_void(q, t, c) + acc * v
}

pub fn dmt_flower_of_life(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let breath = 1.0 + 0.05 * (t * 0.7).sin();
    let sc = 0.3 * breath;
    let p = q.rot(t * 0.05) / sc;
    let wl = (pw(c) / sc).max(0.006);
    let fl = flower(p, wl);
    let l = line(fl.x, wl * 1.3, 18.0 / 1.0);
    let hue = q.len() * 0.45 - t * 0.05 + fl.y * 0.05;
    let mut col = deep_void(q, t, c);
    col = col + spectrum(hue) * (fl.y * 0.07);
    col = col + spectrum(hue + 0.2) * (l * 0.85);
    // the seed ring at the centre
    let seed = ((p.len() - 1.0).abs() - wl).max(0.0);
    col = col + Vec3::WHITE * ((-seed * 20.0).exp() * 0.2);
    col * vignette(q)
}

/// (distance to nearest circle outline, number of circles covering) for a
/// flower-of-life lattice whose circles have radius 1.
fn flower(p: Vec2, w: f32) -> Vec2 {
    let by = p.y / 0.866_025_4;
    let bx = p.x - 0.5 * by;
    let i0 = bx.floor();
    let j0 = by.floor();
    let mut dmin: f32 = 10.0;
    let mut cover: f32 = 0.0;
    for dj in 0..5 {
        for di in 0..5 {
            let i = i0 + di as f32 - 2.0;
            let j = j0 + dj as f32 - 2.0;
            let ctr = vec2(i + 0.5 * j, 0.866_025_4 * j);
            let r = (p - ctr).len();
            dmin = dmin.min((r - 1.0).abs());
            cover += fill_aa(r - 1.0, w);
        }
    }
    vec2(dmin, cover)
}

fn mpt(i: u32, r: f32, rot: f32) -> Vec2 {
    if i == 0 {
        vec2(0.0, 0.0)
    } else {
        let k = ((i - 1) % 6) as f32;
        let ring = if i < 7 { 1.0 } else { 2.0 };
        let a = k * TAU / 6.0 + rot;
        vec2(a.cos(), a.sin()) * (r * ring)
    }
}

pub fn dmt_metatron(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let r = 0.3 * (1.0 + 0.03 * (t * 0.8).sin());
    let rot = t * 0.12;
    let mut dl: f32 = 10.0;
    let mut hue: f32 = 0.0;
    for i in 0..13u32 {
        for j in (i + 1)..13u32 {
            let d = sd_segment(q, mpt(i, r, rot), mpt(j, r, rot));
            if d < dl {
                dl = d;
                hue = (i * 13 + j) as f32 / 169.0;
            }
        }
    }
    let mut dc: f32 = 10.0;
    let mut chue: f32 = 0.0;
    for i in 0..13u32 {
        let d = (q - mpt(i, r, rot)).len();
        let ring = (d - r * 0.5).abs();
        if ring < dc {
            dc = ring;
            chue = i as f32 / 13.0;
        }
    }
    let outer = ((q.len() - (r * 2.0 + r * 0.5)).abs()).min((q.len() - r * 3.3).abs());
    let mut col = deep_void(q, t, c);
    col = col + spectrum(hue + t * 0.04) * (line(dl, w * 0.9, 40.0) * 0.55);
    col = col + spectrum(chue + 0.5 + t * 0.04) * (line(dc, w * 1.2, 30.0) * 0.9);
    col = col + c.pal.c * (line(outer, w, 30.0) * 0.5);
    let core = (-q.len() * 14.0).exp();
    (col + Vec3::WHITE * (core * 0.5)) * vignette(q)
}

fn sd_tri(p: Vec2, r: f32) -> f32 {
    let k = 1.732_050_8;
    let mut p = vec2(p.x.abs() - r, p.y + r / k);
    if p.x + k * p.y > 0.0 {
        p = vec2(p.x - k * p.y, -k * p.x - p.y) / 2.0;
    }
    p = vec2(p.x - p.x.clamp(-2.0 * r, 0.0), p.y);
    let l = p.len();
    if p.y > 0.0 { -l } else { l }
}

/// (half-side, y shift) of the upward triangles.
fn up_tri(k: u32) -> Vec2 {
    match k {
        0 => vec2(0.56, -0.02),
        1 => vec2(0.44, 0.03),
        2 => vec2(0.33, 0.05),
        _ => vec2(0.21, 0.06),
    }
}

/// (half-side, y shift) of the downward triangles.
fn down_tri(k: u32) -> Vec2 {
    match k {
        0 => vec2(0.53, 0.05),
        1 => vec2(0.43, 0.0),
        2 => vec2(0.34, -0.03),
        3 => vec2(0.25, -0.03),
        _ => vec2(0.15, -0.02),
    }
}

pub fn dmt_sri_yantra(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let pulse = 1.0 + 0.02 * (t * 0.9).sin();
    let p = q / pulse;
    let r = p.len();
    let a = p.y.atan2(p.x);
    let mut col = deep_void(q, t, c);
    // lotus petals: two rings
    let p8 = 0.74 + 0.12 * (4.0 * a + t * 0.2).cos().abs();
    let p16 = 0.86 + 0.09 * (8.0 * a - t * 0.2).cos().abs();
    col = col + spectrum(0.92 + a / TAU * 0.3) * (line(r - p8, w, 38.0) * 0.8);
    col = col + spectrum(0.1 + a / TAU * 0.3) * (line(r - p16, w, 38.0) * 0.6);
    col = col + c.pal.c * (line(r - 0.69, w, 36.0) * 0.7);
    // gate
    let gate = sd_box(p, vec2(0.97, 0.97));
    col = col + c.pal.b * (line(gate, w, 36.0) * 0.35);
    // triangles
    let mut hue: f32;
    for k in 0..4u32 {
        let u = up_tri(k);
        let d = sd_tri(vec2(p.x, p.y - u.y), u.x);
        hue = k as f32 * 0.08;
        col = col + spectrum(0.55 + hue + t * 0.03) * (line(d, w, 46.0) * 0.8);
    }
    for k in 0..5u32 {
        let u = down_tri(k);
        let d = sd_tri(vec2(p.x, -(p.y - u.y)), u.x);
        hue = k as f32 * 0.07;
        col = col + spectrum(0.85 + hue + t * 0.03) * (line(d, w, 46.0) * 0.8);
    }
    let bindu = (-r * 40.0).exp() * 1.2 + fill_aa(r - 0.012, w);
    (col + Vec3::splat(1.0) * bindu) * vignette(q)
}

fn tet_v(i: u32, flip: f32) -> Vec3 {
    let v = match i {
        0 => vec3(1.0, 1.0, 1.0),
        1 => vec3(1.0, -1.0, -1.0),
        2 => vec3(-1.0, 1.0, -1.0),
        _ => vec3(-1.0, -1.0, 1.0),
    };
    v * flip
}

pub fn dmt_merkaba(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let mut col = deep_void(q, t, c);
    let s = 0.5;
    for which in 0..2u32 {
        let flip = if which == 0 { 1.0 } else { -1.0 };
        let ang = t * 0.45 * flip;
        let tint = if which == 0 { c.pal.a } else { c.pal.b };
        let mut d: f32 = 10.0;
        let mut depth: f32 = 0.0;
        for i in 0..4u32 {
            for j in (i + 1)..4u32 {
                let a = rot3(tet_v(i, flip) * s, ang, 0.5 + 0.2 * (t * 0.3).sin());
                let b = rot3(tet_v(j, flip) * s, ang, 0.5 + 0.2 * (t * 0.3).sin());
                let dd = sd_segment(q, proj(a), proj(b));
                if dd < d {
                    d = dd;
                    depth = (a.z + b.z) * 0.5;
                }
            }
        }
        let br = 0.65 - depth * 0.7;
        col = col + tint * (line(d, w * 1.5, 30.0) * br);
        for i in 0..4u32 {
            let a = rot3(tet_v(i, flip) * s, ang, 0.5 + 0.2 * (t * 0.3).sin());
            let dv = (q - proj(a)).len();
            col = col + Vec3::WHITE * (fill_aa(dv - 0.014, w) * 0.9 + (-dv * 30.0).exp() * 0.25);
        }
    }
    let ring = (q.len() - 0.86).abs();
    col = col + c.pal.c * (line(ring, w, 30.0) * 0.45);
    col = col + Vec3::WHITE * ((-q.len() * 9.0).exp() * 0.25);
    col * vignette(q)
}

fn cube_v(i: u32) -> Vec3 {
    vec3(
        if i & 1 == 0 { -1.0 } else { 1.0 },
        if i & 2 == 0 { -1.0 } else { 1.0 },
        if i & 4 == 0 { -1.0 } else { 1.0 },
    )
}

fn octa_v(i: u32) -> Vec3 {
    let s = if i % 2 == 0 { 1.0 } else { -1.0 };
    match i / 2 {
        0 => vec3(s, 0.0, 0.0),
        1 => vec3(0.0, s, 0.0),
        _ => vec3(0.0, 0.0, s),
    }
}

pub fn dmt_platonic(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let mut col = deep_void(q, t, c);
    // cube (edges join vertices that differ in one bit)
    let mut dc: f32 = 10.0;
    for i in 0..8u32 {
        for bit in 0..3u32 {
            let j = i ^ (1 << bit);
            if i < j {
                let a = rot3(cube_v(i) * 0.42, t * 0.35, t * 0.22);
                let b = rot3(cube_v(j) * 0.42, t * 0.35, t * 0.22);
                dc = dc.min(sd_segment(q, proj(a), proj(b)));
            }
        }
    }
    // octahedron (all pairs except opposite vertices)
    let mut dof: f32 = 10.0;
    for i in 0..6u32 {
        for j in (i + 1)..6u32 {
            if j != (i ^ 1) {
                let a = rot3(octa_v(i) * 0.72, -t * 0.28, t * 0.18 + 1.0);
                let b = rot3(octa_v(j) * 0.72, -t * 0.28, t * 0.18 + 1.0);
                dof = dof.min(sd_segment(q, proj(a), proj(b)));
            }
        }
    }
    let sphere = (q.len() - 0.95).abs();
    col = col + c.pal.a * (line(dc, w * 1.1, 34.0) * 0.9);
    col = col + c.pal.b * (line(dof, w * 1.1, 34.0) * 0.9);
    col = col + c.pal.c * (line(sphere, w, 28.0) * 0.5);
    for i in 0..8u32 {
        let a = rot3(cube_v(i) * 0.42, t * 0.35, t * 0.22);
        let d = (q - proj(a)).len();
        col = col + Vec3::WHITE * (fill_aa(d - 0.012, w) * 0.9);
    }
    col * vignette(q)
}

pub fn dmt_lotus_mandala(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let r = q.len();
    let a = q.y.atan2(q.x);
    let mut col = deep_void(q, t, c);
    for l in 0..7u32 {
        let fl = l as f32;
        let rr = 0.98 - 0.125 * fl;
        let n = 24.0 - 3.0 * fl;
        let dir = if l % 2 == 0 { 1.0 } else { -1.0 };
        let pet = rr * (0.6 + 0.4 * (0.5 * n * a + dir * t * (0.15 + 0.03 * fl)).cos().abs());
        let d = r - pet;
        let fill = fill_aa(d, w);
        let hue = fl / 7.0 * 0.8 + t * 0.03 + r * 0.25;
        let inner = spectrum(hue) * (0.45 + 0.55 * smoothstep(-0.25, 0.0, d));
        col = col.mix(inner * 0.8, fill * 0.85);
        col = col + spectrum(hue + 0.12) * (fill_aa(d.abs() - w, w) * 0.9);
    }
    let core = (-r * 12.0).exp();
    col = col + Vec3::WHITE * core * 0.7 + c.pal.c * ((-r * 5.0).exp() * 0.2);
    col * vignette(q)
}

/// Seated-figure signed distance in figure units (about -0.3..0.65 tall).
fn body_sd(p: Vec2) -> f32 {
    let head = (p - vec2(0.0, 0.53)).len() - 0.095;
    let neck = sd_segment(p, vec2(0.0, 0.46), vec2(0.0, 0.38)) - 0.04;
    let chest = sd_round_box(p - vec2(0.0, 0.2), vec2(0.15, 0.19), 0.08);
    let hips = sd_round_box(p - vec2(0.0, -0.07), vec2(0.13, 0.07), 0.05);
    let shoulders = sd_segment(p, vec2(-0.16, 0.32), vec2(0.16, 0.32)) - 0.05;
    let mut d = head.min(neck).min(chest).min(hips).min(shoulders);
    let ax = p.x.abs();
    let q = vec2(ax, p.y);
    let upper = sd_segment(q, vec2(0.17, 0.31), vec2(0.23, 0.1)) - 0.034;
    let fore = sd_segment(q, vec2(0.23, 0.1), vec2(0.07, -0.07)) - 0.03;
    let thigh = sd_segment(q, vec2(0.1, -0.08), vec2(0.3, -0.17)) - 0.06;
    let shin = sd_segment(q, vec2(0.3, -0.18), vec2(-0.08, -0.22)) - 0.045;
    d = d.min(upper).min(fore).min(thigh).min(shin);
    d
}

pub fn dmt_energy_body(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let k = 0.62;
    let p = vec2(q.x * k, q.y * k + 0.17);
    let w = pw(c) * k;
    let d = body_sd(p);
    let r = q.len();
    let mut col = deep_void(q, t, c);
    // faint sacred circles behind
    let circ = (r - 0.55)
        .abs()
        .min((r - 0.9).abs())
        .min(((q - vec2(0.0, 0.0)).len() - 1.25).abs());
    col = col + c.pal.b * (fill_aa(circ - pw(c), pw(c)) * 0.15);
    // aura: echoing contours of the silhouette, each outer one fainter
    let out = d.max(0.0);
    let band = (out * 16.0 - t * 0.55).fract();
    let ring = smoothstep(0.0, 0.1, band) * smoothstep(0.26, 0.12, band);
    let fade = (-out * 3.2).exp();
    col = col + spectrum(out * 2.6 - t * 0.05) * (ring * fade * 0.95);
    col = col + spectrum(out * 2.6 - t * 0.05 + 0.15) * ((-out * 9.0).exp() * 0.3);
    // radiant rays
    let a = q.y.atan2(q.x);
    let rays =
        (a * 18.0 + t * 0.1).sin().abs().powf(18.0) * smoothstep(0.1, 1.5, r) * (-r * 0.8).exp();
    col = col + c.pal.c * (rays * 0.35);
    // inside the body: translucent, brighter at the rim
    let inside = fill_aa(d, w);
    let depth = (-d).max(0.0);
    let rim = (-depth * 48.0).exp();
    let mut body =
        c.pal.bg * 2.0 + c.pal.a * 0.12 + c.pal.b * (0.1 + 0.25 * (depth * 6.0).min(1.0));
    // nervous web
    let web = (noise(p * 28.0 + vec2(0.0, t * 0.1)) - 0.5).abs();
    body = body + c.pal.b * (smoothstep(0.03, 0.0, web) * 0.4);
    // ribs
    let rib_zone =
        smoothstep(0.1, 0.12, p.y) * smoothstep(0.36, 0.3, p.y) * smoothstep(0.0, 0.03, p.x.abs());
    let rib = (p.y * 46.0 + p.x.abs() * p.x.abs() * 120.0).sin().abs();
    body = body
        + Vec3::WHITE
            * (smoothstep(0.88, 1.0, 1.0 - rib)
                * rib_zone
                * 0.3
                * smoothstep(0.17, 0.05, p.x.abs()));
    // spine
    let spine = sd_segment(p, vec2(0.0, -0.08), vec2(0.0, 0.46));
    let vert = (p.y * 90.0).sin().abs();
    body = body + Vec3::WHITE * (fill_aa(spine - 0.006, w) * (0.4 + 0.5 * vert));
    // seven chakras, red to violet
    for i in 0..7u32 {
        let fi = i as f32;
        let y = match i {
            0 => -0.1,
            1 => 0.0,
            2 => 0.1,
            3 => 0.22,
            4 => 0.36,
            5 => 0.53,
            _ => 0.66,
        };
        let cc = hsv(fi / 7.0 * 0.78, 0.9, 1.0);
        let pulse = 0.75 + 0.25 * (t * 1.6 + fi * 0.9).sin();
        let dd = (p - vec2(0.0, y)).len();
        body = body + cc * ((-dd * 22.0).exp() * 1.3 * pulse);
        body = body + cc * (fill_aa((dd - 0.03).abs() - w * 0.8, w) * 0.9);
        col = col + cc * ((-dd * 14.0).exp() * 0.35 * pulse);
    }
    col = col.mix(body, inside * 0.92) + spectrum(0.55 + t * 0.03) * (rim * inside * 0.45);
    col = col + Vec3::WHITE * (fill_aa(d.abs() - w * 0.8, w) * 0.5);
    col * vignette(q)
}

pub fn dmt_third_eye(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let t = c.time;
    let w = pw(c);
    let r = q.len();
    let a = q.y.atan2(q.x);
    let big = 1.0;
    let lens_h = 0.32;
    let d1 = (q - vec2(0.0, big - lens_h)).len() - big;
    let d2 = (q + vec2(0.0, big - lens_h)).len() - big;
    let lens = d1.max(d2);
    let mut col = deep_void(q, t, c);
    // halo contours around the lid
    let out = lens.max(0.0);
    let band = (out * 9.0 - t * 0.4).fract();
    let ring = smoothstep(0.0, 0.08, band) * smoothstep(0.2, 0.1, band);
    col = col + spectrum(out * 2.0 - t * 0.05) * (ring * (-out * 2.4).exp() * 0.85);
    // radiant rays and lashes
    let rays = (a * 28.0 + (t * 0.3).sin()).sin().abs().powf(22.0);
    let ray_zone = smoothstep(0.4, 0.8, r) * smoothstep(1.6, 0.9, r);
    col = col + c.pal.c * (rays * ray_zone * 0.55 * smoothstep(0.0, 0.04, lens));
    col = col + c.pal.a * (fill_aa((r - 1.15).abs() - w, w) * 0.3);
    // sclera
    let sclera = c.pal.bg * 2.5
        + c.pal.a * (0.2 * smoothstep(-0.3, 0.0, lens))
        + c.pal.b * (0.18 * (1.0 - smoothstep(0.0, 0.25, r)));
    let inside = fill_aa(lens, w);
    // iris: fractal spokes and rings
    let ir = r / 0.3;
    let spokes = (a * 30.0 + (ir * 9.0 - t * 1.2).sin() * 1.6).sin().abs();
    let rings = (ir * 14.0 - t * 0.8).sin().abs();
    let hue = ir * 0.35 + t * 0.04 + spokes * 0.08;
    let mut iris = spectrum(hue) * (0.45 + 0.55 * spokes) * (0.6 + 0.4 * rings);
    iris = iris * (0.35 + 0.65 * smoothstep(0.1, 0.55, ir));
    iris = iris + c.pal.c * (fill_aa((ir - 1.0).abs() - 0.04, 0.04) * 0.9);
    let iris_mask = fill_aa(r - 0.3, w);
    // pupil: dark, with a thin starburst halo
    let pr = 0.095 + 0.02 * (t * 0.9).sin();
    let pupil = fill_aa(r - pr, w);
    let mut eye = sclera.mix(iris, iris_mask);
    eye = eye.mix(Vec3::BLACK, pupil);
    eye = eye + Vec3::WHITE * (fill_aa((r - pr).abs() - w * 0.8, w) * 0.5);
    // catchlight
    let hl = (q - vec2(0.08, 0.08)).len();
    eye = eye + Vec3::WHITE * (fill_aa(hl - 0.022, w) * 0.9);
    col = col.mix(eye, inside);
    // lid outline, twin lines
    col = col + Vec3::WHITE * (fill_aa(lens.abs() - w * 1.2, w) * 0.8);
    col =
        col + c.pal.b * (line((lens + 0.035).abs(), w, 40.0) * 0.45 * smoothstep(0.0, -0.02, lens));
    col * vignette(q)
}

// ----------------------------------------------------------------- post --

pub fn dmt_kaleidoscope(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let q = c.centered(uv);
    let r = q.len();
    let a = q.y.atan2(q.x) + c.time * 0.06;
    let seg = TAU / 6.0;
    let m = fmod(a + TAU * 4.0, seg);
    let f = (m - seg * 0.5).abs();
    let p = vec2(f.cos(), f.sin()) * r * (1.0 + 0.05 * (c.time * 0.5).sin());
    let u = vec2(p.x / (2.0 * c.aspect()) + 0.5, 0.5 - p.y * 0.5);
    let out = src.sample(vec2(fract(u.x * 0.999), fract(u.y * 0.999)));
    s.mix(out, c.intensity)
}

pub fn dmt_breath(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let ctr = vec2(0.5, 0.5);
    let d = uv - ctr;
    let r = d.len();
    let k = 0.035 * (c.time * 1.1 + r * 5.0).sin();
    let split = 0.012 * (0.4 + 0.6 * (c.time * 0.7).sin().abs());
    let sc = |m: f32| ctr + d * (1.0 + k + m * split * (0.5 + r * 2.0));
    let col = vec3(
        src.sample(sc(1.0)).x,
        src.sample(sc(0.0)).y,
        src.sample(sc(-1.0)).z,
    );
    s.mix(col, c.intensity)
}

pub fn dmt_tracers(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let ctr = vec2(0.5, 0.5);
    let mut out = s;
    for i in 1..7 {
        let fi = i as f32;
        let ang = 0.012 * fi * (c.time * 0.4).sin() + 0.01 * fi;
        let sc = 1.0 - 0.022 * fi;
        let p = ctr + (uv - ctr).rot(ang) * sc;
        let e = src.sample(p);
        let tint = spectrum(fi / 6.0 + c.time * 0.1);
        let fade = (0.78f32).powf(fi);
        let echo = (e * tint * 1.4) * fade;
        out = vec3(out.x.max(echo.x), out.y.max(echo.y), out.z.max(echo.z));
    }
    s.mix(out, c.intensity)
}

pub fn dmt_aura(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let px = c.texel();
    let l = |dx: f32, dy: f32| src.sample(uv + vec2(dx * px.x, dy * px.y)).luma();
    let gx = l(1.5, 0.0) - l(-1.5, 0.0);
    let gy = l(0.0, 1.5) - l(0.0, -1.5);
    let edge = sat((gx * gx + gy * gy).sqrt() * 5.0);
    let mut halo = Vec3::BLACK;
    for i in 0..8 {
        let a = i as f32 * 0.8;
        let h = src.sample(uv + vec2(a.cos() * px.x * 9.0, a.sin() * px.y * 9.0));
        halo = halo + h;
    }
    halo = halo / 8.0;
    let col = spectrum(s.luma() * 1.5 + c.time * 0.08 + (uv - vec2(0.5, 0.5)).len());
    let out = s * 0.55 + col * (edge * 1.6) + halo * col * 0.45;
    s.mix(out, c.intensity)
}

pub fn dmt_liquid(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let t = c.time;
    let n1 = noise(vec2(uv.x * 5.0 + t * 0.2, uv.y * 5.0)) - 0.5;
    let n2 = noise(vec2(uv.x * 5.0 + 7.0, uv.y * 5.0 - t * 0.17)) - 0.5;
    let m1 = noise(vec2(uv.x * 11.0 - t * 0.3, uv.y * 11.0 + 3.0)) - 0.5;
    let m2 = noise(vec2(uv.x * 11.0 + 9.0, uv.y * 11.0 + t * 0.25)) - 0.5;
    let off = vec2(n1 * 0.07 + m1 * 0.025, n2 * 0.07 + m2 * 0.025);
    s.mix(src.sample(uv + off), c.intensity)
}

pub fn dmt_sacred_overlay(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let q = c.centered(uv);
    let sc = 0.28 * (1.0 + 0.04 * (c.time * 0.6).sin());
    let p = q.rot(c.time * 0.04) / sc;
    let wl = (pw(c) / sc).max(0.006);
    let fl = flower(p, wl);
    let l = line(fl.x, wl * 1.2, 20.0);
    let col = spectrum(q.len() * 0.4 - c.time * 0.05 + fl.y * 0.04);
    let out = s * 0.85 + col * (l * 0.55) + col * (fl.y * 0.03);
    s.mix(out, c.intensity)
}

pub fn dmt_hue_flow(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let rot = hue_rot(s, c.time * 0.5 + (uv - vec2(0.5, 0.5)).len() * 2.0);
    let g = Vec3::splat(rot.luma());
    let out = g.mix(rot, 1.5);
    s.mix(out, c.intensity)
}
