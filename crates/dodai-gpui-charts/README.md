# dodai-gpui-charts

Interactive, animated charts for gpui in the Dodai designs. Every chart eases to new data, highlights the hovered item, keeps a click selection and shows a readout line. Geometry and maths live in `dodai-charts-core`; the look comes from the active design through `dodai-gpui`.

```rust,no_run
use dodai_gpui_charts::{LineChart, Series};
let chart = LineChart::new("sales", [Series::new("2025", [3.0, 5.0, 4.0, 8.0])]);
```

See every chart in the showcase: `cargo run -p dodai-showcase`.
