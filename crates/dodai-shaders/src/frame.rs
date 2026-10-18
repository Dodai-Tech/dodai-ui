//! Frames, the shader context and the CPU renderer.

use crate::math::*;
use dodai_core::{Color, Theme};

pub use crate::ctx::{Ctx, ShaderPalette};

fn col(c: Color) -> Vec3 {
    Vec3::from_u8(c.rgb())
}

impl ShaderPalette {
    pub fn from_theme(t: &Theme) -> Self {
        let p = &t.palette;
        Self {
            bg: col(p.background),
            fg: col(p.text),
            a: col(p.primary),
            b: col(p.secondary),
            c: col(p.accent),
        }
    }
}

/// Float image in display (sRGB) space; palettes are authored that way, so no
/// gamma step happens on output.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub w: u32,
    pub h: u32,
    pub px: Vec<Vec3>,
}

impl Frame {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            px: vec![Vec3::BLACK; (w * h) as usize],
        }
    }
    pub fn get(&self, x: i32, y: i32) -> Vec3 {
        let x = x.clamp(0, self.w as i32 - 1) as u32;
        let y = y.clamp(0, self.h as i32 - 1) as u32;
        self.px[(y * self.w + x) as usize]
    }
    /// Nearest sample at `uv` (`0..1`).
    pub fn nearest(&self, uv: Vec2) -> Vec3 {
        self.get(
            (uv.x * self.w as f32).floor() as i32,
            (uv.y * self.h as f32).floor() as i32,
        )
    }
    /// Bilinear sample at `uv` (`0..1`), clamped at the edges.
    pub fn sample(&self, uv: Vec2) -> Vec3 {
        let x = uv.x * self.w as f32 - 0.5;
        let y = uv.y * self.h as f32 - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let (x0, y0) = (x0 as i32, y0 as i32);
        let top = self.get(x0, y0).mix(self.get(x0 + 1, y0), fx);
        let bot = self.get(x0, y0 + 1).mix(self.get(x0 + 1, y0 + 1), fx);
        top.mix(bot, fy)
    }
    fn byte(v: f32) -> u8 {
        (sat(v) * 255.0 + 0.5) as u8
    }
    /// Tightly packed sRGB RGBA bytes.
    pub fn to_rgba8(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(self.px.len() * 4);
        for p in &self.px {
            o.extend([Self::byte(p.x), Self::byte(p.y), Self::byte(p.z), 255]);
        }
        o
    }
    /// Tightly packed sRGB BGRA bytes (what GPUI's `RenderImage` wants).
    pub fn to_bgra8(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(self.px.len() * 4);
        for p in &self.px {
            o.extend([Self::byte(p.z), Self::byte(p.y), Self::byte(p.x), 255]);
        }
        o
    }
    /// Binary PPM (P6), handy for dumping frames without an image crate.
    /// Terminal rendering: each cell shows two vertically stacked pixels as
    /// `▀` with fg = top and bg = bottom. Returns `cols × rows` cells,
    /// row-major, resampling the frame to `cols × rows*2`.
    pub fn to_half_blocks(&self, cols: u32, rows: u32) -> Vec<([u8; 3], [u8; 3])> {
        let q = |c: Vec3| [c.x, c.y, c.z].map(|v| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
        let mut out = Vec::with_capacity((cols * rows) as usize);
        for r in 0..rows {
            for c in 0..cols {
                let u = (c as f32 + 0.5) / cols as f32;
                let top = self.nearest(vec2(
                    u,
                    (r * 2) as f32 / (rows * 2) as f32 + 0.25 / rows as f32,
                ));
                let bot = self.nearest(vec2(
                    u,
                    (r * 2 + 1) as f32 / (rows * 2) as f32 + 0.25 / rows as f32,
                ));
                out.push((q(top), q(bot)));
            }
        }
        out
    }

    pub fn to_ppm(&self) -> Vec<u8> {
        let mut o = format!("P6\n{} {}\n255\n", self.w, self.h).into_bytes();
        for p in &self.px {
            o.extend([Self::byte(p.x), Self::byte(p.y), Self::byte(p.z)]);
        }
        o
    }
    /// Mean luma, for tests and auto-exposure.
    pub fn mean_luma(&self) -> f32 {
        self.px.iter().map(|p| p.luma()).sum::<f32>() / self.px.len().max(1) as f32
    }
}

