//! How the shaders are grouped for browsing: by what they look like, not by whose look they
//! belong to. A source group holds generators, an effect group holds post effects, and the two
//! lists never share a name, so a group name alone says which kind it is.

use crate::ShaderInfo;

/// The groups of sources, in the order a gallery shows them.
pub const SOURCE_GROUPS: [&str; 8] = [
    "Waves & flow",
    "Space & sacred geometry",
    "Neon & HUDs",
    "Cities & streets",
    "Landscapes & weather",
    "Game & pixel scenes",
    "Terminals & test screens",
    "Material 3 shapes",
];

/// The groups of effects, in the order a gallery shows them.
pub const EFFECT_GROUPS: [&str; 6] = [
    "Glow & light",
    "CRT & screens",
    "Glitch & distortion",
    "Colour & grade",
    "Palettes & pixels",
    "Overlays & particles",
];

/// The group of a shader, or `None` for an id this table does not know (a test pins that there
/// is none).
pub fn group(s: &ShaderInfo) -> Option<&'static str> {
    let ids: [(&str, &[&str]); 14] = [
        (
            SOURCE_GROUPS[0],
            &[
                "glow_waves",
                "m3_waves",
                "sine_waves",
                "interference",
                "gradient_flow",
                "gt_noise_flow",
                "plasma",
                "smoke",
                "aurora",
                "ocean",
                "caustics",
                "lava_lamp",
                "metaballs",
                "m3_blobs",
                "m3_mesh",
                "voronoi",
                "rain_drops",
                "spotlight",
                "gh_sea",
                "pixel_sea",
            ],
        ),
        (
            SOURCE_GROUPS[1],
            &[
                "starfield",
                "galaxy",
                "tunnel",
                "julia",
                "mandelbrot_zoom",
                "gt_gears",
                "dmt_breakthrough",
                "dmt_machine_elves",
                "dmt_flower_of_life",
                "dmt_metatron",
                "dmt_sri_yantra",
                "dmt_merkaba",
                "dmt_platonic",
                "dmt_lotus_mandala",
                "dmt_energy_body",
                "dmt_third_eye",
                "dmt_julia_mandala",
                "dmt_apollonian",
                "dmt_sierpinski",
                "dmt_phyllotaxis",
                "dmt_tree_of_life",
                "dmt_chakra_wheels",
                "dmt_sacred_mirror",
                "dmt_indras_net",
            ],
        ),
        (
            SOURCE_GROUPS[2],
            &[
                "neon_grid",
                "hex_pulse",
                "circuit",
                "hud_radar",
                "data_corrupt",
                "kv_glitch",
                "phantom_liberty",
                "synthwave",
                "neon_signs",
                "rain_glass",
                "br_eye",
            ],
        ),
        (
            SOURCE_GROUPS[3],
            &[
                "neon_city",
                "br_city",
                "br_street",
                "br_tyrell",
                "br_off_world",
                "px_city",
                "gh_rain_stop",
                "gh_night_train",
            ],
        ),
        (
            SOURCE_GROUPS[4],
            &[
                "gh_laputa",
                "gh_pod",
                "gh_meadow",
                "gh_clouds",
                "gh_forest",
                "gh_sunset",
                "gh_lanterns",
                "gh_wheat",
                "gh_snow_cabin",
                "gh_sakura",
                "gh_soot",
                "px_ocean",
                "px_forest",
                "px_snow",
                "px_reef",
                "forest_parallax",
                "pixel_mountains",
                "snow",
                "clouds",
                "lightning",
                "embers",
                "fireworks",
            ],
        ),
        (
            SOURCE_GROUPS[5],
            &[
                "px_space",
                "px_dungeon",
                "px_campfire",
                "platformer",
                "metroid_cave",
                "castle_hall",
                "pixel_items",
                "invaders",
                "mario_sky",
            ],
        ),
        (
            SOURCE_GROUPS[6],
            &[
                "digital_rain",
                "matrix_hallway",
                "matrix_wall",
                "matrix_depth",
                "gt_inside_matrix",
                "terminal_scene",
                "vga_text",
                "c64_screen",
                "zx_loading",
                "teletext",
                "test_pattern",
                "smpte_bars",
                "tv_snow",
                "phosphor_radar",
                "oscilloscope",
            ],
        ),
        (
            SOURCE_GROUPS[7],
            &[
                "m3_ripple",
                "m3_elevation",
                "m3_morph",
                "m3_stripes",
                "m3_dots",
                "m3_loader",
                "m3_bars",
            ],
        ),
        (
            EFFECT_GROUPS[0],
            &[
                "bloom",
                "br_anamorphic",
                "br_smog",
                "br_searchlights",
                "energy_rise",
                "gh_godrays",
                "gh_dream",
                "gt_underwater",
                "gt_glow_rgb",
                "neon_edges",
                "vector_glow",
                "dmt_aura",
                "dmt_tracers",
                "hologram",
            ],
        ),
        (
            EFFECT_GROUPS[1],
            &[
                "crt",
                "phosphor",
                "tft",
                "crt_boot",
                "roll_bar",
                "vhs",
                "composite_ntsc",
                "analog_tracking",
                "crt_degauss",
                "amber_monitor",
                "green_phosphor",
                "paper_white",
                "trinitron",
                "shadow_mask",
                "dot_matrix",
                "gt_retro_terminal",
                "gt_better_crt",
                "gt_crt_lottes",
                "gt_in_game_crt",
                "matrix_vision",
                "matrix_tint",
                "static_noise",
            ],
        ),
        (
            EFFECT_GROUPS[2],
            &[
                "glitch",
                "chromatic",
                "gt_glitchy",
                "gt_drunkard",
                "water_warp",
                "heat_haze",
                "dmt_liquid",
                "dmt_breath",
                "kaleidoscope",
                "dmt_kaleidoscope",
            ],
        ),
        (
            EFFECT_GROUPS[3],
            &[
                "br_grade",
                "gh_cinema",
                "gh_painted",
                "gh_watercolor",
                "sepia",
                "negative",
                "vignette",
                "film_grain",
                "dmt_hue_flow",
                "m3_scrim",
                "m3_blur",
            ],
        ),
        (
            EFFECT_GROUPS[4],
            &[
                "dither",
                "cga",
                "c64",
                "apple_ii",
                "zx_spectrum",
                "ega",
                "pico16",
                "gameboy",
                "pixelate",
                "posterize",
                "px_posterize",
                "px_duotone",
                "px_outline",
                "halftone",
                "ascii_art",
            ],
        ),
        (
            EFFECT_GROUPS[5],
            &[
                "br_rain",
                "br_wet",
                "gh_rain",
                "gh_petals",
                "gt_sparks",
                "gt_smoke",
                "gt_lava",
                "gt_cubes",
                "gt_gradient_bg",
                "gt_animated_gradient",
                "dmt_sacred_overlay",
            ],
        ),
    ];
    ids.iter()
        .find(|(_, list)| list.contains(&s.id))
        .map(|(g, _)| *g)
}

