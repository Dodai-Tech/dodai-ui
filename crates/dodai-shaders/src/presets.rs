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

/// Rainy megacity presets (use with `ShaderPalette::bladerunner`).
pub const BLADE: &[Preset] = &[
    Preset {
        id: "br-rain-city",
        name: "Rain city",
        desc: "Orange smog, neon and a spinner, graded teal and orange",
        chain: &["br_city", "br_grade", "film_grain"],
        palette: ShaderPalette::bladerunner,
    },
    Preset {
        id: "br-spinner",
        name: "Flying car",
        desc: "Anamorphic streaks off every light",
        chain: &["br_city", "br_anamorphic", "br_grade"],
        palette: ShaderPalette::bladerunner,
    },
    Preset {
        id: "br-alley",
        name: "Neon alley",
        desc: "Street level in smog, wet road, umbrellas",
        chain: &["br_street", "br_smog", "br_grade"],
        palette: ShaderPalette::bladerunner,
    },
    Preset {
        id: "br-pyramid",
        name: "Pyramid searchlights",
        desc: "The ziggurat under sweeping beams",
        chain: &["br_tyrell", "br_searchlights", "br_grade"],
        palette: ShaderPalette::bladerunner,
    },
    Preset {
        id: "br-fires",
        name: "Refinery fires",
        desc: "Smokestacks venting flame through smog",
        chain: &["br_off_world", "br_smog", "br_grade", "vignette"],
        palette: ShaderPalette::bladerunner,
    },
    Preset {
        id: "br-eye",
        name: "Empathy test",
        desc: "The scanning iris, graded and grainy",
        chain: &["br_eye", "br_grade", "film_grain"],
        palette: ShaderPalette::bladerunner,
    },
    Preset {
        id: "br-downpour",
        name: "Downpour",
        desc: "Cyberpunk city in a storm with wet pavement",
        chain: &["neon_city", "br_rain", "br_wet"],
        palette: ShaderPalette::cyberpunk,
    },
];

/// Pixel-art scenes with palette-limiting effects.
pub const DMT: &[Preset] = &[
    Preset {
        id: "dmt-breakthrough-flow",
        name: "Breakthrough",
        desc: "The tunnel with its colours sliding round the wheel",
        chain: &["dmt_breakthrough", "dmt_hue_flow"],
        palette: ShaderPalette::hyperspace,
    },
    Preset {
        id: "dmt-elves-tracers",
        name: "Machine elves, tracers",
        desc: "Neon lattices trailing spectral echoes",
        chain: &["dmt_machine_elves", "dmt_tracers"],
        palette: ShaderPalette::hyperspace,
    },
    Preset {
        id: "dmt-aura-body",
        name: "Aura vision",
        desc: "The energy body outlined in rainbow light",
        chain: &["dmt_energy_body", "dmt_aura"],
        palette: ShaderPalette::hyperspace,
    },
    Preset {
        id: "dmt-mandala-kaleido",
        name: "Mandala kaleidoscope",
        desc: "Lotus layers folded into six-fold symmetry, breathing",
        chain: &["dmt_lotus_mandala", "dmt_kaleidoscope", "dmt_breath"],
        palette: ShaderPalette::hyperspace,
    },
    Preset {
        id: "dmt-metatron-drift",
        name: "Metatron drift",
        desc: "The cube melting slowly, with echoes",
        chain: &["dmt_metatron", "dmt_liquid", "dmt_tracers"],
        palette: ShaderPalette::hyperspace,
    },
    Preset {
        id: "dmt-eye-liquid",
        name: "Liquid eye",
        desc: "The third eye underwater, in shifting hues",
        chain: &["dmt_third_eye", "dmt_liquid", "dmt_hue_flow"],
        palette: ShaderPalette::hyperspace,
    },
    Preset {
        id: "dmt-merkaba-grid",
        name: "Merkaba in the lattice",
        desc: "Star tetrahedron over a glowing flower of life",
        chain: &["dmt_merkaba", "dmt_sacred_overlay"],
        palette: ShaderPalette::hyperspace,
    },
    Preset {
        id: "dmt-yantra-kaleido",
        name: "Yantra mirror",
        desc: "The yantra mirrored through six-fold glass",
        chain: &["dmt_sri_yantra", "dmt_kaleidoscope"],
        palette: ShaderPalette::hyperspace,
    },
    Preset {
        id: "dmt-sacred-mirror-aura",
        name: "Sacred mirror, aura",
        desc: "The standing figure outlined in drifting rainbow light",
        chain: &["dmt_sacred_mirror", "dmt_aura"],
        palette: ShaderPalette::hyperspace,
    },
    Preset {
        id: "dmt-julia-kaleido",
        name: "Julia kaleidoscope",
        desc: "The fractal mandala mirrored again and breathing",
        chain: &["dmt_julia_mandala", "dmt_kaleidoscope", "dmt_breath"],
        palette: ShaderPalette::hyperspace,
    },
    Preset {
        id: "dmt-indra-liquid",
        name: "Indra's net, liquid",
        desc: "The jewel web drifting like water, leaving tracers",
        chain: &["dmt_indras_net", "dmt_liquid", "dmt_tracers"],
        palette: ShaderPalette::hyperspace,
    },
    Preset {
        id: "dmt-chakra-hue",
        name: "Chakra flow",
        desc: "The seven wheels with sliding colours",
        chain: &["dmt_chakra_wheels", "dmt_hue_flow"],
        palette: ShaderPalette::hyperspace,
    },
];

