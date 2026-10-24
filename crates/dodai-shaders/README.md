# dodai-shaders

Pixel shaders for the Dodai design system and beyond, written in plain Rust and run on the GPU.
A shader is a function `fn(uv, &Ctx) -> Vec3`. The sources are compiled once by
[rust-gpu](https://github.com/Rust-GPU/rust-gpu) to WGSL and embedded, so using the crate needs no
shader language and no special toolchain. It runs on every wgpu backend (Metal, Vulkan, DX12 and,
through WebGPU, the browser). There is no CPU renderer.

| Family | Shaders |
|---|---|
| Cyberpunk | neon grid, rain city, hex pulse, circuit, HUD radar, data corruption, synthwave sunset, neon rain glass, neon signs · bloom, glitch, chromatic, neon edges, hologram |
| Material 3 | dynamic colour, ripple, tonal waves, tonal elevation, shape morph, wavy progress, mesh gradient, dot wave, shape loader, tonal bars · tonal scrim, frosted blur |
| Matrix | digital rain, code hallway, glyph wall, rain in depth · Matrix vision, green screen |
| Retro terminals | terminal screen, test pattern, TV snow, oscilloscope, teletext · CRT, phosphor, VHS, TFT, static, 1-bit dither, negative, CRT power-on, vertical roll, halftone, ASCII art, CGA, vignette, film grain, sepia, amber monitor, green phosphor, paper-white, Trinitron, shadow mask, dot matrix, vector display, C64, Apple ][, ZX Spectrum, EGA, composite NTSC, tracking error, degauss · radar, DOS text, C64 READY., ZX loading, colour bars |
| Pixel art | platformer world, metroidvania cave, castle hall, night mountains, item sprites, pine forest, sunset sea, block sky, invaders · pixelate, Game Boy, 16-colour, posterize |
| Ghostty | retro terminal, better CRT, Lottes CRT, in-game CRT, glow + RGB split, drunkard, glitchy, underwater, cubes, sparks, smoke, lava, gradients · gears, inside the Matrix, noise flow |
| Cosmic & nature | starfield, galaxy, fireworks, plasma, caustics, ocean, aurora, snow, lava lamp, fire, interference, gradient, spotlight, smoke, voronoi, clouds, tunnel, metaballs, Julia set, lightning, rain on water, sine waves · water warp, heat haze, kaleidoscope |

```rust
use dodai_shaders::{gpu::Gpu, ShaderPalette};

let mut gpu = Gpu::new().expect("no GPU");
// a generator first, then post effects; returns packed BGRA8
let bgra = gpu.render(&["terminal_scene", "crt"], 1280, 720, 1.0, 1.0, &ShaderPalette::amber())?;
```

`catalog()`, `by_id()` and `of_family()` list the shaders (id, name, family, kind) without a GPU.

Presets (`dodai_shaders::presets`) are ready chains: Amber VT220, Green mainframe, Pip-Boy,
Nostromo, DOS, Commodore 64, ZX Spectrum loading, Apple ][, broadcast test, radar room, haunted
VHS, Ghostty classic and more.

Palettes: `ShaderPalette::{cyberpunk, material3, matrix, amber, platformer, metroidvania}` or
`ShaderPalette::from_theme(&theme)`.

`just dump-shaders` renders every shader to PPM files; the gpui showcase has a live Shaders page.

## How it works

The shaders are plain Rust in `src/shaders`, one implementation for both paths:

* **GPU (first):** `dodai-shaders-kernels` (unpublished) includes the same files by path and
  `just build-kernels` compiles them with rust-gpu to `kernels.wgsl`, which is checked in and
  embedded here. `Gpu` owns one compute pipeline per shader (compiled on first use, or in the
  background with `warm_chain`) and runs a chain into a packed BGRA image.
* **CPU (fallback):** `cpu::render` calls the same functions per pixel on all cores. Output matches
  the GPU to within a few levels of 255 (`tests/cpu_matches_gpu.rs` checks every shader).
* `Renderer` picks: GPU when `dodai-gpu` finds a real adapter, else the CPU (large pictures are
  rendered at most `CPU_MAX_PIXELS` big and scaled up). There is no way to choose a backend.

### Changing a shader

Edit the file in `crates/dodai-shaders/src/shaders`, then `just build-kernels` (needs
`cargo install cargo-gpu`; the first run installs a pinned nightly, ~5 min). Keep shaders
GPU-friendly: no iterators over arrays, no `&[..]` of constants, no strings, no `u8`/`u64`, no
`contains(&x)`; use index loops and `match`.

## Using it from any toolkit

`Gpu::render` returns BGRA8 bytes, so an adapter only uploads them (wgpu toolkits can skip the
read-back and bind the storage buffer instead):

| Toolkit | Upload |
|---|---|
| gpui | `dodai_gpui::ShaderView` (done) |
| egui · iced · bevy | texture from the pixels (swap B and R, or use a BGRA format) |
| slint | `SharedPixelBuffer::<Rgba8Pixel>` |
| web | wgpu on WebGPU (the WGSL runs as is) |

## Credits

The Ghostty family re-implements the looks in
[0xhckr/ghostty-shaders](https://github.com/0xhckr/ghostty-shaders) in Rust (no GLSL copied).
Ideas from the Shadertoy retro-terminal CRT, Timothy Lottes' CRTS (public domain), qwerasd205,
moni-dz, kalgynirae, Jan Mróz, FabriceNeyret2 and otaviogood, rymdlego, unkn0wncode,
Alex Sherwin and pkazmier. Oklab maths by Björn Ottosson.

