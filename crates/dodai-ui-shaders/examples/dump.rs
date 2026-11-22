//! `cargo run -p dodai-ui-shaders --example dump -- <out-dir> [time]`
//! Renders every shader on the GPU to a PPM so it can be inspected without a window.

use dodai_ui_shaders::{by_id, catalog, gpu::Gpu};

fn main() {
    let mut a = std::env::args().skip(1);
    let dir = a.next().unwrap_or_else(|| "shader-dump".into());
    let time: f32 = a.next().and_then(|t| t.parse().ok()).unwrap_or(1.5);
    std::fs::create_dir_all(&dir).unwrap();
    let mut gpu = Gpu::new().expect("no GPU");
    let (w, h) = (320u32, 180u32);
    for s in catalog() {
        let chain: Vec<&str> = if s.is_generator() {
            vec![s.id]
        } else {
            vec![by_id(s.family.default_source()).unwrap().id, s.id]
        };
        let bgra = gpu
            .render(&chain, w, h, time, 1.0, &s.family.palette())
            .unwrap();
        let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
        for p in bgra.chunks(4) {
            out.extend([p[2], p[1], p[0]]);
        }
        std::fs::write(format!("{dir}/{}.ppm", s.id), out).unwrap();
    }
    println!("wrote {} shaders to {dir}", catalog().len());
}
