# dodai-shaders

Pure-Rust pixel shaders for the Dodai design system and beyond. A shader is a plain
function `fn(uv, &Ctx) -> Vec3`; frames are rendered on the CPU (multi-threaded) and can be
shown as an image by any toolkit. No GPU API, no shader language.

| Family | Shaders |
|---|---|
| Cyberpunk | neon grid, rain city, hex pulse, circuit, HUD radar, data corruption, synthwave sunset, neon rain glass, neon signs · bloom, glitch, chromatic, neon edges, hologram |
| Material 3 | dynamic colour, ripple, tonal waves, tonal elevation, shape morph, wavy progress, mesh gradient, dot wave, shape loader, tonal bars · tonal scrim, frosted blur |
| Matrix | digital rain, code hallway, glyph wall, rain in depth · Matrix vision, green screen |
| Retro terminals | terminal screen, test pattern, TV snow, oscilloscope, teletext · CRT, phosphor, VHS, TFT, static, 1-bit dither, negative, CRT power-on, vertical roll, halftone, ASCII art, CGA, vignette, film grain, sepia |
| Pixel art | platformer world, metroidvania cave, castle hall, night mountains, item sprites, pine forest, sunset sea, block sky, invaders · pixelate, Game Boy, 16-colour, posterize |
| Cosmic & nature | starfield, galaxy, fireworks, plasma, caustics, ocean, aurora, snow, lava lamp, fire, interference, gradient, spotlight, smoke, voronoi, clouds, tunnel, metaballs, Julia set, lightning, rain on water, sine waves · water warp, heat haze, kaleidoscope |

```rust
use dodai_shaders::{by_id, render, Ctx, ShaderPalette};

let chain = [by_id("terminal_scene").unwrap(), by_id("crt").unwrap()];
let frame = render(&chain, &Ctx::new(320, 180, 1.0, ShaderPalette::amber()));
let rgba = frame.to_rgba8();
```

Palettes: `ShaderPalette::{cyberpunk, material3, matrix, amber, platformer, metroidvania}` or
`ShaderPalette::from_theme(&theme)`.

Demo: `cargo run --release -p dodai-shaders --example gallery`
(`dump` writes every shader as PPM files).

## Performance

* Rows are pulled from a shared queue by all cores, so uneven shaders balance.
* `Player` caps the frame rate, renders a static view once, and coarsens the
  picture automatically when a frame exceeds its budget (`budget_ms`, default 8).
* `cargo run --release -p dodai-shaders --example bench` prints ms per 320×180 frame
  (all shaders: ≈ 1.3 ms average on 14 cores).

## Using it from any toolkit

The crate knows nothing about UI. A frame is plain RGB floats and an adapter only does
`Player::tick` and uploads the pixels:

| Toolkit | Upload |
|---|---|
| gpui | `dodai_gpui::ShaderView` (done) |
| egui | `ColorImage::from_rgba_unmultiplied(size, &frame.to_rgba8())` → texture |
| iced | `image::Handle::from_rgba(w, h, frame.to_rgba8())` |
| slint | `SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&rgba, w, h)` |
| bevy | `Image::new(.., TextureFormat::Rgba8UnormSrgb)` data = `to_rgba8()` |
| dioxus · leptos · yew | `ImageData` on a `<canvas>`, or a PNG data URL |
| ratatui | `frame.to_half_blocks(cols, rows)`: `▀` with fg = top, bg = bottom |

```rust
let mut player = dodai_shaders::Player::new(vec![by_id("crt").unwrap()], pal);
let (frame, fresh) = player.tick(seconds, width_px, height_px, 0);
if fresh { upload(frame.to_rgba8(), frame.w, frame.h); }
```

Only gpui has an adapter so far; the others are the snippets above.