pub const PIXEL: &[Preset] = &[
    Preset {
        id: "px-gameboy-city",
        name: "Game Boy city",
        desc: "Rainy skyline in four green shades",
        chain: &["px_city", "gameboy"],
        palette: ShaderPalette::metroidvania,
    },
    Preset {
        id: "px-pico-sunset",
        name: "Pico sunset",
        desc: "Sunset sea snapped to the 16-colour palette",
        chain: &["px_ocean", "pico16"],
        palette: ShaderPalette::metroidvania,
    },
    Preset {
        id: "px-outlined-camp",
        name: "Outlined camp",
        desc: "Campfire with dark pixel outlines",
        chain: &["px_campfire", "px_outline"],
        palette: ShaderPalette::metroidvania,
    },
    Preset {
        id: "px-cave-ramp",
        name: "Dungeon duotone",
        desc: "Torch-lit dungeon on the theme's colour ramp",
        chain: &["px_dungeon", "px_duotone"],
        palette: ShaderPalette::metroidvania,
    },
    Preset {
        id: "px-posterized-reef",
        name: "Posterized reef",
        desc: "Underwater reef in a few dithered levels",
        chain: &["px_reef", "px_posterize"],
        palette: ShaderPalette::metroidvania,
    },
    Preset {
        id: "px-snow-outline",
        name: "Snow village",
        desc: "Village with crisp outlines",
        chain: &["px_snow", "px_outline"],
        palette: ShaderPalette::metroidvania,
    },
];

/// Hand-painted anime presets (use with `ShaderPalette::ghibli`).
pub const GHIBLI: &[Preset] = &[
    Preset {
        id: "gh-golden-cinema",
        name: "Golden hour film",
        desc: "Sunset lake with widescreen bars and warm halation",
        chain: &["gh_sunset", "gh_cinema"],
        palette: ShaderPalette::ghibli,
    },
    Preset {
        id: "gh-rainy-watercolour",
        name: "Rainy watercolour",
        desc: "The lamp-lit stop as a damp watercolour wash",
        chain: &["gh_rain_stop", "gh_watercolor"],
        palette: ShaderPalette::ghibli,
    },
    Preset {
        id: "gh-blossom-rays",
        name: "Blossom breeze",
        desc: "Cherry branch with sunbeams and drifting petals",
        chain: &["gh_sakura", "gh_godrays", "gh_petals"],
        palette: ShaderPalette::ghibli,
    },
    Preset {
        id: "gh-wheat-film",
        name: "Wheat field film",
        desc: "Golden gusts, graded like a feature film",
        chain: &["gh_wheat", "gh_godrays", "gh_cinema"],
        palette: ShaderPalette::ghibli,
    },
    Preset {
        id: "gh-lantern-dream",
        name: "Lantern dream",
        desc: "Sky lanterns with a soft warm bloom",
        chain: &["gh_lanterns", "gh_dream"],
        palette: ShaderPalette::ghibli,
    },
    Preset {
        id: "gh-winter-cabin",
        name: "Winter cabin",
        desc: "Snowy cabin painted onto paper",
        chain: &["gh_snow_cabin", "gh_painted"],
        palette: ShaderPalette::ghibli,
    },
    Preset {
        id: "gh-painted-meadow",
        name: "Painted meadow",
        desc: "Wind in the grass, gouache colour and paper grain",
        chain: &["gh_meadow", "gh_painted"],
        palette: ShaderPalette::ghibli,
    },
    Preset {
        id: "gh-dream-sea",
        name: "Dream sea",
        desc: "Cel waves with a soft warm glow",
        chain: &["gh_sea", "gh_dream"],
        palette: ShaderPalette::ghibli,
    },
    Preset {
        id: "gh-sky-island",
        name: "Sky island",
        desc: "Cel-shaded floating island, inked on paper",
        chain: &["gh_laputa", "gh_painted"],
        palette: ShaderPalette::ghibli,
    },
    Preset {
        id: "gh-green-island",
        name: "Pod on the island",
        desc: "Round capsule ship landed by the sea",
        chain: &["gh_pod", "gh_dream"],
        palette: ShaderPalette::ghibli,
    },
    Preset {
        id: "gh-spirit-forest",
        name: "Spirit forest",
        desc: "Mist, god rays and drifting motes",
        chain: &["gh_forest", "gh_dream"],
        palette: ShaderPalette::ghibli,
    },
];

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
    RETRO
        .iter()
        .chain(GHOSTTY.iter())
        .chain(BLADE.iter())
        .chain(GHIBLI.iter())
        .chain(DMT.iter())
        .chain(PIXEL.iter())
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
