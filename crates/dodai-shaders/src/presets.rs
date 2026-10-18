//! Ready-made chains: a source, then post effects, with a palette.

use crate::{by_id, ShaderInfo, ShaderPalette};

/// A named, ready-to-play chain.
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    /// Catalogue ids: the first is a generator, the rest are post effects.
    pub chain: &'static [&'static str],
    pub palette: fn() -> ShaderPalette,
}

impl Preset {
    /// The resolved chain.
    pub fn shaders(&self) -> Vec<&'static ShaderInfo> {
        self.chain.iter().filter_map(|id| by_id(id)).collect()
    }
}

/// Retro terminal and monitor presets.
pub const RETRO: &[Preset] = &[
    Preset {
        id: "vt220",
        name: "Amber VT220",
        desc: "Curved amber tube with a halo",
        chain: &["terminal_scene", "amber_monitor"],
        palette: ShaderPalette::amber,
    },
    Preset {
        id: "mainframe",
        name: "Green mainframe",
        desc: "P1 phosphor with a slow vertical roll",
        chain: &["terminal_scene", "green_phosphor", "roll_bar"],
        palette: ShaderPalette::matrix,
    },
    Preset {
        id: "pipboy",
        name: "Pip-Boy",
        desc: "Green tube, grain and heavy vignette",
        chain: &["terminal_scene", "green_phosphor", "film_grain", "vignette"],
        palette: ShaderPalette::matrix,
    },
    Preset {
        id: "nostromo",
        name: "Nostromo",
        desc: "Amber ship terminal: flicker, grain, vignette",
        chain: &["terminal_scene", "amber_monitor", "film_grain", "vignette"],
        palette: ShaderPalette::amber,
    },
    Preset {
        id: "dos",
        name: "DOS text mode",
        desc: "Blue 80×25 screen on a shadow-mask tube",
        chain: &["vga_text", "shadow_mask"],
        palette: ShaderPalette::dos,
    },
    Preset {
        id: "c64",
        name: "Commodore 64",
        desc: "READY. through composite video",
        chain: &["c64_screen", "composite_ntsc", "shadow_mask"],
        palette: ShaderPalette::c64,
    },
    Preset {
        id: "spectrum",
        name: "ZX Spectrum loading",
        desc: "Border stripes while the screen paints in",
        chain: &["zx_loading", "trinitron"],
        palette: ShaderPalette::dos,
    },
    Preset {
        id: "apple2",
        name: "Apple ][",
        desc: "Six artifact colours on a TV",
        chain: &[
            "terminal_scene",
            "apple_ii",
            "composite_ntsc",
            "shadow_mask",
        ],
        palette: ShaderPalette::matrix,
    },
    Preset {
        id: "paper",
        name: "Paper terminal",
        desc: "Black ink on a warm 1-bit screen",
        chain: &["terminal_scene", "paper_white"],
        palette: ShaderPalette::dos,
    },
    Preset {
        id: "vector",
        name: "Vector arcade",
        desc: "Outlines drawn as bright phosphor",
        chain: &["terminal_scene", "vector_glow", "bloom"],
        palette: ShaderPalette::matrix,
    },
    Preset {
        id: "broadcast",
        name: "Broadcast test",
        desc: "Colour bars with tracking trouble",
        chain: &[
            "smpte_bars",
            "composite_ntsc",
            "trinitron",
            "analog_tracking",
        ],
        palette: ShaderPalette::dos,
    },
    Preset {
        id: "radar",
        name: "Radar room",
        desc: "Sweep, afterglow and blips on a tube",
        chain: &["phosphor_radar", "crt"],
        palette: ShaderPalette::matrix,
    },
    Preset {
        id: "haunted",
        name: "Haunted VHS",
        desc: "Tape wear, degauss and grain",
        chain: &[
            "terminal_scene",
            "vhs",
            "analog_tracking",
            "crt_degauss",
            "film_grain",
        ],
        palette: ShaderPalette::cyberpunk,
    },
    Preset {
        id: "printer",
        name: "Dot matrix",
        desc: "Amber dots on a dark grid",
        chain: &["terminal_scene", "dot_matrix"],
        palette: ShaderPalette::amber,
    },
];

/// Ghostty-inspired terminal presets.
pub const GHOSTTY: &[Preset] = &[
    Preset {
        id: "gt_classic",
        name: "Ghostty classic",
        desc: "In-game CRT with a perceptual glow",
        chain: &["terminal_scene", "gt_in_game_crt", "gt_glow_rgb"],
        palette: ShaderPalette::matrix,
    },
    Preset {
        id: "gt_fire",
        name: "Cyber embers",
        desc: "Sparks behind the text on a Lottes tube",
        chain: &["terminal_scene", "gt_sparks", "gt_crt_lottes"],
        palette: ShaderPalette::cyberpunk,
    },
    Preset {
        id: "gt_deep",
        name: "Deep terminal",
        desc: "Rays and drift under glass",
        chain: &["terminal_scene", "gt_underwater", "gt_better_crt"],
        palette: ShaderPalette::cyberpunk,
    },
    Preset {
        id: "gt_lava",
        name: "Lava lounge",
        desc: "Colour-cycling lava with a gentle wobble",
        chain: &["terminal_scene", "gt_lava", "gt_drunkard"],
        palette: ShaderPalette::cyberpunk,
    },
];

/// Every preset.
pub fn all() -> impl Iterator<Item = &'static Preset> {
    RETRO.iter().chain(GHOSTTY.iter())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_resolve() {
        let mut ids = std::collections::HashSet::new();
        for p in all() {
            assert!(ids.insert(p.id), "duplicate preset {}", p.id);
            assert_eq!(p.shaders().len(), p.chain.len(), "{}: unknown id", p.id);
            assert!(
                p.shaders()[0].is_generator(),
                "{}: must start with a generator",
                p.id
            );
            assert!(
                p.shaders()[1..].iter().all(|s| !s.is_generator()),
                "{}",
                p.id
            );
        }
    }
}
