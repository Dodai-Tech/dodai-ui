# dodai-frost

Frosted glass on plain RGBA8 pixels, with no dependencies: it knows nothing about Dodai themes or any UI toolkit.

* `blur(src, w, h, sigma)`: Gaussian-like blur (three box passes), edges clamped.
* `blurred(src, w, h, sigma) -> Reduced`: the blur on a reduced-size copy (cheap for big sigmas); `Reduced::crop(...)` cuts a full-resolution rectangle out of it by bilinear sampling, so a panel lines up with the sharp picture behind it.
* `glass(backdrop, w, h, &FrostParams) -> Vec<u8>`: the finished pane: blurred backdrop, tint over it, optional hard edge.

```rust
use dodai_frost::{glass, FrostParams};

let backdrop = vec![200u8; 64 * 32 * 4];
let pane = glass(&backdrop, 64, 32, &FrostParams { tint: [20, 20, 30, 140], blur: 12.0, edge: Some(([255, 0, 128, 255], 1.0)), saturation: 1.0 });
assert_eq!(pane.len(), backdrop.len());
```

## CPU or GPU

`Blurrer::new()` (feature `gpu`, wgpu compute) opens the GPU once and always blurs there when a real adapter exists (it keeps the CPU free and the UI smooth, even for small pictures); with no adapter, or only a software one, it stays on the CPU.  There is no backend setting: the free functions (`blur`, `glass`) are the CPU path. Both run the same integer maths, so the output is **byte-identical** (tested). Measured on an Apple GPU, sigma 12, including upload and readback (`cargo run --release -p dodai-frost --features gpu --example frost_bench`):

| picture | CPU | GPU |
|---|---|---|
| 256×256 | 3.2 ms | 2.3 ms |
| 1280×720 | 50 ms | 5.4 ms |
| 1920×1080 | 79 ms | 9.1 ms |
| 3840×2160 | 688 ms | 18 ms |

The GPU path is faster, not different: it is the same box-blur approximation. A toolkit that can blur its own live backdrop in a render pass should do that instead.

`dodai-core`'s `Theme::frost()` gives each design's tint, blur and edge; map it onto `FrostParams`.
