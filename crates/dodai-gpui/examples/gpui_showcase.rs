//! `cargo run -p dodai-gpui --example gpui_showcase`
//!
//! Every dodai-gpui component in one scrollable window.
//! Keys: `t` / `T` next / previous theme. Glow +/- buttons change
//! `Theme::with_glow_intensity` live.

use dodai_core::{ButtonVariant, Theme, Tone};
use dodai_gpui::{
    cycle_next, cycle_prev, family_title, fuzzy_filter, glow_intensity, hsla, install,
    scanlines_enabled, set_glow_intensity, set_scanlines, theme_of, ActiveDodai, BubblePlacement,
    Button, Card, CardVariant, Checkbox, DesignSystemSelector, Direction, Divider, DodaiStyled,
    DodaiTooltip, Fab, FabSize, Heading, IconButton, InputBox, Knob, Label, ListItem, Mono, Muted,
    NavItem, NavLayout, Navigation, Panel, ProgressBar, Radio, Scanlines, SearchBar, Slider, Tabs,
    Tag, ThemeSelector, Toggle, TopAppBar,
};
use dodai_gpui::{
    BarChart, BoxPlot, BubbleChart, Candle, Candlestick, Date, DatePicker, EmptyState, Funnel,
    Gauge, Heatmap, Histogram, IconBadge, LineChart, Marker, NumberStepper, PieChart, RadarChart,
    RadialBar, RangeSlider, Rating, RichTooltip, ScatterPlot, SearchView, Series, Snackbar,
    Sparkline, StackedAreaChart, TagField, TextInput, TimePicker, Timeline, TimelineEvent, Toolbar,
    TreeItem, TreeView, Treemap, Waterfall,
};
use dodai_gpui::{
    Easing, EntryKind, EntrySet, IndeterminateBar, Motion, Select, Skeleton, Spinner,
};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Entity, FocusHandle,
    KeyDownEvent, Window, WindowBounds, WindowOptions,
};
use gpui::{Animation, AnimationExt, SharedString};
use std::time::Duration;

