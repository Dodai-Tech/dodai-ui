# dodai-material

The **Material 3** design system for [Dodai UI](https://github.com/sorinirimies/dodai-ui): a light/dark pair (baseline tonal palette), rounded shapes, mixed-case labels, flat surfaces, no glow.

```rust
use dodai_material as m3;
let themes = m3::variants();              // [light, dark]
// feature `icons` (default): the full Material Symbols catalogue
let home = m3::icons::outlined::get("home");
```

Features: `icons` (default, outlined), `rounded`, `sharp` select the Material Symbols style. Depends only on `dodai-core` (and optionally `dodai-material-icons`).
