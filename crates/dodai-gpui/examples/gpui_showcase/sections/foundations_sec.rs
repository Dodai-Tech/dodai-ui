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
        let rounded = t.shape.kind() == dodai_core::ShapeKind::Rounded;
        out.push((
            "Foundations",
            "Icons",
            section!("Icons")
                .child(Muted::new(if rounded {
                    "`Icon` follows the theme: rounded themes draw the matching Material Symbols glyph. Click to copy `IconName::…`."
                } else {
                    "`Icon` follows the theme: chamfered themes draw the Cyberpunk pack (straight lines, 45° chamfers, octagons). Click to copy `IconName::…`."
                }))
                .child(icons_view(&t))
                .into_any_element(),
            "Icons",
        ));
        let browser = self.icon_browser(&t, cx);
        out.push((
            "Foundations",
            "Icon browser",
            section!("Icon browser")
                .child(Muted::new(
                    "The entire Material Symbols catalogue. Type to search; faceted drawing is the default on non-rounded themes.",
                ))
                .child(browser)
                .into_any_element(),
            "Icon browser",
        ));
        out.push((
            "Foundations",
            "Material Symbols",
            section!("Material Symbols")
                .child(Muted::new(
                    "`MaterialIcon::new(name)`: any of the 3,900+ catalogue icons, outlined and `.filled(true)`. Non-rounded themes draw them faceted (curves → straight cuts); override with `.faceted(bool)`.",
                ))
                .child(material_symbols_view(&t))
                .into_any_element(),
            "Material Symbols",
        ));
        out
    }
}

impl Showcase {
    /// Searchable, paged grid of every Material Symbols icon.
    fn icon_browser(&mut self, t: &dodai_core::Theme, cx: &mut Context<Self>) -> gpui::Div {
        let me = cx.entity();
        let (me_f, me_s) = (me.clone(), me.clone());
        let q = self
            .icon_q
            .read(cx)
            .text()
            .trim()
            .to_lowercase()
            .replace(' ', "_");
        let facet = self
            .icon_facet
            .unwrap_or(t.shape.kind() != dodai_core::ShapeKind::Rounded);
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
            .child(Checkbox::new("ib-facet", facet).on_change(move |v, _, cx| {
                me_s.update(cx, |s, cx| {
                    s.icon_facet = Some(v);
                    cx.notify();
                })
            }))
            .child(Label::new("Faceted"))
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
                            .faceted(facet)
                            .color(hsla(t.palette.primary)),
                    )
                    .child(
                        div()
                            .w_full()
                            .px(px(4.0))
                            .text_center()
                            .truncate()
                            .text_size(px(t.typography.size_xs))
                            .text_color(hsla(t.palette.text_muted))
                            .child(n),
                    ),
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
