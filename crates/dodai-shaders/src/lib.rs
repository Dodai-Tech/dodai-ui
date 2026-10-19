//! # dodai-shaders
//!
//! Pixel shaders for the Dodai design system and beyond, written in plain Rust
//! and run on the GPU. A shader is a function evaluated per pixel
//! (`fn(uv, &Ctx) -> Vec3`); the sources are compiled once by rust-gpu to WGSL
//! (`dodai-shaders-kernels`, an unpublished build step) and the result is
//! embedded here, so using the crate needs no special toolchain. It runs on
//! every wgpu backend: Metal, Vulkan, DX12 and, through WebGPU, the browser.
//!
//! * **Generators** make a picture from nothing (`neon_grid`, `digital_rain`).
//! * **Post effects** transform the picture before them (`crt`, `vhs`, `gameboy`).
//! * Both read colours from a [`ShaderPalette`], so they follow the theme.
//!
//! ```no_run
//! use dodai_shaders::{gpu::Gpu, ShaderPalette};
//!
//! let mut gpu = Gpu::new().expect("no GPU");
//! // a generator first, then post effects; returns packed BGRA8
//! let bgra = gpu
//!     .render(&["neon_grid", "crt"], 1280, 720, 1.5, 1.0, &ShaderPalette::cyberpunk())
//!     .unwrap();
//! assert_eq!(bgra.len(), 1280 * 720 * 4);
//! ```
//!
//! [`catalog`] lists every shader (id, name, family, kind) without a GPU.
// Index loops, `match` ladders and range comparisons are deliberate: the GPU
// build (rust-gpu) cannot lower the iterator / `contains` forms.
#![allow(
    clippy::needless_range_loop,
    clippy::manual_range_contains,
    clippy::manual_memcpy
)]

mod catalog;
pub mod ctx;
pub mod gpu;
pub mod math;
pub mod prelude;
pub mod presets;

pub use ctx::{Ctx, ShaderPalette};
pub use math::{vec2, vec3, Vec2, Vec3};
pub use presets::Preset;

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

/// One catalogued shader.
#[derive(Clone, Copy, Debug)]
pub struct ShaderInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub family: Family,
    pub desc: &'static str,
    pub kind: Kind,
}

impl ShaderInfo {
    pub(crate) const fn generator(
        id: &'static str,
        name: &'static str,
        family: Family,
        desc: &'static str,
    ) -> Self {
        Self {
            id,
            name,
            family,
            desc,
            kind: Kind::Generator,
        }
    }
    pub(crate) const fn post(
        id: &'static str,
        name: &'static str,
        family: Family,
        desc: &'static str,
    ) -> Self {
        Self {
            id,
            name,
            family,
            desc,
            kind: Kind::PostFx,
        }
    }
    pub fn is_generator(&self) -> bool {
        self.kind == Kind::Generator
    }
}

/// Every shader, in family order.
pub fn catalog() -> &'static [ShaderInfo] {
    catalog::SHADERS
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
