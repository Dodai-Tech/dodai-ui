//! [`Renderer`]: GPU first, CPU as the fallback. There is deliberately no way to choose.

use crate::{cpu, gpu::Gpu, RenderError, ShaderPalette};

/// Renders at most this many pixels on the CPU; larger requests are rendered smaller and scaled
/// up, so a machine without a GPU stays responsive.
pub const CPU_MAX_PIXELS: u32 = 640 * 360;

/// Renders shader chains on the GPU when a real adapter exists, else on the CPU.
pub struct Renderer {
    gpu: Option<Gpu>,
    cpu_out: Vec<u8>,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    /// Opens the GPU if there is a real one (see `dodai-gpu`).
    pub fn new() -> Self {
        Self::from_gpu(Gpu::new())
    }

    /// A renderer on an already opened (or absent) GPU.
    pub fn from_gpu(gpu: Option<Gpu>) -> Self {
        Self {
            gpu,
            cpu_out: Vec::new(),
        }
    }

    /// Whether renders run on the GPU.
    pub fn uses_gpu(&self) -> bool {
        self.gpu.is_some()
    }

    /// The GPU, for callers that warm pipelines. `None` when rendering on the CPU.
    pub fn gpu(&mut self) -> Option<&mut Gpu> {
        self.gpu.as_mut()
    }

    /// Renders `chain` (a generator id, then post effects) to packed **BGRA8**, `w * h * 4`
    /// bytes. A failing GPU render falls back to the CPU.
    pub fn render(
        &mut self,
        chain: &[&str],
        w: u32,
        h: u32,
        time: f32,
        intensity: f32,
        pal: &ShaderPalette,
    ) -> Result<&[u8], RenderError> {
        let (w, h) = (w.max(1), h.max(1));
        if let Some(g) = self.gpu.as_mut() {
            // Borrowck cannot see that the Ok arm is the only one returning the borrow.
            if g.render(chain, w, h, time, intensity, pal).is_ok() {
                return self
                    .gpu
                    .as_mut()
                    .expect("gpu")
                    .render(chain, w, h, time, intensity, pal);
            }
        }
        self.cpu_out = render_cpu(chain, w, h, time, intensity, pal)?;
        Ok(&self.cpu_out)
    }
}

/// The CPU path: small enough requests directly, larger ones at reduced size, scaled up.
fn render_cpu(
    chain: &[&str],
    w: u32,
    h: u32,
    time: f32,
    intensity: f32,
    pal: &ShaderPalette,
) -> Result<Vec<u8>, RenderError> {
    if w * h <= CPU_MAX_PIXELS {
        return cpu::render(chain, w, h, time, intensity, pal);
    }
    let k = (CPU_MAX_PIXELS as f32 / (w * h) as f32).sqrt();
    let (sw, sh) = (
        ((w as f32 * k) as u32).max(1),
        ((h as f32 * k) as u32).max(1),
    );
    let small = cpu::render(chain, sw, sh, time, intensity, pal)?;
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let sy = (y * sh / h) as usize;
        for x in 0..w {
            let sx = (x * sw / w) as usize;
            let i = (sy * sw as usize + sx) * 4;
            out.extend_from_slice(&small[i..i + 4]);
        }
    }
    Ok(out)
}
