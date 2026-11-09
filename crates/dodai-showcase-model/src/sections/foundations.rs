//! Foundations: text, palette, type scale, spacing, shape, motion, icons.

use dodai_core::{Color, Theme, Tone, TypeRole};
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::{Font, Gap, Ink, Node, TextRole};

const CAT: &str = "Foundations";

pub(crate) const SECTIONS: [Section; 9] = [
    Section::new(CAT, "Text · Heading · Label · Mono", text_section).heading("Text"),
    Section::new(CAT, "Colour palette", palette),
    Section::new(CAT, "Type scale", type_scale),
    Section::new(CAT, "Motion tokens", motion),
    Section::new(CAT, "Spacing · Elevation", spacing),
    Section::new(CAT, "Layout · Fluid scales", layout),
    Section::new(CAT, "Frosted glass", frosted),
    Section::new(CAT, "Shape tokens", shape),
    Section::new(CAT, "Icons", icons),
];

fn text_section(_: &Ctx) -> Node {
    col(vec![
        heading("Heading 1", 1),
        heading("Heading 2", 2),
        heading("Heading 3", 3),
        label("Label text"),
        muted("Muted text"),
        text("mono_text() // 0xDEADBEEF", TextRole::Mono),
    ])
}

/// Small monospaced secondary text (token names and values).
fn mono_xs(t: &Theme, s: impl Into<String>) -> Node {
    styled(s, t.typography.size_xs, Font::Mono, Ink::Muted)
}

fn swatch(t: &Theme, name: &str, c: Color) -> Node {
    Node::Swatch {
        name: name.into(),
        color: c,
        detail: format!("{} · {:.1}:1", c.to_hex(), c.contrast(t.palette.background)),
    }
}

fn palette(c: &Ctx) -> Node {
    let t = c.theme;
    let p = &t.palette;
    let groups: [(&str, Vec<(&str, Color)>); 6] = [
        (
            "Surfaces",
            vec![
                ("background", p.background),
                ("surface", p.surface),
                ("surface_raised", p.surface_raised),
                ("overlay", p.overlay),
                ("border", p.border),
                ("border_strong", p.border_strong),
            ],
        ),
        (
            "Text",
            vec![
                ("text", p.text),
                ("text_muted", p.text_muted),
                ("text_disabled", p.text_disabled),
                ("link", p.link),
            ],
        ),
        (
            "Brand",
            vec![
                ("primary", p.primary),
                ("primary_hover", p.primary_hover),
                ("primary_active", p.primary_active),
                ("on_primary", p.on_primary),
                ("secondary", p.secondary),
                ("on_secondary", p.on_secondary),
                ("accent", p.accent),
            ],
        ),
        (
            "Status",
            vec![
                ("info", p.info),
                ("success", p.success),
                ("warning", p.warning),
                ("danger", p.danger),
                ("selection", p.selection),
                ("focus_ring", p.focus_ring),
            ],
        ),
        (
            "Extended roles (containers, on-colours, outline, inverse)",
            p.roles.list(),
        ),
        (
            "Series",
            (0..8).map(|i| ("series", p.series_at(i))).collect(),
        ),
    ];
    let mut out = vec![muted("Every Palette token of the active design system.")];
    for (g, items) in groups {
        out.push(muted(g));
        out.push(row(items
            .iter()
            .map(|(n, col)| swatch(t, n, *col))
            .collect()));
    }
    out.push(muted(
        "Click a swatch to copy its hex. The ratio is WCAG contrast against the background.",
    ));
    col(out)
}

