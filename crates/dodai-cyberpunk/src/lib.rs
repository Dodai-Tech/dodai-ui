//! **Cyberpunk** — a cyberpunk-inspired theme.
//!
//! Source values (from the site's stylesheet): signature yellow `#fcee0a`
//! (hover `#f2e900`), cyan `#00f0ff`, red `#fe1038`, orange `#ff6000`, pure
//! black surfaces, greys `#333 / #4c4c4c / #cbcbcb`, `Blender Pro` display
//! type, uppercase labels, hard 0-radius edges, 48 px controls and
//! chamfered (cut-corner) buttons/panels.

use dodai_core::{
    Color, Corners, Effects, Mode, Palette, Shape, Spacing, Theme, ThemeMeta, Typography,
};

const YELLOW: u32 = 0xfcee0a;
const YELLOW_HOVER: u32 = 0xf2e900;
const CYAN: u32 = 0x00f0ff;
const RED: u32 = 0xfe1038;
const ORANGE: u32 = 0xff6000;
const MAGENTA: u32 = 0xff2a6d;

/// Cyberpunk moves fast and decisively: short, front-loaded, no overshoot.
fn motion() -> dodai_core::Motion {
    dodai_core::Motion::new(
        std::time::Duration::from_millis(160),
        dodai_core::Easing::CubicBezier(0.1, 0.9, 0.2, 1.0),
    )
    .personality(dodai_core::dodai_motion::Personality::Snappy)
}

pub mod fx;
mod neon;
pub mod shape;
mod spec;
pub use neon::{preset_themes, Backdrop, Neon, NeonPreset, PRESETS};
pub use shape::{shape_presets, CHAMFER_SCALE as SHAPE_SCALE};
pub use spec::{Cyberpunk, SPEC};

fn typography() -> Typography {
    Typography {
        display: "Blender Pro, Rajdhani, Orbitron, Segoe UI, sans-serif",
        body: "Blender Pro, Rajdhani, Segoe UI, sans-serif",
        mono: "JetBrains Mono, Fira Code, Menlo, Consolas, monospace",
        size_xs: 11.0,
        size_sm: 13.0,
        size_md: 15.0,
        size_lg: 18.0,
        size_xl: 24.0,
        size_xxl: 36.0,
        uppercase_labels: true,
        label_tracking: 0.08,
    }
}

fn spacing() -> Spacing {
    Spacing {
        xs: 4.0,
        sm: 8.0,
        md: 16.0,
        lg: 24.0,
        xl: 40.0,
    }
}

fn shape() -> Shape {
    Shape {
        corners: Corners::BOTTOM_RIGHT,
        radius: 0.0,
        chamfer: 16.0,
        border_width: 2.0,
        control_height: 48.0,
    }
}

/// Build a variant from its brand colours.
fn build(
    meta: ThemeMeta,
    primary: u32,
    primary_hover: u32,
    secondary: u32,
    accent: u32,
    scan: bool,
) -> Theme {
    let primary_c = Color::hex(primary);
    let secondary_c = Color::hex(secondary);
    let black = Color::hex(0x000000);
    let palette = Palette {
        background: black,
        surface: Color::hex(0x0a0a0c),
        surface_raised: Color::hex(0x16161a),
        overlay: black.with_alpha(0.88),
        border: Color::hex(0x333333),
        border_strong: Color::hex(0x4c4c4c),
        text: Color::hex(0xffffff),
        text_muted: Color::hex(0xcbcbcb),
        text_disabled: Color::hex(0x7a7a7a),
        primary: primary_c,
        primary_hover: Color::hex(primary_hover),
        primary_active: primary_c.darken(0.18),
        on_primary: black,
        secondary: secondary_c,
        secondary_hover: secondary_c.lighten(0.25),
        on_secondary: black,
        accent: Color::hex(accent),
        info: Color::hex(CYAN),
        success: Color::hex(0x39ff88),
        warning: Color::hex(ORANGE),
        danger: Color::hex(RED),
        selection: primary_c.with_alpha(0.28),
        focus_ring: secondary_c,
        link: secondary_c,
        series: [
            Color::hex(YELLOW),
            Color::hex(CYAN),
            Color::hex(MAGENTA),
            Color::hex(ORANGE),
            Color::hex(0x39ff88),
            Color::hex(0x9d4dff),
            Color::hex(RED),
            Color::hex(0xcbcbcb),
        ],
        roles: dodai_core::Roles::placeholder(),
    }
    .with_derived_roles(true);
    let fx = palette.effect_colours(true);
    Theme {
        meta,
        motion: motion(),
        shape_system: None,
        spec: dodai_core::SpecRef::new(&SPEC),
        surface_style: surface_style(true),
        palette,
        typography: typography(),
        spacing: spacing(),
        shape: shape(),
        effects: Effects {
            glow: primary_c.with_alpha(0.55),
            glow_blur: 14.0,
            glow_intensity: 1.0,
            scanlines: scan,
            hard_shadow: 4.0,
            shadow: fx.0,
            scanline: fx.1,
            scanline_pitch: Effects::SCANLINE_PITCH,
            highlight: fx.2,
        },
    }
}

