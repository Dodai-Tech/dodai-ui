# dodai-cyberpunk

The **Cyberpunk** design system for [Dodai UI](https://github.com/sorinirimies/dodai-ui): six chamfered themes (yellow / cyan / red, each dark and light), hard edges, tracked uppercase labels, glow and optional scanlines.

```rust
use dodai_cyberpunk as cp;
let all = cp::variants();                 // 6 themes
let t = cp::night_city();                 // or netrunner(), samurai(), *_light()
// feature `icons` (default): the 136-icon chamfered pack
let n = cp::icons::ALL.len();
```

Depends only on `dodai-core` (and optionally `dodai-cyberpunk-icons`). Combine with other design systems through the `dodai` umbrella crate.
