//! The box-blur passes on the GPU (`wgpu` compute), identical to the CPU maths.

use wgpu::util::DeviceExt;

/// An open GPU device with the blur pipeline compiled. Create once, reuse for every blur.
pub struct GpuBlur {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipe: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
}

impl GpuBlur {
    /// Opens the best adapter; `None` without a real GPU (software adapters do not count).
    pub fn new() -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        }))
        .ok()?;
        // A software rasteriser (llvmpipe, WARP, ...) is slower than the CPU path.
        if matches!(adapter.get_info().device_type, wgpu::DeviceType::Cpu) {
            return None;
        }
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("dodai-ui-fx"),
            source: wgpu::ShaderSource::Wgsl(include_str!("blur.wgsl").into()),
        });
        let entry = |binding, ty| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty,
            count: None,
        };
        let storage = |read_only| wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        };
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                entry(0, storage(true)),
                entry(1, storage(false)),
                entry(
                    2,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                ),
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let pipe = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&layout),
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Some(Self {
            device,
            queue,
            pipe,
            bgl,
        })
    }

    /// Three horizontal + vertical box passes of radius `r` over RGBA8 `src`; `None` if the
    /// device fails (the caller then blurs on the CPU).
    pub fn blur(&mut self, src: &[u8], w: usize, h: usize, r: usize) -> Option<Vec<u8>> {
        let size = (w * h * 4) as u64;
        let packed: Vec<u32> = src
            .chunks_exact(4)
            .map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]]))
            .collect();
        let usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC;
        let a = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &packed
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect::<Vec<u8>>(),
                usage: usage | wgpu::BufferUsages::COPY_DST,
            });
        let b = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage,
            mapped_at_creation: false,
        });
        let read = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let params = |horizontal: u32| {
            let words = [w as u32, h as u32, r as u32, horizontal];
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &words
                        .iter()
                        .flat_map(|v| v.to_le_bytes())
                        .collect::<Vec<u8>>(),
                    usage: wgpu::BufferUsages::UNIFORM,
                })
        };
        let (ph, pv) = (params(1), params(0));
        let group = |from: &wgpu::Buffer, to: &wgpu::Buffer, p: &wgpu::Buffer| {
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: from.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: to.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: p.as_entire_binding(),
                    },
                ],
            })
        };
        let (gh, gv) = (group(&a, &b, &ph), group(&b, &a, &pv));

        let mut enc = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.pipe);
            for _ in 0..3 {
                pass.set_bind_group(0, &gh, &[]);
                pass.dispatch_workgroups((h as u32).div_ceil(64), 1, 1);
                pass.set_bind_group(0, &gv, &[]);
                pass.dispatch_workgroups((w as u32).div_ceil(64), 1, 1);
            }
        }
        enc.copy_buffer_to_buffer(&a, 0, &read, 0, size);
        self.queue.submit([enc.finish()]);

        let (tx, rx) = std::sync::mpsc::channel();
        read.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        rx.recv().ok()?.ok()?;
        let out = read.slice(..).get_mapped_range().ok()?.to_vec();
        read.unmap();
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Skipped without an adapter (CI runners usually have none).
    #[test]
    fn gpu_blur_is_bit_identical_to_the_cpu() {
        let Some(mut gpu) = GpuBlur::new() else {
            return;
        };
        let (w, h) = (37, 23);
        let src: Vec<u8> = (0..w * h * 4).map(|i| (i * 7 % 251) as u8).collect();
        for sigma in [1.0f32, 3.0, 8.0] {
            let want = crate::blur(&src, w, h, sigma);
            let got = gpu
                .blur(&src, w, h, crate::box_radius(sigma))
                .expect("gpu blur");
            assert_eq!(got, want, "sigma {sigma}");
        }
    }
}