struct Showcase {
    anim_kind: usize,
    anim_ease: usize,
    anim_ms: f32,
    replay: usize,
    sec_kind: std::collections::HashMap<&'static str, EntrySet>,
    pill_hover: Option<(&'static str, EntryKind)>,
    pop_hover: Option<(&'static str, EntryKind)>,
    filter: Entity<TextInput>,
    search: Entity<TextInput>,
    cat: usize,
    cat_count: usize,
    side_focus: FocusHandle,
    rating: u32,
    stepper: i32,
    range: (f32, f32),
    vrange: (f32, f32),
    stepped: f32,
    db: f32,
    bub: (f32, f32),
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

impl Showcase {
    /// Per-section animation chips (hover a chip for its duration popover).
    fn chips(
        &self,
        title: &'static str,
        t: &Theme,
        me: &gpui::Entity<Showcase>,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let tid = title
            .bytes()
            .fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32));
        let kind = self.sec_kind.get(title).copied();
        let chip = |id: (&'static str, u32), label: SharedString, on: bool| {
            div()
                .id(id)
                .px(px(6.0))
                .py(px(1.0))
                .rounded(px(4.0))
                .text_size(px(10.0))
                .cursor_pointer()
                .border_1()
                .border_color(hsla(t.palette.border))
                .text_color(hsla(if on {
                    t.palette.on_primary
                } else {
                    t.palette.text_muted
                }))
                .when(on, |d| d.bg(hsla(t.palette.primary)))
                .child(label)
        };
        let mut chips = div()
            .flex()
            .flex_none()
            .flex_wrap()
            .items_center()
            .justify_end()
            .gap(px(4.0))
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(hsla(t.palette.text_muted))
                    .child("Animations"),
            );
        let me_g = me.clone();
        chips = chips.child(
            chip(("sec-g", tid), "Global".into(), kind.is_none()).on_click(move |_, _, cx| {
                me_g.update(cx, |s, cx| {
                    s.sec_kind.remove(title);
                    s.replay += 1;
                    cx.notify();
                })
            }),
        );
        for (ki, k) in EntryKind::ALL.iter().copied().enumerate().skip(1) {
            let on = kind.is_some_and(|s| s.contains(k));
            let ms = kind.map_or(0, |s| s.ms_of(k));
            let open =
                on && (self.pill_hover == Some((title, k)) || self.pop_hover == Some((title, k)));
            let me_c = me.clone();
            let me_p = me.clone();
            let uid = tid.wrapping_mul(8).wrapping_add(ki as u32);
            let text: SharedString = if on && ms > 0 {
                format!("{} · {ms}ms", k.label()).into()
            } else {
                k.label().into()
            };
            let mut wrap = div()
                .id(("pw", uid))
                .relative()
                .on_hover(cx.listener(move |s, h: &bool, _, cx| {
                    if *h {
                        s.pill_hover = Some((title, k));
                    } else if s.pill_hover == Some((title, k)) {
                        s.pill_hover = None;
                    }
                    cx.notify();
                }))
                .child(chip(("sec-k", uid), text, on).on_click(move |_, _, cx| {
                    me_c.update(cx, |s, cx| {
                        let cur = s.sec_kind.get(title).copied().unwrap_or(EntrySet::NONE);
                        s.sec_kind.insert(title, cur.toggled(k));
                        s.replay += 1;
                        cx.notify();
                    })
                }));
            if open {
                let popup = div()
                    .id(("pp", uid))
                    .absolute()
                    .top(px(17.0))
                    .right_0()
                    .w(px(230.0))
                    .p(px(t.spacing.md))
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(hsla(t.palette.border_strong))
                    .bg(hsla(t.palette.surface_raised))
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.xs))
                    .on_hover(cx.listener(move |s, h: &bool, _, cx| {
                        if *h {
                            s.pop_hover = Some((title, k));
                        } else if s.pop_hover == Some((title, k)) {
                            s.pop_hover = None;
                        }
                        cx.notify();
                    }))
                    .child(Muted::new(format!("{} duration", k.label())))
                    .child(
                        Slider::new(("pd", uid), ms as f32)
                            .range(0.0, 2000.0)
                            .step(50.0)
                            .show_value(true)
                            .format(|v| {
                                if v < 1.0 {
                                    "default".to_string()
                                } else {
                                    format!("{v:.0} ms")
                                }
                            })
                            .on_change(move |v, _, cx| {
                                me_p.update(cx, |s, cx| {
                                    let cur =
                                        s.sec_kind.get(title).copied().unwrap_or(EntrySet::NONE);
                                    s.sec_kind.insert(title, cur.with_ms(k, v as u16));
                                    cx.notify();
                                })
                            }),
                    );
                wrap = wrap.child(gpui::deferred(popup).with_priority(60));
            }
            chips = chips.child(wrap);
        }
        chips.into_any_element()
    }
}

fn cat_icon(name: &str) -> dodai_gpui::IconName {
    use dodai_gpui::IconName as I;
    match name {
        "All" => I::Grid,
        "Foundations" => I::Layers,
        "Inputs" => I::Sliders,
        "Actions" => I::Play,
        "Navigation" => I::Menu,
        "Data display" => I::Table,
        "Charts" => I::BarChart,
        "Animation" => I::Sparkle,
        _ => I::Dot,
    }
}

fn easings_of(i: usize) -> Easing {
    [
        Easing::Linear,
        Easing::EaseIn,
        Easing::EaseOut,
        Easing::EaseInOut,
        Easing::CubicBezier(0.2, 0.8, 0.2, 1.0),
        Easing::CubicBezier(0.5, -0.4, 0.3, 1.4),
    ][i.min(5)]
}

impl Render for Showcase {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let narrow = window.viewport_size().width < px(760.0);
        let t = theme_of(cx);
        let me = cx.entity();
        let gap = px(t.spacing.md);
        let glow = glow_intensity(cx);
        let scan = scanlines_enabled(cx);

