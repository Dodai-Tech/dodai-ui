//! # dodai-shaders
//!
//! Pixel shaders for the Dodai design system and beyond, written in plain Rust
//! and run on the GPU first, with the CPU as the fallback. A shader is a function evaluated per pixel
//! (`fn(uv, &Ctx) -> Vec3`). The same sources run on the CPU as ordinary Rust and are
//! compiled once by rust-gpu to WGSL (`dodai-shaders-kernels`, an unpublished build step);
//! the result is embedded here, so using the crate needs no special toolchain. It runs on
//! every wgpu backend: Metal, Vulkan, DX12 and, through WebGPU, the browser.
//!
//! * **Generators** make a picture from nothing (`neon_grid`, `digital_rain`).
//! * **Post effects** transform the picture before them (`crt`, `vhs`, `gameboy`).
//! * Both read colours from a [`ShaderPalette`], so they follow the theme.
//!
//! ```no_run
//! use dodai_shaders::{Renderer, ShaderPalette};
//!
//! // GPU when a real adapter exists, else the CPU: there is no choice to make
//! let mut renderer = Renderer::new();
//! // a generator first, then post effects; returns packed BGRA8
//! let bgra = renderer
//!     .render(&["neon_grid", "crt"], 1280, 720, 1.5, 1.0, &ShaderPalette::cyberpunk())
//!     .unwrap();
//! assert_eq!(bgra.len(), 1280 * 720 * 4);
//! ```
//!
//! [`catalog`] lists every shader (id, name, family, kind) without a GPU.
// Index loops, `match` ladders and range comparisons are deliberate: the GPU
// build (rust-gpu) cannot lower the iterator / `contains` forms.
// `use crate::prelude::*` is empty on the host but needed by the GPU build.
#![allow(
    unused_imports,
    clippy::needless_range_loop,
    clippy::manual_range_contains,
    clippy::manual_memcpy
)]
#![recursion_limit = "256"]

/// `ShaderNode` for bevy.
#[cfg(feature = "bevy")]
pub mod bevy;
pub mod cpu;
mod ctx;
/// `ShaderView` for dioxus.
#[cfg(feature = "dioxus")]
pub mod dioxus;
mod frame;
pub mod gpu;
/// `ShaderView` for gpui.
#[cfg(feature = "gpui")]
pub mod gpui;
pub mod groups;
/// `ShaderView` for iced.
#[cfg(feature = "iced")]
pub mod iced;
/// Shader maths, shared verbatim with the GPU kernels (vectors, hashes, noise, easing).
pub mod math;
pub mod player;
mod prelude;
pub mod presets;
/// 2D light transport with shadows (radiance cascades, arXiv:2408.14425); CPU, host only.
pub mod radiance;
mod renderer;
mod shaders;
// The shader sources refer to each other as `crate::matrix` etc. (also in the GPU build).
use shaders::{matrix, pixel, retro2};

pub use ctx::{Ctx, ShaderPalette};
pub use math::{vec2, vec3, Vec2, Vec3};
pub use presets::Preset;
pub use renderer::Renderer;

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Generator,
    PostFx,
}

/// A generator's per-pixel function.
type GenFn = fn(Vec2, &Ctx) -> Vec3;
/// A post effect's per-pixel function.
type PostFn = fn(&frame::Frame, Vec2, &Ctx) -> Vec3;

#[derive(Clone, Copy, Debug)]
enum Func {
    Gen(GenFn),
    Post(PostFn),
}

/// One catalogued shader.
#[derive(Clone, Copy, Debug)]
pub struct ShaderInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub family: Family,
    pub desc: &'static str,
    pub kind: Kind,
    func: Func,
}

impl ShaderInfo {
    pub(crate) const fn generator(
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
            kind: Kind::Generator,
            func: Func::Gen(f),
        }
    }
    pub(crate) const fn post(
        id: &'static str,
        name: &'static str,
        family: Family,
        desc: &'static str,
        f: PostFn,
    ) -> Self {
        Self {
            id,
            name,
            family,
            desc,
            kind: Kind::PostFx,
            func: Func::Post(f),
        }
    }
    pub fn is_generator(&self) -> bool {
        self.kind == Kind::Generator
    }
}

/// Every shader, in family order.
pub fn catalog() -> &'static [ShaderInfo] {
    static ALL: std::sync::OnceLock<Vec<ShaderInfo>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        use shaders::*;
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

/// Why a render did not happen.
#[derive(Debug)]
pub enum RenderError {
    /// The chain is empty, or does not start with a generator.
    BadChain(String),
    /// Unknown shader id.
    Unknown(String),
    /// Reading the image back failed.
    Readback(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::BadChain(s) => write!(f, "bad chain: {s}"),
            RenderError::Unknown(s) => write!(f, "unknown shader `{s}`"),
            RenderError::Readback(s) => write!(f, "readback failed: {s}"),
        }
    }
}
impl std::error::Error for RenderError {}

/// The shaders of `chain`: a generator, then post effects.
pub(crate) fn check_chain(chain: &[&str]) -> Result<Vec<&'static ShaderInfo>, RenderError> {
    if chain.is_empty() {
        return Err(RenderError::BadChain("empty".into()));
    }
    let mut out = Vec::with_capacity(chain.len());
    for (i, id) in chain.iter().enumerate() {
        let info = by_id(id).ok_or_else(|| RenderError::Unknown((*id).to_string()))?;
        if (i == 0) != info.is_generator() {
            return Err(RenderError::BadChain(format!(
                "`{id}` is {} at position {i}",
                if info.is_generator() {
                    "a generator"
                } else {
                    "a post effect"
                }
            )));
        }
        out.push(info);
    }
    Ok(out)
}

/// Shaders of one family.
pub fn of_family(f: Family) -> impl Iterator<Item = &'static ShaderInfo> {
    catalog().iter().filter(move |s| s.family == f)
}

/// Look up by id.
pub fn by_id(id: &str) -> Option<&'static ShaderInfo> {
    catalog().iter().find(|s| s.id == id)
}

impl ShaderPalette {
    /// The palette of a Dodai theme, so shaders follow the design system.
    pub fn from_theme(t: &dodai_core::Theme) -> Self {
        let c = |c: dodai_core::Color| Vec3::from_u8(c.rgb());
        let p = &t.palette;
        Self {
            bg: c(p.background),
            fg: c(p.text),
            a: c(p.primary),
            b: c(p.secondary),
            c: c(p.accent),
        }
    }
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
        assert!(catalog().len() >= 190, "{}", catalog().len());
    }

    #[test]
    fn every_family_has_generators_and_a_valid_default_source() {
        for f in Family::ALL {
            assert!(
                of_family(f).any(|s| s.is_generator()),
                "{} has no generator",
                f.name()
            );
            let src = by_id(f.default_source()).expect("default source exists");
            assert!(src.is_generator(), "{}", src.id);
        }
    }

    #[test]
    fn presets_resolve() {
        for p in presets::all() {
            assert_eq!(p.shaders().len(), p.chain.len(), "{}", p.id);
            assert!(p.shaders()[0].is_generator(), "{}", p.id);
        }
    }
}
