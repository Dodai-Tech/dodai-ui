//! Every Dodai design token as a typed value: colour (light and dark, with the alpha ramps,
//! frosted-glass fills and status tones), typography, spacing (static and fluid), radii, blur,
//! shadows, durations, easings, borders, focus, breakpoints, the page grid, the media fade and
//! the dodai gradient. The theme in this crate is built from these; they are public so a layout
//! can use the ones the generic token set has no slot for (shadows, blur, fluid scales, grid).

use crate::Color;

// ---------------------------------------------------------------------------------- colour

/// An `hsl()` / `hsl(… / a)` token: hue in degrees, saturation and lightness in percent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsla {
    pub h: f32,
    pub s: f32,
    pub l: f32,
    pub a: f32,
}

impl Hsla {
    const fn new(h: f32, s: f32, l: f32, a: f32) -> Self {
        Self { h, s, l, a }
    }

    /// The colour, with alpha.
    pub fn color(self) -> Color {
        let (s, l) = (self.s / 100.0, self.l / 100.0);
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let hp = (self.h.rem_euclid(360.0)) / 60.0;
        let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
        let (r, g, b) = match hp as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let m = l - c / 2.0;
        let ch = |v: f32| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
        Color {
            r: ch(r),
            g: ch(g),
            b: ch(b),
            a: (self.a * 255.0).round() as u8,
        }
    }

    /// The colour composited over an opaque `under`, i.e. what the eye sees.
    pub fn over(self, under: Color) -> Color {
        let c = self.color();
        let u = under;
        let k = self.a;
        let mix = |a: u8, b: u8| (f32::from(a) * k + f32::from(b) * (1.0 - k)).round() as u8;
        Color {
            r: mix(c.r, u.r),
            g: mix(c.g, u.g),
            b: mix(c.b, u.b),
            a: 255,
        }
    }
}

const fn hsl(h: f32, s: f32, l: f32) -> Hsla {
    Hsla::new(h, s, l, 1.0)
}
const fn hsla(h: f32, s: f32, l: f32, a: f32) -> Hsla {
    Hsla::new(h, s, l, a)
}

/// One status colour with its four companions: `medium`, `low`, `glass` and `glass_soft`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatusTokens {
    pub solid: Hsla,
    pub muted: Hsla,
    pub tint: Hsla,
    pub glass: Hsla,
    pub glass_soft: Hsla,
}

/// Every colour token of one mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTokens {
    pub canvas: Hsla,
    pub surface: Hsla,
    pub glass: Hsla,
    pub glass_soft: Hsla,
    pub glass_strong: Hsla,
    pub veil: Hsla,
    pub ink: Hsla,
    pub ink_90: Hsla,
    pub ink_75: Hsla,
    pub ink_60: Hsla,
    pub ink_45: Hsla,
    pub ink_25: Hsla,
    pub info: StatusTokens,
    pub success: StatusTokens,
    pub warning: StatusTokens,
    pub danger: StatusTokens,
    /// The focus ring colour (the same in both modes).
    pub focus: Color,
}

const fn status(
    b: (f32, f32, f32),
    glass: (f32, f32, f32, f32),
    soft: (f32, f32, f32, f32),
) -> StatusTokens {
    StatusTokens {
        solid: hsl(b.0, b.1, b.2),
        muted: hsla(b.0, b.1, b.2, 0.6),
        tint: hsla(b.0, b.1, b.2, 0.18),
        glass: hsla(glass.0, glass.1, glass.2, glass.3),
        glass_soft: hsla(soft.0, soft.1, soft.2, soft.3),
    }
}

/// The light-mode palette: pearl canvas, indigo ink, white glass.
pub const LIGHT: ColorTokens = ColorTokens {
    canvas: hsl(248.0, 40.0, 98.0),
    surface: hsl(250.0, 36.0, 95.0),
    glass: hsla(252.0, 100.0, 99.0, 0.58),
    glass_soft: hsla(252.0, 100.0, 99.0, 0.34),
    glass_strong: hsla(252.0, 100.0, 99.0, 0.78),
    veil: hsla(246.0, 55.0, 10.0, 0.45),
    ink: hsl(246.0, 45.0, 14.0),
    ink_90: hsla(246.0, 30.0, 18.0, 0.84),
    ink_75: hsla(246.0, 28.0, 22.0, 0.72),
    ink_60: hsla(246.0, 26.0, 26.0, 0.6),
    ink_45: hsla(248.0, 24.0, 32.0, 0.46),
    ink_25: hsla(250.0, 24.0, 40.0, 0.22),
    info: status(
        (238.0, 82.0, 56.0),
        (232.0, 100.0, 93.0, 0.6),
        (232.0, 90.0, 96.0, 0.6),
    ),
    success: status(
        (162.0, 72.0, 28.0),
        (158.0, 80.0, 90.0, 0.6),
        (158.0, 70.0, 95.0, 0.6),
    ),
    warning: status(
        (34.0, 94.0, 34.0),
        (42.0, 100.0, 90.0, 0.6),
        (42.0, 90.0, 95.0, 0.6),
    ),
    danger: status(
        (346.0, 80.0, 36.0),
        (350.0, 100.0, 92.0, 0.6),
        (350.0, 90.0, 96.0, 0.6),
    ),
    focus: Color::hex(0x7c5cff),
};

