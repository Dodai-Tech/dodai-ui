# dodai-dash

A dashboard engine for the Dodai UI design system. It knows what a dashboard **is**, how to check one, how to move one between people and programs as TOML or JSON, and how to turn one into plain data a toolkit can draw. It does not collect data, store data or draw anything: you plug your own metrics and your own data in.

```text
   host app                         dodai-dash                       toolkit crate
┌──────────────┐  Catalogue   ┌────────────────────┐             ┌──────────────────┐
│ its metrics, │─────────────►│ format · validate  │             │ dodai-gpui       │
│ units,       │              │ import · export    │  DashboardView  │ dodai-dioxus     │
│ dimensions   │  SeriesSource│ layout · variables │────────────►│ dodai-ratatui    │
│ its data     │◄─────────────│ resolve            │  (plain data)│ SVG on a server  │
└──────────────┘  Query       └────────────────────┘             └──────────────────┘
```

## What it is

- **A format**: a dashboard is panels on a 12-column grid, with variables written `$name`, a time range, an optional comparison period (the same range a day ago, laid over today), and display thresholds in semantic tones (`info`, `success`, `warning`, `danger`) that every Dodai design colours for itself.
- **Import and export**: TOML for people, JSON for tools, one model, converted without loss.
- **Validation**: against the format always, and against **your** metrics when you give it a catalogue.
- **A resolver**: asks your data source for each panel and returns a `DashboardView` of series and render hints. A panel that fails becomes an error panel; the others still draw.

## What it is not

It has no business logic and no monitoring logic. It is not a monitoring product.

| Stays with the host | Why |
|---|---|
| What the metrics mean: names, units, dimensions | you supply a `Catalogue` |
| Collecting, storing, rolling up, retaining data | you answer `SeriesSource::query`, already aggregated |
| Alert rules, evaluation, notifications | a threshold here only **colours** a value; it never alerts |
| Domain rules (cost, grading, scheduling, ...) | none of it is in this crate |
| Drawing | toolkit crates take the view model; charts come from `dodai-charts` |
| A query language | none: a panel picks a metric, filters, one group-by and an aggregation |

There are no network calls, no files read, no clock read and no code executed. Time (`now`) is passed in, so a resolved dashboard is reproducible and testable.

## The format

```toml
version = 1
name = "Overview"
refresh = "5s"          # how often a host should re-query
range = "1h"            # the time range, ending now
compare = "1d"          # also show the same range this long ago

[[variables]]           # chosen by the viewer, used as $name in filters and titles
name = "group"
default = "all"

[[panels]]
title = "Power of $group"
kind = "stacked_area"   # line area stacked_area bar gauge stat heatmap treemap sankey table state_timeline
metric = "gpu.power_w"  # a name from the host's catalogue
group_by = "host"       # one series per value of this dimension
filter = { group = "$group" }
agg = "sum"             # sum avg min max count last p50 p95 p99 rate delta
unit = "W"
span = { x = 0, y = 0, w = 6, h = 3 }     # omit it to place the panel automatically
thresholds = [{ above = 3000.0, tone = "warning" }]   # the last matching entry wins
```

Rules worth knowing:

- **Grid**: 12 columns; a span needs `w` of 1 to 12 and `h` of at least 1. Panels without a span flow left to right below the placed ones, each at a default size for its kind. A narrow screen shows panels in reading order, one under another.
- **Variables** are `$name` or `${name}` in filter values and titles; each must be declared. A default is used until the viewer picks another value.
- **Time spans** are a whole number and a unit: `5s`, `15m`, `1h`, `7d`, `2w`.
- **Thresholds** have `above`, `below` or both (then the value must be in the band). The last matching entry wins, so list them from mild to severe.
- **Bundles**: one file may hold several dashboards under `[[dashboards]]` to share a set.

Full examples: [`samples/overview.toml`](samples/overview.toml), the same dashboard as [`samples/overview.json`](samples/overview.json), and a [`bundle`](samples/bundle.toml). A tested file keeps them valid.

## TOML and JSON, import and export

Both formats go through one serde model, so a dashboard converts between them without loss. TOML is friendlier to write by hand; JSON is easier for tools and scripts.

```rust
use dodai_dash::{convert, import, Dashboard, Format};

let toml = std::fs::read_to_string("overview.toml")?;

// Read: the format is detected from the content, or pass a hint (from a file extension).
let parsed = Dashboard::import(&toml, None)?;
for field in &parsed.unknown_fields {
    eprintln!("ignored: {field}");   // fields this version does not know, by path
}
let dashboard = parsed.value;

// Write either way.
let json = dashboard.export(Format::Json)?;
let toml = dashboard.export(Format::Toml)?;

// Or convert text directly.
let json = convert(&toml, None, Format::Json)?;

// A file may hold one dashboard or a bundle.
let doc = import(&json, Some(Format::Json))?.value;
let dashboards = doc.into_dashboards();
```

