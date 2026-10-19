//! The "All" landing page: hero, design-system selector, category cards.

use super::*;

impl Showcase {
    /// Children of the overview page, in order.
    pub(crate) fn overview(
        &mut self,
        me: &Entity<Self>,
        cats: &[&'static str],
        sections_meta: &[(&'static str, &'static str)],
        mut ds_panel: Option<Panel>,
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
            (dodai_core::themes::all().len().to_string(), "themes"),
            (dodai_icons::ALL.len().to_string(), "icons"),
            (n_shaders.to_string(), "shaders"),
        ];
        let mut stat_row = div().flex().gap(px(t.spacing.md)).w_full();
        for (n, label) in stats {
            stat_row = stat_row.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_size(px(t.typography.size_xxl + 6.0))
                            .text_color(hsla(t.palette.primary))
                            .child(n),
                    )
                    .child(Muted::new(label.to_uppercase())),
            );
        }
        let hero = Panel::new()
            .child(Heading::new("Dodai UI").level(1))
            .child(Muted::new(
                "One design system for gpui, iced, ratatui, dioxus, bevy and the web: \
                     tokens, components, charts and shaders. Pick a category, jump to a \
                     component, or just scroll.",
            ))
            .child(stat_row);
        out.push(hero.into_any_element());
        if let Some(d) = ds_panel.take() {
            out.push(d.into_any_element());
        }
        let mut grid = div().flex().flex_wrap().gap(px(t.spacing.md)).w_full();
        for (ci, cname) in cats.iter().enumerate() {
            let names: Vec<&'static str> = sections_meta
                .iter()
                .filter(|(c, _)| c == cname)
                .map(|(_, l)| *l)
                .collect();
            let m0 = me.clone();
            let mut card = Panel::new().refine(move |d| d.flex_1().min_w(px(250.0)));
            let head = div()
                .id(("ov-cat", ci))
                .flex()
                .items_center()
                .gap(px(t.spacing.sm))
                .cursor_pointer()
                .child(
                    dodai_gpui::Icon::new(cat_icon(cname))
                        .size(16.0)
                        .color(hsla(t.palette.primary)),
                )
                .child(Heading::new(*cname).level(3))
                .child(Muted::new(format!("· {}", names.len())))
                .on_click(move |_, _, cx| {
                    m0.update(cx, |s, cx| {
                        s.cat = ci + 1;
                        s.sub = None;
                        s.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                        cx.notify()
                    })
                });
            card = card.child(head).child(Muted::new(cat_blurb(cname)));
            let hover = hsla(t.palette.primary);
            for (k, title) in names.iter().take(6).enumerate() {
                let (m, title, cname_s) = (me.clone(), *title, *cname);
                card = card.child(
                    div()
                        .id(("ov", ci * 1000 + k))
                        .text_size(px(t.typography.size_sm))
                        .text_color(hsla(t.palette.text))
                        .cursor_pointer()
                        .hover(move |s| s.text_color(hover))
                        .child(title)
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
            if names.len() > 6 {
                let m = me.clone();
                card = card.child(
                    div()
                        .id(("ov-more", ci))
                        .cursor_pointer()
                        .child(Muted::new(format!("+ {} more →", names.len() - 6)))
                        .on_click(move |_, _, cx| {
                            m.update(cx, |s, cx| {
                                s.cat = ci + 1;
                                s.sub = None;
                                s.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                                cx.notify()
                            })
                        }),
                );
            }
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
