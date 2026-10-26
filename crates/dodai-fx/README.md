# dodai-fx

Self-contained effects with no dependencies: surface textures and effects (`dodai_fx::*`) and frosted glass (`dodai_fx::frost`).

## Surface effects

The geometry of Dodai's surface textures and effects, written once and drawn by any toolkit: grain, halftone, dither, hatch, grid, circuit traces, scanlines, radar, glitch, sheen, flicker, bloom, pulse, chase, gloss, tape, strips and click ripples.

A toolkit implements four primitives and calls `paint` each frame:

```rust,no_run
use dodai_fx::{paint, Clip, Frame, Painter, Rgba, SurfaceFx, SurfacePaint, SurfaceTuning, Texture};

struct MyPainter; // draw into your toolkit's canvas
impl Painter for MyPainter {
    fn quad(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _c: Rgba) {}
    fn dot(&mut self, _cx: f32, _cy: f32, _d: f32, _c: Rgba) {}
    fn stroke(&mut self, _pts: &[(f32, f32)], _closed: bool, _width: f32, _c: Rgba) {}
    fn fill(&mut self, _pts: &[(f32, f32)], _c: Rgba) {}
}

let ink = Rgba { r: 0.9, g: 0.9, b: 0.9, a: 1.0 };
let sp = SurfacePaint {
    fx: SurfaceFx { texture: Texture::Grid, strength: 0.6, ..SurfaceFx::NONE },
    ink,
    hole: Rgba { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
    a: ink, b: ink, light: ink, neon: ink, shade: ink, wash: [ink; 3],
    tune: SurfaceTuning::DEFAULT,
    pressable: false,
    fill_lightness: 0.1,
};
let clip = Clip { radius: 8.0, border_width: 1.0, ..Clip::default() };
let frame = Frame { w: 160.0, h: 48.0, shape: &clip, seed: 1, time: 0.0, presses: &[] };
let wants_another_frame = paint(&mut MyPainter, &frame, &sp);
```

Everything is clipped to a `Clip` (corner radius, cut corners, border inset) before it reaches the painter. The crate has no dependencies: what to draw comes from `SurfaceFx`, the colours from `SurfacePaint`. With `dodai-core`, `surface_paint(&theme, ..)` fills a `SurfacePaint` from a theme and `Shape::clip()` gives the `Clip`. gpui's painter is `dodai_gpui::paint_surface`.

## Frosted glass (`dodai_fx::frost`)

Frosted glass on plain RGBA8 pixels, with no dependencies: it knows nothing about Dodai themes or any UI toolkit.

* `blur(src, w, h, sigma)`: Gaussian-like blur (three box passes), edges clamped.
* `blurred(src, w, h, sigma) -> Reduced`: the blur on a reduced-size copy (cheap for big sigmas); `Reduced::crop(...)` cuts a full-resolution rectangle out of it by bilinear sampling, so a panel lines up with the sharp picture behind it.
* `glass(backdrop, w, h, &Glass) -> Vec<u8>`: the finished pane: blurred backdrop, tint over it, optional hard edge.

```rust
use dodai_fx::frost::{glass, Glass};

let backdrop = vec![200u8; 64 * 32 * 4];
let pane = glass(&backdrop, 64, 32, &Glass { tint: [20, 20, 30, 140], blur: 12.0, edge: Some(([255, 0, 128, 255], 1.0)) });
assert_eq!(pane.len(), backdrop.len());
```

## CPU or GPU

`Blurrer::new()` (feature `gpu`, wgpu compute) opens the GPU once and always blurs there when a real adapter exists (it keeps the CPU free and the UI smooth, even for small pictures); with no adapter, or only a software one, it stays on the CPU.  There is no backend setting: the free functions (`blur`, `glass`) are the CPU path. Both run the same integer maths, so the output is **byte-identical** (tested). Measured on an Apple GPU, sigma 12, including upload and readback (`cargo run --release -p dodai-fx --features gpu --example frost_bench`):

| picture | CPU | GPU |
|---|---|---|
| 256×256 | 3.2 ms | 2.3 ms |
| 1280×720 | 50 ms | 5.4 ms |
| 1920×1080 | 79 ms | 9.1 ms |
| 3840×2160 | 688 ms | 18 ms |

The GPU path is faster, not different: it is the same box-blur approximation. A toolkit that can blur its own live backdrop in a render pass should do that instead.

`dodai-core`'s `Theme::frost()` gives each design's tint, blur and edge; map it onto `Glass`.
