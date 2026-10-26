//! `cargo run -p dodai-gpui --example gpui_showcase`
//!
//! Every dodai-gpui component in one scrollable window.
//! Keys: `t` / `T` next / previous theme. Glow +/- buttons change
//! `Theme::with_glow_intensity` live.

use dodai_core::{ButtonVariant, Tone};
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
    Annotation, Bullet, CalendarHeatmap, Lollipop, PairedBars, RoseChart, Sankey, SlopeChart,
    ViolinChart,
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
use gpui::{actions, AnimationExt, SharedString};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Entity, FocusHandle,
    KeyDownEvent, Window, WindowBounds, WindowOptions,
};
use std::time::Duration;

mod app;
mod catalog;
mod charts_page;
mod code;
mod foundations;
mod impl_src;
mod motion_page;
mod overview;
mod player;
mod section_view;
mod sections;
mod shaders_page;
mod sidebar;
mod snippets;
use app::{window_shortcuts, DEFAULT_H, DEFAULT_W, SPY_SLACK};
use catalog::{cat_blurb, cat_icon, section_id, sub_of, subs};
use charts_page::ChartDemos;
use code::code_toggle;
use foundations::{
    frosted_view, layout_view, modifier_variants_view, motion_tokens_view, palette_view,
    shape_tokens_view, spacing_elevation_view, type_scale_view, type_view,
};
use player::Player;
pub(crate) use shaders_page::{Group, Section};

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
    section_views: std::collections::HashMap<&'static str, Entity<section_view::SectionView>>,
    /// Elements of the groups built this frame, by section title (siblings of a rebuilt section).
    built: std::collections::HashMap<&'static str, gpui::AnyElement>,
    /// Heights the sections measured themselves (a cached view is sized from its style).
    real_h: std::collections::HashMap<&'static str, f32>,
    /// Set while a scroll notifies: the sections need no rebuild for it.
    quiet: bool,
    /// Per section title: its category, the index of the group that builds it, and its key.
    section_group: std::collections::HashMap<&'static str, (&'static str, usize, &'static str)>,
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
    /// Oscilloscope afterimage: 0 off, 1 subtle, 2 default, 3 long.
    inst_trail: usize,
    /// Needle response of the instruments: 0 the design's own, then `NeedleResponse::ALL`.
    inst_response: usize,
    /// Whether the instrument demos change their values on their own.
    inst_live: bool,
    /// Pulse monitor options: rhythm, mode, grid, noise, rate, trail, gain.
    pulse_cfg: [usize; 7],
    sh_gen: &'static str,
    sh_fx: Vec<&'static str>,
    sh_int: f32,
    sh_theme: bool,
    sh_px: f32,
}

