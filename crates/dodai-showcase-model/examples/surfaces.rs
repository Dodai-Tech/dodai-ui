//! Writes every surface texture and effect each design draws, its frosted-glass parameters and
//! the backdrop picture, per design system, for the design file:
//! `cargo run -p dodai-showcase-model --example surfaces -- dist/surfaces`.
//!
//! The textures and effects are `dodai_core::surface_svg`, the same vector the web and Slint
//! toolkits show, so the design file draws what the designs draw. Only what the design owns is
//! written (`Theme::surface_textures`, `surface_effects`).

use std::fmt::Write as _;
use std::path::PathBuf;

use base64::Engine as _;
use dodai_core::{
    frost_scene, surface_svg, themes, Color, ResolvedSurface, Surface, SurfaceClass,
    SurfaceTargets, Texture, Theme,
};

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn hex(c: Color) -> String {
    format!("{}{:02x}", c.to_hex(), c.a)
}

fn png_b64(rgba: &[u8], w: u32, h: u32) -> String {
    let mut bytes = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut bytes, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().expect("png header");
        writer.write_image_data(rgba).expect("png data");
    }
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Every effect with the setter that turns it on at full amount.
#[allow(clippy::type_complexity)]
fn effects() -> Vec<(&'static str, Box<dyn Fn(&mut ResolvedSurface)>)> {
    vec![
        ("glitch", Box::new(|f| f.glitch = 1.0)),
        ("sheen", Box::new(|f| f.sheen = 1.0)),
        ("flicker", Box::new(|f| f.flicker = 1.0)),
        ("bloom", Box::new(|f| f.bloom = 1.0)),
        ("pulse", Box::new(|f| f.pulse = 1.0)),
        ("chase", Box::new(|f| f.chase = 1.0)),
        ("edge", Box::new(|f| f.edge = 1.0)),
        ("brackets", Box::new(|f| f.brackets = 1.0)),
        ("depth", Box::new(|f| f.depth = 1.0)),
        ("tint", Box::new(|f| f.tint = 1.0)),
        ("pop", Box::new(|f| f.pop = 1.0)),
        ("tape", Box::new(|f| f.tape = 1.0)),
        ("gloss", Box::new(|f| f.gloss = 1.0)),
        ("strip", Box::new(|f| f.strip = 1.0)),
        ("wash", Box::new(|f| f.wash = 1.0)),
        ("rim", Box::new(|f| f.rim = 1.0)),
    ]
}

fn tile(theme: &Theme, fx: ResolvedSurface) -> Option<String> {
    let fx = ResolvedSurface {
        targets: SurfaceTargets::ALL,
        ..fx
    };
    surface_svg(
        theme,
        SurfaceClass::Button,
        &Surface::INHERIT,
        fx,
        (160.0, 64.0),
        (7, 0.6),
    )
    .map(|(svg, _)| svg)
}

fn default_theme(family: &str, all: Vec<Theme>) -> Theme {
    let pick = all.iter().position(|t| t.meta.id == family).unwrap_or(0);
    all.into_iter()
        .nth(pick)
        .expect("a design system has a theme")
}

fn main() {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "dist/surfaces".into()),
    );
    std::fs::create_dir_all(&out).expect("create output dir");
    for sys in themes::systems() {
        let theme = default_theme(sys.family, (sys.themes)());
        let mut textures = Vec::new();
        for &tex in theme.surface_textures() {
            if tex == Texture::None {
                continue;
            }
            let fx = ResolvedSurface {
                texture: tex,
                strength: 0.8,
                scale: 1.0,
                ..ResolvedSurface::NONE
            };
            if let Some(svg) = tile(&theme, fx) {
                textures.push(format!(
                    "{{\"name\":\"{}\",\"svg\":\"{}\"}}",
                    tex.label(),
                    esc(&svg)
                ));
            }
        }
        let mut fxs = Vec::new();
        for (name, set) in effects() {
            let mut fx = ResolvedSurface::NONE;
            set(&mut fx);
            if let Some(svg) = tile(&theme, fx) {
                fxs.push(format!("{{\"name\":\"{name}\",\"svg\":\"{}\"}}", esc(&svg)));
            }
        }
        let frost = theme.frost();
        let (w, h) = (480usize, 270usize);
        let scene = frost_scene(&theme.palette, w, h);
        let edge = frost
            .edge
            .map(|(c, wd)| format!("[\"{}\",{wd}]", hex(c)))
            .unwrap_or_else(|| "null".into());
        // The glass the toolkits draw (blur, saturation and tint) over the whole scene at three
        // radii, baked, so the design file does not depend on the editor's backdrop blur.
        let panes: Vec<String> = [0.0, frost.blur / 2.0, frost.blur]
            .into_iter()
            .map(|sigma| {
                let params = dodai_frost::FrostParams {
                    tint: [frost.tint.r, frost.tint.g, frost.tint.b, frost.tint.a],
                    blur: sigma,
                    edge: None,
                    saturation: frost.saturation,
                };
                let px = dodai_frost::glass(&scene, w, h, &params);
                format!(
                    "{{\"blur\":{sigma},\"png\":\"{}\"}}",
                    png_b64(&px, w as u32, h as u32)
                )
            })
            .collect();
        let mut text = String::new();
        let _ = writeln!(
            text,
            "{{\"panes\":[{}],\"frost\":{{\"tint\":\"{}\",\"blur\":{},\"saturation\":{},\"edge\":{}}},\"scene\":\"{}\",\"textures\":[{}],\"effects\":[{}]}}",
            panes.join(","),
            hex(frost.tint),
            frost.blur,
            frost.saturation,
            edge,
            png_b64(&scene, w as u32, h as u32),
            textures.join(","),
            fxs.join(",")
        );
        std::fs::write(out.join(format!("{}.json", sys.family)), text).expect("write surfaces");
    }
}
