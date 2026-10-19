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
