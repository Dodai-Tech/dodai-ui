# dodai-gpu

Internal helper (`publish = false`): opens a wgpu device for the Dodai effect crates (`dodai-shaders`). `dodai-frost` stays standalone and opens its own device. Not a public API.

The rule behind it: **GPU first, CPU as the fallback**. [`open`] returns `None` when there is no
real adapter (a software rasteriser such as llvmpipe or WARP counts as none, it is slower than
the CPU paths), and the caller then runs its CPU implementation. There is deliberately no way to
pick a backend. `DODAI_GPU=0` pretends there is no adapter, for testing.
