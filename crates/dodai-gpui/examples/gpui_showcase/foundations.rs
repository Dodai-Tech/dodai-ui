//! Foundations: read-only views of the active theme's tokens.

use dodai_gpui::{hsla, Muted};
use gpui::{div, prelude::*, px};

// ── Foundations: read-only views of the active theme's tokens ──────────────

pub(crate) fn swatch(
    id: usize,
    name: &str,
    c: dodai_core::Color,
    t: &dodai_core::Theme,
) -> gpui::AnyElement {
    let hex = c.to_hex();
    let copy = hex.clone();
    let on_bg = c.contrast(t.palette.background);
    div()
        .id(("sw", id))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .w(px(118.0))
        .cursor_pointer()
        .on_click(move |_, _, cx| {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(copy.clone()))
        })
        .child(
            div()
                .h(px(44.0))
                .w_full()
                .bg(hsla(c))
                .border_1()
                .border_color(hsla(t.palette.border)),
        )
        .child(
            div()
                .text_size(px(t.typography.size_xs))
                .text_color(hsla(t.palette.text))
                .child(name.to_string()),
        )
        .child(
            div()
                .font_family(dodai_gpui::text::first_family(t.typography.mono))
                .text_size(px(t.typography.size_xs))
                .text_color(hsla(t.palette.text_muted))
                .child(format!("{hex} · {on_bg:.1}:1")),
        )
        .into_any_element()
}

pub(crate) fn palette_view(t: &dodai_core::Theme) -> gpui::Div {
    let p = &t.palette;
    let groups: [(&str, Vec<(&str, dodai_core::Color)>); 6] = [
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
    let mut out = div().flex().flex_col().gap(px(t.spacing.md));
    let mut id = 0;
    for (g, items) in groups {
        let mut row = div().flex().flex_wrap().gap(px(t.spacing.md));
        for (n, c) in items {
            row = row.child(swatch(id, n, c, t));
            id += 1;
        }
        out = out.child(Muted::new(g)).child(row);
    }
    out.child(Muted::new(
        "Click a swatch to copy its hex. The ratio is WCAG contrast against the background.",
    ))
}

/// The 15 type-scale roles with their resolved size / line height / weight.
pub(crate) fn type_scale_view(t: &dodai_core::Theme) -> gpui::Div {
    let mut out = div().flex().flex_col().gap(px(t.spacing.xs));
    for role in dodai_core::TypeRole::ALL {
        let s = t.type_style(role);
        out = out.child(
            div()
                .flex()
                .items_baseline()
                .gap(px(t.spacing.md))
                .child(
                    div()
                        .w(px(190.0))
                        .flex_none()
                        .font_family(dodai_gpui::text::first_family(t.typography.mono))
                        .text_size(px(t.typography.size_xs))
                        .text_color(hsla(t.palette.text_muted))
                        .child(format!(
                            "{} · {:.0}/{:.0} · {}",
                            role.name(),
                            s.size,
                            s.line_height,
                            s.weight
                        )),
                )
                .child(dodai_gpui::Text::new(role, "Sphinx of black quartz")),
        );
    }
    out
}

pub(crate) fn type_view(t: &dodai_core::Theme) -> gpui::Div {
    let ty = &t.typography;
    let mut out = div().flex().flex_col().gap(px(t.spacing.sm));
    for (n, sz) in [
        ("xxl", ty.size_xxl),
        ("xl", ty.size_xl),
        ("lg", ty.size_lg),
        ("md", ty.size_md),
        ("sm", ty.size_sm),
        ("xs", ty.size_xs),
    ] {
        out = out.child(
            div()
                .flex()
                .items_baseline()
                .gap(px(t.spacing.md))
                .child(
                    div()
                        .w(px(84.0))
                        .flex_none()
                        .font_family(dodai_gpui::text::first_family(ty.mono))
                        .text_size(px(ty.size_xs))
                        .text_color(hsla(t.palette.text_muted))
                        .child(format!("{n} · {sz:.0}px")),
                )
                .child(
                    div()
                        .font_family(dodai_gpui::text::first_family(ty.display))
                        .text_size(px(sz))
                        .text_color(hsla(t.palette.text))
                        .child("Sphinx of black quartz"),
                ),
        );
    }
    let fam = |label: &str, f: &'static str| {
        div()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(Muted::new(format!(
                "{label} · {}",
                dodai_gpui::text::first_family(f)
            )))
            .child(
                div()
                    .font_family(dodai_gpui::text::first_family(f))
                    .text_size(px(ty.size_lg))
                    .text_color(hsla(t.palette.text))
                    .child("Pack my box with five dozen liquor jugs 0123456789"),
            )
    };
    out.child(div().h(px(t.spacing.sm)))
        .child(fam("display", ty.display))
        .child(fam("body", ty.body))
        .child(fam("mono", ty.mono))
        .child(Muted::new(format!(
            "uppercase labels: {} · label tracking {:.2}em",
            ty.uppercase_labels, ty.label_tracking
        )))
}

