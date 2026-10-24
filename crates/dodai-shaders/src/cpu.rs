//! The same shaders on the CPU, for machines without a real GPU.
//!
//! [`render`] runs the shader functions the GPU kernels are built from, one pixel at a time,
//! spread over all cores. Output matches the GPU to within a few levels of 255 (floating-point
//! differences, which only show on hard edges). It is the fallback of [`crate::Renderer`]; there
//! is no reason to pick it when a GPU exists.

use crate::frame::Frame;
use crate::math::{vec2, Vec3};
use crate::{check_chain, Ctx, Func, RenderError, ShaderPalette};

/// Renders `chain` (a generator id, then post-effect ids) and returns tightly packed **BGRA8**
/// bytes, `w * h * 4` long.
pub fn render(
    chain: &[&str],
    w: u32,
    h: u32,
    time: f32,
    intensity: f32,
    pal: &ShaderPalette,
) -> Result<Vec<u8>, RenderError> {
    let shaders = check_chain(chain)?;
    let (w, h) = (w.max(1), h.max(1));
    let ctx = Ctx::new(w, h, time, *pal).intensity(intensity.clamp(0.0, 1.0));
    let mut cur: Vec<Vec3> = Vec::new();
    for info in shaders {
        let mut next = vec![Vec3::default(); (w * h) as usize];
        let prev = Frame::new(w, h, &cur);
        pass(&mut next, w, |uv| match info.func {
            Func::Gen(f) => f(uv, &ctx),
            Func::Post(f) => f(&prev, uv, &ctx),
        });
        cur = next;
    }
    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    Ok(cur
        .iter()
        .flat_map(|c| [q(c.z), q(c.y), q(c.x), 255])
        .collect())
}

/// Fills `out` (`w` pixels per row) by calling `f(uv)` for each pixel centre, one band of rows
/// per core.
fn pass(out: &mut [Vec3], w: u32, f: impl Fn(crate::Vec2) -> Vec3 + Sync) {
    let h = out.len() as u32 / w;
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let band = (h as usize).div_ceil(threads).max(1) * w as usize;
    std::thread::scope(|s| {
        for (bi, chunk) in out.chunks_mut(band).enumerate() {
            let f = &f;
            s.spawn(move || {
                for (i, px) in chunk.iter_mut().enumerate() {
                    let n = (bi * band + i) as u32;
                    let (x, y) = ((n % w) as f32, (n / w) as f32);
                    *px = f(vec2((x + 0.5) / w as f32, (y + 0.5) / h as f32));
                }
            });
        }
    });
}