        let me_tab = me.clone();
        let me_sw = me.clone();
        let me_ck = me.clone();
        let me_sl = me.clone();

        macro_rules! section {
            ($t:expr) => {
                Panel::new()
                    .title($t)
                    .subheader(self.chips($t, &t, &me, cx))
            };
        }

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
        let ds_panel = section!("Design system")
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
            .child(Sparkline::new([3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 12.0]))
            .child(
                Sparkline::new([3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 12.0]).marker(Marker::Diamond),
            )
            .child(
                Sparkline::new([3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 12.0])
                    .marker(Marker::Square)
                    .area(false),
            )
            .child(
                Sparkline::new([3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 12.0])
                    .marker(Marker::Triangle)
                    .all_points(true),
            );
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
        let (mb1, mb2) = (me.clone(), me.clone());
        let stepped = col()
            .child(
                Slider::new("bub-up", self.bub.0)
                    .range(0.0, 100.0)
                    .bubble(BubblePlacement::Above)
                    .format(|v| format!("{v:.0}%"))
                    .on_change(move |v, _, cx| {
                        mb1.update(cx, |s, cx| {
                            s.bub.0 = v;
                            cx.notify()
                        })
                    }),
            )
            .child(
                Slider::new("bub-down", self.bub.1)
                    .range(0.0, 100.0)
                    .step(10.0)
                    .ticks(true)
                    .bubble(BubblePlacement::Below)
                    .format(|v| format!("{v:.0}%"))
                    .on_change(move |v, _, cx| {
                        mb2.update(cx, |s, cx| {
                            s.bub.1 = v;
                            cx.notify()
                        })
                    }),
            )
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

        let w = |e: gpui::AnyElement, px_w: f32| div().w(px(px_w)).child(e).into_any_element();
        let charts_more1 = row()
            .child(w(
                StackedAreaChart::new([
                    Series::new("Web", [12.0, 18.0, 14.0, 26.0, 22.0, 31.0]),
                    Series::new("App", [8.0, 11.0, 13.0, 15.0, 17.0, 16.0]),
                    Series::new("API", [4.0, 6.0, 9.0, 8.0, 12.0, 15.0]),
                ])
                .labels(months)
                .title("Traffic by source")
                .into_any_element(),
                420.0,
            ))
            .child(w(
                Histogram::new(
                    "hist",
                    (0..120).map(|i| ((i * 37 % 100) as f32 / 10.0 + (i % 7) as f32).min(18.0)),
                )
                .bins(9)
                .title("Latency distribution")
                .format(|v| format!("{v:.0}ms"))
                .into_any_element(),
                420.0,
            ))
            .child(w(
                Waterfall::new(
                    "wf",
                    [
                        ("Start", 40.0),
                        ("Sales", 25.0),
                        ("Refunds", -12.0),
                        ("Ads", -8.0),
                        ("Upsell", 15.0),
                    ],
                )
                .title("Revenue bridge")
                .format(|v| format!("${v:.0}k"))
                .into_any_element(),
                420.0,
            ));
        let charts_more2 = row()
            .child(w(
                Funnel::new([
                    ("Visits", 1000.0),
                    ("Signups", 620.0),
                    ("Trials", 340.0),
                    ("Paid", 120.0),
                ])
                .title("Conversion funnel")
                .into_any_element(),
                380.0,
            ))
            .child(w(
                Treemap::new(
                    "tm",
                    [
                        ("Rust", 52.0),
                        ("Kotlin", 28.0),
                        ("TS", 18.0),
                        ("Go", 12.0),
                        ("C", 9.0),
                        ("Other", 5.0),
                    ],
                )
                .title("Code by language")
                .into_any_element(),
                380.0,
            ))
            .child(
                RadialBar::new([
                    ("CPU", 72.0, 100.0),
                    ("RAM", 48.0, 100.0),
                    ("Disk", 31.0, 100.0),
                ])
                .into_any_element(),
            );
        let charts_more3 = row()
            .child(w(
                Candlestick::new(
                    "ohlc",
                    [
                        Candle::new(10.0, 14.0, 9.0, 13.0),
                        Candle::new(13.0, 15.0, 11.0, 12.0),
                        Candle::new(12.0, 16.0, 11.5, 15.5),
                        Candle::new(15.5, 17.0, 13.0, 14.0),
                        Candle::new(14.0, 18.0, 13.5, 17.5),
                        Candle::new(17.5, 19.0, 15.0, 16.0),
                    ],
                )
                .labels(months)
                .title("OHLC")
                .into_any_element(),
                380.0,
            ))
            .child(w(
                BoxPlot::new([
                    ("A", vec![2.0, 3.0, 3.5, 4.0, 5.0, 6.0, 9.0]),
                    ("B", vec![4.0, 5.0, 6.0, 6.5, 7.0, 8.0, 12.0]),
                    ("C", vec![1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5]),
                ])
                .title("Box plot")
                .into_any_element(),
                380.0,
            ))
            .child(w(
                BubbleChart::new(
                    "bub",
                    [
                        (
                            "EU",
                            vec![(1.0, 2.0, 30.0), (3.0, 5.0, 80.0), (5.0, 3.0, 50.0)],
                        ),
                        (
                            "US",
                            vec![(2.0, 6.0, 100.0), (4.0, 2.0, 20.0), (6.0, 7.0, 60.0)],
                        ),
                    ],
                )
                .title("Bubble chart")
                .into_any_element(),
                380.0,
            ));