pub(crate) fn spacing_elevation_view(t: &dodai_core::Theme) -> gpui::Div {
    let sp = &t.spacing;
    let mut out = div().flex().flex_col().gap(px(t.spacing.sm));
    for (n, v) in [
        ("xs", sp.xs),
        ("sm", sp.sm),
        ("md", sp.md),
        ("lg", sp.lg),
        ("xl", sp.xl),
    ] {
        out = out.child(
            div()
                .flex()
                .items_center()
                .gap(px(t.spacing.md))
                .child(
                    div()
                        .w(px(84.0))
                        .flex_none()
                        .font_family(dodai_gpui::text::first_family(t.typography.mono))
                        .text_size(px(t.typography.size_xs))
                        .text_color(hsla(t.palette.text_muted))
                        .child(format!("{n} · {v:.0}px")),
                )
                .child(div().h(px(10.0)).w(px(v * 8.0)).bg(hsla(t.palette.primary))),
        );
    }
    let layer = |n: &str, c: dodai_core::Color| {
        div()
            .flex()
            .items_center()
            .justify_center()
            .w(px(132.0))
            .h(px(64.0))
            .bg(hsla(c))
            .border_1()
            .border_color(hsla(t.palette.border))
            .text_size(px(t.typography.size_xs))
            .text_color(hsla(t.palette.text))
            .child(n.to_string())
    };
    out.child(div().h(px(t.spacing.sm)))
        .child(Muted::new("Elevation layers"))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(t.spacing.md))
                .child(layer("background", t.palette.background))
                .child(layer("surface", t.palette.surface))
                .child(layer("surface_raised", t.palette.surface_raised))
                .child(layer("overlay", t.palette.overlay)),
        )
}

pub(crate) fn shape_tokens_view(t: &dodai_core::Theme) -> gpui::Div {
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
    let mut list = div().flex().flex_col().gap(px(4.0)).min_w(px(240.0));
    for (k, v) in rows {
        list = list.child(
            div()
                .flex()
                .gap(px(t.spacing.md))
                .child(
                    div()
                        .w(px(110.0))
                        .text_size(px(t.typography.size_sm))
                        .text_color(hsla(t.palette.text_muted))
                        .child(k),
                )
                .child(
                    div()
                        .font_family(dodai_gpui::text::first_family(t.typography.mono))
                        .text_size(px(t.typography.size_sm))
                        .text_color(hsla(t.palette.text))
                        .child(v),
                ),
        );
    }
    let sample = dodai_gpui::shaped(&t.shape, dodai_gpui::panel::panel_style(t, true))
        .flex()
        .items_center()
        .justify_center()
        .w(px(200.0))
        .h(px(96.0))
        .text_size(px(t.typography.size_sm))
        .text_color(hsla(t.palette.text))
        .child("Live sample");
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(px(t.spacing.xl))
        .child(list)
        .child(sample)
}

/// Display order and label of the icon types.
const ICON_TYPES: &[(&str, &str)] = &[
    ("interface", "Interface"),
    ("arrows", "Arrows & navigation"),
    ("actions", "Actions"),
    ("media", "Media"),
    ("files", "Files & storage"),
    ("edit", "Text & editing"),
    ("status", "Status & alerts"),
    ("data", "Data & code"),
    ("charts", "Charts"),
    ("dev", "Developer"),
    ("device", "Devices"),
    ("layout", "Layout"),
    ("access", "Accessibility"),
    ("home", "Home"),
    ("design", "Design"),
    ("sports", "Sports"),
    ("food", "Food & drink"),
    ("nature", "Nature"),
    ("places", "Places"),
    ("commerce", "Commerce"),
    ("finance", "Finance"),
    ("people", "People"),
    ("social", "Social"),
    ("security", "Security"),
    ("travel", "Travel & maps"),
    ("weather", "Weather"),
    ("health", "Health"),
    ("objects", "Objects"),
    ("cyber", "Cyber"),
];

/// The hand-drawn pack, grouped by type and filtered by `query` (matches the
/// icon name or its type).
/// An icon picked in the Icons section: a shared-vocabulary id or a Material catalogue symbol.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum IconPick {
    Id(dodai_gpui::IconId),
    Material(dodai_material_icons::MaterialSymbol),
}

impl IconPick {
    /// The name shown on a tile and used for file names.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Id(i) => i.name(),
            Self::Material(m) => m.name(),
        }
    }

    /// The Rust expression that draws it.
    pub(crate) fn usage(self) -> String {
        match self {
            Self::Id(i) => format!("Icon::new(IconId::{i:?})"),
            Self::Material(m) => format!("MaterialIcon::new(MaterialSymbol::{m:?})"),
        }
    }
}

