//! Experiment: do the shader sources run as plain Rust on the CPU, and does that match the GPU?
//!
//! The shaders are ordinary functions, so the same source the GPU kernels are built from is
//! called here per pixel. Prints timing and the difference to the GPU's BGRA output.

use dodai_shaders::gpu::Gpu;
use dodai_shaders_kernels::{
    blade,
    ctx::{Ctx, ShaderPalette},
    cyberpunk,
    math::{vec2, Vec3},
};
use std::time::Instant;

type Gen = fn(dodai_shaders_kernels::math::Vec2, &Ctx) -> Vec3;

fn cpu(f: Gen, w: u32, h: u32, time: f32, pal: ShaderPalette) -> Vec<u8> {
    let ctx = Ctx::new(w, h, time, pal);
    let mut out = vec![0u8; (w * h * 4) as usize];
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let rows = (h as usize).div_ceil(threads);
    std::thread::scope(|s| {
        for (ci, chunk) in out.chunks_mut(rows * w as usize * 4).enumerate() {
            let ctx = &ctx;
            s.spawn(move || {
                for (i, px) in chunk.chunks_exact_mut(4).enumerate() {
                    let n = ci * rows * w as usize + i;
                    let (x, y) = ((n as u32 % w) as f32, (n as u32 / w) as f32);
                    let c = f(vec2((x + 0.5) / w as f32, (y + 0.5) / h as f32), ctx);
                    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                    px.copy_from_slice(&[q(c.z), q(c.y), q(c.x), 255]);
                }
            });
        }
    });
    out
}

fn main() {
    let mut gpu = Gpu::new();
    println!("gpu: {}", if gpu.is_some() { "yes" } else { "no adapter" });
    let table: [(&str, Gen); 6] = [
        ("neon_grid", cyberpunk::neon_grid),
        ("hex_pulse", cyberpunk::hex_pulse),
        ("circuit", cyberpunk::circuit),
        ("hud_radar", cyberpunk::hud_radar),
        ("neon_city", blade::neon_city),
        ("br_city", blade::br_city),
    ];
    let pal = dodai_shaders::ShaderPalette::cyberpunk();
    let kpal = ShaderPalette {
        bg: vec3(pal.bg),
        fg: vec3(pal.fg),
        a: vec3(pal.a),
        b: vec3(pal.b),
        c: vec3(pal.c),
    };
    for (w, h) in [(320u32, 180u32), (1280, 720)] {
        for (id, f) in table {
            let t = Instant::now();
            let c = cpu(f, w, h, 1.7, kpal);
            let cpu_ms = t.elapsed().as_secs_f64() * 1e3;
            let Some(g) = gpu.as_mut() else {
                println!("{id} {w}x{h}: cpu {cpu_ms:.1} ms");
                continue;
            };
            let _ = g.render(&[id], w, h, 1.7, 1.0, &pal);
            let t = Instant::now();
            let gb = g.render(&[id], w, h, 1.7, 1.0, &pal).unwrap().to_vec();
            let gpu_ms = t.elapsed().as_secs_f64() * 1e3;
            let (mut max, mut sum, mut bad) = (0i32, 0i64, 0usize);
            for (a, b) in c.iter().zip(&gb) {
                let d = (*a as i32 - *b as i32).abs();
                max = max.max(d);
                sum += d as i64;
                bad += (d > 8) as usize;
            }
            println!(
                "{id:10} {w}x{h}: cpu {cpu_ms:7.1} ms | gpu {gpu_ms:5.1} ms | max diff {max:3} mean {:.3} | >8: {:.3}%",
                sum as f64 / c.len() as f64,
                bad as f64 * 100.0 / c.len() as f64
            );
        }
    }
}

fn vec3(v: dodai_shaders::math::Vec3) -> Vec3 {
    dodai_shaders_kernels::math::vec3(v.x, v.y, v.z)
}
