//! `cargo run -p dodai-gpui --example gpui_showcase`
//!
//! Every dodai-gpui component in one scrollable window.
//! Keys: `t` / `T` next / previous theme. Glow +/- buttons change
//! `Theme::with_glow_intensity` live.

use dodai_core::{ButtonVariant, Theme, Tone};
use dodai_gpui::{
    cycle_next, cycle_prev, family_title, fuzzy_filter, glow_intensity, hsla, install,
    scanlines_enabled, set_glow_intensity, set_scanlines, theme_of, ActiveDodai, Button, Card,
    CardVariant, Checkbox, DesignSystemSelector, Direction, Divider, DodaiStyled, DodaiTooltip,
    Fab, FabSize, Heading, IconButton, InputBox, Knob, Label, ListItem, Mono, Muted, NavItem,
    NavLayout, Navigation, Panel, ProgressBar, Radio, Scanlines, SearchBar, Slider, Tabs, Tag,
    ThemeSelector, Toggle, TopAppBar,
};
use dodai_gpui::{
    BarChart, Date, DatePicker, EmptyState, Gauge, Heatmap, IconBadge, LineChart, NumberStepper,
    PieChart, RadarChart, RangeSlider, Rating, RichTooltip, ScatterPlot, SearchView, Series,
    Snackbar, Sparkline, TagField, TextInput, TimePicker, Timeline, TimelineEvent, Toolbar,
    TreeItem, TreeView,
};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Entity, FocusHandle,
    KeyDownEvent, Window, WindowBounds, WindowOptions,
};

struct Showcase {
    filter: Entity<TextInput>,
    search: Entity<TextInput>,
    cat: usize,
    rating: u32,
    stepper: i32,
    range: (f32, f32),
    vrange: (f32, f32),
    stepped: f32,
    db: f32,
    date: (i32, u32, Option<Date>),
    time: (u32, u32),
    tags: Vec<&'static str>,
    focus: FocusHandle,
    tab: usize,
    switch: bool,
    check: bool,
    radio: usize,
    slider: f32,
    nav: usize,
    knob: [f32; 3],
    vslider: [f32; 3],
    list: usize,
    clicks: u32,
}

impl Render for Showcase {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme_of(cx);
        let me = cx.entity();
        let gap = px(t.spacing.md);
        let glow = glow_intensity(cx);
        let scan = scanlines_enabled(cx);

        let me_tab = me.clone();
        let me_sw = me.clone();
        let me_ck = me.clone();
        let me_sl = me.clone();

        let section = |title: &'static str| Panel::new().title(title);

        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let col = || div().flex().flex_col().gap(gap);

        let buttons = row()
            .child(
                Button::new("b-primary", "Primary").on_click(cx.listener(|s, _, _, cx| {
                    s.clicks += 1;
                    cx.notify();
                })),
            )
            .child(Button::new("b-secondary", "Secondary").variant(ButtonVariant::Secondary))
            .child(Button::new("b-outline", "Outline").variant(ButtonVariant::Outline))
            .child(Button::new("b-outline2", "Outline 2").variant(ButtonVariant::OutlineSecondary))
            .child(Button::new("b-danger", "Danger").variant(ButtonVariant::Danger))
            .child(Button::new("b-ghost", "Ghost").variant(ButtonVariant::Ghost))
            .child(Button::new("b-dis", "Disabled").disabled(true))
            .child(Muted::new(format!("primary clicked {}×", self.clicks)));

        let tags = row()
            .child(Tag::new("online", Tone::Success).solid(true))
            .child(Tag::new("warn", Tone::Warning).solid(true))
            .child(Tag::new("breach", Tone::Danger).solid(true))
            .child(Tag::new("info", Tone::Info).solid(true))
            .child(Tag::new("online", Tone::Success))
            .child(Tag::new("warn", Tone::Warning))
            .child(Tag::new("breach", Tone::Danger))
            .child(Tag::new("info", Tone::Info));

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

        let mut radios = row();
        for (i, name) in ["Netwatch", "Arasaka", "Militech"].into_iter().enumerate() {
            let m = me.clone();
            radios = radios
                .child(
                    Radio::new(("radio", i), self.radio == i).on_change(move |_, _, cx| {
                        m.update(cx, |s, cx| {
                            s.radio = i;
                            cx.notify();
                        })
                    }),
                )
                .child(Label::new(name));
        }

