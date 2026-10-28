# dodai-core

Framework-agnostic design tokens (colour roles, typography, spacing, shape, effects),
resolved component styles, `ThemeSwitcher` and the four design systems of Dodai UI.

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
Icons live in `dodai-icons`. See the workspace README.
