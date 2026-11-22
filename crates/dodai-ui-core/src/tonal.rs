//! Perceptual **tonal palettes**: a hue and a chroma, sampled at any tone `0..=100`.
//!
//! Tone is CIE L* (0 black, 100 white), so two tones a fixed distance apart always give the
//! same contrast whatever the hue; this is what lets a design system derive a whole colour
//! scheme (light or dark) from a single seed. Colours that fall outside sRGB keep their hue
//! and tone and lose chroma until they fit.
//!
//! ```
//! use dodai_ui_core::{Color, TonalPalette};
//! let p = TonalPalette::from_seed(Color::hex(0x6750a4), 36.0);
//! assert!(p.tone(90.0).contrast(p.tone(10.0)) > 7.0);
//! ```

use crate::Color;

const WHITE: [f64; 3] = [0.950_47, 1.0, 1.088_83]; // D65

fn srgb_to_linear(c: u8) -> f64 {
    let c = c as f64 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f64) -> f64 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn lab_f(t: f64) -> f64 {
    if t > 216.0 / 24_389.0 {
        t.cbrt()
    } else {
        (24_389.0 / 27.0 * t + 16.0) / 116.0
    }
}

fn lab_f_inv(t: f64) -> f64 {
    let t3 = t * t * t;
    if t3 > 216.0 / 24_389.0 {
        t3
    } else {
        (116.0 * t - 16.0) / (24_389.0 / 27.0)
    }
}

/// CIE LCh(ab) of a colour: `(L*, chroma, hue in degrees)`.
pub fn to_lch(c: Color) -> (f32, f32, f32) {
    let (r, g, b) = (
        srgb_to_linear(c.r),
        srgb_to_linear(c.g),
        srgb_to_linear(c.b),
    );
    let x = 0.412_456_4 * r + 0.357_576_1 * g + 0.180_437_5 * b;
    let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175 * b;
    let z = 0.019_333_9 * r + 0.119_192 * g + 0.950_304_1 * b;
    let (fx, fy, fz) = (
        lab_f(x / WHITE[0]),
        lab_f(y / WHITE[1]),
        lab_f(z / WHITE[2]),
    );
    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let bb = 200.0 * (fy - fz);
    let h = bb.atan2(a).to_degrees().rem_euclid(360.0);
    (l as f32, a.hypot(bb) as f32, h as f32)
}

fn lch_to_linear(l: f64, c: f64, h: f64) -> [f64; 3] {
    let (a, b) = (c * h.to_radians().cos(), c * h.to_radians().sin());
    let fy = (l + 16.0) / 116.0;
    let (fx, fz) = (fy + a / 500.0, fy - b / 200.0);
    let (x, y, z) = (
        lab_f_inv(fx) * WHITE[0],
        lab_f_inv(fy) * WHITE[1],
        lab_f_inv(fz) * WHITE[2],
    );
    [
        3.240_454_2 * x - 1.537_138_5 * y - 0.498_531_4 * z,
        -0.969_266 * x + 1.876_010_8 * y + 0.041_556 * z,
        0.055_643_4 * x - 0.204_025_9 * y + 1.057_225_2 * z,
    ]
}

fn in_gamut(rgb: [f64; 3]) -> bool {
    rgb.iter().all(|v| (-0.000_5..=1.000_5).contains(v))
}

/// Colour at CIE `l`, `chroma`, `hue`; chroma is reduced until the colour fits in sRGB.
pub fn from_lch(l: f32, chroma: f32, hue: f32) -> Color {
    let l = (l as f64).clamp(0.0, 100.0);
    if l >= 99.999 {
        return Color::hex(0xffffff);
    }
    if l <= 0.001 {
        return Color::hex(0x000000);
    }
    let h = (hue as f64).rem_euclid(360.0);
    let mut c = (chroma as f64).max(0.0);
    let mut rgb = lch_to_linear(l, c, h);
    if !in_gamut(rgb) {
        let (mut lo, mut hi) = (0.0, c);
        for _ in 0..24 {
            let mid = (lo + hi) / 2.0;
            if in_gamut(lch_to_linear(l, mid, h)) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        c = lo;
        rgb = lch_to_linear(l, c, h);
    }
    let q = |v: f64| (linear_to_srgb(v.clamp(0.0, 1.0)) * 255.0).round() as u8;
    Color::rgba(q(rgb[0]), q(rgb[1]), q(rgb[2]), 255)
}

/// A hue and a chroma, sampled by tone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TonalPalette {
    pub hue: f32,
    pub chroma: f32,
}

impl TonalPalette {
    pub const fn new(hue: f32, chroma: f32) -> Self {
        Self { hue, chroma }
    }

    /// The hue of `seed` with the given chroma.
    pub fn from_seed(seed: Color, chroma: f32) -> Self {
        Self::new(to_lch(seed).2, chroma)
    }

    /// The same palette with its hue rotated by `degrees`.
    pub fn rotated(self, degrees: f32) -> Self {
        Self::new((self.hue + degrees).rem_euclid(360.0), self.chroma)
    }

    /// The colour at tone `t` (`0` black … `100` white).
    pub fn tone(&self, t: f32) -> Color {
        from_lch(t, self.chroma, self.hue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tones_are_monotonic_in_luminance() {
        let p = TonalPalette::new(280.0, 40.0);
        let mut last = -1.0;
        for t in (0..=100).step_by(10) {
            let y = p.tone(t as f32).luminance();
            assert!(y >= last, "tone {t}");
            last = y;
        }
    }

    #[test]
    fn round_trips_hue_and_tone() {
        let c = Color::hex(0x6750a4);
        let (l, ch, h) = to_lch(c);
        let back = from_lch(l, ch, h);
        assert!(
            c.r.abs_diff(back.r) <= 1 && c.g.abs_diff(back.g) <= 1 && c.b.abs_diff(back.b) <= 1
        );
    }

    #[test]
    fn equal_tone_gaps_give_equal_contrast_across_hues() {
        let lo: Vec<f32> = [20.0, 90.0, 150.0, 220.0, 300.0]
            .iter()
            .map(|h| {
                TonalPalette::new(*h, 40.0)
                    .tone(90.0)
                    .contrast(TonalPalette::new(*h, 40.0).tone(30.0))
            })
            .collect();
        let (mn, mx) = lo
            .iter()
            .fold((f32::MAX, 0f32), |(a, b), v| (a.min(*v), b.max(*v)));
        assert!(mx - mn < 1.0, "{lo:?}");
    }

    #[test]
    fn extremes_and_gamut() {
        let p = TonalPalette::new(30.0, 200.0);
        assert_eq!(p.tone(100.0), Color::hex(0xffffff));
        assert_eq!(p.tone(0.0), Color::hex(0x000000));
        let _ = p.tone(55.0); // must not panic out of gamut
    }
}
