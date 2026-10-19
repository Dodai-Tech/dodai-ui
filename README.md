# Dodai UI

A **design system** — not just a colour switcher — shipped as one crate per UI framework.
One token set (colour roles, typography, spacing, shape, glow/scanline effects, resolved
component styles) feeds four native adapters:

| Crate | Framework | What you get |
|---|---|---|
| [`dodai-core`](crates/dodai-core) | — | `Theme`, `Palette`, tokens, `ThemeSwitcher`, WCAG contrast, ANSI-256 fallback |
| [`dodai-charts-core`](crates/dodai-charts-core) | — | Renderer-agnostic chart maths: ticks, Heikin-Ashi, indicators, Renko, depth, Gantt, binning, gradients |
| [`dodai-shaders-gpu`](crates/dodai-shaders-gpu) | wgpu | the same shaders on the GPU (rust-gpu → WGSL); `dodai-shaders-kernels` is the build source |
| [`dodai-shaders`](crates/dodai-shaders) | — (CPU) | 195 pure-Rust pixel shaders: Ghibli-style painted and cel-shaded 3D scenes, cyberpunk (rainy megacities, Blade Runner-inspired scenes), Material 3, Matrix, CRT/VHS/retro monitors, Ghostty ports, DMT hyperspace sacred geometry, pixel-art platformer/metroidvania, cosmic; follows the theme palette |
| [`dodai-ratatui`](crates/dodai-ratatui) | Ratatui 0.30 | `TuiTheme`: chamfered `╱ ╲` blocks, `◢ BUTTON ◤`, tabs, badges, inputs, status bar, `Scanlines`, `ThemeBadge` |
| [`dodai-iced`](crates/dodai-iced) | Iced 0.14 | `IcedTheme`: native `iced::Theme`, button/container/text-input/text styles with neon glow, canvas `ChamferFrame` |
| [`dodai-gpui`](crates/dodai-gpui) | GPUI 0.2 | `Global` theme store, `cx.dodai()`, `DodaiStyled` (`.dodai_panel()`, `.dodai_button()`, `.dodai_input()`) |
| [`dodai-topcoat`](crates/dodai-topcoat) | Topcoat / any SSR web | CSS custom properties, `.dodai-btn/.dodai-panel/.dodai-input/.dodai-tag/.dodai-tabs/.dodai-table`, `clip-path` chamfers, JS theme switcher |

## Themes

**Cyberpunk** (a cyberpunk-inspired theme):

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

Variants: `cyberpunk` (yellow), `cyberpunk-netrunner` (cyan), `cyberpunk-samurai` (red), each also as a light theme on warm paper: `cyberpunk-light`, `cyberpunk-netrunner-light`, `cyberpunk-samurai-light` (same chamfered shape, deeper neons for contrast, glow dialled down).

**Material 3** (a Material-3-inspired light/dark pair): `material`, `material-dark` — rounded, mixed-case, flat tonal surfaces, no glow.
Every theme is contrast-tested (text ≥ 7:1, primary label ≥ 4.5:1).

## Use it

```toml
[dependencies]
dodai         = "0.1"   # everything: tokens + Cyberpunk + Material 3 + icons + shaders
dodai-ratatui = "0.1"   # or -iced / -gpui / -topcoat
```

```rust
use dodai::{ThemeSwitcher, ButtonVariant, ControlState};
use dodai_ratatui::TuiTheme;

let mut sw = dodai::switcher();              // all built-in themes
let mut tui = TuiTheme::new(sw.current().clone());
tui.set_theme(sw.cycle_next().clone());      // switch at runtime — shape, type and glow change too
let btn = tui.button("Continue", ButtonVariant::Primary, ControlState::Idle);
```

Persist `theme.id()` (e.g. `"cyberpunk-netrunner"`) in your config and restore with `switcher.select(id)`.

## Pick only what you need

Every design system and icon pack is its own crate; `dodai` is the one-stop
umbrella (features `cyberpunk`, `material`, `icons`, `shaders`, all on by default).

