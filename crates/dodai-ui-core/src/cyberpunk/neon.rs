//! **Neon schemes**: a Cyberpunk theme (dark and light) from three accent colours and a backdrop.
//!
//! The dark build keeps the pure neons on a tinted near-black [`Backdrop`]; the light build
//! pulls each neon down to a tone that holds contrast on warm paper. Both keep the hard,
//! chamfered shape, uppercase labels and glow of the design system.
//!
//! ```
//! use dodai_ui_core::cyberpunk::{Backdrop, Neon};
//! use dodai_ui_core::{Color, Mode, ThemeMeta};
//! let neon = Neon::new(Color::hex(0xff2a8a), Color::hex(0x00e5ff), Color::hex(0xffe600))
//!     .backdrop(Backdrop::Violet);
//! let meta = |mode| ThemeMeta { id: "mine", name: "Mine", family: "cyberpunk", description: "", mode };
//! let (dark, light) = (neon.dark(meta(Mode::Dark)), neon.light(meta(Mode::Light)));
//! assert!(dark.palette.link.contrast(dark.palette.background) >= 4.5);
//! assert!(light.palette.link.contrast(light.palette.background) >= 4.5);
//! ```

use crate::cyberpunk::{build, build_light, Light};
use crate::tonal::{from_lch, to_lch};
use crate::{Color, Mode, Theme, ThemeMeta};

/// The tinted near-black the dark build sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backdrop {
    /// Pure black (the signature look).
    Black,
    /// Blue-black.
    Midnight,
    /// Purple-black.
    Violet,
    /// Green-black.
    Moss,
    /// Wine-black.
    Wine,
}

impl Backdrop {
    /// Every backdrop.
    pub const ALL: [Backdrop; 5] = [
        Self::Black,
        Self::Midnight,
        Self::Violet,
        Self::Moss,
        Self::Wine,
    ];

    /// `(background, surface, raised, border, border_strong)`.
    fn colours(self) -> [u32; 5] {
        match self {
            Self::Black => [0x000000, 0x0a0a0c, 0x16161a, 0x333333, 0x4c4c4c],
            Self::Midnight => [0x03060f, 0x070b18, 0x0f1626, 0x1c2740, 0x33415f],
            Self::Violet => [0x07030d, 0x0d0716, 0x181026, 0x2a1d44, 0x46346a],
            Self::Moss => [0x020805, 0x06110b, 0x0d1c13, 0x17321f, 0x2b5a39],
            Self::Wine => [0x0b0306, 0x140810, 0x21101b, 0x3a1c2e, 0x5e2f4a],
        }
    }
}

/// Three neon accents and a backdrop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Neon {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub backdrop: Backdrop,
}

fn rgb(c: Color) -> u32 {
    ((c.r as u32) << 16) | ((c.g as u32) << 8) | c.b as u32
}

/// `c` at the given tone, keeping its hue and (at least `min_chroma`) chroma.
fn at_tone(c: Color, tone: f32, min_chroma: f32) -> Color {
    let (_, chroma, hue) = to_lch(c);
    from_lch(tone, chroma.max(min_chroma), hue)
}

/// Black or white, whichever reads better on `bg`.
fn on(bg: Color) -> Color {
    let (b, w) = (Color::hex(0), Color::hex(0xffffff));
    if b.contrast(bg) >= w.contrast(bg) {
        b
    } else {
        w
    }
}

impl Neon {
    pub fn new(primary: Color, secondary: Color, accent: Color) -> Self {
        Self {
            primary,
            secondary,
            accent,
            backdrop: Backdrop::Black,
        }
    }

    /// Choose the dark backdrop.
    pub fn backdrop(mut self, b: Backdrop) -> Self {
        self.backdrop = b;
        self
    }

    /// The dark theme.
    pub fn dark(&self, meta: ThemeMeta) -> Theme {
        // Neons must read as text on the backdrop: lift a dull secondary until it does.
        let mut secondary = self.secondary;
        let mut tone = to_lch(secondary).0;
        while secondary.contrast(Color::hex(self.backdrop.colours()[0])) < 4.5 && tone < 95.0 {
            tone += 4.0;
            secondary = at_tone(self.secondary, tone, 0.0);
        }
        let mut t = build(
            meta,
            rgb(self.primary),
            rgb(self.primary.lighten(0.22)),
            rgb(secondary),
            rgb(self.accent),
            true,
        );
        let [bg, surface, raised, border, strong] = self.backdrop.colours().map(Color::hex);
        let p = &mut t.palette;
        p.background = bg;
        p.surface = surface;
        p.surface_raised = raised;
        p.overlay = bg.with_alpha(0.88);
        p.border = border;
        // Control outlines: at least 3:1 on the panels they sit on.
        p.border_strong = strong.with_contrast(surface, 3.2);
        p.on_primary = on(self.primary);
        p.on_secondary = on(secondary);
        let palette = t.palette.clone().with_derived_roles(true);
        let fx = palette.effect_colours(true);
        t.effects.shadow = fx.0;
        t.effects.scanline = fx.1;
        t.effects.highlight = fx.2;
        t.palette = palette;
        t.meta.mode = Mode::Dark;
        t
    }

