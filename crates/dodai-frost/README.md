# dodai-frost

Frosted glass on plain RGBA8 pixels, with no dependencies: it knows nothing about Dodai themes or any UI toolkit.

* `blur(src, w, h, sigma)`: Gaussian-like blur (three box passes), edges clamped.
* `blurred(src, w, h, sigma) -> Reduced`: the blur on a reduced-size copy (cheap for big sigmas); `Reduced::crop(...)` cuts a full-resolution rectangle out of it by bilinear sampling, so a panel lines up with the sharp picture behind it.
* `glass(backdrop, w, h, &Glass) -> Vec<u8>`: the finished pane: blurred backdrop, tint over it, optional hard edge.

```rust
use dodai_frost::{glass, Glass};

let backdrop = vec![200u8; 64 * 32 * 4];
let pane = glass(&backdrop, 64, 32, &Glass { tint: [20, 20, 30, 140], blur: 12.0, edge: Some(([255, 0, 128, 255], 1.0)) });
assert_eq!(pane.len(), backdrop.len());
```

`dodai-core`'s `Theme::frost()` gives each design's tint, blur and edge; map it onto `Glass`.
