//! `cargo run -p dodai-gpui --example gpui_showcase`
//!
//! Every dodai-gpui component in one scrollable window.
//! Keys: `t` / `T` next / previous theme. Glow +/- buttons change
//! `Theme::with_glow_intensity` live.

use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, FocusHandle, KeyDownEvent,
    Window, WindowBounds, WindowOptions,
};
use dodai_core::{ButtonVariant, Theme, Tone};
use dodai_gpui::{
    cycle_next, cycle_prev, family_title, glow_intensity, hsla, install, scanlines_enabled,
    set_glow_intensity, set_scanlines, theme_of, ActiveDodai, Button, Card, CardVariant,
    Checkbox, DesignSystemSelector, Direction, Divider, Fab, FabSize, Heading, IconButton,
    InputBox, Knob, Label, DodaiStyled, DodaiTooltip, ListItem, Mono, Muted, NavItem,
    NavLayout, Navigation, Panel, ProgressBar, Radio, Scanlines, SearchBar, Slider, Tabs, Tag,
    ThemeSelector, Toggle, TopAppBar,
};

struct Showcase {
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
        let theme_panel = Panel::new().title(family)
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

        let body = div()
            .flex()
            .flex_col()
            .gap(gap)
            .p(px(t.spacing.xl))
            .child(Heading::new("Dodai UI").level(1))
            .child(ds_panel)
            .child(theme_panel)
            .child(section("Buttons").child(buttons))
            .child(section("Tags").child(tags))
            .child(
                section("Tabs")
                    .child(tabs)
                    .child(Muted::new(format!("active tab: {}", self.tab))),
            )
            .child(
                section("Toggle · Checkbox · Radio")
                    .child(controls)
                    .child(radios),
            )
            .child(section("Slider · Progress").child(sliders))
            .child(section("Knob · Vertical slider").child(dials))
            .child(section("App bar · Cards · FAB · Navigation").child(structure))
            .child(
                section("Input")
                    .child(InputBox::new().placeholder("search the net…"))
                    .child(InputBox::new().focused(true).child("focused input"))
                    .child(
                        div()
                            .lg_input(&t, false)
                            .h(px(t.shape.control_height))
                            .flex()
                            .items_center()
                            .child("legacy lg_input container"),
                    ),
            )
            .child(section("List · Divider").child(list).child(Divider::new()))
            .child(
                section("Text")
                    .child(Heading::new("Heading 1").level(1))
                    .child(Heading::new("Heading 2"))
                    .child(Heading::new("Heading 3").level(3))
                    .child(Label::new("Label text"))
                    .child(Muted::new("Muted text"))
                    .child(Mono::new("mono_text() // 0xDEADBEEF")),
            )
            .child(
                section("Tooltip").child(
                    div()
                        .id("tip")
                        .w(px(220.0))
                        .text_color(hsla(t.palette.link))
                        .tooltip(DodaiTooltip::text("Themed tooltip"))
                        .child("Hover me"),
                ),
            );

        div()
            .id("root")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|_, e: &KeyDownEvent, _, cx| {
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
            .lg_background(&t)
            .child(
                div()
                    .id("scroll")
                    .size_full()
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
                    cx.new(|cx| Showcase {
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
