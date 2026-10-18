//! Run [`dodai_shaders`] on the GPU.
//!
//! The shaders are the *same Rust sources* as the CPU catalogue, compiled to
//! WGSL by rust-gpu (`dodai-shaders-kernels`, regenerated with
//! `just build-kernels`). This crate is the wgpu host: it owns a compute
//! pipeline per shader and runs a chain (one generator, then post effects)
//! into a packed BGRA image.
//!
//! ```no_run
//! use dodai_shaders_gpu::Gpu;
//! use dodai_shaders::ShaderPalette;
//!
//! let mut gpu = Gpu::new().expect("no GPU");
//! let bgra = gpu
//!     .render(&["neon_grid", "crt"], 1280, 720, 1.5, 1.0, &ShaderPalette::cyberpunk())
//!     .unwrap();
//! assert_eq!(bgra.len(), 1280 * 720 * 4);
//! ```

use bytemuck::{Pod, Zeroable};
use dodai_shaders::{by_id, ShaderPalette};
use std::collections::HashMap;

const WGSL: &str = include_str!("../kernels.wgsl");

/// Mirrors `dodai_shaders_kernels::Params` byte for byte.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    res: [f32; 2],
    time: f32,
    intensity: f32,
    bg: [f32; 4],
    fg: [f32; 4],
    a: [f32; 4],
    b: [f32; 4],
    c: [f32; 4],
}

struct Targets {
    w: u32,
    h: u32,
    params: wgpu::Buffer,
    ping: wgpu::Buffer,
    pong: wgpu::Buffer,
    out: wgpu::Buffer,
    read: wgpu::Buffer,
    bytes: Vec<u8>,
}

/// Why a render did not happen.
#[derive(Debug)]
pub enum GpuError {
    /// The chain is empty, or does not start with a generator.
    BadChain(String),
    /// Unknown shader id.
    Unknown(String),
    /// Reading the image back failed.
    Readback(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuError::BadChain(s) => write!(f, "bad chain: {s}"),
            GpuError::Unknown(s) => write!(f, "unknown shader `{s}`"),
            GpuError::Readback(s) => write!(f, "readback failed: {s}"),
        }
    }
}
impl std::error::Error for GpuError {}

/// A wgpu device with the shader module and its pipelines.
pub struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    module: wgpu::ShaderModule,
    bgl: wgpu::BindGroupLayout,
    layout: wgpu::PipelineLayout,
    pipes: HashMap<String, wgpu::ComputePipeline>,
    entries: HashMap<String, String>,
    targets: Option<Targets>,
    /// Name of the adapter in use.
    pub adapter: String,
}

/// Shader id → WGSL entry point. naga appends `_` to names that end in a digit
/// (`pico16` becomes `pico16_`), so read the real names from the module.
fn entry_names() -> HashMap<String, String> {
    let mut out = HashMap::new();
    for line in WGSL.lines() {
        if let Some(rest) = line.strip_prefix("fn ") {
            if rest.contains("@builtin(global_invocation_id)") {
                let name = rest.split('(').next().unwrap_or("");
                out.insert(name.trim_end_matches('_').to_string(), name.to_string());
            }
        }
    }
    out
}

