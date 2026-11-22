//! **Colour schemes**: a whole Material 3 theme (light or dark) from one seed colour.
//!
//! A [`Scheme`] picks a [`SchemeStyle`] (how saturated and how far the secondary and tertiary
//! hues move from the seed) and a [`Mode`]; tones are fixed per mode so every role pair keeps
//! its contrast whatever the seed. [`Pastel`](SchemeStyle::Pastel) lifts the tones for soft,
//! low-contrast-feeling surfaces while staying readable.
//!
//! ```
//! use dodai_ui_core::material::{Scheme, SchemeStyle};
//! use dodai_ui_core::{Color, Mode};
//! let t = Scheme::new(Color::hex(0x7aa874), SchemeStyle::Pastel, Mode::Light)
//!     .theme(dodai_ui_core::ThemeMeta {
//!         id: "my-mint", name: "Mint", family: "material",
//!         description: "custom", mode: Mode::Light,
//!     });
//! assert!(t.palette.text.contrast(t.palette.background) >= 7.0);
//! ```

use crate::material::{shape, spacing, typography, SPEC};
use crate::themes::ThemeBuilder;
use crate::{Color, Mode, Roles, Theme, ThemeMeta, TonalPalette};

/// How a seed colour is spread over the scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemeStyle {
    /// The Material default: medium chroma, calm secondary, tertiary +60°.
    TonalSpot,
    /// Maximum chroma on every accent.
    Vibrant,
    /// Hue-shifted accents with strong chroma, for playful palettes.
    Expressive,
    /// Almost grey; a hint of the seed only.
    Neutral,
    /// Greyscale.
    Monochrome,
    /// Follows the seed's own chroma, so a muted seed gives a muted scheme.
    Content,
    /// Soft, lifted tones with a tinted surface.
    Pastel,
}

impl SchemeStyle {
    /// Every style.
    pub const ALL: [SchemeStyle; 7] = [
        Self::TonalSpot,
        Self::Vibrant,
        Self::Expressive,
        Self::Neutral,
        Self::Monochrome,
        Self::Content,
        Self::Pastel,
    ];

    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            Self::TonalSpot => "Tonal spot",
            Self::Vibrant => "Vibrant",
            Self::Expressive => "Expressive",
            Self::Neutral => "Neutral",
            Self::Monochrome => "Monochrome",
            Self::Content => "Content",
            Self::Pastel => "Pastel",
        }
    }

    /// `(primary chroma, secondary chroma, secondary hue shift, tertiary chroma,
    /// tertiary hue shift, neutral chroma, neutral-variant chroma)`.
    fn chroma(self, seed_chroma: f32) -> [f32; 7] {
        match self {
            Self::TonalSpot => [36.0, 16.0, 0.0, 24.0, 60.0, 6.0, 8.0],
            Self::Vibrant => [90.0, 28.0, 20.0, 36.0, 60.0, 10.0, 12.0],
            Self::Expressive => [48.0, 24.0, -50.0, 32.0, 120.0, 8.0, 12.0],
            Self::Neutral => [12.0, 8.0, 0.0, 16.0, 60.0, 2.0, 2.0],
            Self::Monochrome => [0.0; 7],
            Self::Content => {
                let p = seed_chroma.max(12.0);
                [
                    p,
                    p / 3.0,
                    0.0,
                    p / 2.0 + 6.0,
                    60.0,
                    (p / 10.0).min(8.0),
                    (p / 10.0).min(8.0) + 4.0,
                ]
            }
            Self::Pastel => [30.0, 20.0, 0.0, 26.0, 50.0, 7.0, 10.0],
        }
    }
}

/// Tones used for each role in one mode.
struct Tones {
    primary: f32,
    on_primary: f32,
    container: f32,
    on_container: f32,
    background: f32,
    /// lowest, low, container, high, highest
    surfaces: [f32; 5],
    on_surface: f32,
    on_surface_variant: f32,
    outline: f32,
    outline_variant: f32,
    disabled: f32,
    inverse_surface: f32,
    inverse_on: f32,
    inverse_primary: f32,
    /// Status colours: tone and chroma.
    status: (f32, f32),
}

