# dodai-charts

Renderer-agnostic chart maths, layout and indicators for the Dodai UI design system. No UI dependencies; toolkit crates paint the results.

## gpui (feature `gpui`)

Interactive, animated charts for gpui in the Dodai designs. Every chart eases to new data, highlights the hovered item, keeps a click selection and shows a readout line. Geometry and maths live in `dodai-charts`; the look comes from the active design through `dodai-gpui`.

```rust,no_run
use dodai_charts::gpui::{LineChart, Series};
let chart = LineChart::new([Series::new("2025", [3.0, 5.0, 4.0, 8.0])]);
```

Turn on a design with a feature (`dodai`, `cyberpunk`, `material` or `neobrutalism`) and install its themes once with `dodai_gpui::install_switcher(cx, dodai_core::switcher())`. Without a design the charts use a plain neutral theme.

See every chart in the showcase: `cargo run -p dodai-showcase`.
