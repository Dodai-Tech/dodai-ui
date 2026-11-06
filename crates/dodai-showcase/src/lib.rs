//! The gpui showcase (`cargo run -p dodai-showcase`).
//!
//! Every dodai-gpui component in one scrollable window.
//! Keys: `t` / `T` next / previous theme. Glow +/- buttons change
//! `Theme::with_glow_intensity` live.

use dodai_core::{ButtonVariant, Tone};
use dodai_gpui::{
    cycle_next, cycle_prev, fuzzy_filter, glow_intensity, hsla, set_glow_intensity, set_scanlines,
    theme_of, Accordion, BottomSheet, Breadcrumb, BubblePlacement, Button, ButtonGroup, Card,
    CardVariant, Carousel, Checkbox, Dialog, DialogCard, Divider, DodaiStyled, DodaiTooltip, Fab,
    FabSize, Heading, IconButton, InputBox, Knob, Label, ListItem, LoadingIndicator, Mono, Muted,
    Orientation, Pagination, Panel, ProgressBar, Radio, ResponsiveShell, SearchBar, Segmented,
    SideSheet, Slider, SplitButton, Stepper, Tabs, Tag, Toggle, TopAppBar,
};
use dodai_gpui::{CountUp, Easing, EntryKind, IndeterminateBar, Motion, Skeleton, Spinner};
use dodai_gpui::{
    Date, DatePicker, EmptyState, IconBadge, IconName, NumberStepper, RangeSlider, Rating,
    RichTooltip, SearchView, Snackbar, TagField, TextInput, TimePicker, Timeline, TimelineEvent,
    Toolbar, TreeItem, TreeNavigation, TreeView,
};
use dodai_gpui_charts::{
    Annotation, Bullet, CalendarHeatmap, Lollipop, PairedBars, RoseChart, Sankey, SlopeChart,
    ViolinChart,
};
use dodai_gpui_charts::{
    BarChart, BoxPlot, Candle, Candlestick, Funnel, Heatmap, Histogram, LineChart, Marker,
    PieChart, PieStyle, RadarChart, RadialBar, ScatterPlot, Series, SparkGuide, SparkStyle,
    Sparkline, StackedAreaChart, Treemap, Waterfall,
};
use dodai_gpui_charts::{
    CandleOverlay, CandlePane, CandleStyle, ChartStack, DepthChart, Gantt, GanttTask, RenkoChart,
    TradeMarker,
};
use dodai_gpui_instruments::{
    Blip, DotMatrix, Gauge, GaugeStyle, HexMap, Oscilloscope, RadarScope, RingStack, SignalBars,
    Spectrum,
};
use gpui::{actions, AnimationExt, SharedString};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Entity, FocusHandle,
    KeyDownEvent, Window, WindowBounds, WindowOptions,
};
use std::time::Duration;

mod app;
mod catalog;
mod charts_gauges;
mod charts_instruments;
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
mod sidebar_view;
mod snippets;
use app::{window_shortcuts, DEFAULT_H, DEFAULT_W, SPY_SLACK};
use catalog::{
    cat_blurb, cat_icon, keywords, section_id, sub_of, subs, CHART_FAMILIES, CHART_TILE_MAX_W,
    WIDE_CHARTS,
};
use charts_page::ChartDemos;
use code::code_toggle;
use foundations::{
    frosted_view, layout_view, modifier_variants_view, motion_tokens_view, palette_view,
    shape_tokens_view, spacing_elevation_view, type_scale_view, type_view,
};
pub(crate) use shaders_page::{Group, Section};

