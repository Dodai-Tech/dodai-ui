//! A tiny dependency-free RGBA colour type shared by every framework adapter.

use std::fmt;

/// 8-bit-per-channel sRGB colour with alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Opaque colour from a `0xRRGGBB` literal.
    pub const fn hex(rgb: u32) -> Self {
        Self {
            r: ((rgb >> 16) & 0xff) as u8,
            g: ((rgb >> 8) & 0xff) as u8,
            b: (rgb & 0xff) as u8,
            a: 255,
        }
    }

    /// Colour from channels.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Same colour with a new alpha (`0.0..=1.0`).
    pub fn with_alpha(self, alpha: f32) -> Self {
        Self {
            a: (alpha.clamp(0.0, 1.0) * 255.0).round() as u8,
            ..self
        }
    }

    /// Linear blend towards `other`; `t = 0.0` is `self`, `1.0` is `other`.
    pub fn mix(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let l = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
        Self {
            r: l(self.r, other.r),
            g: l(self.g, other.g),
            b: l(self.b, other.b),
            a: l(self.a, other.a),
        }
    }

    /// Blend towards white.
    pub fn lighten(self, t: f32) -> Self {
        self.mix(Self::hex(0xffffff), t)
    }

    /// Blend towards black.
    pub fn darken(self, t: f32) -> Self {
        self.mix(Self::hex(0x000000), t)
    }

    /// Channels as `[r, g, b]`.
    pub const fn rgb(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    /// Channels as normalised `[r, g, b, a]` floats.
    pub fn to_f32(self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    /// `#rrggbb` (alpha dropped).
    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// CSS colour: `#rrggbb` when opaque, `rgba(…)` otherwise.
    pub fn to_css(self) -> String {
        if self.a == 255 {
            self.to_hex()
        } else {
            format!(
                "rgba({},{},{},{:.2})",
                self.r,
                self.g,
                self.b,
                self.a as f32 / 255.0
            )
        }
    }

    /// Relative luminance (WCAG), `0.0..=1.0`.
    pub fn luminance(self) -> f32 {
        let f = |c: u8| {
            let c = c as f32 / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(self.r) + 0.7152 * f(self.g) + 0.0722 * f(self.b)
    }

    /// WCAG contrast ratio against `other` (`1.0..=21.0`).
    pub fn contrast(self, other: Self) -> f32 {
        let (a, b) = (self.luminance(), other.luminance());
        let (hi, lo) = if a > b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// Nearest xterm-256 palette index (for terminals without truecolor).
    pub fn to_ansi256(self) -> u8 {
        let cube = |c: u8| -> u8 {
            if c < 48 {
                0
            } else if c < 115 {
                1
            } else {
                ((c as u16 - 35) / 40) as u8
            }
        };
        let (ri, gi, bi) = (cube(self.r), cube(self.g), cube(self.b));
        const LV: [i32; 6] = [0, 95, 135, 175, 215, 255];
        let cube_rgb = (LV[ri as usize], LV[gi as usize], LV[bi as usize]);
        let cube_idx = 16 + 36 * ri + 6 * gi + bi;
        let avg = (self.r as i32 + self.g as i32 + self.b as i32) / 3;
        let gi2 = ((avg - 8).max(0) / 10).min(23);
        let gv = 8 + 10 * gi2;
        let dist = |p: (i32, i32, i32)| {
            let (dr, dg, db) = (
                self.r as i32 - p.0,
                self.g as i32 - p.1,
                self.b as i32 - p.2,
            );
            dr * dr + dg * dg + db * db
        };
        if dist((gv, gv, gv)) < dist(cube_rgb) {
            232 + gi2 as u8
        } else {
            cube_idx
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_css())
    }
}

impl From<u32> for Color {
    fn from(rgb: u32) -> Self {
        Self::hex(rgb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip() {
        let c = Color::hex(0xfcee0a);
        assert_eq!((c.r, c.g, c.b, c.a), (0xfc, 0xee, 0x0a, 255));
        assert_eq!(c.to_hex(), "#fcee0a");
        assert_eq!(c.to_css(), "#fcee0a");
    }

    #[test]
    fn alpha_css() {
        assert_eq!(Color::hex(0).with_alpha(0.5).to_css(), "rgba(0,0,0,0.50)");
    }

    #[test]
    fn mix_endpoints() {
        let (a, b) = (Color::hex(0x000000), Color::hex(0xffffff));
        assert_eq!(a.mix(b, 0.0), a);
        assert_eq!(a.mix(b, 1.0), b);
        assert_eq!(a.lighten(1.0), b);
        assert_eq!(b.darken(1.0), a);
    }

    #[test]
    fn contrast_black_white() {
        let r = Color::hex(0).contrast(Color::hex(0xffffff));
        assert!((r - 21.0).abs() < 0.01);
    }

    #[test]
    fn ansi256_extremes() {
        assert_eq!(Color::hex(0x000000).to_ansi256(), 16);
        assert_eq!(Color::hex(0xffffff).to_ansi256(), 231);
        assert_eq!(Color::hex(0xff0000).to_ansi256(), 196);
    }
}
