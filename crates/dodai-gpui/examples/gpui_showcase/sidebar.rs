//! The left navigation: filter box, category list, sub-groups and section jumps.

use super::*;
use gpui::Div;

impl Showcase {
    /// Build the sidebar. Returns it with the px width its longest label needs.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn sidebar(
        &mut self,
        me: &Entity<Self>,
        cats: &[&'static str],
        sections_meta: &[(&'static str, &'static str)],
        active: Option<&'static str>,
        query: &str,
        count_in: impl Fn(Option<&'static str>) -> usize,
        cx: &mut Context<Self>,
    ) -> (gpui::Stateful<Div>, f32) {
        let t = theme_of(cx);
        let me = me.clone();
        let rail = self.rail;
        // Widest row (px estimate) so the sidebar grows with its labels up to a
        // cap, after which labels wrap.
        let mut side_need: f32 = 0.0;
        let est = |chars: usize, depth: f32| depth * t.spacing.md + chars as f32 * 8.6 + 64.0;
        let mut side = div()
            .id("side-nav")
            .track_focus(&self.side_focus)
            .on_key_down(cx.listener(|s, e: &KeyDownEvent, _, cx| {
                let last = s.cat_count.saturating_sub(1);
                match e.keystroke.key.as_str() {
                    "down" => s.cat = (s.cat + 1).min(last),
                    "up" => s.cat = s.cat.saturating_sub(1),
                    "home" => s.cat = 0,
                    "end" => s.cat = last,
                    _ => return,
                }
                s.sub = None;
                dodai_gpui::set_tree_expanded(cx, true);
                s.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                cx.notify();
            }))
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .min_w_0()
            .flex_shrink_0()
            .h_full()
            .overflow_y_scroll()
            .p(px(if rail { t.spacing.xs } else { t.spacing.md }))
            .child({
                let m = me.clone();
                let toggle = IconButton::new("rail-toggle", if rail { "»" } else { "«" }).on_click(
                    move |_, _, cx| {
                        m.update(cx, |s, cx| {
                            s.rail = !s.rail;
                            cx.notify()
                        })
                    },
                );
                if rail {
                    div()
                        .flex()
                        .justify_center()
                        .pb(px(t.spacing.sm))
                        .child(toggle)
                } else {
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(Heading::new("Dodai UI").level(2))
                        .child(toggle)
                }
            });
        if !rail {
            side = side.child(div().pb(px(t.spacing.sm)).child(self.filter.clone()));
        }
        for (idx, name) in std::iter::once("All")
            .chain(cats.iter().copied())
            .enumerate()
        {
            let m = me.clone();
            let n = count_in(if idx == 0 { None } else { Some(cats[idx - 1]) });
            side_need = side_need.max(est(name.len() + 8, 0.0) + 24.0);
            side = side.child(
                ListItem::new(
                    ("cat", idx),
                    if rail {
                        String::new()
                    } else {
                        format!("{name}  ·  {n}")
                    },
                )
                .icon(cat_icon(name))
                .selected(self.cat == idx)
                .on_click(move |_, w, cx| {
                    m.update(cx, |s, cx| {
                        w.focus(&s.side_focus);
                        s.cat = idx;
                        s.sub = None;
                        dodai_gpui::set_tree_expanded(cx, true);
                        s.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                        cx.notify()
                    })
                }),
            );
            if self.cat == idx && idx > 0 && !rail {
                for sub in std::iter::once(None).chain(subs(name).iter().copied().map(Some)) {
                    if sub.is_none() && subs(name).is_empty() {
                        break;
                    }
                    let m = me.clone();
                    side_need = side_need.max(est(sub.map_or(3, str::len) + 8, 1.0));
                    let cnt = sections_meta
                        .iter()
                        .filter(|(c, l)| *c == name && sub.is_none_or(|x| sub_of(c, l) == Some(x)))
                        .count();
                    side = side.child(
                        ListItem::new(
                            (
                                "sub",
                                idx * 100
                                    + sub.map_or(0, |x| {
                                        1 + subs(name).iter().position(|y| *y == x).unwrap_or(0)
                                    }),
                            ),
                            format!("{}  ·  {cnt}", sub.unwrap_or("All")),
                        )
                        .depth(1)
                        .selected(self.sub == sub)
                        .on_click(move |_, _, cx| {
                            m.update(cx, |s, cx| {
                                s.sub = sub;
                                dodai_gpui::set_tree_expanded(cx, true);
                                s.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                                cx.notify()
                            })
                        }),
                    );
                }
                // Jump list: every visible section of this category.
                let mut k = 0usize;
                for (c, title) in sections_meta {
                    if *c != name
                        || self.sub.is_some_and(|x| sub_of(c, title) != Some(x))
                        || !(query.is_empty()
                            || !fuzzy_filter(query, &[format!("{c} {title}")]).is_empty())
                    {
                        continue;
                    }
                    side_need = side_need.max(est(title.len(), 2.0));
                    let (m, title) = (me.clone(), *title);
                    side = side.child(
                        ListItem::new(("jump", idx * 1000 + k), title)
                            .depth(2)
                            .selected(active == Some(title))
                            .on_click(move |_, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.jump = Some((title, 1));
                                    cx.notify()
                                })
                            }),
                    );
                    k += 1;
                }
            }
        }
        if !rail {
            side = side.child(
                div()
                    .pt(px(t.spacing.md))
                    .child(Muted::new("t / T theme · l scanlines")),
            );
        }

        (side, side_need)
    }
}
