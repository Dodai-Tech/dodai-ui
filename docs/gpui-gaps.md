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
- **Reduced motion**: `motion::set_reduce_motion`, `set_enabled` and `set_speed` pace every
  component animation (`motion::pace`, `play_on_change`). The OS "reduce motion" setting is not
  read yet.
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
- No RTL: layouts use left/right padding and chevrons are fixed; text input has no bidi handling.
- Number/date formatting is English-only (`format_date_time`, the time axis).

## 7. Hard-coded values still in components

`px(<literal>)` remains in `slider`, `paint`, `datatable`, `feedback`, `extras`, `text_input`,
`tabs` (the capsule padding, glitch amplitude), `nav`, plus ~640 geometry literals in
`surface_fx.rs`. The `no_raw_colours` test covers colours only; a matching check for spacing
would stop new ones.

## 8. Charts (from earlier work)

No shared value tooltip on line/bar; hover/select via `Focus` only on RadialBar, Line, Bar and
legends; no bar dash patterns; no zoom/pan/brush; no log scale or second axis on bars; grow-in
of Violin, Rose, Renko and Gauge shares one start time per type.

## Status

Done: controls that run without a controlling parent (`Toggle`, `Checkbox`, `Radio`, `Tabs`,
`Segmented`, `ButtonGroup`, `Carousel`, `Select`, `Table`, `Slider`, `Rating`, `Pagination`,
`ColorPicker`, `DatePicker`, `TimePicker`; helper `own.rs`); `Tabs` arrow/Home/End keys;
`set_reduce_motion` (showcase key `r`); `ThemeSwitcher::toggle_mode` + `toggle_theme_mode`
(showcase key `m`).

Still open: keyboard for the rest (`TreeView`, `Menu`, `Slider`, `Stepper`, `List`), per-tab
`disabled`, OS appearance / reduce-motion detection, the per-design token gaps in section 2,
RTL, tests for `menu` / `popover`, the hard-coded literals, the chart items in section 8.
