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
use dodai_gpui::{Easing, EntryKind, EntrySet, IndeterminateBar, Motion, Skeleton, Spinner};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Entity, FocusHandle,
    KeyDownEvent, Window, WindowBounds, WindowOptions,
};
use gpui::{AnimationExt, SharedString};
use std::time::Duration;

struct Showcase {
    anim_ms: f32,
    replay: usize,
    sec_replay: std::collections::HashMap<&'static str, u32>,
    sec_mode: std::collections::HashMap<&'static str, Player>,
    loop_gen: u32,
    ld_pb: dodai_gpui::Playback,
    ld_ms: u16,
    scroll: gpui::ScrollHandle,
    sec_kind: std::collections::HashMap<&'static str, EntrySet>,
    pill_hover: Option<(&'static str, u8)>,
    pop_hover: Option<(&'static str, u8)>,
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

/// Per-section animation player state.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Player {
    /// Animate when the section appears or is replayed.
    #[default]
    Idle,
    /// Replay repeatedly.
    Looping,
    /// Show the final state, no entry animation.
    Stopped,
}

fn section_id(title: &str) -> u32 {
    title
        .bytes()
        .fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32))
}

impl Showcase {
    /// Entry set applied to a section's components (`None` = app default).
    fn scope_of(&self, title: &'static str, cx: &gpui::App) -> EntrySet {
        if self.sec_mode.get(title) == Some(&Player::Stopped) {
            return EntrySet::NONE;
        }
        self.sec_kind
            .get(title)
            .copied()
            .unwrap_or(Motion::of(cx).entry)
    }

    /// Replay one section only (re-keys just its subtree).
    fn bump(&mut self, title: &'static str) {
        *self.sec_replay.entry(title).or_insert(0) += 1;
    }

    fn set_mode(&mut self, title: &'static str, mode: Player, cx: &mut Context<Self>) {
        self.sec_mode.insert(title, mode);
        self.loop_gen += 1;
        self.bump(title);
        if mode == Player::Looping {
            let gen = self.loop_gen;
            cx.spawn(async move |this, cx| loop {
                let wait = this
                    .update(cx, |s, _| {
                        let set = s.sec_kind.get(title).copied().unwrap_or_default();
                        set.duration(Duration::from_millis(s.anim_ms as u64))
                            + Duration::from_millis(700)
                    })
                    .unwrap_or_default();
                cx.background_executor().timer(wait).await;
                let alive = this
                    .update(cx, |s, cx| {
                        let ok =
                            s.sec_mode.get(title) == Some(&Player::Looping) && s.loop_gen == gen;
                        if ok {
                            s.bump(title);
                            cx.notify();
                        }
                        ok
                    })
                    .unwrap_or(false);
                if !alive {
                    break;
                }
            })
            .detach();
        }
        cx.notify();
    }

