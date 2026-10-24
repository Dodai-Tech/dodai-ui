//! Tiny GLSL-flavoured maths: vectors, hashes, noise, easing. No dependencies.

#[cfg(target_arch = "spirv")]
use crate::prelude::*;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
#[derive(Clone, Copy, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
#[derive(Clone, Copy, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Colour from 0..=255 channels (no `u8` values, so it also runs on the GPU).
pub const fn rgb8(r: u32, g: u32, b: u32) -> Vec3 {
    Vec3 {
        x: r as f32 / 255.0,
        y: g as f32 / 255.0,
        z: b as f32 / 255.0,
    }
}
pub const fn rgb8a(c: [u32; 3]) -> Vec3 {
    rgb8(c[0], c[1], c[2])
}
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}
pub const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

macro_rules! ops {
    ($t:ident { $($f:ident),+ }) => {
        impl Add for $t { type Output = $t; fn add(self, o: $t) -> $t { $t { $($f: self.$f + o.$f),+ } } }
        impl Sub for $t { type Output = $t; fn sub(self, o: $t) -> $t { $t { $($f: self.$f - o.$f),+ } } }
        impl Mul for $t { type Output = $t; fn mul(self, o: $t) -> $t { $t { $($f: self.$f * o.$f),+ } } }
        impl Div for $t { type Output = $t; fn div(self, o: $t) -> $t { $t { $($f: self.$f / o.$f),+ } } }
        impl Mul<f32> for $t { type Output = $t; fn mul(self, o: f32) -> $t { $t { $($f: self.$f * o),+ } } }
        impl Div<f32> for $t { type Output = $t; fn div(self, o: f32) -> $t { $t { $($f: self.$f / o),+ } } }
        impl Add<f32> for $t { type Output = $t; fn add(self, o: f32) -> $t { $t { $($f: self.$f + o),+ } } }
        impl Sub<f32> for $t { type Output = $t; fn sub(self, o: f32) -> $t { $t { $($f: self.$f - o),+ } } }
        impl Mul<$t> for f32 { type Output = $t; fn mul(self, o: $t) -> $t { o * self } }
        impl Neg for $t { type Output = $t; fn neg(self) -> $t { $t { $($f: -self.$f),+ } } }
    };
}
ops!(Vec2 { x, y });
ops!(Vec3 { x, y, z });

impl Vec2 {
    pub fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn dot(self, o: Vec2) -> f32 {
        self.x * o.x + self.y * o.y
    }
    pub fn floor(self) -> Vec2 {
        vec2(self.x.floor(), self.y.floor())
    }
    pub fn fract(self) -> Vec2 {
        vec2(fract(self.x), fract(self.y))
    }
    pub fn abs(self) -> Vec2 {
        vec2(self.x.abs(), self.y.abs())
    }
    pub fn rot(self, a: f32) -> Vec2 {
        let (s, c) = a.sin_cos();
        vec2(self.x * c - self.y * s, self.x * s + self.y * c)
    }
}

impl Vec3 {
    pub const BLACK: Vec3 = vec3(0.0, 0.0, 0.0);
    pub const WHITE: Vec3 = vec3(1.0, 1.0, 1.0);
    pub const fn splat(v: f32) -> Vec3 {
        vec3(v, v, v)
    }
    pub fn from_u8(c: [u8; 3]) -> Vec3 {
        vec3(
            c[0] as f32 / 255.0,
            c[1] as f32 / 255.0,
            c[2] as f32 / 255.0,
        )
    }
    pub fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn dot(self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    pub fn mix(self, o: Vec3, t: f32) -> Vec3 {
        self + (o - self) * t
    }
    pub fn clamp01(self) -> Vec3 {
        vec3(sat(self.x), sat(self.y), sat(self.z))
    }
    pub fn luma(self) -> f32 {
        0.2126 * self.x + 0.7152 * self.y + 0.0722 * self.z
    }
    pub fn powf(self, p: f32) -> Vec3 {
        vec3(
            self.x.max(0.0).powf(p),
            self.y.max(0.0).powf(p),
            self.z.max(0.0).powf(p),
        )
    }
    /// Screen blend: brightens without clipping.
    pub fn screen(self, o: Vec3) -> Vec3 {
        Vec3::WHITE - (Vec3::WHITE - self) * (Vec3::WHITE - o)
    }
}

/// GLSL `mod`: result has the sign of `y`.
pub fn fmod(x: f32, y: f32) -> f32 {
    x - y * (x / y).floor()
}

/// Hex lattice: local coords inside the cell and a cell centre (as id).
pub fn hex_cell(p: Vec2) -> (Vec2, Vec2) {
    let r = vec2(1.0, 1.732_050_8);
    let h = r * 0.5;
    let a = vec2(fmod(p.x, r.x), fmod(p.y, r.y)) - h;
    let q = p - h;
    let b = vec2(fmod(q.x, r.x), fmod(q.y, r.y)) - h;
    let g = if a.dot(a) < b.dot(b) { a } else { b };
    (g, p - g)
}
/// Distance to the hex edge from local coords (`0` at the border, grows inward... as `0.5 - d`).
pub fn hex_dist(g: Vec2) -> f32 {
    let g = g.abs();
    g.dot(vec2(0.5, 0.866_025_4)).max(g.x)
}

pub fn sat(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}
pub fn fract(x: f32) -> f32 {
    x - x.floor()
}
pub fn mix(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
pub fn step(edge: f32, x: f32) -> f32 {
    if x < edge {
        0.0
    } else {
        1.0
    }
}
pub fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = sat((x - a) / (b - a));
    t * t * (3.0 - 2.0 * t)
}

/// Hash `u32` → `[0,1)`.
pub fn hash_u(mut x: u32) -> f32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    (x >> 8) as f32 / 16_777_216.0
}
pub fn hash1(n: f32) -> f32 {
    hash_u(n.to_bits().wrapping_mul(0x9E37_79B1))
}
pub fn hash2(x: i32, y: i32) -> f32 {
    hash_u(
        (x as u32).wrapping_mul(0x85eb_ca6b) ^ (y as u32).wrapping_mul(0xc2b2_ae35) ^ 0x27d4_eb2f,
    )
}
pub fn hash3(x: i32, y: i32, z: i32) -> f32 {
    hash_u(
        (x as u32).wrapping_mul(0x85eb_ca6b)
            ^ (y as u32).wrapping_mul(0xc2b2_ae35)
            ^ (z as u32).wrapping_mul(0x27d4_eb2f),
    )
}

