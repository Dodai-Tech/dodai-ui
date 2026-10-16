# Legendary UI

A **design system** — not just a colour switcher — shipped as one crate per UI framework.
One token set (colour roles, typography, spacing, shape, glow/scanline effects, resolved
component styles) feeds four native adapters:

| Crate | Framework | What you get |
|---|---|---|
| [`legendary-core`](crates/legendary-core) | — | `Theme`, `Palette`, tokens, `ThemeSwitcher`, WCAG contrast, ANSI-256 fallback |
| [`legendary-ratatui`](crates/legendary-ratatui) | Ratatui 0.30 | `TuiTheme`: chamfered `╱ ╲` blocks, `◢ BUTTON ◤`, tabs, badges, inputs, status bar, `Scanlines`, `ThemeBadge` |
| [`legendary-iced`](crates/legendary-iced) | Iced 0.14 | `IcedTheme`: native `iced::Theme`, button/container/text-input/text styles with neon glow, canvas `ChamferFrame` |
| [`legendary-gpui`](crates/legendary-gpui) | GPUI 0.2 | `Global` theme store, `cx.legendary()`, `LegendaryStyled` (`.lg_panel()`, `.lg_button()`, `.lg_input()`) |
| [`legendary-topcoat`](crates/legendary-topcoat) | Topcoat / any SSR web | CSS custom properties, `.lg-btn/.lg-panel/.lg-input/.lg-tag/.lg-tabs/.lg-table`, `clip-path` chamfers, JS theme switcher |

## Themes

**Cyberpunk** (inspired by [cyberpunk.net](https://www.cyberpunk.net/us/en/); values taken from its stylesheet):

| Token | Value |
|---|---|
| Signature yellow | `#fcee0a` (hover `#f2e900`) |
| Cyan | `#00f0ff` |
| Red | `#fe1038` |
| Orange | `#ff6000` |
| Surfaces / lines | `#000` · `#333` · `#4c4c4c` · text `#fff` / `#cbcbcb` |
| Type | Blender Pro → Rajdhani fallback, UPPERCASE labels, tracked |
| Shape | 0 radius, 2 px borders, 48 px controls, 16 px chamfer (cut top-left + bottom-right) |
| Effects | yellow glow on hover/focus, optional CRT scanlines |

Glow is configurable: `theme.with_glow_intensity(1.5)` (0 = off, 1 = as designed) scales glow alpha and radius in every adapter.

Variants: `cyberpunk` (Night City), `cyberpunk-netrunner` (cyan-led), `cyberpunk-samurai` (red-led).
Every theme is contrast-tested (text ≥ 7:1, primary label ≥ 4.5:1).

## Use it

```toml
[dependencies]
legendary-core    = "0.1"
legendary-ratatui = "0.1"   # or -iced / -gpui / -topcoat
```

```rust
use legendary_core::{ThemeSwitcher, ButtonVariant, ControlState};
use legendary_ratatui::TuiTheme;

let mut sw = ThemeSwitcher::new();           // all built-in themes
let mut tui = TuiTheme::new(sw.current().clone());
tui.set_theme(sw.cycle_next().clone());      // switch at runtime — shape, type and glow change too
let btn = tui.button("Jack in", ButtonVariant::Primary, ControlState::Idle);
```

Persist `theme.id()` (e.g. `"cyberpunk-netrunner"`) in your config and restore with `switcher.select(id)`.

## Showcases

```sh
just run-ratatui   # t / T cycle themes
just run-iced
just run-gpui
just run-topcoat   # writes showcase.html and opens it
```

## Adding a theme

1. Add `crates/legendary-core/src/themes/<name>.rs` returning `Theme`s (copy `cyberpunk.rs`).
2. Register it in `themes::all()` — every adapter picks it up automatically.
3. `just test` — the shared contrast/uniqueness tests run on it.

## Development

```sh
just check-all     # fmt + clippy -D warnings + cargo tests + nushell tests
just bump 0.2.0    # gate → bump versions → git-cliff changelog → commit + tag
just release 0.2.0 # bump + push; CI publishes to crates.io in dependency order
```

Layout mirrors [gitkraft](../gitkraft) / [tokenburn](../tokenburn): `justfile`, `cliff.toml`
(conventional commits → changelog), `deny.toml`, Nushell scripts in `scripts/` with tests in
`scripts/tests/`, GitHub workflows `ci.yml` · `release.yml` · `security.yml`.

> GPUI needs the Xcode Metal toolchain on macOS; Linux needs the libs listed in `ci.yml`.

## License

MIT
