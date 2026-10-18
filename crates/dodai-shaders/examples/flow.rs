//! Vertical motion test: `cargo run --release -p dodai-shaders --example flow`.
//! Prints, per generator, the dominant vertical shift between two close frames
//! (positive = content moves down the screen).
use dodai_shaders::*;
fn luma(f: &Frame, x: i32, y: i32) -> f32 {
    f.get(x, y).luma()
}
fn main() {
    let (w, h) = (192i32, 144i32);
    let only: Vec<String> = std::env::args().skip(1).collect();
    for s in catalog()
        .iter()
        .filter(|s| s.is_generator() && (only.is_empty() || only.contains(&s.id.to_string())))
    {
        let a = render(
            &[s],
            &Ctx::new(w as u32, h as u32, 3.0, ShaderPalette::cyberpunk()),
        );
        let b = render(
            &[s],
            &Ctx::new(w as u32, h as u32, 3.03, ShaderPalette::cyberpunk()),
        );
        let mut best = (f32::MAX, 0);
        let mut zero = 0.0;
        for dy in -3..=3 {
            let mut e = 0.0;
            for y in 4..h - 4 {
                for x in 0..w {
                    let d = luma(&b, x, y) - luma(&a, x, y - dy);
                    e += d * d;
                }
            }
            if dy == 0 {
                zero = e;
            }
            if e < best.0 {
                best = (e, dy);
            }
        }
        if !only.is_empty() || (best.1 != 0 && best.0 < zero * 0.8) {
            println!(
                "{:<18} dy={} {}",
                s.id,
                best.1,
                if best.1 > 0 {
                    "DOWN"
                } else if best.1 < 0 {
                    "UP"
                } else {
                    "none"
                }
            );
        }
    }
}
