# Component coverage

Goal: every Material Design 3 component (https://m3.material.io/components) plus
Dodai extras, token-driven so any theme (Cyberpunk first, Material 3 planned)
can restyle them. Reference implementation: `dodai-gpui`.

Legend: ✅ done · 🟡 partial · ⬜ todo

| M3 component | gpui | Notes |
|---|---|---|
| Badges | ✅ `Badge`, `IconBadge` | |
| Bottom app bar | ✅ `BottomAppBar` | |
| Bottom sheets | ✅ `BottomSheet` | |
| Buttons (common, icon, segmented) | ✅ `Button`, `IconButton`, `Segmented` | |
| FAB / Extended FAB | ✅ `Fab` | |
| Cards | ✅ `Card` | elevated/filled/outlined |
| Carousel | ✅ `Carousel` | |
| Snap stack (vertical pages) | ✅ `SnapStack` | wheel notch, swipe, mouse drag or `.arrows(true)` chevrons turn a page; the snap is slow and eased |
| Checkbox | ✅ | |
| Chips (assist/filter/input/suggestion) | 🟡 `Chip` | |
| Date / Time pickers | ✅ `DatePicker`, `TimePicker` | |
| Dialogs | ✅ `Dialog`, `FullScreenDialog` | |
| Divider | ✅ | |
| Lists | ✅ `ListItem` | |
| Menus | ✅ `Menu`, `Select` | |
| Navigation bar / drawer / rail | ✅ `Navigation` | |
| Progress indicators | ✅ `ProgressBar`, `IndeterminateBar`, `Spinner` | |
| Radio button | ✅ | |
| Search | 🟡 `SearchBar` | shell only; wire `TextInput` |
| Side sheets | ✅ `SideSheet` | |
| Sliders | ✅ `Slider` (step, ticks, value readout, formatter), `RangeSlider` (horizontal + vertical) | |
| Snackbar | ✅ `Toast`, `Snackbar` (with action) | |
| Switch | ✅ `Toggle` | |
| Tabs | ✅ | |
| Text fields | ✅ `InputBox`, `TextInput` | |
| Tooltips | ✅ `DodaiTooltip`, `RichTooltip` | |
| Top app bar / Toolbars | ✅ `TopAppBar`, `Toolbar` | |

| Search view | ✅ `SearchView` (+ `SearchBar` shell) | |

Beyond M3 (gpui): `TreeView`, `Table` (sort + select), `Combobox`, `NumberStepper`,
`ColorPicker`, `FileDropZone`, `Timeline`, `Rating`, `TagField`, `SplitPane`,
`CommandPalette`, `EmptyState`, `AxisGizmo` (3D orientation widget: axis balls, home, drag to orbit; also in `dodai-bevy` as `axis_gizmo`; geometry in `dodai_instruments_core::orientation`).

Charts (gpui, `charts.rs`, palette `series` colours, glow on lines, themed bar shape):
`LineChart` (area/smooth), `BarChart` (grouped/stacked/horizontal), `PieChart`
(+ donut), `Sparkline`, `Gauge`, `RadarChart`, `ScatterPlot`, `Heatmap`, `Legend`.
More (`charts_trend.rs`, `charts_flow.rs`, `charts_distribution.rs`, `charts_part_whole.rs`): `StackedAreaChart`, `Histogram`, `Waterfall`, `Funnel`,
`Candlestick`, `BoxPlot`, `RadialBar`, `Treemap`.
Config: `.title()`, `.format()`, `.height()`; Line: `.y_range()`, `.stepped()`,
`.stroke_width()`, `.grid()`; Bar: `.max()`, `.value_labels()`, `.bar_width()`.
Showcase: `cargo run -p dodai-showcase`.

## Showcases in the other toolkits

`dodai-showcase-model` describes the showcase as data (pages, sections, a demo widget tree and
the state every control writes to); bevy, iced and dioxus each draw it with their own components
(`just run-bevy`, `just run-iced`, `just run-dioxus`). It covers Foundations, Shape, Actions,
Data display, Navigation, Inputs, Loading and Animation (easing curves and physics lanes as
`Node::Plot` polylines, drawn whole: no live entry card, moving dot or replay yet), plus the knob,
vertical sliders and scroll area. Not ported yet: charts, instruments, shaders, Surface effects,
the one-shot mixins and the shape tuning controls. Shared token styles of the small boxes (cards, alerts, chips, avatars, code, tooltips,
rows, steps, skeletons, popups) live in `dodai_core::Kit` (`Theme::kit`), so every toolkit and
the CSS read the same numbers.

Dodai extras: `Knob` (rotary), `Kbd`, `Avatar`, `Table`, `Accordion`,
`Alert`, `Skeleton`, `ThemeSelector`, `Scanlines`.

Planned theme: **Material 3** (tonal palette, rounded shapes, no glow) — built with
`dodai_core::themes::ThemeBuilder`; needs elevation/state-layer tokens in core.


## Motion

`Easing` (Linear/EaseIn/EaseOut/EaseInOut/CubicBezier), `Motion { duration, easing }` (`.millis`, `.speed`, `Motion::set_default`), composable `Curve` (`window`, `reverse`, `ping_pong`, `then`, `pipe`, `stagger`). `Spinner`/`IndeterminateBar`/`Skeleton` take `.motion(..)`. `Sparkline` has `.marker(Marker)` / `.all_points(bool)`.
