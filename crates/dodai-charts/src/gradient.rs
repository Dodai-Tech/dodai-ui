//! Colour gradients for heatmaps and continuous scales, as sRGB stops.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Gradient {
    /// Theme-driven; renderers mix surface → primary themselves.
    #[default]
    Theme,
    Viridis,
    Magma,
    Plasma,
    /// Blue → white → red, for values around a midpoint.
    Diverging,
    Grayscale,
}

const VIRIDIS: &[[u8; 3]] = &[
    [68, 1, 84],
    [59, 82, 139],
    [33, 145, 140],
    [94, 201, 98],
    [253, 231, 37],
];
const MAGMA: &[[u8; 3]] = &[
    [0, 0, 4],
    [81, 18, 124],
    [183, 55, 121],
    [252, 137, 97],
    [252, 253, 191],
];
const PLASMA: &[[u8; 3]] = &[
    [13, 8, 135],
    [126, 3, 168],
    [204, 71, 120],
    [248, 149, 64],
    [240, 249, 33],
];
const DIVERGING: &[[u8; 3]] = &[
    [49, 54, 149],
    [145, 191, 219],
    [247, 247, 247],
    [252, 141, 89],
    [165, 0, 38],
];
const GRAY: &[[u8; 3]] = &[[16, 16, 16], [240, 240, 240]];

impl Gradient {
    pub const ALL: [Gradient; 6] = [
        Gradient::Theme,
        Gradient::Viridis,
        Gradient::Magma,
        Gradient::Plasma,
        Gradient::Diverging,
        Gradient::Grayscale,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Gradient::Theme => "Theme",
            Gradient::Viridis => "Viridis",
            Gradient::Magma => "Magma",
            Gradient::Plasma => "Plasma",
            Gradient::Diverging => "Diverging",
            Gradient::Grayscale => "Gray",
        }
    }

    fn stops(self) -> Option<&'static [[u8; 3]]> {
        match self {
            Gradient::Theme => None,
            Gradient::Viridis => Some(VIRIDIS),
            Gradient::Magma => Some(MAGMA),
            Gradient::Plasma => Some(PLASMA),
            Gradient::Diverging => Some(DIVERGING),
            Gradient::Grayscale => Some(GRAY),
        }
    }

    /// sRGB at `t` in `0..=1`; `None` for [`Gradient::Theme`].
    pub fn sample(self, t: f32) -> Option<[u8; 3]> {
        let s = self.stops()?;
        let t = t.clamp(0.0, 1.0) * (s.len() - 1) as f32;
        let i = (t.floor() as usize).min(s.len() - 2);
        let f = t - i as f32;
        let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * f).round() as u8;
        Some([
            mix(s[i][0], s[i + 1][0]),
            mix(s[i][1], s[i + 1][1]),
            mix(s[i][2], s[i + 1][2]),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints_and_theme() {
        assert_eq!(Gradient::Viridis.sample(0.0), Some([68, 1, 84]));
        assert_eq!(Gradient::Viridis.sample(1.0), Some([253, 231, 37]));
        assert_eq!(Gradient::Theme.sample(0.5), None);
        assert_eq!(
            Gradient::Grayscale.sample(2.0),
            Gradient::Grayscale.sample(1.0)
        );
    }

    #[test]
    fn midpoint_of_diverging_is_light() {
        let m = Gradient::Diverging.sample(0.5).unwrap();
        assert_eq!(m, [247, 247, 247]);
    }
}