/// The dark-mode palette: deep night, lavender-white ink, smoked glass.
pub const DARK: ColorTokens = ColorTokens {
    canvas: hsl(246.0, 50.0, 5.0),
    surface: hsl(246.0, 32.0, 11.0),
    glass: hsla(250.0, 70.0, 88.0, 0.17),
    glass_soft: hsla(250.0, 60.0, 82.0, 0.06),
    glass_strong: hsla(250.0, 60.0, 82.0, 0.19),
    veil: hsla(246.0, 60.0, 3.0, 0.6),
    ink: hsl(250.0, 60.0, 97.0),
    ink_90: hsla(250.0, 40.0, 96.0, 0.82),
    ink_75: hsla(250.0, 40.0, 96.0, 0.7),
    ink_60: hsla(250.0, 40.0, 96.0, 0.58),
    ink_45: hsla(250.0, 40.0, 96.0, 0.44),
    ink_25: hsla(250.0, 30.0, 70.0, 0.26),
    info: status(
        (224.0, 100.0, 70.0),
        (224.0, 70.0, 22.0, 0.68),
        (224.0, 55.0, 16.0, 0.68),
    ),
    success: status(
        (160.0, 70.0, 50.0),
        (160.0, 70.0, 20.0, 0.68),
        (160.0, 55.0, 15.0, 0.68),
    ),
    warning: status(
        (38.0, 95.0, 60.0),
        (38.0, 70.0, 20.0, 0.68),
        (38.0, 55.0, 15.0, 0.68),
    ),
    danger: status(
        (350.0, 90.0, 66.0),
        (350.0, 70.0, 22.0, 0.68),
        (350.0, 55.0, 16.0, 0.68),
    ),
    focus: Color::hex(0x7c5cff),
};

// ----------------------------------------------------------------------------- typography

/// Font weights: light-leaning body, medium emphasis.
pub const WEIGHT_NORMAL: u16 = 400;
pub const WEIGHT_SEMIBOLD: u16 = 500;
pub const WEIGHT_BOLD: u16 = 600;

/// Body line height: 1.5 × the font size.
pub fn leading_normal(font_px: f32) -> f32 {
    font_px * 1.5
}

/// The type stack: a geometric humanist sans, then whatever the platform has.
pub const FONT_STACK: &str = "Inter, Segoe UI, Roboto, Helvetica Neue, Arial, sans-serif";

/// A `clamp(min rem, vw·x + base rem, max rem)` type step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FluidRem {
    pub min: f32,
    pub vw: f32,
    pub base: f32,
    pub max: f32,
}

impl FluidRem {
    /// Size in px at a viewport width (1 rem = 16 px).
    pub fn at(self, viewport_px: f32) -> f32 {
        ((self.vw / 100.0 * viewport_px + self.base * 16.0).clamp(self.min * 16.0, self.max * 16.0))
            .max(0.0)
    }
}

const fn fixed_rem(r: f32) -> FluidRem {
    FluidRem {
        min: r,
        vw: 0.0,
        base: r,
        max: r,
    }
}
const fn fluid_rem(min: f32, vw: f32, base: f32, max: f32) -> FluidRem {
    FluidRem { min, vw, base, max }
}

/// The type scale, `2xs … 5xl`.
pub const TYPESCALE: [(&str, FluidRem); 10] = [
    ("2xs", fixed_rem(0.70)),
    ("xs", fixed_rem(0.84)),
    ("sm", fixed_rem(0.92)),
    ("md", fluid_rem(1.05, 0.14, 0.98, 1.25)),
    ("lg", fluid_rem(1.2, 0.38, 1.05, 1.6)),
    ("xl", fluid_rem(1.35, 0.7, 1.1, 2.0)),
    ("2xl", fluid_rem(1.5, 1.2, 1.15, 2.7)),
    ("3xl", fluid_rem(1.7, 1.9, 1.2, 3.6)),
    ("4xl", fluid_rem(1.9, 2.8, 1.2, 4.7)),
    ("5xl", fluid_rem(2.1, 4.0, 1.15, 6.2)),
];

