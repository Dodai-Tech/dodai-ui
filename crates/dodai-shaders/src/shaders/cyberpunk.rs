//! Cyberpunk-inspired shaders: neon grids, rainy skylines, HUDs, glitches.

use crate::frame::Ctx;
use crate::math::*;
use crate::prelude::*;
use crate::{Family, ShaderInfo};

const F: Family = Family::Cyberpunk;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::generator(
        "neon_grid",
        "Neon grid",
        F,
        "Synthwave sun over a perspective grid",
        neon_grid,
    ),
    ShaderInfo::generator(
        "glow_waves",
        "Glow waves",
        F,
        "Ten stacked sine curves glowing over a blocky gradient",
        glow_waves,
    ),
    ShaderInfo::generator(
        "hex_pulse",
        "Hex pulse",
        F,
        "Hex cells with expanding energy rings",
        hex_pulse,
    ),
    ShaderInfo::generator(
        "circuit",
        "Circuit board",
        F,
        "Traces with data pulses running along them",
        circuit,
    ),
    ShaderInfo::generator(
        "hud_radar",
        "HUD radar",
        F,
        "Sweeping radar with blips and corner brackets",
        hud_radar,
    ),
    ShaderInfo::generator(
        "data_corrupt",
        "Data corruption",
        F,
        "Displaced colour bands and RGB tearing",
        data_corrupt,
    ),
    ShaderInfo::generator(
        "kv_glitch",
        "Key-art glitch",
        F,
        "Yellow washing into red, vertical streaks, dark skyline",
        kv_glitch,
    ),
    ShaderInfo::generator(
        "phantom_liberty",
        "Phantom Liberty",
        F,
        "Red key-art field, yellow slashes, datamosh blocks, magenta decay",
        phantom_liberty,
    ),
];

/// Glowing waves over a checkerboard of two gradients. After a well-known Shadertoy: ten curves,
/// each bent by a sine that runs with time and shifted a little more than the last, glowing as
/// `1 / distance`. Here the colours come from the palette, the blocks are square at any aspect,
/// and the waves are split into a warm and a cool ink so the stack shimmers.
pub fn glow_waves(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    // Checkerboard of square blocks, a gradient along x on one colour and along y on the other.
    let block = 0.02;
    let (bx, by) = (uv.x * c.aspect() / block, uv.y / block);
    let check = step(1.0, fmod(bx.floor() + by.floor(), 2.0));
    let bg = (p.a * uv.x * 0.35).mix(p.b * uv.y * 0.35, check) * 0.8 + p.bg * 0.2;
    let mut y = uv.y * 2.0 - 1.0 + 0.1;
    let x = (uv.x * 2.0 - 1.0) * c.aspect();
    let mut warm = 0.0;
    let mut cool = 0.0;
    for i in 0..10 {
        y += 0.07 * (x + i as f32 / 7.0 + c.time).sin();
        let w = 1.0 / (150.0 * y.abs().max(0.004));
        warm += w * (1.0 - i as f32 / 9.0);
        cool += w * (i as f32 / 9.0);
    }
    (bg + p.c * (warm * 1.6) + p.fg * (cool * 0.9) + p.a.mix(p.fg, 0.5) * (cool * 0.3)).clamp01()
}