    /// Per-section animation chips (hover a chip for its duration popover).
    fn chips(
        &self,
        title: &'static str,
        t: &Theme,
        me: &gpui::Entity<Showcase>,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let tid = section_id(title);
        let base = Motion::of(cx).entry;
        let kind = Some(self.sec_kind.get(title).copied().unwrap_or(base));
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
        for (ki, k) in EntryKind::ALL.iter().copied().enumerate().skip(1) {
            let on = kind.is_some_and(|s| s.contains(k));
            let ms = kind.map_or(0, |s| s.ms_of(k));
            let open = on
                && (self.pill_hover == Some((title, k as u8))
                    || self.pop_hover == Some((title, k as u8)));
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
                        s.pill_hover = Some((title, k as u8));
                    } else if s.pill_hover == Some((title, k as u8)) {
                        s.pill_hover = None;
                    }
                    cx.notify();
                }))
                .child(chip(("sec-k", uid), text, on).on_click(move |_, _, cx| {
                    me_c.update(cx, |s, cx| {
                        let cur = s.sec_kind.get(title).copied().unwrap_or(base);
                        s.sec_kind.insert(title, cur.toggled(k));
                        s.bump(title);
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
                            s.pop_hover = Some((title, k as u8));
                        } else if s.pop_hover == Some((title, k as u8)) {
                            s.pop_hover = None;
                        }
                        cx.notify();
                    }))
                    .child(Muted::new(format!("{} duration", k.label())))
                    .child(
                        Slider::new(("pd", uid), ms as f32)
                            .range(0.0, 10000.0)
                            .step(100.0)
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
                                    let cur = s.sec_kind.get(title).copied().unwrap_or(base);
                                    s.sec_kind.insert(title, cur.with_ms(k, v as u16));
                                    cx.notify();
                                })
                            }),
                    );
                wrap = wrap.child(gpui::deferred(popup).with_priority(60));
            }
            chips = chips.child(wrap);
        }
        // Fill (value-in duration) and Stagger (delay between components).
        for (tag, name, help, slot, vmax, vstep) in [
            (
                100u8,
                "Fill",
                "Fill duration (charts, progress, sliders)",
                0u32,
                10000.0,
                100.0,
            ),
            (
                101u8,
                "Stagger",
                "Delay between components in this section",
                6u32,
                1000.0,
                25.0,
            ),
        ] {
            let gms = kind.map_or(0, |s| {
                if slot == 0 {
                    s.grow_ms()
                } else {
                    s.stagger_ms()
                }
            });
            let key = (title, tag);
            let open = self.pill_hover == Some(key) || self.pop_hover == Some(key);
            let uid = tid.wrapping_mul(8).wrapping_add(slot);
            let me_p = me.clone();
            let text: SharedString = if gms > 0 {
                format!("{name} · {gms}ms").into()
            } else {
                name.into()
            };
            let mut wrap = div()
                .id(("pw", uid))
                .relative()
                .on_hover(cx.listener(move |s, h: &bool, _, cx| {
                    if *h {
                        s.pill_hover = Some(key);
                    } else if s.pill_hover == Some(key) {
                        s.pill_hover = None;
                    }
                    cx.notify();
                }))
                .child(chip(("sec-f", uid), text, gms > 0));
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
                            s.pop_hover = Some(key);
                        } else if s.pop_hover == Some(key) {
                            s.pop_hover = None;
                        }
                        cx.notify();
                    }))
                    .child(Muted::new(help))
                    .child(
                        Slider::new(("pd", uid), gms as f32)
                            .range(0.0, vmax)
                            .step(vstep)
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
                                    let cur = s.sec_kind.get(title).copied().unwrap_or(base);
                                    let nw = if slot == 0 {
                                        cur.with_grow_ms(v as u16)
                                    } else {
                                        cur.with_stagger_ms(v as u16)
                                    };
                                    s.sec_kind.insert(title, nw);
                                    cx.notify();
                                })
                            }),
                    );
                wrap = wrap.child(gpui::deferred(popup).with_priority(60));
            }
            chips = chips.child(wrap);
        }
        // Player: play once / loop / stop.
        let mode = self.sec_mode.get(title).copied().unwrap_or_default();
        let btn = |id: u32, icon: dodai_gpui::IconName, on: bool| {
            let fg = hsla(if on {
                t.palette.on_primary
            } else {
                t.palette.text_muted
            });
            div()
                .id(("pl", tid.wrapping_mul(4).wrapping_add(id)))
                .px(px(5.0))
                .py(px(2.0))
                .rounded(px(4.0))
                .cursor_pointer()
                .border_1()
                .border_color(hsla(t.palette.border))
                .when(on, |d| d.bg(hsla(t.palette.primary)))
                .child(dodai_gpui::Icon::new(icon).size(11.0).color(fg))
        };
        let (m1, m2, m3) = (me.clone(), me.clone(), me.clone());
        chips = chips
            .child(div().w(px(8.0)))
            .child(
                btn(1, dodai_gpui::IconName::Play, false).on_click(move |_, _, cx| {
                    m1.update(cx, |s, cx| s.set_mode(title, Player::Idle, cx))
                }),
            )
            .child(
                btn(2, dodai_gpui::IconName::Repeat, mode == Player::Looping).on_click(
                    move |_, _, cx| m2.update(cx, |s, cx| s.set_mode(title, Player::Looping, cx)),
                ),
            )
            .child(
                btn(3, dodai_gpui::IconName::Stop, mode == Player::Stopped).on_click(
                    move |_, _, cx| m3.update(cx, |s, cx| s.set_mode(title, Player::Stopped, cx)),
                ),
            );
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
        "Loading" => I::Repeat,
        _ => I::Dot,
    }
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
                    .scope(Some(self.scope_of($t, cx)))
                    .scope_key(self.sec_replay.get($t).copied().unwrap_or(0))
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
        let ds_panel = Panel::new()
            .focused(true)
            .child(
                row()
                    .child(Label::new("Design system"))
                    .child(DesignSystemSelector::new("ds-select")),
            )
            .child(
                row()
                    .child(Label::new(format!("Theme · {family}")))
                    .child(ThemeSelector::new("theme-select")),
            )
            .child(
                row().child(Label::new("Glow intensity")).child(
                    div().w(px(220.0)).child(
                        Slider::new("glow-slider", glow)
                            .range(0.0, 2.0)
                            .step(0.05)
                            .show_value(true)
                            .format(|v| format!("{v:.2}"))
                            .on_change(move |v, _, cx| set_glow_intensity(cx, v)),
                    ),
                ),
            )
            .child(
                row()
                    .child(Label::new("Scanlines"))
                    .child(Toggle::new("scan", scan).on_change(move |on, _, cx| {
                        set_scanlines(cx, on);
                    }))
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
                    (0..24)
                        .map(|r| {
                            (0..64)
                                .map(|c| {
                                    let (x, y) = (c as f32, r as f32);
                                    ((x * 0.21).sin()
                                        + (y * 0.45).cos()
                                        + ((x + y) * 0.13).sin() * 1.5)
                                        + ((x - 30.0).hypot((y - 12.0) * 2.0) * -0.08).exp() * 3.0
                                })
                                .collect()
                        })
                        .collect(),
                )
                .cell(11.0)
                .row_labels((0..24).map(|r| {
                    if r % 4 == 0 {
                        format!("{r:02}h")
                    } else {
                        String::new()
                    }
                }))
                .col_labels((0..64).map(|c| {
                    if c % 8 == 0 {
                        format!("{c}")
                    } else {
                        String::new()
                    }
                })),
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

        let (pb, lms) = (self.ld_pb, self.ld_ms);
        let run = self.sec_replay.get("Loading").copied().unwrap_or(0);
        // Duration override (0 = each indicator's own default).
        let mo = move |ms: u64, ease: Easing| {
            Motion::new(
                Duration::from_millis(if lms > 0 { lms as u64 } else { ms }),
                if lms > 0 { Easing::Linear } else { ease },
            )
        };
        let lbtn = |id: u32, icon: dodai_gpui::IconName, on: bool| {
            let fg = hsla(if on {
                t.palette.on_primary
            } else {
                t.palette.text_muted
            });
            div()
                .id(("ldb", id))
                .px(px(5.0))
                .py(px(2.0))
                .rounded(px(4.0))
                .cursor_pointer()
                .border_1()
                .border_color(hsla(t.palette.border))
                .when(on, |d| d.bg(hsla(t.palette.primary)))
                .child(dodai_gpui::Icon::new(icon).size(11.0).color(fg))
        };
        let (l1, l2, l3, l4) = (me.clone(), me.clone(), me.clone(), me.clone());
        let ld_controls = div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(
                lbtn(
                    1,
                    dodai_gpui::IconName::Play,
                    pb == dodai_gpui::Playback::Once,
                )
                .on_click(move |_, _, cx| {
                    l1.update(cx, |s, cx| {
                        s.ld_pb = dodai_gpui::Playback::Once;
                        s.bump("Loading");
                        cx.notify()
                    })
                }),
            )
            .child(
                lbtn(
                    2,
                    dodai_gpui::IconName::Repeat,
                    pb == dodai_gpui::Playback::Loop,
                )
                .on_click(move |_, _, cx| {
                    l2.update(cx, |s, cx| {
                        s.ld_pb = dodai_gpui::Playback::Loop;
                        s.bump("Loading");
                        cx.notify()
                    })
                }),
            )
            .child(
                lbtn(
                    3,
                    dodai_gpui::IconName::Stop,
                    pb == dodai_gpui::Playback::Stopped,
                )
                .on_click(move |_, _, cx| {
                    l3.update(cx, |s, cx| {
                        s.ld_pb = dodai_gpui::Playback::Stopped;
                        s.bump("Loading");
                        cx.notify()
                    })
                }),
            )
            .child(div().w(px(10.0)))
            .child(Muted::new("Duration per run"))
            .child(
                div().w(px(220.0)).child(
                    Slider::new("ld-dur", lms as f32)
                        .range(0.0, 10000.0)
                        .step(100.0)
                        .show_value(true)
                        .format(|v| {
                            if v < 1.0 {
                                "default".to_string()
                            } else {
                                format!("{v:.0} ms")
                            }
                        })
                        .on_change(move |v, _, cx| {
                            l4.update(cx, |s, cx| {
                                s.ld_ms = v as u16;
                                s.bump("Loading");
                                cx.notify()
                            })
                        }),
                ),
            );
        let pbk = pb;
        let loading = Panel::new()
            .title("Loading")
            .subheader(ld_controls)
            .child(Label::new("Spinner"))
            .child(
                row()
                    .child(Spinner::new(("ld-sp1", run)).motion(mo(900, Easing::Linear)).playback(pbk))
                    .child(Spinner::new(("ld-sp2", run)).motion(mo(450, Easing::Linear)).playback(pbk))
                    .child(
                        Spinner::new(("ld-sp3", run))
                            .motion(mo(1800, Easing::EaseInOut))
                            .playback(pbk),
                    )
                    .child(Muted::new("900 ms · 450 ms · 1.8 s ease-in-out (unless a duration is set)")),
            )
            .child(Label::new("Indeterminate bar"))
            .child(IndeterminateBar::new(("ld-ib1", run)).motion(mo(1400, Easing::Linear)).playback(pbk))
            .child(IndeterminateBar::new(("ld-ib2", run)).motion(mo(700, Easing::EaseInOut)).playback(pbk))
            .child(Label::new("Determinate progress"))
            .child(ProgressBar::new(0.35))
            .child(ProgressBar::new(0.7).tone(Tone::Success))
            .child(Label::new("Skeleton placeholders"))
            .child(
                row()
                    .items_start()
                    .gap(px(16.0))
                    .child(
                        Skeleton::new(("ld-av", run))
                            .width(48.0)
                            .height(48.0)
                            .circle(true)
                            .motion(mo(1600, Easing::Linear))
                            .playback(pbk),
                    )
                    .child(
                        col()
                            .flex_1()
                            .child(Skeleton::new(("ld-l1", run)).height(14.0).motion(mo(1600, Easing::Linear)).playback(pbk))
                            .child(Skeleton::new(("ld-l2", run)).height(14.0).width(260.0).motion(mo(1600, Easing::Linear)).playback(pbk))
                            .child(Skeleton::new(("ld-l3", run)).height(14.0).width(180.0).motion(mo(1600, Easing::Linear)).playback(pbk)),
                    ),
            )
            .child(Muted::new(
                "Play runs one cycle, Loop repeats, Stop holds a still frame. Loading demos live on this page only (not on All) to keep idle CPU at zero.",
            ));

        // Animation page: one Panel per entry type (curve · live example · text).
        let anim_defs: [(&'static str, EntryKind, Easing, &'static str); 6] = [
            (
                EntryKind::Fade.label(),
                EntryKind::Fade,
                Easing::EaseOut,
                "Opacity 0 → 1. The default entry style: quiet, never shifts layout. Ease-out starts fast and settles.",
            ),
            (
                EntryKind::Rise.label(),
                EntryKind::Rise,
                Easing::CubicBezier(0.2, 0.8, 0.2, 1.0),
                "Fades in while moving up 14 px. A snappy cubic-bezier(.2,.8,.2,1) gives a quick start and a soft landing.",
            ),
            (
                EntryKind::Drop.label(),
                EntryKind::Drop,
                Easing::EaseInOut,
                "Fades in while dropping down 14 px. Ease-in-out accelerates, then decelerates; good for menus and toasts.",
            ),
            (
                EntryKind::SlideLeft.label(),
                EntryKind::SlideLeft,
                Easing::CubicBezier(0.7, 0.0, 0.3, 1.0),
                "Fades in sliding from the left. A strong ease-in-out (.7,0,.3,1) feels deliberate; fits sidebars and drawers.",
            ),
            (
                EntryKind::SlideRight.label(),
                EntryKind::SlideRight,
                Easing::EaseIn,
                "Fades in sliding from the right. Ease-in starts slowly and finishes fast; use for trailing panels.",
            ),
            (
                "Spring",
                EntryKind::Rise,
                Easing::SPRING,
                "Physics-based easing: a damped spring (stiffness 170, damping 12). Overshoots its target and settles; entry effects clamp at 1.0, value animations (charts, sliders) show the overshoot.",
            ),
        ];
        let anim_titles: Vec<&'static str> = anim_defs.iter().map(|d| d.0).collect();
        let mut anim_sections = Vec::new();
        for (i, (title, kind, ease, text)) in anim_defs.into_iter().enumerate() {
            let n = self.sec_replay.get(title).copied().unwrap_or(0);
            let me_r = me.clone();
            let ctrl = ease_ctrl(ease);
            let (x1, y1, x2, y2) = ctrl.unwrap_or((0.0, 0.0, 1.0, 1.0));
            let ymax = if ctrl.is_none() { 1.4 } else { 1.0 };
            let (border, accent, muted) = (
                hsla(t.palette.border),
                hsla(t.palette.primary),
                hsla(t.palette.text_muted),
            );
            let make_plot = move |p: f32| {
                gpui::canvas(
                    |_, _, _| (),
                    move |b: Bounds<gpui::Pixels>, _, w, _| {
                        let pad = 10.0;
                        let side = f32::from(b.size.width.min(b.size.height)) - 2.0 * pad;
                        let at = |x: f32, y: f32| {
                            gpui::point(
                                b.origin.x + px(pad + x * side),
                                b.origin.y + px(pad + (1.0 - y / ymax) * side),
                            )
                        };
                        let line =
                            |pts: &[(f32, f32)], width: f32, c: gpui::Hsla, w: &mut Window| {
                                let mut p = gpui::PathBuilder::stroke(px(width));
                                for (k, &(x, y)) in pts.iter().enumerate() {
                                    if k == 0 {
                                        p.move_to(at(x, y));
                                    } else {
                                        p.line_to(at(x, y));
                                    }
                                }
                                if let Ok(path) = p.build() {
                                    w.paint_path(path, c);
                                }
                            };
                        line(
                            &[(0.0, 0.0), (1.0, 0.0), (1.0, ymax), (0.0, ymax), (0.0, 0.0)],
                            1.0,
                            border,
                            w,
                        );
                        line(&[(0.0, 0.0), (1.0, 1.0)], 1.0, border.opacity(0.5), w);
                        if ctrl.is_some() {
                            line(&[(0.0, 0.0), (x1, y1)], 1.0, muted, w);
                            line(&[(1.0, 1.0), (x2, y2)], 1.0, muted, w);
                        } else {
                            line(&[(0.0, 1.0), (1.0, 1.0)], 1.0, muted.opacity(0.5), w);
                        }
                        let pts = |upto: f32| -> Vec<(f32, f32)> {
                            let n = ((48.0 * upto).ceil() as usize).max(1);
                            (0..=n)
                                .map(|k| {
                                    let x = (k as f32 / n as f32) * upto;
                                    (x, ease.apply(x).clamp(0.0, ymax))
                                })
                                .collect()
                        };
                        line(&pts(1.0), 2.0, accent.opacity(0.25), w);
                        line(&pts(p), 3.0, accent, w);
                        // Playhead: x = time, y = eased progress.
                        let y = ease.apply(p).clamp(0.0, ymax);
                        line(&[(p, 0.0), (p, y)], 1.0, accent.opacity(0.5), w);
                        line(&[(0.0, y), (p, y)], 1.0, accent.opacity(0.5), w);
                        w.paint_quad(gpui::fill(
                            Bounds::new(
                                at(p, y) - gpui::point(px(4.5), px(4.5)),
                                gpui::size(px(9.0), px(9.0)),
                            ),
                            gpui::white(),
                        ));
                        for (hx, hy) in [(x1, y1), (x2, y2)].into_iter().filter(|_| ctrl.is_some())
                        {
                            let c = at(hx.clamp(0.0, 1.0), hy.clamp(0.0, 1.0));
                            w.paint_quad(gpui::fill(
                                Bounds::new(
                                    c - gpui::point(px(3.0), px(3.0)),
                                    gpui::size(px(6.0), px(6.0)),
                                ),
                                accent,
                            ));
                        }
                    },
                )
                .size_full()
            };
            let motion = Motion::new(Duration::from_millis(700), ease).entry(kind);
            let demo = div()
                .id(("an-demo", (i as u32) * 1000 + n))
                .w(px(280.0))
                .h(px(110.0))
                .overflow_hidden()
                .flex()
                .items_center()
                .justify_center()
                .child(dodai_gpui::entry_animation(
                    div().w(px(170.0)).child(
                        Card::new(CardVariant::Elevated)
                            .child(Label::new(format!("{title} entry"))),
                    ),
                    "an-card",
                    motion,
                ));
            anim_sections.push(
                Panel::new()
                    .title(title)
                    .child(
                        row()
                            .items_start()
                            .gap(px(24.0))
                            .child(
                                div()
                                    .id(("an-plot", (i as u32) * 1000 + n))
                                    .w(px(200.0))
                                    .h(px(200.0))
                                    .flex_none()
                                    .with_animation(
                                        ("an-plot-a", (i as u32) * 1000 + n),
                                        gpui::Animation::new(Duration::from_millis(1400)),
                                        move |el, p| {
                                            // hold at the end for the last 50%
                                            el.child(make_plot((p * 2.0).min(1.0)))
                                        },
                                    ),
                            )
                            .child(col().child(demo).child(
                                Button::new(("an-replay", i as u32), "Replay").on_click(
                                    move |_, _, cx| {
                                        me_r.update(cx, |s, cx| {
                                            s.bump(title);
                                            cx.notify();
                                        })
                                    },
                                ),
                            ))
                            .child(div().flex_1().min_w(px(220.0)).child(Muted::new(format!(
                                "{text}  {}",
                                if ctrl.is_some() {
                                    format!("cubic-bezier({x1}, {y1}, {x2}, {y2})")
                                } else {
                                    String::new()
                                }
                            )))),
                    )
                    .into_any_element(),
            );
        }
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
        ];
        let mut sections = sections;
        sections.push((
            "Loading",
            "Spinner · Indeterminate · Skeleton",
            loading.into_any_element(),
            "Loading",
        ));
        for (i, el) in anim_sections.into_iter().enumerate() {
            let k = anim_titles[i];
            sections.push(("Animation", k, el, k));
        }

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
            .gap(px(t.spacing.lg + 8.0))
            .w_full()
            .min_w_0()
            .p(px(if narrow { t.spacing.md } else { t.spacing.xl }))
            .child(ds_panel);
        let mut shown = 0;
        let mut last_cat = "";
        for (c, title, el, _key) in sections {
            // Loading demos repaint every frame forever: own page only.
            let in_cat = selected_cat.map_or(c != "Loading", |sc| sc == c);
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
            body = body.child(div().flex().flex_col().w_full().min_w_0().child(el));
        }
        if shown == 0 {
            body = body.child(
                EmptyState::new("⌕", "No components match")
                    .message("Clear the filter or pick another category."),
            );
        }

        if std::env::var_os("DODAI_AUTOSCROLL").is_some() {
            // Perf probe: sweep the scroll position every frame.
            let t = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0.0, |d| d.as_secs_f32())
                * 0.5)
                % 2.0;
            let tri = if t < 1.0 { t } else { 2.0 - t };
            self.scroll
                .set_offset(gpui::point(px(0.0), px(-4000.0 * tri)));
            window.request_animation_frame();
            thread_local! {
                static FPS: std::cell::Cell<(Option<std::time::Instant>, u32)> =
                    const { std::cell::Cell::new((None, 0)) };
            }
            FPS.with(|c| {
                let (t0, n) = c.get();
                let t0 = t0.unwrap_or_else(std::time::Instant::now);
                if t0.elapsed().as_secs_f32() >= 1.0 {
                    eprintln!("fps {n}");
                    c.set((Some(std::time::Instant::now()), 0));
                } else {
                    c.set((Some(t0), n + 1));
                }
            });
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
                    .track_scroll(&self.scroll)
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
        set_glow_intensity(cx, 0.5);
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
                        anim_ms: 220.0,
                        replay: 0,
                        sec_replay: Default::default(),
                        sec_mode: Default::default(),
                        loop_gen: 0,
                        ld_pb: dodai_gpui::Playback::Loop,
                        ld_ms: 0,
                        scroll: gpui::ScrollHandle::new(),
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

/// CSS cubic-bezier control points for an easing.
fn ease_ctrl(e: Easing) -> Option<(f32, f32, f32, f32)> {
    Some(match e {
        Easing::Linear => (0.0, 0.0, 1.0, 1.0),
        Easing::EaseIn => (0.42, 0.0, 1.0, 1.0),
        Easing::EaseOut => (0.0, 0.0, 0.58, 1.0),
        Easing::EaseInOut => (0.42, 0.0, 0.58, 1.0),
        Easing::CubicBezier(a, b, c, d) => (a, b, c, d),
        Easing::Spring { .. } => return None,
    })
}