pub struct Showcase {
    /// Window width in px, as of the last render (sections size pictures from it).
    viewport_w: f32,
    replay: usize,
    sec_replay: std::collections::HashMap<&'static str, u32>,
    /// Sections whose "</> Code" panel is open.
    code_open: std::collections::HashSet<&'static str>,
    /// Which component's implementation (or the usage snippet) each open code panel shows.
    code_pick: std::collections::HashMap<&'static str, usize>,
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
    icon_sel: Option<sections::IconEntry>,
    icon_code: bool,
    icon_msg: Option<String>,
    /// Material icon categories shown in full (the rest show a preview).
    icon_open: std::collections::HashSet<sections::IconPick>,
    /// The branch of the Icons tree the sidebar picked (what the section shows).
    icon_pick: sections::IconPick,
    date_range: dodai_gpui::DateRange,
    /// Start and end time of day of the date-time range demo.
    range_times: ((u32, u32), (u32, u32)),
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
    /// The sidebar view, the width it asked for, and the scroll-spy state it last showed.
    sidebar_view: Option<Entity<sidebar_view::SidebarView>>,
    side_need: f32,
    side_spied: (Option<&'static str>, bool),
    /// The overview's last laid-out height, and whether it is realised (not a spacer).
    overview_h: f32,
    overview_real: bool,
    /// When the page last scrolled: sections below are measured ahead only once it rests.
    scrolled_at: Option<std::time::Instant>,
    /// A wake-up for when the page comes to rest is already scheduled.
    rest_pending: bool,
    /// Lay out every section, not only those near the viewport (`DODAI_NOCULL`, tests).
    no_cull: bool,
    /// Set while a demo step notifies: only the sections in `live_sections` rebuild.
    demo_only: bool,
    /// Sections whose build reads the playing demo values (it stamps `demo_seen`/`inst_seen`).
    live_sections: std::collections::HashSet<&'static str>,
    /// Per section title: its category, the index of the group that builds it, and its key.
    section_group: std::collections::HashMap<&'static str, (&'static str, usize, &'static str)>,
    spy: Vec<(&'static str, usize)>,
    /// Click-pinned current section (see `dodai_gpui::ScrollSpy`).
    pin: dodai_gpui::ScrollSpy<&'static str>,
    /// Selected subsection inside the current category (`None` = all).
    sub: Option<&'static str>,
    /// Categories whose sub-group list is folded (the open one is the current category).
    shut: Vec<&'static str>,
    /// Sub-groups whose sections are listed in the sidebar.
    open_subs: Vec<&'static str>,
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
    /// The selection animation shown on tabs, segmented controls and toggles: `None` follows
    /// the design.
    animation: Option<dodai_gpui::motion::SelectionMotion>,
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
    /// Glow of every chart and instrument: 0 the theme's own, 1 off, 2 soft, 3 strong.
    inst_glow: usize,
    /// Channel-plan blocks the user toggled: `(band, row, block)`.
    excluded_slots: std::collections::BTreeSet<(usize, usize, usize)>,
    /// When the Instruments page last rendered; it repaints every frame while it is showing.
    inst_seen: Option<std::time::Instant>,
    /// When an input demo page last rendered; its values play on their own while it shows.
    demo_seen: Option<std::time::Instant>,
    /// A demo step is already waiting for the next frame.
    demo_tick_queued: bool,
    demo_email: String,
    /// The controlled `TreeView` demo: its nodes (with their folded state) and selection.
    tree_items: Vec<TreeItem>,
    tree_sel: Option<Vec<usize>>,
    /// The orientation gizmo demo: orbit yaw and pitch, radians.
    gizmo_yaw: f32,
    gizmo_pitch: f32,
    demo_pin: String,
    demo_otp: String,
    live_a: dodai_gpui::Live,
    live_b: dodai_gpui::Live,
    live_c: dodai_gpui::Live,
    /// The speed-test demo's download and upload windows.
    speed_down: dodai_gpui::Live,
    speed_up: dodai_gpui::Live,
    track_sel: usize,
    station_sel: usize,
    /// Pulse monitor options: rhythm, mode, grid, noise, rate, trail, gain.
    pulse_cfg: [usize; 7],
    sh_gen: &'static str,
    sh_fx: Vec<&'static str>,
    sh_int: f32,
    sh_theme: bool,
    sh_px: f32,
}

/// How long after the last scroll the page counts as resting (and measures off-screen
/// sections).
const REST: std::time::Duration = std::time::Duration::from_millis(200);

/// Window width below which the sidebar becomes a drawer behind the menu bar.
pub(crate) const SHELL_BREAKPOINT: f32 = 640.0;

impl Showcase {
    /// The width the shell gives the sidebar column (the collapsed rail when it is folded).
    pub(crate) fn side_width(&self, t: &dodai_core::Theme) -> f32 {
        if self.rail && self.viewport_w >= SHELL_BREAKPOINT {
            t.navigation().rail_width
        } else if self.viewport_w < 760.0 {
            self.side_need.clamp(170.0, 230.0)
        } else {
            self.side_need.clamp(240.0, 340.0)
        }
    }

    /// Render the page again next frame as a scroll does: the cached sections and the sidebar
    /// keep their last build (a plain frame request would rebuild every one of them).
    fn next_frame_quiet(&self, window: &mut Window, cx: &mut Context<Self>) {
        cx.on_next_frame(window, |s, _, cx| {
            s.quiet = true;
            cx.notify();
        });
    }
}

impl Render for Showcase {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _prof = Prof::start();
        self.quiet = false;
        self.built.clear();
        self.viewport_w = window.viewport_size().width.into();
        let narrow = window.viewport_size().width < px(760.0);
        // Medium widths keep the rail but still need a compact top bar and preview strip.
        let compact = window.viewport_size().width < px(1100.0);
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

        let sections_meta = &meta;
        // Sidebar: its own cached view (filter + category list with counts). A scroll only
        // reaches it when the scroll-spy (the section at the top, or being at the top) moves.
        let spied = (
            self.pin.current(&self.scroll, &self.spy, SPY_SLACK),
            f32::from(self.scroll.offset().y) > -1.0,
        );
        let side = self
            .sidebar_view
            .get_or_insert_with(|| cx.new(|cx| sidebar_view::SidebarView::new(me.clone(), cx)))
            .clone();
        if spied != self.side_spied {
            self.side_spied = spied;
            side.update(cx, |_, cx| cx.notify());
        }
        let side_need = self.side_need;
        let pinned_icons = self.pinned_icon_panel(&t, cx);
        // One needle response for the instruments without their own; the Vehicle page shows the
        // switch pinned above its sections.
        let in_instruments = selected_cat == Some("Instruments");
        dodai_gpui::follow::set_default_response(
            in_instruments
                .then(|| {
                    (self.inst_response > 0)
                        .then(|| dodai_gpui::NeedleResponse::ALL[self.inst_response - 1])
                })
                .flatten(),
        );
        let needle_bar =
            (in_instruments && matches!(self.sub, None | Some("Vehicle"))).then(|| {
                let m = me.clone();
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_2()
                    .child(dodai_gpui::Label::new("Needle"))
                    .child(
                        dodai_gpui::Segmented::new(
                            "veh-response",
                            ["Design", "Instant", "Snappy", "Smooth", "Heavy", "Bouncy"],
                        )
                        .selected(self.inst_response)
                        .on_select(move |i, _, cx| {
                            m.update(cx, |s, cx| {
                                s.inst_response = i;
                                cx.notify();
                            })
                        }),
                    )
            });
        let side = gpui::AnyView::from(side).cached(gpui::StyleRefinement::default().size_full());

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
                s.scrolled_at = Some(std::time::Instant::now());
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
            // One child, virtualised like a section: scrolled away it is a spacer of its last
            // height, so its cards and wrapped text leave the per-frame layout.
            let vp = self.scroll.bounds();
            let shift: f32 = self.scroll.offset().y.into();
            let seen = self
                .scroll
                .bounds_for_item(0)
                .filter(|_| self.overview_h > 0.0);
            if let Some(b) = seen.filter(|_| self.overview_real) {
                self.overview_h = b.size.height.into();
            }
            let near = seen.is_none_or(|b| {
                f32::from(b.origin.y) + shift + self.overview_h
                    > f32::from(vp.origin.y) - (f32::from(vp.size.height) * 0.2).max(200.0)
            });
            self.overview_real = near;
            body = body.child(if near {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.lg + 8.0))
                    .flex_none()
                    .children(self.overview(&me, &cats, sections_meta, cx))
            } else {
                div().flex_none().w_full().h(px(self.overview_h))
            });
            n_children += 1;
            if self.overview_h == 0.0 {
                // Measure it on the next frame.
                self.overview_h = 1.0;
                window.request_animation_frame();
            }
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
            let hit = query.is_empty()
                || !fuzzy_filter(&query, &[format!("{c} {title} {}", keywords(title))]).is_empty();
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
        // A scroll moves every realised section, and gpui rebuilds a cached view whose
        // bounds moved: the margin is all extra work per scroll frame, so keep it to what
        // one fast frame can scroll.
        let vp = self.scroll.bounds();
        let vp_top: f32 = vp.origin.y.into();
        let vp_h: f32 = vp.size.height.into();
        let valid = vp_h > 0.0 && order == self.spy;
        let margin = (vp_h * 0.2).max(200.0);
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
        // Sections below the viewport get measured one per frame once the page rests (a
        // first build costs several ms: during a scroll it would drop frames), so spacer
        // heights become exact without one big hitch.
        let resting = self.scrolled_at.is_none_or(|t| t.elapsed() > REST);
        let mut budget = usize::from(resting);
        // A first build costs several ms: sections entering the margin (not yet on screen)
        // come in one per frame, so two never land in the same frame.
        let mut entering = 1usize;
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
                    let on_screen = bottom > vp_top && top < vp_top + vp_h;
                    let mut near = bottom > vp_top - margin && top < vp_top + vp_h + margin;
                    if near && !on_screen && !prev_real.contains(title) {
                        near = entering > 0;
                        unmeasured |= !near;
                        entering = entering.saturating_sub(1);
                    }
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
            if realise || self.no_cull {
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
            match self.scrolled_at.map(|t| REST.saturating_sub(t.elapsed())) {
                // Still scrolling: wake once the page has rested instead of every frame.
                // One pending wake-up at a time; it re-arms if the page scrolled since.
                Some(wait) if !wait.is_zero() => {
                    if !self.rest_pending {
                        self.rest_pending = true;
                        cx.spawn(async move |me, cx| {
                            cx.background_executor().timer(wait).await;
                            let _ = me.update(cx, |s, cx| {
                                s.rest_pending = false;
                                s.quiet = true;
                                cx.notify();
                            });
                        })
                        .detach();
                    }
                }
                _ => self.next_frame_quiet(window, cx),
            }
        }
        self.tick_demos(window, cx);
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
        // Debug: DODAI_SCROLL=<px> scrolls the page that far further down, two frames after the
        // jump to the chosen sub-group has landed: for screenshots of sections taller than the
        // screen.
        static FRAMES: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        if let Some(y) = std::env::var("DODAI_SCROLL")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|_| self.jump.is_none())
        {
            match FRAMES.fetch_add(1, std::sync::atomic::Ordering::Relaxed) {
                0 | 1 => cx.notify(),
                2 => {
                    let top = self.scroll.offset().y;
                    self.scroll.set_offset(gpui::point(px(0.0), top - px(y)));
                    cx.notify();
                }
                _ => {}
            }
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
            self.scrolled_at = Some(std::time::Instant::now());
            self.scroll.set_offset(gpui::point(
                px(0.0),
                -self.scroll.max_offset().height.max(px(1.0)) * tri,
            ));
            self.next_frame_quiet(window, cx);
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
                if e.keystroke.key == "r" {
                    dodai_gpui::set_reduce_motion(cx, !dodai_gpui::reduce_motion());
                }
                if e.keystroke.key == "m" {
                    dodai_gpui::toggle_theme_mode(cx);
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
                    .breakpoint(SHELL_BREAKPOINT)
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
                                // Top bar: the theme picker, always in reach. Label + control
                                // pairs wrap onto further rows instead of overflowing.
                                {
                                    // The pickers grow to fill the row, the small controls keep
                                    // their size; the bar runs from the left edge to the right one.
                                    let pair = |label: &'static str, control: gpui::AnyElement| {
                                        let grows = matches!(label, "Design" | "Theme");
                                        div()
                                            .flex()
                                            .when(grows, |d| d.flex_1())
                                            .when(!grows, |d| d.flex_none())
                                            .items_center()
                                            .gap(px(t.spacing.sm))
                                            .children(
                                                (!(compact && matches!(label, "Design" | "Theme")))
                                                    .then(|| Muted::new(label)),
                                            )
                                            .child(control)
                                    };
                                    let (dw, tw) = if narrow {
                                        (170.0, 210.0)
                                    } else if compact {
                                        (190.0, 230.0)
                                    } else {
                                        (230.0, 360.0)
                                    };
                                    div()
                                        .flex()
                                        .flex_none()
                                        .flex_wrap()
                                        .items_center()
                                        .gap_x(px(t.spacing.md))
                                        .gap_y(px(t.spacing.sm))
                                        .px(px(if narrow { t.spacing.md } else { t.spacing.lg }))
                                        .py(px(t.spacing.sm))
                                        .border_b_1()
                                        .border_color(hsla(t.palette.border))
                                        .child(pair(
                                            "Animate",
                                            dodai_gpui::Segmented::new(
                                                "top-animate",
                                                ["Off", "On"],
                                            )
                                            .selected(usize::from(dodai_gpui::motion::enabled()))
                                            .on_select(|i, _, cx| {
                                                dodai_gpui::motion::set_enabled(cx, i == 1)
                                            })
                                            .into_any_element(),
                                        ))
                                        .child(pair(
                                            "Speed",
                                            dodai_gpui::Segmented::new(
                                                "top-speed",
                                                ["0.5×", "1×", "2×", "4×"],
                                            )
                                            .selected(
                                                [0.5, 1.0, 2.0, 4.0]
                                                    .iter()
                                                    .position(|v| {
                                                        (*v - dodai_gpui::motion::speed()).abs()
                                                            < 0.01
                                                    })
                                                    .unwrap_or(1),
                                            )
                                            .on_select(|i, _, cx| {
                                                dodai_gpui::motion::set_speed(
                                                    cx,
                                                    [0.5, 1.0, 2.0, 4.0][i],
                                                )
                                            })
                                            .into_any_element(),
                                        ))
                                        .child(pair(
                                            "Mode",
                                            dodai_gpui::Segmented::new(
                                                "top-mode",
                                                ["System", "Light", "Dark"],
                                            )
                                            .selected(
                                                if dodai_gpui::following_system_appearance() {
                                                    0
                                                } else if t.meta.mode == dodai_core::Mode::Dark {
                                                    2
                                                } else {
                                                    1
                                                },
                                            )
                                            .on_select(|i, w, cx| match i {
                                                0 => {
                                                    dodai_gpui::follow_system_appearance(w, cx);
                                                }
                                                1 => {
                                                    dodai_gpui::set_theme_mode(
                                                        cx,
                                                        dodai_core::Mode::Light,
                                                    );
                                                }
                                                _ => {
                                                    dodai_gpui::set_theme_mode(
                                                        cx,
                                                        dodai_core::Mode::Dark,
                                                    );
                                                }
                                            })
                                            .into_any_element(),
                                        ))
                                        .child(pair(
                                            "Direction",
                                            dodai_gpui::Segmented::new(
                                                "top-direction",
                                                ["LTR", "RTL"],
                                            )
                                            .selected(usize::from(dodai_gpui::is_rtl()))
                                            .on_select(|i, _, cx| {
                                                dodai_gpui::set_direction(
                                                    cx,
                                                    if i == 1 {
                                                        dodai_gpui::Direction::Rtl
                                                    } else {
                                                        dodai_gpui::Direction::Ltr
                                                    },
                                                )
                                            })
                                            .into_any_element(),
                                        ))
                                        .child(pair(
                                            "Patterns",
                                            dodai_gpui::Segmented::new(
                                                "top-patterns",
                                                ["Off", "On"],
                                            )
                                            .selected(usize::from(
                                                dodai_gpui_charts::common::patterns(),
                                            ))
                                            .on_select(|i, _, cx| {
                                                dodai_gpui_charts::common::set_patterns(cx, i == 1)
                                            })
                                            .into_any_element(),
                                        ))
                                        .child(pair(
                                            "Design",
                                            div()
                                                .flex_1()
                                                .min_w(px(dw))
                                                .child(dodai_gpui::DesignSystemSelector::new(
                                                    "top-design",
                                                ))
                                                .into_any_element(),
                                        ))
                                        .child(pair(
                                            "Theme",
                                            div()
                                                .flex_1()
                                                .min_w(px(tw))
                                                .child(dodai_gpui::ThemeSelector::new("top-theme"))
                                                .into_any_element(),
                                        ))
                                        .child(pair(
                                            "Colour",
                                            sections::shape::control_colour_picker("top-fill", cx)
                                                .into_any_element(),
                                        ))
                                },
                            )
                            .children(
                                matches!(selected_cat, Some("Shape" | "Surface effects"))
                                    .then(|| sections::shape::live_preview(&t, compact)),
                            )
                            .children(pinned_icons)
                            .children(needle_bar)
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