        let easings: [(&str, Easing); 6] = [
            ("Linear", Easing::Linear),
            ("Ease in", Easing::EaseIn),
            ("Ease out", Easing::EaseOut),
            ("Ease in-out", Easing::EaseInOut),
            (
                "Snappy · cubic-bezier(.2,.8,.2,1)",
                Easing::CubicBezier(0.2, 0.8, 0.2, 1.0),
            ),
            (
                "Overshoot · cubic-bezier(.5,-.4,.3,1.4)",
                Easing::CubicBezier(0.5, -0.4, 0.3, 1.4),
            ),
        ];
        let kind_labels: Vec<&'static str> = EntryKind::ALL.iter().map(|k| k.label()).collect();
        let (me_k, me_e, me_d, me_r) = (me.clone(), me.clone(), me.clone(), me.clone());
        let apply = |s: &Showcase, cx: &mut gpui::App| {
            Motion::new(
                Duration::from_millis(s.anim_ms as u64),
                easings_of(s.anim_ease),
            )
            .entry(EntryKind::ALL[s.anim_kind])
            .set_global(cx);
        };
        let anim_settings = section!("Animation settings")
            .child(Muted::new(
                "Applies to every component: entry style, easing and duration. Charts, sliders, progress, knobs and gauges also animate their values in.",
            ))
            .child(
                row()
                    .child(Label::new("Entry"))
                    .child(
                        div().w(px(200.0)).child(
                            Select::new("anim-kind")
                                .options(kind_labels.clone())
                                .selected(Some(self.anim_kind))
                                .on_change(move |i, _, cx| {
                                    me_k.update(cx, |s, cx| {
                                        s.anim_kind = i;
                                        apply(s, cx);
                                        s.replay += 1;
                                        cx.notify();
                                    })
                                }),
                        ),
                    )
                    .child(Label::new("Easing"))
                    .child(
                        div().w(px(260.0)).child(
                            Select::new("anim-ease")
                                .options(easings.iter().map(|e| e.0).collect::<Vec<_>>())
                                .selected(Some(self.anim_ease))
                                .on_change(move |i, _, cx| {
                                    me_e.update(cx, |s, cx| {
                                        s.anim_ease = i;
                                        apply(s, cx);
                                        s.replay += 1;
                                        cx.notify();
                                    })
                                }),
                        ),
                    ),
            )
            .child(
                row()
                    .child(Label::new("Duration"))
                    .child(
                        div().w(px(260.0)).child(
                            Slider::new("anim-ms", self.anim_ms)
                                .range(100.0, 1500.0)
                                .step(50.0)
                                .show_value(true)
                                .format(|v| format!("{v:.0} ms"))
                                .on_change(move |v, _, cx| {
                                    me_d.update(cx, |s, cx| {
                                        s.anim_ms = v;
                                        apply(s, cx);
                                        cx.notify();
                                    })
                                }),
                        ),
                    )
                    .child(Button::new("anim-replay", "Replay").on_click(cx.listener(
                        |s, _, _, cx| {
                            s.replay += 1;
                            cx.notify();
                        },
                    ))),
            );
        let _ = me_r;
        let mut curves = col();
        for (i, (name, e)) in easings.iter().enumerate() {
            let e = *e;
            curves = curves.child(
                row()
                    .child(div().w(px(300.0)).child(Mono::new(*name)))
                    .child(
                        div()
                            .relative()
                            .w(px(260.0))
                            .h(px(14.0))
                            .child(
                                div()
                                    .absolute()
                                    .top(px(6.0))
                                    .w_full()
                                    .h(px(2.0))
                                    .bg(hsla(t.palette.border)),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .top_0()
                                    .size(px(14.0))
                                    .rounded_full()
                                    .bg(hsla(t.palette.primary))
                                    .with_animation(
                                        ("ease-dot", i),
                                        Animation::new(Duration::from_millis(1600))
                                            .repeat()
                                            .with_easing(move |p| e.apply(p).clamp(0.0, 1.0)),
                                        |d, p| d.left(px(p * 246.0)),
                                    ),
                            ),
                    ),
            );
        }
        let anim_curves = section!("Easing curves")
            .child(Muted::new(
                "Each dot loops with its curve (1.6 s). Custom curves: Easing::CubicBezier.",
            ))
            .child(curves);
        let anim_loading = section!("Loading · looping")
            .child(Muted::new("Spinner 900 ms · IndeterminateBar 1.4 s · Skeleton 1.6 s (linear). Override with .motion(Motion)."))
            .child(
                row()
                    .child(Spinner::new("an-sp"))
                    .child(Spinner::new("an-sp2").motion(Motion::new(Duration::from_millis(500), Easing::EaseInOut)))
                    .child(div().w(px(200.0)).child(IndeterminateBar::new("an-ib")))
                    .child(div().w(px(200.0)).child(Skeleton::new("an-sk"))),
            );
        let anim_docs = section!("What animates")
            .child(Label::new("Entry — every component"))
            .child(Muted::new("Fade, Rise, Drop, Slide from left/right or None; set globally above or per section with the dropdown on each section header, where you can add and remove effects to compose them (Fade + Rise + Slide ...). The set overrides every component in that section; Global removes the override and no chips selected means no animation."))
            .child(Label::new("Value-in — progress, slider, range slider, knob, gauge, bar, line, pie (sweep), radial bar, histogram"))
            .child(Muted::new("Values grow from zero to their target using the same duration and easing."))
            .child(Label::new("Pie / donut — hover pop and selection"))
            .child(Muted::new("Slices ease out 6 px on hover (Tween), selected slice keeps a half pop and an accent outline (selected_color)."))
            .child(Label::new("Composable — Curve"))
            .child(Muted::new("Curve::from(easing).window(a, b) / reverse / ping_pong / then / pipe / stagger(i, n, overlap)."));
        let sections: Vec<(&'static str, &'static str, gpui::AnyElement, &'static str)> = vec![
            (
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
            ),
            (
                "Actions",
                "Buttons · IconButton · FAB",
                section!("Buttons").child(buttons).into_any_element(),
                "Buttons",
            ),
            (
                "Actions",
                "Toolbar · Snackbar",
                section!("Toolbar · Snackbar")
                    .child(w_toolbar)
                    .child(w_snack)
                    .into_any_element(),
                "Toolbar · Snackbar",
            ),
            (
                "Data display",
                "Tags · Chips · Badges",
                section!("Tags").child(tags).into_any_element(),
                "Tags",
            ),
            (
                "Data display",
                "Badges · Rating · Tooltips",
                section!("Rating · Stepper · Badges · Tooltips")
                    .child(w_misc)
                    .into_any_element(),
                "Rating · Stepper · Badges · Tooltips",
            ),
            (
                "Data display",
                "List · Divider",
                section!("List · Divider")
                    .child(list)
                    .child(Divider::new())
                    .into_any_element(),
                "List · Divider",
            ),
            (
                "Data display",
                "Tree · Timeline · Empty state",
                section!("Tree · timeline · empty state")
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
                "Tree · timeline · empty state",
            ),
            (
                "Data display",
                "Tooltip · Rich tooltip",
                section!("Tooltip")
                    .child(
                        div()
                            .id("tip")
                            .w(px(220.0))
                            .text_color(hsla(t.palette.link))
                            .tooltip(DodaiTooltip::text("Themed tooltip"))
                            .child("Hover me"),
                    )
                    .into_any_element(),
                "Tooltip",
            ),
            (
                "Inputs",
                "Toggle · Checkbox · Radio",
                section!("Toggle · Checkbox · Radio")
                    .child(controls)
                    .child(radios)
                    .into_any_element(),
                "Toggle · Checkbox · Radio",
            ),
            (
                "Inputs",
                "Slider · Progress",
                section!("Slider · Progress")
                    .child(sliders)
                    .into_any_element(),
                "Slider · Progress",
            ),
            (
                "Inputs",
                "Stepped slider · ticks · formatted value",
                section!("Slider (stepped, ticks, value readout)")
                    .child(stepped)
                    .into_any_element(),
                "Slider (stepped, ticks, value readout)",
            ),
            (
                "Inputs",
                "Range slider",
                section!("Range slider")
                    .child(w_range)
                    .child(w_range_txt)
                    .into_any_element(),
                "Range slider",
            ),
            (
                "Inputs",
                "Knob · Vertical slider · Vertical range slider",
                section!("Knob · Vertical sliders")
                    .child(dials)
                    .into_any_element(),
                "Knob · Vertical sliders",
            ),
            (
                "Inputs",
                "Input field",
                section!("Input")
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
                "Input",
            ),
            (
                "Inputs",
                "Search view · SearchBar",
                section!("Search view").child(w_search).into_any_element(),
                "Search view",
            ),
            (
                "Inputs",
                "Tag field · chips",
                section!("Tag field").child(w_tags).into_any_element(),
                "Tag field",
            ),
            (
                "Inputs",
                "Date picker · Time picker",
                section!("Date / time pickers")
                    .child(pickers)
                    .into_any_element(),
                "Date / time pickers",
            ),
            (
                "Navigation",
                "Tabs",
                section!("Tabs")
                    .child(tabs)
                    .child(Muted::new(format!("active tab: {}", self.tab)))
                    .into_any_element(),
                "Tabs",
            ),
            (
                "Navigation",
                "App bar · Cards · FAB · Navigation",
                section!("App bar · Cards · FAB · Navigation")
                    .child(structure)
                    .into_any_element(),
                "App bar · Cards · FAB · Navigation",
            ),
            (
                "Charts",
                "Line · Bar · Area · Stacked charts",
                section!("Line & bar charts")
                    .child(charts_xy)
                    .into_any_element(),
                "Line & bar charts",
            ),
            (
                "Charts",
                "Pie · Donut · Gauge · Sparkline",
                section!("Pie · donut · gauge · sparkline")
                    .child(charts_round)
                    .into_any_element(),
                "Pie · donut · gauge · sparkline",
            ),
            (
                "Charts",
                "Stacked area · Histogram · Waterfall",
                section!("Area · histogram · waterfall")
                    .child(charts_more1)
                    .into_any_element(),
                "Area · histogram · waterfall",
            ),
            (
                "Charts",
                "Funnel · Treemap · Radial bar",
                section!("Funnel · treemap · radial bar")
                    .child(charts_more2)
                    .into_any_element(),
                "Funnel · treemap · radial bar",
            ),
            (
                "Charts",
                "Candlestick · Box plot · Bubble",
                section!("Candlestick · box plot · bubble")
                    .child(charts_more3)
                    .into_any_element(),
                "Candlestick · box plot · bubble",
            ),
            (
                "Charts",
                "Radar · Scatter · Heatmap",
                section!("Radar · scatter · heatmap")
                    .child(charts_misc)
                    .into_any_element(),
                "Radar · scatter · heatmap",
            ),
            (
                "Animation",
                "Settings",
                anim_settings.into_any_element(),
                "Animation settings",
            ),
            (
                "Animation",
                "Easing curves",
                anim_curves.into_any_element(),
                "Easing curves",
            ),
            (
                "Animation",
                "Loading",
                anim_loading.into_any_element(),
                "Loading · looping",
            ),
            (
                "Animation",
                "What animates",
                anim_docs.into_any_element(),
                "What animates",
            ),
        ];