| Crate | What it is |
|---|---|
| [`dodai-core`](crates/dodai-core) | Tokens, `Theme`, `ThemeSwitcher`, `Spec` builder, DTCG export. No concrete themes. |
| [`dodai-cyberpunk`](crates/dodai-cyberpunk) | The 6 Cyberpunk themes (dark + light) + `icons` feature |
| [`dodai-material`](crates/dodai-material) | The Material 3 light/dark pair + `icons` feature |
| [`dodai-cyberpunk-icons`](crates/dodai-cyberpunk-icons) | 136 chamfered, curve-free icons. Standalone, no deps |
| [`dodai-material-icons`](crates/dodai-material-icons) | The full Material Symbols catalogue (3,900+ icons, outlined/rounded/sharp + fill). Standalone, no deps |
| [`dodai-shaders`](crates/dodai-shaders) | Pixel shaders |
| [`dodai`](crates/dodai) | All of the above behind features |

```toml
dodai-core     = "0.1"    # just the engine
dodai-material = "0.1"    # + the Material 3 themes and icons (features: rounded, sharp)
```

## Showcases

```sh
just run-ratatui   # t / T cycle themes
just run-iced
just run-gpui
just run-topcoat   # writes showcase.html and opens it
```

## Motion (gpui)

Entry effects, value-in (grow) for charts/sliders/gauges, `CountUp`, eased toggle/tab/accordion
transitions, hover/press blend and a shape-adaptive `Shimmer`, all driven by `Motion`/`EntrySet`.
Try them per section in `just run-gpui`.

## Importable tokens and icons

`just export` (or `cargo run -p dodai-topcoat --example export -- dist`) regenerates `dist/`, which is
committed and checked in CI:

| File | For |
|---|---|
| `dist/tokens/dodai.tokens.json`, `dist/tokens/<theme>.tokens.json` | W3C Design Tokens (DTCG): Style Dictionary v4, Tokens Studio / Figma Variables, Penpot |
| `dist/css/dodai.css` | CSS custom properties (`--dodai-*`, `:root[data-theme=…]`) plus component CSS |
| `dist/icons/svg/*.svg`, `dist/icons/dodai-cyberpunk-icons.svg` | The icon pack as SVG (`currentColor`), one file each and as a `<symbol>` sprite |

The pack (`dodai-cyberpunk-icons`) has 136 icons in 7 groups, all straight lines, 45° chamfers and octagons.

## Charts only (gpui)

`dodai-gpui` has two default features, `components` and `charts`. For charts without buttons, tables
or dialogs: `dodai-gpui = { version = "0.1", default-features = false, features = ["charts"] }`.

Instrument-style pieces: `Gauge` with `GaugeStyle` (arc, segmented, dial, meter, reticle, bar, chevron, level)
and `Oscilloscope`, `Spectrum`, `RadarScope`, `HexMap`, `RingStack`, `DotMatrix`, `SignalBars`.

## Shaders

`dodai-shaders` is a dependency-free set of per-pixel shaders written as plain Rust functions
(generators like `neon_grid`, `digital_rain`, `platformer`, post effects like `crt`, `vhs`,
`gameboy`), drawn from the active theme's palette. In gpui: `ShaderView::new("id", "neon_grid").fx("crt")`
(feature `shaders`). Demo of everything: `just run-shaders`; also the *Shaders* category of the showcase.

## Adding a theme

1. Copy `crates/dodai-material` to `crates/dodai-<name>` (themes via `dodai_core::themes::Spec`); optionally add an icon crate.
2. Add it as a feature in `crates/dodai` and to `themes::all()` — every adapter picks it up through `dodai::switcher()`.
3. `just test` — the shared contrast/uniqueness tests in `crates/dodai/tests` run on it.

## Development

```sh
just check-all     # fmt + clippy -D warnings + cargo tests + nushell tests
just bump 0.2.0    # gate → bump versions → git-cliff changelog → commit + tag
just release 0.2.0 # bump + push; CI publishes to crates.io in dependency order
just push-all      # push to GitHub + Gitea remotes (`just setup-remotes` adds them)
```

Layout mirrors [gitkraft](../gitkraft) / [tokenburn](../tokenburn): `justfile`, `cliff.toml`
(conventional commits → changelog), `deny.toml`, Nushell scripts in `scripts/` with tests in
`scripts/tests/`, GitHub workflows `ci.yml` · `release.yml` · `security.yml`.

> GPUI needs the Xcode Metal toolchain on macOS; Linux needs the libs listed in `ci.yml`.

## License

MIT