/// Source colour per pixel; `uv` is `0..1`, y down.
pub type GenFn = fn(Vec2, &Ctx) -> Vec3;
/// Post effect: reads the incoming frame.
pub type FxFn = fn(&Frame, Vec2, &Ctx) -> Vec3;

fn par_rows(w: u32, h: u32, f: impl Fn(u32, u32) -> Vec3 + Sync) -> Frame {
    let mut fr = Frame::new(w, h);
    if w == 0 || h == 0 {
        return fr;
    }
    let threads = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(16);
    let band = 4 * w as usize;
    // Pixel work is very uneven (fireworks, bloom): threads pull small row
    // bands from a shared queue instead of owning a fixed slab.
    let queue = std::sync::Mutex::new(fr.px.chunks_mut(band).enumerate());
    let run = || loop {
        let next = queue.lock().ok().and_then(|mut q| q.next());
        let Some((bi, chunk)) = next else { break };
        for (i, p) in chunk.iter_mut().enumerate() {
            let idx = (bi * band + i) as u32;
            *p = f(idx % w, idx / w);
        }
    };
    if (w as usize * h as usize) < 2048 || threads == 1 {
        run();
    } else {
        std::thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(run);
            }
        });
    }
    drop(queue);
    fr
}

/// Run a generator over a `ctx.w × ctx.h` frame.
pub fn render_gen(f: GenFn, ctx: &Ctx) -> Frame {
    let (w, h) = (ctx.w, ctx.h);
    par_rows(w, h, |x, y| {
        let uv = vec2((x as f32 + 0.5) / w as f32, (y as f32 + 0.5) / h as f32);
        f(uv, ctx).clamp01_soft()
    })
}

/// Run a post effect over `src`.
pub fn apply_fx(f: FxFn, src: &Frame, ctx: &Ctx) -> Frame {
    let (w, h) = (src.w, src.h);
    let ctx = Ctx { w, h, ..*ctx };
    par_rows(w, h, |x, y| {
        let uv = vec2((x as f32 + 0.5) / w as f32, (y as f32 + 0.5) / h as f32);
        f(src, uv, &ctx).clamp01_soft()
    })
}

impl Vec3 {
    /// Guard against NaN and negatives; allows >1 for later bloom.
    fn clamp01_soft(self) -> Vec3 {
        let g = |v: f32| {
            if v.is_finite() {
                v.clamp(0.0, 4.0)
            } else {
                0.0
            }
        };
        vec3(g(self.x), g(self.y), g(self.z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(_: Vec2, _: &Ctx) -> Vec3 {
        vec3(0.5, 0.25, 1.0)
    }
    fn invert(s: &Frame, uv: Vec2, _: &Ctx) -> Vec3 {
        Vec3::WHITE - s.nearest(uv)
    }
    fn ramp(uv: Vec2, _: &Ctx) -> Vec3 {
        Vec3::splat(uv.x)
    }

    #[test]
    fn renders_every_pixel() {
        let ctx = Ctx::new(33, 17, 0.0, ShaderPalette::default());
        let f = render_gen(flat, &ctx);
        assert_eq!(f.px.len(), 33 * 17);
        assert!(f.px.iter().all(|p| *p == vec3(0.5, 0.25, 1.0)));
    }

    #[test]
    fn rows_land_in_place() {
        let ctx = Ctx::new(40, 9, 0.0, ShaderPalette::default());
        let f = render_gen(ramp, &ctx);
        assert!(f.get(0, 5).x < f.get(39, 5).x);
        assert_eq!(f.get(7, 0), f.get(7, 8));
    }

    #[test]
    fn fx_reads_source() {
        let ctx = Ctx::new(8, 8, 0.0, ShaderPalette::default());
        let a = render_gen(flat, &ctx);
        let b = apply_fx(invert, &a, &ctx);
        assert_eq!(b.get(3, 3), vec3(0.5, 0.75, 0.0));
    }

    #[test]
    fn bilinear_between_texels() {
        let ctx = Ctx::new(2, 1, 0.0, ShaderPalette::default());
        let f = render_gen(ramp, &ctx);
        let m = f.sample(vec2(0.5, 0.5));
        assert!(m.x > f.get(0, 0).x && m.x < f.get(1, 0).x);
    }

    #[test]
    fn byte_formats() {
        let f = Frame::new(2, 2);
        assert_eq!(f.to_rgba8().len(), 16);
        assert_eq!(f.to_bgra8()[3], 255);
        assert!(f.to_ppm().starts_with(b"P6\n2 2\n255\n"));
    }
}
