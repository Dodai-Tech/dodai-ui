//! **Arctic** — calm slate/ice-blue, small radius, no glow (a quiet, professional theme).

use super::Spec;
use crate::{Corners, Mode, Shape, Spacing, Theme, ThemeMeta, Typography};

pub fn arctic() -> Theme {
    Spec {
        meta: ThemeMeta {
            id: "arctic",
            name: "Arctic",
            family: "arctic",
            description: "Calm slate and ice-blue, soft corners, no glow",
            mode: Mode::Dark,
        },
        mode: Mode::Dark,
        background: 0x1b2028,
        surface: 0x222833,
        raised: 0x2c3442,
        border: 0x3b4556,
        border_strong: 0x566177,
        text: 0xeceff4,
        muted: 0xb8c2d3,
        disabled: 0x717d92,
        primary: 0x88c0d0,
        primary_hover: 0xa3d1de,
        on_primary: 0x10202a,
        secondary: 0x81a1c1,
        on_secondary: 0x0f1a28,
        accent: 0xebcb8b,
        info: 0x81a1c1,
        success: 0xa3be8c,
        warning: 0xebcb8b,
        danger: 0xe07a85,
        series: [
            0x88c0d0, 0x81a1c1, 0xa3be8c, 0xebcb8b, 0xd08770, 0xb48ead, 0xe07a85, 0xb8c2d3,
        ],
        typography: Typography {
            display: "Inter, Segoe UI, system-ui, sans-serif",
            body: "Inter, Segoe UI, system-ui, sans-serif",
            mono: "JetBrains Mono, Fira Code, Menlo, Consolas, monospace",
            size_xs: 11.0,
            size_sm: 13.0,
            size_md: 15.0,
            size_lg: 18.0,
            size_xl: 24.0,
            size_xxl: 36.0,
            uppercase_labels: false,
            label_tracking: 0.0,
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
            radius: 6.0,
            chamfer: 0.0,
            border_width: 1.0,
            control_height: 36.0,
        },
        glow: 0.0,
        glow_blur: 0.0,
        scanlines: false,
        hard_shadow: 0.0,
    }
    .build()
}