An import is safe by construction:

- **Data only**: nothing is run, nothing is fetched, nothing is looked up.
- **Size-limited**: text over 1 MiB is refused before parsing (`MAX_IMPORT_BYTES`).
- **Version-checked**: a file with a version this build does not read is refused with a clear error; a missing version means the current one.
- **Unknown fields are reported**, not fatal, and are dropped on re-export.
- **Errors name the problem**: the format that was tried, the position the parser gave, and for an unknown panel kind the kind itself.
- **Validate afterwards**: `dashboard.validate(Some(&catalogue))` checks metric names (with "did you mean" for typos), aggregations, dimensions for grouping and filtering, panel kinds and units against the host's metrics; and without a catalogue, the format itself (names, spans, thresholds, scale, variables, overlaps, limits).
- **Mapping after import**: `dashboard.fixed_filters()` lists the fixed values a file names (a machine, a group); a host can check them against its own data and call `remap_filter` to point them at what exists.

A command-line converter ships as an example: `cargo run -p dodai-dash --example convert -- overview.toml json`.

### JSON Schema

With the `schema` feature, `dodai_dash::schema()` returns a JSON Schema of the format (one dashboard or a bundle), also committed as [`schema/dashboard.schema.json`](schema/dashboard.schema.json) and kept current by a test. Editors can use it to validate and complete both the JSON and, with a TOML-aware tool, the TOML form. Regenerate it with:

```sh
cargo run -q -p dodai-dash --features schema --example schema > crates/dodai-dash/schema/dashboard.schema.json
```

## Plugging it in

A host implements two traits.

```rust
use dodai_dash::{
    resolve, Aggregation, Dashboard, MetricInfo, PanelKind, Query, Series, SeriesSource,
    SourceError, StaticCatalogue, Variables,
};

// 1. What exists: names, units, dimensions, allowed aggregations and panel kinds.
let catalogue = StaticCatalogue::new([MetricInfo::new("gpu.power_w")
    .describe("Board power")
    .unit("W")
    .dimensions(["host", "gpu", "group"])
    .aggregations([Aggregation::Sum, Aggregation::Avg, Aggregation::Max])
    .kinds([PanelKind::Line, PanelKind::StackedArea, PanelKind::Stat])]);
// `Catalogue` is a trait: implement it over whatever the host already has instead.

// 2. The data: answer a query with series, already filtered, grouped and aggregated.
struct MyData;
impl SeriesSource for MyData {
    fn query(&self, q: &Query) -> Result<Vec<Series>, SourceError> {
        // q.metric, q.filter, q.group_by, q.agg, q.from..q.to (ms), q.step
        Ok(vec![])
    }
}

// 3. Validate, resolve, hand the view model to a toolkit.
let report = dashboard.validate(Some(&catalogue));
assert!(report.is_ok());
let view = resolve(&dashboard, &Variables::new(), &MyData, now_ms);
for panel in &view.panels {
    // panel.kind, panel.series, panel.span, panel.unit, panel.thresholds, panel.error ...
    // panel.tone(value) is the Dodai tone for a value under the panel's thresholds
    // (convert with `dodai_core::Tone::from`).
}
```

Two implementations are typical: one over a database (long ranges, rollups) and one over a small in-memory ring buffer (a device showing its own recent data with no server). A test implementation over fake data makes dashboards testable without any backend.

## Panel kinds

| Kind | Typical use |
|---|---|
| `line`, `area`, `stacked_area` | a value over time; shares over time |
| `bar` | totals per group or per step |
| `gauge` | one value on a scale (`min`, `max`) |
| `stat` | one big number with a small trend |
| `heatmap` | a value by two dimensions, such as host and hour |
| `treemap`, `sankey` | where a total went; flows between groups |
| `table` | rows of groups with their values |
| `state_timeline` | states over time as coloured spans |

The crate decides nothing about how they look: `dodai-charts` supplies charts and gauges, and each toolkit crate styles them from the active design.

## Where it sits

- Depends only on `dodai-core` (for the `Tone` mapping); nothing else in Dodai depends on this crate, so design-only users never build it.
- No toolkit code. A renderer per toolkit (native widgets, terminal, SVG for the web or a server) takes the `DashboardView`.
- Works anywhere Rust does: no async runtime, no threads, no I/O.

## Status

First version of the format (`version = 1`); the API is new and may change before a release. Not published yet.

Not in this version: event and log panels (they need an event source in addition to series), a stored dashboard library, and any renderer.

## Tests

```sh
cargo test -p dodai-dash --features schema
cargo clippy -p dodai-dash --all-targets --features schema -- -D warnings
```

## Licence

MIT, like the rest of Dodai UI.
