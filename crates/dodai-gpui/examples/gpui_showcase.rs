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
    BarChart, BoxPlot, BubbleChart, Candle, Candlestick, Date, DatePicker, EmptyState, Funnel,
    Gauge, Heatmap, Histogram, IconBadge, LineChart, Marker, NumberStepper, PieChart, RadarChart,
    RadialBar, RangeSlider, Rating, RichTooltip, ScatterPlot, SearchView, Series, Snackbar,
    Sparkline, StackedAreaChart, TagField, TextInput, TimePicker, Timeline, TimelineEvent, Toolbar,
    TreeItem, TreeView, Treemap, Waterfall,
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

struct Showcase {
    replay: usize,
    sec_replay: std::collections::HashMap<&'static str, u32>,
    /// Sections whose one-shot Play run is still in progress.
    sec_playing: std::collections::HashMap<&'static str, u32>,
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
    in_name: Entity<TextInput>,
    in_notes: Entity<TextInput>,
    cat: usize,
    cat_count: usize,
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

/// Which animation controls apply to a section (derived from the components it shows).
#[derive(Clone, Copy)]
struct Caps {
    /// Entry effects (fade / rise / drop / slide): any component wrapped by `motion::entry`.
    entry: bool,
    /// Value-in "Fill": charts, gauges, progress, sliders, knobs.
    fill: bool,
    /// Stagger: needs several animated components.
    stagger: bool,
}

fn caps(title: &str) -> Caps {
    let c = |entry, fill, stagger| Caps {
        entry,
        fill,
        stagger,
    };
    match title {
        // Shaders animate themselves; no entry/fill/stagger controls.
        t if t == "Shader playground"
            || t.ends_with(" shaders")
            || t.ends_with(" effects")
            || t.ends_with(" presets") =>
        {
            c(false, false, false)
        }
        // Raw / non-animated components only.
        "Tooltip" | "Input" | "Collapsible sidebar" | "Border" | "Glow · Scanlines" => {
            c(false, false, false)
        }
        // Single animated component.
        "Search view" | "Tag field" => c(true, false, false),
        // Value-in components.
        "Slider · Progress" | "Slider (stepped, ticks, value readout)" | "Range slider" => {
            c(true, true, true)
        }
        "Knob · Vertical sliders" => c(true, true, true),
        t if t.contains("chart")
            || [
                "Pie · donut · gauge · sparkline",
                "Area · histogram · waterfall",
                "Funnel · treemap · radial bar",
                "Candlestick · box plot · bubble",
                "Radar · scatter · heatmap",
            ]
            .contains(&t) =>
        {
            c(true, true, true)
        }
        _ => c(true, false, true),
    }
}

/// Subsections of a category (shown in the sidebar under the selected category).
fn subs(cat: &str) -> &'static [&'static str] {
    match cat {
        "Animation" => &["Entry", "Physics"],
        "Shaders" => &[
            "Playground",
            "Cyberpunk",
            "Material 3",
            "Matrix",
            "Retro terminals",
            "Pixel art",
            "Cosmic & nature",
            "Ghostty",
            "Ghibli",
            "DMT Hyperspace",
        ],
        _ => &[],
    }
}

/// Subsection a section belongs to.
fn sub_of(cat: &str, label: &str) -> Option<&'static str> {
    match cat {
        "Animation" => Some(
            if ["Spring", "Bounce", "Elastic", "Back"]
                .iter()
                .any(|p| label.starts_with(p))
            {
                "Physics"
            } else {
                "Entry"
            },
        ),
        "Shaders" => Some(
            dodai_gpui::dodai_shaders::Family::ALL
                .iter()
                .find(|f| label.starts_with(f.name()))
                .map_or("Playground", |f| f.name()),
        ),
        _ => None,
    }
}

