# dodai-shaders-gpu

Run the [`dodai-shaders`](../dodai-shaders) catalogue (137 shaders) on the GPU, with wgpu compute.

The shaders are **the same Rust source files** as the CPU catalogue. They are compiled to WGSL by
[rust-gpu](https://github.com/Rust-GPU/rust-gpu) (`crates/dodai-shaders-kernels`), and the result is
checked in as `kernels.wgsl`, so building this crate needs no special toolchain.

```rust
let mut gpu = dodai_shaders_gpu::Gpu::new().expect("no GPU");
// generator first, then post effects; returns packed BGRA8
let bgra = gpu.render(&["terminal_scene", "crt"], 1920, 1080, t, 1.0, &ShaderPalette::matrix())?;
```

Measured on an M4 Pro at 1080p, all 137 shaders: **0.7 s on the GPU vs 3.4 s on 14 CPU cores**
(`just parity 1920 1080`). It also checks every shader against the CPU result: the mean difference
is under 3/255. Small frames gain little, because each render waits for the read-back.

* `dodai-gpui` uses it automatically (`gpu` feature, on by default) and falls back to the CPU.
  `DODAI_GPU=0` forces the CPU.
* wgpu toolkits (Bevy, egui-wgpu, iced) can skip the read-back and bind the storage buffer directly.

## Changing a shader

Edit the file in `crates/dodai-shaders/src` as usual, then `just build-kernels` (needs
`cargo install cargo-gpu`; the first run installs a pinned nightly and builds the backend, ~5 min).
Shader files must stay GPU-friendly: no iterators over arrays, no `&[..]` of constants, no strings,
no `u8`/`u64` values, no `contains(&x)`; use index loops and `match`. `just build-kernels` reports
what rust-gpu rejects.
