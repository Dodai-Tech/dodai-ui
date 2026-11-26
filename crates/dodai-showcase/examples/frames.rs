//! One window with a grid of components, drawn like the design file's component frames, for
//! `scripts/compare-frames.nu`. Prints the grid on stdout as tab-separated lines:
//! `window W H`, then `name X Y` (the top-left of each component, in window points).
//!
//! `DODAI_THEME=<theme id>` picks the design. Add a component by adding one entry to [`frames`]:
//! the name must be the design frame's name.

use dodai_core::Tone;
use dodai_gpui::{
    hsla, theme_of, Alert, Badge, Breadcrumb, Checkbox, Chip, CircularProgress, CodeBlock, Divider,
    Fab, Heading, IconButton, IndeterminateBar, Kbd, Knob, Label, Mono, Muted, NumberStepper,
    Pagination, ProgressBar, Rating, Skeleton, Stepper, Tag, Toast, Toggle,
};
use gpui::prelude::*;
use gpui::{div, px, size, AnyElement, App, Application, Bounds, Context, Window, WindowOptions};

/// Space between cells and around the grid, in points.
const GAP: f32 = 24.0;
/// Extra room in each cell beyond the design frame's size.
const SLACK: f32 = 40.0;
/// The grid wraps to a new row past this width.
const MAX_WIDTH: f32 = 1600.0;

type Make = fn() -> AnyElement;

/// Every component of the grid, by design-frame name.
fn frames() -> Vec<(&'static str, Make)> {
    vec![
        ("Switch states", || {
            div()
                .flex()
                .gap(px(16.0))
                .child(Toggle::new("sw-on", true))
                .child(Toggle::new("sw-off", false))
                .into_any_element()
        }),
        ("Checkbox states", || {
            div()
                .flex()
                .gap(px(16.0))
                .child(Checkbox::new("cb-on", true))
                .child(Checkbox::new("cb-off", false))
                .child(Checkbox::new("cb-mix", false).indeterminate(true))
                .into_any_element()
        }),
        ("Stepper", || {
            Stepper::new(["Cart", "Ship", "Pay"], 1).into_any_element()
        }),
        ("Skeleton", || {
            div()
                .w(px(200.0))
                .child(Skeleton::new("sk"))
                .into_any_element()
        }),
        ("IndeterminateBar", || {
            div()
                .w(px(220.0))
                .child(IndeterminateBar::new("ib-bar"))
                .into_any_element()
        }),
        ("CircularProgress", || {
            CircularProgress::new(0.6).into_any_element()
        }),
        ("NumberStepper", || {
            NumberStepper::new("ns", 3).into_any_element()
        }),
        ("CodeBlock", || {
            CodeBlock::new("cbk", "let x = 1;").into_any_element()
        }),
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
        ("Heading / Text / Label", || {
            div()
                .flex()
                .flex_col()
                .child(Heading::new("Heading").level(1))
                .child(Label::new("Body text"))
                .child(Muted::new("Label"))
                .child(Mono::new("Mono 0x2A"))
                .into_any_element()
        }),
        ("Mono / Muted", || {
            div()
                .flex()
                .flex_col()
                .child(Mono::new("fn main()"))
                .child(Muted::new("muted caption"))
                .into_any_element()
        }),
        ("Fab", || Fab::new("f", "+").into_any_element()),
        ("IconButton", || {
            IconButton::new("ib", "+")
                .icon(dodai_gpui::icons::IconName::Heart)
                .variant(dodai_core::ButtonVariant::Outline)
                .into_any_element()
        }),
        ("Rating", || Rating::new("r", 3.0).into_any_element()),
        ("Knob", || Knob::new("k", 0.5).into_any_element()),
        ("Pagination", || {
            Pagination::new("pg", 2, 10).into_any_element()
        }),
        ("Breadcrumb", || {
            Breadcrumb::new("bc", ["Home", "Library", "Data"]).into_any_element()
        }),
        ("Toast", || {
            Toast::new(Tone::Success, "Saved").into_any_element()
        }),
        ("Banner", || {
            div()
                .w(px(360.0))
                .child(Alert::new(Tone::Info, "Update available").banner(true))
                .into_any_element()
        }),
        ("Chip states", || {
            Chip::new("c", "Assist").into_any_element()
        }),
        ("IconBadge", || {
            Badge::new("3", Tone::Danger).into_any_element()
        }),
        ("Divider", || {
            div().w(px(200.0)).child(Divider::new()).into_any_element()
        }),
    ]
}

