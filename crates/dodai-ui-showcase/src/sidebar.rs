//! The left navigation: filter box, category list, sub-groups and section jumps.

use super::*;
use dodai_ui_gpui::{NavigationDrawer, NavigationItem, NavigationRail};
use gpui::Div;

impl Showcase {
    /// Build the sidebar. Returns it with the px width its longest label needs.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn sidebar(
        &mut self,
        me: &Entity<Self>,
        narrow: bool,
        drawer: bool,
        cx: &mut Context<Self>,
    ) -> (gpui::Stateful<Div>, f32) {
        let t = theme_of(cx);
        let meta = self.meta.clone();
        let sections_meta = &meta;
        let mut cats: Vec<&'static str> = Vec::new();
        for (c, _) in sections_meta {
            if !cats.contains(c) {
                cats.push(c);
            }
        }
        let query = self.filter.read(cx).text().to_string();
        let query = query.as_str();
        let active = self.pin.current(&self.scroll, &self.spy, SPY_SLACK);
        let count_in = |c: Option<&'static str>| {
            sections_meta
                .iter()
                .filter(|(sc, title)| {
                    c.is_none_or(|c| c == *sc)
                        && (query.is_empty()
                            || !fuzzy_filter(query, &[format!("{sc} {title}")]).is_empty())
                })
                .count()
        };
        let me = me.clone();
        // In the phone drawer there is no rail: the shell draws the drawer's own close icon.
        let rail = self.rail && !drawer;
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
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(t.spacing.sm))
                                .child(crate::brand::mark(t.typography.size_xl * 1.6))
                                .child(Heading::new("Dodai UI").level(2)),
                        )
                        .children((!drawer).then_some(toggle))
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
            let has_kids = idx > 0 && cur_cat == idx;
            let is_open = has_kids && !self.shut.contains(&name);
            let mut cat_row = NavigationItem::new(("cat", idx), name)
                .icon(cat_icon(name))
                .selected(cur_cat == idx);
            if name == "Shaders" {
                cat_row = cat_row.badge(
                    crate::catalog::shader_count(None, None)
                        .unwrap_or(n)
                        .to_string(),
                );
            } else {
                cat_row = cat_row.badge(n.to_string());
            }
            if idx > 0 {
                cat_row = cat_row.expanded(is_open);
            }
            drawer = drawer.item(cat_row.on_click({
                let m = me.clone();
                move |_, _, cx| {
                    m.update(cx, |s, cx| {
                        // Picking the open category again folds it; another one opens.
                        if idx > 0 && s.cat == idx {
                            // A chosen sub-group: the category row goes back to all of it.
                            if s.sub.is_some() {
                                s.sub = None;
                                s.pin.release();
                                s.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                                cx.notify();
                                return;
                            }
                            if let Some(i) = s.shut.iter().position(|c| *c == name) {
                                s.shut.remove(i);
                            } else {
                                s.shut.push(name);
                            }
                            cx.notify();
                            return;
                        }
                        s.shut.retain(|c| *c != name);
                        s.cat = idx;
                        s.sub = None;
                        s.pin.release();
                        s.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                        cx.notify()
                    })
                }
            }));
            if is_open {
                // The sub-group the reader is in: the chosen filter, else the one scrolled to.
                let cur_sub = self.sub.or_else(|| active.and_then(|a| sub_of(name, a)));
                let shown_subs = if self.cat == idx { subs(name) } else { &[] };
                // The rows that jump to a section of this category (those of one sub-group when
                // given): visible sections only.
                let jump_rows = |side_need: &mut f32, only: Option<&'static str>| {
                    let mut rows: Vec<NavigationItem> = Vec::new();
                    let mut k = 0usize;
                    let depth = if only.is_some() { 2 } else { 1 };
                    for (c, title) in sections_meta {
                        if *c != name
                            || only.is_some_and(|x| sub_of(c, title) != Some(x))
                            || !(query.is_empty()
                                || !fuzzy_filter(query, &[format!("{c} {title}")]).is_empty())
                        {
                            continue;
                        }
                        *side_need = side_need.max(est(title.len(), depth as f32 + 1.0));
                        let (m, title) = (me.clone(), *title);
                        let mut row = NavigationItem::new(
                            ("jump", idx * 1000 + k),
                            title
                                .split_once(" · ")
                                .filter(|(g, _)| {
                                    ["Vehicle", "Industrial", "Medical", "Military", "Aviation"]
                                        .contains(g)
                                })
                                .map_or(title, |(_, rest)| rest),
                        );
                        // The Icons section shows how many icons the active design ships.
                        if title == "Icons" {
                            row = row.badge(group_thousands(crate::sections::icon_total()));
                        }
                        if let Some(n) = (name == "Shaders")
                            .then(|| crate::catalog::shader_count(None, Some(title)))
                            .flatten()
                        {
                            row = row.badge(n.to_string());
                        }
                        rows.push(row.depth(depth).selected(active == Some(title)).on_click(
                            move |_, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.pin.pin(title);
                                    s.jump = Some((title, 1));
                                    if title == "Icons" {
                                        s.icon_pick = Default::default();
                                    }
                                    cx.notify()
                                })
                            },
                        ));
                        // The Icons section has its own tree: every block of the single catalogue.
                        if title == "Icons" {
                            for (r, row) in crate::sections::icon_tree().into_iter().enumerate() {
                                *side_need =
                                    side_need.max(est(row.label.len() + 6, row.depth as f32));
                                let m = me.clone();
                                let pick = row.pick;
                                rows.push(
                                    NavigationItem::new(("icon-tree", idx * 1000 + r), row.label)
                                        .depth(row.depth)
                                        .badge(group_thousands(row.count))
                                        .selected(self.icon_pick == pick)
                                        .on_click(move |_, _, cx| {
                                            m.update(cx, |s, cx| {
                                                // A second click on the open branch folds it.
                                                s.icon_pick = if s.icon_pick == pick {
                                                    pick.up()
                                                } else {
                                                    pick
                                                };
                                                s.pin.pin("Icons");
                                                s.jump = Some(("Icons", 1));
                                                cx.notify()
                                            })
                                        }),
                                );
                            }
                        }
                        k += 1;
                    }
                    rows
                };
                if shown_subs.is_empty() {
                    // A category without sub-groups: its sections right below it.
                    for row in jump_rows(&mut side_need, None) {
                        drawer = drawer.item(row);
                    }
                }
                // The Shaders playground is the way in: it goes above "All".
                let mut order: Vec<Option<&'static str>> = std::iter::once(None)
                    .chain(shown_subs.iter().copied().map(Some))
                    .collect();
                if name == "Shaders" && order.len() > 2 {
                    order.swap(0, 1);
                }
                // No "All" row: the category row itself shows everything again.
                order.retain(Option::is_some);
                for sub in order {
                    if sub.is_none() && shown_subs.is_empty() {
                        break;
                    }
                    let m = me.clone();
                    side_need = side_need.max(est(sub.map_or(3, str::len) + 8, 1.0));
                    let cnt = sections_meta
                        .iter()
                        .filter(|(c, l)| *c == name && sub.is_none_or(|x| sub_of(c, l) == Some(x)))
                        .count();
                    // A sub-group with several sections is a branch of the tree; its sections
                    // show while it is open.
                    let branch = sub.is_some() && cnt > 1;
                    let open = branch && sub.is_some_and(|x| self.open_subs.contains(&x));
                    let mut row = NavigationItem::new(
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
                    .selected(cur_sub == sub);
                    if name == "Shaders" {
                        if let Some(n) = crate::catalog::shader_count(sub, None) {
                            row = row.badge(n.to_string());
                        }
                    } else {
                        row = row.badge(cnt.to_string());
                    }
                    if branch {
                        row = row.expanded(open);
                    }
                    drawer = drawer.item(row.on_click(move |_, _, cx| {
                        m.update(cx, |s, cx| {
                            // The chosen branch again folds or opens it; another one opens.
                            if let Some(x) = sub {
                                if s.sub == sub {
                                    if let Some(i) = s.open_subs.iter().position(|o| *o == x) {
                                        s.open_subs.remove(i);
                                    } else {
                                        s.open_subs.push(x);
                                    }
                                    cx.notify();
                                    return;
                                }
                                if !s.open_subs.contains(&x) {
                                    s.open_subs.push(x);
                                }
                            }
                            s.sub = sub;
                            s.pin.release();
                            s.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                            cx.notify()
                        })
                    }));
                    if open {
                        for row in jump_rows(&mut side_need, sub) {
                            drawer = drawer.item(row);
                        }
                    }
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
