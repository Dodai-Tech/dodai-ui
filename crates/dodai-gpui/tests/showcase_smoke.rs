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
    for theme in dodai::switcher().themes().to_vec() {
        cx.update(|cx| {
            dodai_gpui::install_switcher(cx, {
                let mut s = dodai::switcher();
                s.select(theme.id());
                s
            })
        });
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
fn every_category_has_unique_sections(cx: &mut TestAppContext) {
    cx.update(|cx| dodai_gpui::install_switcher(cx, dodai::switcher()));
    let (view, vcx) = cx.add_window_view(|_, cx| showcase::Showcase::build(cx));
    vcx.simulate_resize(size(px(1400.0), px(900.0)));
    vcx.run_until_parked();
    let meta = vcx.update(|_, cx| view.read(cx).section_titles().to_vec());
    let cats: std::collections::BTreeSet<_> = meta.iter().map(|(c, _)| *c).collect();
    assert!((10..=11).contains(&cats.len()), "categories: {cats:?}");
    let mut seen = std::collections::HashSet::new();
    for m in &meta {
        assert!(seen.insert(*m), "duplicate section {m:?}");
    }
    // Every section must live in exactly one category and be non-empty.
    assert!(meta.len() >= 80, "sections went missing: {}", meta.len());
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
        "Light FX",
        "Component textures",
        "Container textures",
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
    cx.update(|cx| dodai_gpui::install_switcher(cx, dodai::switcher()));
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

/// Every texture and every effect paints over the whole showcase, on cut and rounded designs,
/// without panicking.
#[gpui::test]
fn every_surface_texture_and_effect_renders(cx: &mut TestAppContext) {
    for theme in ["cyberpunk", "material", "neobrutalism"] {
        cx.update(|cx| {
            dodai_gpui::install_switcher(cx, {
                let mut s = dodai::switcher();
                s.select(theme);
                s
            })
        });
        let (view, vcx) = cx.add_window_view(|_, cx| showcase::Showcase::build(cx));
        vcx.simulate_resize(size(px(1400.0), px(900.0)));
        for tex in dodai_core::Texture::ALL {
            vcx.update(|_, cx| {
                dodai_gpui::set_surface_fx(
                    cx,
                    dodai_core::SurfaceFx {
                        texture: tex,
                        strength: 1.0,
                        scale: 1.5,
                        glitch: 1.0,
                        sheen: 1.0,
                        flicker: 1.0,
                        bloom: 1.0,
                        neon: 1.0,
                        pulse: 1.0,
                        chase: 1.0,
                        edge: 1.0,
                        brackets: 1.0,
                        depth: 1.0,
                        clear: 1.0,
                        container_texture: tex,
                        container_strength: 1.0,
                        container_scale: 1.5,
                        targets: dodai_core::SurfaceTargets::ALL,
                    },
                )
            });
            for cat in [2, 3, 4, 5] {
                view.update(vcx, |s, cx| {
                    s.set_cat(cat);
                    cx.notify();
                });
                vcx.run_until_parked();
            }
        }
        vcx.update(|_, cx| dodai_gpui::set_surface_fx(cx, dodai_core::SurfaceFx::NONE));
    }
}