/// Brand colours of a light variant (darkened neons that hold contrast on paper).
struct Light {
    primary: u32,
    primary_hover: u32,
    on_primary: u32,
    secondary: u32,
    accent: u32,
}

/// Light build: warm paper surfaces, ink text, deep neons, same hard shape.
fn build_light(meta: ThemeMeta, l: Light) -> Theme {
    let primary_c = Color::hex(l.primary);
    let secondary_c = Color::hex(l.secondary);
    let paper = Color::hex(0xf1f0e6);
    let palette = Palette {
        background: paper,
        surface: Color::hex(0xe8e6d9),
        surface_raised: Color::hex(0xfbfaf4),
        overlay: paper.with_alpha(0.9),
        border: Color::hex(0xb5b3a3),
        border_strong: Color::hex(0x6b695c),
        text: Color::hex(0x0a0a0c),
        text_muted: Color::hex(0x3d3c36),
        text_disabled: Color::hex(0x8d8b80),
        primary: primary_c,
        primary_hover: Color::hex(l.primary_hover),
        primary_active: primary_c.darken(0.14),
        on_primary: Color::hex(l.on_primary),
        secondary: secondary_c,
        secondary_hover: secondary_c.darken(0.12),
        on_secondary: Color::hex(0xffffff),
        accent: Color::hex(l.accent),
        info: Color::hex(0x00788a),
        success: Color::hex(0x138a47),
        warning: Color::hex(0xd04a00),
        danger: Color::hex(0xc8102e),
        selection: primary_c.with_alpha(0.28),
        focus_ring: secondary_c,
        link: secondary_c,
        series: [
            Color::hex(0xe0b000),
            Color::hex(0x00909f),
            Color::hex(0xd01c62),
            Color::hex(0xdd5200),
            Color::hex(0x1b9c55),
            Color::hex(0x7a2fe0),
            Color::hex(0xc8102e),
            Color::hex(0x55544a),
        ],
        roles: dodai_core::Roles::placeholder(),
    }
    .with_derived_roles(false);
    let fx = palette.effect_colours(false);
    Theme {
        meta,
        motion: motion(),
        shape_system: None,
        spec: dodai_core::SpecRef::new(&SPEC),
        surface_style: surface_style(false),
        palette,
        typography: typography(),
        spacing: spacing(),
        shape: shape(),
        effects: Effects {
            glow: primary_c.with_alpha(0.3),
            glow_blur: 8.0,
            glow_intensity: 0.4,
            scanlines: false,
            hard_shadow: 4.0,
            shadow: fx.0,
            scanline: fx.1,
            scanline_pitch: Effects::SCANLINE_PITCH,
            highlight: fx.2,
        },
    }
}

/// Yellow on paper.
pub fn yellow_light() -> Theme {
    build_light(
        ThemeMeta {
            id: "cyberpunk-light",
            name: "Cyberpunk · Yellow Light",
            family: "cyberpunk",
            description: "Signature yellow on warm paper, deep cyan secondary",
            mode: Mode::Light,
        },
        Light {
            primary: 0xf0c400,
            primary_hover: 0xffd51f,
            on_primary: 0x000000,
            secondary: 0x006f7d,
            accent: 0xd01c62,
        },
    )
}

