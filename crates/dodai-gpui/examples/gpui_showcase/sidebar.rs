//! The left navigation: filter box, category list, sub-groups and section jumps.

use super::*;
use dodai_gpui::{NavigationDrawer, NavigationItem, NavigationRail};
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
        narrow: bool,
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
            .track_scroll(&self.side_scroll)
            .relative()
            // Arrows move the keyboard focus through the rows (every row is a tab stop, so
            // Tab works too); Enter / Space activate the focused row through its click.
            .on_key_down(cx.listener(|s, e: &KeyDownEvent, window, cx| {
                match e.keystroke.key.as_str() {
                    "down" => window.focus_next(),
                    "up" => window.focus_prev(),
                    "home" | "end" => {
                        let down = e.keystroke.key == "end";
                        for _ in 0..512 {
                            let before = window.focused(cx);
                            if down {
                                window.focus_next();
                            } else {
                                window.focus_prev();
                            }
                            if !s.side_focus.contains_focused(window, cx) {
                                if down {
                                    window.focus_prev();
                                } else {
                                    window.focus_next();
                                }
                                break;
                            }
                            if window.focused(cx) == before {
                                break;
                            }
                        }
                    }
                    // Left collapses the sidebar to the rail, Right expands it.
                    "left" | "right" => {
                        s.rail = e.keystroke.key == "left";
                        cx.notify();
                        return;
                    }
                    _ => return,
                }
                // The sidebar's tab stops are bracketed by other controls: don't leave it.
                if !s.side_focus.contains_focused(window, cx) {
                    if e.keystroke.key == "up" {
                        window.focus_next();
                    } else if e.keystroke.key == "down" {
                        window.focus_prev();
                    }
                }
                cx.stop_propagation();
            }))
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .min_w_0()
            .flex_shrink_0()
            .h_full()
            .overflow_y_scroll()
            .p(px(if rail { 0.0 } else { t.spacing.md }))
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
        // Where the reader is: on a category page that is the page itself; on All it follows the
        // scroll (the category of the section at the top, "All" while the hero is showing).
        let at_top = f32::from(self.scroll.offset().y) > -1.0;
        let cur_cat = if self.cat > 0 {
            self.cat
        } else if at_top {
            0
        } else {
            active
                .and_then(|a| sections_meta.iter().find(|(_, t)| *t == a))
                .and_then(|(c, _)| cats.iter().position(|x| x == c))
                .map_or(0, |i| i + 1)
        };
        let mut drawer = NavigationDrawer::new();
        let mut rail_nav = NavigationRail::new();
        for (idx, name) in std::iter::once("All")
            .chain(cats.iter().copied())
            .enumerate()
        {
            let m = me.clone();
            let n = count_in(if idx == 0 { None } else { Some(cats[idx - 1]) });
            side_need = side_need.max(est(name.len() + 8, 0.0) + 24.0);
            let go = move |_: &gpui::ClickEvent, _: &mut Window, cx: &mut App| {
                m.update(cx, |s, cx| {
                    s.cat = idx;
                    s.sub = None;
                    s.pin.release();
                    s.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                    cx.notify()
                })
            };
            if rail {
                rail_nav = rail_nav.item(
                    NavigationItem::new(("cat", idx), name)
                        .icon(cat_icon(name))
                        .selected(cur_cat == idx)
                        .on_click(go),
                );
                continue;
            }
            drawer = drawer.item(
                NavigationItem::new(("cat", idx), name)
                    .icon(cat_icon(name))
                    .badge(n.to_string())
                    .selected(cur_cat == idx)
                    .on_click(go),
            );
            if cur_cat == idx && idx > 0 {
                // The sub-group the reader is in: the chosen filter, else the one scrolled to.
                let cur_sub = self.sub.or_else(|| active.and_then(|a| sub_of(name, a)));
                let shown_subs = if self.cat == idx { subs(name) } else { &[] };
                for sub in std::iter::once(None).chain(shown_subs.iter().copied().map(Some)) {
                    if sub.is_none() && shown_subs.is_empty() {
                        break;
                    }
                    let m = me.clone();
                    side_need = side_need.max(est(sub.map_or(3, str::len) + 8, 1.0));
                    let cnt = sections_meta
                        .iter()
                        .filter(|(c, l)| *c == name && sub.is_none_or(|x| sub_of(c, l) == Some(x)))
                        .count();
                    drawer = drawer.item(
                        NavigationItem::new(
                            (
                                "sub",
                                idx * 100
                                    + sub.map_or(0, |x| {
                                        1 + subs(name).iter().position(|y| *y == x).unwrap_or(0)
                                    }),
                            ),
                            sub.unwrap_or("All"),
                        )
                        .depth(1)
                        .badge(cnt.to_string())
                        .selected(cur_sub == sub)
                        .on_click(move |_, _, cx| {
                            m.update(cx, |s, cx| {
                                s.sub = sub;
                                s.pin.release();
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
                    let mut row = NavigationItem::new(("jump", idx * 1000 + k), title);
                    // The Icons section shows how many icons the active design ships.
                    if title == "Icons" {
                        row = row.badge(group_thousands(dodai_gpui::IconId::ALL.len()));
                    }
                    drawer = drawer.item(row.depth(2).selected(active == Some(title)).on_click(
                        move |_, _, cx| {
                            m.update(cx, |s, cx| {
                                s.pin.pin(title);
                                s.jump = Some((title, 1));
                                cx.notify()
                            })
                        },
                    ));
                    k += 1;
                }
            }
        }
        side = if rail {
            side.child(rail_nav)
        } else {
            let w = if narrow {
                side_need.clamp(170.0, 230.0)
            } else {
                side_need.clamp(240.0, 340.0)
            };
            side.child(drawer.width(w - 2.0 * t.spacing.md))
        };
        if !rail {
            side = side.child(
                div()
                    .pt(px(t.spacing.md))
                    .child(Muted::new("↑↓ / Tab move · Enter select · t / T theme")),
            );
        }

        (side, side_need)
    }
}

fn group_thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}
