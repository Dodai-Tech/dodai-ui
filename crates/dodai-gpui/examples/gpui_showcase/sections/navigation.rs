//! The "Navigation" category sections.

use super::super::*;

impl Showcase {
    #[allow(unused_variables, clippy::redundant_clone, clippy::vec_init_then_push)]
    pub(crate) fn sections_navigation(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
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
        let gap = px(t.spacing.md);
        let me_tab = me.clone();
        let me_sw = me.clone();
        let me_ck = me.clone();
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let col = || div().flex().flex_col().gap(gap);
        let tabs = Tabs::new("tabs")
            .labels(["Overview", "Netrunning", "Gear", "Settings"])
            .active(self.tab)
            .on_select(move |i, _, cx| {
                me_tab.update(cx, |s, cx| {
                    s.tab = i;
                    cx.notify();
                })
            });
        let controls = row()
            .child(Toggle::new("sw", self.switch).on_change(move |v, _, cx| {
                me_sw.update(cx, |s, cx| {
                    s.switch = v;
                    cx.notify();
                })
            }))
            .child(Label::new("Switch"))
            .child(Toggle::new("sw-dis", true).disabled(true))
            .child(Checkbox::new("ck", self.check).on_change(move |v, _, cx| {
                me_ck.update(cx, |s, cx| {
                    s.check = v;
                    cx.notify();
                })
            }))
            .child(Label::new("Checkbox"))
            .child(Checkbox::new("ck-dis", false).disabled(true));
        let m3 = me.clone();
        let nav_items = || {
            [
                NavItem::new("⌂", "Home"),
                NavItem::new("✉", "Mail").badge(3),
                NavItem::new("★", "Saved"),
                NavItem::new("⚙", "Settings"),
            ]
        };
        let structure = div()
            .flex()
            .flex_col()
            .gap(gap)
            .child(
                TopAppBar::new(dodai_gpui::family_title(t.meta.family))
                    .leading("☰")
                    .child(IconButton::new("ab-search", "⌕").icon(IconName::Search))
                    .child(IconButton::new("ab-more", "⋮").icon(IconName::Dots)),
            )
            .child(SearchBar::new("search-bar").placeholder("Search the net…"))
            .child(
                row()
                    .child(
                        IconButton::new("ib1", "♥")
                            .icon(IconName::Heart)
                            .toggled(true),
                    )
                    .child(
                        IconButton::new("ib2", "♥")
                            .icon(IconName::Heart)
                            .toggled(false),
                    )
                    .child(IconButton::new("ib3", "✎").icon(IconName::Edit))
                    .child(
                        IconButton::new("ib4", "✎")
                            .icon(IconName::Edit)
                            .disabled(true),
                    )
                    .child(
                        Fab::new("fab-s", "+")
                            .icon(IconName::Plus)
                            .size(FabSize::Small),
                    )
                    .child(Fab::new("fab", "+").icon(IconName::Plus))
                    .child(dodai_gpui::FabMenu::new("fab-menu", IconName::Plus).items([
                        (IconName::Edit, "Compose"),
                        (IconName::Copy, "Duplicate"),
                        (IconName::Heart, "Save"),
                    ]))
                    .child(
                        Fab::new("fab-l", "+")
                            .icon(IconName::Plus)
                            .size(FabSize::Large),
                    )
                    .child(
                        Fab::new("fab-x", "✎")
                            .icon(IconName::Edit)
                            .extended("Compose"),
                    ),
            )
            .child(
                row()
                    .items_start()
                    .child(
                        div().flex_1().min_w(px(220.0)).child(
                            Card::new(CardVariant::Elevated)
                                .headline("Elevated")
                                .supporting("Raised surface."),
                        ),
                    )
                    .child(
                        div().flex_1().min_w(px(220.0)).child(
                            Card::new(CardVariant::Filled)
                                .headline("Filled")
                                .supporting("Flat raised surface."),
                        ),
                    )
                    .child(
                        div().flex_1().min_w(px(220.0)).child(
                            Card::new(CardVariant::Outlined)
                                .headline("Outlined")
                                .supporting("Border only."),
                        ),
                    ),
            )
            .child(
                Navigation::new("nav-bar", NavLayout::Bar)
                    .items(nav_items())
                    .active(self.nav)
                    .on_select({
                        let m = m3.clone();
                        move |i, _, cx| {
                            m.update(cx, |s, cx| {
                                s.nav = i;
                                cx.notify();
                            })
                        }
                    }),
            )
            .child(
                row()
                    .items_start()
                    .child(
                        Navigation::new("nav-rail", NavLayout::Rail)
                            .header(
                                Fab::new("rail-fab", "+")
                                    .icon(IconName::Edit)
                                    .size(FabSize::Small),
                            )
                            .items(nav_items())
                            .active(self.nav)
                            .on_select({
                                let m = m3.clone();
                                move |i, _, cx| {
                                    m.update(cx, |s, cx| {
                                        s.nav = i;
                                        cx.notify();
                                    })
                                }
                            }),
                    )
                    .child(
                        Navigation::new("nav-drawer", NavLayout::Drawer)
                            .items(nav_items())
                            .active(self.nav)
                            .on_select(move |i, _, cx| {
                                m3.update(cx, |s, cx| {
                                    s.nav = i;
                                    cx.notify();
                                })
                            }),
                    ),
            );
        let open = !self.search.read(cx).text().is_empty();
        let m = me.clone();
        let search = SearchView::new("search", self.search.clone())
            .items(["Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"])
            .recent(2)
            .open(open)
            .on_clear({
                let m = me.clone();
                move |w, cx| {
                    m.update(cx, |s, cx| {
                        s.search.update(cx, |i, cx| {
                            i.set_text("", cx);
                            let _ = w;
                        })
                    })
                }
            })
            .on_pick(move |_, _, _| {
                let _ = &m;
            });
        let (m1, m2, m3, m4, m5, m6) = (
            me.clone(),
            me.clone(),
            me.clone(),
            me.clone(),
            me.clone(),
            me.clone(),
        );
        let popovers = row()
            .child(
                dodai_gpui::Popover::new("pop")
                    .trigger(Button::new("pop-btn", "Popover").variant(ButtonVariant::Outline))
                    .content(
                        col()
                            .child(Label::new("Popover"))
                            .child(Muted::new("Click outside or press Esc to close."))
                            .child(Button::new("pop-in", "Action")),
                    )
                    .width(240.0),
            )
            .child(
                dodai_gpui::HoverCard::new("hc")
                    .trigger(
                        div()
                            .px(px(8.0))
                            .text_color(hsla(t.palette.link))
                            .child("@hover-me"),
                    )
                    .content(
                        row().child(dodai_gpui::Avatar::new("DD").size(40.0)).child(
                            col()
                                .child(Label::new("Dodai UI"))
                                .child(Muted::new("Design system for Rust UIs")),
                        ),
                    ),
            )
            .child(
                dodai_gpui::ContextMenu::new("ctx")
                    .child(
                        div()
                            .px(px(24.0))
                            .py(px(14.0))
                            .rounded(px(t.shape.radius.min(8.0)))
                            .border_1()
                            .border_color(hsla(t.palette.border_strong))
                            .text_color(hsla(t.palette.text_muted))
                            .child("Right-click here"),
                    )
                    .items([
                        dodai_gpui::MenuItem::new("Copy").shortcut("⌘C"),
                        dodai_gpui::MenuItem::new("Paste").shortcut("⌘V"),
                        dodai_gpui::MenuItem::separator(),
                        dodai_gpui::MenuItem::new("Delete").danger(true),
                    ]),
            )
            .child(dodai_gpui::DropdownMenu::new("dd", "Actions").items([
                dodai_gpui::MenuItem::new("Rename"),
                dodai_gpui::MenuItem::new("Duplicate"),
                dodai_gpui::MenuItem::separator(),
                dodai_gpui::MenuItem::new("Archive").disabled(true),
                dodai_gpui::MenuItem::new("Delete").danger(true),
            ]));
        let tree = TreeView::new(
            "tree",
            vec![
                TreeItem::new("src")
                    .expanded(true)
                    .child(TreeItem::new("lib.rs"))
                    .child(TreeItem::new("charts.rs")),
                TreeItem::new("examples").child(TreeItem::new("gpui_showcase.rs")),
            ],
        );
        out.push((
            "Navigation",
            "Popover · Hover card · Context menu · Dropdown menu",
            section!("Popover · Hover card · Context menu · Dropdown")
                .child(popovers)
                .into_any_element(),
            "Popover · Hover card · Context menu · Dropdown",
        ));
        out.push((
            "Navigation",
            "Menu · Sub-menu",
            section!("Menu · Sub-menu")
                .child(Muted::new(
                    "Leading icons, shortcuts, and a sub-menu (hover \"Share\").",
                ))
                .child(
                    dodai_gpui::Menu::new("menu-demo").width(240.0).items([
                        dodai_gpui::MenuItem::new("Edit")
                            .icon(IconName::Edit)
                            .shortcut("⌘E"),
                        dodai_gpui::MenuItem::new("Copy")
                            .icon(IconName::Copy)
                            .shortcut("⌘C"),
                        dodai_gpui::MenuItem::new("Share")
                            .icon(IconName::Heart)
                            .submenu([
                                dodai_gpui::MenuItem::new("Mail"),
                                dodai_gpui::MenuItem::new("Link"),
                                dodai_gpui::MenuItem::new("Export…"),
                            ]),
                        dodai_gpui::MenuItem::separator(),
                        dodai_gpui::MenuItem::new("Delete").danger(true),
                    ]),
                )
                .into_any_element(),
            "Menu · Sub-menu",
        ));
        out.push((
            "Navigation",
            "Tabs",
            section!("Tabs")
                .child(tabs)
                .child(Muted::new("Secondary"))
                .child(
                    Tabs::new("tabs-secondary")
                        .labels(["Overview", "Netrunning", "Gear", "Settings"])
                        .active(self.tab)
                        .secondary(true),
                )
                .child(Muted::new("Scrollable"))
                .child(
                    div().w(px(360.0)).child(
                        Tabs::new("tabs-scroll")
                            .labels([
                                "Overview",
                                "Netrunning",
                                "Gear",
                                "Settings",
                                "Contacts",
                                "Gigs",
                                "Archive",
                                "Help",
                            ])
                            .active(self.tab)
                            .scrollable(true),
                    ),
                )
                .child(Muted::new("With icons"))
                .child(
                    Tabs::new("tabs-icons")
                        .labels(["Home", "Saved", "Search"])
                        .icons([IconName::Home, IconName::Heart, IconName::Search])
                        .active(self.tab % 3),
                )
                .child(Muted::new(format!("active tab: {}", self.tab)))
                .into_any_element(),
            "Tabs",
        ));
        out.push((
                "Navigation",
                "Design system tree",
                section!("Design system tree")
                    .child(Muted::new(
                        "Click the active system to hide / show its themes. Selecting a theme applies it live.",
                    ))
                    .child(
                        row()
                            .items_start()
                            .gap(px(24.0))
                            .child(
                                div()
.flex()
.flex_col()
.gap(gap)
.flex_1()
.min_w(px(220.0))
                                    .child(Label::new("Tree (default)"))
                                    .child(DesignSystemTree::new("dst-default")),
                            )
                            .child(
                                div()
.flex()
.flex_col()
.gap(gap)
.flex_1()
.min_w(px(220.0))
                                    .child(Label::new("With properties"))
                                    .child(DesignSystemTree::new("dst-props").properties(
                                        "cyberpunk",
                                        Muted::new("Family-specific controls go here."),
                                    )),
                            )
                            .child(
                                div()
.flex()
.flex_col()
.gap(gap)
.flex_1()
.min_w(px(220.0))
                                    .child(Label::new("Dropdown (dropdown_over 0)"))
                                    .child(DesignSystemTree::new("dst-dd").dropdown_over(0)),
                            ),
                    )
                    .into_any_element(),
                "Design system tree",
            ));
        out.push((
                "Navigation",
                "Collapsible sidebar",
                section!("Collapsible sidebar")
                    .child(Muted::new(
                        "ResponsiveShell: drag the edge handle (spring + fling snap), click it, or use the button. Below the breakpoint it becomes a drawer.",
                    ))
                    .child({
                        let demo = self.demo_rail;
                        let m = me.clone();
                        let m2 = me.clone();
                        let items = ["Inbox", "Drafts", "Sent", "Archive"];
                        let mut side = div()
                            .flex()
                            .flex_col()
                            .gap(px(t.spacing.xs))
                            .p(px(t.spacing.sm))
                            .child(
                                Button::new("demo-rail-btn", if demo { "»" } else { "« Collapse" })
                                    .variant(ButtonVariant::Outline)
                                    .on_click(move |_, _, cx| {
                                        m2.update(cx, |s, cx| {
                                            s.demo_rail = !s.demo_rail;
                                            cx.notify()
                                        })
                                    }),
                            );
                        for (i, name) in items.into_iter().enumerate() {
                            let label = if demo {
                                name.chars().take(1).collect::<String>()
                            } else {
                                name.to_string()
                            };
                            side = side.child(ListItem::new(("demo-li", i), label));
                        }
                        div()
                            .relative()
                            .h(px(220.0))
                            .w_full()
                            .overflow_hidden()
                            .border_1()
                            .border_color(dodai_gpui::hsla(t.palette.border))
                            .child(
                                ResponsiveShell::new("demo-shell")
                                    .breakpoint(0.0)
                                    .sidebar_width(200.0)
                                    .rail_width(64.0)
                                    .sidebar(side)
                                    .collapsed(demo)
                                    .on_collapse(move |c, _, cx| {
                                        m.update(cx, |s, cx| {
                                            s.demo_rail = c;
                                            cx.notify()
                                        })
                                    })
                                    .content(
                                        div()
                                            .p(px(t.spacing.md))
                                            .child(Muted::new("Content area follows the sidebar width.")),
                                    ),
                            )
                    })
                    .into_any_element(),
                "Collapsible sidebar",
            ));
        let sel_me = me.clone();
        let strips = div()
            .flex()
            .flex_col()
            .gap(gap)
            .child(
                div().flex().child(
                    Segmented::new("seg", ["Day", "Week", "Month"])
                        .selected(self.group_sel)
                        .on_change(move |i, _, cx| {
                            sel_me.update(cx, |s, cx| {
                                s.group_sel = i;
                                cx.notify()
                            })
                        }),
                ),
            )
            .child(Breadcrumb::new("crumbs", ["Home", "Library", "Data"]))
            .child(Pagination::new("pages", 5, 20))
            .child(Stepper::new(["Account", "Profile", "Done"], 1))
            .child(
                Accordion::new("acc")
                    .item("What is Dodai?", "A themeable design system for Rust UIs.")
                    .item(
                        "Does it support Material 3?",
                        "Yes, tonal roles, shapes and state layers.",
                    )
                    .open([0]),
            );
        out.push((
            "Navigation",
            "Segmented · Breadcrumb · Pagination · Stepper",
            section!("Segmented · Breadcrumb · Pagination · Stepper")
                .child(strips)
                .into_any_element(),
            "Segmented · Breadcrumb · Pagination · Stepper",
        ));
        let tile = |c: dodai::Color, label: &'static str| {
            div()
                .h(px(120.0))
                .w_full()
                .rounded(px(t.role_shape(dodai::ShapeRole::Carousel).radius))
                .bg(hsla(c))
                .p(px(t.spacing.md))
                .whitespace_nowrap()
                .overflow_hidden()
                .text_color(hsla(t.on(c)))
                .child(label)
        };
        let r = &t.palette.roles;
        let carousel = Carousel::new("car")
            .visible(3)
            .item_width(220.0)
            .active(self.group_sel)
            .item(tile(r.primary_container, "Primary"))
            .item(tile(r.secondary_container, "Secondary"))
            .item(tile(r.tertiary_container, "Tertiary"))
            .item(tile(r.surface_container_high, "Surface"))
            .item(tile(r.error_container, "Error"));
        let mut layouts = div().flex().flex_col().gap(gap).child(carousel);
        for (name, l) in [
            ("Multi-browse", dodai_gpui::CarouselLayout::MultiBrowse),
            ("Hero", dodai_gpui::CarouselLayout::Hero),
            (
                "Center-aligned hero",
                dodai_gpui::CarouselLayout::CenterHero,
            ),
        ] {
            let mut c = Carousel::new(name)
                .layout(l)
                .width(560.0)
                .item_height(140.0)
                .active(self.group_sel);
            for (col, label) in [
                (r.primary_container, "Primary"),
                (r.secondary_container, "Secondary"),
                (r.tertiary_container, "Tertiary"),
                (r.surface_container_high, "Surface"),
                (r.error_container, "Error"),
            ] {
                c = c.item(tile(col, label).h_full());
            }
            layouts = layouts.child(Muted::new(name)).child(c);
        }
        out.push((
            "Navigation",
            "Carousel",
            section!("Carousel").child(layouts).into_any_element(),
            "Carousel",
        ));
        // Overlays shown inside a frame: they are absolutely positioned against their parent.
        let frame = |h: f32| {
            div()
                .relative()
                .h(px(h))
                .w_full()
                .overflow_hidden()
                .border_1()
                .border_color(dodai_gpui::hsla(t.palette.border))
                .p(px(t.spacing.md))
                .child(Muted::new("Page content behind the overlay."))
        };
        let dialog = frame(300.0).child(Dialog::new(
            "ov-dialog",
            DialogCard::new("Discard draft?")
                .icon(IconName::Heart)
                .width(360.0)
                .child(Muted::new(
                    "Your unsent message will be lost. This cannot be undone.",
                ))
                .action(Button::new("dg-cancel", "Cancel").variant(ButtonVariant::Ghost))
                .action(Button::new("dg-ok", "Discard").variant(ButtonVariant::Ghost)),
        ));
        let bottom = frame(260.0).child(
            BottomSheet::new("ov-bottom")
                .title("Share")
                .height(150.0)
                .child(Muted::new("Copy link · Message · Mail")),
        );
        let side = frame(260.0).child(
            SideSheet::new("ov-side", "Filters")
                .width(240.0)
                .child(Muted::new("Date range · Owner · Status")),
        );
        let modal_drawer = frame(260.0).child(
            dodai_gpui::ModalDrawer::new("ov-drawer")
                .items(nav_items())
                .active(self.nav),
        );
        out.push((
            "Navigation",
            "Modal navigation drawer",
            section!("Modal navigation drawer")
                .child(Muted::new(
                    "Scrim + drawer from the edge; picking an item dismisses.",
                ))
                .child(modal_drawer)
                .into_any_element(),
            "Modal navigation drawer",
        ));
        out.push((
            "Navigation",
            "Dialog · Bottom sheet · Side sheet",
            section!("Dialog · Bottom sheet · Side sheet")
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(gap)
                        .child(dialog)
                        .child(bottom)
                        .child(side),
                )
                .into_any_element(),
            "Dialog · Bottom sheet · Side sheet",
        ));
        let bar = |t: TopAppBar| {
            t.leading("☰")
                .child(IconButton::new("ab-a", "⌕").icon(IconName::Search))
                .child(IconButton::new("ab-b", "⋮").icon(IconName::Dots))
        };
        let family = dodai_gpui::family_title(t.meta.family);
        let labelled = |name: &'static str, el: gpui::AnyElement| {
            col().gap(px(4.0)).child(Muted::new(name)).child(el)
        };
        out.push((
            "Navigation",
            "App bars",
            section!("App bars")
                .child(
                    col()
                        .child(labelled(
                            "Center-aligned",
                            bar(TopAppBar::new(family.clone())
                                .size(dodai_gpui::AppBarSize::CenterAligned))
                            .into_any_element(),
                        ))
                        .child(labelled(
                            "Small",
                            bar(TopAppBar::new(family.clone())).into_any_element(),
                        ))
                        .child(labelled(
                            "Small · scrolled",
                            bar(TopAppBar::new(family.clone()).scrolled(true)).into_any_element(),
                        ))
                        .child(
                            labelled(
                                "Medium",
                                bar(TopAppBar::new(family.clone())
                                    .size(dodai_gpui::AppBarSize::Medium))
                                .into_any_element(),
                            ),
                        )
                        .child(labelled(
                            "Large",
                            bar(TopAppBar::new(family.clone()).size(dodai_gpui::AppBarSize::Large))
                                .into_any_element(),
                        ))
                        .child(labelled(
                            "Large · collapsing (50 %)",
                            bar(TopAppBar::new(family.clone())
                                .size(dodai_gpui::AppBarSize::Large)
                                .collapse(0.5)
                                .scrolled(true))
                            .into_any_element(),
                        ))
                        .child(labelled(
                            "Bottom",
                            dodai_gpui::BottomAppBar::new()
                                .child(IconButton::new("bb-1", "✎").icon(IconName::Edit))
                                .child(IconButton::new("bb-2", "♥").icon(IconName::Heart))
                                .child(IconButton::new("bb-3", "⋮").icon(IconName::Dots))
                                .trailing(Fab::new("bb-fab", "+").icon(IconName::Plus))
                                .into_any_element(),
                        )),
                )
                .into_any_element(),
            "App bars",
        ));
        out.push((
            "Navigation",
            "App bar · Cards · FAB · Navigation",
            section!("App bar · Cards · FAB · Navigation")
                .child(structure)
                .into_any_element(),
            "App bar · Cards · FAB · Navigation",
        ));
        out
    }
}
