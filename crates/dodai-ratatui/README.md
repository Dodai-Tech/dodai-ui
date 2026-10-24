# dodai-ratatui
Ratatui adapter for Dodai UI: `TuiTheme` (styles, chamfered blocks, buttons, tabs, badges, inputs), `Scanlines`, `ThemeBadge`, 256-colour fallback.

Components (all token-driven; border glyphs follow `Shape::kind()`: square `┌┐└┘`, rounded `╭╮╰╯`, chamfered `╱╲` on the cut corners only):
`TuiTheme::{block, button, button_block, tabs, badge, status_bar, input}`, `Tag` (solid/outline), `Toggle` (checkbox/radio/switch), `Progress` (eighth-block bar), `SelectList`, `Divider`, `InputField`, `Scanlines` (when `effects.scanlines`), `border_set`, `dim_color`. Labels honour `typography.uppercase_labels`.

Showcase: `cargo run -p dodai-ratatui --example ratatui_showcase` (`DODAI_DUMP=out.html [DODAI_THEME=n]` for a headless HTML frame).

## Ecosystem widgets

`TuiTheme` is a [`tui_nuances::ThemeProvider`], so any widget crate that speaks `Themed` is styled from Dodai tokens with one call. The adapter depends on and re-exports `tui-chart`, `tui-piechart`, `tui-spinner`, `tui-rating`, `tui-slider` and `tui-checkbox`:

```rust
use dodai_ratatui::{tui_checkbox::Checkbox, tui_piechart::PieChart, Themed, TuiTheme};

let tui = TuiTheme::new(dodai_core::Theme::default());
let cb = Checkbox::new("Neural link", true).themed(&tui);
```

The showcase's **Widgets** and **Charts** pages show them all (`DODAI_TAB=2` with `DODAI_DUMP` renders a page headlessly).
