# dodai-gpui: gaps across the design systems

Findings from a read of `dodai-gpui` (201 public components, ~39k lines, 189 tests, 80 showcase
sections) against the four designs (Dodai, Material, Cyberpunk, Neo-Brutalism). Ordered by value.
Nothing here was verified with live clicks or keys; it comes from code and the unit/layout tests.

## 1. Interaction

- **Keyboard** is handled in 8 files only (`content`, `forms`, `sheets`, `select`, `popover`,
  `text_input`, `interact`, `display`). No arrow-key model for `Tabs`, `TreeView`/`TreeNavigation`,
  `Menu`, `Carousel`, `Slider`, `Stepper`, `Pagination`, `List`. A roving-focus helper shared by
  these would close most of it at once.
- **`disabled`** exists on 12 components. Missing on `Tabs` (per tab), `TreeView`, `Slider`,
  `Carousel`, `List`, `Chip`-like rows, `Stepper`.
- **Focus ring** is drawn through one path (`interact.rs`) and every design supplies
  `focus_ring`. Good; but components that never take focus never show it (see keyboard).
- **Reduced motion**: `set_speed` paces everything. `set_enabled(false)` stills what moves on
  its own (entries, value-in, loops, live data: `motion::ambient`, `time`) while answers to
  input (toggles, tabs, dialogs: `motion::pace`, `play_on_change`, `appear`) keep moving;
  `set_reduce_motion` stills both. The OS "reduce motion" setting is not read yet.
- **Frame requests** rely on `Window::on_next_frame` plus a notify. In an unfocused window with no
  input a lone tween (no 16 ms loop on that page) may not redraw; worth a regression test.

## 2. Design coverage (token overrides per design)

Trait `DesignSpec` has ~25 component tokens. Overrides per design:

| Token | Dodai | Material | Cyberpunk | Neo-Brutalism |
|---|---|---|---|---|
| `step` (stepper) | yes | **default** | yes | yes |
| `badge` | yes | **default** | yes | yes |
| `choice_toggle` | yes | yes | **default** | **default** |
| `icon_button`, `elevated_button` | yes / n/a | yes | **default** | **default** |
| `sizes` | yes | **default** | **default** | yes |
| `type_style` (type roles) | n/a | yes | **default** | **default** |
| `page_grid`, `fluid` | yes | yes | **default** | yes |
| `elevation_layers` | yes | n/a | **default** | n/a |

"default" is not a bug (it falls back to the shared tokens) but each is a place where the design
looks generic. Cyberpunk and Neo-Brutalism have no type-role overrides, so their headings are the
shared scale.

## 3. Selection motion per design (done)

`Look::selection` (a `SelectionMotion`) picks each design's own; `.animation()` / `.motion()`
override it on `Tabs`, `Segmented`, `Toggle`, `Checkbox` and `Radio` (`SelectionTrack` for rows).
- Dodai: stretch (leading edge first, on a spring); the tick pops in.
- Material: slide/underline, eased.
- Cyberpunk: glitch-in (jittering slices, colour-split ghosts), keeps the cut corners.
- Neo-Brutalism: the chosen block rises; a switch thumb jumps and nudges.
`Accordion` opens to its measured height on the design's travel; `FormField` shakes on a new
error. Still on the shared transition: nav bar/rail items, menus, popovers.
`Carousel` follows a drag or a two-finger swipe (rubber band past the ends) and throws to the page it
coasts to (`Glide::throw`, `fling_to`).

## 4. Themes and appearance

- Light/dark pairing is inconsistent: Cyberpunk is dark with `-light` siblings, Material has both,
  Neo-Brutalism and Dodai have `-dark` siblings only by name. `ThemeMeta.mode` exists but nothing
  uses it to "switch to the other mode of this theme".
- No **follow the OS appearance** and no `prefers-contrast` variant.
- Dodai now has four accent hues; the other designs have their own families. There is no accent
  hue for Dodai's dark that is independently tuned (hues are rotated, brightness-matched).

## 5. Tests

- Files with no tests: `menu`, `popover`, `frosted`, `switcher`, `count`, `testing`, all
  instruments and most charts (`charts_bar`, `charts_radial`, `charts_market`, …). Layout smoke
  tests cover them, interactions do not.
