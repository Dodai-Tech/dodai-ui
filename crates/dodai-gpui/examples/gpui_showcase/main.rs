//! `cargo run -p dodai-gpui --example gpui_showcase`
//!
//! Every dodai-gpui component in one scrollable window.
//! Keys: `t` / `T` next / previous theme. Glow +/- buttons change
//! `Theme::with_glow_intensity` live.

use dodai_core::{ButtonVariant, Theme, Tone};
use dodai_gpui::{
    cycle_next, cycle_prev, fuzzy_filter, glow_intensity, hsla, install, scanlines_enabled,
    set_glow_intensity, set_scanlines, theme_of, BubblePlacement, Button, Card, CardVariant,
    Checkbox, DesignSystemTree, Direction, Divider, DodaiStyled, DodaiTooltip, Fab, FabSize,
    Heading, IconButton, InputBox, Knob, Label, ListItem, Mono, Muted, NavItem, NavLayout,
    Navigation, Panel, ProgressBar, Radio, ResponsiveShell, Scanlines, SearchBar, Slider, Tabs,
    Tag, Toggle, TopAppBar,
};
use dodai_gpui::{
    BarChart, Blip, BoxPlot, BubbleChart, Candle, Candlestick, Date, DatePicker, DotMatrix,
    EmptyState, Funnel, Gauge, GaugeStyle, Heatmap, HexMap, Histogram, IconBadge, IconName,
    LineChart, Marker, NumberStepper, Oscilloscope, PieChart, PieStyle, RadarChart, RadarScope,
    RadialBar, RangeSlider, Rating, RichTooltip, RingStack, ScatterPlot, SearchView, Series,
    SignalBars, Snackbar, SparkStyle, Sparkline, Spectrum, StackedAreaChart, TagField, TextInput,
    TimePicker, Timeline, TimelineEvent, Toolbar, TreeItem, TreeView, Treemap, Waterfall,
};
use dodai_gpui::{
    CandleStyle, ChartStack, DepthChart, Gantt, GanttTask, Overlay, Pane, RenkoChart, TradeMarker,
};
use dodai_gpui::{
    CountUp, Easing, EntryKind, EntrySet, IndeterminateBar, Motion, Skeleton, Spinner,
};
use gpui::{actions, AnimationExt, KeyBinding, Menu, MenuItem, SharedString};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Entity, FocusHandle,
    KeyDownEvent, Window, WindowBounds, WindowOptions,
};
use std::time::Duration;

mod catalog;
mod charts_page;
mod foundations;
mod motion_page;
mod overview;
mod shaders_page;
mod sidebar;
use catalog::{caps, cat_blurb, cat_icon, section_id, sub_of, subs};
use charts_page::ChartDemos;
use foundations::{icons_view, palette_view, shape_tokens_view, spacing_elevation_view, type_view};