/// Synthwave: striped sun, horizon haze and a scrolling perspective grid.
pub fn neon_grid(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let horizon = 0.56;
    let mut col;
    if uv.y < horizon {
        let t = uv.y / horizon;
        col = p.bg.mix(p.c * 0.35, t * t);
        // stars
        let g = vec2(uv.x * 160.0 * c.aspect(), uv.y * 90.0).floor();
        let h = hash2(g.x as i32, g.y as i32);
        if h > 0.992 && t < 0.7 {
            col = col + p.fg * ((c.time * 2.0 + h * 40.0).sin() * 0.3 + 0.5) * (1.0 - t);
        }
        // sun with slits that grow toward the bottom
        let q = vec2((uv.x - 0.5) * c.aspect(), uv.y - 0.34);
        let d = q.len();
        let slit = {
            let y = (q.y + 0.18) / 0.36;
            let f = fract(y * 9.0 - c.time * 0.2);
            step(0.12 + 0.35 * (1.0 - sat(y)), f)
        };
        let sun = smoothstep(0.205, 0.195, d) * if q.y > -0.02 { slit } else { 1.0 };
        let sun_col = p.a.mix(p.c, sat((q.y + 0.2) / 0.4));
        col = col.mix(sun_col, sun);
        col = col + p.c * glow(d - 0.2, 7.0) * 0.18;
    } else {
        let y = uv.y - horizon;
        let depth = 0.06 / y.max(0.002); // 1/y perspective
        let x = (uv.x - 0.5) * c.aspect() * depth;
        let z = depth * 2.0 + c.time * 1.5;
        let lx = (fract(x) - 0.5).abs();
        let lz = (fract(z) - 0.5).abs();
        let w = 0.03 + 0.012 * depth;
        let line =
            sat(smoothstep(0.5 - w, 0.5, lx.max(0.0) * 1.0 + 0.0) + smoothstep(0.5 - w, 0.5, lz));
        let fade = sat(1.0 - depth * 0.07);
        col = p.bg.mix(p.b * 0.12, sat(y * 3.0));
        col = col + p.b * line * fade * 1.4 + p.c * line * fade * 0.25;
    }
    // horizon glow
    col + p.c * glow((uv.y - horizon) * 6.0, 14.0) * 0.45
}

/// Hex lattice with energy rings travelling outward from the centre.
pub fn hex_pulse(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv) * 4.5;
    let (g, id) = hex_cell(q);
    let edge_d = hex_dist(g);
    let edge = smoothstep(0.42, 0.5, edge_d);
    let cell_id = id.floor();
    let rid = id.len();
    let ring = (rid * 1.3 - c.time * 1.4).sin() * 0.5 + 0.5;
    let pulse = smoothstep(0.82, 1.0, ring);
    let rnd = hash2(cell_id.x as i32, cell_id.y as i32);
    let twinkle = step(0.93, rnd) * (c.time * 2.0 + rnd * 20.0).sin().abs();
    let mut col = p.bg + p.b * 0.05 * (1.0 - edge_d);
    col = col + p.a * edge * (0.35 + 0.9 * pulse);
    col = col + p.c * pulse * 0.25 * (1.0 - edge);
    col + p.c * twinkle * 0.35
}

/// Manhattan traces with data pulses.
pub fn circuit(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let s = 28.0;
    let q = vec2(uv.x * s * c.aspect(), uv.y * s);
    let cell = q.floor();
    let f = q.fract();
    let (ix, iy) = (cell.x as i32, cell.y as i32);
    let h = hash2(ix, iy);
    let mut col = p.bg;
    // trace orientation per cell
    let horiz = h < 0.45;
    let vert = (0.45..0.9).contains(&h);
    let t = 0.06;
    let d_line = if horiz {
        (f.y - 0.5).abs()
    } else if vert {
        (f.x - 0.5).abs()
    } else {
        1.0
    };
    let line = smoothstep(t + 0.02, t, d_line);
    col = col + p.b * line * 0.25;
    // pulse travelling along the trace
    let along = if horiz { q.x } else { q.y };
    let ph = fract(along * 0.08 - c.time * (0.25 + hash2(ix / 5, iy / 5) * 0.4) + h);
    let pulse = smoothstep(0.1, 0.0, ph) * line;
    col = col + p.a * pulse * 1.4;
    // via pads
    let pad = sd_circle(f - 0.5, 0.18);
    if h > 0.9 {
        col = col + p.c * (smoothstep(0.02, 0.0, pad.abs() - 0.02) * 0.9);
    }
    col
}