    /// The light theme: neons pulled down to hold contrast on warm paper.
    pub fn light(&self, meta: ThemeMeta) -> Theme {
        let (l, _, _) = to_lch(self.primary);
        let primary = at_tone(self.primary, l.clamp(46.0, 80.0), 60.0);
        let secondary = at_tone(self.secondary, 38.0, 40.0);
        let accent = at_tone(self.accent, 46.0, 50.0);
        build_light(
            meta,
            Light {
                primary: rgb(primary),
                primary_hover: rgb(primary.lighten(0.12)),
                on_primary: rgb(on(primary)),
                secondary: rgb(secondary),
                accent: rgb(accent),
            },
        )
    }
}

/// A shipped neon preset.
#[derive(Debug, Clone, Copy)]
pub struct NeonPreset {
    /// Id stem (`cyberpunk-<stem>` / `cyberpunk-<stem>-light`).
    pub stem: &'static str,
    pub name: &'static str,
    pub primary: u32,
    pub secondary: u32,
    pub accent: u32,
    pub backdrop: Backdrop,
    pub description: &'static str,
}

/// The shipped neon presets (each as a dark and a light theme).
pub const PRESETS: &[NeonPreset] = &[
    NeonPreset {
        stem: "magenta",
        name: "Magenta",
        primary: 0xff2a8a,
        secondary: 0x00e5ff,
        accent: 0xffe600,
        backdrop: Backdrop::Violet,
        description: "Hot magenta over ultraviolet black",
    },
    NeonPreset {
        stem: "acid",
        name: "Acid",
        primary: 0x8dff2f,
        secondary: 0xff2a8a,
        accent: 0x00e5ff,
        backdrop: Backdrop::Moss,
        description: "Acid green on a toxic dark",
    },
    NeonPreset {
        stem: "sunset",
        name: "Sunset",
        primary: 0xff7a1a,
        secondary: 0x8a6cff,
        accent: 0xffd21f,
        backdrop: Backdrop::Wine,
        description: "Burnt orange and dusk violet",
    },
    NeonPreset {
        stem: "violet",
        name: "Ultraviolet",
        primary: 0x9d4dff,
        secondary: 0x00f0ff,
        accent: 0xff2a8a,
        backdrop: Backdrop::Midnight,
        description: "Ultraviolet and cyan on midnight",
    },
    NeonPreset {
        stem: "ice",
        name: "Ice",
        primary: 0x7fd8ff,
        secondary: 0xffd21f,
        accent: 0xff5c8a,
        backdrop: Backdrop::Midnight,
        description: "Cold ice blue with a warm signal",
    },
    NeonPreset {
        stem: "toxic",
        name: "Toxic",
        primary: 0x2fffb0,
        secondary: 0xffb800,
        accent: 0xff3d7f,
        backdrop: Backdrop::Moss,
        description: "Radioactive mint and amber",
    },
    NeonPreset {
        stem: "phantom",
        name: "Phantom",
        primary: 0xfd0130,
        secondary: 0xfcee0a,
        accent: 0x00f0ff,
        backdrop: Backdrop::Black,
        description: "Phantom Liberty red with signal yellow and cyan",
    },
];

impl NeonPreset {
    fn neon(&self) -> Neon {
        Neon::new(
            Color::hex(self.primary),
            Color::hex(self.secondary),
            Color::hex(self.accent),
        )
        .backdrop(self.backdrop)
    }

    /// The preset in `mode`.
    pub fn theme(&self, mode: Mode) -> Theme {
        let dark = mode == Mode::Dark;
        let meta = ThemeMeta {
            id: crate::intern(format!(
                "cyberpunk-{}{}",
                self.stem,
                if dark { "" } else { "-light" }
            )),
            name: crate::intern(format!(
                "Cyberpunk · {}{}",
                self.name,
                if dark { "" } else { " Light" }
            )),
            family: "cyberpunk",
            description: self.description,
            mode,
        };
        if dark {
            self.neon().dark(meta)
        } else {
            self.neon().light(meta)
        }
    }
}

/// Every preset: all dark themes, then all light ones.
pub fn preset_themes() -> Vec<Theme> {
    PRESETS
        .iter()
        .map(|p| p.theme(Mode::Dark))
        .chain(PRESETS.iter().map(|p| p.theme(Mode::Light)))
        .collect()
}
