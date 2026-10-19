//! Headless render of the *whole* showcase: every category (and so every
//! component section) under every built-in theme. Catches panics, key
//! collisions and bad geometry in components the small smoke page misses.

#![allow(dead_code, unused_imports, clippy::all)]

#[path = "../examples/gpui_showcase/main.rs"]
mod showcase;

use dodai_core::ThemeSwitcher;
use gpui::{px, size, TestAppContext};

#[gpui::test]
fn every_category_renders_under_every_theme(cx: &mut TestAppContext) {
    for theme in ThemeSwitcher::new().themes().to_vec() {
        cx.update(|cx| dodai_gpui::install(cx, theme.clone()));
        let (view, vcx) = cx.add_window_view(|_, cx| showcase::Showcase::build(cx));
        vcx.simulate_resize(size(px(1400.0), px(900.0)));
        let cats = vcx.update(|_, cx| view.read(cx).cat_count());
        assert!(cats >= 10, "showcase categories were not laid out: {cats}");
        for cat in 0..cats {
            view.update(vcx, |s, cx| {
                s.set_cat(cat);
                cx.notify();
            });
            vcx.run_until_parked();
        }
        // Narrow window exercises the responsive paths.
        vcx.simulate_resize(size(px(500.0), px(800.0)));
        vcx.run_until_parked();
    }
}

#[gpui::test]
#[ignore = "timing probe: cargo test --release -- --ignored --nocapture"]
fn frame_cost_probe(cx: &mut TestAppContext) {
    const TITLES: &[&str] = &[
        "Text",
        "Colour palette",
        "Type scale",
        "Spacing · Elevation",
        "Shape tokens",
        "Icons",
        "Glow · Scanlines",
        "Corner shape",
        "Border",
        "Buttons",
        "Toolbar · Snackbar",
        "Tags",
        "Rating · Stepper · Badges · Tooltips",
        "List · Divider",
        "Tree · timeline · empty state",
        "Tooltip",
        "Data table",
        "Code block · Markdown",
        "Image · Avatar group",
        "Circular progress · Banner",
        "Clock · Timer",
        "Stopwatch · Tap tempo",
        "Scroll area",
        "Popover · Hover card · Context menu · Dropdown",
        "Toggle · Checkbox · Radio",
        "Slider · Progress",
        "Slider (stepped, ticks, value readout)",
        "Range slider",
        "Knob · Vertical sliders",
        "Input",
        "Search view",
        "Tag field",
        "Date / time pickers",
        "Password · OTP",
        "Form field",
        "Toggle group · Checkbox group",
        "Date range",
        "Tabs",
        "Design system tree",
        "Collapsible sidebar",
        "App bar · Cards · FAB · Navigation",
        "Line & bar charts",
        "Pie · donut · gauge · sparkline",
        "Gauge styles",
        "Instruments",
        "Area · histogram · waterfall",
        "Funnel · treemap · radial bar",
        "Candlestick · box plot · bubble",
        "Market · planning charts",
        "Radar · scatter · heatmap",
        "Shader playground",
    ];
    cx.update(|cx| dodai_gpui::install(cx, dodai_core::Theme::default()));
    let (view, vcx) = cx.add_window_view(|_, cx| showcase::Showcase::build(cx));
    vcx.simulate_resize(size(px(1400.0), px(900.0)));
    vcx.run_until_parked();
    for title in TITLES {
        std::env::set_var("DODAI_ONLY", title);
        view.update(vcx, |_, cx| cx.notify());
        vcx.run_until_parked();
        let t = std::time::Instant::now();
        for _ in 0..10 {
            view.update(vcx, |_, cx| cx.notify());
            vcx.run_until_parked();
        }
        println!("{:6.2} ms  {title}", t.elapsed().as_secs_f64() * 100.0);
    }
}
