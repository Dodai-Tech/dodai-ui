//! Window setup: shortcuts, menu, and building the [`Showcase`].

use super::*;
use dodai_ui_gpui::TextInput;
use gpui::{px, size, App, Context, Window};
use gpui::{KeyBinding, Menu, MenuItem};

/// Px below the scroll container top at which a section counts as current.
pub(crate) const SPY_SLACK: f32 = 60.0;
pub(crate) const DEFAULT_W: f32 = 1400.0;
pub(crate) const DEFAULT_H: f32 = 800.0;

/// Run `f` on the active window (menu/shortcut actions are app-global).
pub(crate) fn on_active(cx: &mut App, f: impl FnOnce(&mut Window) + 'static) {
    if let Some(w) = cx.active_window() {
        let _ = w.update(cx, |_, window, _| f(window));
    }
}

/// Window management: macOS-style menu + keyboard shortcuts.
pub(crate) fn window_shortcuts(cx: &mut App) {
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
    /// Plays the input demos on their own: values glide, boxes tick, text types itself.
    fn demo_step(&mut self, cx: &mut Context<Self>) {
        let t = dodai_ui_gpui::motion::time();
        // `0..=1` demo positions drifting on the animation clock.
        let wave = |key: f32, phase: f32| {
            dodai_ui_gpui::live::wander_between(key * 10.0 + phase, 0.0, 1.0)
        };
        // Live streams: a new sample every 0.2 s / 0.35 s; the charts slide between them.
        self.live_a.advance(0.2, |i| live_signal(i, 0.9));
        self.live_b.advance(0.35, |i| live_signal(i, 0.6));
        self.speed_step();
        self.slider = wave(0.55, 0.0);
        self.stepped = (wave(0.4, 1.0) * 10.0).round();
        self.db = -24.0 + (wave(0.35, 2.0) * 10.0).round() * 3.0;
        self.bub = (
            wave(0.45, 0.5) * 100.0,
            (wave(0.3, 2.5) * 10.0).round() * 10.0,
        );
        for i in 0..3 {
            self.knob[i] = wave(0.5 + 0.12 * i as f32, i as f32);
            self.vslider[i] = wave(0.6 - 0.1 * i as f32, 1.5 * i as f32);
        }
        let lo = wave(0.3, 0.0) * 0.4;
        self.range = (lo, lo + 0.2 + 0.3 * wave(0.45, 1.0));
        let lo = wave(0.37, 2.0) * 0.4;
        self.vrange = (lo, lo + 0.2 + 0.3 * wave(0.5, 0.0));
        // Everything that counts goes up and back down, so nothing snaps at the end of a loop.
        let tri = |speed: f32, max: f32| {
            let p = (t * speed / max).rem_euclid(2.0);
            max * if p < 1.0 { p } else { 2.0 - p }
        };
        self.rating = (tri(0.7, 5.0) * 2.0).round() / 2.0;
        self.stepper = tri(0.8, 10.0).round() as i32;
        self.switch = (t / 2.2) as i64 % 2 == 0;
        self.check = (t / 3.1) as i64 % 2 == 0;
        self.radio = (t / 1.7) as usize % 3;
        // Text types itself, rests, deletes itself and rests again; it never jumps.
        let typed = |text: &str, speed: f32, offset: f32| -> String {
            let n = text.chars().count();
            let rest = 6;
            let cycle = 2 * (n + rest);
            let at = ((t + offset) * speed) as usize % cycle;
            let shown = if at < n {
                at
            } else if at < n + rest {
                n
            } else if at < 2 * n + rest {
                n - (at - n - rest)
            } else {
                0
            };
            text.chars().take(shown).collect()
        };
        for (input, text, off) in [
            (&self.in_name, "Ada Lovelace", 0.0),
            (&self.in_outlined, "ada@dodai.dev", 1.0),
            (&self.in_filled, "analytical engine", 2.0),
            (&self.search, "slider", 3.0),
        ] {
            let v = typed(text, 6.0, off);
            if input.read(cx).text() != v {
                input.update(cx, |i, cx| i.set_text(v, cx));
            }
        }
        self.demo_email = typed("ada@dodai.dev", 5.0, 0.0);
        self.demo_pin = typed("4821", 1.6, 0.0);
        self.demo_otp = typed("381946", 2.2, 0.0);
        // Only the sections showing these values rebuild; the flag is cleared once the
        // observers have run, so a click in a later update still rebuilds everything.
        self.demo_only = true;
        cx.notify();
        let me = cx.entity();
        cx.defer(move |cx| me.update(cx, |s, _| s.demo_only = false));
    }

    /// While a demo that plays on its own is showing, step it once per display frame. The
    /// step rides the frame that is drawn anyway (a scroll, say) instead of a free-running
    /// timer, whose extra draws fall between vsyncs and make scrolling stutter.
    pub(crate) fn tick_demos(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let recent = |t: Option<std::time::Instant>, ms: u128| {
            t.is_some_and(|t| t.elapsed().as_millis() < ms)
        };
        let playing = recent(self.demo_seen, 200) || recent(self.inst_seen, 100);
        if self.demo_tick_queued || !playing || !dodai_ui_gpui::motion::enabled() {
            return;
        }
        self.demo_tick_queued = true;
        cx.on_next_frame(window, |s, _, cx| {
            s.demo_tick_queued = false;
            s.demo_step(cx);
        });
    }

    /// Initial state; the TextInput entities are created here.
    pub fn build(cx: &mut Context<Self>) -> Self {
        // Test aid: start with series patterns on (`DODAI_PATTERNS=1`).
        if std::env::var_os("DODAI_PATTERNS").is_some() {
            dodai_ui_charts::gpui::common::set_patterns(cx, true);
        }
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
            viewport_w: 1100.0,
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
            icon_sel: std::env::var("DODAI_ICON_SEL")
                .ok()
                .and_then(|n| crate::sections::IconEntry::find(&n)),
            icon_code: std::env::var_os("DODAI_CODE").is_some(),
            icon_msg: None,
            icon_open: std::collections::HashSet::new(),
            icon_pick: Default::default(),
            date_range: Default::default(),
            range_times: ((9, 0), (17, 30)),
            cat: std::env::var("DODAI_CAT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            replay: 0,
            sec_replay: Default::default(),
            code_open: Default::default(),
            code_pick: Default::default(),
            ld_pb: dodai_ui_gpui::Playback::Loop,
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
            cat_count: 1,
            meta: Vec::new(),
            meta_family: "",
            section_views: Default::default(),
            built: Default::default(),
            real_h: Default::default(),
            quiet: false,
            scrolled_at: None,
            rest_pending: false,
            no_cull: std::env::var_os("DODAI_NOCULL").is_some(),
            sidebar_view: None,
            side_need: 0.0,
            side_spied: (None, true),
            overview_h: 0.0,
            overview_real: false,
            demo_only: false,
            live_sections: Default::default(),
            section_group: Default::default(),
            spy: Vec::new(),
            pin: dodai_ui_gpui::ScrollSpy::new(),
            // DODAI_SUB=Vehicle opens a sub-group (screenshots); the name must match exactly.
            sub: std::env::var("DODAI_SUB")
                .ok()
                .map(|v| &*Box::leak(v.into_boxed_str())),
            shut: Vec::new(),
            open_subs: Vec::new(),
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
            animation: None,
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
            inst_seen: None,
            excluded_slots: Default::default(),
            demo_seen: None,
            demo_tick_queued: false,
            demo_email: String::new(),
            tree_items: vec![
                dodai_ui_gpui::TreeItem::new("src")
                    .expanded(true)
                    .child(dodai_ui_gpui::TreeItem::new("lib.rs"))
                    .child(dodai_ui_gpui::TreeItem::new("charts.rs")),
                dodai_ui_gpui::TreeItem::new("examples")
                    .child(dodai_ui_gpui::TreeItem::new("gpui_showcase.rs")),
            ],
            tree_sel: None,
            gizmo_yaw: 0.6,
            gizmo_pitch: 0.45,
            demo_pin: String::new(),
            demo_otp: String::new(),
            live_a: dodai_ui_gpui::Live::prefilled(48, |i| live_signal(i, 0.9)),
            live_b: dodai_ui_gpui::Live::prefilled(60, |i| live_signal(i, 0.6)),
            speed_down: dodai_ui_gpui::Live::new(sections::THROUGHPUT_WINDOW),
            speed_up: dodai_ui_gpui::Live::new(sections::THROUGHPUT_WINDOW),
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

    /// Lay out every section of the page instead of only those near the viewport.
    pub fn set_no_cull(&mut self, on: bool) {
        self.no_cull = on;
    }

    /// Sections laid out in the last frame.
    pub fn realised_count(&self) -> usize {
        self.realised.len()
    }

    /// Sections of the page (every category).
    pub fn section_count(&self) -> usize {
        self.meta.len()
    }
}

/// The demo signal for live charts: sample `i` of a stream with pace `k`.
fn live_signal(i: u64, k: f32) -> f32 {
    let x = i as f32 * k;
    50.0 + 30.0 * (x * 0.21).sin() + 14.0 * (x * 0.67 + 1.0).sin() + 6.0 * (x * 1.9).sin()
}
