//! Matrix-style shaders: digital rain, code hallway and a green vision filter.

use crate::frame::{Ctx, Frame};
use crate::math::*;
use crate::prelude::*;
use crate::{Family, ShaderInfo};

const F: Family = Family::Matrix;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::generator(
        "digital_rain",
        "Digital rain",
        F,
        "Falling glyph columns with a bright head and fading tail",
        digital_rain,
    ),
    ShaderInfo::generator(
        "matrix_hallway",
        "Code hallway",
        F,
        "Rain wrapped into a tunnel of glyph walls",
        matrix_hallway,
    ),
    ShaderInfo::generator(
        "matrix_wall",
        "Glyph wall",
        F,
        "Dense flickering glyph field with travelling brightness",
        matrix_wall,
    ),
    ShaderInfo::post(
        "matrix_vision",
        "Matrix vision",
        F,
        "Turns any frame into green phosphor with rain over it",
        matrix_vision,
    ),
];

/// 5×7 pseudo-glyph, mirrored for a katakana-ish look. `f` is local cell
/// coordinates in `0..1`; returns 1 where a stroke is lit.
pub fn glyph(id: i32, f: Vec2) -> f32 {
    let (x, y) = (f.x * 1.25 - 0.125, f.y * 1.14 - 0.07);
    if !(0.0..1.0).contains(&x) || !(0.0..1.0).contains(&y) {
        return 0.0;
    }
    let gx = ((x * 5.0) as i32).min(4);
    let gy = ((y * 7.0) as i32).min(6);
    let mx = gx.min(4 - gx);
    // keep strokes connected: bias toward the centre column
    let bias = if mx == 2 { 0.35 } else { 0.55 };
    step(bias, hash3(id, mx, gy))
}

/// Rain brightness for column `col` at continuous `row`.
/// Returns `(trail, head)`.
fn rain(col: i32, row: f32, rows: f32, t: f32) -> (f32, f32) {
    let speed = 0.25 + 0.55 * hash1(col as f32 * 1.7);
    let span = rows * 1.8;
    let head = fmod(
        t * speed * rows * 0.6 + hash1(col as f32 + 50.0) * span,
        span,
    );
    let d = fmod(head - row, span);
    let len = 6.0 + hash1(col as f32 + 9.0) * 18.0;
    if d < len {
        ((1.0 - d / len).powf(1.6), step(d, 1.0))
    } else {
        (0.0, 0.0)
    }
}

fn rain_color(c: &Ctx, trail: f32, head: f32, g: f32) -> Vec3 {
    let p = &c.pal;
    let body = p.b.mix(p.a, trail) * trail * 1.2;
    (body + p.c * head * 1.4) * g
}

pub fn digital_rain(uv: Vec2, c: &Ctx) -> Vec3 {
    let rows = 38.0;
    let cols = (rows * c.aspect() / 0.62).floor();
    let p = vec2(uv.x * cols, uv.y * rows);
    let (ci, ri) = (p.x.floor() as i32, p.y.floor() as i32);
    let f = p.fract();
    let (trail, head) = rain(ci, p.y, rows, c.time);
    let change = (c.time * (1.5 + hash2(ci, ri) * 3.0)).floor() as i32;
    let g = glyph((hash3(ci, ri, change) * 4096.0) as i32, f);
    let dim = c.pal.b * 0.07 * glyph(ri * 977 + ci, f);
    c.pal.bg + dim + rain_color(c, trail, head, g)
}

pub fn matrix_hallway(uv: Vec2, c: &Ctx) -> Vec3 {
    let q = c.centered(uv);
    let r = q.len().max(0.02);
    let ang = q.y.atan2(q.x) / std::f32::consts::TAU + 0.5;
    let walls = 28.0;
    let rows = 38.0;
    let depth = 0.35 / r + c.time * 0.8;
    let p = vec2(ang * walls, depth * rows * 0.25);
    let (ci, ri) = (p.x.floor() as i32, p.y.floor() as i32);
    let f = p.fract();
    let (trail, head) = rain(ci, p.y, rows, c.time * 0.4);
    let change = (c.time * 2.0 + hash2(ci, ri) * 6.0).floor() as i32;
    let g = glyph((hash3(ci, ri, change) * 4096.0) as i32, f);
    let fog = smoothstep(0.0, 0.45, r);
    let col = rain_color(c, trail.max(0.12), head, g) * fog * 1.5;
    c.pal.bg + col + c.pal.b * glow(r * 6.0, 1.0) * 0.5
}

pub fn matrix_wall(uv: Vec2, c: &Ctx) -> Vec3 {
    let rows = 30.0;
    let cols = (rows * c.aspect() / 0.62).floor();
    let p = vec2(uv.x * cols, uv.y * rows);
    let (ci, ri) = (p.x.floor() as i32, p.y.floor() as i32);
    let f = p.fract();
    let flick = (c.time * 3.0 * (0.3 + hash2(ci, ri)) + hash2(ri, ci) * 40.0).floor() as i32;
    let g = glyph((hash3(ci, ri, flick) * 4096.0) as i32, f);
    let wave = 0.5 + 0.5 * ((uv.x * 3.0 - uv.y * 2.0) * 3.0 - c.time * 1.4).sin();
    let lit = smoothstep(0.55, 1.0, wave);
    let base = c.pal.b * 0.25 + c.pal.a * 0.9 * lit;
    c.pal.bg + base * g * (0.4 + 0.6 * hash2(ci, ri))
}

/// Post: luma to green phosphor, rain drawn over the picture.
pub fn matrix_vision(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let s = src.sample(uv);
    let l = s.luma();
    let green = c.pal.bg + c.pal.b * l * 0.9 + c.pal.a * l * l * 1.1;
    let rows = 40.0;
    let cols = (rows * c.aspect() / 0.62).floor();
    let p = vec2(uv.x * cols, uv.y * rows);
    let (ci, ri) = (p.x.floor() as i32, p.y.floor() as i32);
    let (trail, head) = rain(ci, p.y, rows, c.time);
    let change = (c.time * 2.0 + hash2(ci, ri) * 5.0).floor() as i32;
    let g = glyph((hash3(ci, ri, change) * 4096.0) as i32, p.fract());
    let overlay = rain_color(c, trail, head, g);
    s.mix(green, c.intensity) + overlay * 0.7 * c.intensity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyphs_are_mirrored_and_sparse() {
        let mut lit = 0;
        for y in 0..14 {
            for x in 0..10 {
                let f = vec2((x as f32 + 0.5) / 10.0, (y as f32 + 0.5) / 14.0);
                let m = vec2(1.0 - f.x, f.y);
                assert_eq!(glyph(7, f), glyph(7, m), "mirror at {x},{y}");
                lit += glyph(7, f) as i32;
            }
        }
        assert!(lit > 10 && lit < 120, "{lit}");
    }

    #[test]
    fn rain_head_is_one_cell() {
        let mut heads = 0.0;
        for r in 0..200 {
            heads += rain(3, r as f32 + 0.5, 100.0, 4.0).1;
        }
        assert!((1.0..=2.0).contains(&heads), "{heads}");
    }
}