/// Radar HUD: rings, rotating sweep, blips, brackets.
pub fn hud_radar(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let q = c.centered(uv);
    let r = q.len();
    let radius = 0.82;
    let mut col = p.bg + p.b * 0.04 * (1.0 - r * 0.5);
    // rings and ticks
    for k in 1..=4 {
        col = col + p.b * glow((r - radius * k as f32 / 4.0) * 120.0, 1.0) * 0.5;
    }
    let ang = q.y.atan2(q.x);
    let tick = step(0.9, fract(ang * 36.0 / std::f32::consts::TAU))
        * smoothstep(radius - 0.03, radius, r)
        * step(r, radius + 0.02);
    col = col + p.b * tick;
    // sweep
    let sweep_a = -c.time * 1.3;
    let mut da = (ang - sweep_a) % std::f32::consts::TAU;
    if da < 0.0 {
        da += std::f32::consts::TAU;
    }
    let trail = (1.0 - da / 1.6).max(0.0).powi(2) * step(r, radius);
    col = col + p.a * trail * 0.55 + p.a * glow(da * 30.0, 1.0) * step(r, radius) * 0.6;
    // blips
    for i in 0..7 {
        let h1 = hash1(i as f32 * 3.1);
        let h2 = hash1(i as f32 * 7.7 + 1.0);
        let bp = vec2((h1 * 2.0 - 1.0) * 0.6, (h2 * 2.0 - 1.0) * 0.6);
        let ba = bp.y.atan2(bp.x);
        let mut since = (sweep_a - ba) % std::f32::consts::TAU;
        if since < 0.0 {
            since += std::f32::consts::TAU;
        }
        let life = (1.0 - since / 5.0).max(0.0);
        col = col + p.c * glow((q - bp).len() * 70.0, 1.0) * life;
    }
    // corner brackets
    let a = c.aspect();
    let m = vec2(q.x.abs() - (a - 0.08), q.y.abs() - 0.92);
    let br = (m.x.abs().min(m.y.abs()) < 0.008) as u32 as f32
        * step(m.x.max(m.y), 0.0)
        * step(-0.18, m.x.min(m.y));
    col + p.a * br * 0.9
}

/// Corrupted signal: bands shifting sideways with colour split.
pub fn data_corrupt(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let t = (c.time * 6.0).floor();
    let band = (uv.y * 24.0).floor();
    let r = hash2(band as i32, t as i32);
    let shift = if r > 0.8 {
        (hash2(band as i32 + 5, t as i32) - 0.5) * 0.3
    } else {
        0.0
    };
    let x = uv.x + shift;
    let blocks = |x: f32, off: f32| {
        let bx = (x * 12.0 + off).floor();
        let by = (uv.y * 24.0).floor();
        hash3(bx as i32, by as i32, t as i32 / 2)
    };
    // Sparse, colour-split blocks: stay below white so the palette reads.
    let v = |o: f32| step(0.7, blocks(x, o));
    let col = p.c * (v(0.0) * 0.55) + p.b * (v(0.12) * 0.42) + p.a * (v(-0.12) * 0.3);
    let scan = 0.8 + 0.2 * (uv.y * c.h as f32 * std::f32::consts::PI).sin().abs();
    (p.bg + col * scan) * (0.5 + 0.5 * step(0.1, hash2((uv.y * 80.0) as i32, t as i32)))
}

/// Key-art backdrop: the accent washes into red towards the right, vertical
/// pixel-sort streaks drip down and sometimes jump, a dark skyline sits below.
pub fn kv_glitch(uv: Vec2, c: &Ctx) -> Vec3 {
    let p = &c.pal;
    let t = (c.time * 4.0).floor();
    let front = uv.x + 0.04 * (c.time * 0.5 + uv.y * 3.0).sin();
    let g = smoothstep(0.35, 0.9, front);
    let base = p.a.mix(p.c, g);
    // Streaks: columns whose brightness is smeared along y.
    let col = (uv.x * 90.0).floor();
    let r = hash2(col as i32, 7);
    let live = step(0.55, r) * smoothstep(0.3, 0.7, g);
    let drip = fract(uv.y * (0.4 + r) - c.time * (0.1 + 0.3 * r) + r * 9.0);
    let streak = live * smoothstep(0.0, 0.8, drip);
    // Tears: a few columns flash bright for a moment.
    let jump = step(0.94, hash2(col as i32, t as i32)) * g;
    let lit = base * (1.0 - 0.5 * streak) + p.fg * (0.3 * jump);
    // Skyline: blocks of random height with a few lit windows.
    let bx = (uv.x * 14.0).floor();
    let top = 1.0 - (0.12 + 0.3 * hash2(bx as i32, 3));
    let building = step(top, uv.y);
    let wx = (uv.x * 14.0 * 6.0).floor();
    let wy = (uv.y * 40.0).floor();
    let window = building * step(0.8, hash3(wx as i32, wy as i32, (t / 6.0) as i32));
    let city = p.bg.mix(base, 0.12) + p.a * (0.5 * window);
    lit.mix(city, building)
}

