# dodai

Dodai UI in one dependency: `dodai-core` tokens (re-exported at the root), every design system, icon packs and shaders.

```toml
dodai = "0.1"                                                    # everything
dodai = { version = "0.1", default-features = false, features = ["material"] }
```

| feature | adds |
|---|---|
| `cyberpunk` | `dodai::cyberpunk` (6 themes) |
| `material` | `dodai::material` (2 themes) |
| `icons` | each enabled design system's icon pack |
| `shaders` | `dodai::shaders` |

```rust
let mut sw = dodai::switcher();           // ThemeSwitcher over all enabled themes
let t = dodai::themes::by_id("material-dark");
```