/// The size steps in the display font, then the three families and the label rules.
fn type_view(t: &Theme) -> Node {
    let ty = &t.typography;
    let mut out = Vec::new();
    for (n, sz) in [
        ("xxl", ty.size_xxl),
        ("xl", ty.size_xl),
        ("lg", ty.size_lg),
        ("md", ty.size_md),
        ("sm", ty.size_sm),
        ("xs", ty.size_xs),
    ] {
        out.push(
            row(vec![
                tile(84.0, vec![mono_xs(t, format!("{n} · {sz:.0}px"))]),
                styled("Sphinx of black quartz", sz, Font::Display, Ink::Text),
            ])
            .gap(Gap::Md),
        );
    }
    out.push(Node::Space(t.spacing.sm));
    for (label, f, font) in [
        ("display", ty.display, Font::Display),
        ("body", ty.body, Font::Body),
        ("mono", ty.mono, Font::Mono),
    ] {
        out.push(
            col(vec![
                muted(format!("{label} · {}", first_family(f))),
                styled(
                    "Pack my box with five dozen liquor jugs 0123456789",
                    ty.size_lg,
                    font,
                    Ink::Text,
                ),
            ])
            .gap(Gap::Px(2.0)),
        );
    }
    out.push(muted(format!(
        "uppercase labels: {} · label tracking {:.2}em",
        ty.uppercase_labels, ty.label_tracking
    )));
    col(out).gap(Gap::Sm)
}

/// The first family of a CSS-style font list (as the gpui showcase prints it).
pub(crate) fn first_family(list: &str) -> String {
    list.split(',').next().unwrap_or(list).trim().to_string()
}

fn type_scale(c: &Ctx) -> Node {
    let t = c.theme;
    let roles = TypeRole::ALL
        .iter()
        .map(|&r| {
            let s = t.type_style(r);
            row(vec![
                tile(
                    190.0,
                    vec![mono_xs(
                        t,
                        format!(
                            "{} · {:.0}/{:.0} · {}",
                            r.name(),
                            s.size,
                            s.line_height,
                            s.weight
                        ),
                    )],
                ),
                text("Sphinx of black quartz", TextRole::Type(r)),
            ])
            .gap(Gap::Md)
        })
        .collect();
    col(vec![
        muted("Sizes and families from the theme's Typography."),
        type_view(t),
        muted("Type scale roles (Material 3: exact; others derive them)"),
        col(roles).gap(Gap::Xs),
    ])
}

fn motion(c: &Ctx) -> Node {
    use dodai_core::motion::{material_easing as e, DurationToken};
    let t = c.theme;
    let mut out = Vec::new();
    for d in DurationToken::ALL {
        out.push(
            line(vec![
                tile(
                    130.0,
                    vec![mono_xs(t, format!("{} · {} ms", d.name(), d.millis()))],
                ),
                Node::Block {
                    w: d.millis() as f32 * 0.5,
                    h: 8.0,
                    fill: t.palette.primary,
                    border: None,
                    radius: 4.0,
                    label: None,
                },
            ])
            .gap(Gap::Md),
        );
    }
    for (name, curve) in [
        ("emphasized", e::EMPHASIZED),
        ("emphasized-decelerate", e::EMPHASIZED_DECELERATE),
        ("emphasized-accelerate", e::EMPHASIZED_ACCELERATE),
        ("standard-decelerate", e::STANDARD_DECELERATE),
        ("standard-accelerate", e::STANDARD_ACCELERATE),
    ] {
        out.push(mono_xs(t, format!("{name}: {}", curve.to_css())));
    }
    col(vec![
        muted("Material 3 duration scale (bar = duration) and named easing curves."),
        col(out).gap(Gap::Px(4.0)),
    ])
}

fn spacing(c: &Ctx) -> Node {
    let t = c.theme;
    let p = &t.palette;
    let sp = &t.spacing;
    let mut out = Vec::new();
    for (n, v) in [
        ("xs", sp.xs),
        ("sm", sp.sm),
        ("md", sp.md),
        ("lg", sp.lg),
        ("xl", sp.xl),
    ] {
        out.push(
            line(vec![
                tile(84.0, vec![mono_xs(t, format!("{n} · {v:.0}px"))]),
                Node::Block {
                    w: v * 8.0,
                    h: 10.0,
                    fill: p.primary,
                    border: None,
                    radius: 0.0,
                    label: None,
                },
            ])
            .gap(Gap::Md),
        );
    }
    let layer = |n: &str, c: Color| Node::Block {
        w: 132.0,
        h: 64.0,
        fill: c,
        border: Some(p.border),
        radius: 0.0,
        label: Some(n.into()),
    };
    out.push(Node::Space(sp.sm));
    out.push(muted("Elevation layers"));
    out.push(row(vec![
        layer("background", p.background),
        layer("surface", p.surface),
        layer("surface_raised", p.surface_raised),
        layer("overlay", p.overlay),
    ]));
    col(vec![
        muted("Spacing scale (bars are 8x the value) and surface layers."),
        col(out).gap(Gap::Sm),
    ])
}