/// The viewport width the UI scale in the theme is resolved at.
pub const DESIGN_WIDTH: f32 = 1440.0;

/// A type step by name at a viewport width.
pub fn type_size(step: &str, viewport_px: f32) -> Option<f32> {
    TYPESCALE
        .iter()
        .find(|(n, _)| *n == step)
        .map(|(_, f)| f.at(viewport_px))
}

// --------------------------------------------------------------------------------- spacing

/// Static spacing, `2xs … 2xl` (4, 8, 12, 16, 24, 32, 48 px).
pub const SPACING_STATIC: [(&str, f32); 7] = [
    ("2xs", 4.0),
    ("xs", 8.0),
    ("sm", 12.0),
    ("md", 16.0),
    ("lg", 24.0),
    ("xl", 32.0),
    ("2xl", 48.0),
];

/// A `clamp(min px, vw·x + base px, max px)` spacing step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FluidPx {
    pub min: f32,
    pub vw: f32,
    pub base: f32,
    pub max: f32,
}

impl FluidPx {
    pub fn at(self, viewport_px: f32) -> f32 {
        (self.vw / 100.0 * viewport_px + self.base).clamp(self.min, self.max)
    }
}

/// Fluid spacing, `xs … 2xl`.
pub const SPACING_FLUID: [(&str, FluidPx); 6] = [
    (
        "xs",
        FluidPx {
            min: 4.0,
            vw: 0.25,
            base: 3.0,
            max: 8.0,
        },
    ),
    (
        "sm",
        FluidPx {
            min: 8.0,
            vw: 0.5,
            base: 6.0,
            max: 16.0,
        },
    ),
    (
        "md",
        FluidPx {
            min: 16.0,
            vw: 1.25,
            base: 12.0,
            max: 36.0,
        },
    ),
    (
        "lg",
        FluidPx {
            min: 32.0,
            vw: 2.75,
            base: 23.0,
            max: 76.0,
        },
    ),
    (
        "xl",
        FluidPx {
            min: 48.0,
            vw: 3.0,
            base: 38.0,
            max: 96.0,
        },
    ),
    (
        "2xl",
        FluidPx {
            min: 80.0,
            vw: 7.5,
            base: 56.0,
            max: 200.0,
        },
    ),
];

/// A static spacing step by name.
pub fn spacing_static(step: &str) -> Option<f32> {
    SPACING_STATIC
        .iter()
        .find(|(n, _)| *n == step)
        .map(|(_, v)| *v)
}

/// A fluid spacing step by name at a viewport width.
pub fn spacing_fluid(step: &str, viewport_px: f32) -> Option<f32> {
    SPACING_FLUID
        .iter()
        .find(|(n, _)| *n == step)
        .map(|(_, f)| f.at(viewport_px))
}

// ----------------------------------------------------------------------- radius and border

/// Radii `xs … 4xl` in px (`full` is [`RADIUS_FULL`]). Generous on purpose.
pub const RADIUS: [(&str, f32); 8] = [
    ("xs", 4.0),
    ("sm", 6.0),
    ("md", 8.0),
    ("lg", 12.0),
    ("xl", 16.0),
    ("2xl", 20.0),
    ("3xl", 28.0),
    ("4xl", 40.0),
];

/// A pill. Finite stand-in for an infinite radius.
pub const RADIUS_FULL: f32 = 9999.0;

/// A radius step by name.
pub fn radius(step: &str) -> Option<f32> {
    RADIUS.iter().find(|(n, _)| *n == step).map(|(_, v)| *v)
}

/// Border widths: hairlines, so glass edges stay delicate.
pub const BORDER_WIDTH: f32 = 1.0;
pub const BORDER_WIDTH_THIN: f32 = 0.5;

// ------------------------------------------------------------------ blur, shadow, gradient

/// Frosted glass blur radius in px. Rendered by `dodai_ui_gpui::FrostedScene` (a blurred copy
/// of the scene's own picture under each panel); `gpui` cannot blur arbitrary UI behind a layer.
pub const BLUR_FROSTED: f32 = 24.0;

