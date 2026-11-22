//! Frame latency while every pipeline compiles in the background.
//! `cargo run --release -p dodai-ui-shaders --example warm`
use dodai_ui_shaders::gpu::Gpu;
use dodai_ui_shaders::ShaderPalette;
use std::time::Instant;
fn main() {
    let mut gpu = Gpu::new().expect("no GPU");
    let pal = ShaderPalette::cyberpunk();
    let t0 = Instant::now();
    gpu.render(&["plasma"], 1280, 720, 0.0, 1.0, &pal).unwrap();
    println!("first frame (cold): {:?}", t0.elapsed());
    gpu.prewarm_all();
    let start = Instant::now();
    let mut worst = 0.0f64;
    let mut n = 0;
    while start.elapsed().as_secs_f32() < 6.0 {
        let t = Instant::now();
        gpu.render(&["plasma", "crt"], 1280, 720, n as f32 * 0.016, 1.0, &pal)
            .unwrap();
        worst = worst.max(t.elapsed().as_secs_f64() * 1e3);
        n += 1;
    }
    println!("{n} frames in 6 s while warming, worst frame {worst:.1} ms");
    let t = Instant::now();
    gpu.render(&["gh_laputa", "gh_dream"], 1280, 720, 1.0, 1.0, &pal)
        .unwrap();
    println!("a shader first used after warm-up: {:?}", t.elapsed());
}