/// The token table and a live panel sample (shared with the Shape gallery).
pub(crate) fn shape_tokens_view(t: &Theme) -> Node {
    let (sh, fx) = (&t.shape, &t.effects);
    let rows: [(&str, String); 10] = [
        ("kind", format!("{:?}", sh.kind())),
        ("radius", format!("{:.1}px", sh.radius)),
        ("chamfer", format!("{:.1}px", sh.chamfer)),
        ("border width", format!("{:.2}px", sh.border_width)),
        ("control height", format!("{:.0}px", sh.control_height)),
        ("glow blur", format!("{:.0}px", fx.glow_blur)),
        ("glow intensity", format!("{:.2}", fx.glow_intensity)),
        ("hard shadow", format!("{:.0}px", fx.hard_shadow)),
        ("scanlines", fx.scanlines.to_string()),
        (
            "labels",
            if t.typography.uppercase_labels {
                "UPPERCASE"
            } else {
                "as written"
            }
            .into(),
        ),
    ];
    let size = t.typography.size_sm;
    let list = rows
        .into_iter()
        .map(|(k, v)| {
            line(vec![
                tile(110.0, vec![styled(k, size, Font::Body, Ink::Muted)]),
                styled(v, size, Font::Mono, Ink::Text),
            ])
            .gap(Gap::Md)
            .start()
        })
        .collect();
    row(vec![
        col(list).gap(Gap::Px(4.0)).width(240.0),
        Node::Sample {
            w: 200.0,
            h: 96.0,
            label: "Live sample".into(),
        },
    ])
    .gap(Gap::Xl)
}

/// Two frosted panels over a hard-edged picture in the theme's colours (as wide as the gpui
/// demo on a wide window).
fn frosted(c: &Ctx) -> Node {
    let t = c.theme;
    let f = t.frost();
    let (w, h) = (720.0_f32, 300.0_f32);
    let pw = (w * 0.42).floor();
    let panel = |title: &str, body: String| {
        col(vec![
            styled(title, t.typography.size_md, Font::Body, Ink::Text),
            styled(body, t.typography.size_sm, Font::Body, Ink::Text),
        ])
        .gap(Gap::Xs)
    };
    col(vec![
        muted("Panels over a picture: a blurred copy of it under the design's frosted tint."),
        Node::Frosted {
            w,
            h,
            panels: vec![
                crate::node::FrostPanel {
                    rect: [(w * 0.05).floor(), 30.0, pw, 120.0],
                    body: panel(
                        "Frosted",
                        format!(
                            "blur {:.0} px, tint alpha {:.2}",
                            f.blur,
                            f32::from(f.tint.a) / 255.0
                        ),
                    ),
                },
                crate::node::FrostPanel {
                    rect: [(w * 0.53).floor(), 110.0, pw, 150.0],
                    body: panel(
                        "Over hard edges",
                        "The picture behind is sharp; each panel shows a blurred copy of it."
                            .into(),
                    ),
                },
            ],
        },
    ])
}

fn shape(c: &Ctx) -> Node {
    col(vec![
        muted("Read-only summary of the theme's shape and effect tokens."),
        shape_tokens_view(c.theme),
    ])
}

fn icons(_: &Ctx) -> Node {
    use IconId::*;
    let ids = [
        Home, Search, Settings, Star, Heart, Edit, Copy, Check, Close, Menu, Folder, File, Bell,
        Person, Mail, Calendar, Cloud, Code, Dashboard, Download, Eye, Flag, Bookmark, Archive,
    ];
    col(vec![
        muted("The shared icon vocabulary, drawn in each design's own style."),
        row(ids.iter().map(|&i| Node::Icon(i, 24.0)).collect()),
        row(vec![
            tag("success", Tone::Success, false),
            tag("danger", Tone::Danger, false),
        ]),
    ])
}

