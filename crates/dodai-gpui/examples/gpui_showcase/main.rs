//! `cargo run -p dodai-gpui --example gpui_showcase`
//!
//! Every dodai-gpui component in one scrollable window.
//! Keys: `t` / `T` next / previous theme. Glow +/- buttons change
//! `Theme::with_glow_intensity` live.

use dodai_core::{ButtonVariant, Theme, Tone};
use dodai_gpui::{
    cycle_next, cycle_prev, fuzzy_filter, glow_intensity, hsla, set_glow_intensity, set_scanlines,
    theme_of, Accordion, BottomSheet, Breadcrumb, BubblePlacement, Button, ButtonGroup, Card,
    CardVariant, Carousel, Checkbox, DesignSystemTree, Dialog, DialogCard, Divider, DodaiStyled,
    DodaiTooltip, Fab, FabSize, Heading, IconButton, InputBox, Knob, Label, ListItem,
    LoadingIndicator, Mono, Muted, Orientation, Pagination, Panel, ProgressBar, Radio,
    ResponsiveShell, SearchBar, Segmented, SideSheet, Slider, SplitButton, Stepper, Tabs, Tag,
    Toggle, TopAppBar,
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
    CandleOverlay, CandlePane, CandleStyle, ChartStack, DepthChart, Gantt, GanttTask, RenkoChart,
    TradeMarker,
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
mod impl_src;
mod motion_page;
mod overview;
mod sections;
mod shaders_page;
mod sidebar;
mod snippets;
use catalog::{caps, cat_blurb, cat_icon, section_id, sub_of, subs};
use charts_page::ChartDemos;
use foundations::{
    frosted_view, layout_view, modifier_variants_view, motion_tokens_view, palette_view,
    shape_tokens_view, spacing_elevation_view, type_scale_view, type_view,
};
pub(crate) use shaders_page::Section;

pub struct Showcase {
    replay: usize,
    sec_replay: std::collections::HashMap<&'static str, u32>,
    /// Sections whose one-shot Play run is still in progress.
    sec_playing: std::collections::HashMap<&'static str, u32>,
    sec_mode: std::collections::HashMap<&'static str, Player>,
    /// Sections whose "</> Code" panel is open.
    code_open: std::collections::HashSet<&'static str>,
    /// Which component's implementation (or the usage snippet) each open code panel shows.
    code_pick: std::collections::HashMap<&'static str, usize>,
    loop_gen: u32,
    ld_pb: dodai_gpui::Playback,
    ld_ms: u16,
    scroll: gpui::ScrollHandle,
    /// Last measured height of each section (spacer height while it is culled).
    heights: std::collections::HashMap<&'static str, f32>,
    /// Sections laid out in the previous frame (their measured height is real).
    realised: std::collections::HashSet<&'static str>,
    /// Sections measured at the current width.
    fresh: std::collections::HashSet<&'static str>,
    /// Width the heights were measured at.
    width: f32,
    /// Section title to scroll to (plus frames to wait for a new category to lay out).
    jump: Option<(&'static str, u8)>,
    sec_kind: std::collections::HashMap<&'static str, EntrySet>,
    pill_hover: Option<(&'static str, u8)>,
    pop_hover: Option<(&'static str, u8)>,
    filter: Entity<TextInput>,
    search: Entity<TextInput>,
    search_full: Entity<TextInput>,
    in_name: Entity<TextInput>,
    in_notes: Entity<TextInput>,
    in_outlined: Entity<TextInput>,
    in_filled: Entity<TextInput>,
    pw: Entity<TextInput>,
    dt_filter: Entity<TextInput>,
    icon_search: Entity<TextInput>,
    icon_filled: bool,
    /// The icon whose SVG and code the Icons section shows.
    icon_sel: Option<dodai_gpui::IconId>,
    icon_code: bool,
    icon_msg: Option<String>,
    /// Material icon categories shown in full (the rest show a preview).
    icon_open: std::collections::HashSet<&'static str>,
    date_range: dodai_gpui::DateRange,
    cat: usize,
    cat_count: usize,
    /// `(category, title)` of every section, built once; drives the sidebar.
    meta: Vec<(&'static str, &'static str)>,
    /// The design family `meta` was built for (sections differ per design).
    meta_family: &'static str,
    spy: Vec<(&'static str, usize)>,
    /// Click-pinned current section (see `dodai_gpui::ScrollSpy`).
    pin: dodai_gpui::ScrollSpy<&'static str>,
    /// Selected subsection inside the current category (`None` = all).
    sub: Option<&'static str>,
    side_focus: FocusHandle,
    side_scroll: gpui::ScrollHandle,
    /// Material-style stretch past the ends of the page / sidebar.
    /// Sidebar collapsed to an icon rail.
    rail: bool,
    /// Collapsed state of the Navigation-section demo shell.
    demo_rail: bool,
    rating: f32,
    /// Selected item of the showcase button groups.
    group_sel: usize,
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
            .unwrap_or(dodai_gpui::motion::current(cx).entry)
    }

    /// Replay one section only (re-keys just its subtree).
    fn bump(&mut self, title: &'static str) {
        *self.sec_replay.entry(title).or_insert(0) += 1;
    }

    /// Total time of one run of the section: longest of the entry effects and
    /// the value-in (fill) animation, plus the stagger across its items.
    fn run_len(&self, title: &'static str, cx: &gpui::App) -> Duration {
        let m = dodai_gpui::motion::current(cx);
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
        if mode == Player::Stopped {
            // Stop shows its final state, then dismisses itself after two seconds.
            let gen = self.loop_gen;
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(Duration::from_secs(2)).await;
                let _ = this.update(cx, |s, cx| {
                    if s.sec_mode.get(title) == Some(&Player::Stopped) && s.loop_gen == gen {
                        s.sec_mode.remove(title);
                        cx.notify();
                    }
                });
            })
            .detach();
        }
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
        let base = dodai_gpui::motion::current(cx).entry;
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

impl Showcase {
    /// A section plus, when it has a usage snippet, a code-icon toggle under it (closed by default).
    fn with_code(
        &self,
        title: &'static str,
        key: Option<&'static str>,
        el: gpui::AnyElement,
        t: &Theme,
        me: &gpui::Entity<Showcase>,
    ) -> gpui::AnyElement {
        // What the panel can show: the implementation of each component the section uses, then
        // (when there is one) the usage snippet.
        let comps = impl_src::components(key.unwrap_or(title));
        let usage = key.and_then(snippets::snippet);
        let mut tabs: Vec<String> = comps.clone();
        if usage.is_some() {
            tabs.push("Usage".into());
        }
        if tabs.is_empty() {
            return el;
        }
        let pick = self
            .code_pick
            .get(title)
            .copied()
            .unwrap_or(0)
            .min(tabs.len() - 1);
        let code = match comps.get(pick) {
            Some(name) => impl_src::implementation(name).unwrap_or_default(),
            None => usage.unwrap_or_default(),
        };
        let open = self.code_open.contains(title) || std::env::var_os("DODAI_CODE").is_some();
        let me = me.clone();
        let tid = section_id(title);
        let toggle = code_toggle(("code-toggle", tid), t, open, {
            let me = me.clone();
            move |_, _, cx| {
                me.update(cx, |s, cx| {
                    if !s.code_open.remove(title) {
                        s.code_open.insert(title);
                    }
                    cx.notify();
                })
            }
        });
        let max_h = t.typography.size_md * 26.0;
        div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm - t.spacing.xs / 2.0))
            .child(el)
            .child(div().flex().justify_end().child(toggle))
            .when(open, |d| {
                d.when(tabs.len() > 1, |d| {
                    let me = me.clone();
                    d.child(
                        div().flex().child(
                            dodai_gpui::Segmented::new(("code-tabs", tid), tabs.clone())
                                .selected(pick)
                                .on_select(move |i, _, cx| {
                                    me.update(cx, |s, cx| {
                                        s.code_pick.insert(title, i);
                                        cx.notify();
                                    })
                                }),
                        ),
                    )
                })
                .child(
                    div()
                        .id(SharedString::from(format!("code-scroll-{tid}")))
                        .max_h(px(max_h))
                        .overflow_y_scroll()
                        .child(
                            dodai_gpui::CodeBlock::new(
                                SharedString::from(format!("snippet-{tid}-{pick}")),
                                code,
                            )
                            .language("rust"),
                        ),
                )
            })
            .into_any_element()
    }
}

/// The "Show code" switch every section uses (on = code visible).
pub(crate) fn code_toggle(
    id: impl Into<gpui::ElementId>,
    t: &Theme,
    open: bool,
    on_change: impl Fn(bool, &mut Window, &mut gpui::App) + 'static,
) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .text_size(px(11.0))
        .text_color(hsla(t.palette.text_muted))
        .child(dodai_gpui::Icon::Code.size(14.0))
        .child("Show code")
        .child(dodai_gpui::Toggle::new(id, open).on_change(on_change))
}

impl Render for Showcase {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _prof = Prof::start();
        let narrow = window.viewport_size().width < px(760.0);
        let t = theme_of(cx);
        let me = cx.entity();

        // Global shape properties (glow, scanlines, cut corners): Shape category.

        // ── new components ─────────────────────────────────────────────

        // Stepped / configurable sliders.

        let family = theme_of(cx).meta.family;
        if self.meta.is_empty() || self.meta_family != family {
            self.meta = self.section_meta(cx);
            self.meta_family = family;
        }
        let meta = self.meta.clone();
        let cats: Vec<&'static str> = {
            let mut v: Vec<&'static str> = Vec::new();
            for (c, _) in &meta {
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
            meta.iter()
                .filter(|(sc, title)| {
                    c.is_none_or(|c| c == *sc)
                        && (query.is_empty()
                            || !fuzzy_filter(&query, &[format!("{sc} {title}")]).is_empty())
                })
                .count()
        };
        let sections_meta = &meta;
        // Scroll-spy: the last section whose top has reached the viewport top
        // (positions come from the previous frame's layout).
        let active: Option<&'static str> = self.pin.current(&self.scroll, &self.spy, SPY_SLACK);
        let (side, side_need) = self.sidebar(
            &me,
            &cats,
            sections_meta,
            active,
            &query,
            narrow,
            count_in,
            cx,
        );

        let overview = self.cat == 0
            && self.sub.is_none()
            && query.is_empty()
            && std::env::var_os("DODAI_ONLY").is_none();
        // Loading demos repaint every frame forever: own page only.
        let in_cat = |c: &str| {
            if overview {
                !matches!(c, "Loading" | "Shaders")
            } else {
                selected_cat.map_or(c != "Loading", |sc| sc == c)
            }
        };
        // Build only the categories on screen.
        let mut sections: Vec<Section> = Vec::new();
        for c in &cats {
            if in_cat(c) {
                sections.extend(self.sections_of(c, cx));
            }
        }
        let mut body = div()
            .id(("body", self.cat * 1000 + self.replay))
            .track_scroll(&self.scroll)
            .relative()
            // Scroll-spy reads layout, so repaint on scroll even on static pages.
            .on_scroll_wheel(cx.listener(|s, _: &gpui::ScrollWheelEvent, _, cx| {
                s.pin.release();
                cx.notify()
            }))
            .flex()
            .flex_col()
            .flex_1()
            .gap(px(t.spacing.lg + 8.0))
            .min_w_0()
            .overflow_y_scroll()
            .p(px(if narrow { t.spacing.md } else { t.spacing.xl }));
        let mut n_children = 0usize;
        let mut jump_ix: Option<usize> = None;
        let jump_title = self.jump.map(|(j, _)| j);
        if overview {
            let kids = self.overview(&me, &cats, sections_meta, cx);
            n_children += kids.len();
            body = body.children(kids);
        }
        if std::env::var_os("DODAI_LIST").is_some() {
            for (c, t) in sections_meta {
                eprintln!("{c}\t{t}");
            }
            std::process::exit(0);
        }
        // Pass 1: which sections are shown, and where they sit among the children.
        let mut shown_secs: Vec<(&'static str, &'static str, gpui::AnyElement)> = Vec::new();
        let mut keys: std::collections::HashMap<&'static str, &'static str> = Default::default();
        for (c, title, el, key) in sections {
            keys.insert(title, key);
            let hit =
                query.is_empty() || !fuzzy_filter(&query, &[format!("{c} {title}")]).is_empty();
            let only = std::env::var("DODAI_ONLY").is_ok_and(|o| !title.contains(&o));
            let sub_ok = self.sub.is_none_or(|x| sub_of(c, title) == Some(x));
            if in_cat(c) && hit && sub_ok && !only {
                shown_secs.push((c, title, el));
            }
        }
        let mut order: Vec<(&'static str, usize)> = Vec::new();
        {
            let (mut n, mut lc, mut ls) = (n_children, "", None);
            for (c, title, _) in &shown_secs {
                if *c != lc {
                    n += 1;
                    lc = c;
                    ls = None;
                }
                if let Some(sb) = sub_of(c, title) {
                    if ls != Some(sb) {
                        n += 1;
                        ls = Some(sb);
                    }
                }
                order.push((title, n));
                n += 1;
            }
        }
        // Virtualisation: only sections near the viewport are laid out and painted;
        // the rest are fixed-height spacers (last measured height). Layout of the
        // whole page costs tens of ms, so this keeps scrolling and hovering cheap.
        let vp = self.scroll.bounds();
        let vp_top: f32 = vp.origin.y.into();
        let vp_h: f32 = vp.size.height.into();
        let valid = vp_h > 0.0 && order == self.spy;
        let margin = (vp_h * 0.5).max(300.0);
        // Child bounds are unscrolled layout positions; add the live offset.
        let shift: f32 = self.scroll.offset().y.into();
        let prev_real = std::mem::take(&mut self.realised);
        // Heights depend on the width (text wraps): re-measure after a resize.
        let vp_w: f32 = vp.size.width.into();
        if (vp_w - self.width).abs() > 1.0 {
            self.width = vp_w;
            self.fresh.clear();
        }
        let avg_h = if self.heights.is_empty() {
            420.0
        } else {
            self.heights.values().sum::<f32>() / self.heights.len() as f32
        };
        // Sections below the viewport get measured a few per frame, so spacer
        // heights become exact without one big hitch.
        let mut budget = 3;
        let mut unmeasured = false;
        // Heights that changed above the viewport shift everything: compensate.
        let mut delta_above = 0.0f32;
        let mut shown = 0;
        let mut last_cat = "";
        let mut last_sub: Option<&str> = None;
        let mut spy: Vec<(&'static str, usize)> = Vec::new();
        for (seq, (c, title, el)) in shown_secs.into_iter().enumerate() {
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
            let known = self.heights.get(title).copied();
            let bounds = if valid {
                self.scroll.bounds_for_item(n_children)
            } else {
                None
            };
            if let Some(b) = bounds.filter(|_| prev_real.contains(title)) {
                let h: f32 = b.size.height.into();
                let bottom = f32::from(b.origin.y) + shift + h;
                if let Some(old) = known {
                    if bottom <= vp_top && (h - old).abs() > 0.5 && !self.fresh.contains(title) {
                        delta_above += h - old;
                    }
                }
                self.heights.insert(title, h);
                self.fresh.insert(title);
            }
            let stale = !self.fresh.contains(title);
            unmeasured |= stale;
            let realise = match bounds {
                Some(b) => {
                    let top = f32::from(b.origin.y) + shift;
                    let bottom = top + f32::from(b.size.height);
                    let near = bottom > vp_top - margin && top < vp_top + vp_h + margin;
                    let ahead = !near && stale && top >= vp_top + vp_h && budget > 0;
                    if ahead {
                        budget -= 1;
                    }
                    near || ahead
                }
                // No usable layout yet: the first few, and anything never measured
                // while jumping (so the jump lands on true positions).
                None => seq < 4 || (known.is_none() && self.jump.is_some()),
            };
            shown += 1;
            n_children += 1;
            if realise || std::env::var_os("DODAI_NOCULL").is_some() {
                self.realised.insert(title);
                body = body.child(self.with_code(title, keys.get(title).copied(), el, &t, &me));
            } else {
                body = body.child(div().flex_none().w_full().h(px(known.unwrap_or(avg_h))));
            }
        }
        self.spy = spy;
        if delta_above != 0.0 {
            let o = self.scroll.offset();
            self.scroll
                .set_offset(gpui::point(o.x, o.y - px(delta_above)));
        }
        if unmeasured {
            window.request_animation_frame();
        }
        // Perf probe: redraw every frame so the cost of the visible sections shows up.
        if std::env::var_os("DODAI_BENCH").is_some() {
            window.request_animation_frame();
        }
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

        // Debug probe: DODAI_TABS=n moves the keyboard focus n stops once, after a few frames
        // (shows the focus ring without needing synthetic key events).
        if let Some(n) = std::env::var("DODAI_TABS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
        {
            thread_local! { static F: std::cell::Cell<u32> = const { std::cell::Cell::new(0) }; }
            let f = F.with(|f| {
                f.set(f.get() + 1);
                f.get()
            });
            if f == 20 {
                for _ in 0..n {
                    window.focus_next();
                }
            }
            if f < 40 {
                window.request_animation_frame();
            }
        }
        if std::env::var_os("DODAI_AUTOSCROLL").is_some() {
            // Perf probe: sweep the scroll position every frame.
            thread_local! {
                static T0: std::time::Instant = std::time::Instant::now();
            }
            let t = (T0.with(|t| t.elapsed().as_secs_f32()) * 0.25) % 2.0;
            let tri = if t < 1.0 { t } else { 2.0 - t };
            self.scroll.set_offset(gpui::point(
                px(0.0),
                -self.scroll.max_offset().height.max(px(1.0)) * tri,
            ));
            window.request_animation_frame();
            // (window start, frames, last frame, slowest gap, frames over 12 ms)
            type Probe = (
                Option<std::time::Instant>,
                u32,
                Option<std::time::Instant>,
                f32,
                u32,
            );
            thread_local! {
                static FPS: std::cell::Cell<Probe> =
                    const { std::cell::Cell::new((None, 0, None, 0.0, 0)) };
            }
            // Perf probe: DODAI_CYCLE also switches category every 90 frames.
            if std::env::var_os("DODAI_CYCLE").is_some() {
                thread_local! { static N: std::cell::Cell<u32> = const { std::cell::Cell::new(0) }; }
                let n = N.with(|n| {
                    n.set(n.get() + 1);
                    n.get()
                });
                if n % 90 == 0 {
                    self.cat = (self.cat + 1) % self.cat_count.max(1);
                    self.sub = None;
                    self.scroll.set_offset(gpui::point(px(0.0), px(0.0)));
                }
            }
            FPS.with(|c| {
                let (t0, n, last, worst, slow) = c.get();
                let now = std::time::Instant::now();
                let t0 = t0.unwrap_or(now);
                let gap = last.map_or(0.0, |l| now.duration_since(l).as_secs_f32() * 1000.0);
                let (worst, slow) = (worst.max(gap), slow + u32::from(gap > 12.0));
                if gap > 25.0 {
                    eprintln!("hitch {gap:.0} ms in category {}", self.cat);
                }
                if t0.elapsed().as_secs_f32() >= 1.0 {
                    eprintln!("fps {n} worst {worst:.1} ms slow>12ms {slow}");
                    c.set((Some(now), 0, Some(now), 0.0, 0));
                } else {
                    c.set((Some(t0), n + 1, Some(now), worst, slow));
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
                    .rail_width(t.navigation().rail_width)
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
                            .flex()
                            .flex_col()
                            .size_full()
                            .child(
                                // Top bar: the theme picker, always in reach.
                                div()
                                    .flex()
                                    .flex_none()
                                    .items_center()
                                    .justify_end()
                                    .gap(px(t.spacing.sm))
                                    .px(px(t.spacing.lg))
                                    .py(px(t.spacing.sm))
                                    .border_b_1()
                                    .border_color(hsla(t.palette.border))
                                    .child(Muted::new("Design"))
                                    .child(
                                        div().w(px(230.0)).child(
                                            dodai_gpui::DesignSystemSelector::new("top-design"),
                                        ),
                                    )
                                    .child(Muted::new("Theme"))
                                    .child(
                                        div()
                                            .w(px(360.0))
                                            .child(dodai_gpui::ThemeSelector::new("top-theme")),
                                    ),
                            )
                            .child(body.min_h_0()),
                    ),
            )
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
const DEFAULT_W: f32 = 1400.0;
const DEFAULT_H: f32 = 800.0;

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
        let search_full = cx.new(|cx| TextInput::new(cx).placeholder("Search…").value("a"));
        let filter = cx.new(|cx| TextInput::new(cx).placeholder("Filter components…"));
        let in_name = cx.new(|cx| TextInput::new(cx).placeholder("handle…"));
        let pw = cx.new(|cx| TextInput::new(cx).placeholder("password").password(true));
        let dt_filter = cx.new(|cx| TextInput::new(cx).placeholder("Filter rows…"));
        let icon_search = cx.new(|cx| TextInput::new(cx).placeholder("Search icons or types…"));
        if let Ok(q) = std::env::var("DODAI_ICON_QUERY") {
            icon_search.update(cx, |i, cx| i.set_text(q, cx));
        }
        let in_outlined = cx.new(|cx| TextInput::new(cx).label("Email").value("ada@example.com"));
        let in_filled = cx.new(|cx| TextInput::new(cx).label("Name").filled(true));
        let in_notes = cx.new(|cx| {
            TextInput::new(cx)
                .multiline(4)
                .placeholder("notes (multi-line)…")
        });
        Showcase {
            filter,
            search,
            search_full,
            in_name,
            in_notes,
            in_outlined,
            in_filled,
            pw,
            dt_filter,
            icon_search,
            icon_filled: false,
            icon_sel: std::env::var("DODAI_ICON_SEL").ok().and_then(|n| {
                dodai_gpui::IconId::ALL
                    .iter()
                    .find(|i| i.name() == n)
                    .copied()
            }),
            icon_code: std::env::var_os("DODAI_CODE").is_some(),
            icon_msg: None,
            icon_open: std::collections::HashSet::new(),
            date_range: Default::default(),
            cat: std::env::var("DODAI_CAT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            replay: 0,
            sec_replay: Default::default(),
            sec_playing: Default::default(),
            sec_mode: Default::default(),
            code_open: Default::default(),
            code_pick: Default::default(),
            loop_gen: 0,
            ld_pb: dodai_gpui::Playback::Loop,
            ld_ms: 0,
            scroll: gpui::ScrollHandle::new(),
            heights: Default::default(),
            realised: Default::default(),
            fresh: Default::default(),
            width: 0.0,
            // Test aid: start scrolled to a section (`DODAI_JUMP=Tabs`).
            jump: std::env::var("DODAI_JUMP")
                .ok()
                .map(|j| (&*Box::leak(j.into_boxed_str()), 1)),
            sec_kind: Default::default(),
            pill_hover: None,
            pop_hover: None,
            cat_count: 1,
            meta: Vec::new(),
            meta_family: "",
            spy: Vec::new(),
            pin: dodai_gpui::ScrollSpy::new(),
            sub: None,
            side_focus: cx.focus_handle(),
            side_scroll: gpui::ScrollHandle::new(),
            rail: std::env::var_os("DODAI_RAIL").is_some(),
            demo_rail: std::env::var_os("DODAI_RAIL").is_some(),
            rating: 3.5,
            group_sel: 1,
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
            heat_res: 3,
            heat_grad: 0,
            sh_gen: "neon_grid",
            sh_fx: vec!["crt"],
            sh_int: 1.0,
            sh_theme: false,
            sh_px: 2.0,
        }
    }

    /// `(category, title)` of every section (empty until the first render).
    pub fn section_titles(&self) -> &[(&'static str, &'static str)] {
        &self.meta
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
        let mut switcher = dodai::switcher();
        if let Ok(id) = std::env::var("DODAI_THEME") {
            switcher.select(&id);
        }
        dodai_gpui::install_switcher(cx, switcher);
        dodai_gpui::set_tree_expanded(cx, false); // theme tree starts contracted
        window_shortcuts(cx);
        // Textures and animated effects start off; glow starts at a subtle 0.25 (hover feedback).
        set_glow_intensity(cx, 0.25);
        // The old per-component scanline body stays off; scanlines are a texture now.
        set_scanlines(cx, false);
        // Test aid: DODAI_SFX=halftone,glitch,sheen,flicker,bloom (or `preset`).
        if let Ok(v) = std::env::var("DODAI_SFX") {
            let mut fx = dodai_core::SurfaceFx::NONE;
            for k in v.split(',') {
                match k {
                    "glitch" => fx.glitch = 0.6,
                    "sheen" => fx.sheen = 1.0,
                    "flicker" => fx.flicker = 0.6,
                    "bloom" => fx.bloom = 0.6,
                    "pulse" => fx.pulse = 0.8,
                    "chase" => fx.chase = 0.8,
                    "edge" => fx.edge = 0.8,
                    "brackets" => fx.brackets = 0.8,
                    "depth" => fx.depth = 0.8,
                    "ripple" => fx.ripple = 0.8,
                    "tint" => fx.tint = 0.8,
                    "pop" => fx.pop = 0.8,
                    "tape" => fx.tape = 0.8,
                    "gloss" => fx.gloss = 0.8,
                    "strip" => fx.strip = 0.8,
                    "clear" => fx.clear = 0.7,
                    "preset" => fx = theme_of(cx).surface_preset(),
                    name => {
                        if let Some(t) = dodai_core::Texture::ALL
                            .iter()
                            .find(|t| t.label().eq_ignore_ascii_case(name))
                        {
                            fx.texture = *t;
                            fx.container_texture = *t;
                        }
                    }
                }
            }
            dodai_gpui::set_surface_fx(cx, fx);
        }
        // Buttons start cropped on the left (the Shape page can switch it off or to the right).
        dodai_gpui::set_button_crop(cx, Some(dodai_gpui::Crop::Left));
        // Debug: DODAI_ACCENT=<edge index 0..7> pre-selects the accent line.
        if let Some(e) = std::env::var("DODAI_ACCENT")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .and_then(|i| dodai_gpui::AccentCorner::ALL.get(i).copied())
        {
            let b = dodai_gpui::section_border(cx);
            dodai_gpui::set_section_border(cx, b.accent(e));
        }
        // Starts maximised. DODAI_SIZE=WxH (e.g. 1100x1400) or DODAI_WINDOWED=1 opens a normal window.
        let windowed = std::env::var_os("DODAI_SIZE").is_some()
            || std::env::var_os("DODAI_WINDOWED").is_some();
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
        win.update(cx, |s, w, _| {
            w.focus(&s.focus);
            // Like GitKraft: open at the default size, and maximise at boot.
            if !windowed {
                w.zoom_window();
            }
        })
        .ok();
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

/// `DODAI_PROF=1`: print the average time `render` spends building the element tree.
struct Prof(Option<std::time::Instant>);
impl Prof {
    fn start() -> Self {
        Self(std::env::var_os("DODAI_PROF").map(|_| std::time::Instant::now()))
    }
}
impl Drop for Prof {
    fn drop(&mut self) {
        thread_local! {
            static ACC: std::cell::Cell<(Option<std::time::Instant>, f32, u32)> =
                const { std::cell::Cell::new((None, 0.0, 0)) };
        }
        let Some(t) = self.0 else { return };
        ACC.with(|c| {
            let (t0, sum, n) = c.get();
            let t0 = t0.unwrap_or_else(std::time::Instant::now);
            let (sum, n) = (sum + t.elapsed().as_secs_f32() * 1000.0, n + 1);
            if t0.elapsed().as_secs_f32() >= 1.0 {
                eprintln!("build {:.2} ms avg over {n} renders", sum / n as f32);
                c.set((Some(std::time::Instant::now()), 0.0, 0));
            } else {
                c.set((Some(t0), sum, n));
            }
        });
    }
}
