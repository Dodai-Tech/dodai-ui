//! Run the catalogue on the GPU.
//!
//! The shaders are Rust sources compiled to
//! WGSL by rust-gpu (`dodai-ui-shaders-kernels`, regenerated with
//! `just build-kernels`). This module is the wgpu host: it owns a compute
//! pipeline per shader and runs a chain (one generator, then post effects)
//! into a packed BGRA image.
//!
//! ```no_run
//! use dodai_ui_shaders::{gpu::Gpu, ShaderPalette};
//!
//! let mut gpu = Gpu::new().expect("no GPU");
//! let bgra = gpu
//!     .render(&["neon_grid", "crt"], 1280, 720, 1.5, 1.0, &ShaderPalette::cyberpunk())
//!     .unwrap();
//! assert_eq!(bgra.len(), 1280 * 720 * 4);
//! ```

use crate::ShaderPalette;
use crate::{check_chain, RenderError};
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

const WGSL: &str = include_str!("../kernels.wgsl");

/// Mirrors `dodai_ui_shaders_kernels::Params` byte for byte.
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
    out: wgpu::Buffer,
    read: wgpu::Buffer,
    bytes: Vec<u8>,
    /// Bind groups for the ping-pong pair, built once per size:
    /// `[0]` reads pong and writes ping, `[1]` the reverse.
    groups: [wgpu::BindGroup; 2],
}

type Pipes = Arc<Mutex<HashMap<String, wgpu::ComputePipeline>>>;

/// Everything needed to compile pipelines, detached from the `Gpu` so a
/// background thread can warm them while frames keep rendering.
#[derive(Clone)]
struct Compiler {
    device: wgpu::Device,
    module: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
    entries: Arc<HashMap<String, String>>,
    pipes: Pipes,
}

impl Compiler {
    /// The pipeline for `entry`, compiled on first use. The cache lock is
    /// never held while compiling, so lookups of ready pipelines never wait.
    fn get(&self, entry: &str) -> wgpu::ComputePipeline {
        if let Some(p) = self
            .pipes
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(entry)
        {
            return p.clone();
        }
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
        self.pipes
            .lock()
            .expect("pipes")
            .entry(entry.to_string())
            .or_insert(p)
            .clone()
    }
}

