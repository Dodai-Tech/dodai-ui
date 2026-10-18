//! Renders every shader on the CPU and the GPU and compares them.
//! `cargo run --release -p dodai-shaders-gpu --example parity [w h]`
use dodai_shaders::{catalog, render, Ctx, ShaderPalette};
use dodai_shaders_gpu::Gpu;
use std::time::Instant;

fn main() {
    let args: Vec<u32> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let (w, h) = (
        args.first().copied().unwrap_or(320),
        args.get(1).copied().unwrap_or(180),
    );
    let t0 = Instant::now();
    let mut gpu = Gpu::new().expect("no GPU");
    println!("adapter: {} (init {:?})", gpu.adapter, t0.elapsed());
    let pal = ShaderPalette::cyberpunk();
    let (mut worst, mut sum_cpu, mut sum_gpu) = (Vec::new(), 0.0, 0.0);
    for s in catalog() {
        let mut ids: Vec<&str> = Vec::new();
        if !s.is_generator() {
            ids.push(s.family.default_source());
        }
        ids.push(s.id);
        let chain: Vec<_> = ids.iter().filter_map(|i| dodai_shaders::by_id(i)).collect();
        let ctx = Ctx::new(w, h, 2.0, pal).intensity(1.0);
        let c0 = Instant::now();
        let cpu = render(&chain, &ctx).to_bgra8();
        let cpu_ms = c0.elapsed().as_secs_f64() * 1e3;
        // Warm (pipeline creation), then time.
        let t1 = Instant::now();
        gpu.render(&ids, w, h, 2.0, 1.0, &pal).expect("gpu");
        let first = t1.elapsed();
        let g0 = Instant::now();
        let out = gpu
            .render(&ids, w, h, 2.0, 1.0, &pal)
            .expect("gpu")
            .to_vec();
        let gpu_ms = g0.elapsed().as_secs_f64() * 1e3;
        let diff: f64 = cpu
            .chunks(4)
            .zip(out.chunks(4))
            .map(|(a, b)| {
                (0..3)
                    .map(|k| (a[k] as f64 - b[k] as f64).abs())
                    .sum::<f64>()
                    / 3.0
            })
            .sum::<f64>()
            / (w * h) as f64;
        sum_cpu += cpu_ms;
        sum_gpu += gpu_ms;
        println!(
            "{:22} diff {:6.2}  cpu {:7.2} ms  gpu {:6.2} ms  (first {:?})",
            s.id, diff, cpu_ms, gpu_ms, first
        );
        worst.push((diff, s.id));
    }
    worst.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!("\nTotal cpu {sum_cpu:.0} ms, gpu {sum_gpu:.0} ms at {w}x{h}");
    println!("Largest differences: {:?}", &worst[..8.min(worst.len())]);
}