impl Render for Showcase {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _prof = Prof::start();
        self.quiet = false;
        self.built.clear();
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
        // The sections on screen, in order; their elements are built later, only for the ones
        // that are realised (near the viewport).
        let sections: Vec<(&'static str, &'static str)> =
            meta.iter().filter(|(c, _)| in_cat(c)).copied().collect();
        let mut body = div()
            .id(("body", self.cat * 1000 + self.replay))
            .track_scroll(&self.scroll)
            .relative()
            // Scroll-spy reads layout, so repaint on scroll even on static pages.
            .on_scroll_wheel(cx.listener(|s, _: &gpui::ScrollWheelEvent, _, cx| {
                s.pin.release();
                s.quiet = true;
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
        let mut shown_secs: Vec<(&'static str, &'static str)> = Vec::new();
        let mut keys: std::collections::HashMap<&'static str, &'static str> = Default::default();
        for (c, title) in sections {
            if let Some((_, _, key)) = self.section_group.get(title) {
                keys.insert(title, key);
            }
            let hit =
                query.is_empty() || !fuzzy_filter(&query, &[format!("{c} {title}")]).is_empty();
            let only = std::env::var("DODAI_ONLY").is_ok_and(|o| !title.contains(&o));
            let sub_ok = self.sub.is_none_or(|x| sub_of(c, title) == Some(x));
            if in_cat(c) && hit && sub_ok && !only {
                shown_secs.push((c, title));
            }
        }
        let mut order: Vec<(&'static str, usize)> = Vec::new();
        {
            let (mut n, mut lc, mut ls) = (n_children, "", None);
            for (c, title) in &shown_secs {
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
        for (seq, (c, title)) in shown_secs.into_iter().enumerate() {
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
            let known = self
                .real_h
                .get(title)
                .or_else(|| self.heights.get(title))
                .copied();
            let bounds = if valid {
                self.scroll.bounds_for_item(n_children)
            } else {
                None
            };
            if let Some(b) =
                bounds.filter(|_| prev_real.contains(title) && !self.real_h.contains_key(title))
            {
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
                let view = self
                    .section_views
                    .entry(title)
                    .or_insert_with(|| {
                        cx.new(|cx| section_view::SectionView::new(me.clone(), title, cx))
                    })
                    .clone();
                let h = known.unwrap_or(avg_h);
                body = body.child(
                    gpui::AnyView::from(view).cached(
                        gpui::StyleRefinement::default()
                            .w_full()
                            .flex_none()
                            .h(px(h)),
                    ),
                );
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
            .children(dodai_gpui::backdrop(&t))
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
                            .children(
                                matches!(selected_cat, Some("Shape" | "Surface effects"))
                                    .then(|| sections::shape::live_preview(&t)),
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

fn new_showcase(cx: &mut App) -> Entity<Showcase> {
    cx.new(Showcase::build)
}

fn main() {
    Application::new().run(|cx: &mut App| {
        // Running animations redraw at most 60 times a second (DODAI_FPS=0 lifts the cap).
        let fps = std::env::var("DODAI_FPS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);
        dodai_gpui::set_max_fps(Some(fps));
        // Debug: DODAI_THEME=<theme id> starts on that theme (e.g. material-dark).
        let mut switcher = dodai::switcher();
        if let Ok(id) = std::env::var("DODAI_THEME") {
            switcher.select(&id);
        }
        dodai_gpui::install_switcher(cx, switcher);
        dodai_gpui::set_tree_expanded(cx, false); // theme tree starts contracted
                                                  // Debug: DODAI_MONO=1 starts with monochrome series colours.
        if std::env::var_os("DODAI_MONO").is_some() {
            dodai_gpui::set_chart_monochrome(cx, true);
        }
        window_shortcuts(cx);
        // Textures and animated effects start off; glow starts at a subtle 0.25 (hover feedback).
        set_glow_intensity(cx, 0.25);
        // The old per-component scanline body stays off; scanlines are a texture now.
        set_scanlines(cx, false);
        // Debug: DODAI_TUNE=round=0.0,width=3,all starts with that shape tuning.
        if let Ok(v) = std::env::var("DODAI_TUNE") {
            let mut tune = dodai_core::ShapeTune::DEFAULT;
            for k in v.split(',') {
                match k.split_once('=') {
                    Some(("round", n)) => tune.roundness = n.parse().unwrap_or(1.0),
                    Some(("width", n)) => tune.border_width = n.parse().ok(),
                    _ if k == "all" => tune.outline_all = true,
                    _ => {}
                }
            }
            dodai_gpui::set_shape_tune(cx, tune);
        }
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
                    p if dodai_core::PressStyle::ALL
                        .iter()
                        .any(|s| s.label().eq_ignore_ascii_case(p)) =>
                    {
                        // DODAI_SFX=ring,... also freezes a ripple mid-play for screenshots.
                        fx.ripple = 0.8;
                        if let Some(s) = dodai_core::PressStyle::ALL
                            .iter()
                            .find(|s| s.label().eq_ignore_ascii_case(p))
                        {
                            fx.press = *s;
                        }
                    }
                    "tint" => fx.tint = 0.8,
                    "pop" => fx.pop = 0.8,
                    "tape" => fx.tape = 0.8,
                    "gloss" => fx.gloss = 0.8,
                    "strip" => fx.strip = 0.8,
                    "wash" => fx.wash = 0.8,
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
        // Designs with cut corners start cropped on the left (the Shape page can change it, on any design).
        dodai_gpui::set_design_button_crop(cx, Some(dodai_gpui::Crop::Left));
        // Debug: DODAI_STYLE=round|cut|scoop forces that corner style on every design.
        if let Ok(v) = std::env::var("DODAI_STYLE") {
            use dodai_core::CornerStyle as S;
            let corner_style = match v.as_str() {
                "round" => Some(S::Round),
                "cut" => Some(S::Cut),
                "scoop" => Some(S::Scoop),
                _ => None,
            };
            dodai_gpui::set_shape_tune(
                cx,
                dodai_core::ShapeTune {
                    corner_style,
                    ..dodai_gpui::shape_tune()
                },
            );
        }
        // Debug: DODAI_DENSITY=0.8 scales control height and spacing on every design.
        if let Some(density) = std::env::var("DODAI_DENSITY")
            .ok()
            .and_then(|v| v.parse().ok())
        {
            dodai_gpui::set_shape_tune(
                cx,
                dodai_core::ShapeTune {
                    density,
                    ..dodai_gpui::shape_tune()
                },
            );
        }
        // Debug: DODAI_CROP=off|left|right|top|bottom forces that crop on every design.
        if let Ok(v) = std::env::var("DODAI_CROP") {
            use dodai_gpui::Crop;
            let crop = match v.as_str() {
                "left" => Some(Crop::Left),
                "right" => Some(Crop::Right),
                "top" => Some(Crop::Top),
                "bottom" => Some(Crop::Bottom),
                _ => None,
            };
            dodai_gpui::set_button_crop(cx, crop);
        }
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
