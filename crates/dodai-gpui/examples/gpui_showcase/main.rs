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
mod sections;
mod shaders_page;
mod sidebar;
use catalog::{caps, cat_blurb, cat_icon, section_id, sub_of, subs};
use charts_page::ChartDemos;
use foundations::{icons_view, palette_view, shape_tokens_view, spacing_elevation_view, type_view};
pub(crate) use shaders_page::Section;

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
    /// `(category, title)` of every section, built once; drives the sidebar.
    meta: Vec<(&'static str, &'static str)>,
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

        // Global shape properties (glow, scanlines, cut corners): Shape category.

        let ds_panel = Panel::new().focused(true).child(
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .child(Label::new("Design System Selection"))
                .child(DesignSystemTree::new("ds-tree")),
        );

        // ── new components ─────────────────────────────────────────────

        // Stepped / configurable sliders.

        if self.meta.is_empty() {
            self.meta = self.section_meta(cx);
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
            self.sidebar(&me, &cats, sections_meta, active, &query, count_in, cx);

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
            let kids = self.overview(&me, &cats, sections_meta, ds_panel.take(), cx);
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
            for (c, t) in sections_meta {
                eprintln!("{c}\t{t}");
            }
            std::process::exit(0);
        }
        let mut spy: Vec<(&'static str, usize)> = Vec::new();
        let mut shown = 0;
        let mut last_cat = "";
        let mut last_sub: Option<&str> = None;
        for (c, title, el, _key) in sections {
            let in_cat = in_cat(c);
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
            meta: Vec::new(),
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