/// Smooth value noise in `[0,1)`.
pub fn noise(p: Vec2) -> f32 {
    let i = p.floor();
    let f = p - i;
    let u = vec2(f.x * f.x * (3.0 - 2.0 * f.x), f.y * f.y * (3.0 - 2.0 * f.y));
    let (ix, iy) = (i.x as i32, i.y as i32);
    let a = hash2(ix, iy);
    let b = hash2(ix + 1, iy);
    let c = hash2(ix, iy + 1);
    let d = hash2(ix + 1, iy + 1);
    mix(mix(a, b, u.x), mix(c, d, u.x), u.y)
}

/// Fractal Brownian motion, `oct` octaves, roughly `[0,1)`.
pub fn fbm(mut p: Vec2, oct: u32) -> f32 {
    let (mut a, mut s, mut n) = (0.5, 0.0, 0.0);
    for _ in 0..oct {
        s += a * noise(p);
        n += a;
        p = p * 2.03 + vec2(17.1, 9.2);
        a *= 0.5;
    }
    s / n
}

/// Inigo Quilez cosine palette.
pub fn palette(t: f32, a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Vec3 {
    let f = |a: f32, b: f32, c: f32, d: f32| a + b * (std::f32::consts::TAU * (c * t + d)).cos();
    vec3(
        f(a.x, b.x, c.x, d.x),
        f(a.y, b.y, c.y, d.y),
        f(a.z, b.z, c.z, d.z),
    )
}

/// HSV (all `0..1`) to RGB.
pub fn hsv(h: f32, s: f32, v: f32) -> Vec3 {
    let k = |n: f32| {
        let p = (fmod(h * 6.0 + n, 6.0) - 3.0).abs();
        v * (1.0 - s + s * sat(p - 1.0))
    };
    vec3(k(0.0), k(4.0), k(2.0))
}

/// 4×4 Bayer threshold in `[0,1)`.
pub fn bayer4(x: i32, y: i32) -> f32 {
    const M: [[u32; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    (M[(y & 3) as usize][(x & 3) as usize] as f32 + 0.5) / 16.0
}

/// Signed distance to a box centred at origin with half-size `b`.
pub fn sd_box(p: Vec2, b: Vec2) -> f32 {
    let d = p.abs() - b;
    vec2(d.x.max(0.0), d.y.max(0.0)).len() + d.x.max(d.y).min(0.0)
}
/// Rounded box, corner radius `r`.
pub fn sd_round_box(p: Vec2, b: Vec2, r: f32) -> f32 {
    sd_box(p, b - r) - r
}
pub fn sd_circle(p: Vec2, r: f32) -> f32 {
    p.len() - r
}
/// Distance from `p` to segment `a..b`.
pub fn sd_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = sat(pa.dot(ba) / ba.dot(ba).max(1e-9));
    (pa - ba * h).len()
}
/// Soft glow falloff around an SDF edge: 1 on the edge, 0 far away.
pub fn glow(d: f32, k: f32) -> f32 {
    1.0 / (1.0 + (d.abs() * k).powi(2))
}
/// Anti-aliased fill from a signed distance (`px` = one pixel in the same units).
pub fn fill_aa(d: f32, px: f32) -> f32 {
    sat(0.5 - d / px.max(1e-6))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_are_unit_and_stable() {
        for i in 0..200 {
            let h = hash2(i, -i);
            assert!((0.0..1.0).contains(&h));
            assert_eq!(h, hash2(i, -i));
        }
        assert_ne!(hash2(1, 2), hash2(2, 1));
    }

    #[test]
    fn noise_continuous_and_bounded() {
        let a = noise(vec2(3.2, 1.1));
        let b = noise(vec2(3.2001, 1.1));
        assert!((a - b).abs() < 0.01);
        assert!((0.0..=1.0).contains(&fbm(vec2(9.9, 4.4), 5)));
    }

    #[test]
    fn smoothstep_edges() {
        assert_eq!(smoothstep(0.0, 1.0, -1.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 2.0), 1.0);
        assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn hsv_primaries() {
        let r = hsv(0.0, 1.0, 1.0);
        assert!((r.x - 1.0).abs() < 1e-5 && r.y < 1e-5 && r.z < 1e-5);
    }

    #[test]
    fn bayer_covers_range() {
        let mut v: Vec<f32> = (0..16).map(|i| bayer4(i % 4, i / 4)).collect();
        v.sort_by(f32::total_cmp);
        assert!(v[0] < 0.05 && v[15] > 0.95);
    }
}
