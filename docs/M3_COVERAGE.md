# Material 3 coverage (dodai-gpui)

Checked against the component list at m3.material.io/components. Values come from the
`material-web` component tokens and from memory (the spec pages are JS-rendered); Expressive
items are unverified. ✓ = implemented and spec-tested, ◐ = partly, ✗ = missing.

| Component | Status | Notes |
|---|---|---|
| App bars: top (centre, small, medium, large), scrolled, collapse | ✓ | `TopAppBar`, spec hook `app_bar` |
| App bar: bottom | ✓ | `BottomAppBar` (80 px, FAB slot) |
| App bar: search | ✓ | `SearchBar` |
| Badges | ✓ | `Badge`, `IconBadge` |
| Buttons: filled, tonal, elevated, outlined, text | ✓ | `Button` (+ `.elevated`) |
| Buttons: sizes XS to XL, shape toggle (Expressive) | ✓ | `Button::size`, `.square`; `ControlSize` |
| Button groups, split button | ✓ | `ButtonGroup`, `SplitButton` |
| FAB, small, large, extended | ✓ | `Fab` |
| FAB menu | ✓ | `FabMenu` |
| Icon buttons (standard, filled, tonal, outlined, toggle) | ✓ | sizes XS to XL, narrow/default/wide (`IconWidth`) |
| Cards: elevated, filled, outlined | ✓ | |
| Carousel: uncontained, multi-browse, hero, centre-hero | ✓ | |
| Checkbox, radio | ✓ | sizes via `CheckboxVariant` |
| Chips: assist, filter, input, suggestion, elevated | ✓ | leading icon, check on filter |
| Date pickers (docked, modal, range) | ✓ | text input: `DateInput` |
| Dialogs: basic, full-screen | ✓ | icon header: `DialogCard::icon` |
| Divider | ✓ | |
| Lists: 1/2/3 line, leading, trailing text, overline | ✓ | trailing controls (switch, checkbox) by composition |
| Loading indicator | ✓ | |
| Menus | ✓ | `MenuItem::icon`, `::submenu` |
| Navigation bar, rail, drawer | ✓ | `ModalDrawer`, `Navigation::header` |
| Progress: linear, circular, wavy | ✓ | `ProgressBar::wavy`, `CircularProgress::wavy` |
| Search (docked, full-screen) | ✓ | `SearchView::recent` |
| Sheets: bottom, side | ✓ | |
| Sliders: continuous, discrete, range, centred, inset icon | ✓ | `Slider::centered`, `.icon` |
| Snackbar | ✓ | |
| Switch | ✓ | with icons ✓ |
| Tabs: primary, secondary, icons | ✓ | `Tabs::scrollable` |
| Text fields: filled, outlined | ✓ | floating label |
| Time pickers | ◐ | keyboard mode: `TimeInput`; analogue dial ✗ |
| Tooltips: plain, rich | ✓ | |
| Toolbars (Expressive: docked, floating, vibrant) | ✓ | `Toolbar::floating`, `.vibrant` |
| Colour: baseline + generated schemes | ✓ | 25 + 14 roles, `Scheme`, 7 styles, pastel and seeded presets |
| Shape scale and role mapping | ✓ | |
| Elevation (soft shadows), state layers | ✓ | |
| Typography: M3 type scale | ✓ | `TypeRole`, `Text`; list items, chips, tabs, menus and dialogs use it; buttons use the control-size text; text fields keep their fixed line height |
| Motion: easing curves, 16 duration tokens | ✓ | `dodai_motion::{DurationToken, material_easing}`; showcase "Motion tokens" |

# Cyberpunk coverage

Own `DesignSpec` hooks: card, chip, dialog, sheet, popup, scrim, step, tab, list_item, button,
input, **app_bar** (HUD plate, neon title, lights up when scrolled), **tooltip** (readout),
**snackbar** (glowing outline), **alert** (glow in tone), **badge** (outlined tag), **toggle**
(hollow switch), **progress** (10-segment gauge), **focus ring** (3 px).

Colour theming: 3 base variants × dark/light, plus `Neon::new(primary, secondary, accent)
.backdrop(Backdrop::…)` which yields a readable dark and light theme from any three colours
(5 backdrops), and 6 shipped `PRESETS` (magenta, acid, sunset, ultraviolet, ice, toxic).
