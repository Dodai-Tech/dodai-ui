//! # dodai-shaders
//!
//! Pure-Rust pixel shaders for the Dodai design system and beyond. A shader
//! is a plain function evaluated per pixel (`fn(uv, &Ctx) -> Vec3`), rendered
//! on the CPU across threads into a [`Frame`] that any toolkit can show as an
//! image: no GPU API, no shader language, no build step. The same sources also
//! run on the GPU through the `gpu` feature (rust-gpu), which is why the shader
//! files avoid iterators over arrays, slices of constants, strings and
//! `u8`/`u64` values.
//!
//! * **Generators** make a picture from nothing (`neon_grid`, `digital_rain`).
//! * **Post effects** transform a [`Frame`] (`crt`, `vhs`, `gameboy`).
//! * Both read colours from a [`ShaderPalette`], so they follow the theme.
//!
//! ```
//! use dodai_shaders::{catalog, render, Ctx, ShaderPalette};
//! let info = catalog().iter().find(|s| s.id == "neon_grid").unwrap();
//! let frame = render(&[info], &Ctx::new(64, 36, 0.0, ShaderPalette::cyberpunk()));
//! assert_eq!(frame.px.len(), 64 * 36);
//! ```
//!
//! Families: [`Family::Cyberpunk`], [`Family::Material3`], [`Family::Matrix`],
//! [`Family::Retro`] (CRT/VHS/terminals), [`Family::Pixel`] (platformers and
//! metroidvanias) and [`Family::Nature`] (cosmic, water, fire, abstract).
// Index loops, `match` ladders and range comparisons are deliberate: the GPU
// build (rust-gpu) cannot lower the iterator / `contains` forms.
#![allow(
    clippy::needless_range_loop,
    clippy::manual_range_contains,
    clippy::manual_memcpy
)]

pub mod blade;
pub mod ctx;
pub mod cyberpunk;
pub mod dmt;
pub mod extra;
pub mod frame;
pub mod fx;
pub mod ghibli;
pub mod ghibli2;
pub mod ghostty;
#[cfg(feature = "gpu")]
pub mod gpu;
pub mod material;
pub mod math;
pub mod matrix;
pub mod nature;
pub mod pixel;
pub mod pixel2;
pub mod player;
pub mod prelude;
pub mod presets;
pub mod retro;
pub mod retro2;

pub use frame::{apply_fx, render_gen, Ctx, Frame, FxFn, GenFn, ShaderPalette};
pub use math::{vec2, vec3, Vec2, Vec3};
pub use player::Player;
pub use presets::Preset;

use std::sync::OnceLock;

/// Shader family; the design-system section it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    Cyberpunk,
    Material3,
    Matrix,
    Retro,
    Pixel,
    Nature,
    /// Re-implementations of the ghostty-shaders collection.
    Ghostty,
    /// Hand-painted anime looks and cel-shaded 3D.
    Ghibli,
    /// Sacred geometry, hyperspace tunnels and visionary-art effects.
    Dmt,
}

impl Family {
    pub const ALL: [Family; 9] = [
        Family::Cyberpunk,
        Family::Material3,
        Family::Matrix,
        Family::Retro,
        Family::Pixel,
        Family::Nature,
        Family::Ghostty,
        Family::Ghibli,
        Family::Dmt,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Family::Cyberpunk => "Cyberpunk",
            Family::Material3 => "Material 3",
            Family::Matrix => "Matrix",
            Family::Retro => "Retro terminals",
            Family::Pixel => "Pixel art",
            Family::Nature => "Cosmic & nature",
            Family::Ghostty => "Ghostty",
            Family::Ghibli => "Ghibli",
            Family::Dmt => "DMT Hyperspace",
        }
    }
    /// Generator id that makes a good input for this family's post effects.
    pub fn default_source(self) -> &'static str {
        match self {
            Family::Cyberpunk => "neon_city",
            Family::Material3 => "m3_blobs",
            Family::Matrix | Family::Retro | Family::Ghostty => "terminal_scene",
            Family::Ghibli => "gh_meadow",
            Family::Dmt => "dmt_flower_of_life",
            Family::Pixel => "platformer",
            Family::Nature => "plasma",
        }
    }
    /// Palette that suits the family best.
    pub fn palette(self) -> ShaderPalette {
        match self {
            Family::Cyberpunk => ShaderPalette::cyberpunk(),
            Family::Material3 => ShaderPalette::material3(),
            Family::Matrix => ShaderPalette::matrix(),
            Family::Retro => ShaderPalette::amber(),
            Family::Pixel => ShaderPalette::metroidvania(),
            Family::Nature => ShaderPalette::cyberpunk(),
            Family::Ghostty => ShaderPalette::matrix(),
            Family::Ghibli => ShaderPalette::ghibli(),
            Family::Dmt => ShaderPalette::hyperspace(),
        }
    }
}

/// Generator or post effect.
#[derive(Clone, Copy)]
pub enum Kind {
    Generator(GenFn),
    PostFx(FxFn),
}

/// One catalogued shader.
#[derive(Clone, Copy)]
pub struct ShaderInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub family: Family,
    pub desc: &'static str,
    pub kind: Kind,
}

