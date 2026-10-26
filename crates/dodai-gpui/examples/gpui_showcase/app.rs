//! Window setup: shortcuts, menu, and building the [`Showcase`].

use super::*;
use dodai_gpui::TextInput;
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
        let t = dodai_gpui::motion::now();
        let wave = |speed: f32, phase: f32| 0.5 + 0.5 * (t * speed + phase).sin();
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
        self.rating = ((t * 0.7).rem_euclid(5.5) * 2.0).floor() / 2.0;
        self.stepper = (t * 0.8) as i32 % 11;
        self.switch = (t / 2.2) as i64 % 2 == 0;
        self.check = (t / 3.1) as i64 % 2 == 0;
        self.radio = (t / 1.7) as usize % 3;
        // Text types itself, pauses, then starts over.
        let typed = |text: &str, speed: f32, offset: f32| -> String {
            let n = text.chars().count();
            let at = ((t + offset) * speed) as usize % (n + 14);
            text.chars().take(at.min(n)).collect()
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
        cx.notify();
    }

    /// Initial state; the TextInput entities are created here.
    pub fn build(cx: &mut Context<Self>) -> Self {
        // While instruments animate, the page repaints every frame so the demo values drift
        // smoothly; elsewhere it stays idle.
        cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(16))
                .await;
            if this
                .update(cx, |s, cx| {
                    if dodai_gpui::instruments_animated()
                        && s.demo_seen.is_some_and(|t| t.elapsed().as_millis() < 200)
                    {
                        s.demo_step(cx);
                    }
                    if s.inst_seen.is_some_and(|t| t.elapsed().as_millis() < 100)
                        && dodai_gpui::instruments_animated()
                    {
                        cx.notify();
                    }
                })
                .is_err()
            {
                break;
            }
        })
        .detach();
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
            section_views: Default::default(),
            built: Default::default(),
            real_h: Default::default(),
            quiet: false,
            section_group: Default::default(),
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
            inst_trail: 2,
            inst_response: 0,
            inst_glow: 0,
            inst_seen: None,
            demo_seen: None,
            demo_email: String::new(),
            demo_pin: String::new(),
            demo_otp: String::new(),
            pulse_cfg: [0, 0, 0, 1, 1, 2, 1],
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
