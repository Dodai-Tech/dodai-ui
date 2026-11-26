//! One window with a grid of components, drawn like the design file's component frames, for
//! `scripts/compare-frames.nu`. Prints the grid on stdout as tab-separated lines:
//! `window W H`, then `name X Y` (the top-left of each component, in window points).
//!
//! `DODAI_THEME=<theme id>` picks the design. Add a component by adding one entry to [`frames`]:
//! the name must be the design frame's name.

use dodai_core::Tone;
use dodai_gpui::{
    hsla, theme_of, Alert, Badge, Breadcrumb, Chip, Divider, Fab, Heading, IconButton, Kbd, Knob,
    Label, Mono, Muted, Pagination, ProgressBar, Rating, Tag, Toast,
};
use gpui::prelude::*;
use gpui::{div, px, size, AnyElement, App, Application, Bounds, Context, Window, WindowOptions};

/// Space between cells and around the grid, in points.
const GAP: f32 = 24.0;
/// The grid wraps to a new row past this width.
const MAX_WIDTH: f32 = 1600.0;

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
        ("Heading / Text / Label", || {
            div()
                .flex()
                .flex_col()
                .child(Heading::new("Heading").level(2))
                .child(Label::new("Label"))
                .into_any_element()
        }),
        ("Mono / Muted", || {
            div()
                .flex()
                .flex_col()
                .child(Mono::new("mono"))
                .child(Muted::new("muted"))
                .into_any_element()
        }),
        ("Fab", || Fab::new("f", "+").into_any_element()),
        ("IconButton", || {
            IconButton::new("ib", "+").into_any_element()
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
            Alert::new(Tone::Info, "Heads up").into_any_element()
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
        let (w, h) = sizes.get(name).copied().unwrap_or((300.0, 140.0));
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
        for (_, make, x, y, w, h) in layout().cells {
            root = root.child(
                div()
                    .absolute()
                    .left(px(x))
                    .top(px(y))
                    .w(px(w))
                    .h(px(h))
                    .overflow_hidden()
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
