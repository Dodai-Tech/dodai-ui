//! Every shader (generators, and post effects after their family's source) renders on the CPU
//! to nearly the same picture as on the GPU. Skipped without a real adapter.

use dodai_shaders::{catalog, cpu, gpu::Gpu};

/// (mean absolute difference per channel in levels, share of channels off by more than 24).
fn diff(a: &[u8], b: &[u8]) -> (f64, f64) {
    let (mut sum, mut big) = (0u64, 0usize);
    for (x, y) in a.iter().zip(b) {
        let d = (*x as i32 - *y as i32).unsigned_abs();
        sum += d as u64;
        big += (d > 24) as usize;
    }
    (sum as f64 / a.len() as f64, big as f64 / a.len() as f64)
}

#[test]
fn cpu_and_gpu_render_the_same_picture() {
    let Some(mut gpu) = Gpu::new() else {
        eprintln!("no GPU adapter: skipped");
        return;
    };
    let (w, h) = (96, 54);
    let mut worst = (String::new(), 0.0f64);
    let mut bad = Vec::new();
    for info in catalog() {
        let pal = info.family.palette();
        let chain: Vec<&str> = if info.is_generator() {
            vec![info.id]
        } else {
            vec![info.family.default_source(), info.id]
        };
        let g = gpu.render(&chain, w, h, 1.3, 1.0, &pal).unwrap().to_vec();
        let c = cpu::render(&chain, w, h, 1.3, 1.0, &pal).unwrap();
        assert_eq!(g.len(), c.len());
        let (mean, big) = diff(&g, &c);
        if mean > worst.1 {
            worst = (info.id.to_string(), mean);
        }
        // Hard edges and chaotic shaders may differ in a few pixels, never in most.
        if mean > 6.0 || big > 0.08 {
            bad.push(format!(
                "{} (mean {mean:.2}, off {:.1}%)",
                info.id,
                big * 100.0
            ));
        }
    }
    eprintln!("worst mean difference: {} {:.3}", worst.0, worst.1);
    assert!(bad.is_empty(), "CPU and GPU disagree: {bad:#?}");
}
