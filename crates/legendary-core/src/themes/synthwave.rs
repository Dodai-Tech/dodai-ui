//! **Synthwave** — neon magenta/cyan on deep indigo, rounded, heavy glow.

use super::Spec;
use crate::{Corners, Mode, Shape, Spacing, Theme, ThemeMeta, Typography};

pub fn synthwave() -> Theme {
    Spec {
        meta: ThemeMeta {
            id: "synthwave",
            name: "Synthwave '84",
            family: "synthwave",
            description: "Neon magenta and cyan on deep indigo, soft glow",
            mode: Mode::Dark,
        },
        mode: Mode::Dark,
        background: 0x0f0a1f,
        surface: 0x170f2e,
        raised: 0x221747,
        border: 0x3a2a6e,
        border_strong: 0x5b43a8,
        text: 0xf4eeff,
        muted: 0xc7b8ee,
        disabled: 0x7d6fa8,
        primary: 0xff2fd6,
        primary_hover: 0xff6be4,
        on_primary: 0x140a24,
        secondary: 0x24e5ff,
        on_secondary: 0x07202a,
        accent: 0xffd84a,
        info: 0x24e5ff,
        success: 0x4dff9a,
        warning: 0xffb02e,
        danger: 0xff4d6d,
        series: [
            0xff2fd6, 0x24e5ff, 0xffd84a, 0x4dff9a, 0x9d6bff, 0xff7a45, 0xff4d6d, 0xc7b8ee,
        ],
        typography: Typography {
            display: "Orbitron, Audiowide, Rajdhani, Segoe UI, sans-serif",
            body: "Rajdhani, Segoe UI, sans-serif",
            mono: "JetBrains Mono, Fira Code, Menlo, Consolas, monospace",
            size_xs: 11.0,
            size_sm: 13.0,
            size_md: 15.0,
            size_lg: 18.0,
            size_xl: 24.0,
            size_xxl: 36.0,
            uppercase_labels: true,
            label_tracking: 0.06,
        },
        spacing: Spacing {
            xs: 4.0,
            sm: 8.0,
            md: 14.0,
            lg: 24.0,
            xl: 40.0,
        },
        shape: Shape {
            corners: Corners::NONE,
            radius: 10.0,
            chamfer: 0.0,
            border_width: 1.5,
            control_height: 44.0,
        },
        glow: 0.7,
        glow_blur: 20.0,
        scanlines: false,
        hard_shadow: 0.0,
    }
    .build()
}
