# dodai-fx

Self-contained effects with no dependencies: surface textures and effects drawn through a tiny `Painter` trait.

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

Frosted glass (blur on RGBA pixels, CPU or GPU) lives in its own crate, `dodai-frost`.