fn tones(style: SchemeStyle, mode: Mode) -> Tones {
    let pastel = style == SchemeStyle::Pastel;
    match (mode, pastel) {
        (Mode::Light, false) => Tones {
            primary: 40.0,
            on_primary: 100.0,
            container: 90.0,
            on_container: 10.0,
            background: 98.0,
            surfaces: [100.0, 96.0, 94.0, 92.0, 90.0],
            on_surface: 10.0,
            on_surface_variant: 30.0,
            outline: 50.0,
            outline_variant: 80.0,
            disabled: 60.0,
            inverse_surface: 20.0,
            inverse_on: 95.0,
            inverse_primary: 80.0,
            status: (40.0, 70.0),
        },
        (Mode::Dark, false) => Tones {
            primary: 80.0,
            on_primary: 20.0,
            container: 30.0,
            on_container: 90.0,
            background: 6.0,
            surfaces: [4.0, 10.0, 12.0, 17.0, 22.0],
            on_surface: 90.0,
            on_surface_variant: 80.0,
            outline: 60.0,
            outline_variant: 30.0,
            disabled: 50.0,
            inverse_surface: 90.0,
            inverse_on: 20.0,
            inverse_primary: 40.0,
            status: (80.0, 50.0),
        },
        (Mode::Light, true) => Tones {
            primary: 72.0,
            on_primary: 12.0,
            container: 92.0,
            on_container: 22.0,
            background: 97.0,
            surfaces: [99.0, 96.0, 94.0, 92.0, 89.0],
            on_surface: 18.0,
            on_surface_variant: 38.0,
            outline: 56.0,
            outline_variant: 82.0,
            disabled: 66.0,
            inverse_surface: 26.0,
            inverse_on: 95.0,
            inverse_primary: 80.0,
            status: (72.0, 40.0),
        },
        (Mode::Dark, true) => Tones {
            primary: 78.0,
            on_primary: 18.0,
            container: 28.0,
            on_container: 93.0,
            background: 16.0,
            surfaces: [12.0, 18.0, 21.0, 25.0, 30.0],
            on_surface: 92.0,
            on_surface_variant: 78.0,
            outline: 58.0,
            outline_variant: 36.0,
            disabled: 50.0,
            inverse_surface: 90.0,
            inverse_on: 24.0,
            inverse_primary: 46.0,
            status: (78.0, 36.0),
        },
    }
}

/// One generated scheme: a seed, a style and a mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scheme {
    pub seed: Color,
    pub style: SchemeStyle,
    pub mode: Mode,
}

/// The palettes a scheme samples.
struct Palettes {
    primary: TonalPalette,
    secondary: TonalPalette,
    tertiary: TonalPalette,
    neutral: TonalPalette,
    neutral_variant: TonalPalette,
    error: TonalPalette,
}

impl Scheme {
    pub fn new(seed: Color, style: SchemeStyle, mode: Mode) -> Self {
        Self { seed, style, mode }
    }

    fn palettes(&self) -> Palettes {
        let (_, seed_c, hue) = crate::tonal::to_lch(self.seed);
        let [pc, sc, sh, tc, th, nc, nvc] = self.style.chroma(seed_c);
        Palettes {
            primary: TonalPalette::new(hue, pc),
            secondary: TonalPalette::new(hue + sh, sc),
            tertiary: TonalPalette::new(hue + th, tc),
            neutral: TonalPalette::new(hue, nc),
            neutral_variant: TonalPalette::new(hue, nvc),
            error: TonalPalette::new(25.0, 84.0),
        }
    }