/// A wgpu device with the shader module and its pipelines.
pub struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    module: wgpu::ShaderModule,
    bgl: wgpu::BindGroupLayout,
    layout: wgpu::PipelineLayout,
    pipes: Pipes,
    /// Ids being compiled by a background thread.
    pending: Arc<Mutex<std::collections::HashSet<String>>>,
    entries: Arc<HashMap<String, String>>,
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
    /// Opens the best adapter; `None` without a real GPU (software adapters do not count).
    pub fn new() -> Option<Self> {
        // `DODAI_GPU=0` pretends there is no adapter (for testing the CPU fallback).
        if std::env::var("DODAI_GPU").is_ok_and(|v| v == "0") {
            return None;
        }
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .ok()?;
        let info = adapter.get_info();
        // A software rasteriser (llvmpipe, WARP, ...) is slower than the CPU path.
        if matches!(info.device_type, wgpu::DeviceType::Cpu) {
            return None;
        }
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
        let adapter = info.name;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("dodai-ui-shaders"),
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
            adapter,
            device,
            queue,
            module,
            bgl,
            layout,
            pipes: Arc::default(),
            pending: Arc::default(),
            entries: Arc::new(entry_names()),
            targets: None,
        })
    }

    fn compiler(&self) -> Compiler {
        Compiler {
            device: self.device.clone(),
            module: self.module.clone(),
            layout: self.layout.clone(),
            entries: self.entries.clone(),
            pipes: self.pipes.clone(),
        }
    }

    /// Compiles the pipelines for `ids` on a background thread, so the first
    /// frame that uses them does not stall (a pipeline takes 40-180 ms).
    /// Rendering carries on meanwhile; ids are compiled in the order given.
    pub fn prewarm<I: IntoIterator<Item = String>>(&self, ids: I) {
        let c = self.compiler();
        let ids: Vec<String> = ids.into_iter().collect();
        let _ = std::thread::Builder::new()
            .name("dodai-gpu-warm".into())
            .spawn(move || {
                c.get("pack");
                for id in ids {
                    c.get(&id);
                }
            });
    }

    /// Whether every pipeline `chain` needs is already compiled, so rendering
    /// it will not stall on shader compilation.
    pub fn is_ready(&self, chain: &[&str]) -> bool {
        let pipes = self.pipes.lock().unwrap_or_else(PoisonError::into_inner);
        pipes.contains_key("pack") && chain.iter().all(|id| pipes.contains_key(*id))
    }

    /// Compile what `chain` needs on a background thread (at most one thread
    /// at a time per missing id). Use with [`Gpu::is_ready`]: keep showing
    /// the old picture until it is.
    pub fn warm_chain(&self, chain: &[&str]) {
        let missing: Vec<String> = {
            let pipes = self.pipes.lock().unwrap_or_else(PoisonError::into_inner);
            chain
                .iter()
                .filter(|id| !pipes.contains_key(**id))
                .map(|id| (*id).to_string())
                .collect()
        };
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        let todo: Vec<String> = missing
            .into_iter()
            .filter(|id| pending.insert(id.clone()))
            .collect();
        drop(pending);
        if todo.is_empty() {
            return;
        }
        let c = self.compiler();
        let pending = self.pending.clone();
        let _ = std::thread::Builder::new()
            .name("dodai-gpu-warm".into())
            .spawn(move || {
                for id in todo {
                    c.get(&id);
                    pending
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .remove(&id);
                }
            });
    }

    /// [`Gpu::prewarm`] for the whole catalogue.
    pub fn prewarm_all(&self) {
        self.prewarm(crate::catalog().iter().map(|s| s.id.to_string()));
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
        let params = buf(
            std::mem::size_of::<Params>() as u64,
            st | wgpu::BufferUsages::COPY_DST,
        );
        let ping = buf(px * 16, st);
        let pong = buf(px * 16, st);
        let out = buf(px * 4, st | wgpu::BufferUsages::COPY_SRC);
        let bind = |src: &wgpu::Buffer, dst: &wgpu::Buffer| {
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params.as_entire_binding(),
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
                        resource: out.as_entire_binding(),
                    },
                ],
            })
        };
        let groups = [bind(&pong, &ping), bind(&ping, &pong)];
        self.targets = Some(Targets {
            w,
            h,
            params,
            out,
            read: buf(
                px * 4,
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
            bytes: vec![0; (px * 4) as usize],
            groups,
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
    ) -> Result<&[u8], RenderError> {
        self.run(chain, w, h, time, intensity, pal)?;
        Ok(self.pixels())
    }

    /// The picture of the last successful [`render`](Self::render) / `run`.
    pub(crate) fn pixels(&self) -> &[u8] {
        self.targets.as_ref().map_or(&[], |t| &t.bytes)
    }

    /// [`render`](Self::render) without borrowing the result.
    pub(crate) fn run(
        &mut self,
        chain: &[&str],
        w: u32,
        h: u32,
        time: f32,
        intensity: f32,
        pal: &ShaderPalette,
    ) -> Result<(), RenderError> {
        let (w, h) = (w.max(1), h.max(1));
        check_chain(chain)?;
        self.ensure_targets(w, h);
        let comp = self.compiler();
        let pipes: Vec<wgpu::ComputePipeline> = chain.iter().map(|id| comp.get(id)).collect();
        let pack = comp.get("pack");

        let col = |v: crate::math::Vec3| [v.x, v.y, v.z, 1.0];
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

        let groups = (w.div_ceil(8), h.div_ceil(8));
        let mut enc = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_compute_pass(&Default::default());
            // Ping-pong: the generator writes `ping`; each effect reads the
            // last result and writes the other buffer.
            for (i, pipe) in pipes.iter().enumerate() {
                pass.set_pipeline(pipe);
                pass.set_bind_group(0, &t.groups[i % 2], &[]);
                pass.dispatch_workgroups(groups.0, groups.1, 1);
            }
            // The last result is in the buffer the next group reads.
            pass.set_pipeline(&pack);
            pass.set_bind_group(0, &t.groups[pipes.len() % 2], &[]);
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
            .map_err(|e| RenderError::Readback(e.to_string()))?;
        rx.recv()
            .map_err(|e| RenderError::Readback(e.to_string()))?
            .map_err(|e| RenderError::Readback(e.to_string()))?;
        let t = self.targets.as_mut().expect("targets");
        {
            let data = t
                .read
                .slice(..)
                .get_mapped_range()
                .map_err(|e| RenderError::Readback(e.to_string()))?;
            t.bytes.copy_from_slice(&data);
        }
        t.read.unmap();
        Ok(())
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
        for s in crate::catalog() {
            assert!(names.contains_key(s.id), "no GPU kernel for `{}`", s.id);
        }
        assert!(names.contains_key("pack"));
    }

    /// Every shader draws something finite and not a flat colour.
    #[test]
    fn every_shader_renders_a_varied_picture() {
        let Some(mut gpu) = gpu() else { return };
        let pal = ShaderPalette::cyberpunk();
        let (w, h) = (48u32, 27u32);
        let base = crate::by_id("test_pattern").unwrap().id;
        for s in crate::catalog() {
            let chain: Vec<&str> = if s.is_generator() {
                vec![s.id]
            } else {
                vec![base, s.id]
            };
            let out = gpu.render(&chain, w, h, 1.7, 1.0, &pal).unwrap().to_vec();
            assert_eq!(out.len(), (w * h * 4) as usize, "{}", s.id);
            let first = &out[..3];
            let varied = out
                .chunks(4)
                .any(|p| (0..3).any(|i| (p[i] as i32 - first[i] as i32).abs() > 6));
            assert!(varied, "{} is a flat colour", s.id);
        }
    }

    #[test]
    fn animated_generators_change_over_time() {
        let Some(mut gpu) = gpu() else { return };
        let pal = ShaderPalette::cyberpunk();
        for id in [
            "neon_grid",
            "digital_rain",
            "starfield",
            "m3_blobs",
            "platformer",
        ] {
            let a = gpu.render(&[id], 40, 24, 0.0, 1.0, &pal).unwrap().to_vec();
            let b = gpu.render(&[id], 40, 24, 2.3, 1.0, &pal).unwrap().to_vec();
            assert_ne!(a, b, "{id} is static");
        }
    }

    #[test]
    fn rejects_bad_chains() {
        let Some(mut gpu) = gpu() else { return };
        let pal = ShaderPalette::default();
        assert!(matches!(
            gpu.render(&[], 8, 8, 0.0, 1.0, &pal),
            Err(RenderError::BadChain(_))
        ));
        assert!(matches!(
            gpu.render(&["crt"], 8, 8, 0.0, 1.0, &pal),
            Err(RenderError::BadChain(_))
        ));
        assert!(matches!(
            gpu.render(&["nope"], 8, 8, 0.0, 1.0, &pal),
            Err(RenderError::Unknown(_))
        ));
    }
}
