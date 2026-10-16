# dodai-slint

[Slint](https://slint.dev) adapter for Dodai UI. Everything derives from
`dodai-core` tokens; no theme-id branching.

## Files
- `ui/dodai.slint` – import entry (re-exports `Theme` + components).
- `ui/components.slint` – `LFrame` (chamfer `Path` / rounded / square + glow),
  `LText` (kind 0 heading, 1 label, 2 muted, 3 mono, 4 body), `LButton` (variant 0-5
  primary/secondary/outline/outline-secondary/danger/ghost, `enabled`), `LPanel`,
  `LTag`, `LTab`/`LTabs`, `LToggle`, `LCheckbox`, `LRadio`, `LProgress`, `LSlider`,
  `LInput`, `LListItem`, `LDivider`, `LTooltip`, `LScanlines`.
- `ui/theme.slint` – generated `global Theme` (defaults = default theme). Regenerate:
  `cargo run -p dodai-slint --example slint_gen [theme-id] > ui/theme.slint`
  or call `generate_theme_slint(&theme)` from your `build.rs`.

## Use
```rust
slint::slint! { import { Theme, LButton } from "ui/dodai.slint"; export { Theme } /* … */ }
// push a theme (+ glow) at runtime; call once before showing
dodai_slint::install_theme!(Theme, &ui, &theme.with_glow_intensity(1.5));
```
Copy `ui/*.slint` next to your UI (or point `SLINT_INCLUDE_PATH` at it).
`install_theme!` also wires `Theme.label-case` so `uppercase_labels` themes upper-case labels.

Example: `cargo run -p dodai-slint --example slint_showcase` (theme buttons, `t`/`T`, glow ±).