/// Where each component sits: a shelf layout of cells the size of the design frame (read from the
/// tab-separated `name w h` file in `DODAI_SIZES`), `GAP` apart, so nothing overlaps.
struct Layout {
    cells: Vec<(&'static str, Make, f32, f32, f32, f32)>,
    width: f32,
    height: f32,
}

fn layout() -> Layout {
    let sizes: std::collections::HashMap<String, (f32, f32)> = std::env::var("DODAI_SIZES")
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            Some((
                f.first()?.to_string(),
                (f.get(1)?.parse().ok()?, f.get(2)?.parse().ok()?),
            ))
        })
        .collect();
    let (mut x, mut y, mut row_h, mut width) = (GAP, GAP, 0.0f32, 0.0f32);
    let mut cells = Vec::new();
    for (name, make) in frames() {
        // room beyond the design frame, so a component that comes out bigger is shown whole
        let (w, h) = sizes.get(name).copied().unwrap_or((300.0, 140.0));
        let (w, h) = (w + SLACK, h + SLACK);
        if x > GAP && x + w > MAX_WIDTH {
            (x, y, row_h) = (GAP, y + row_h + GAP, 0.0);
        }
        cells.push((name, make, x, y, w, h));
        width = width.max(x + w + GAP);
        row_h = row_h.max(h);
        x += w + GAP;
    }
    Layout {
        cells,
        width,
        height: y + row_h + GAP,
    }
}

struct Grid;

impl Render for Grid {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = hsla(theme_of(cx).palette.background);
        let mut root = div().size_full().bg(bg).relative();
        let probe = std::env::var("DODAI_PROBE").is_ok_and(|v| v == "1");
        for (_, make, x, y, w, h) in layout().cells {
            if probe {
                // alignment self-test: a magenta square at each cell origin, read by `dodai-diff probe`
                root = root.child(
                    div()
                        .absolute()
                        .left(px(x))
                        .top(px(y))
                        .size(px(16.0))
                        .bg(gpui::rgb(0xff00ff)),
                );
                continue;
            }
            root = root.child(
                div()
                    .absolute()
                    .left(px(x))
                    .top(px(y))
                    .w(px(w))
                    .h(px(h))
                    .child(div().flex().child(make())),
            );
        }
        root
    }
}

fn main() {
    let lay = layout();
    let (w, h) = (lay.width, lay.height);
    println!("window\t{w}\t{h}");
    for (name, _, x, y, _, _) in &lay.cells {
        println!("{name}\t{x}\t{y}");
    }
    Application::new().run(move |cx: &mut App| {
        let mut sw = dodai_core::switcher();
        if let Ok(id) = std::env::var("DODAI_THEME") {
            sw.select(&id);
        }
        dodai_gpui::install_switcher(cx, sw);
        dodai_gpui::motion::set_enabled(cx, false);
        // the biggest display: the frames sit well there and a lock or covering window is rare
        let big = cx
            .displays()
            .into_iter()
            .max_by_key(|d| {
                let b = d.bounds();
                (f32::from(b.size.width) * f32::from(b.size.height)) as i64
            })
            .map(|d| d.id());
        let bounds = Bounds::centered(big, size(px(w), px(h)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
                display_id: big,
                ..Default::default()
            },
            |_, cx| cx.new(|_| Grid),
        )
        .expect("open window");
        cx.activate(true);
    });
}