fn section_id(title: &str) -> u32 {
    title
        .bytes()
        .fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32))
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
        "Shaders" | "Ghibli" | "DMT Hyperspace" => I::Sparkle,
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

        let hres = self.heat_res.clamp(1, 8);
        let (hr, hc) = (12 * hres, 32 * hres);
        let heat_cell = (264.0 / hr as f32).clamp(3.0, 22.0);
        let hm = me.clone();
        let heat_block = col()
            .flex_1()
            .min_w(px(480.0))
            .child(
                row()
                    .child(Label::new(format!("Resolution {hc}×{hr}")))
                    .child(
                        Button::new(
                            "heat-grad",
                            format!(
                                "Gradient: {}",
                                dodai_gpui::Gradient::ALL
                                    [self.heat_grad % dodai_gpui::Gradient::ALL.len()]
                                .name()
                            ),
                        )
                        .variant(ButtonVariant::Secondary)
                        .on_click({
                            let m = hm.clone();
                            move |_, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.heat_grad += 1;
                                    cx.notify();
                                })
                            }
                        }),
                    )
                    .child(
                        Button::new("heat-minus", "−")
                            .variant(ButtonVariant::Secondary)
                            .on_click({
                                let m = hm.clone();
                                move |_, _, cx| {
                                    m.update(cx, |s, cx| {
                                        s.heat_res = s.heat_res.saturating_sub(1).max(1);
                                        cx.notify();
                                    })
                                }
                            }),
                    )
                    .child(
                        Button::new("heat-plus", "+")
                            .variant(ButtonVariant::Secondary)
                            .on_click({
                                let m = hm.clone();
                                move |_, _, cx| {
                                    m.update(cx, |s, cx| {
                                        s.heat_res = (s.heat_res + 1).min(8);
                                        cx.notify();
                                    })
                                }
                            }),
                    ),
            )
            .child(
                Heatmap::new(
                    format!("heat-{hres}"),
                    (0..hr)
                        .map(|r| {
                            (0..hc)
                                .map(|c| {
                                    let (x, y) = (c as f32 / hres as f32, r as f32 / hres as f32);
                                    ((x * 0.21 * 2.0).sin()
                                        + (y * 0.45).cos()
                                        + ((x * 2.0 + y) * 0.13).sin() * 1.5)
                                        + ((x - 16.0).hypot((y - 6.0) * 2.0) * -0.16).exp() * 3.0
                                })
                                .collect()
                        })
                        .collect(),
                )
                .cell(heat_cell)
                .gradient(
                    dodai_gpui::Gradient::ALL[self.heat_grad % dodai_gpui::Gradient::ALL.len()],
                )
                .row_labels((0..hr).map(|r| {
                    if r % (4 * hres) == 0 {
                        format!("{:02}h", r / hres * 2)
                    } else {
                        String::new()
                    }
                }))
                .col_labels((0..hc).map(|c| {
                    if c % (8 * hres) == 0 {
                        format!("{}", c / hres)
                    } else {
                        String::new()
                    }
                })),
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
                        div().flex_1().min_w(px(220.0)).child(
                            Card::new(CardVariant::Elevated)
                                .headline("Elevated")
                                .supporting("Raised surface with hard shadow."),
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
                        div().flex_1().min_w(px(300.0)).child(
                            LineChart::new("line", [a.clone(), b.clone()])
                                .labels(months)
                                .area(true)
                                .smooth(true),
                        ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(300.0))
                            .child(BarChart::new("bars", [a.clone(), b.clone()]).labels(months)),
                    ),
            )
            .child(
                row()
                    .child(
                        div().flex_1().min_w(px(300.0)).child(
                            BarChart::new("stack", [a.clone(), b.clone()])
                                .labels(months)
                                .stacked(true),
                        ),
                    )
                    .child(
                        div().flex_1().min_w(px(300.0)).child(
                            BarChart::new("hbars", [a, b])
                                .labels(months)
                                .horizontal(true),
                        ),
                    ),
            );
        let charts_round = row()
            .justify_around()
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
            .child(CountUp::new("cu1", 12840.0).prefix("$ ").size(28.0))
            .child(
                CountUp::new("cu2", 98.6)
                    .decimals(1)
                    .suffix(" %")
                    .size(28.0),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(120.0))
                    .child(Sparkline::new([3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 12.0]).fluid(true)),
            )
            .child(
                div().flex_1().min_w(px(120.0)).child(
                    Sparkline::new([3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 12.0])
                        .marker(Marker::Diamond)
                        .fluid(true),
                ),
            )
            .child(
                div().flex_1().min_w(px(120.0)).child(
                    Sparkline::new([3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 12.0])
                        .marker(Marker::Square)
                        .area(false)
                        .fluid(true),
                ),
            )
            .child(
                div().flex_1().min_w(px(120.0)).child(
                    Sparkline::new([3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 12.0])
                        .marker(Marker::Triangle)
                        .all_points(true)
                        .fluid(true),
                ),
            );
        let w = |e: gpui::AnyElement, px_w: f32| {
            div()
                .flex_1()
                .min_w(px(px_w * 0.7))
                .child(e)
                .into_any_element()
        };
        let walk: Vec<f32> = (0..70)
            .scan(100.0f32, |p, i| {
                *p += ((i * 53 % 17) as f32 - 7.5) * 0.45;
                Some(*p)
            })
            .collect();
        let ohlc: Vec<Candle> = walk
            .windows(2)
            .enumerate()
            .map(|(i, p)| {
                let (o, c) = (p[0], p[1]);
                Candle::new(o, o.max(c) + 0.8, o.min(c) - 0.8, c)
                    .volume(50.0 + (i * 37 % 60) as f32)
            })
            .collect();
        let charts_market = row()
            .child(w(
                Candlestick::new("ext", ohlc.clone())
                    .style(CandleStyle::HeikinAshi)
                    .volume(true)
                    .overlay(Overlay::Sma(10))
                    .overlay(Overlay::Bollinger(20, 2.0))
                    .pane(Pane::Rsi(14))
                    .pane(Pane::Macd(12, 26, 9))
                    .markers([
                        TradeMarker::buy(20, ohlc[20].low),
                        TradeMarker::sell(45, ohlc[45].high),
                    ])
                    .height(240.0)
                    .title("Heikin-Ashi · volume · SMA · Bollinger · RSI · MACD")
                    .into_any_element(),
                520.0,
            ))
            .child(w(
                RenkoChart::new("renko", walk.clone(), 2.0)
                    .title("Renko · 2.0 bricks")
                    .into_any_element(),
                420.0,
            ))
            .child(w(
                DepthChart::new(
                    "depth",
                    (1..12).map(|i| (100.0 - i as f32 * 0.5, 4.0 + (i * 7 % 5) as f32)),
                    (1..12).map(|i| (100.5 + i as f32 * 0.5, 3.0 + (i * 5 % 6) as f32)),
                )
                .title("Order book depth")
                .into_any_element(),
                420.0,
            ))
            .child(w(
                Gantt::new(
                    "gantt",
                    [
                        GanttTask::new("Design", 0.0, 10.0).progress(1.0),
                        GanttTask::new("Build", 8.0, 24.0).progress(0.6).after(0),
                        GanttTask::new("Test", 20.0, 30.0).progress(0.25).after(1),
                        GanttTask::new("Ship", 30.0, 34.0).after(2),
                    ],
                )
                .title("Release plan")
                .format(|v| format!("d{v:.0}"))
                .into_any_element(),
                420.0,
            ))
            .child(w(
                LineChart::new(
                    "dual",
                    [Series::new("Revenue", [12.0, 18.0, 15.0, 26.0, 22.0, 31.0])],
                )
                .secondary([Series::new("Margin %", [0.21, 0.24, 0.2, 0.3, 0.28, 0.33])])
                .labels(months)
                .title("Secondary y axis")
                .into_any_element(),
                420.0,
            ))
            .child(w(
                ChartStack::new()
                    .height(300.0)
                    .pane(3.0, |h| {
                        LineChart::new(
                            "st1",
                            [Series::new("Price", [3.0, 4.0, 3.5, 5.0, 6.0, 5.5])],
                        )
                        .height(h - 24.0)
                        .legend(false)
                        .into_any_element()
                    })
                    .pane(1.0, |h| {
                        BarChart::new("st2", [Series::new("Vol", [2.0, 3.0, 1.0, 4.0, 5.0, 3.0])])
                            .height(h - 24.0)
                            .legend(false)
                            .into_any_element()
                    })
                    .into_any_element(),
                420.0,
            ));
        let charts_misc = row()
            .justify_around()
            .child(RadarChart::new(
                ["Speed", "Power", "Range", "Armor", "Stealth"],
                [
                    Series::new("Unit A", [8.0, 6.0, 7.0, 4.0, 9.0]),
                    Series::new("Unit B", [5.0, 9.0, 4.0, 8.0, 3.0]),
                ],
            ))
            .child(
                div().flex_1().min_w(px(300.0)).child(ScatterPlot::new(
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
            .child(heat_block);

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
        let anim_defs: [(&'static str, EntryKind, Easing, &'static str); 12] = [
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
            (
                "Spring · stiff",
                EntryKind::Rise,
                Easing::SPRING_STIFF,
                "Stiffness 400, damping 26: snaps into place with a tiny overshoot. Used by the Toggle thumb.",
            ),
            (
                "Spring · wobbly",
                EntryKind::Rise,
                Easing::SPRING_WOBBLY,
                "Stiffness 120, damping 6: loose, lingering wobble. Playful; use sparingly.",
            ),
            (
                "Spring · gentle",
                EntryKind::Drop,
                Easing::SPRING_GENTLE,
                "Stiffness 100, damping 20: heavily damped, no overshoot, soft landing.",
            ),
            (
                "Bounce",
                EntryKind::Drop,
                Easing::Bounce,
                "Ball drop: falls to the target and bounces to rest. Fits dropped-in toasts and badges.",
            ),
            (
                "Elastic",
                EntryKind::SlideLeft,
                Easing::Elastic,
                "Rubber band: shoots past the target and wobbles with a decaying sine.",
            ),
            (
                "Back · anticipate",
                EntryKind::SlideRight,
                Easing::Back(1.7),
                "Back overshoots the target about 10% and eases back; Anticipate (the mirror) winds up first by pulling away before moving.",
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
                .w_full()
                .min_w(px(220.0))
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
                            .child(col().flex_1().min_w(px(240.0)).child(demo).child(
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
                        use dodai_gpui::{BorderStyle as B, Edge};
                        let mut r = row();
                        for e in Edge::ALL {
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
                        use dodai_gpui::{BorderStyle as B, Edge};
                        let cur = dodai_gpui::section_border(cx);
                        let mut r = row().child(Label::new("Accent line (all sections)"));
                        let mut opts: Vec<Option<Edge>> = vec![None];
                        opts.extend(Edge::ALL.map(Some));
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

        // ── Shaders ────────────────────────────────────────────────────────
        {
            use dodai_gpui::dodai_shaders::{catalog, Family, Kind};
            use dodai_gpui::{PaletteSource, ShaderView};
            let pal_src = if self.sh_theme {
                PaletteSource::Theme
            } else {
                PaletteSource::Family
            };
            let card =
                |id: String, shader: &'static dodai_gpui::dodai_shaders::ShaderInfo, fx: bool| {
                    let view = if fx {
                        ShaderView::fx_over_default(id.clone(), shader.id)
                    } else {
                        ShaderView::new(id.clone(), shader.id)
                    };
                    col()
                        .flex_1()
                        .min_w(px(220.0))
                        .child(view.palette(pal_src).pixel(2.0).rounded(t.shape.radius))
                        .child(row().child(Mono::new(shader.name)).child(if fx {
                            Tag::new("effect", Tone::Warning)
                        } else {
                            Tag::new("source", Tone::Info)
                        }))
                        .child(Muted::new(shader.desc))
                        .into_any_element()
                };
            // Playground: pick a generator, stack effects, tune.
            let me_p = me.clone();
            let gens: Vec<_> = catalog()
                .iter()
                .filter(|s| matches!(s.kind, Kind::Generator(_)))
                .collect();
            let fxs: Vec<_> = catalog()
                .iter()
                .filter(|s| matches!(s.kind, Kind::PostFx(_)))
                .collect();
            let opt_label = |s: &dodai_gpui::dodai_shaders::ShaderInfo| {
                format!("{} · {}", s.family.name(), s.name)
            };
            let gen_ids: Vec<&'static str> = gens.iter().map(|g| g.id).collect();
            let gen_sel = gen_ids.iter().position(|i| *i == self.sh_gen);
            let m_g = me_p.clone();
            let gen_row = row()
                .child(Tag::new("SOURCE", Tone::Info).solid(true))
                .child(
                    div().flex_1().min_w(px(200.0)).max_w(px(380.0)).child(
                        dodai_gpui::Select::new("sh-gen")
                            .options(gens.iter().map(|g| opt_label(g)))
                            .selected(gen_sel)
                            .on_change(move |i, _, cx| {
                                let id = gen_ids[i];
                                m_g.update(cx, |s, cx| {
                                    s.sh_gen = id;
                                    cx.notify();
                                })
                            }),
                    ),
                )
                .child(Muted::new(format!(
                    "{} sources: draw the picture",
                    gens.len()
                )));
            let fx_ids: Vec<&'static str> = fxs.iter().map(|f| f.id).collect();
            let m_f = me_p.clone();
            let mut fx_row = row()
                .child(Tag::new("EFFECT", Tone::Warning).solid(true))
                .child(
                    div().flex_1().min_w(px(200.0)).max_w(px(380.0)).child(
                        dodai_gpui::Select::new("sh-fx")
                            .placeholder("Add an effect…")
                            .options(fxs.iter().map(|f| opt_label(f)))
                            .selected(None)
                            .on_change(move |i, _, cx| {
                                let id = fx_ids[i];
                                m_f.update(cx, |s, cx| {
                                    if !s.sh_fx.contains(&id) {
                                        s.sh_fx.push(id);
                                    }
                                    cx.notify();
                                })
                            }),
                    ),
                )
                .child(Muted::new(format!(
                    "{} effects: filter the picture, stacked in order",
                    fxs.len()
                )));
            // The active chain: click a chip to remove that effect.
            let mut chain_row = row().child(Muted::new("Chain"));
            chain_row = chain_row.child(Tag::new(
                gens.iter()
                    .find(|g| g.id == self.sh_gen)
                    .map_or("?", |g| g.name),
                Tone::Info,
            ));
            for (ci, id) in self.sh_fx.iter().enumerate() {
                let (id, m) = (*id, me_p.clone());
                let name = dodai_gpui::dodai_shaders::by_id(id).map_or(id, |s| s.name);
                chain_row = chain_row.child(Muted::new("→")).child(
                    Button::new(("shx", ci), format!("{name} ✕"))
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, _, cx| {
                            m.update(cx, |s, cx| {
                                s.sh_fx.retain(|x| *x != id);
                                cx.notify();
                            })
                        }),
                );
            }
            if !self.sh_fx.is_empty() {
                let m = me_p.clone();
                chain_row = chain_row.child(
                    Button::new("shx-clear", "Clear effects")
                        .variant(ButtonVariant::Secondary)
                        .on_click(move |_, _, cx| {
                            m.update(cx, |s, cx| {
                                s.sh_fx.clear();
                                cx.notify();
                            })
                        }),
                );
            }
            let _ = &mut fx_row;
            let mut big = ShaderView::new("sh-play", self.sh_gen)
                .palette(pal_src)
                .pixel(self.sh_px)
                .intensity(self.sh_int)
                .rounded(t.shape.radius);
            for f in &self.sh_fx {
                big = big.fx(f);
            }
            let (m1, m2, m3, m4, m5) = (me.clone(), me.clone(), me.clone(), me.clone(), me.clone());
            let tune = row()
                .child(Label::new(format!("Intensity {:.0}%", self.sh_int * 100.0)))
                .child(
                    Button::new("shi-", "−")
                        .variant(ButtonVariant::Secondary)
                        .on_click(move |_, _, cx| {
                            m1.update(cx, |s, cx| {
                                s.sh_int = (s.sh_int - 0.1).max(0.0);
                                cx.notify();
                            })
                        }),
                )
                .child(
                    Button::new("shi+", "+")
                        .variant(ButtonVariant::Secondary)
                        .on_click(move |_, _, cx| {
                            m2.update(cx, |s, cx| {
                                s.sh_int = (s.sh_int + 0.1).min(1.0);
                                cx.notify();
                            })
                        }),
                )
                .child(Label::new(format!("Pixel {:.0}px", self.sh_px)))
                .child(
                    Button::new("shp-", "−")
                        .variant(ButtonVariant::Secondary)
                        .on_click(move |_, _, cx| {
                            m3.update(cx, |s, cx| {
                                s.sh_px = (s.sh_px - 1.0).max(1.0);
                                cx.notify();
                            })
                        }),
                )
                .child(
                    Button::new("shp+", "+")
                        .variant(ButtonVariant::Secondary)
                        .on_click(move |_, _, cx| {
                            m4.update(cx, |s, cx| {
                                s.sh_px = (s.sh_px + 1.0).min(10.0);
                                cx.notify();
                            })
                        }),
                )
                .child(
                    Button::new(
                        "sh-theme",
                        if self.sh_theme {
                            "Colours: design system"
                        } else {
                            "Colours: shader family"
                        },
                    )
                    .variant(ButtonVariant::Outline)
                    .on_click(move |_, _, cx| {
                        m5.update(cx, |s, cx| {
                            s.sh_theme = !s.sh_theme;
                            cx.notify();
                        })
                    }),
                );
            sections.push((
                "Shaders",
                "Playground",
                section!("Shader playground")
                    .child(Muted::new(format!(
                        "Pure-Rust shaders from `dodai-shaders`, rendered on: {}. Pick a source, stack effects.",
                        dodai_gpui::shader_backend()
                    )))
                    .child(gen_row)
                    .child(fx_row)
                    .child(chain_row)
                    .child(tune)
                    .child(div().w_full().max_w(px(960.0)).child(big))
                    .into_any_element(),
                "Shader playground",
            ));
            const TITLES: [(&str, &str); 9] = [
                ("Cyberpunk shaders", "Cyberpunk effects"),
                ("Material 3 shaders", "Material 3 effects"),
                ("Matrix shaders", "Matrix effects"),
                ("Retro terminals shaders", "Retro terminals effects"),
                ("Pixel art shaders", "Pixel art effects"),
                ("Cosmic & nature shaders", "Cosmic & nature effects"),
                ("Ghostty shaders", "Ghostty effects"),
                ("Ghibli shaders", "Ghibli effects"),
                ("DMT Hyperspace shaders", "DMT Hyperspace effects"),
            ];
            for (fi, fam) in Family::ALL.iter().enumerate() {
                let gens: Vec<_> = catalog()
                    .iter()
                    .filter(|s| s.family == *fam && s.is_generator())
                    .collect();
                let effs: Vec<_> = catalog()
                    .iter()
                    .filter(|s| s.family == *fam && !s.is_generator())
                    .collect();
                let (gt, et) = TITLES[fi];
                // Ghibli and DMT have their own category and also live under Shaders.
                let cats: &[&'static str] = match *fam {
                    Family::Ghibli => &["Ghibli", "Shaders"],
                    Family::Dmt => &["DMT Hyperspace", "Shaders"],
                    _ => &["Shaders"],
                };
                for cat in cats {
                    let mut g_row = row();
                    for g in &gens {
                        g_row = g_row.child(card(format!("shc-{cat}-{}", g.id), g, false));
                    }
                    sections.push((*cat, gt, section!(gt).child(g_row).into_any_element(), gt));
                    if !effs.is_empty() {
                        let mut e_row = row();
                        for e in &effs {
                            e_row = e_row.child(card(format!("shc-{cat}-{}", e.id), e, true));
                        }
                        sections.push((*cat, et, section!(et).child(e_row).into_any_element(), et));
                    }
                }
            }
            // Ready-made chains.
            for (title, list) in [
                (
                    "Retro terminals presets",
                    dodai_gpui::dodai_shaders::presets::RETRO,
                ),
                (
                    "Ghostty presets",
                    dodai_gpui::dodai_shaders::presets::GHOSTTY,
                ),
                (
                    "Blade Runner presets",
                    dodai_gpui::dodai_shaders::presets::BLADE,
                ),
                ("Ghibli presets", dodai_gpui::dodai_shaders::presets::GHIBLI),
                (
                    "DMT Hyperspace presets",
                    dodai_gpui::dodai_shaders::presets::DMT,
                ),
                (
                    "Pixel art presets",
                    dodai_gpui::dodai_shaders::presets::PIXEL,
                ),
            ] {
                let cats: &[&'static str] = match title {
                    "Ghibli presets" => &["Ghibli", "Shaders"],
                    "DMT Hyperspace presets" => &["DMT Hyperspace", "Shaders"],
                    _ => &["Shaders"],
                };
                for cat in cats {
                    let mut r = row();
                    for p in list {
                        r = r.child(
                            col()
                                .flex_1()
                                .min_w(px(220.0))
                                .child(
                                    ShaderView::preset(format!("shp-{cat}-{}", p.id), p)
                                        .pixel(2.0)
                                        .rounded(t.shape.radius),
                                )
                                .child(Mono::new(p.name))
                                .child(Muted::new(p.desc)),
                        );
                    }
                    sections.push((
                        *cat,
                        title,
                        section!(title).child(r).into_any_element(),
                        title,
                    ));
                }
            }
        }
        for (i, el) in anim_sections.into_iter().enumerate() {
            let k = anim_titles[i];
            sections.push(("Animation", k, el, k));
        }

        // Shaders are the biggest category and design-independent: always last.
        sections.sort_by_key(|(c, _, _, _)| match *c {
            "Ghibli" => 1,
            "DMT Hyperspace" => 2,
            "Shaders" => 3,
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
        let rail = self.rail;
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
                    let cnt = sections_meta
                        .iter()
                        .filter(|(c, l)| *c == name && sub.is_none_or(|x| sub_of(c, l) == Some(x)))
                        .count();
                    side = side.child(
                        div().pl(px(t.spacing.lg)).child(
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
                            .selected(self.sub == sub)
                            .on_click(move |_, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.sub = sub;
                                    dodai_gpui::set_tree_expanded(cx, true);
                                    s.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                                    cx.notify()
                                })
                            }),
                        ),
                    );
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

        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.lg + 8.0))
            .min_w_0()
            .p(px(if narrow { t.spacing.md } else { t.spacing.xl }))
            .child(ds_panel);
        let mut shown = 0;
        let mut last_cat = "";
        let mut last_sub: Option<&str> = None;
        for (c, title, el, _key) in sections {
            // Loading demos repaint every frame forever: own page only.
            let in_cat = selected_cat.map_or(c != "Loading", |sc| sc == c);
            let hit =
                query.is_empty() || !fuzzy_filter(&query, &[format!("{c} {title}")]).is_empty();
            let only = std::env::var("DODAI_ONLY").is_ok_and(|o| !title.contains(&o));
            let sub_ok = self.sub.is_none_or(|x| sub_of(c, title) == Some(x));
            if !(in_cat && hit && sub_ok) || only {
                continue;
            }
            if c != last_cat {
                body = body.child(Heading::new(c).level(2));
                last_cat = c;
                last_sub = None;
            }
            if let Some(sb) = sub_of(c, title) {
                if last_sub != Some(sb) {
                    body = body.child(Heading::new(sb).level(3));
                    last_sub = Some(sb);
                }
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
                    .sidebar_width(if narrow { 170.0 } else { 240.0 })
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
                    .content(
                        div()
                            .id("scroll")
                            .track_scroll(&self.scroll)
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .overflow_y_scroll()
                            .child(body.id(("body", self.cat * 1000 + self.replay))),
                    ),
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

fn main() {
    Application::new().run(|cx: &mut App| {
        install(cx, Theme::default());
        window_shortcuts(cx);
        set_glow_intensity(cx, 0.5);
        // Debug: DODAI_ACCENT=<edge index 0..7> pre-selects the accent line.
        if let Some(e) = std::env::var("DODAI_ACCENT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .and_then(|i| dodai_gpui::Edge::ALL.get(i).copied())
        {
            let b = dodai_gpui::section_border(cx);
            dodai_gpui::set_section_border(cx, b.accent(Some(e)));
        }
        let bounds = Bounds::centered(None, size(px(DEFAULT_W), px(DEFAULT_H)), cx);
        let win = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(420.0), px(360.0))),
                    ..Default::default()
                },
                |_, cx| {
                    let search = cx.new(|cx| TextInput::new(cx).placeholder("Search…"));
                    let filter = cx.new(|cx| TextInput::new(cx).placeholder("Filter components…"));
                    let in_name = cx.new(|cx| TextInput::new(cx).placeholder("handle…"));
                    let in_notes = cx.new(|cx| {
                        TextInput::new(cx)
                            .multiline(4)
                            .placeholder("notes (multi-line)…")
                    });
                    cx.new(|cx| Showcase {
                        filter,
                        search,
                        in_name,
                        in_notes,
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
                        sec_kind: Default::default(),
                        pill_hover: None,
                        pop_hover: None,
                        cat_count: 1,
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
        Easing::Spring { .. }
        | Easing::Bounce
        | Easing::Elastic
        | Easing::Back(_)
        | Easing::Anticipate(_) => return None,
    })
}