pub(crate) fn icons_view(
    t: &dodai_core::Theme,
    query: &str,
    filled: bool,
    variants: bool,
    pick_icon: impl Fn(dodai_gpui::IconId, &mut gpui::App) + Clone + 'static,
) -> gpui::Div {
    let q = query.trim().to_lowercase().replace(' ', "_");
    let mut out = div().flex().flex_col().gap(px(t.spacing.lg));
    let (mut idx, mut total) = (0usize, 0usize);
    for &(g, label) in ICON_TYPES {
        let members: Vec<_> = dodai_cyberpunk_icons::ALL
            .iter()
            .filter(|n| n.group() == g)
            .filter(|n| {
                q.is_empty()
                    || n.name().contains(q.as_str())
                    || g.contains(q.as_str())
                    || label
                        .to_lowercase()
                        .contains(query.trim().to_lowercase().as_str())
            })
            .collect();
        if members.is_empty() {
            continue;
        }
        total += members.len();
        let mut row = div().flex().flex_wrap().w_full().gap(px(t.spacing.sm));
        for n in &members {
            idx += 1;
            let pick = dodai_gpui::IconId::from(**n);
            let pick_icon = pick_icon.clone();
            row = row.child(
                div()
                    .id(("ico", idx))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(6.0))
                    .w(px(104.0))
                    .py(px(t.spacing.sm))
                    .cursor_pointer()
                    .border_1()
                    .border_color(hsla(t.palette.border))
                    .on_click(move |_, _, cx| pick_icon(pick, cx))
                    .child(
                        dodai_gpui::Icon::new(**n)
                            .filled(filled)
                            .size(28.0)
                            .color(hsla(t.palette.primary)),
                    )
                    .child({
                        // Centred when it fits; long names stay left-aligned so
                        // the ellipsis clips the end, not both sides.
                        let mut label = div()
                            .w_full()
                            .px(px(4.0))
                            .truncate()
                            .text_size(px(t.typography.size_xs))
                            .text_color(hsla(t.palette.text_muted))
                            .child(n.name());
                        if n.name().len() <= 12 {
                            label = label.text_center();
                        }
                        label
                    }),
            );
        }
        // Each type is laid out only near the window: 380 icons cost milliseconds.
        out = out.child(dodai_gpui::lazy(
            g,
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.lg))
                .child(Muted::new(format!(
                    "{} · {}",
                    label.to_uppercase(),
                    members.len()
                )))
                .child(row),
        ));
    }
    if q.is_empty() && variants {
        let mut row = div().flex().flex_wrap().w_full().gap(px(t.spacing.sm));
        for base in [
            dodai_gpui::IconName::Heart,
            dodai_gpui::IconName::Cloud,
            dodai_gpui::IconName::Camera,
            dodai_gpui::IconName::Bell,
        ] {
            for m in dodai_gpui::Modifier::ALL {
                let name = format!("{}-{}", base.name(), m.suffix());
                row = row.child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(6.0))
                        .w(px(104.0))
                        .py(px(t.spacing.sm))
                        .border_1()
                        .border_color(hsla(t.palette.border))
                        .child(
                            dodai_gpui::Icon::new(base)
                                .modifier(m)
                                .size(28.0)
                                .color(hsla(t.palette.primary)),
                        )
                        .child(
                            div()
                                .text_size(px(t.typography.size_xs))
                                .text_color(hsla(t.palette.text_muted))
                                .child(name),
                        ),
                );
            }
        }
        out = out
            .child(Muted::new("VARIANTS · derived with a suffix"))
            .child(row);
    } else if total == 0 {
        out = out.child(Muted::new("No icons match."));
    }
    out
}

pub(crate) fn motion_tokens_view(t: &dodai_core::Theme) -> gpui::Div {
    use dodai_gpui::dodai_motion::{material_easing as e, DurationToken};
    let mut out = div().flex().flex_col().gap(px(4.0));
    for d in DurationToken::ALL {
        out = out.child(
            div()
                .flex()
                .items_center()
                .gap(px(t.spacing.md))
                .child(
                    div()
                        .w(px(130.0))
                        .flex_none()
                        .font_family(dodai_gpui::text::first_family(t.typography.mono))
                        .text_size(px(t.typography.size_xs))
                        .text_color(hsla(t.palette.text_muted))
                        .child(format!("{} · {} ms", d.name(), d.millis())),
                )
                .child(
                    div()
                        .h(px(8.0))
                        .w(px(d.millis() as f32 * 0.5))
                        .rounded(px(4.0))
                        .bg(hsla(t.palette.primary)),
                ),
        );
    }
    let curves = [
        ("emphasized", e::EMPHASIZED),
        ("emphasized-decelerate", e::EMPHASIZED_DECELERATE),
        ("emphasized-accelerate", e::EMPHASIZED_ACCELERATE),
        ("standard-decelerate", e::STANDARD_DECELERATE),
        ("standard-accelerate", e::STANDARD_ACCELERATE),
    ];
    for (name, c) in curves {
        out = out.child(
            div()
                .font_family(dodai_gpui::text::first_family(t.typography.mono))
                .text_size(px(t.typography.size_xs))
                .text_color(hsla(t.palette.text_muted))
                .child(format!("{name}: {}", c.to_css())),
        );
    }
    out
}