/// Type and spacing resolved at five viewport widths, the page grid at 800 px, and the media
/// fade.
fn layout(c: &Ctx) -> Node {
    let t = c.theme;
    let mut table = vec![mono_xs(
        t,
        "viewport · type xs sm md lg xl xxl · spacing md lg · grid cols gap margin",
    )];
    for w in [480.0, 760.0, 1000.0, 1440.0, 1920.0] {
        table.push(mono_xs(t, layout_line(t, w)));
    }
    col(vec![
        muted("Type and spacing resolved at five viewport widths, the page grid at 800 px, and the media fade."),
        col(vec![
            col(table).gap(Gap::Xs),
            page_grid(t, 800.0, &[(4, "4"), (4, "4"), (8, "8"), (16, "16 (clamped to the grid)")]),
            Node::Image {
                src: None,
                video: false,
                contain: false,
                w: 360.0,
                h: 180.0,
                caption: Some("Caption on the design's fade".into()),
                on_image: true,
            },
        ]),
    ])
}

/// Width of `span` columns of the page grid (as the gpui `PageGrid`).
pub(crate) fn grid_span_width(inner: f32, columns: u32, gap: f32, span: u32) -> f32 {
    let columns = columns.max(1);
    let span = span.clamp(1, columns);
    let col = (inner - gap * (columns - 1) as f32) / columns as f32;
    col * span as f32 + gap * (span - 1) as f32
}

/// Cells laid on the design's page grid at `viewport` px.
fn page_grid(t: &Theme, viewport: f32, cells: &[(u32, &str)]) -> Node {
    let g = t.page_grid(viewport);
    let page = g.max_width.map_or(viewport, |m| viewport.min(m));
    let inner = (page - 2.0 * g.margin).max(0.0);
    let items = cells
        .iter()
        .map(|&(span, l)| Node::Block {
            w: grid_span_width(inner, g.columns, g.gap, span),
            h: t.sizes().touch_target,
            fill: t.palette.surface_raised,
            border: Some(t.palette.border),
            radius: 0.0,
            label: Some(l.into()),
        })
        .collect();
    row(items).gap(Gap::Px(g.gap)).width(inner)
}

/// One row of the fluid-scale table.
fn layout_line(t: &Theme, w: f32) -> String {
    let r = t.responsive(w);
    let g = t.page_grid(w);
    let ty = &r.typography;
    format!(
        "{w:>5.0} · {:>4.1} {:>4.1} {:>4.1} {:>4.1} {:>4.1} {:>4.1} · {:>4.1} {:>4.1} · {:>2} {:>4.1} {:>5.1}",
        ty.size_xs,
        ty.size_sm,
        ty.size_md,
        ty.size_lg,
        ty.size_xl,
        ty.size_xxl,
        r.spacing.md,
        r.spacing.lg,
        g.columns,
        g.gap,
        g.margin,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_table_has_a_row_per_width_with_all_columns() {
        let t = dodai_core::themes::neutral();
        let line = layout_line(&t, 1000.0);
        assert!(line.trim_start().starts_with("1000 ·"));
        assert_eq!(line.matches('·').count(), 3);
    }
}

#[cfg(test)]
mod grid_tests {
    use super::grid_span_width;

    #[test]
    fn spans_add_up_to_the_row_and_clamp_to_the_grid() {
        let (inner, cols, gap) = (1000.0, 12, 20.0);
        let half = grid_span_width(inner, cols, gap, 6);
        assert!((half * 2.0 + gap - inner).abs() < 0.01);
        assert_eq!(
            grid_span_width(inner, cols, gap, 99),
            grid_span_width(inner, cols, gap, 12)
        );
    }

    #[test]
    fn first_family_is_the_first_entry_of_the_list() {
        assert_eq!(super::first_family("Inter, sans-serif"), "Inter");
        assert_eq!(super::first_family(""), "");
    }
}
