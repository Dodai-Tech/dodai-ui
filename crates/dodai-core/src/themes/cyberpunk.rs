//! **Cyberpunk** — a cyberpunk-inspired theme.
//!
//! Source values (from the site's stylesheet): signature yellow `#fcee0a`
//! (hover `#f2e900`), cyan `#00f0ff`, red `#fe1038`, orange `#ff6000`, pure
//! black surfaces, greys `#333 / #4c4c4c / #cbcbcb`, `Blender Pro` display
//! type, uppercase labels, hard 0-radius edges, 48 px controls and
//! chamfered (cut-corner) buttons/panels.

use crate::{Color, Corners, Effects, Mode, Palette, Shape, Spacing, Theme, ThemeMeta, Typography};

const YELLOW: u32 = 0xfcee0a;
const YELLOW_HOVER: u32 = 0xf2e900;
const CYAN: u32 = 0x00f0ff;
const RED: u32 = 0xfe1038;
const ORANGE: u32 = 0xff6000;
const MAGENTA: u32 = 0xff2a6d;

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
    };
    Theme {
        meta,
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
        },
    }
}

/// Signature yellow-on-black (the signature cyberpunk look). Default theme.
pub fn night_city() -> Theme {
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
pub fn netrunner() -> Theme {
    build(
        ThemeMeta {
            id: "cyberpunk-netrunner",
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
pub fn samurai() -> Theme {
    build(
        ThemeMeta {
            id: "cyberpunk-samurai",
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

/// All Cyberpunk variants.
pub fn variants() -> Vec<Theme> {
    vec![night_city(), netrunner(), samurai()]
}
