//! GPU kernels for `dodai-shaders`.
//!
//! The shader source files of `dodai-shaders` are included verbatim and
//! compiled by rust-gpu to WGSL, so there is one implementation of every
//! shader. Each shader becomes a compute entry point (`gen_kernel!` /
//! `post_kernel!`, listed in `entries.rs`), plus `pack` for the final BGRA
//! conversion. Build with `just build-kernels`.
//!
//! Bindings (group 0): 0 `Params`, 1 source pixels, 2 destination pixels,
//! 3 packed output.
#![cfg_attr(target_arch = "spirv", no_std)]
#![allow(clippy::all, dead_code, unused_imports, unused_variables)]

#[cfg(target_arch = "spirv")]
extern crate core as std;

use glam::{UVec3, Vec2 as GVec2, Vec4};
use spirv_std::spirv;

#[path = "../../dodai-shaders/src/ctx.rs"]
pub mod ctx;
pub mod frame;
#[path = "../../dodai-shaders/src/math.rs"]
pub mod math;
pub use ctx::ShaderPalette;
pub mod prelude {
    #[cfg(target_arch = "spirv")]
    pub use spirv_std::num_traits::Float;
}

// Registration stubs: the shared sources declare `ShaderInfo` tables for the
// CPU catalogue; here they are inert.
#[derive(Clone, Copy)]
pub enum Family {
    Cyberpunk,
    Material3,
    Matrix,
    Retro,
    Pixel,
    Nature,
    Ghostty,
    Ghibli,
    Dmt,
}
pub struct ShaderInfo;
impl ShaderInfo {
    pub const fn generator<F: Copy>(_: &str, _: &str, _: Family, _: &str, _: F) -> Self {
        Self
    }
    pub const fn post<F: Copy>(_: &str, _: &str, _: Family, _: &str, _: F) -> Self {
        Self
    }
}

#[path = "../../dodai-shaders/src/blade.rs"]
pub mod blade;
#[path = "../../dodai-shaders/src/cyberpunk.rs"]
pub mod cyberpunk;
#[path = "../../dodai-shaders/src/dmt.rs"]
pub mod dmt;
#[path = "../../dodai-shaders/src/extra.rs"]
pub mod extra;
#[path = "../../dodai-shaders/src/fx.rs"]
pub mod fx;
#[path = "../../dodai-shaders/src/ghibli.rs"]
pub mod ghibli;
#[path = "../../dodai-shaders/src/ghibli2.rs"]
pub mod ghibli2;
#[path = "../../dodai-shaders/src/ghostty.rs"]
pub mod ghostty;
#[path = "../../dodai-shaders/src/material.rs"]
pub mod material;
#[path = "../../dodai-shaders/src/matrix.rs"]
pub mod matrix;
#[path = "../../dodai-shaders/src/nature.rs"]
pub mod nature;
#[path = "../../dodai-shaders/src/pixel.rs"]
pub mod pixel;
#[path = "../../dodai-shaders/src/pixel2.rs"]
pub mod pixel2;
#[path = "../../dodai-shaders/src/retro.rs"]
pub mod retro;
#[path = "../../dodai-shaders/src/retro2.rs"]
pub mod retro2;

/// Per-dispatch constants. Mirrored byte for byte by the host.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Params {
    pub res: GVec2,
    pub time: f32,
    pub intensity: f32,
    pub bg: Vec4,
    pub fg: Vec4,
    pub a: Vec4,
    pub b: Vec4,
    pub c: Vec4,
}

impl Params {
    pub fn ctx(&self) -> ctx::Ctx {
        let v = |p: Vec4| math::vec3(p.x, p.y, p.z);
        ctx::Ctx {
            time: self.time,
            w: self.res.x as u32,
            h: self.res.y as u32,
            intensity: self.intensity,
            pal: ctx::ShaderPalette {
                bg: v(self.bg),
                fg: v(self.fg),
                a: v(self.a),
                b: v(self.b),
                c: v(self.c),
            },
        }
    }
}

macro_rules! gen_kernel {
    ($name:ident, $f:path) => {
        #[spirv(compute(threads(8, 8)))]
        pub fn $name(
            #[spirv(global_invocation_id)] id: UVec3,
            #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] p: &Params,
            #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] dst: &mut [Vec4],
        ) {
            let (w, h) = (p.res.x as u32, p.res.y as u32);
            if id.x < w && id.y < h {
                let uv = math::vec2(
                    (id.x as f32 + 0.5) / w as f32,
                    (id.y as f32 + 0.5) / h as f32,
                );
                let c = $f(uv, &p.ctx());
                dst[(id.y * w + id.x) as usize] = Vec4::new(c.x, c.y, c.z, 1.0);
            }
        }
    };
}

macro_rules! post_kernel {
    ($name:ident, $f:path) => {
        #[spirv(compute(threads(8, 8)))]
        pub fn $name(
            #[spirv(global_invocation_id)] id: UVec3,
            #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] p: &Params,
            #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] src: &[Vec4],
            #[spirv(storage_buffer, descriptor_set = 0, binding = 2)] dst: &mut [Vec4],
        ) {
            let (w, h) = (p.res.x as u32, p.res.y as u32);
            if id.x < w && id.y < h {
                let uv = math::vec2(
                    (id.x as f32 + 0.5) / w as f32,
                    (id.y as f32 + 0.5) / h as f32,
                );
                let frame = frame::Frame::new(w, h, src);
                let c = $f(&frame, uv, &p.ctx());
                dst[(id.y * w + id.x) as usize] = Vec4::new(c.x, c.y, c.z, 1.0);
            }
        }
    };
}

/// Converts the final float image to packed BGRA8 (`b | g << 8 | r << 16 | a << 24`).
#[spirv(compute(threads(8, 8)))]
pub fn pack(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] p: &Params,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] src: &[Vec4],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 3)] out: &mut [u32],
) {
    let (w, h) = (p.res.x as u32, p.res.y as u32);
    if id.x < w && id.y < h {
        let i = (id.y * w + id.x) as usize;
        let c = src[i];
        let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
        out[i] = q(c.z) | q(c.y) << 8 | q(c.x) << 16 | 0xff << 24;
    }
}

include!("entries.rs");