    /// The 25 extended roles for this scheme.
    pub fn roles(&self) -> Roles {
        let (p, t) = (self.palettes(), tones(self.style, self.mode));
        let dark = self.mode == Mode::Dark;
        let (on_c, c) = (t.on_container, t.container);
        let on_acc = t.on_primary;
        Roles {
            surface_container_lowest: p.neutral.tone(t.surfaces[0]),
            surface_container_low: p.neutral.tone(t.surfaces[1]),
            surface_container: p.neutral.tone(t.surfaces[2]),
            surface_container_high: p.neutral.tone(t.surfaces[3]),
            surface_container_highest: p.neutral.tone(t.surfaces[4]),
            on_surface: p.neutral.tone(t.on_surface),
            on_surface_variant: p.neutral_variant.tone(t.on_surface_variant),
            outline: p.neutral_variant.tone(t.outline),
            outline_variant: p.neutral_variant.tone(t.outline_variant),
            primary_container: p.primary.tone(c),
            on_primary_container: p.primary.tone(on_c),
            secondary_container: p.secondary.tone(c),
            on_secondary_container: p.secondary.tone(on_c),
            tertiary: p.tertiary.tone(t.primary),
            on_tertiary: p.tertiary.tone(on_acc),
            tertiary_container: p.tertiary.tone(c),
            on_tertiary_container: p.tertiary.tone(on_c),
            error_container: p.error.tone(c),
            on_error: p.error.tone(t.on_primary),
            on_error_container: p.error.tone(on_c),
            inverse_surface: p.neutral.tone(t.inverse_surface),
            inverse_on_surface: p.neutral.tone(t.inverse_on),
            inverse_primary: p.primary.tone(t.inverse_primary),
            scrim: Color::hex(0x000000),
            shadow: Color::hex(0x000000),
            surface_dim: p.neutral.tone(if dark { 6.0 } else { 87.0 }),
            surface_bright: p.neutral.tone(if dark { 24.0 } else { 98.0 }),
            primary_fixed: p.primary.tone(90.0),
            primary_fixed_dim: p.primary.tone(80.0),
            on_primary_fixed: p.primary.tone(10.0),
            on_primary_fixed_variant: p.primary.tone(30.0),
            secondary_fixed: p.secondary.tone(90.0),
            secondary_fixed_dim: p.secondary.tone(80.0),
            on_secondary_fixed: p.secondary.tone(10.0),
            on_secondary_fixed_variant: p.secondary.tone(30.0),
            tertiary_fixed: p.tertiary.tone(90.0),
            tertiary_fixed_dim: p.tertiary.tone(80.0),
            on_tertiary_fixed: p.tertiary.tone(10.0),
            on_tertiary_fixed_variant: p.tertiary.tone(30.0),
        }
    }

    /// Build the full theme. The meta (id, name, description) is supplied by the caller.
    pub fn theme(&self, meta: ThemeMeta) -> Theme {
        let (p, t) = (self.palettes(), tones(self.style, self.mode));
        let dark = self.mode == Mode::Dark;
        let x = |c: Color| ((c.r as u32) << 16) | ((c.g as u32) << 8) | c.b as u32;
        let primary = p.primary.tone(t.primary);
        let hover = if dark {
            primary.lighten(0.1)
        } else {
            primary.darken(0.1)
        };
        let (st, sc) = t.status;
        let status = |hue: f32| x(TonalPalette::new(hue, sc).tone(st));
        let seed_hue = p.primary.hue;
        let series_c = if self.style == SchemeStyle::Pastel {
            34.0
        } else {
            56.0
        };
        let series_t = if dark {
            76.0
        } else if self.style == SchemeStyle::Pastel {
            66.0
        } else {
            46.0
        };
        let mut series = [0u32; 8];
        for (i, s) in series.iter_mut().enumerate() {
            *s = x(TonalPalette::new(seed_hue + 45.0 * i as f32, series_c).tone(series_t));
        }
        let roles = self.roles();
        let mut th = ThemeBuilder {
            meta: ThemeMeta {
                mode: self.mode,
                ..meta
            },
            background: x(p.neutral.tone(t.background)),
            surface: x(roles.surface_container_low),
            raised: x(roles.surface_container_high),
            border: x(roles.outline_variant),
            border_strong: x(roles.outline),
            text: x(roles.on_surface),
            muted: x(roles.on_surface_variant),
            disabled: x(p.neutral_variant.tone(t.disabled)),
            primary: x(primary),
            primary_hover: x(hover),
            on_primary: x(p.primary.tone(t.on_primary)),
            secondary: x(p.secondary.tone(t.primary)),
            on_secondary: x(p.secondary.tone(t.on_primary)),
            accent: x(roles.tertiary),
            info: status(250.0),
            success: status(145.0),
            warning: status(70.0),
            danger: x(p.error.tone(t.primary)),
            series,
            typography: typography(),
            spacing: spacing(),
            shape: shape(),
            glow: 0.0,
            glow_blur: 0.0,
            scanlines: false,
            hard_shadow: 0.0,
        }
        .build();
        th.palette.roles = roles;
        th.motion = crate::material::motion();
        th.shape_system = Some(crate::material::shape_system());
        th.spec = crate::SpecRef::new(&SPEC);
        th
    }
}

