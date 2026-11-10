# dodai-gpui

GPUI adapter for Dodai UI: the themed components of four designs (Dodai, Cyberpunk, Material,
Neo-Brutalism). The designs are plain tokens and specs in `dodai-core`; this crate only paints
them. The chart and instrument crates in the repository (`dodai-charts`, `dodai-instruments`)
are built on it too, but are not published yet.

## Use it

```toml
[dependencies]
dodai-gpui = { version = "0.1", default-features = false, features = ["components", "dodai"] }
```

```rust,no_run
use dodai_gpui::prelude::*;

// In your app setup:
fn setup(cx: &mut gpui::App) {
    dodai_gpui::install(cx, dodai_gpui::dodai::dark());
}
```

## Features

| Feature | Adds |
|---|---|
| `components` | Buttons, inputs, tables, dialogs, pickers, navigation, … |
| `frost` | Frosted glass: on any component (`SurfaceFx::frost` for every container, `Surface::new().frost(1.0)` per component) over the backdrop or a wallpaper (`set_wallpaper`), and `FrostedScene` over a picture; blurred off the UI thread via `dodai-frost` (GPU first); on with `dodai` |
| `cyberpunk` / `material` / `neobrutalism` / `dodai` | The design itself, re-exported as `dodai_gpui::<design>` (`light()`, `dark()`, …), plus its icon pack and design-only widgets |
| `symbols` | Material Symbols glyphs (enabled by `material` and `dodai`) |

Each slice compiles on its own (CI checks it). With no features you get the kit the chart and
instrument crates are built on: theme store, `paint` (strokes, fills, shaped boxes), `plot`
(chart glow), `follow` (values easing to new data), `motion`, `live` (`Live` sample windows) and
the `entry_render!` / `follow_api!` / `callback!` / `corners_setter!` macros.

## Charts and instruments (not published yet)

- Every chart follows new data with springs that obey the app Motion set; `.animate(false)` or
  `.response(..)` override it per chart.
- Items pop and dim on hover and select (bar, pie, funnel, treemap, lollipop, radar, …).
- `Live` + `Series::live(..)` / `Sparkline::live(..)` / `Oscilloscope::stream(..)` stream
  samples with sub-sample scrolling. Geometry and logic live in `dodai-charts`.
- The top-bar Animate switch and Speed apply to everything (`motion::set_enabled`,
  `motion::set_speed`).
- Charts without an `id` key their animation state by call site plus their order in the frame, so
  several built on one line in a loop work; give them `.id(..)` if that order can change.

## Try it

```sh
cargo run -p dodai-showcase
```

`DODAI_CAT`, `DODAI_THEME`, `DODAI_ONLY` and `DODAI_SIZE` preselect a category, theme, section
and window size.

## Status

Version 0.1. The components are checked against the shared showcase by screenshots; mouse and keyboard behaviour is not covered on every control yet. Needs a recent stable Rust (see `rust-version`). MIT licensed.
