//! The "Foundations" category sections.

use super::super::*;

impl Showcase {
    #[allow(unused_variables, clippy::redundant_clone, clippy::vec_init_then_push)]
    pub(crate) fn sections_foundations_sec(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let me = cx.entity();
        #[allow(unused_macros)]
        macro_rules! section {
            ($t:expr) => {
                Panel::new()
                    .title($t)
                    .subheader(self.chips($t, &t, &me, cx))
                    .scope(Some(self.scope_of($t, cx)))
                    .scope_key(self.sec_replay.get($t).copied().unwrap_or(0))
            };
        }
        let mut out: Vec<Section> = Vec::new();

        out.push((
            "Foundations",
            "Text · Heading · Label · Mono",
            section!("Text")
                .child(Heading::new("Heading 1").level(1))
                .child(Heading::new("Heading 2"))
                .child(Heading::new("Heading 3").level(3))
                .child(Label::new("Label text"))
                .child(Muted::new("Muted text"))
                .child(Mono::new("mono_text() // 0xDEADBEEF"))
                .into_any_element(),
            "Text",
        ));
        out.push((
            "Foundations",
            "Colour palette",
            section!("Colour palette")
                .child(Muted::new(
                    "Every Palette token of the active design system.",
                ))
                .child(palette_view(&t))
                .into_any_element(),
            "Colour palette",
        ));
        out.push((
            "Foundations",
            "Type scale",
            section!("Type scale")
                .child(Muted::new(
                    "Sizes and families from the theme's Typography.",
                ))
                .child(type_view(&t))
                .child(Muted::new(
                    "Type scale roles (Material 3: exact; others derive them)",
                ))
                .child(type_scale_view(&t))
                .into_any_element(),
            "Type scale",
        ));
        out.push((
            "Foundations",
            "Motion tokens",
            section!("Motion tokens")
                .child(Muted::new(
                    "Material 3 duration scale (bar = duration) and named easing curves.",
                ))
                .child(motion_tokens_view(&t))
                .into_any_element(),
            "Motion tokens",
        ));
        out.push((
            "Foundations",
            "Spacing · Elevation",
            section!("Spacing · Elevation")
                .child(Muted::new(
                    "Spacing scale (bars are 8x the value) and surface layers.",
                ))
                .child(spacing_elevation_view(&t))
                .into_any_element(),
            "Spacing · Elevation",
        ));
        out.push((
            "Foundations",
            "Shape tokens",
            section!("Shape tokens")
                .child(Muted::new(
                    "Read-only summary of the theme's shape and effect tokens.",
                ))
                .child(shape_tokens_view(&t))
                .into_any_element(),
            "Shape tokens",
        ));
        let set = t.look().icons;
        let query = self.icon_search.read(cx).text().to_string();
        let count = dodai_cyberpunk_icons::ALL.len();
        let intro = match set {
            dodai_core::IconSet::Symbols => "The Material Symbols catalogue by Material's own categories (`MaterialIcon::new(name)`). Click to copy.".to_string(),
            dodai_core::IconSet::Bold => format!("{count} bold icons with a filled form each (Phosphor), by type. Click to copy `IconName::…`."),
            dodai_core::IconSet::Stroked => format!("{count} hand-drawn icons by type: straight lines, 45° chamfers, octagons. Any name takes -off / -add / -remove / -check / -lock / -circle (e.g. `heart-add`). Click to copy `IconName::…`."),
        };
        let mut section = section!("Icons").child(Muted::new(intro)).child(
            div()
                .w(px(360.0))
                .max_w_full()
                .child(self.icon_search.clone()),
        );
        let filled_toggle = {
            let me_f = me.clone();
            div()
                .flex()
                .items_center()
                .gap(px(t.spacing.sm))
                .child(
                    Checkbox::new("ib-fill", self.icon_filled).on_change(move |v, _, cx| {
                        me_f.update(cx, |s, cx| {
                            s.icon_filled = v;
                            cx.notify();
                        })
                    }),
                )
                .child(Label::new("Filled"))
        };
        section = match set {
            dodai_core::IconSet::Symbols => section
                .child(filled_toggle)
                .child(Muted::new("MATERIAL SYMBOLS"))
                .child(self.icon_browser(&t, &query, cx, IconSource::Material)),
            dodai_core::IconSet::Bold => {
                section
                    .child(filled_toggle)
                    .child(icons_view(&t, &query, self.icon_filled, false))
            }
            dodai_core::IconSet::Stroked => section.child(icons_view(&t, &query, false, true)),
        };
        // Parity: every design draws the whole shared vocabulary, in its own style.
        let vocab = dodai_phosphor_icons::names().count();
        section = section
            .child(Muted::new(format!(
                "SHARED VOCABULARY · {vocab} Phosphor names, drawn in this design's style"
            )))
            .child(self.icon_browser(&t, &query, cx, IconSource::Phosphor));
        out.push(("Foundations", "Icons", section.into_any_element(), "Icons"));
        out
    }
}

