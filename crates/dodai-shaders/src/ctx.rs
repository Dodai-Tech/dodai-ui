//! The shader context and palette: shared verbatim by the CPU renderer and
//! the GPU kernels (`dodai-shaders-kernels`), so no std here.

use crate::math::*;

/// Five-colour palette every shader draws from, so each shader follows the
/// active design system.
#[cfg_attr(not(target_arch = "spirv"), derive(Debug, PartialEq))]
#[derive(Clone, Copy)]
pub struct ShaderPalette {
    pub bg: Vec3,
    pub fg: Vec3,
    /// Primary brand colour.
    pub a: Vec3,
    /// Secondary brand colour.
    pub b: Vec3,
    /// Accent.
    pub c: Vec3,
}

impl ShaderPalette {
    /// Neon yellow / cyan / magenta on near-black.
    pub fn cyberpunk() -> Self {
        Self {
            bg: rgb8(6, 6, 12),
            fg: rgb8(235, 235, 220),
            a: rgb8(252, 238, 10),
            b: rgb8(0, 240, 255),
            c: rgb8(255, 40, 110),
        }
    }
    /// Soft meadow greens and sky blues.
    pub fn ghibli() -> Self {
        Self {
            bg: rgb8(24, 52, 70),
            fg: rgb8(252, 248, 236),
            a: rgb8(126, 190, 80),
            b: rgb8(100, 176, 226),
            c: rgb8(255, 170, 120),
        }
    }
    /// Teal night, orange smog and magenta neon: a rainy-megacity look.
    pub fn bladerunner() -> Self {
        Self {
            bg: rgb8(4, 9, 13),
            fg: rgb8(240, 226, 200),
            a: rgb8(255, 138, 40),
            b: rgb8(46, 200, 214),
            c: rgb8(255, 52, 120),
        }
    }
    /// Material 3 baseline (purple) scheme, dark.
    pub fn material3() -> Self {
        Self {
            bg: rgb8(20, 18, 24),
            fg: rgb8(230, 224, 233),
            a: rgb8(208, 188, 255),
            b: rgb8(204, 194, 220),
            c: rgb8(239, 184, 200),
        }
    }
    /// Phosphor green.
    pub fn matrix() -> Self {
        Self {
            bg: rgb8(0, 6, 2),
            fg: rgb8(200, 255, 210),
            a: rgb8(0, 255, 70),
            b: rgb8(0, 140, 40),
            c: rgb8(180, 255, 190),
        }
    }
    /// Amber monochrome terminal.
    pub fn amber() -> Self {
        Self {
            bg: rgb8(14, 7, 0),
            fg: rgb8(255, 190, 70),
            a: rgb8(255, 176, 0),
            b: rgb8(200, 110, 0),
            c: rgb8(255, 220, 140),
        }
    }
    /// Commodore 64 READY. screen: blue paper, light-blue ink.
    pub fn c64() -> Self {
        Self {
            bg: rgb8(53, 40, 121),
            fg: rgb8(136, 126, 203),
            a: rgb8(170, 255, 238),
            b: rgb8(0, 136, 255),
            c: rgb8(255, 119, 119),
        }
    }
    /// DOS / VGA text mode: blue, grey and cyan.
    pub fn dos() -> Self {
        Self {
            bg: rgb8(0, 0, 170),
            fg: rgb8(170, 170, 170),
            a: rgb8(85, 255, 255),
            b: rgb8(0, 170, 170),
            c: rgb8(255, 85, 85),
        }
    }
    /// Super-Mario-ish sky/brick/coin colours.
    pub fn platformer() -> Self {
        Self {
            bg: rgb8(92, 148, 252),
            fg: rgb8(252, 252, 252),
            a: rgb8(200, 76, 12),
            b: rgb8(0, 168, 0),
            c: rgb8(252, 188, 60),
        }
    }
    /// Dark castle-vania blues and torch orange.
    pub fn metroidvania() -> Self {
        Self {
            bg: rgb8(10, 8, 24),
            fg: rgb8(200, 200, 230),
            a: rgb8(70, 80, 150),
            b: rgb8(40, 120, 110),
            c: rgb8(255, 150, 40),
        }
    }
}

impl Default for ShaderPalette {
    fn default() -> Self {
        Self::cyberpunk()
    }
}

/// Everything a shader may read besides its pixel position.
#[cfg_attr(not(target_arch = "spirv"), derive(Debug))]
#[derive(Clone, Copy)]
pub struct Ctx {
    /// Seconds.
    pub time: f32,
    pub w: u32,
    pub h: u32,
    pub pal: ShaderPalette,
    /// Effect strength `0..=1` (post effects; some generators too).
    pub intensity: f32,
}

impl Ctx {
    pub fn new(w: u32, h: u32, time: f32, pal: ShaderPalette) -> Self {
        Self {
            time,
            w,
            h,
            pal,
            intensity: 1.0,
        }
    }
    pub fn intensity(mut self, i: f32) -> Self {
        self.intensity = i.clamp(0.0, 1.0);
        self
    }
    pub fn aspect(&self) -> f32 {
        self.w as f32 / self.h.max(1) as f32
    }
    /// `uv` (`0..1`, y down) to centred coords: x in `±aspect`, y up in `±1`.
    pub fn centered(&self, uv: Vec2) -> Vec2 {
        vec2((uv.x - 0.5) * 2.0 * self.aspect(), (0.5 - uv.y) * 2.0)
    }
    /// Size of one pixel in uv units.
    pub fn texel(&self) -> Vec2 {
        vec2(1.0 / self.w.max(1) as f32, 1.0 / self.h.max(1) as f32)
    }
}