/// Cyan on paper.
pub fn cyan_light() -> Theme {
    build_light(
        ThemeMeta {
            id: "cyberpunk-cyan-light",
            name: "Cyberpunk · Cyan Light",
            family: "cyberpunk",
            description: "Deep cyan on warm paper, amber secondary",
            mode: Mode::Light,
        },
        Light {
            primary: 0x00a3b5,
            primary_hover: 0x00bccf,
            on_primary: 0x000000,
            secondary: 0x765a00,
            accent: 0xd01c62,
        },
    )
}

/// Red on paper.
pub fn red_light() -> Theme {
    build_light(
        ThemeMeta {
            id: "cyberpunk-red-light",
            name: "Cyberpunk · Red Light",
            family: "cyberpunk",
            description: "Hot red on warm paper, deep cyan secondary",
            mode: Mode::Light,
        },
        Light {
            primary: 0xd6183a,
            primary_hover: 0xec2a4c,
            on_primary: 0xffffff,
            secondary: 0x006f7d,
            accent: 0x765a00,
        },
    )
}

/// Signature yellow-on-black (the signature cyberpunk look). Default theme.
pub fn yellow() -> Theme {
    build(
        ThemeMeta {
            id: "cyberpunk",
            name: "Cyberpunk · Yellow",
            family: "cyberpunk",
            description: "Signature yellow on black with cyan secondary",
            mode: Mode::Dark,
        },
        YELLOW,
        YELLOW_HOVER,
        CYAN,
        MAGENTA,
        true,
    )
}

/// Cyan-led variant.
pub fn cyan() -> Theme {
    build(
        ThemeMeta {
            id: "cyberpunk-cyan",
            name: "Cyberpunk · Cyan",
            family: "cyberpunk",
            description: "Cyan primary with yellow secondary",
            mode: Mode::Dark,
        },
        CYAN,
        0x5cf6ff,
        YELLOW,
        MAGENTA,
        true,
    )
}

/// Red-led variant.
pub fn red() -> Theme {
    build(
        ThemeMeta {
            id: "cyberpunk-red",
            name: "Cyberpunk · Red",
            family: "cyberpunk",
            description: "Hot red primary with cyan secondary",
            mode: Mode::Dark,
        },
        0xff3355,
        0xff5c75,
        CYAN,
        YELLOW,
        false,
    )
}

/// The default dark theme (yellow on black).
pub fn dark() -> Theme {
    yellow()
}

/// The default light theme (yellow on paper).
pub fn light() -> Theme {
    yellow_light()
}

/// All Cyberpunk variants (dark first, then light).
pub fn variants() -> Vec<Theme> {
    let mut v = vec![
        yellow(),
        cyan(),
        red(),
        yellow_light(),
        cyan_light(),
        red_light(),
    ];
    v.extend(preset_themes());
    v
}

/// The Cyberpunk icon pack (`icons` feature).
#[cfg(feature = "icons")]
pub use dodai_cyberpunk_icons as icons;

/// Cyberpunk owns every texture and the animated effects. The dark variants suggest grain on
/// components, circuit traces on containers with lit lines, a glitch burst and a bloom;
/// the light variants suggest the same, quieter.
fn surface_style(dark: bool) -> dodai_core::SurfaceStyle {
    let calm = if dark { 1.0 } else { 0.6 };
    // Light surfaces show motion more, so the animations run slower there.
    let slow = if dark { 1.0 } else { 1.5 };
    dodai_core::SurfaceStyle {
        preset: dodai_core::SurfaceFx {
            texture: dodai_core::Texture::Grain,
            strength: 0.45 * calm,
            container_texture: dodai_core::Texture::Circuit,
            neon: 0.3,
            container_strength: 0.22 * calm,
            glitch: 0.3,
            flicker: 0.15,
            bloom: 0.3 * calm,
            edge: 0.2 * calm,
            clear: 0.5,
            ..dodai_core::SurfaceFx::NONE
        },
        textures: &dodai_core::Texture::ALL,
        effects: true,
        tuning: dodai_core::SurfaceTuning {
            halo: 1.5,
            glitch_period: 3.2 * slow,
            sheen_period: 3.6 * slow,
            flicker_chance: 0.18 / slow,
            pulse_period: 2.4 * slow,
            chase_period: 4.0 * slow,
            ..dodai_core::SurfaceTuning::DEFAULT
        },
    }
}
