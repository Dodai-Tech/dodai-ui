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
                .into_any_element(),
            "Type scale",
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
        let rounded = t.look().symbol_icons;
        let query = self.icon_search.read(cx).text().to_string();
        let count = dodai_cyberpunk_icons::ALL.len();
        out.push((
            "Foundations",
            "Icons",
            section!("Icons")
                .child(Muted::new(if rounded {
                    format!("{count} icons by type. `Icon` follows the theme: rounded themes draw the matching Material Symbols glyph. Any name takes -off / -add / -remove / -check / -lock / -circle (e.g. `heart-add`). Click to copy `IconName::…`.")
                } else {
                    format!("{count} hand-drawn icons by type: straight lines, 45° chamfers, octagons. `Icon` follows the theme. Any name takes -off / -add / -remove / -check / -lock / -circle (e.g. `heart-add`). Click to copy `IconName::…`.")
                }))
                .child(div().w(px(360.0)).max_w_full().child(self.icon_search.clone()))
                .child(icons_view(&t, &query))
                .into_any_element(),
            "Icons",
        ));
        // The Material catalogue only belongs to Material themes.
        if !rounded {
            return out;
        }
        let browser = self.icon_browser(&t, cx);
        out.push((
            "Foundations",
            "Icons Material",
            section!("Icons Material")
                .child(Muted::new(
                    "The entire Material Symbols catalogue (`MaterialIcon::new(name)`, outlined or `.filled(true)`). It is the Material pack, always drawn smooth.",
                ))
                .child(browser)
                .into_any_element(),
            "Icons Material",
        ));
        out
    }
}

impl Showcase {
    /// Searchable, paged grid of every Material Symbols icon.
    fn icon_browser(&mut self, t: &dodai_core::Theme, cx: &mut Context<Self>) -> gpui::Div {
        let me = cx.entity();
        let me_f = me.clone();
        let q = self
            .icon_q
            .read(cx)
            .text()
            .trim()
            .to_lowercase()
            .replace(' ', "_");
        let hits: Vec<&'static str> = dodai_material_icons::outlined::ALL
            .iter()
            .map(|i| i.name)
            .filter(|n| !n.ends_with("-fill") && n.contains(q.as_str()))
            .collect();
        let shown = hits.len().min(self.icon_limit);
        let controls = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(t.spacing.md))
            .child(div().flex_1().min_w(px(220.0)).child(self.icon_q.clone()))
            .child(
                Checkbox::new("ib-fill", self.icon_filled).on_change(move |v, _, cx| {
                    me_f.update(cx, |s, cx| {
                        s.icon_filled = v;
                        cx.notify();
                    })
                }),
            )
            .child(Label::new("Filled"))
            .child(Muted::new(format!("{shown} of {} icons", hits.len())));
        let mut grid = div().flex().flex_wrap().gap(px(t.spacing.xs));
        for &n in hits.iter().take(shown) {
            grid = grid.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(4.0))
                    .w(px(92.0))
                    .py(px(t.spacing.sm))
                    .border_1()
                    .border_color(hsla(t.palette.border))
                    .child(
                        dodai_gpui::MaterialIcon::new(n)
                            .size(28.0)
                            .filled(self.icon_filled)
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
                            .child(n);
                        if n.len() <= 12 {
                            label = label.text_center();
                        }
                        label
                    }),
            );
        }
        let mut out = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.md))
            .child(controls)
            .child(grid);
        if hits.len() > shown {
            out = out.child(
                div().flex().child(
                    Button::new("ib-more", "Show more")
                        .variant(ButtonVariant::Secondary)
                        .on_click(cx.listener(|s, _, _, cx| {
                            s.icon_limit += 240;
                            cx.notify();
                        })),
                ),
            );
        }
        out
    }
}