        let cats: Vec<&'static str> = {
            let mut v: Vec<&'static str> = Vec::new();
            for (c, _, _, _) in &sections {
                if !v.contains(c) {
                    v.push(c);
                }
            }
            v
        };
        self.cat_count = cats.len() + 1;
        let query = self.filter.read(cx).text().to_string();
        let selected_cat = self.cat.checked_sub(1).and_then(|i| cats.get(i).copied());

        // Sidebar: filter + category list with counts.
        let count_in = |c: Option<&'static str>| {
            sections
                .iter()
                .filter(|(sc, title, _, _)| {
                    c.is_none_or(|c| c == *sc)
                        && (query.is_empty()
                            || !fuzzy_filter(&query, &[format!("{sc} {title}")]).is_empty())
                })
                .count()
        };
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
                cx.notify();
            }))
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
                    .icon(cat_icon(name))
                    .selected(self.cat == idx)
                    .on_click(move |_, w, cx| {
                        m.update(cx, |s, cx| {
                            w.focus(&s.side_focus);
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
            .w_full()
            .min_w_0()
            .p(px(if narrow { t.spacing.md } else { t.spacing.xl }))
            .child(ds_panel)
            .child(theme_panel);
        let mut shown = 0;
        let mut last_cat = "";
        for (c, title, el, key) in sections {
            let in_cat = selected_cat.is_none_or(|sc| sc == c);
            let hit =
                query.is_empty() || !fuzzy_filter(&query, &[format!("{c} {title}")]).is_empty();
            let only = std::env::var("DODAI_ONLY").is_ok_and(|o| !title.contains(&o));
            if !(in_cat && hit) || only {
                continue;
            }
            if c != last_cat {
                body = body.child(Heading::new(c).level(2));
                last_cat = c;
            }
            shown += 1;
            let kind = self.sec_kind.get(key).copied();
            let content = match kind {
                None => el,
                Some(set) => dodai_gpui::EntryScope::new(set, el).into_any_element(),
            };
            body = body.child(div().flex().flex_col().w_full().min_w_0().child(content));
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
            .child(
                div()
                    .id("side")
                    .flex_none()
                    .h_full()
                    .overflow_y_scroll()
                    .when(narrow, |d| d.max_w(px(170.0)))
                    .child(side),
            )
            .child(
                div()
                    .id("scroll")
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .overflow_y_scroll()
                    .child(body.id(("body", self.cat * 1000 + self.replay))),
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
                        cat: std::env::var("DODAI_CAT")
                            .ok()
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(0),
                        anim_kind: 1,
                        anim_ease: 2,
                        anim_ms: 220.0,
                        replay: 0,
                        sec_kind: Default::default(),
                        pill_hover: None,
                        pop_hover: None,
                        cat_count: 1,
                        side_focus: cx.focus_handle(),
                        rating: 3,
                        stepper: 5,
                        range: (0.2, 0.7),
                        vrange: (0.2, 0.8),
                        stepped: 4.0,
                        db: -6.0,
                        bub: (35.0, 60.0),
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
