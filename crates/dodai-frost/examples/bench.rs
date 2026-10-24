//! `cargo run --release -p dodai-frost --features gpu --example bench`: CPU vs GPU blur timings.

use dodai_frost::{blur, Backend, Blurrer};
use std::time::Instant;

fn main() {
    let mut gpu = Blurrer::new(Backend::Gpu);
    for (w, h) in [
        (256usize, 256usize),
        (1280, 720),
        (1920, 1080),
        (3840, 2160),
    ] {
        let src: Vec<u8> = (0..w * h * 4).map(|i| (i * 31 % 255) as u8).collect();
        let t = Instant::now();
        let cpu = blur(&src, w, h, 12.0);
        let cpu_ms = t.elapsed().as_secs_f64() * 1e3;
        gpu.blur(&src, w, h, 12.0); // warm-up
        let t = Instant::now();
        let out = gpu.blur(&src, w, h, 12.0);
        let gpu_ms = t.elapsed().as_secs_f64() * 1e3;
        println!(
            "{w}x{h}: cpu {cpu_ms:7.1} ms   {} {gpu_ms:7.1} ms   identical: {}",
            if gpu.uses_gpu(w * h) {
                "gpu"
            } else {
                "gpu unavailable, cpu"
            },
            out == cpu
        );
    }
}