/// Saturation of the blurred backdrop under a frosted pane: glass over colour reads richer
/// than the colour itself.
pub const FROST_SATURATION: f32 = 1.7;
/// The thin blur used under small floating parts (chips, tooltips).
pub const BLUR_THIN: f32 = 10.0;

/// A drop shadow tinted with the ink colour: offset y, blur radius and alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub y: f32,
    pub blur: f32,
    pub alpha: f32,
}

/// Small shadow: chips and inputs.
pub const SHADOW_SM: Shadow = Shadow {
    y: 2.0,
    blur: 8.0,
    alpha: 0.10,
};
/// Medium shadow: cards.
pub const SHADOW_MD: Shadow = Shadow {
    y: 6.0,
    blur: 20.0,
    alpha: 0.12,
};
/// Large shadow: dialogs and sheets.
pub const SHADOW_LG: Shadow = Shadow {
    y: 16.0,
    blur: 48.0,
    alpha: 0.16,
};

/// `(alpha, position %)` stops of the eased fade-out gradient used under media captions.
pub const GRADIENT_FADE_DARK: [(f32, f32); 16] = [
    (0.8, 0.0),
    (0.8, 8.1),
    (0.8, 15.5),
    (0.8, 22.5),
    (0.78, 29.0),
    (0.73, 35.3),
    (0.67, 41.2),
    (0.6, 47.1),
    (0.52, 52.9),
    (0.44, 58.8),
    (0.33, 64.7),
    (0.22, 71.0),
    (0.12, 77.5),
    (0.05, 84.5),
    (0.011, 91.9),
    (0.0, 100.0),
];

// ---------------------------------------------------------------------------------- motion

/// Durations `sm / md / lg / xl` in milliseconds.
pub const DURATION_SM: u64 = 180;
pub const DURATION_MD: u64 = 280;
pub const DURATION_LG: u64 = 460;
pub const DURATION_XL: u64 = 800;

/// Easings as cubic-bezier control points: a soft start and a long, gentle settle.
pub const EASE_IN_OUT: (f32, f32, f32, f32) = (0.22, 0.8, 0.2, 1.0);
pub const EASE_IN: (f32, f32, f32, f32) = (0.4, 0.0, 1.0, 1.0);
pub const EASE_OUT: (f32, f32, f32, f32) = (0.0, 0.0, 0.2, 1.0);

// ------------------------------------------------------------------------ focus, layout

/// Focus ring: a 2 px solid outline in the focus colour, offset 2 px.
pub const FOCUS_WIDTH: f32 = 2.0;
pub const FOCUS_OFFSET: f32 = 2.0;

/// Breakpoints `xs … 2xl` (px).
pub const BREAKPOINTS: [(&str, f32); 6] = [
    ("xs", 480.0),
    ("sm", 720.0),
    ("md", 1024.0),
    ("lg", 1360.0),
    ("xl", 1760.0),
    ("2xl", 1920.0),
];

/// The widest the grid grows and the narrowest it shrinks, in px.
pub const GRID_WIDTH_MAX: f32 = 2560.0;
pub const GRID_WIDTH_MIN: f32 = 320.0;

/// Grid gap: the fluid `md` spacing.
pub fn grid_gap(viewport_px: f32) -> f32 {
    SPACING_FLUID[2].1.at(viewport_px)
}

/// The outer margin column width of the page grid at a viewport width.
pub fn grid_safe_zone(viewport_px: f32) -> f32 {
    if viewport_px >= 1920.0 {
        (viewport_px / 2.0 - 840.0).min(360.0)
    } else if viewport_px >= 720.0 {
        viewport_px * 0.06 - 16.0
    } else {
        (viewport_px * 0.08).max(20.0)
    }
}

/// Columns of the page grid between the outer margins: 4 below `sm`, 12 from `sm` up.
pub fn grid_columns(viewport_px: f32) -> u32 {
    if viewport_px >= 720.0 {
        12
    } else {
        4
    }
}

// --------------------------------------------------------------------------- the gradient

/// A gradient stop: position `0.0..=1.0` and the colour there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stop {
    pub at: f32,
    pub color: Hsla,
}

/// The signature dodai gradient of one mode, violet through blue to teal, at
/// [`GRADIENT_ANGLE`] degrees (0 = towards the top, clockwise).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dodai {
    pub stops: [Stop; 3],
}

const fn stop(at: f32, h: f32, s: f32, l: f32) -> Stop {
    Stop {
        at,
        color: hsl(h, s, l),
    }
}

