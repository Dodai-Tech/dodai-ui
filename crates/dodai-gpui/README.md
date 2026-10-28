# dodai-gpui

GPUI adapter for Dodai UI: themed components, charts and instruments for four designs
(Dodai, Cyberpunk, Material, Neo-Brutalism). The designs are plain tokens and specs in their
own crates; this crate only paints them.

## Use it

```toml
[dependencies]
dodai-gpui = { version = "0.1", default-features = false, features = ["components", "charts", "dodai"] }
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
| `charts` | Charts, gauges and instruments (clusters, scopes, flight, tactical); live streams (`Live`) |
| `shaders` | GPU pixel shaders as `ShaderView` |
| `cyberpunk` / `material` / `neobrutalism` / `dodai` | The design itself, re-exported as `dodai_gpui::<design>` (`light()`, `dark()`, …), plus its icon pack and design-only widgets |
| `symbols` | Material Symbols glyphs (enabled by `material` and `dodai`) |

Each slice compiles on its own (CI checks it).

## Charts and instruments

- Every chart follows new data with springs that obey the app Motion set; `.animate(false)` or
  `.response(..)` override it per chart.
- Items pop and dim on hover and select (bar, pie, funnel, treemap, lollipop, radar, …).
- `Live` + `Series::live(..)` / `Sparkline::live(..)` / `Oscilloscope::stream(..)` stream
  samples with sub-sample scrolling. Geometry and logic live in `dodai-charts-core`.
- The top-bar Animate switch and Speed apply to everything (`set_instruments_animated`,
  `set_animation_speed`).
- Charts without an `id` key their animation state by call site plus their order in the frame, so
  several built on one line in a loop work; give them `.id(..)` if that order can change.

## Try it

```sh
cargo run -p dodai-gpui --example gpui_showcase
```

`DODAI_CAT`, `DODAI_THEME`, `DODAI_ONLY` and `DODAI_SIZE` preselect a category, theme, section
and window size.
