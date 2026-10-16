# dodai-ratatui
Ratatui adapter for Dodai UI: `TuiTheme` (styles, chamfered blocks, buttons, tabs, badges, inputs), `Scanlines`, `ThemeBadge`, 256-colour fallback.

Components (all token-driven; border glyphs follow `Shape::kind()`: square `┌┐└┘`, rounded `╭╮╰╯`, chamfered `╱╲` on the cut corners only):
`TuiTheme::{block, button, button_block, tabs, badge, status_bar, input}`, `Tag` (solid/outline), `Toggle` (checkbox/radio/switch), `Progress` (eighth-block bar), `SelectList`, `Divider`, `InputField`, `Scanlines` (when `effects.scanlines`), `border_set`, `dim_color`. Labels honour `typography.uppercase_labels`.

Showcase: `cargo run -p dodai-ratatui --example ratatui_showcase` (`DODAI_DUMP=out.html [DODAI_THEME=n]` for a headless HTML frame).