- Interaction tests cannot drive hover, keys or the clock; animation correctness (pill stretch,
  glitch, grow-in) is unit-tested only for its pure parts.

## 6. Accessibility and i18n

- No accessibility tree: gpui 0.2 has no a11y API, so labels, roles and live regions are absent.
- RTL is partial: `set_direction(Direction::Rtl)` mirrors the row arrow keys (`roving::next`) and `ListItem` (marker side, indent, order). Not mirrored: text-input caret and selection, sliders, chevron icons, other layouts' left/right padding.
- Number/date formatting is English-only (`format_date_time`, the time axis).

## 7. Hard-coded values still in components

`px(<literal>)` remains in `slider`, `paint`, `datatable`, `feedback`, `extras`, `text_input`,
`tabs` (the capsule padding, glitch amplitude), `nav`, plus ~640 geometry literals in
`surface_fx.rs`. The `no_raw_colours` test covers colours only; a matching check for spacing
would stop new ones.

## 8. Charts (from earlier work)

No shared value tooltip on line/bar; hover/select via `Focus` on RadialBar, Line, Bar, legends and
the newer charts (`Focus::pointer`), not yet on Treemap, Heatmap, Sankey or Violin; no bar dash patterns; no zoom/pan/brush; no log scale or second axis on bars; grow-in
of Violin, Rose, Renko and Gauge shares one start time per type.

Not built yet (chart types): circle packing, alluvial, 2D density contour, Q-Q plot, polar /
cyclical line, spiral, point & figure, tile-grid map, streaming heat strip. The newer charts
(sunburst to KPI tile) read out the hovered item under the chart; none has a floating tooltip.

## Status

Done: controls that run without a controlling parent; `Tabs` keys; `set_reduce_motion` (showcase
key `r`); `ThemeSwitcher::toggle_mode` + `toggle_theme_mode` (key `m`).

Since the findings above were written:
- **Keyboard** goes through one rule, `roving::next` (arrows, Home, End; skips disabled items):
  `Tabs`, `Pagination`, `Carousel`, `Menu` (up/down/Enter/Right/Left), `TreeView` (`tree_key`) and
  `Slider` (`key_value`: arrows, Page Up/Down, Home, End). `Stepper` is a display, not a control.
  `ListItem` rows take Enter and Space like any focused click target; there is no `List`
  container with arrow walking yet.
- **`disabled`**: `Tabs::disabled([i..])`, `Slider`, `Carousel`, `ListItem`, `TreeItem`.
- **OS settings**: `follow_system_appearance(window, cx)` (light/dark counterpart, live) and
  `follow_system_motion(cx)` (macOS, GNOME, Windows; best effort, read once). Neither is wired
  into the showcase.
- **Tokens**: Cyberpunk and Neo-Brutalism set their own type weights (`type_style`). Overrides
  that are still the shared default are a design choice, not a bug: Material `step` / `badge` /
  `sizes`; Cyberpunk `choice_toggle` / `icon_button` / `elevated_button` / `page_grid` / `fluid` /
  `elevation_layers`; Neo-Brutalism `choice_toggle` / `icon_button` / `fluid` / `elevation_layers`.
- **Tests**: `tests/interaction.rs` covers `Menu`, `List`, `Popover` (open, Escape, outside click),
  `ThemeSwitcherBar` and the light/dark mode switch; `tests/pods_render.rs` lays out every dash
  pod and look under every theme, `dodai-gpui-charts/tests/layout_all_themes.rs` does the same
  for the chart families, `render_smoke.rs` draws `FrostedScene`. Charts beyond the main families
  and the other instruments have degenerate-data tests only. `tests/no_new_spacing_literals.rs`
  stops new `px(<number>)` in components (38 lines remain: hairlines and zeros use
  `HAIRLINE` / `Pixels::ZERO`, gaps use `spacing.half()`; the rest is component geometry; the
  `surface_fx.rs` geometry is not `px(..)` and not covered).
- **OS follow**: the showcase has a Mode switch (System, Light, Dark) on `follow_system_appearance`
  / `set_theme_mode` and follows the OS's reduced motion at start.

Still open: full RTL, hover and clock in interaction tests, the chart items in section 8, the
per-design token choices above.