        let sliders = div()
            .flex()
            .flex_col()
            .gap(gap)
            .child(
                Slider::new("slider", self.slider).on_change(move |v, _, cx| {
                    me_sl.update(cx, |s, cx| {
                        s.slider = v;
                        cx.notify();
                    })
                }),
            )
            .child(Mono::new(format!("value {:.2}", self.slider)))
            .child(ProgressBar::new(self.slider))
            .child(ProgressBar::new(0.7).tone(Tone::Success))
            .child(ProgressBar::new(0.4).tone(Tone::Warning))
            .child(ProgressBar::new(0.15).tone(Tone::Danger));

        let mut dials = row().gap(px(24.0));
        for i in 0..3 {
            let m = me.clone();
            dials = dials.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(gap)
                    .child(
                        Knob::new(("knob", i), self.knob[i])
                            .size(if i == 1 { 96.0 } else { 72.0 })
                            .disabled(i == 2)
                            .on_change(move |v, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.knob[i] = v;
                                    cx.notify();
                                })
                            }),
                    )
                    .child(Mono::new(format!("{:.2}", self.knob[i]))),
            );
        }
        for i in 0..3 {
            let m = me.clone();
            dials = dials.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(gap)
                    .child(
                        Slider::new(("vslider", i), self.vslider[i])
                            .direction(Direction::Vertical)
                            .length(120.0)
                            .on_change(move |v, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.vslider[i] = v;
                                    cx.notify();
                                })
                            }),
                    )
                    .child(Mono::new(format!("{:.2}", self.vslider[i]))),
            );
        }

        {
            let m = me.clone();
            dials = dials.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(gap)
                    .child(
                        RangeSlider::new("vrange", self.vrange.0, self.vrange.1)
                            .direction(Direction::Vertical)
                            .length(120.0)
                            .step(0.1)
                            .ticks(true)
                            .on_change(move |a, b, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.vrange = (a, b);
                                    cx.notify()
                                })
                            }),
                    )
                    .child(Mono::new(format!(
                        "{:.1}–{:.1}",
                        self.vrange.0, self.vrange.1
                    ))),
            );
        }

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
                TopAppBar::new("Cyberpunk")
                    .leading("☰")
                    .child(IconButton::new("ab-search", "⌕"))
                    .child(IconButton::new("ab-more", "⋮")),
            )
            .child(SearchBar::new("search-bar").placeholder("Search the net…"))
            .child(
                row()
                    .child(IconButton::new("ib1", "♥").toggled(true))
                    .child(IconButton::new("ib2", "♥").toggled(false))
                    .child(IconButton::new("ib3", "✎"))
                    .child(IconButton::new("ib4", "✎").disabled(true))
                    .child(Fab::new("fab-s", "+").size(FabSize::Small))
                    .child(Fab::new("fab", "+"))
                    .child(Fab::new("fab-l", "+").size(FabSize::Large))
                    .child(Fab::new("fab-x", "✎").extended("Compose")),
            )
            .child(
                row()
                    .items_start()
                    .child(
                        Card::new(CardVariant::Elevated)
                            .headline("Elevated")
                            .supporting("Raised surface with hard shadow.")
                            .width(220.0),
                    )
                    .child(
                        Card::new(CardVariant::Filled)
                            .headline("Filled")
                            .supporting("Flat raised surface.")
                            .width(220.0),
                    )
                    .child(
                        Card::new(CardVariant::Outlined)
                            .headline("Outlined")
                            .supporting("Border only.")
                            .width(220.0),
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

        let mut list = div().flex().flex_col().gap(px(t.spacing.xs));
        for (i, name) in ["Inbox", "Contracts", "Fixers", "Black market"]
            .into_iter()
            .enumerate()
        {
            let m = me.clone();
            list = list.child(
                ListItem::new(("li", i), name)
                    .selected(self.list == i)
                    .on_click(move |_, _, cx| {
                        m.update(cx, |s, cx| {
                            s.list = i;
                            cx.notify();
                        })
                    }),
            );
        }

        let family = family_title(cx.dodai().switcher().current_family());
        let ds_panel = section("Design system")
            .focused(true)
            .child(DesignSystemSelector::new("ds-select"));
        let theme_panel = Panel::new()
            .title(family)
            .child(
                row()
                    .child(Label::new("Theme"))
                    .child(ThemeSelector::new("theme-select")),
            )
            .child(
                row()
                    .child(Label::new("Glow intensity"))
                    .child(
                        Button::new("glow-", "−")
                            .variant(ButtonVariant::Outline)
                            .on_click(move |_, _, cx| {
                                let g = glow_intensity(cx);
                                set_glow_intensity(cx, (g - 0.25).max(0.0));
                            }),
                    )
                    .child(Mono::new(format!("{glow:.2}")))
                    .child(
                        Button::new("glow+", "+")
                            .variant(ButtonVariant::Outline)
                            .on_click(move |_, _, cx| {
                                let g = glow_intensity(cx);
                                set_glow_intensity(cx, g + 0.25);
                            }),
                    )
                    .child(
                        Button::new(
                            "scan",
                            if scan {
                                "Scanlines: on"
                            } else {
                                "Scanlines: off"
                            },
                        )
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, _, cx| {
                            let on = scanlines_enabled(cx);
                            set_scanlines(cx, !on);
                        }),
                    )
                    .child(Muted::new("keys: t / T theme · l scanlines")),
            );

        let open = !self.search.read(cx).text().is_empty();
        let m = me.clone();
        let search = SearchView::new("search", self.search.clone())
            .items(["Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"])
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
        let w_search = search;
        let w_misc = row()
            .child(
                Rating::new("rating", self.rating).on_change(move |v, _, cx| {
                    m1.update(cx, |s, cx| {
                        s.rating = v;
                        cx.notify()
                    })
                }),
            )
            .child(
                NumberStepper::new("step", self.stepper)
                    .range(0, 20)
                    .on_change(move |v, _, cx| {
                        m2.update(cx, |s, cx| {
                            s.stepper = v;
                            cx.notify()
                        })
                    }),
            )
            .child(
                div()
                    .id("rich")
                    .text_color(hsla(t.palette.link))
                    .tooltip(RichTooltip::build(
                        "Rich tooltip",
                        "Title, body and an action.",
                        Some("Learn more"),
                    ))
                    .child("Hover (rich)"),
            )
            .child(IconBadge::new(IconButton::new("bell", "🔔")).count(12, Tone::Danger))
            .child(IconBadge::new(IconButton::new("mail", "✉")).dot(Tone::Success));
        let w_range =
            RangeSlider::new("range", self.range.0, self.range.1).on_change(move |a, b, _, cx| {
                m3.update(cx, |s, cx| {
                    s.range = (a, b);
                    cx.notify()
                })
            });
        let w_range_txt = Muted::new(format!("range {:.2} – {:.2}", self.range.0, self.range.1));
        let w_tags = TagField::new("tags", self.tags.clone()).on_remove(move |i, _, cx| {
            m4.update(cx, |s, cx| {
                s.tags.remove(i);
                cx.notify()
            })
        });
        let w_snack = Snackbar::new("snack", "Message archived").action("Undo", |_, _| {});
        let w_toolbar = Toolbar::new()
            .child(IconButton::new("tb1", "✎"))
            .child(IconButton::new("tb2", "⎘"));

        let (y, mo, sel) = self.date;
        let pickers = row()
            .child(
                DatePicker::new("date", y, mo)
                    .selected(sel)
                    .on_select({
                        let m = m5.clone();
                        move |d, _, cx| {
                            m.update(cx, |s, cx| {
                                s.date.2 = Some(d);
                                cx.notify()
                            })
                        }
                    })
                    .on_month(move |y, mo, _, cx| {
                        m5.update(cx, |s, cx| {
                            s.date.0 = y;
                            s.date.1 = mo;
                            cx.notify()
                        })
                    }),
            )
            .child(TimePicker::new("time", self.time.0, self.time.1).on_change(
                move |h, mi, _, cx| {
                    m6.update(cx, |s, cx| {
                        s.time = (h, mi);
                        cx.notify()
                    })
                },
            ));

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
        let timeline = Timeline::new([
            TimelineEvent::new("Charts added")
                .time("today")
                .tone(Tone::Success),
            TimelineEvent::new("Search view")
                .time("today")
                .detail("fuzzy + highlight"),
            TimelineEvent::new("Build failed")
                .time("earlier")
                .tone(Tone::Danger),
        ]);

        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
        let a = Series::new("Revenue", [12.0, 18.0, 14.0, 26.0, 22.0, 31.0]);
        let b = Series::new("Cost", [8.0, 11.0, 13.0, 15.0, 17.0, 16.0]);
        let charts_xy = col()
            .child(
                row()
                    .child(
                        div().w(px(420.0)).child(
                            LineChart::new("line", [a.clone(), b.clone()])
                                .labels(months)
                                .area(true)
                                .smooth(true),
                        ),
                    )
                    .child(
                        div()
                            .w(px(420.0))
                            .child(BarChart::new("bars", [a.clone(), b.clone()]).labels(months)),
                    ),
            )
            .child(
                row()
                    .child(
                        div().w(px(420.0)).child(
                            BarChart::new("stack", [a.clone(), b.clone()])
                                .labels(months)
                                .stacked(true),
                        ),
                    )
                    .child(
                        div().w(px(420.0)).child(
                            BarChart::new("hbars", [a, b])
                                .labels(months)
                                .horizontal(true),
                        ),
                    ),
            );
        let charts_round = row()
            .child(PieChart::new([
                ("Rust", 52.0),
                ("Kotlin", 28.0),
                ("Other", 20.0),
            ]))
            .child(
                PieChart::new([("A", 3.0), ("B", 2.0), ("C", 1.0)])
                    .donut(0.62)
                    .center("6"),
            )
            .child(Gauge::new(72.0).label("72%").tone(Tone::Success))
            .child(Gauge::new(31.0).tone(Tone::Warning))
            .child(Sparkline::new([3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 12.0]));
        let charts_misc = row()
            .child(RadarChart::new(
                ["Speed", "Power", "Range", "Armor", "Stealth"],
                [
                    Series::new("Unit A", [8.0, 6.0, 7.0, 4.0, 9.0]),
                    Series::new("Unit B", [5.0, 9.0, 4.0, 8.0, 3.0]),
                ],
            ))
            .child(
                div().w(px(380.0)).child(ScatterPlot::new(
                    "sc",
                    [
                        (
                            "Group 1",
                            (0..12)
                                .map(|i| (i as f32, (i * i % 7) as f32 + 1.0))
                                .collect(),
                        ),
                        (
                            "Group 2",
                            (0..12)
                                .map(|i| (i as f32 + 0.5, (i * 3 % 8) as f32))
                                .collect(),
                        ),
                    ],
                )),
            )
            .child(
                Heatmap::new(
                    "heat",
                    (0..5)
                        .map(|r| (0..8).map(|c| ((r * 3 + c * 5) % 11) as f32).collect())
                        .collect(),
                )
                .row_labels(["Mon", "Tue", "Wed", "Thu", "Fri"])
                .col_labels(["0", "3", "6", "9", "12", "15", "18", "21"]),
            );

        // Stepped / configurable sliders.
        let (ms1, ms2) = (me.clone(), me.clone());
        let stepped = col()
            .child(
                Slider::new("stepped", self.stepped)
                    .range(0.0, 10.0)
                    .step(1.0)
                    .ticks(true)
                    .show_value(true)
                    .on_change(move |v, _, cx| {
                        ms1.update(cx, |s, cx| {
                            s.stepped = v;
                            cx.notify()
                        })
                    }),
            )
            .child(
                Slider::new("db", self.db)
                    .range(-24.0, 6.0)
                    .step(3.0)
                    .ticks(true)
                    .show_value(true)
                    .format(|v| format!("{v:+.0} dB"))
                    .on_change(move |v, _, cx| {
                        ms2.update(cx, |s, cx| {
                            s.db = v;
                            cx.notify()
                        })
                    }),
            );

        let sections: Vec<(&'static str, &'static str, gpui::AnyElement)> = vec![
            (
                "Foundations",
                "Text · Heading · Label · Mono",
                section("Text")
                    .child(Heading::new("Heading 1").level(1))
                    .child(Heading::new("Heading 2"))
                    .child(Heading::new("Heading 3").level(3))
                    .child(Label::new("Label text"))
                    .child(Muted::new("Muted text"))
                    .child(Mono::new("mono_text() // 0xDEADBEEF"))
                    .into_any_element(),
            ),
            (
                "Actions",
                "Buttons · IconButton · FAB",
                section("Buttons").child(buttons).into_any_element(),
            ),
            (
                "Actions",
                "Toolbar · Snackbar",
                section("Toolbar · Snackbar")
                    .child(w_toolbar)
                    .child(w_snack)
                    .into_any_element(),
            ),
            (
                "Data display",
                "Tags · Chips · Badges",
                section("Tags").child(tags).into_any_element(),
            ),
            (
                "Data display",
                "Badges · Rating · Tooltips",
                section("Rating · Stepper · Badges · Tooltips")
                    .child(w_misc)
                    .into_any_element(),
            ),
            (
                "Data display",
                "List · Divider",
                section("List · Divider")
                    .child(list)
                    .child(Divider::new())
                    .into_any_element(),
            ),
            (
                "Data display",
                "Tree · Timeline · Empty state",
                section("Tree · timeline · empty state")
                    .child(
                        row()
                            .child(div().w(px(240.0)).child(tree))
                            .child(div().w(px(280.0)).child(timeline))
                            .child(
                                EmptyState::new("∅", "Nothing here")
                                    .message("Add something to get started."),
                            ),
                    )
                    .into_any_element(),
            ),
            (
                "Data display",
                "Tooltip · Rich tooltip",
                section("Tooltip")
                    .child(
                        div()
                            .id("tip")
                            .w(px(220.0))
                            .text_color(hsla(t.palette.link))
                            .tooltip(DodaiTooltip::text("Themed tooltip"))
                            .child("Hover me"),
                    )
                    .into_any_element(),
            ),
            (
                "Inputs",
                "Toggle · Checkbox · Radio",
                section("Toggle · Checkbox · Radio")
                    .child(controls)
                    .child(radios)
                    .into_any_element(),
            ),
            (
                "Inputs",
                "Slider · Progress",
                section("Slider · Progress")
                    .child(sliders)
                    .into_any_element(),
            ),
            (
                "Inputs",
                "Stepped slider · ticks · formatted value",
                section("Slider (stepped, ticks, value readout)")
                    .child(stepped)
                    .into_any_element(),
            ),
            (
                "Inputs",
                "Range slider",
                section("Range slider")
                    .child(w_range)
                    .child(w_range_txt)
                    .into_any_element(),
            ),
            (
                "Inputs",
                "Knob · Vertical slider · Vertical range slider",
                section("Knob · Vertical sliders")
                    .child(dials)
                    .into_any_element(),
            ),
            (
                "Inputs",
                "Input field",
                section("Input")
                    .child(InputBox::new().placeholder("search the net…"))
                    .child(InputBox::new().focused(true).child("focused input"))
                    .child(
                        div()
                            .dodai_input(&t, false)
                            .h(px(t.shape.control_height))
                            .flex()
                            .items_center()
                            .child("legacy dodai_input container"),
                    )
                    .into_any_element(),
            ),
            (
                "Inputs",
                "Search view · SearchBar",
                section("Search view").child(w_search).into_any_element(),
            ),
            (
                "Inputs",
                "Tag field · chips",
                section("Tag field").child(w_tags).into_any_element(),
            ),
            (
                "Inputs",
                "Date picker · Time picker",
                section("Date / time pickers")
                    .child(pickers)
                    .into_any_element(),
            ),
            (
                "Navigation",
                "Tabs",
                section("Tabs")
                    .child(tabs)
                    .child(Muted::new(format!("active tab: {}", self.tab)))
                    .into_any_element(),
            ),
            (
                "Navigation",
                "App bar · Cards · FAB · Navigation",
                section("App bar · Cards · FAB · Navigation")
                    .child(structure)
                    .into_any_element(),
            ),
            (
                "Charts",
                "Line · Bar · Area · Stacked charts",
                section("Line & bar charts")
                    .child(charts_xy)
                    .into_any_element(),
            ),
            (
                "Charts",
                "Pie · Donut · Gauge · Sparkline",
                section("Pie · donut · gauge · sparkline")
                    .child(charts_round)
                    .into_any_element(),
            ),
            (
                "Charts",
                "Radar · Scatter · Heatmap",
                section("Radar · scatter · heatmap")
                    .child(charts_misc)
                    .into_any_element(),
            ),
        ];

        let cats: Vec<&'static str> = {
            let mut v: Vec<&'static str> = Vec::new();
            for (c, _, _) in &sections {
                if !v.contains(c) {
                    v.push(c);
                }
            }
            v
        };
        let query = self.filter.read(cx).text().to_string();
        let selected_cat = self.cat.checked_sub(1).and_then(|i| cats.get(i).copied());

        // Sidebar: filter + category list with counts.
        let count_in = |c: Option<&'static str>| {
            sections
                .iter()
                .filter(|(sc, title, _)| {
                    c.is_none_or(|c| c == *sc)
                        && (query.is_empty()
                            || !fuzzy_filter(&query, &[format!("{sc} {title}")]).is_empty())
                })
                .count()
        };
        let mut side = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w(px(240.0))
            .flex_shrink_0()
            .h_full()
            .p(px(t.spacing.md))
            .child(Heading::new("Dodai UI").level(2))
            .child(div().pb(px(t.spacing.sm)).child(self.filter.clone()));
        for (idx, name) in std::iter::once("All")
            .chain(cats.iter().copied())
            .enumerate()
        {
            let m = me.clone();
            let n = count_in(if idx == 0 { None } else { Some(cats[idx - 1]) });
            side = side.child(
                ListItem::new(("cat", idx), format!("{name}  ·  {n}"))
                    .selected(self.cat == idx)
                    .on_click(move |_, _, cx| {
                        m.update(cx, |s, cx| {
                            s.cat = idx;
                            cx.notify()
                        })
                    }),
            );
        }
        side = side.child(
            div()
                .pt(px(t.spacing.md))
                .child(Muted::new("t / T theme · l scanlines")),
        );

        let mut body = div()
            .flex()
            .flex_col()
            .gap(gap)
            .p(px(t.spacing.xl))
            .child(ds_panel)
            .child(theme_panel);
        let mut shown = 0;
        let mut last_cat = "";
        for (c, title, el) in sections {
            let in_cat = selected_cat.is_none_or(|sc| sc == c);
            let hit =
                query.is_empty() || !fuzzy_filter(&query, &[format!("{c} {title}")]).is_empty();
            if !(in_cat && hit) {
                continue;
            }
            if c != last_cat {
                body = body.child(Heading::new(c).level(2));
                last_cat = c;
            }
            shown += 1;
            body = body.child(el);
        }
        if shown == 0 {
            body = body.child(
                EmptyState::new("⌕", "No components match")
                    .message("Clear the filter or pick another category."),
            );
        }

        div()
            .id("root")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|s, e: &KeyDownEvent, w, cx| {
                if !s.focus.is_focused(w) {
                    return;
                }
                if e.keystroke.key == "l" {
                    let on = scanlines_enabled(cx);
                    set_scanlines(cx, !on);
                }
                if e.keystroke.key == "t" {
                    if e.keystroke.modifiers.shift {
                        cycle_prev(cx)
                    } else {
                        cycle_next(cx)
                    }
                }
            }))
            .relative()
            .size_full()
            .dodai_background(&t)
            .flex()
            .child(div().id("side").h_full().overflow_y_scroll().child(side))
            .child(
                div()
                    .id("scroll")
                    .flex_1()
                    .h_full()
                    .overflow_y_scroll()
                    .child(body),
            )
            .child(Scanlines::new())
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        install(cx, Theme::default());
        let bounds = Bounds::centered(None, size(px(980.0), px(760.0)), cx);
        let win = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    let search = cx.new(|cx| TextInput::new(cx).placeholder("Search…"));
                    let filter = cx.new(|cx| TextInput::new(cx).placeholder("Filter components…"));
                    cx.new(|cx| Showcase {
                        filter,
                        search,
                        cat: 0,
                        rating: 3,
                        stepper: 5,
                        range: (0.2, 0.7),
                        vrange: (0.2, 0.8),
                        stepped: 4.0,
                        db: -6.0,
                        date: (2025, 6, None),
                        time: (9, 30),
                        tags: vec!["rust", "gpui", "charts"],
                        focus: cx.focus_handle(),
                        tab: 0,
                        switch: true,
                        check: true,
                        radio: 0,
                        slider: 0.4,
                        nav: 0,
                        knob: [0.35, 0.7, 0.5],
                        vslider: [0.3, 0.6, 0.9],
                        list: 0,
                        clicks: 0,
                    })
                },
            )
            .expect("open window");
        win.update(cx, |s, w, _| w.focus(&s.focus)).ok();
        cx.activate(true);
    });
}