/// Light mode: saturated enough to read on white glass.
pub const DODAI_LIGHT: Dodai = Dodai {
    stops: [
        stop(0.0, 268.0, 82.0, 62.0),
        stop(0.55, 226.0, 90.0, 60.0),
        stop(1.0, 184.0, 80.0, 46.0),
    ],
};

/// Dark mode: the same hues, lit from within.
pub const DODAI_DARK: Dodai = Dodai {
    stops: [
        stop(0.0, 268.0, 90.0, 68.0),
        stop(0.55, 226.0, 100.0, 66.0),
        stop(1.0, 184.0, 90.0, 56.0),
    ],
};

/// The gradient direction in degrees, 0 = towards the top, clockwise: a 135° diagonal.
pub const GRADIENT_ANGLE: f32 = 135.0;

impl Dodai {
    /// The colour at `t` (`0.0..=1.0`), blended between the stops.
    pub fn at(self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let s = &self.stops;
        let (a, b) = if t <= s[1].at {
            (s[0], s[1])
        } else {
            (s[1], s[2])
        };
        let k = ((t - a.at) / (b.at - a.at)).clamp(0.0, 1.0);
        a.color.color().mix(b.color.color(), k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(c: Color) -> u32 {
        (u32::from(c.r) << 16) | (u32::from(c.g) << 8) | u32::from(c.b)
    }

    #[test]
    fn solid_colours_resolve_to_the_documented_hex() {
        assert_eq!(hex(LIGHT.canvas.color()), 0xf8f8fc);
        assert_eq!(hex(DARK.canvas.color()), 0x080613);
        assert_eq!(hex(LIGHT.focus), 0x7c5cff);
        assert_ne!(hex(LIGHT.ink.color()), hex(DARK.ink.color()));
    }

    #[test]
    fn alpha_tokens_keep_their_alpha() {
        assert_eq!(LIGHT.veil.color().a, 115);
        assert_eq!(DARK.ink_25.color().a, 66);
        assert!(LIGHT.glass.color().a > LIGHT.glass_soft.color().a);
    }

    #[test]
    fn contrast_ramps_get_lighter_towards_lower() {
        for (m, c) in [(&LIGHT, LIGHT.canvas.color()), (&DARK, DARK.canvas.color())] {
            let l = |t: Hsla| {
                let c = t.over(c);
                u32::from(c.r) + u32::from(c.g) + u32::from(c.b)
            };
            let ramp = [m.ink_90, m.ink_75, m.ink_60, m.ink_45, m.ink_25].map(l);
            let light = m.canvas.l > 50.0;
            assert!(
                ramp.windows(2)
                    .all(|w| if light { w[0] <= w[1] } else { w[0] >= w[1] }),
                "{ramp:?}"
            );
        }
    }

    #[test]
    fn fluid_scales_clamp_at_both_ends() {
        assert_eq!(spacing_fluid("md", 100.0), Some(16.0));
        assert_eq!(spacing_fluid("md", 4000.0), Some(36.0));
        assert_eq!(spacing_fluid("2xl", 4000.0), Some(200.0));
        assert_eq!(type_size("sm", 1440.0), Some(14.72));
        assert!((type_size("5xl", 4000.0).unwrap() - 6.2 * 16.0).abs() < 0.01);
    }

    #[test]
    fn static_scales_match_the_tables() {
        assert_eq!(spacing_static("lg"), Some(24.0));
        assert_eq!(radius("xl"), Some(16.0));
        assert_eq!(radius("4xl"), Some(40.0));
        assert!(radius("nope").is_none());
    }

    #[test]
    fn grid_follows_the_breakpoints() {
        assert_eq!(grid_columns(600.0), 4);
        assert_eq!(grid_columns(1440.0), 12);
        assert_eq!(grid_safe_zone(2400.0), 360.0);
        assert_eq!(grid_gap(1440.0), 30.0);
    }

    #[test]
    fn the_dodai_runs_from_violet_to_teal() {
        for g in [DODAI_LIGHT, DODAI_DARK] {
            let (a, b) = (g.at(0.0), g.at(1.0));
            assert!(a.b > a.g && b.g > b.r, "{a:?} {b:?}");
            assert_eq!(g.at(0.55), g.stops[1].color.color());
            assert_eq!(g.at(-3.0), a);
        }
    }

    #[test]
    fn gradient_fades_monotonically() {
        assert!(GRADIENT_FADE_DARK
            .windows(2)
            .all(|w| w[0].0 >= w[1].0 && w[0].1 < w[1].1));
    }
}
