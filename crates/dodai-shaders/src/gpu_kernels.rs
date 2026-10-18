// GPU entry points, compiled only for the rust-gpu target. Included into the
// crate root so entry names stay flat.
//
// Bindings (group 0): 0 `Params`, 1 source pixels, 2 destination pixels,
// 3 packed output.
use glam::{UVec3, Vec2 as GVec2, Vec4};
use spirv_std::spirv;

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
    pub fn ctx(&self) -> crate::ctx::Ctx {
        let v = |p: Vec4| crate::math::vec3(p.x, p.y, p.z);
        crate::ctx::Ctx {
            time: self.time,
            w: self.res.x as u32,
            h: self.res.y as u32,
            intensity: self.intensity,
            pal: crate::ctx::ShaderPalette {
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
                let uv = crate::math::vec2(
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
                let uv = crate::math::vec2(
                    (id.x as f32 + 0.5) / w as f32,
                    (id.y as f32 + 0.5) / h as f32,
                );
                let frame = crate::frame::Frame::new(w, h, src);
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
