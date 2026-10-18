//! `cargo run --release -p dodai-shaders --example sheet -- <out.ppm> <w> <h> <t> id[,post...] ...`
use dodai_shaders::*;
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let (w, h, t): (u32, u32, f32) = (
        a[1].parse().unwrap(),
        a[2].parse().unwrap(),
        a[3].parse().unwrap(),
    );
    let mut f = Vec::new();
    for spec in &a[4..] {
        let chain: Vec<_> = spec.split(',').filter_map(by_id).collect();
        f.push(render(
            &chain,
            &Ctx::new(w, h, t, ShaderPalette::bladerunner()),
        ));
    }
    let cols = 2usize;
    let rows = f.len().div_ceil(cols);
    let mut out = format!("P6\n{} {}\n255\n", w as usize * cols, h as usize * rows).into_bytes();
    for r in 0..rows {
        for y in 0..h {
            for cc in 0..cols {
                match f.get(r * cols + cc) {
                    Some(fr) => {
                        for x in 0..w {
                            let p = fr.get(x as i32, y as i32);
                            for v in [p.x, p.y, p.z] {
                                out.push((v.clamp(0.0, 1.0) * 255.0) as u8);
                            }
                        }
                    }
                    None => out.extend(std::iter::repeat_n(0u8, w as usize * 3)),
                }
            }
        }
    }
    std::fs::write(&a[0], out).unwrap();
}
