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
    let groups: [(&str, Vec<(&str, dodai_core::Color)>); 5] = [
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

pub(crate) fn icons_view(t: &dodai_core::Theme) -> gpui::Div {
    let mut row = div().flex().flex_wrap().gap(px(t.spacing.md));
    for (i, n) in dodai_icons::ALL.iter().enumerate() {
        let name = format!("{n:?}");
        let copy = name.clone();
        row = row.child(
            div()
                .id(("ico", i))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(6.0))
                .w(px(88.0))
                .py(px(t.spacing.sm))
                .cursor_pointer()
                .border_1()
                .border_color(hsla(t.palette.border))
                .on_click(move |_, _, cx| {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(copy.clone()))
                })
                .child(
                    dodai_gpui::Icon::new(*n)
                        .size(24.0)
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
    row
}
