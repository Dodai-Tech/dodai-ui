//! **Phosphor** — green-on-black CRT terminal: square corners, mono type, scanlines.

use super::Spec;
use crate::{Corners, Mode, Shape, Spacing, Theme, ThemeMeta, Typography};

pub fn phosphor() -> Theme {
    Spec {
        meta: ThemeMeta {
            id: "phosphor",
            name: "Phosphor",
            family: "phosphor",
            description: "Green phosphor CRT, monospace, scanlines",
            mode: Mode::Dark,
        },
        mode: Mode::Dark,
        background: 0x020a04,
        surface: 0x04130a,
        raised: 0x082013,
        border: 0x0f3a20,
        border_strong: 0x1d6b3a,
        text: 0xc9ffd9,
        muted: 0x8fe0aa,
        disabled: 0x4a8a62,
        primary: 0x33ff77,
        primary_hover: 0x7dffa5,
        on_primary: 0x02120a,
        secondary: 0xffb000,
        on_secondary: 0x1a1000,
        accent: 0xffb000,
        info: 0x5ce1ff,
        success: 0x33ff77,
        warning: 0xffb000,
        danger: 0xff5a4d,
        series: [
            0x33ff77, 0xffb000, 0x5ce1ff, 0xff5a4d, 0xc9ffd9, 0x8fe0aa, 0xd68cff, 0xffe566,
        ],
        typography: Typography {
            display: "JetBrains Mono, IBM Plex Mono, Menlo, Consolas, monospace",
            body: "JetBrains Mono, IBM Plex Mono, Menlo, Consolas, monospace",
            mono: "JetBrains Mono, Fira Code, Menlo, Consolas, monospace",
            size_xs: 11.0,
            size_sm: 12.0,
            size_md: 14.0,
            size_lg: 16.0,
            size_xl: 22.0,
            size_xxl: 32.0,
            uppercase_labels: true,
            label_tracking: 0.04,
        },
        spacing: Spacing {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 20.0,
            xl: 32.0,
        },
        shape: Shape {
            corners: Corners::NONE,
            radius: 0.0,
            chamfer: 0.0,
            border_width: 1.0,
            control_height: 32.0,
        },
        glow: 0.55,
        glow_blur: 10.0,
        scanlines: true,
        hard_shadow: 0.0,
    }
    .build()
}