/// Opens the showcase window and runs until it is closed.
pub fn run() {
    Application::new().run(|cx: &mut App| {
        // Running animations redraw at most 60 times a second (DODAI_FPS=0 lifts the cap).
        let fps = std::env::var("DODAI_FPS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);
        dodai_gpui::set_max_fps(Some(fps));
        // Debug: DODAI_THEME=<theme id> starts on that theme (e.g. material-dark).
        let mut switcher = dodai_core::switcher();
        if let Ok(id) = std::env::var("DODAI_THEME") {
            switcher.select(&id);
        }
        let pinned_theme = std::env::var_os("DODAI_THEME").is_some();
        dodai_gpui::install_switcher(cx, switcher);
        // Honour the OS's reduced-motion setting.
        dodai_gpui::follow_system_motion(cx);
        // Debug: DODAI_STILL=<seconds> shows that moment of the demo values with motion reduced,
        // for screenshots of a window that gets no frames (a locked screen, a hidden window).
        if let Some(at) = std::env::var("DODAI_STILL")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
        {
            dodai_gpui::motion::set_reduce_motion(cx, true);
            dodai_gpui::motion::set_enabled(cx, false);
            dodai_gpui::motion::skip_clock(at);
        }
        // Debug: DODAI_MONO=1 starts with monochrome series colours.
        if std::env::var_os("DODAI_MONO").is_some() {
            dodai_gpui::set_chart_monochrome(cx, true);
        }
        window_shortcuts(cx);
        // Textures and animated effects start off; glow starts at a subtle 0.25 (hover feedback).
        set_glow_intensity(cx, 0.25);
        // The old per-component scanline body stays off; scanlines are a texture now.
        set_scanlines(cx, false);
        // Debug: DODAI_TUNE=round=0.0,width=3,fill=accent,all starts with that tuning.
        if let Ok(v) = std::env::var("DODAI_TUNE") {
            let mut tune = dodai_core::ShapeTune::DEFAULT;
            for k in v.split(',') {
                match k.split_once('=') {
                    Some(("round", n)) => tune.roundness = n.parse().unwrap_or(1.0),
                    Some(("width", n)) => tune.border_width = n.parse().ok(),
                    Some(("fill", n)) => {
                        if let Some(c) = dodai_core::ControlColour::ALL
                            .into_iter()
                            .find(|c| c.label().eq_ignore_ascii_case(n))
                        {
                            tune.control_colour = c;
                        }
                    }
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
                    "rim" => fx.rim = 0.8,
                    "clear" => fx.clear = 0.7,
                    "frost" => fx.frost = 1.0,
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
        // Test aid: DODAI_WALLPAPER=1 puts the hard-edged picture behind the app (frost demo).
        if std::env::var("DODAI_WALLPAPER").is_ok() {
            let (w, h) = (1200, 750);
            let p = theme_of(cx).palette;
            dodai_gpui::set_wallpaper(cx, sections::shape::glass_picture(&p, w, h), w, h);
        }
        // Designs with cut corners start cropped on the left (the Shape page can change it; every
        // design draws the crop its own way).
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
        // Debug: DODAI_GRADIENT=off|subtle|bold sets how strongly a gradient design shows it.
        if let Ok(v) = std::env::var("DODAI_GRADIENT") {
            use dodai_core::GradientMode;
            let gradient = match v.as_str() {
                "off" => GradientMode::Off,
                "bold" => GradientMode::Bold,
                _ => GradientMode::Subtle,
            };
            dodai_gpui::set_shape_tune(
                cx,
                dodai_core::ShapeTune {
                    gradient,
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
                    // Debug: DODAI_FLOAT=1 keeps the window above the others, so a screenshot run
                    // gets frames even when another app holds the focus.
                    kind: if std::env::var_os("DODAI_FLOAT").is_some() {
                        gpui::WindowKind::PopUp
                    } else {
                        gpui::WindowKind::Normal
                    },
                    ..Default::default()
                },
                |_, cx| new_showcase(cx),
            )
            .expect("open window");
        win.update(cx, |s, w, cx| {
            w.focus(&s.focus);
            // Light or dark like the OS, unless a theme was asked for.
            if !pinned_theme {
                dodai_gpui::follow_system_appearance(w, cx);
            }
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
