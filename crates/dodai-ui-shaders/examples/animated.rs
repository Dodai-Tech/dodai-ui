//! `cargo run -p dodai-ui-shaders --example animated`
//! Lists the shaders whose picture does not change with time (a source that never moves, or an
//! effect that does nothing over a still source).

use dodai_ui_shaders::{catalog, gpu::Gpu};

fn frame(gpu: &mut Gpu, chain: &[&str], t: f32, pal: &dodai_ui_shaders::ShaderPalette) -> Vec<u8> {
    gpu.render(chain, 160, 90, t, 1.0, pal).unwrap().to_vec()
}

fn main() {
    let mut gpu = Gpu::new().expect("no GPU");
    let mut still_sources = Vec::new();
    for s in catalog().iter().filter(|s| s.is_generator()) {
        let p = s.family.palette();
        let a = frame(&mut gpu, &[s.id], 1.0, &p);
        let b = frame(&mut gpu, &[s.id], 2.7, &p);
        let c = frame(&mut gpu, &[s.id], 5.3, &p);
        if a == b && b == c {
            still_sources.push(s.id);
        }
    }
    println!("sources that never change: {still_sources:?}");
    // An effect is still only when it is still over every still source.
    let mut still_fx = Vec::new();
    for s in catalog().iter().filter(|s| !s.is_generator()) {
        let p = s.family.palette();
        let moves = still_sources.iter().any(|base| {
            let a = frame(&mut gpu, &[base, s.id], 1.0, &p);
            [2.7, 5.3]
                .iter()
                .any(|t| frame(&mut gpu, &[base, s.id], *t, &p) != a)
        });
        if !moves {
            still_fx.push(s.id);
        }
    }
    println!("effects that never change over any of them: {still_fx:?}");
}