/// Which category list an icon browser walks.
#[derive(Clone, Copy)]
enum IconSource {
    /// Material Symbols by Material's own categories.
    Material,
    /// The shared Phosphor vocabulary by Phosphor's categories.
    Phosphor,
}

impl Showcase {
    /// The Material Symbols catalogue grouped by Material's own categories. A category shows a
    /// preview until opened; a search lists every match.
    fn icon_browser(
        &mut self,
        t: &dodai_core::Theme,
        query: &str,
        cx: &mut Context<Self>,
        kind: IconSource,
    ) -> gpui::Div {
        let (source, tag): (&'static [(&'static str, &'static [&'static str])], &str) = match kind {
            IconSource::Material => (dodai_material_icons::CATEGORIES, "mi"),
            IconSource::Phosphor => (dodai_phosphor_icons::CATEGORIES, "ph"),
        };
        const PREVIEW: usize = 24;
        const MAX_HITS: usize = 480;
        let q = query.trim().to_lowercase().replace(' ', "_");
        let searching = !q.is_empty();
        let mut out = div().flex().flex_col().gap(px(t.spacing.lg));
        let mut total = 0usize;
        for &(cat, names) in source {
            let hits: Vec<&'static str> = names
                .iter()
                .copied()
                .filter(|n| {
                    !searching
                        || n.replace('-', "_").contains(q.as_str())
                        || cat
                            .to_lowercase()
                            .contains(query.trim().to_lowercase().as_str())
                })
                .collect();
            if hits.is_empty() {
                continue;
            }
            let open = self.icon_open.contains(cat);
            let cap = if searching {
                MAX_HITS.saturating_sub(total)
            } else if open {
                hits.len()
            } else {
                PREVIEW
            };
            total += hits.len().min(cap);
            let mut grid = div().flex().flex_wrap().gap(px(t.spacing.xs));
            for (i, &n) in hits.iter().take(cap).enumerate() {
                grid = grid.child(
                    div()
                        .id(SharedString::from(format!("{tag}-{cat}-{i}")))
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .w(px(92.0))
                        .py(px(t.spacing.sm))
                        .cursor_pointer()
                        .border_1()
                        .border_color(hsla(t.palette.border))
                        .on_click(move |_, _, cx| {
                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(match kind {
                                IconSource::Material => format!("MaterialIcon::new(\"{n}\")"),
                                IconSource::Phosphor => format!("Icon::named(\"{n}\")"),
                            }))
                        })
                        .child(match kind {
                            IconSource::Material => dodai_gpui::MaterialIcon::new(n)
                                .size(28.0)
                                .filled(self.icon_filled)
                                .color(hsla(t.palette.primary))
                                .into_any_element(),
                            // Drawn by the active design: its own icon if it has one, else
                            // the Phosphor drawing in the design's style.
                            IconSource::Phosphor => dodai_gpui::Icon::named(n)
                                .pack(dodai_gpui::IconPack::Auto)
                                .size(28.0)
                                .filled(self.icon_filled)
                                .color(hsla(t.palette.primary))
                                .into_any_element(),
                        })
                        .child({
                            // Centred when it fits; long names stay left-aligned so the
                            // ellipsis clips the end, not both sides.
                            let mut label = div()
                                .w_full()
                                .px(px(4.0))
                                .truncate()
                                .text_size(px(t.typography.size_xs))
                                .text_color(hsla(t.palette.text_muted))
                                .child(n);
                            if n.len() <= 12 {
                                label = label.text_center();
                            }
                            label
                        }),
                );
            }
            let mut block = div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.md))
                .child(Muted::new(format!(
                    "{} · {}",
                    cat.to_uppercase(),
                    hits.len()
                )))
                .child(grid);
            if !searching && hits.len() > PREVIEW {
                block = block.child(
                    div().flex().child(
                        Button::new(
                            SharedString::from(format!("{tag}-more-{cat}")),
                            if open {
                                "Show fewer".to_string()
                            } else {
                                format!("Show all {}", hits.len())
                            },
                        )
                        .variant(ButtonVariant::Secondary)
                        .on_click(cx.listener(move |s, _, _, cx| {
                            if !s.icon_open.remove(cat) {
                                s.icon_open.insert(cat);
                            }
                            cx.notify();
                        })),
                    ),
                );
            }
            out = out.child(dodai_gpui::lazy(
                SharedString::from(format!("{tag}-cat-{cat}")),
                block,
            ));
        }
        if total == 0 {
            out = out.child(Muted::new("No icons match."));
        }
        out
    }
}