/// Shaders worth seeing first; a gallery lists them at the top of their group.
pub const FEATURED: [&str; 10] = [
    "glow_waves",
    "energy_rise",
    "br_searchlights",
    "neon_city",
    "aurora",
    "gh_forest",
    "dmt_flower_of_life",
    "px_city",
    "crt",
    "vhs",
];

/// Whether a shader is one of the [`FEATURED`] ones.
pub fn is_featured(s: &ShaderInfo) -> bool {
    FEATURED.contains(&s.id)
}

/// The shaders of one group: the featured ones first, then the rest, each by name.
pub fn in_group(name: &str) -> Vec<&'static ShaderInfo> {
    let mut v: Vec<_> = crate::catalog()
        .iter()
        .filter(|s| group(s) == Some(name))
        .collect();
    v.sort_by_key(|s| (!is_featured(s), s.name));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shader_has_a_group_of_its_own_kind() {
        for s in crate::catalog() {
            let g = group(s).unwrap_or_else(|| panic!("`{}` has no group", s.id));
            let sources = SOURCE_GROUPS.contains(&g);
            assert_eq!(sources, s.is_generator(), "`{}` is in `{g}`", s.id);
        }
    }

    #[test]
    fn no_shader_is_listed_twice_and_no_group_is_empty() {
        for g in SOURCE_GROUPS.iter().chain(&EFFECT_GROUPS) {
            assert!(!in_group(g).is_empty(), "`{g}` is empty");
        }
        let mut seen = std::collections::HashSet::new();
        for g in SOURCE_GROUPS.iter().chain(&EFFECT_GROUPS) {
            for s in in_group(g) {
                assert!(seen.insert(s.id), "`{}` appears twice", s.id);
            }
        }
        assert_eq!(seen.len(), crate::catalog().len());
    }

    #[test]
    fn featured_ones_exist_and_come_first_then_by_name() {
        for id in FEATURED {
            assert!(crate::by_id(id).is_some(), "`{id}` is not a shader");
        }
        let v = in_group("Glow & light");
        let firsts = v.iter().take_while(|s| is_featured(s)).count();
        assert!(firsts > 0);
        assert!(v[firsts..].iter().all(|s| !is_featured(s)));
        assert!(v[firsts..].windows(2).all(|w| w[0].name <= w[1].name));
    }
}
