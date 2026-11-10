# dodai-core

Framework-agnostic design tokens (colour roles, typography, spacing, shape, effects),
resolved component styles, `ThemeSwitcher` and the four design systems of Dodai UI:
Cyberpunk, Material 3, Neo-Brutalism and Dodai. It knows no UI toolkit; the adapters
(`dodai-gpui`, `dodai-iced`, `dodai-bevy`, `dodai-dioxus`, `dodai-topcoat`, `dodai-ratatui`)
turn its tokens into widgets.

```toml
dodai-core = { version = "0.1", features = ["material"] }   # cyberpunk, material, neobrutalism, dodai
```

```rust
# #[cfg(feature = "material")] {
let mut sw = dodai_core::switcher();            // over every enabled design
let t = dodai_core::themes::by_id("material-dark");
# }
```

Every design is off by default; with no feature you get the engine and `themes::neutral()`.

## What is in it

* `Theme`, `Palette`, `Typography`, `Spacing`, `Shape` and `Effects`: one token set per theme.
  Components never branch on a theme id; they read shape, glow and type from the tokens.
* `ThemeSwitcher` and `themes::{all, by_id}`: switch themes at run time, persist `theme.id()`.
* Resolved component styles (`Theme::button`, `input`, `tab`, ...) for every control state.
* `Surface`, `SurfaceFx` and `Glass`: per-component fill opacity, frosted glass and surface
  effects, resolved once for every toolkit.
* `motion`: easing, springs, curves, tweens and mixins, driven by the clock you pass in.
* WCAG contrast helpers and an ANSI-256 fallback for terminals.
* DTCG token export (`export`).

Icons live in `dodai-icons`. See the [workspace README](https://github.com/sorinirimies/dodai-ui).

## Status

Version 0.1. The component set is checked against the shared showcase by screenshots; mouse
and keyboard behaviour is not covered on every control yet. Needs a recent stable Rust (see
`rust-version`). MIT licensed.
