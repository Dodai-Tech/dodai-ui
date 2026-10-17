# dodai-shaders

Pure-Rust pixel shaders for the Dodai design system and beyond. A shader is a plain
function `fn(uv, &Ctx) -> Vec3`; frames are rendered on the CPU (multi-threaded) and can be
shown as an image by any toolkit. No GPU API, no shader language.

| Family | Shaders |
|---|---|
| Cyberpunk | neon grid, rain city, hex pulse, circuit, HUD radar, data corruption · bloom, glitch, chromatic, neon edges |
| Material 3 | dynamic colour, ripple, tonal waves, tonal elevation, shape morph, wavy progress · tonal scrim |
| Matrix | digital rain, code hallway, glyph wall · Matrix vision |
| Retro terminals | terminal screen, test pattern, TV snow · CRT, phosphor, VHS, TFT, static, 1-bit dither, negative |
| Pixel art | platformer world, metroidvania cave, castle hall, night mountains, item sprites · pixelate, Game Boy, 16-colour |
| Cosmic & nature | starfield, galaxy, fireworks, plasma, caustics, ocean, aurora, snow, lava lamp, fire, interference, gradient, spotlight, smoke · water warp, heat haze, kaleidoscope |

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
