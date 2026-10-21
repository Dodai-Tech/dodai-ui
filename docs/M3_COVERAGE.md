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
| Buttons: sizes XS to XL, shape toggle (Expressive) | ✗ | |
| Button groups, split button | ✓ | `ButtonGroup`, `SplitButton` |
| FAB, small, large, extended | ✓ | `Fab` |
| FAB menu | ✗ | |
| Icon buttons (standard, filled, tonal, outlined, toggle) | ✓ | sizes XS to XL ✗ |
| Cards: elevated, filled, outlined | ✓ | |
| Carousel: uncontained, multi-browse, hero, centre-hero | ✓ | |
| Checkbox, radio | ✓ | sizes via `CheckboxVariant` |
| Chips: assist, filter, input, suggestion, elevated | ✓ | leading icon, check on filter |
| Date pickers (docked, modal, range) | ◐ | text-input mode ✗ |
| Dialogs: basic, full-screen | ✓ | icon header ✗ |
| Divider | ✓ | |
| Lists: 1/2/3 line, leading, trailing text, overline | ✓ | trailing controls (switch, checkbox) by composition |
| Loading indicator | ✓ | |
| Menus | ◐ | sub-menus ✗, leading icons ✗ |
| Navigation bar, rail, drawer | ◐ | modal drawer ✗, rail FAB/menu slot ✗ |
| Progress: linear, circular | ✓ | wavy (Expressive) ✗ |
| Search (docked, full-screen) | ✓ | history icons ✗ |
| Sheets: bottom, side | ✓ | |
| Sliders: continuous, discrete, range | ◐ | centred ✗, inset icon ✗ |
| Snackbar | ✓ | |
| Switch | ✓ | with icons ✓ |
| Tabs: primary, secondary, icons | ✓ | scrollable ✗ |
| Text fields: filled, outlined | ✓ | floating label |
| Time pickers | ◐ | keyboard input mode ✗ |
| Tooltips: plain, rich | ✓ | |
| Toolbars (Expressive: docked, floating) | ◐ | `Toolbar` approximates the docked one |
| Colour: baseline + generated schemes | ✓ | 25 + 14 roles, `Scheme`, 7 styles, pastel and seeded presets |
| Shape scale and role mapping | ✓ | |
| Elevation (soft shadows), state layers | ✓ | |
| Typography: M3 type scale | ✗ | components use the theme sizes, not the 15 M3 styles |
| Motion: emphasized easing, durations | ◐ | `fx.rs` mixins; duration tokens not exhaustive |

# Cyberpunk coverage

Own `DesignSpec` hooks: card, chip, dialog, sheet, popup, scrim, step, tab, list_item, button,
input, **app_bar** (HUD plate, neon title, lights up when scrolled), **tooltip** (readout),
**snackbar** (glowing outline), **alert** (glow in tone). Still on the neutral baseline: toggle,
progress, badge, focus ring (a segmented HUD progress bar would fit).

Colour theming: 3 base variants × dark/light, plus `Neon::new(primary, secondary, accent)
.backdrop(Backdrop::…)` which yields a readable dark and light theme from any three colours
(5 backdrops), and 6 shipped `PRESETS` (magenta, acid, sunset, ultraviolet, ice, toxic).
