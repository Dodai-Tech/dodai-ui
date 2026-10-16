# Component coverage

Goal: every Material Design 3 component (https://m3.material.io/components) plus
Legendary extras, token-driven so any theme (Cyberpunk first, Material 3 planned)
can restyle them. Reference implementation: `legendary-gpui`.

Legend: ✅ done · 🟡 partial · ⬜ todo

| M3 component | gpui | Notes |
|---|---|---|
| Badges | ✅ `Badge` | |
| Bottom app bar | ⬜ | |
| Bottom sheets | ⬜ | |
| Buttons (common, icon, segmented) | ✅ `Button`, `Segmented` | icon button ⬜ |
| FAB / Extended FAB | ⬜ | |
| Cards | 🟡 `Panel` | elevated/filled/outlined variants ⬜ |
| Carousel | ⬜ | |
| Checkbox | ✅ | |
| Chips (assist/filter/input/suggestion) | 🟡 `Chip` | |
| Date / Time pickers | ⬜ | |
| Dialogs | ✅ `Dialog` | full-screen ⬜ |
| Divider | ✅ | |
| Lists | ✅ `ListItem` | |
| Menus | ✅ `Menu`, `Select` | |
| Navigation bar / drawer / rail | ⬜ | `Tabs`, `Breadcrumb`, `Pagination`, `Stepper` exist |
| Progress indicators | ✅ `ProgressBar`, `IndeterminateBar`, `Spinner` | |
| Radio button | ✅ | |
| Search | ⬜ | |
| Side sheets | ⬜ | |
| Sliders | ✅ `Slider` (horizontal/vertical) | range slider ⬜ |
| Snackbar | ✅ `Toast` | |
| Switch | ✅ `Toggle` | |
| Tabs | ✅ | |
| Text fields | ✅ `InputBox`, `TextInput` | |
| Tooltips | ✅ | |
| Top app bar / Toolbars | ⬜ | |

Legendary extras: `Knob` (rotary), `Kbd`, `Avatar`, `Table`, `Accordion`,
`Alert`, `Skeleton`, `ThemeSelector`, `Scanlines`.

Planned theme: **Material 3** (tonal palette, rounded shapes, no glow) — built with
`legendary_core::themes::Spec`; needs elevation/state-layer tokens in core.