pub struct Showcase {
    replay: usize,
    sec_replay: std::collections::HashMap<&'static str, u32>,
    /// Sections whose one-shot Play run is still in progress.
    sec_playing: std::collections::HashMap<&'static str, u32>,
    sec_mode: std::collections::HashMap<&'static str, Player>,
    loop_gen: u32,
    ld_pb: dodai_gpui::Playback,
    ld_ms: u16,
    scroll: gpui::ScrollHandle,
    /// Section title to scroll to (plus frames to wait for a new category to lay out).
    jump: Option<(&'static str, u8)>,
    sec_kind: std::collections::HashMap<&'static str, EntrySet>,
    pill_hover: Option<(&'static str, u8)>,
    pop_hover: Option<(&'static str, u8)>,
    filter: Entity<TextInput>,
    search: Entity<TextInput>,
    in_name: Entity<TextInput>,
    in_notes: Entity<TextInput>,
    pw: Entity<TextInput>,
    dt_filter: Entity<TextInput>,
    date_range: dodai_gpui::DateRange,
    cat: usize,
    cat_count: usize,
    spy: Vec<(&'static str, usize)>,
    /// Selected subsection inside the current category (`None` = all).
    sub: Option<&'static str>,
    side_focus: FocusHandle,
    /// Sidebar collapsed to an icon rail.
    rail: bool,
    /// Collapsed state of the Navigation-section demo shell.
    demo_rail: bool,
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
    /// Corner cut override for the Buttons demo (`None` = theme default).
    btn_corners: Option<dodai_core::Corners>,
    heat_res: usize,
    heat_grad: usize,
    sh_gen: &'static str,
    sh_fx: Vec<&'static str>,
    sh_int: f32,
    sh_theme: bool,
    sh_px: f32,
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

impl Showcase {
    /// Entry set applied to a section's components (`None` = app default).
    fn scope_of(&self, title: &'static str, cx: &gpui::App) -> EntrySet {
        let cp = caps(title);
        if !(cp.entry || cp.fill || cp.stagger) {
            return EntrySet::NONE;
        }
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

    /// Total time of one run of the section: longest of the entry effects and
    /// the value-in (fill) animation, plus the stagger across its items.
    fn run_len(&self, title: &'static str, cx: &gpui::App) -> Duration {
        let m = Motion::of(cx);
        let set = self.sec_kind.get(title).copied().unwrap_or(m.entry);
        let entry = if set.is_empty() {
            Duration::ZERO
        } else {
            set.duration(m.duration)
        };
        let fill = match set.grow_ms() {
            0 if set.is_empty() => Duration::ZERO,
            0 => m.duration,
            ms => Duration::from_millis(ms as u64),
        };
        entry.max(fill) + Duration::from_millis(set.stagger_ms() as u64 * 8)
    }

    /// Replay and mark the section as playing until the run should be over.
    fn replay(&mut self, title: &'static str, cx: &mut Context<Self>) {
        self.bump(title);
        let tok = self.sec_replay.get(title).copied().unwrap_or(0);
        self.sec_playing.insert(title, tok);
        let run = self.run_len(title, cx);
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(run).await;
            let _ = this.update(cx, |s, cx| {
                if s.sec_playing.get(title) == Some(&tok) {
                    s.sec_playing.remove(title);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn set_mode(&mut self, title: &'static str, mode: Player, cx: &mut Context<Self>) {
        self.sec_mode.insert(title, mode);
        self.loop_gen += 1;
        self.replay(title, cx);
        if mode == Player::Looping {
            let gen = self.loop_gen;
            cx.spawn(async move |this, cx| loop {
                let wait = this
                    .update(cx, |s, cx| {
                        s.run_len(title, cx) + Duration::from_millis(600)
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
        let cp = caps(title);
        if !(cp.entry || cp.fill || cp.stagger) {
            return div().into_any_element();
        }
        for (ki, k) in EntryKind::ALL
            .iter()
            .copied()
            .enumerate()
            .skip(1)
            .filter(|_| cp.entry)
        {
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
                        s.replay(title, cx);
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
                                    s.replay(title, cx);
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
        ]
        .into_iter()
        .filter(|(tag, ..)| if *tag == 100 { cp.fill } else { cp.stagger })
        {
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
            let me_c = me.clone();
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
                .child(
                    chip(("sec-f", uid), text, gms > 0).on_click(move |_, _, cx| {
                        // Toggle: active → off (default), off → sensible default.
                        me_c.update(cx, |s, cx| {
                            let cur = s.sec_kind.get(title).copied().unwrap_or(base);
                            let v = if gms > 0 {
                                0
                            } else if slot == 0 {
                                800
                            } else {
                                100
                            };
                            let nw = if slot == 0 {
                                cur.with_grow_ms(v)
                            } else {
                                cur.with_stagger_ms(v)
                            };
                            s.sec_kind.insert(title, nw);
                            s.replay(title, cx);
                            cx.notify();
                        })
                    }),
                );
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
                                    s.replay(title, cx);
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
                btn(
                    1,
                    dodai_gpui::IconName::Play,
                    mode == Player::Idle && self.sec_playing.contains_key(title),
                )
                .on_click(move |_, _, cx| {
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

        let cut = |b: Button| b;
        let buttons = row()
            .child(cut(Button::new("b-primary", "Primary").on_click(
                cx.listener(|s, _, _, cx| {
                    s.clicks += 1;
                    cx.notify();
                }),
            )))
            .child(cut(
                Button::new("b-secondary", "Secondary").variant(ButtonVariant::Secondary)
            ))
            .child(cut(
                Button::new("b-outline", "Outline").variant(ButtonVariant::Outline)
            ))
            .child(cut(
                Button::new("b-outline2", "Outline 2").variant(ButtonVariant::OutlineSecondary)
            ))
            .child(cut(
                Button::new("b-danger", "Danger").variant(ButtonVariant::Danger)
            ))
            .child(cut(
                Button::new("b-ghost", "Ghost").variant(ButtonVariant::Ghost)
            ))
            .child(cut(Button::new("b-dis", "Disabled").disabled(true)))
            .child(cut(Button::new("b-shim", "Loading…").shimmer(true)))
            .child(cut(Button::new("b-shim2", "Outline…")
                .variant(ButtonVariant::Outline)
                .shimmer(true)))
            .child(Muted::new(format!("primary clicked {}×", self.clicks)));

        let corner_picker = {
            use dodai_core::Corners as C;
            let me2 = me.clone();
            let cur = self.btn_corners;
            let chip = |id: &'static str,
                        label: &'static str,
                        on: bool,
                        f: Box<dyn Fn(Option<C>) -> Option<C>>| {
                let m = me2.clone();
                Button::new(id, label)
                    .variant(if on {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Outline
                    })
                    .on_click(move |_, _, cx| {
                        m.update(cx, |s, cx| {
                            s.btn_corners = f(s.btn_corners);
                            cx.notify()
                        })
                    })
            };
            let base = cur.unwrap_or(C::BOTTOM_RIGHT);
            let tog = |i: usize| -> Box<dyn Fn(Option<C>) -> Option<C>> {
                Box::new(move |c| {
                    let b = c.unwrap_or(C::BOTTOM_RIGHT);
                    Some(b.set(i, !b.get(i)))
                })
            };
            let preset =
                |c: Option<C>| -> Box<dyn Fn(Option<C>) -> Option<C>> { Box::new(move |_| c) };
            row()
                .child(Label::new("Cut corner"))
                .child(chip("cc-ul", "UL", cur.is_some() && base.get(0), tog(0)))
                .child(chip("cc-ur", "UR", cur.is_some() && base.get(1), tog(1)))
                .child(chip("cc-lr", "LR", cur.is_some() && base.get(2), tog(2)))
                .child(chip("cc-ll", "LL", cur.is_some() && base.get(3), tog(3)))
                .child(chip(
                    "cc-diag",
                    "Diagonal",
                    cur == Some(C::DIAGONAL),
                    preset(Some(C::DIAGONAL)),
                ))
                .child(chip(
                    "cc-anti",
                    "Anti-diagonal",
                    cur == Some(C::ANTI_DIAGONAL),
                    preset(Some(C::ANTI_DIAGONAL)),
                ))
                .child(chip(
                    "cc-left",
                    "Left",
                    cur == Some(C::LEFT),
                    preset(Some(C::LEFT)),
                ))
                .child(chip(
                    "cc-right",
                    "Right",
                    cur == Some(C::RIGHT),
                    preset(Some(C::RIGHT)),
                ))
                .child(chip(
                    "cc-all",
                    "All",
                    cur == Some(C::ALL),
                    preset(Some(C::ALL)),
                ))
                .child(chip(
                    "cc-none",
                    "None",
                    cur == Some(C::NONE),
                    preset(Some(C::NONE)),
                ))
                .child(chip("cc-theme", "Theme", cur.is_none(), preset(None)))
                .child(
                    Button::new("cc-apply", "Set as theme")
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, _, cx| {
                            let c = me2.read(cx).btn_corners;
                            if let Some(c) = c {
                                dodai_gpui::set_corners(cx, c);
                            }
                        }),
                )
        };

        let ov = self.btn_corners;
        let sc = move |b: Button| match ov {
            Some(c) => b.corners(c),
            None => b,
        };
        let sc_tag = move |t: Tag| match ov {
            Some(c) => t.corners(c),
            None => t,
        };
        let sc_card = move |c: Card| match ov {
            Some(k) => c.corners(k),
            None => c,
        };
        let sc_panel = move |p: Panel| match ov {
            Some(k) => p.corners(k),
            None => p,
        };
        let shape_gallery = col()
            .child(
                row()
                    .child(sc(Button::new("sg-p", "Primary")))
                    .child(sc(
                        Button::new("sg-s", "Secondary").variant(ButtonVariant::Secondary)
                    ))
                    .child(sc(
                        Button::new("sg-o", "Outline").variant(ButtonVariant::Outline)
                    ))
                    .child(sc(
                        Button::new("sg-d", "Danger").variant(ButtonVariant::Danger)
                    )),
            )
            .child(
                row()
                    .child(sc_tag(Tag::new("online", Tone::Success).solid(true)))
                    .child(sc_tag(Tag::new("warn", Tone::Warning)))
                    .child(sc_tag(Tag::new("breach", Tone::Danger).solid(true))),
            )
            .child(
                row()
                    .items_start()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(220.0))
                            .child(sc_card(Card::new(CardVariant::Outlined).headline("Card"))),
                    )
                    .child(div().flex_1().min_w(px(220.0)).child(sc_card(
                        Card::new(CardVariant::Elevated).headline("Elevated card"),
                    )))
                    .child(div().flex_1().min_w(px(220.0)).child(sc_panel(
                        Panel::new().title("Panel").child(Muted::new("Panel body")),
                    ))),
            );

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

        // Global shape properties (glow, scanlines, cut corners): Shape category.
        let shape_props = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .child(
                row().child(Label::new("Glow")).child(
                    div().w(px(200.0)).child(
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
                    .child(Muted::new("key: l")),
            );

        let ds_panel = Panel::new().focused(true).child(
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .child(Label::new("Design System Selection"))
                .child(DesignSystemTree::new("ds-tree")),
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
            .child(
                IconBadge::new(IconButton::new("bell", "🔔").icon(IconName::Bell))
                    .count(12, Tone::Danger),
            )
            .child(
                IconBadge::new(IconButton::new("mail", "✉").icon(IconName::Mail))
                    .dot(Tone::Success),
            );
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
            .child(IconButton::new("tb1", "✎").icon(IconName::Edit))
            .child(IconButton::new("tb2", "⎘").icon(IconName::Copy));

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

        // ── new components ─────────────────────────────────────────────
        let m_range = me.clone();
        let range_picker = row()
            .child(
                DatePicker::new("date-range", y, mo)
                    .range(self.date_range)
                    .on_select({
                        let m = m_range.clone();
                        move |d, _, cx| {
                            m.update(cx, |s, cx| {
                                s.date_range = dodai_gpui::range_pick(s.date_range, d);
                                cx.notify()
                            })
                        }
                    })
                    .on_month({
                        let m = m_range.clone();
                        move |y, mo, _, cx| {
                            m.update(cx, |s, cx| {
                                s.date.0 = y;
                                s.date.1 = mo;
                                cx.notify()
                            })
                        }
                    }),
            )
            .child(Muted::new(match self.date_range.span() {
                Some((a, b)) => format!(
                    "{}-{:02}-{:02} → {}-{:02}-{:02}",
                    a.year, a.month, a.day, b.year, b.month, b.day
                ),
                None => match self.date_range.start {
                    Some(a) => format!("from {}-{:02}-{:02}: pick the end", a.year, a.month, a.day),
                    None => "click a start day, then an end day".to_string(),
                },
            }));
        let dt_rows: std::rc::Rc<Vec<Vec<SharedString>>> = std::rc::Rc::new(
            (0..300)
                .map(|i| {
                    let roles = ["Engineer", "Designer", "Analyst", "Manager"];
                    let regions = ["EMEA", "APAC", "AMER"];
                    vec![
                        SharedString::from(format!("User {:03}", (i * 37) % 300)),
                        SharedString::from(roles[i % 4]),
                        SharedString::from(regions[(i / 3) % 3]),
                        SharedString::from(format!("{}", (i * 7919) % 1000)),
                    ]
                })
                .collect(),
        );
        let dt_filter_text = self.dt_filter.read(cx).text().to_string();
        let data_table = col()
            .child(
                row()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(200.0))
                            .child(self.dt_filter.clone()),
                    )
                    .child(Muted::new(
                        "click a header to sort · drag a column edge · click rows to select",
                    )),
            )
            .child(
                dodai_gpui::DataTable::new("dt")
                    .columns([
                        dodai_gpui::Column::new("Name").width(160.0),
                        dodai_gpui::Column::new("Role").width(140.0),
                        dodai_gpui::Column::new("Region").width(110.0),
                        dodai_gpui::Column::new("Score").width(100.0).numeric(true),
                    ])
                    .rows_shared(dt_rows)
                    .filter(dt_filter_text)
                    .height(260.0),
            );
        let md_src = "# Markdown\n\nSupports **bold**, *italic*, `inline code` and [links](https://example.com).\n\n- headings and paragraphs\n- nested lists\n  - indent two spaces\n    1. numbered inside\n    2. restarts per level\n- fenced code and tables\n\n| Token | Value | Note |\n|:--|--:|:-:|\n| radius | 12 | rounded |\n| chamfer | 0 | none |\n\n> quotes read muted\n\n```rust\nfn main() {\n    let n: u32 = 42; // answer\n    println!(\"{n}\");\n}\n```";
        let sample = format!("{}/examples/assets/sample.png", env!("CARGO_MANIFEST_DIR"));
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

        let ChartDemos {
            charts_xy,
            gauge_styles,
            instruments,
            pies,
            donuts,
            sparks,
            charts_market,
            charts_misc,
            charts_more1,
            charts_more2,
            charts_more3,
        } = self.chart_demos(&me, cx);

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

        let (loading, anim_sections, anim_titles) = self.motion_sections(&me, cx);
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
                "Foundations",
                "Colour palette",
                section!("Colour palette")
                    .child(Muted::new("Every Palette token of the active design system."))
                    .child(palette_view(&t))
                    .into_any_element(),
                "Colour palette",
            ),
            (
                "Foundations",
                "Type scale",
                section!("Type scale")
                    .child(Muted::new("Sizes and families from the theme's Typography."))
                    .child(type_view(&t))
                    .into_any_element(),
                "Type scale",
            ),
            (
                "Foundations",
                "Spacing · Elevation",
                section!("Spacing · Elevation")
                    .child(Muted::new("Spacing scale (bars are 8x the value) and surface layers."))
                    .child(spacing_elevation_view(&t))
                    .into_any_element(),
                "Spacing · Elevation",
            ),
            (
                "Foundations",
                "Shape tokens",
                section!("Shape tokens")
                    .child(Muted::new("Read-only summary of the theme's shape and effect tokens."))
                    .child(shape_tokens_view(&t))
                    .into_any_element(),
                "Shape tokens",
            ),
            (
                "Foundations",
                "Icons",
                section!("Icons")
                    .child(Muted::new("Cyberpunk icon pack: straight lines, 45° chamfers, octagons. Click to copy `IconName::…`."))
                    .child(icons_view(&t))
                    .into_any_element(),
                "Icons",
            ),
            (
                "Shape",
                "Glow · Scanlines",
                section!("Glow · Scanlines")
                    .child(Muted::new(
                        "Theme-wide effects. They apply to every component. Cut corners: pick below, then \"Set as theme\".",
                    ))
                    .child(shape_props)
                    .into_any_element(),
                "Glow · Scanlines",
            ),
            (
                "Shape",
                "Corner shape",
                section!("Corner shape")
                    .child(Muted::new(
                        "Override the corner cut on any component. \"Theme\" follows the design system's own shape.",
                    ))
                    .child(corner_picker)
                    .child(shape_gallery)
                    .into_any_element(),
                "Corner shape",
            ),
            (
                "Shape",
                "Border",
                section!("Border")
                    .child(Muted::new(
                        "BorderStyle { width, opacity }: set per panel with `.border(..)`, or for every panel with `set_section_border`. Sections use THIN by default.",
                    ))
                    .child({
                        use dodai_gpui::BorderStyle as B;
                        let mut r = row();
                        for (name, b) in [
                            ("NONE", B::NONE),
                            ("SUBTLE", B::SUBTLE),
                            ("THIN (default)", B::THIN),
                            ("NORMAL", B::NORMAL),
                            ("BOLD", B::BOLD),
                        ] {
                            r = r.child(
                                div().flex_1().min_w(px(140.0)).child(
                                    Panel::new().border(b).child(
                                        col()
                                            .child(Label::new(name))
                                            .child(Muted::new(format!(
                                                "{:.2}px · {:.0}%",
                                                b.width,
                                                b.opacity * 100.0
                                            ))),
                                    ),
                                ),
                            );
                        }
                        r
                    })
                    .child({
                        use dodai_gpui::{BorderStyle as B, Corner};
                        let mut r = row();
                        for e in Corner::ALL {
                            r = r.child(
                                div().flex_1().min_w(px(110.0)).child(
                                    Panel::new()
                                        .border(B::THIN.accent(Some(e)))
                                        .child(Label::new(e.name())),
                                ),
                            );
                        }
                        r
                    })
                    .child({
                        use dodai_gpui::{BorderStyle as B, Corner};
                        let cur = dodai_gpui::section_border(cx);
                        let mut r = row().child(Label::new("Accent line (all sections)"));
                        let mut opts: Vec<Option<Corner>> = vec![None];
                        opts.extend(Corner::ALL.map(Some));
                        for (i, o) in opts.into_iter().enumerate() {
                            r = r.child(
                                Button::new(
                                    ("bd-acc", i),
                                    o.map_or("None", |e| e.name()),
                                )
                                .variant(if cur.accent == o {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Outline
                                })
                                .on_click(move |_, _, cx| {
                                    let b: B = dodai_gpui::section_border(cx);
                                    dodai_gpui::set_section_border(cx, b.accent(o));
                                }),
                            );
                        }
                        r
                    })
                    .child({
                        let b = dodai_gpui::section_border(cx);
                        row()
                            .child(Label::new("Grey / yellow thickness"))
                            .child(
                                div().w(px(180.0)).child(
                                    Slider::new("bd-width", b.width)
                                        .range(0.0, 4.0)
                                        .step(0.25)
                                        .show_value(true)
                                        .format(|v| format!("grey {v:.2}px"))
                                        .on_change(move |v, _, cx| {
                                            let b = dodai_gpui::section_border(cx);
                                            dodai_gpui::set_section_border(
                                                cx,
                                                dodai_gpui::BorderStyle { width: v.clamp(0.0, 8.0), ..b },
                                            );
                                        }),
                                ),
                            )
                            .child(
                                div().w(px(180.0)).child(
                                    Slider::new("bd-accent", b.accent_width)
                                        .range(0.5, 6.0)
                                        .step(0.25)
                                        .show_value(true)
                                        .format(|v| format!("yellow {v:.2}px"))
                                        .on_change(move |v, _, cx| {
                                            let b = dodai_gpui::section_border(cx);
                                            dodai_gpui::set_section_border(
                                                cx,
                                                b.accent_width(v),
                                            );
                                        }),
                                ),
                            )
                            .child(
                                div().w(px(180.0)).child(
                                    Slider::new("bd-opacity", b.opacity)
                                        .range(0.0, 1.0)
                                        .step(0.05)
                                        .show_value(true)
                                        .format(|v| format!("{v:.2}"))
                                        .on_change(move |v, _, cx| {
                                            let b = dodai_gpui::section_border(cx);
                                            dodai_gpui::set_section_border(
                                                cx,
                                                dodai_gpui::BorderStyle { opacity: v.clamp(0.0, 1.0), ..b },
                                            );
                                        }),
                                ),
                            )
                    })
                    .into_any_element(),
                "Border",
            ),
            (
                "Actions",
                "Buttons · IconButton · FAB",
                section!("Buttons")
                    .child(buttons)
                    .into_any_element(),
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
                            .child(div().flex_1().min_w(px(200.0)).child(tree))
                            .child(div().flex_1().min_w(px(220.0)).child(timeline))
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
                "Data display",
                "Data table · sort filter select resize virtual scroll",
                section!("Data table").child(data_table).into_any_element(),
                "Data table",
            ),
            (
                "Data display",
                "Code block · Markdown · rich text",
                section!("Code block · Markdown")
                    .child(
                        dodai_gpui::CodeBlock::new(
                            "code",
                            "use std::collections::HashMap;\n\n/// Count words.\nfn count(text: &str) -> HashMap<String, usize> {\n    let mut m = HashMap::new();\n    for w in text.split_whitespace() {\n        *m.entry(w.to_lowercase()).or_insert(0) += 1;\n    }\n    m // 42 buckets max\n}",
                        )
                        .language("rust"),
                    )
                    .child(dodai_gpui::Markdown::new("md", md_src))
                    .into_any_element(),
                "Code block · Markdown",
            ),
            (
                "Data display",
                "Image · media · Avatar group",
                section!("Image · Avatar group")
                    .child(
                        row()
                            .items_start()
                            .child(
                                dodai_gpui::Image::path(sample.clone())
                                    .size(260.0, 160.0)
                                    .caption("Image (cover)"),
                            )
                            .child(
                                dodai_gpui::Image::path(sample.clone())
                                    .size(260.0, 160.0)
                                    .video(true)
                                    .caption("Video poster"),
                            )
                            .child(
                                dodai_gpui::Image::path(sample)
                                    .size(160.0, 160.0)
                                    .fit(dodai_gpui::ImageFit::Contain)
                                    .caption("Contain"),
                            ),
                    )
                    .child(
                        row()
                            .child(dodai_gpui::AvatarGroup::new(["AL", "BK", "CM", "DN"]))
                            .child(dodai_gpui::AvatarGroup::new(["AL", "BK", "CM", "DN", "EO", "FP", "GQ"]).max(4))
                            .child(dodai_gpui::AvatarGroup::new(["AL", "BK", "CM", "DN", "EO"]).max(3).size(28.0)),
                    )
                    .into_any_element(),
                "Image · Avatar group",
            ),
            (
                "Data display",
                "Circular progress · Banner",
                section!("Circular progress · Banner")
                    .child(
                        row()
                            .child(dodai_gpui::CircularProgress::new(0.25))
                            .child(dodai_gpui::CircularProgress::new(0.72).tone(Tone::Success).size(72.0).thickness(7.0))
                            .child(dodai_gpui::CircularProgress::new(1.0).tone(Tone::Info).size(40.0).thickness(4.0))
                            .child(dodai_gpui::CircularProgress::indeterminate()),
                    )
                    .child(dodai_gpui::Banner::new("bn1", Tone::Info, "A new version is available.").action("Update", |_, _| {}))
                    .child(dodai_gpui::Banner::new("bn2", Tone::Warning, "Your trial ends in 3 days."))
                    .child(dodai_gpui::Banner::new("bn3", Tone::Danger, "Connection lost. Retrying…").dismissible(false))
                    .into_any_element(),
                "Circular progress · Banner",
            ),
            (
                "Data display",
                "Clock · Timer",
                section!("Clock · Timer")
                    .child(
                        row()
                            .child(dodai_gpui::Clock::new("clk").local())
                            .child(dodai_gpui::Timer::new("tmr").minutes(1)),
                    )
                    .into_any_element(),
                "Clock · Timer",
            ),
            (
                "Data display",
                "Stopwatch · Tap tempo",
                section!("Stopwatch · Tap tempo")
                    .child(
                        row()
                            .child(dodai_gpui::Stopwatch::new("sw"))
                            .child(dodai_gpui::TapTempo::new("tap")),
                    )
                    .into_any_element(),
                "Stopwatch · Tap tempo",
            ),
            (
                "Data display",
                "Scroll area · styled scrollbar",
                section!("Scroll area")
                    .child(
                        dodai_gpui::ScrollArea::new("sa").height(200.0).child(
                            div().flex().flex_col().gap(px(8.0)).children(
                                (1..=40).map(|i| Label::new(format!("Row {i} · drag the thumb or use the wheel"))),
                            ),
                        ),
                    )
                    .into_any_element(),
                "Scroll area",
            ),
            (
                "Navigation",
                "Popover · Hover card · Context menu · Dropdown menu",
                section!("Popover · Hover card · Context menu · Dropdown")
                    .child(popovers)
                    .into_any_element(),
                "Popover · Hover card · Context menu · Dropdown",
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
                    .child(div().flex_1().min_w(px(260.0)).child(self.in_name.clone()))
                    .child(div().flex_1().min_w(px(260.0)).child(self.in_notes.clone()))
                    .child(InputBox::new().placeholder("static placeholder box"))
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
                "Inputs",
                "Password · One-time code",
                section!("Password · OTP")
                    .child(
                        dodai_gpui::FormField::new("Password")
                            .required(true)
                            .help("Shown as dots; copy is disabled")
                            .child(self.pw.clone()),
                    )
                    .child(
                        dodai_gpui::FormField::new("Verification code")
                            .help("Type or paste 6 digits")
                            .child(dodai_gpui::OtpInput::new("otp", 6)),
                    )
                    .child(
                        dodai_gpui::FormField::new("PIN (masked)")
                            .child(dodai_gpui::OtpInput::new("pin", 4).masked(true)),
                    )
                    .into_any_element(),
                "Password · OTP",
            ),
            (
                "Inputs",
                "Form field · label help error validation",
                section!("Form field")
                    .child(
                        row()
                            .items_start()
                            .child(div().flex_1().min_w(px(220.0)).child(
                                dodai_gpui::FormField::new("Name")
                                    .required(true)
                                    .help("As on your ID")
                                    .child(InputBox::new().placeholder("Ada Lovelace")),
                            ))
                            .child(div().flex_1().min_w(px(220.0)).child(
                                dodai_gpui::FormField::new("Email")
                                    .error("Enter a valid address")
                                    .child(InputBox::new().placeholder("name@")),
                            ))
                            .child(div().flex_1().min_w(px(220.0)).child(
                                dodai_gpui::FormField::new("Username")
                                    .success("Available")
                                    .child(InputBox::new().placeholder("ada")),
                            ))
                            .child(div().flex_1().min_w(px(220.0)).child(
                                dodai_gpui::FormField::new("Bio")
                                    .warning("Over 80% of the limit")
                                    .child(InputBox::new().placeholder("…")),
                            )),
                    )
                    .into_any_element(),
                "Form field",
            ),
            (
                "Inputs",
                "Toggle group · Checkbox group · select all · switches",
                section!("Toggle group · Checkbox group")
                    .child(
                        row()
                            .child(
                                dodai_gpui::ToggleGroup::new("tg1", ["Left", "Center", "Right"])
                                    .selected([0]),
                            )
                            .child(
                                dodai_gpui::ToggleGroup::new("tg2", ["Bold", "Italic", "Underline"])
                                    .multiple(true)
                                    .selected([0, 2]),
                            ),
                    )
                    .child(
                        row()
                            .items_start()
                            .gap(px(48.0))
                            .child(
                                dodai_gpui::CheckboxGroup::new("cg", ["Email", "SMS", "Push", "Slack"])
                                    .select_all("All channels")
                                    .checked([0, 2]),
                            )
                            .child(
                                dodai_gpui::CheckboxGroup::new("sg", ["Wi-Fi", "Bluetooth", "Location"])
                                    .select_all("Everything")
                                    .switches(true)
                                    .checked([0]),
                            ),
                    )
                    .into_any_element(),
                "Toggle group · Checkbox group",
            ),
            (
                "Inputs",
                "Date range picker",
                section!("Date range").child(range_picker).into_any_element(),
                "Date range",
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
            ),
            (
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
                        let items = ["Inbox", "Contracts", "Fixers", "Market"];
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
                "Pie",
                section!("Pie").child(pies).into_any_element(),
                "Pie",
            ),
            (
                "Charts",
                "Donut",
                section!("Donut").child(donuts).into_any_element(),
                "Donut",
            ),
            (
                "Charts",
                "Gauges",
                section!("Gauges").child(gauge_styles).into_any_element(),
                "Gauges",
            ),
            (
                "Charts",
                "Sparkline",
                section!("Sparkline").child(sparks).into_any_element(),
                "Sparkline",
            ),
            (
                "Charts",
                "Instruments",
                section!("Instruments")
                    .child(instruments)
                    .into_any_element(),
                "Instruments",
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
                "Market · Planning",
                section!("Market · planning charts")
                    .child(charts_market)
                    .into_any_element(),
                "Market · planning charts",
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

        self.shader_sections(&me, &mut sections, cx);

        for (i, el) in anim_sections.into_iter().enumerate() {
            let k = anim_titles[i];
            sections.push(("Animation", k, el, k));
        }

        // Shaders are the biggest category and design-independent: always last.
        sections.sort_by_key(|(c, _, _, _)| match *c {
            "Shaders" => 1,
            _ => 0,
        });
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
        let sections_meta: Vec<(&'static str, &'static str)> =
            sections.iter().map(|(c, l, _, _)| (*c, *l)).collect();
        // Scroll-spy: the last section whose top has reached the viewport top
        // (positions come from the previous frame's layout).
        let active: Option<&'static str> = {
            // Item bounds are in unscrolled content space, so add the offset.
            let top = self.scroll.bounds().top();
            let off = self.scroll.offset().y;
            self.spy
                .iter()
                .rfind(|(_, ix)| {
                    self.scroll
                        .bounds_for_item(*ix)
                        .is_some_and(|b| b.top() + off <= top + px(SPY_SLACK))
                })
                .or(self.spy.first())
                .map(|(t, _)| *t)
        };
        let (side, side_need) =
            self.sidebar(&me, &cats, &sections_meta, active, &query, count_in, cx);

        let overview = self.cat == 0
            && self.sub.is_none()
            && query.is_empty()
            && std::env::var_os("DODAI_ONLY").is_none();
        let mut body = div()
            .id(("body", self.cat * 1000 + self.replay))
            .track_scroll(&self.scroll)
            // Scroll-spy reads layout, so repaint on scroll even on static pages.
            .on_scroll_wheel(cx.listener(|_, _, _, cx| cx.notify()))
            .flex()
            .flex_col()
            .flex_1()
            .gap(px(t.spacing.lg + 8.0))
            .min_w_0()
            .overflow_y_scroll()
            .p(px(if narrow { t.spacing.md } else { t.spacing.xl }));
        let mut ds_panel = Some(ds_panel);
        let mut n_children = 0usize;
        let mut jump_ix: Option<usize> = None;
        let jump_title = self.jump.map(|(j, _)| j);
        if overview {
            let kids = self.overview(&me, &cats, &sections_meta, ds_panel.take(), cx);
            n_children += kids.len();
            body = body.children(kids);
        }
        // DODAI_ONLY (debug captures) hides the selector so the section fills the view.
        if let Some(d) = ds_panel
            .take()
            .filter(|_| std::env::var_os("DODAI_ONLY").is_none())
        {
            body = body.child(d);
            n_children += 1;
        }
        if std::env::var_os("DODAI_LIST").is_some() {
            for (c, t) in &sections_meta {
                eprintln!("{c}\t{t}");
            }
            std::process::exit(0);
        }
        let mut spy: Vec<(&'static str, usize)> = Vec::new();
        let mut shown = 0;
        let mut last_cat = "";
        let mut last_sub: Option<&str> = None;
        for (c, title, el, _key) in sections {
            // Loading demos repaint every frame forever: own page only.
            let in_cat = if overview {
                !matches!(c, "Loading" | "Shaders")
            } else {
                selected_cat.map_or(c != "Loading", |sc| sc == c)
            };
            let hit =
                query.is_empty() || !fuzzy_filter(&query, &[format!("{c} {title}")]).is_empty();
            let only = std::env::var("DODAI_ONLY").is_ok_and(|o| !title.contains(&o));
            let sub_ok = self.sub.is_none_or(|x| sub_of(c, title) == Some(x));
            if !(in_cat && hit && sub_ok) || only {
                continue;
            }
            if c != last_cat {
                body = body.child(Heading::new(c).level(2));
                n_children += 1;
                last_cat = c;
                last_sub = None;
            }
            if let Some(sb) = sub_of(c, title) {
                if last_sub != Some(sb) {
                    body = body.child(Heading::new(sb).level(3));
                    n_children += 1;
                    last_sub = Some(sb);
                }
            }
            spy.push((title, n_children));
            if jump_title == Some(title) {
                jump_ix = Some(n_children);
            }
            shown += 1;
            n_children += 1;
            body = body.child(el);
        }
        self.spy = spy;
        if let Some((j, wait)) = self.jump {
            if wait > 0 {
                self.jump = Some((j, wait - 1));
            } else {
                self.jump = None;
                if let Some(ix) = jump_ix {
                    self.scroll.scroll_to_top_of_item(ix);
                }
            }
            window.request_animation_frame();
        }
        if shown == 0 && !overview {
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
                ResponsiveShell::new("shell")
                    .breakpoint(640.0)
                    .sidebar_width(if narrow {
                        side_need.clamp(170.0, 230.0)
                    } else {
                        side_need.clamp(240.0, 340.0)
                    })
                    .sidebar(side)
                    .collapsed(self.rail)
                    .on_collapse({
                        let m = me.clone();
                        move |c, _, cx| {
                            m.update(cx, |s, cx| {
                                s.rail = c;
                                cx.notify()
                            })
                        }
                    })
                    .content(body),
            )
            .child(Scanlines::new())
    }
}

actions!(
    showcase,
    [
        Quit,
        CloseWindow,
        Minimize,
        ZoomWindow,
        ToggleFullscreen,
        WindowLarger,
        WindowSmaller,
        WindowReset
    ]
);

/// Px below the scroll container top at which a section counts as current.
const SPY_SLACK: f32 = 60.0;
const DEFAULT_W: f32 = 980.0;
const DEFAULT_H: f32 = 760.0;

/// Run `f` on the active window (menu/shortcut actions are app-global).
fn on_active(cx: &mut App, f: impl FnOnce(&mut Window) + 'static) {
    if let Some(w) = cx.active_window() {
        let _ = w.update(cx, |_, window, _| f(window));
    }
}

/// Window management: macOS-style menu + keyboard shortcuts.
fn window_shortcuts(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-w", CloseWindow, None),
        KeyBinding::new("cmd-m", Minimize, None),
        KeyBinding::new("cmd-ctrl-f", ToggleFullscreen, None),
        KeyBinding::new("cmd-ctrl-z", ZoomWindow, None),
        KeyBinding::new("cmd-=", WindowLarger, None),
        KeyBinding::new("cmd-shift-=", WindowLarger, None),
        KeyBinding::new("cmd--", WindowSmaller, None),
        KeyBinding::new("cmd-0", WindowReset, None),
    ]);
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &CloseWindow, cx| on_active(cx, |w| w.remove_window()));
    cx.on_action(|_: &Minimize, cx| on_active(cx, |w| w.minimize_window()));
    cx.on_action(|_: &ZoomWindow, cx| on_active(cx, |w| w.zoom_window()));
    cx.on_action(|_: &ToggleFullscreen, cx| on_active(cx, |w| w.toggle_fullscreen()));
    cx.on_action(|_: &WindowLarger, cx| {
        on_active(cx, |w| {
            let b = w.bounds().size;
            w.resize(size(b.width * 1.1, b.height * 1.1));
        })
    });
    cx.on_action(|_: &WindowSmaller, cx| {
        on_active(cx, |w| {
            let b = w.bounds().size;
            w.resize(size(b.width / 1.1, b.height / 1.1));
        })
    });
    cx.on_action(|_: &WindowReset, cx| {
        on_active(cx, |w| w.resize(size(px(DEFAULT_W), px(DEFAULT_H))))
    });
    cx.set_menus(vec![
        Menu {
            name: "Dodai".into(),
            items: vec![MenuItem::action("Quit", Quit)],
        },
        Menu {
            name: "Window".into(),
            items: vec![
                MenuItem::action("Minimize", Minimize),
                MenuItem::action("Zoom", ZoomWindow),
                MenuItem::action("Enter Full Screen", ToggleFullscreen),
                MenuItem::separator(),
                MenuItem::action("Larger", WindowLarger),
                MenuItem::action("Smaller", WindowSmaller),
                MenuItem::action("Actual Size", WindowReset),
                MenuItem::separator(),
                MenuItem::action("Close", CloseWindow),
            ],
        },
    ]);
}

/// Build the showcase view (also used by the headless smoke test).
impl Showcase {
    /// Initial state; the TextInput entities are created here.
    pub fn build(cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| TextInput::new(cx).placeholder("Search…"));
        let filter = cx.new(|cx| TextInput::new(cx).placeholder("Filter components…"));
        let in_name = cx.new(|cx| TextInput::new(cx).placeholder("handle…"));
        let pw = cx.new(|cx| TextInput::new(cx).placeholder("password").password(true));
        let dt_filter = cx.new(|cx| TextInput::new(cx).placeholder("Filter rows…"));
        let in_notes = cx.new(|cx| {
            TextInput::new(cx)
                .multiline(4)
                .placeholder("notes (multi-line)…")
        });
        Showcase {
            filter,
            search,
            in_name,
            in_notes,
            pw,
            dt_filter,
            date_range: Default::default(),
            cat: std::env::var("DODAI_CAT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            replay: 0,
            sec_replay: Default::default(),
            sec_playing: Default::default(),
            sec_mode: Default::default(),
            loop_gen: 0,
            ld_pb: dodai_gpui::Playback::Loop,
            ld_ms: 0,
            scroll: gpui::ScrollHandle::new(),
            jump: None,
            sec_kind: Default::default(),
            pill_hover: None,
            pop_hover: None,
            cat_count: 1,
            spy: Vec::new(),
            sub: None,
            side_focus: cx.focus_handle(),
            rail: false,
            demo_rail: false,
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
            btn_corners: None,
            heat_res: 3,
            heat_grad: 0,
            sh_gen: "neon_grid",
            sh_fx: vec!["crt"],
            sh_int: 1.0,
            sh_theme: false,
            sh_px: 2.0,
        }
    }

    pub fn cat_count(&self) -> usize {
        self.cat_count
    }

    pub fn set_cat(&mut self, cat: usize) {
        self.cat = cat;
    }
}

fn new_showcase(cx: &mut App) -> Entity<Showcase> {
    cx.new(Showcase::build)
}

fn main() {
    Application::new().run(|cx: &mut App| {
        // Debug: DODAI_THEME=<theme id> starts on that theme (e.g. material-dark).
        let first = std::env::var("DODAI_THEME")
            .ok()
            .and_then(|id| dodai_core::themes::by_id(&id))
            .unwrap_or_default();
        install(cx, first);
        window_shortcuts(cx);
        set_glow_intensity(cx, 0.5);
        if std::env::var_os("DODAI_NOSCAN").is_some() {
            set_scanlines(cx, false);
        }
        // Debug: DODAI_ACCENT=<edge index 0..7> pre-selects the accent line.
        if let Some(e) = std::env::var("DODAI_ACCENT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .and_then(|i| dodai_gpui::Corner::ALL.get(i).copied())
        {
            let b = dodai_gpui::section_border(cx);
            dodai_gpui::set_section_border(cx, b.accent(Some(e)));
        }
        // Debug: DODAI_SIZE=WxH opens a differently sized window (e.g. 1100x1400).
        let (dw, dh) = std::env::var("DODAI_SIZE")
            .ok()
            .and_then(|v| {
                let (w, h) = v.split_once('x')?;
                Some((w.parse().ok()?, h.parse().ok()?))
            })
            .unwrap_or((DEFAULT_W, DEFAULT_H));
        let bounds = Bounds::centered(None, size(px(dw), px(dh)), cx);
        let win = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(420.0), px(360.0))),
                    ..Default::default()
                },
                |_, cx| new_showcase(cx),
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
        Easing::Spring { .. }
        | Easing::Bounce
        | Easing::Elastic
        | Easing::Back(_)
        | Easing::Anticipate(_) => return None,
    })
}
