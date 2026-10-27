//! **Neo-Brutalism** — flat saturated colour blocks, thick solid outlines and hard offset
//! shadows that never blur. Modelled on the shadcn-based neobrutalism component kit:
//! a 2 px outline, a 4 px shadow, 5 px corners, black on bright, and controls that *sink*
//! into their shadow on hover.
//!
//! Fourteen themes: seven accents ([`PRESETS`](crate::neobrutalism::PRESETS)), each light and dark. `Paper` is the editorial
//! variant: warm off-white stock, near-black ink, square corners.

use crate::themes::ThemeBuilder;
use crate::{Corners, Mode, Shape, Spacing, Theme, ThemeMeta, Typography};

mod spec;
pub mod surface;
pub use spec::{shape_presets, shape_system, NeoBrutalism, SHAPE_SCALE, SPEC};

/// Chunky and a little bouncy: short, with a slight overshoot.
fn motion() -> crate::Motion {
    crate::Motion::new(
        std::time::Duration::from_millis(150),
        crate::Easing::CubicBezier(0.3, 1.4, 0.5, 1.0),
    )
    .personality(crate::motion::Personality::Playful)
}

fn typography() -> Typography {
    Typography {
        display: "DM Sans, Space Grotesk, Inter, Helvetica Neue, sans-serif",
        body: "DM Sans, Inter, Helvetica Neue, sans-serif",
        mono: "Space Mono, JetBrains Mono, Menlo, Consolas, monospace",
        size_xs: 12.0,
        size_sm: 14.0,
        size_md: 16.0,
        size_lg: 20.0,
        size_xl: 26.0,
        size_xxl: 38.0,
        uppercase_labels: false,
        label_tracking: 0.0,
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
        corners: Corners::NONE,
        radius: 5.0,
        chamfer: 0.0,
        scoop: false,
        border_width: 2.0,
        control_height: 40.0,
    }
}

/// One accent: its "main" colour and the page tint that goes with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preset {
    /// Theme id suffix (`""` for the default).
    pub key: &'static str,
    /// Display name.
    pub name: &'static str,
    /// The main (button / active) colour.
    pub main: u32,
    /// A second block colour.
    pub second: u32,
    /// A third, for accents and charts.
    pub third: u32,
    /// Light page tint.
    pub paper: u32,
    /// Near-black ink for outlines and text in light mode.
    pub ink: u32,
    /// Square corners (no radius).
    pub square: bool,
}

/// The seven accents; the first is the default.
pub const PRESETS: [Preset; 7] = [
    Preset {
        key: "",
        name: "Blue",
        main: 0x5294ff,
        second: 0xfacc00,
        third: 0xff4d50,
        paper: 0xdcebfe,
        ink: BLACK,
        square: false,
    },
    Preset {
        key: "-yellow",
        name: "Yellow",
        main: 0xfacc00,
        second: 0x5294ff,
        third: 0xff90e8,
        paper: 0xfff3c4,
        ink: BLACK,
        square: false,
    },
    Preset {
        key: "-pink",
        name: "Pink",
        main: 0xff90e8,
        second: 0xfacc00,
        third: 0x5294ff,
        paper: 0xffe6f8,
        ink: BLACK,
        square: false,
    },
    Preset {
        key: "-green",
        name: "Green",
        main: 0x05e17a,
        second: 0xff90e8,
        third: 0xfacc00,
        paper: 0xd7f8e7,
        ink: BLACK,
        square: false,
    },
    Preset {
        key: "-red",
        name: "Red",
        main: 0xff4d50,
        second: 0xfacc00,
        third: 0x5294ff,
        paper: 0xffe2e2,
        ink: BLACK,
        square: false,
    },
    Preset {
        key: "-violet",
        name: "Violet",
        main: 0x7a83ff,
        second: 0x05e17a,
        third: 0xfacc00,
        paper: 0xe5e7ff,
        ink: BLACK,
        square: false,
    },
    Preset {
        key: "-paper",
        name: "Paper",
        main: 0x2d81ff,
        second: 0xffb347,
        third: 0xff7ab8,
        paper: 0xf2f1ee,
        ink: 0x101014,
        square: true,
    },
];

const BLACK: u32 = 0x000000;

/// Build one accent in one mode.
pub fn build(p: &Preset, mode: Mode) -> Theme {
    let dark = mode == Mode::Dark;
    let id: &'static str = crate::intern(format!(
        "neobrutalism{}{}",
        p.key,
        if dark { "-dark" } else { "" }
    ));
    let name: &'static str = crate::intern(format!(
        "Neo-Brutalism · {}{}",
        p.name,
        if dark { " · Dark" } else { "" }
    ));
    // Dark: warm graphite paper, off-white outlines (black ones would vanish).
    let (bg, surface, raised, line, text, muted, disabled) = if dark {
        (
            0x272933, 0x1d1f27, 0x343746, 0xf4f4f0, 0xffffff, 0xcfd2dc, 0x7d8194,
        )
    } else {
        (
            p.paper, 0xfaf9f7, 0xe9e8e4, p.ink, p.ink, 0x3b3b3b, 0x8a8a8a,
        )
    };
    let mut t = ThemeBuilder {
        meta: ThemeMeta {
            id,
            name,
            family: "neobrutalism",
            description: "Thick outlines, hard shadows, flat colour",
            mode,
        },
        mode,
        background: bg,
        surface,
        raised,
        border: line,
        border_strong: line,
        text,
        muted,
        disabled,
        primary: p.main,
        primary_hover: p.main,
        on_primary: if dark { BLACK } else { p.ink },
        secondary: p.second,
        on_secondary: if dark { BLACK } else { p.ink },
        accent: p.third,
        info: 0x5294ff,
        success: 0x05e17a,
        warning: 0xfacc00,
        danger: 0xff4d50,
        series: [
            0x5294ff, 0xff4d50, 0xfacc00, 0x05e17a, 0x7a83ff, 0xff90e8, 0xff9f1c, 0x00d4d4,
        ],
        typography: typography(),
        spacing: spacing(),
        shape: if p.square {
            Shape {
                radius: 0.0,
                ..shape()
            }
        } else {
            shape()
        },
        glow: 0.0,
        glow_blur: 0.0,
        scanlines: false,
        hard_shadow: 4.0,
    }
    .build();
    // Keep every "on" role readable on its block.
    t.palette.roles.on_tertiary = t.on(t.palette.roles.tertiary);
    t.palette.roles.on_error = t.on(t.palette.danger);
    // Links are plain ink, like the kit: underline carries the affordance.
    t.palette.link = t.palette.text;
    t.palette.primary_active = crate::Color::hex(p.main);
    t.motion = motion();
    t.surface_style = surface::style(matches!(mode, Mode::Dark));
    t.shape_system = Some(if p.square {
        crate::ShapeSystem::sharp()
    } else {
        shape_system()
    });
    t.spec = crate::SpecRef::new(&SPEC);
    t
}

/// The default theme: blue on pale-blue paper.
pub fn light() -> Theme {
    build(&PRESETS[0], Mode::Light)
}

/// The default dark theme.
pub fn dark() -> Theme {
    build(&PRESETS[0], Mode::Dark)
}

/// Every theme: each accent light, then dark.
pub fn variants() -> Vec<Theme> {
    PRESETS
        .iter()
        .flat_map(|p| [build(p, Mode::Light), build(p, Mode::Dark)])
        .collect()
}
