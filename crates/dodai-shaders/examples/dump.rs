//! `cargo run -p dodai-shaders --example dump -- <out-dir> [time]`
//! Writes every shader as a PPM so it can be inspected without a window.

use dodai_shaders::*;

fn main() {
    let mut a = std::env::args().skip(1);
    let dir = a.next().unwrap_or_else(|| "shader-dump".into());
    let time: f32 = a.next().and_then(|t| t.parse().ok()).unwrap_or(1.5);
    std::fs::create_dir_all(&dir).unwrap();
    for s in catalog() {
        let ctx = Ctx::new(320, 180, time, s.family.palette());
        let f = if s.is_generator() {
            render(&[s], &ctx)
        } else {
            render(&[by_id(s.family.default_source()).unwrap(), s], &ctx)
        };
        std::fs::write(format!("{dir}/{}.ppm", s.id), f.to_ppm()).unwrap();
    }
    println!("wrote {} shaders to {dir}", catalog().len());
}