/// Phantom Liberty key art: a red field with slanted yellow slashes, blocky datamosh
/// smear that turns magenta on the right, and a fade to black. Brand colours are fixed
/// (the look is a specific piece of art, not a palette).
pub fn phantom_liberty(uv: Vec2, c: &Ctx) -> Vec3 {
    let t = (c.time * 6.0).floor();
    let red = vec3(0.96, 0.12, 0.17);
    let deep = vec3(0.55, 0.05, 0.12);
    let yellow = vec3(0.99, 0.93, 0.04);
    let magenta = vec3(0.86, 0.12, 0.55);
    // Red field, darker toward the corners.
    let v = (uv.x - 0.5) * (uv.x - 0.5) + (uv.y - 0.4) * (uv.y - 0.4);
    let mut col = red.mix(deep, smoothstep(0.05, 0.5, v));
    // Right side decays toward magenta.
    col = col.mix(magenta, smoothstep(0.62, 1.0, uv.x) * 0.7);
    // Yellow slashes: irregular slanted bands; each has its own width and vertical extent,
    // and rows shift sideways in jumps like a corrupted frame.
    let row = (uv.y * 36.0).floor();
    let jitter =
        (hash2(row as i32, (t / 4.0) as i32) - 0.5) * 0.05 * step(0.6, hash2(row as i32, 5));
    let slant = (uv.x + jitter) * 3.2 + uv.y * 1.6 - 0.03 * (c.time * 0.4 + uv.y * 5.0).sin();
    let cell = slant.floor();
    let r1 = hash2(cell as i32, 5);
    let r2 = hash2(cell as i32, 9);
    let r3 = hash2(cell as i32, 13);
    let width = 0.22 + 0.4 * r1;
    let lo = 0.12 + 0.25 * r2;
    let hi = 0.5 + 0.3 * r3;
    let strip = smoothstep(lo, lo + 0.03, uv.y) * (1.0 - smoothstep(hi, hi + 0.03, uv.y));
    let slash = step(fract(slant), width) * step(0.15, r1) * strip;
    // Datamosh blocks: some cells shift tone, some wash toward yellow.
    let bx = (uv.x * 48.0).floor();
    let by = (uv.y * 27.0).floor();
    let h = hash3(bx as i32, by as i32, (t / 3.0) as i32);
    let mosh = step(0.86, h) * (0.4 + 0.6 * uv.x);
    let tone = red.mix(magenta, hash2(bx as i32, by as i32 + 40));
    col = col.mix(tone, mosh * 0.8);
    // Vertical smear columns that drip down from the slashes.
    let colid = (uv.x * 120.0).floor();
    let r = hash2(colid as i32, 11);
    let drip = fract(uv.y * (0.5 + r) - c.time * (0.05 + 0.2 * r) + r * 7.0);
    let smear = step(0.7, r) * smoothstep(0.0, 0.9, drip) * slash;
    let sl = yellow * (1.0 - 0.35 * smear);
    col = col.mix(sl, slash);
    // Horizontal tearing: a thin row flashes bright for a moment.
    let ty = (uv.y * 90.0).floor();
    let tear = step(0.985, hash2(ty as i32, t as i32));
    col = col + vec3(0.25, 0.2, 0.1) * tear;
    // Fade to black at the bottom, a touch at the top.
    let fade = smoothstep(0.7, 1.0, uv.y) + (1.0 - smoothstep(0.0, 0.12, uv.y)) * 0.4;
    col * (1.0 - fade.min(1.0))
}
