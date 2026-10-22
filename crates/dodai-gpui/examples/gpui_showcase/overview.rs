//! The "All" landing page: hero, design-system selector, category cards.

use super::*;

impl Showcase {
    /// Children of the overview page, in order.
    pub(crate) fn overview(
        &mut self,
        me: &Entity<Self>,
        cats: &[&'static str],
        sections_meta: &[(&'static str, &'static str)],
        cx: &mut Context<Self>,
    ) -> Vec<gpui::AnyElement> {
        let t = theme_of(cx);
        let me = me.clone();
        let mut out: Vec<gpui::AnyElement> = Vec::new();
        let total = sections_meta.len();
        let n_shaders = sections_meta
            .iter()
            .filter(|(c, _)| *c == "Shaders")
            .count();
        let stats: [(String, &str); 5] = [
            (total.to_string(), "components"),
            (cats.len().to_string(), "categories"),
            (dodai::themes::all().len().to_string(), "themes"),
            (dodai_gpui::IconId::ALL.len().to_string(), "icons"),
            (n_shaders.to_string(), "shaders"),
        ];
        let line = hsla(t.palette.border);
        let mut stat_row = div()
            .flex()
            .w_full()
            .border_t_1()
            .border_color(line)
            .pt(px(t.spacing.md));
        for (i, (n, label)) in stats.into_iter().enumerate() {
            let mut cell = div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_size(px(t.typography.size_xxl))
                        .text_color(hsla(t.palette.primary))
                        .child(n),
                )
                .child(Muted::new(label.to_uppercase()));
            if i > 0 {
                cell = cell.pl(px(t.spacing.md)).border_l_1().border_color(line);
            }
            stat_row = stat_row.child(cell);
        }
        let mut toolkits = div().flex().flex_wrap().gap(px(t.spacing.xs));
        for name in [
            "gpui", "iced", "egui", "slint", "ratatui", "dioxus", "leptos", "yew", "bevy", "web",
        ] {
            toolkits = toolkits.child(dodai_gpui::Tag::new(name, dodai_core::Tone::Info));
        }
        let live = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(t.spacing.sm))
            .child(Button::new("ov-b1", "Get started"))
            .child(Button::new("ov-b2", "Secondary").variant(ButtonVariant::Secondary))
            .child(Button::new("ov-b3", "Outline").variant(ButtonVariant::Outline))
            .child(dodai_gpui::Toggle::new("ov-t", true));
        let hero = Panel::new().focused(true).child(
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.md))
                .child(Muted::new("DESIGN SYSTEM · RUST · MULTI-TOOLKIT"))
                .child(
                    div()
                        .text_size(px(t.typography.size_xxl * 1.5))
                        .text_color(hsla(t.palette.text))
                        .child("Dodai UI"),
                )
                .child(
                    div()
                        .max_w(px(820.0))
                        .text_size(px(t.typography.size_lg))
                        .text_color(hsla(t.palette.text_muted))
                        .child("One set of tokens, components, charts and shaders. Three designs, every toolkit."),
                )
                .child(toolkits)
                .child(live)
                .child(stat_row),
        );
        out.push(hero.into_any_element());
        out.push(
            Heading::new("Browse by category")
                .level(2)
                .into_any_element(),
        );
        let mut grid = div().flex().flex_wrap().gap(px(t.spacing.md)).w_full();
        for (ci, cname) in cats.iter().enumerate() {
            let names: Vec<&'static str> = sections_meta
                .iter()
                .filter(|(c, _)| c == cname)
                .map(|(_, l)| *l)
                .collect();
            let open = {
                let m = me.clone();
                move |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| {
                    m.update(cx, |s, cx| {
                        s.cat = ci + 1;
                        s.sub = None;
                        s.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                        cx.notify()
                    })
                }
            };
            let primary = hsla(t.palette.primary);
            let icon_box = div()
                .flex()
                .items_center()
                .justify_center()
                .size(px(36.0))
                .border_1()
                .border_color(primary)
                .child(
                    dodai_gpui::Icon::new(cat_icon(cname))
                        .size(20.0)
                        .color(primary),
                );
            let head = div()
                .id(("ov-cat", ci))
                .flex()
                .items_center()
                .gap(px(t.spacing.md))
                .cursor_pointer()
                .child(icon_box)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .child(Heading::new(*cname).level(3))
                        .child(Muted::new(format!(
                            "{} section{}",
                            names.len(),
                            if names.len() == 1 { "" } else { "s" }
                        ))),
                )
                .on_click(open.clone());
            let mut chips = div().flex().flex_wrap().gap(px(t.spacing.xs));
            for (k, title) in names.iter().take(4).enumerate() {
                let (m, title, cname_s) = (me.clone(), *title, *cname);
                let short = title.split(" · ").next().unwrap_or(title).to_string();
                chips = chips.child(
                    div()
                        .id(("ov", ci * 1000 + k))
                        .px(px(t.spacing.sm))
                        .py(px(2.0))
                        .border_1()
                        .border_color(line)
                        .text_size(px(t.typography.size_xs))
                        .text_color(hsla(t.palette.text))
                        .cursor_pointer()
                        .hover(move |s| s.border_color(primary).text_color(primary))
                        .child(short)
                        .on_click(move |_, _, cx| {
                            // Scroll to it right here in the overview, unless it
                            // lives on its own page (Loading, Shaders).
                            m.update(cx, |s, cx| {
                                if matches!(cname_s, "Loading" | "Shaders") {
                                    s.cat = ci + 1;
                                    s.sub = None;
                                    s.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                                }
                                s.jump = Some((title, 3));
                                cx.notify()
                            })
                        }),
                );
            }
            let footer = div()
                .id(("ov-more", ci))
                .cursor_pointer()
                .text_size(px(t.typography.size_sm))
                .text_color(primary)
                .child(if names.len() > 4 {
                    format!("Open all {} →", names.len())
                } else {
                    "Open →".to_string()
                })
                .on_click(open);
            let card = Panel::new()
                .refine(move |d| d.flex_1().min_w(px(260.0)).min_h(px(190.0)))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(t.spacing.md))
                        .child(head)
                        .child(Muted::new(cat_blurb(cname)))
                        .child(chips)
                        .child(footer),
                );
            grid = grid.child(card);
        }
        out.push(grid.into_any_element());
        out.push(
            Heading::new("Everything, in one scroll")
                .level(2)
                .into_any_element(),
        );
        out.push(
                Muted::new(
                    "Every component below. Loading and Shaders animate constantly, so they live on their own pages.",
                )
                .into_any_element(),
            );
        out
    }
}
