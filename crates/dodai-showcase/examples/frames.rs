//! One window with a grid of components, drawn like the design file's component frames, for
//! `scripts/compare-frames.nu`. Prints the grid on stdout as tab-separated lines:
//! `window W H`, then `name X Y` (the top-left of each component, in window points).
//!
//! `DODAI_THEME=<theme id>` picks the design. Add a component by adding one entry to [`frames`]:
//! the name must be the design frame's name.

use dodai_core::Tone;
use dodai_gpui::{hsla, theme_of, Divider, Kbd, ProgressBar, Tag};
use gpui::prelude::*;
use gpui::{div, px, size, AnyElement, App, Application, Bounds, Context, Window, WindowOptions};

const COLS: usize = 3;
/// Cell side in points; the component sits `PAD` in from the corner.
const CELL: (f32, f32) = (360.0, 140.0);
const PAD: f32 = 20.0;

type Make = fn() -> AnyElement;

/// Every component of the grid, by design-frame name.
fn frames() -> Vec<(&'static str, Make)> {
    vec![
        ("ProgressBar", || {
            div()
                .w(px(220.0))
                .child(ProgressBar::new(140.0 / 220.0))
                .into_any_element()
        }),
        ("Tag", || {
            div()
                .flex()
                .gap(px(8.0))
                .child(Tag::new("Info", Tone::Info).solid(true))
                .child(Tag::new("Done", Tone::Success).solid(true))
                .child(Tag::new("Wait", Tone::Warning).solid(true))
                .child(Tag::new("Error", Tone::Danger).solid(true))
                .into_any_element()
        }),
        ("Kbd", || Kbd::new("⌘ K").into_any_element()),
        ("Divider", || {
            div().w(px(200.0)).child(Divider::new()).into_any_element()
        }),
    ]
}

struct Grid;

impl Render for Grid {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = hsla(theme_of(cx).palette.background);
        let mut root = div().size_full().bg(bg).flex().flex_wrap().content_start();
        for (_, make) in frames() {
            root = root.child(
                div()
                    .w(px(CELL.0))
                    .h(px(CELL.1))
                    .flex_none()
                    .p(px(PAD))
                    .child(div().flex().child(make())),
            );
        }
        root
    }
}

fn main() {
    let (w, h) = (
        COLS as f32 * CELL.0,
        frames().len().div_ceil(COLS) as f32 * CELL.1,
    );
    println!("window\t{w}\t{h}");
    for (i, (name, _)) in frames().iter().enumerate() {
        let (c, r) = (i % COLS, i / COLS);
        println!(
            "{name}\t{}\t{}",
            c as f32 * CELL.0 + PAD,
            r as f32 * CELL.1 + PAD
        );
    }
    Application::new().run(move |cx: &mut App| {
        let mut sw = dodai_core::switcher();
        if let Ok(id) = std::env::var("DODAI_THEME") {
            sw.select(&id);
        }
        dodai_gpui::install_switcher(cx, sw);
        dodai_gpui::motion::set_enabled(cx, false);
        let bounds = Bounds::centered(None, size(px(w), px(h)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Grid),
        )
        .expect("open window");
        cx.activate(true);
    });
}