/// A shipped colour preset.
#[derive(Debug, Clone, Copy)]
pub struct Preset {
    /// Id stem (`material-<stem>-light` / `-dark`).
    pub stem: &'static str,
    /// Display name, e.g. `"Pastel Mint"`.
    pub name: &'static str,
    pub seed: u32,
    pub style: SchemeStyle,
    pub description: &'static str,
}

/// The shipped presets (each as a light and a dark theme).
pub const PRESETS: &[Preset] = &[
    Preset {
        stem: "pastel-lavender",
        name: "Pastel Lavender",
        seed: 0xb7a6e8,
        style: SchemeStyle::Pastel,
        description: "Soft lilac on a powdery surface",
    },
    Preset {
        stem: "pastel-mint",
        name: "Pastel Mint",
        seed: 0x8fd1b0,
        style: SchemeStyle::Pastel,
        description: "Fresh mint green, gentle and airy",
    },
    Preset {
        stem: "pastel-peach",
        name: "Pastel Peach",
        seed: 0xf2a98c,
        style: SchemeStyle::Pastel,
        description: "Warm peach and apricot",
    },
    Preset {
        stem: "pastel-sky",
        name: "Pastel Sky",
        seed: 0x8bbbe8,
        style: SchemeStyle::Pastel,
        description: "Cloud blue with a cool tint",
    },
    Preset {
        stem: "ocean",
        name: "Ocean",
        seed: 0x0061a4,
        style: SchemeStyle::TonalSpot,
        description: "Deep blue, tonal spot",
    },
    Preset {
        stem: "forest",
        name: "Forest",
        seed: 0x2e7d32,
        style: SchemeStyle::TonalSpot,
        description: "Leaf green, tonal spot",
    },
    Preset {
        stem: "rose",
        name: "Rose",
        seed: 0xb4004e,
        style: SchemeStyle::Vibrant,
        description: "Vivid raspberry rose",
    },
    Preset {
        stem: "amber",
        name: "Amber",
        seed: 0xc77700,
        style: SchemeStyle::Expressive,
        description: "Golden amber with shifted accents",
    },
];

impl Preset {
    /// The theme of this preset in `mode`.
    pub fn theme(&self, mode: Mode) -> Theme {
        let suffix = if mode == Mode::Dark { "dark" } else { "light" };
        let label = if mode == Mode::Dark { "Dark" } else { "Light" };
        Scheme::new(Color::hex(self.seed), self.style, mode).theme(ThemeMeta {
            id: crate::intern(format!("material-{}-{suffix}", self.stem)),
            name: crate::intern(format!("Material 3 · {} {label}", self.name)),
            family: "material",
            description: self.description,
            mode,
        })
    }
}

/// Every preset, light then dark.
pub fn preset_themes() -> Vec<Theme> {
    PRESETS
        .iter()
        .flat_map(|p| [p.theme(Mode::Light), p.theme(Mode::Dark)])
        .collect()
}
