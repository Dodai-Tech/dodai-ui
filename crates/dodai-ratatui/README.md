# dodai-ratatui
Ratatui adapter for Dodai UI: `TuiTheme` (styles, chamfered blocks, buttons, tabs, badges, inputs), `Scanlines`, `ThemeBadge`, 256-colour fallback.

Components (all token-driven; border glyphs follow `Shape::kind()`: square `┌┐└┘`, rounded `╭╮╰╯`, chamfered `╱╲` on the cut corners only):
`TuiTheme::{block, button, button_block, tabs, badge, status_bar, input}`, `Tag` (solid/outline), `Checkbox` (design-aware, see below), `Toggle` (radio/switch), `Progress` (eighth-block bar), `SelectList`, `Divider`, `InputField`, `Scanlines` (when `effects.scanlines`), `border_set`, `dim_color`. Labels honour `typography.uppercase_labels`.

## Checkbox

`Checkbox::new(&tui, "Neural link", true)` draws the checkbox in the design system of the theme: the design comes from the theme's `Look` flags (tonal → Material, sinking hard-shadow controls → Neo-brutalism, cut corners or luminous → Cyberpunk, else Dodai) and the colours from its palette tokens. Glyphs are block elements, so they are gap-free in any terminal font.

```rust
use dodai_ratatui::{Checkbox, TuiTheme};
use dodai_core::ControlState;

let tui = TuiTheme::new(dodai_core::Theme::default());
let cb = Checkbox::new(&tui, "Neural link", true)
    .mixed(false)                  // partly checked
    .state(ControlState::Focused)  // hovered / pressed / focused light it, disabled dims it
    .block(true)                   // 5×3 instead of 3×1
    .readout(true)                 // //ONLINE, On, ON ... after the label
    .shadow(true);                 // Cyberpunk only (adds a row and a column); off by default
```

`Checkbox::design(CheckboxDesign::Material)` forces a design; `widget()` returns the underlying `tui_checkbox::Checkbox`. See `cargo run -p dodai-ratatui --example ratatui_checkboxes`.

Showcase: `cargo run -p dodai-ratatui --example ratatui_showcase` (`DODAI_DUMP=out.html [DODAI_THEME=n]` for a headless HTML frame).

## Ecosystem widgets

`TuiTheme` is a [`tui_nuances::ThemeProvider`], so any widget crate that speaks `Themed` is styled from Dodai tokens with one call. The adapter depends on and re-exports `tui-chart`, `tui-piechart`, `tui-spinner`, `tui-rating`, `tui-slider` and `tui-checkbox`:

```rust
use dodai_ratatui::{tui_checkbox::Checkbox, tui_piechart::PieChart, Themed, TuiTheme};

let tui = TuiTheme::new(dodai_core::Theme::default());
let cb = Checkbox::new("Neural link", true).themed(&tui);
```

The showcase's **Widgets** and **Charts** pages show them all (`DODAI_TAB=2` with `DODAI_DUMP` renders a page headlessly).
