#![doc = include_str!("../README.md")]

/// An open GPU device.
pub struct Gpu {
    /// Name of the adapter in use.
    pub adapter: String,
    /// The device.
    pub device: wgpu::Device,
    /// Its queue.
    pub queue: wgpu::Queue,
}

/// Opens the best real adapter; `None` without a usable GPU (or with `DODAI_GPU=0`).
pub fn open() -> Option<Gpu> {
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
    if matches!(info.device_type, wgpu::DeviceType::Cpu) {
        return None;
    }
    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
    Some(Gpu {
        adapter: info.name,
        device,
        queue,
    })
}
