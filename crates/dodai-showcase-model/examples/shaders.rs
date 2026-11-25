//! Writes a thumbnail of every shader (rendered on the GPU) and its catalogue entry, for the
//! design file: `cargo run --release -p dodai-showcase-model --example shaders -- dist/shaders`.
//!
//! Each shader is drawn the way the Shaders page first shows it: a post effect over its family's
//! default source, in the palette of each design system's default theme (`shaders-<family>.json`).

use std::fmt::Write as _;
use std::path::PathBuf;

use base64::Engine as _;
use dodai_core::themes;
use dodai_shaders::{by_id, catalog, gpu::Gpu, Kind, ShaderPalette};

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn png_b64(rgba: &[u8], w: u32, h: u32) -> String {
    let mut bytes = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut bytes, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(png::Compression::High);
        let mut writer = enc.write_header().expect("png header");
        writer.write_image_data(rgba).expect("png data");
    }
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn main() {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "dist/shaders".into()),
    );
    std::fs::create_dir_all(&out).expect("create output dir");
    let mut gpu = Gpu::new().expect("no GPU");
    let (w, h) = (192u32, 108u32);
    for sys in themes::systems() {
        let all = (sys.themes)();
        let pick = all
            .iter()
            .position(|t| t.meta.id == sys.family)
            .unwrap_or(0);
        let pal = ShaderPalette::from_theme(&all[pick]);
        let mut rows = Vec::new();
        for s in catalog() {
            let chain: Vec<&str> = if s.is_generator() {
                vec![s.id]
            } else {
                vec![by_id(s.family.default_source()).expect("source").id, s.id]
            };
            let bgra = gpu.render(&chain, w, h, 1.5, 1.0, &pal).expect("render");
            let mut rgba = Vec::with_capacity(bgra.len());
            for p in bgra.chunks(4) {
                rgba.extend([p[2], p[1], p[0], 255]);
            }
            let kind = match s.kind {
                Kind::Generator => "generator",
                Kind::PostFx => "effect",
            };
            let mut row = String::new();
            let _ = write!(
                row,
                "{{\"id\":\"{}\",\"name\":\"{}\",\"family\":\"{}\",\"kind\":\"{kind}\",\"desc\":\"{}\",\"png\":\"{}\"}}",
                s.id,
                esc(s.name),
                esc(s.family.name()),
                esc(s.desc),
                png_b64(&rgba, w, h)
            );
            rows.push(row);
        }
        std::fs::write(
            out.join(format!("shaders-{}.json", sys.family)),
            format!("[{}]\n", rows.join(",")),
        )
        .expect("write");
        println!("wrote {} shaders for {}", rows.len(), sys.family);
    }
}