impl ShaderInfo {
    pub const fn generator(
        id: &'static str,
        name: &'static str,
        family: Family,
        desc: &'static str,
        f: GenFn,
    ) -> Self {
        Self {
            id,
            name,
            family,
            desc,
            kind: Kind::Generator(f),
        }
    }
    pub const fn post(
        id: &'static str,
        name: &'static str,
        family: Family,
        desc: &'static str,
        f: FxFn,
    ) -> Self {
        Self {
            id,
            name,
            family,
            desc,
            kind: Kind::PostFx(f),
        }
    }
    pub fn is_generator(&self) -> bool {
        matches!(self.kind, Kind::Generator(_))
    }
}

/// Every shader, in family order.
pub fn catalog() -> &'static [ShaderInfo] {
    static ALL: OnceLock<Vec<ShaderInfo>> = OnceLock::new();
    ALL.get_or_init(|| {
        [
            cyberpunk::SHADERS,
            blade::SHADERS,
            ghibli::SHADERS,
            ghibli2::SHADERS,
            dmt::SHADERS,
            pixel2::SHADERS,
            material::SHADERS,
            matrix::SHADERS,
            retro::SHADERS,
            pixel::SHADERS,
            nature::SHADERS,
            fx::SHADERS,
            extra::SHADERS,
            retro2::SHADERS,
            ghostty::SHADERS,
        ]
        .concat()
    })
}

/// Shaders of one family.
pub fn of_family(f: Family) -> impl Iterator<Item = &'static ShaderInfo> {
    catalog().iter().filter(move |s| s.family == f)
}

/// Look up by id.
pub fn by_id(id: &str) -> Option<&'static ShaderInfo> {
    catalog().iter().find(|s| s.id == id)
}

/// Render a chain: the first entry must be a generator, the rest post
/// effects applied in order. Extra generators in the middle are skipped.
pub fn render(chain: &[&ShaderInfo], ctx: &Ctx) -> Frame {
    let mut it = chain.iter();
    let mut frame = match it.next().map(|s| s.kind) {
        Some(Kind::Generator(g)) => render_gen(g, ctx),
        _ => Frame::new(ctx.w, ctx.h),
    };
    for s in it {
        if let Kind::PostFx(f) = s.kind {
            frame = apply_fx(f, &frame, ctx);
        }
    }
    frame
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_unique_and_lookup_works() {
        let mut seen = HashSet::new();
        for s in catalog() {
            assert!(seen.insert(s.id), "duplicate id {}", s.id);
            assert!(by_id(s.id).is_some());
        }
        assert!(catalog().len() >= 40, "{}", catalog().len());
    }

    #[test]
    fn every_family_has_generators() {
        for f in Family::ALL {
            assert!(
                of_family(f).any(|s| s.is_generator()),
                "{} has no generator",
                f.name()
            );
        }
    }

    #[test]
    fn every_shader_renders_finite_and_varied() {
        let ctx = Ctx::new(48, 27, 1.7, ShaderPalette::cyberpunk());
        let base = by_id("test_pattern").unwrap();
        for s in catalog() {
            let f = if s.is_generator() {
                render(&[s], &ctx)
            } else {
                render(&[base, s], &ctx)
            };
            assert_eq!(f.px.len(), 48 * 27, "{}", s.id);
            assert!(
                f.px.iter()
                    .all(|p| p.x.is_finite() && p.y.is_finite() && p.z.is_finite()),
                "{} produced NaN",
                s.id
            );
            let first = f.px[0];
            let varied = f.px.iter().any(|p| (*p - first).len() > 0.02);
            assert!(varied, "{} is a flat colour", s.id);
        }
    }

    #[test]
    fn animated_generators_change_over_time() {
        let a = Ctx::new(40, 24, 0.0, ShaderPalette::cyberpunk());
        let b = Ctx { time: 2.3, ..a };
        let moving = [
            "neon_grid",
            "digital_rain",
            "starfield",
            "m3_blobs",
            "platformer",
        ];
        for id in moving {
            let s = by_id(id).unwrap();
            assert_ne!(render(&[s], &a), render(&[s], &b), "{id} is static");
        }
    }

    /// Vertical flow between two close frames: positive = content moves down.
    fn flow(id: &str) -> i32 {
        let (w, h) = (192i32, 144i32);
        let pal = ShaderPalette::cyberpunk();
        let s = by_id(id).unwrap();
        let a = render(&[s], &Ctx::new(w as u32, h as u32, 3.0, pal));
        let b = render(&[s], &Ctx::new(w as u32, h as u32, 3.12, pal));
        let mut best = (f32::MAX, 0);
        for dy in -3..=3 {
            let mut e = 0.0;
            for y in 4..h - 4 {
                for x in 0..w {
                    let d = b.get(x, y).luma() - a.get(x, y - dy).luma();
                    e += d * d;
                }
            }
            if e < best.0 {
                best = (e, dy);
            }
        }
        best.1
    }

    #[test]
    fn falling_things_fall() {
        for id in ["snow", "matrix_depth"] {
            assert!(flow(id) > 0, "{id} moves up");
        }
        assert!(flow("embers") < 0, "embers should rise");
    }
}
