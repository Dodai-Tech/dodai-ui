# dodai-surface

The geometry of Dodai's surface textures and effects, written once and drawn by any toolkit: grain, halftone, dither, hatch, grid, circuit traces, scanlines, radar, glitch, sheen, flicker, bloom, pulse, chase, gloss, tape, strips and click ripples.

A toolkit implements four primitives and calls `paint` each frame:

```rust
use dodai_surface::{paint, Frame, Painter, Rgba, SurfacePaint};

struct MyPainter; // draw into your toolkit's canvas
impl Painter for MyPainter {
    fn quad(&mut self, _x: f32, _y: f32, _w: f32, _h: f32, _c: Rgba) {}
    fn dot(&mut self, _cx: f32, _cy: f32, _d: f32, _c: Rgba) {}
    fn stroke(&mut self, _pts: &[(f32, f32)], _closed: bool, _width: f32, _c: Rgba) {}
    fn fill(&mut self, _pts: &[(f32, f32)], _c: Rgba) {}
}

let theme = dodai_core::Theme::default();
let fx = dodai_core::SurfaceFx::NONE;
if let Some(sp) = SurfacePaint::resolve(&theme, dodai_core::SurfaceClass::Button, &dodai_core::Surface::INHERIT, fx) {
    let frame = Frame { w: 160.0, h: 48.0, shape: &theme.shape, seed: 1, time: 0.0, presses: &[] };
    let wants_another_frame = paint(&mut MyPainter, &frame, &sp);
    # let _ = wants_another_frame;
}
```

Everything is clipped to the shape (cut corners included) and to the inside of its border before it reaches the painter. Nothing looks at a theme id: what to draw comes from `SurfaceFx`, and the colours from `SurfacePaint`. gpui's painter is `dodai_gpui::paint_surface`.