fn storage(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

impl Gpu {
    /// Opens the default adapter. `None` without a usable GPU.
    pub fn new() -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .ok()?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("dodai-shaders"),
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                storage(0, true),
                storage(1, true),
                storage(2, false),
                storage(3, false),
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        Some(Self {
            adapter: adapter.get_info().name,
            device,
            queue,
            module,
            bgl,
            layout,
            pipes: HashMap::new(),
            entries: entry_names(),
            targets: None,
        })
    }

    fn pipeline(&mut self, entry: &str) -> &wgpu::ComputePipeline {
        if !self.pipes.contains_key(entry) {
            let real = self.entries.get(entry).map_or(entry, String::as_str);
            let p = self
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(entry),
                    layout: Some(&self.layout),
                    module: &self.module,
                    entry_point: Some(real),
                    compilation_options: Default::default(),
                    cache: None,
                });
            self.pipes.insert(entry.to_string(), p);
        }
        &self.pipes[entry]
    }

    fn ensure_targets(&mut self, w: u32, h: u32) {
        if self.targets.as_ref().is_some_and(|t| t.w == w && t.h == h) {
            return;
        }
        let buf = |size: u64, usage: wgpu::BufferUsages| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let px = (w as u64) * (h as u64);
        let st = wgpu::BufferUsages::STORAGE;
        self.targets = Some(Targets {
            w,
            h,
            params: buf(
                std::mem::size_of::<Params>() as u64,
                st | wgpu::BufferUsages::COPY_DST,
            ),
            ping: buf(px * 16, st),
            pong: buf(px * 16, st),
            out: buf(px * 4, st | wgpu::BufferUsages::COPY_SRC),
            read: buf(
                px * 4,
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
            bytes: vec![0; (px * 4) as usize],
        });
    }

    /// Renders `chain` (a generator id, then post-effect ids) and returns
    /// tightly packed **BGRA8** bytes, `w * h * 4` long.
    pub fn render(
        &mut self,
        chain: &[&str],
        w: u32,
        h: u32,
        time: f32,
        intensity: f32,
        pal: &ShaderPalette,
    ) -> Result<&[u8], GpuError> {
        let (w, h) = (w.max(1), h.max(1));
        for (i, id) in chain.iter().enumerate() {
            let info = by_id(id).ok_or_else(|| GpuError::Unknown((*id).to_string()))?;
            if (i == 0) != info.is_generator() {
                return Err(GpuError::BadChain(format!(
                    "`{id}` is {} at position {i}",
                    if info.is_generator() {
                        "a generator"
                    } else {
                        "a post effect"
                    }
                )));
            }
        }
        if chain.is_empty() {
            return Err(GpuError::BadChain("empty".into()));
        }
        self.ensure_targets(w, h);
        for id in chain {
            self.pipeline(id);
        }
        self.pipeline("pack");

        let col = |v: dodai_shaders::math::Vec3| [v.x, v.y, v.z, 1.0];
        let params = Params {
            res: [w as f32, h as f32],
            time,
            intensity: intensity.clamp(0.0, 1.0),
            bg: col(pal.bg),
            fg: col(pal.fg),
            a: col(pal.a),
            b: col(pal.b),
            c: col(pal.c),
        };
        let t = self.targets.as_ref().expect("targets");
        self.queue
            .write_buffer(&t.params, 0, bytemuck::bytes_of(&params));

        let bind = |src: &wgpu::Buffer, dst: &wgpu::Buffer| {
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: t.params.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: src.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: dst.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: t.out.as_entire_binding(),
                    },
                ],
            })
        };
        let groups = (w.div_ceil(8), h.div_ceil(8));
        let mut enc = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_compute_pass(&Default::default());
            // Ping-pong: the generator writes `ping`; each effect reads the
            // last result and writes the other buffer.
            let (mut cur, mut other) = (&t.ping, &t.pong);
            for id in chain.iter() {
                pass.set_pipeline(&self.pipes[*id]);
                pass.set_bind_group(0, &bind(other, cur), &[]);
                pass.dispatch_workgroups(groups.0, groups.1, 1);
                std::mem::swap(&mut cur, &mut other);
            }
            // `other` now holds the final image.
            pass.set_pipeline(&self.pipes["pack"]);
            pass.set_bind_group(0, &bind(other, cur), &[]);
            pass.dispatch_workgroups(groups.0, groups.1, 1);
        }
        enc.copy_buffer_to_buffer(&t.out, 0, &t.read, 0, (w as u64) * (h as u64) * 4);
        self.queue.submit([enc.finish()]);

        let slice = t.read.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| GpuError::Readback(e.to_string()))?;
        rx.recv()
            .map_err(|e| GpuError::Readback(e.to_string()))?
            .map_err(|e| GpuError::Readback(e.to_string()))?;
        let t = self.targets.as_mut().expect("targets");
        {
            let data = t
                .read
                .slice(..)
                .get_mapped_range()
                .map_err(|e| GpuError::Readback(e.to_string()))?;
            t.bytes.copy_from_slice(&data);
        }
        t.read.unmap();
        Ok(&t.bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Skipped without an adapter (CI runners usually have none).
    fn gpu() -> Option<Gpu> {
        Gpu::new()
    }

    #[test]
    fn every_catalogue_entry_has_a_kernel() {
        let names = entry_names();
        for s in dodai_shaders::catalog() {
            assert!(names.contains_key(s.id), "no GPU kernel for `{}`", s.id);
        }
        assert!(names.contains_key("pack"));
    }

    #[test]
    fn renders_and_matches_cpu() {
        let Some(mut gpu) = gpu() else { return };
        let pal = ShaderPalette::cyberpunk();
        let (w, h) = (64, 36);
        let out = gpu
            .render(&["plasma", "vignette"], w, h, 1.0, 1.0, &pal)
            .unwrap()
            .to_vec();
        assert_eq!(out.len(), (w * h * 4) as usize);
        let chain = [by_id("plasma").unwrap(), by_id("vignette").unwrap()];
        let cpu =
            dodai_shaders::render(&chain, &dodai_shaders::Ctx::new(w, h, 1.0, pal)).to_bgra8();
        let diff: f64 = out
            .iter()
            .zip(&cpu)
            .map(|(a, b)| (*a as f64 - *b as f64).abs())
            .sum::<f64>()
            / out.len() as f64;
        assert!(diff < 3.0, "GPU and CPU differ by {diff}");
    }

    #[test]
    fn rejects_bad_chains() {
        let Some(mut gpu) = gpu() else { return };
        let pal = ShaderPalette::default();
        assert!(matches!(
            gpu.render(&[], 8, 8, 0.0, 1.0, &pal),
            Err(GpuError::BadChain(_))
        ));
        assert!(matches!(
            gpu.render(&["crt"], 8, 8, 0.0, 1.0, &pal),
            Err(GpuError::BadChain(_))
        ));
        assert!(matches!(
            gpu.render(&["nope"], 8, 8, 0.0, 1.0, &pal),
            Err(GpuError::Unknown(_))
        ));
    }
}
