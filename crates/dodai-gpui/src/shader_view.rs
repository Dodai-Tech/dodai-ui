//! `ShaderView`: shows a `dodai-shaders` chain as a live, fluid-width image.

use crate::theme_of;
use dodai_shaders::{by_id, Family, Player, ShaderInfo, ShaderPalette};
use gpui::{
    canvas, div, px, App, Bounds, Corners, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderImage, RenderOnce, SharedString, Styled, Window,
};
use std::sync::Arc;
use std::time::Instant;

/// Where a view takes its colours from.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum PaletteSource {
    /// The shader family's own palette (neon, phosphor, platformer, ...).
    #[default]
    Family,
    /// The active Dodai theme, so shaders follow the design system.
    Theme,
    Fixed(ShaderPalette),
}

#[derive(Default)]
struct Slot {
    player: Option<Player>,
    gpu_last: Option<(Instant, u64, (u32, u32))>,
    start: Option<Instant>,
    image: Option<Arc<RenderImage>>,
}

/// Live shader. Fluid width, `aspect` high; each image pixel covers `pixel`
/// device pixels, so a bigger `pixel` is chunkier and cheaper.
#[derive(IntoElement)]
pub struct ShaderView {
    id: SharedString,
    chain: Vec<&'static ShaderInfo>,
    palette: PaletteSource,
    aspect: f32,
    pixel: f32,
    intensity: f32,
    speed: f32,
    animated: bool,
    rounded: f32,
    fps: f32,
    gpu: bool,
}

impl ShaderView {
    /// `shader` is a catalogue id such as `"neon_grid"`. Unknown ids render nothing.
    pub fn new(id: impl Into<SharedString>, shader: &str) -> Self {
        let chain = by_id(shader).into_iter().collect();
        Self {
            id: id.into(),
            chain,
            palette: PaletteSource::Family,
            aspect: 16.0 / 9.0,
            pixel: 2.0,
            intensity: 1.0,
            speed: 1.0,
            animated: true,
            rounded: 0.0,
            fps: 30.0,
            gpu: true,
        }
    }
    /// A ready-made chain from [`dodai_shaders::presets`].
    pub fn preset(id: impl Into<SharedString>, p: &dodai_shaders::Preset) -> Self {
        Self {
            chain: p.shaders(),
            palette: PaletteSource::Fixed((p.palette)()),
            ..Self::new(id, "")
        }
    }
    /// Append a post effect by id.
    pub fn fx(mut self, id: &str) -> Self {
        self.chain.extend(by_id(id));
        self
    }
    /// Make `id` post effect run over the family's default source: handy for
    /// previewing effects alone.
    pub fn fx_over_default(id: impl Into<SharedString>, fx: &str) -> Self {
        let info = by_id(fx);
        let src = info.map(|i| i.family.default_source()).unwrap_or("plasma");
        Self::new(id, src).fx(fx)
    }
    pub fn palette(mut self, p: PaletteSource) -> Self {
        self.palette = p;
        self
    }
    /// Width ÷ height (default 16:9).
    pub fn aspect(mut self, a: f32) -> Self {
        self.aspect = a.max(0.1);
        self
    }
    /// Device pixels per shader pixel (default 2).
    pub fn pixel(mut self, p: f32) -> Self {
        self.pixel = p.max(1.0);
        self
    }
    /// Post-effect strength `0..=1`.
    pub fn intensity(mut self, i: f32) -> Self {
        self.intensity = i.clamp(0.0, 1.0);
        self
    }
    /// Time multiplier.
    pub fn speed(mut self, s: f32) -> Self {
        self.speed = s;
        self
    }
    /// `false` renders once and reuses the picture (cheap thumbnails).
    pub fn animated(mut self, a: bool) -> Self {
        self.animated = a;
        self
    }
    /// Max renders per second while animated (default 30).
    pub fn fps(mut self, f: f32) -> Self {
        self.fps = f.max(1.0);
        self
    }
    /// Use the GPU when the `gpu` feature is on and an adapter exists (default
    /// `true`). On the GPU the picture is rendered at device resolution unless
    /// `pixel` is above 2, which keeps chunky pixel-art looks.
    pub fn gpu(mut self, on: bool) -> Self {
        self.gpu = on;
        self
    }
    pub fn rounded(mut self, r: f32) -> Self {
        self.rounded = r;
        self
    }
}

/// The shared GPU, once ready. Opening the device and compiling the shader
/// module takes 100-200 ms, so it happens on a background thread: until it
/// finishes, views render on the CPU instead of stalling the first frame.
#[cfg(feature = "gpu")]
fn gpu() -> Option<&'static std::sync::Mutex<dodai_shaders_gpu::Gpu>> {
    static GPU: std::sync::OnceLock<Option<std::sync::Mutex<dodai_shaders_gpu::Gpu>>> =
        std::sync::OnceLock::new();
    static START: std::sync::Once = std::sync::Once::new();
    START.call_once(|| {
        // `DODAI_GPU=0` forces the CPU renderer.
        if std::env::var("DODAI_GPU").is_ok_and(|v| v == "0") {
            let _ = GPU.set(None);
            return;
        }
        let _ = std::thread::Builder::new()
            .name("dodai-gpu-init".into())
            .spawn(|| {
                let g = dodai_shaders_gpu::Gpu::new();
                if let Some(g) = &g {
                    // Compile every pipeline in the background so switching
                    // shaders never stalls a frame.
                    g.prewarm_all();
                }
                let _ = GPU.set(g.map(std::sync::Mutex::new));
            });
    });
    GPU.get().and_then(Option::as_ref)
}

/// Renders on the GPU when possible: `Some(frame)` if a new picture was made.
#[cfg(feature = "gpu")]
#[allow(clippy::too_many_arguments)]
fn gpu_frame(
    s: &mut Slot,
    chain: &[&'static ShaderInfo],
    pal: &ShaderPalette,
    (bw, bh, scale): (f32, f32, f32),
    (pixel, intensity, t, fps, animated): (f32, f32, f32, f32, bool),
    cfg: u64,
    now: Instant,
) -> Option<Option<Arc<RenderImage>>> {
    let g = gpu()?;
    let dev_px = if pixel > 2.0 { pixel * scale } else { 1.0 };
    let (mut w, mut h) = ((bw * scale / dev_px) as u32, (bh * scale / dev_px) as u32);
    // Keep very large views to about 1080p; the image is scaled up.
    let big = (w as f32 / 1920.0).max(h as f32 / 1080.0).max(1.0);
    w = ((w as f32 / big) as u32).max(16);
    h = ((h as f32 / big) as u32).max(9);
    let due = match s.gpu_last {
        None => true,
        Some((_, k, sz)) if k != cfg || sz != (w, h) || s.image.is_none() => true,
        Some((at, _, _)) => animated && (now - at).as_secs_f32() >= 1.0 / fps,
    };
    if !due {
        return Some(None);
    }
    let ids: Vec<&str> = chain.iter().map(|i| i.id).collect();
    let t = if animated { t } else { 1.5 };
    let mut gpu = g.lock().ok()?;
    let bytes = gpu.render(&ids, w, h, t, intensity, pal).ok()?;
    let buf = image::RgbaImage::from_raw(w, h, bytes.to_vec())?;
    s.gpu_last = Some((now, cfg, (w, h)));
    Some(Some(Arc::new(RenderImage::new(vec![image::Frame::new(
        buf,
    )]))))
}

/// Which renderer `ShaderView` uses: `"GPU · <adapter>"` or `"CPU"`.
pub fn shader_backend() -> String {
    #[cfg(feature = "gpu")]
    if let Some(g) = gpu() {
        if let Ok(g) = g.lock() {
            return format!("GPU · {}", g.adapter);
        }
    }
    "CPU".to_string()
}

fn hash_cfg(parts: &[u64]) -> u64 {
    parts.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, p| {
        (h ^ p).wrapping_mul(0x100_0000_01b3)
    })
}

impl RenderOnce for ShaderView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = theme_of(cx);
        let slot = window.use_keyed_state(self.id.clone(), cx, |_, _| Slot::default());
        let pal = match self.palette {
            PaletteSource::Family => self
                .chain
                .first()
                .map_or(Family::Cyberpunk, |s| s.family)
                .palette(),
            PaletteSource::Theme => ShaderPalette::from_theme(&theme),
            PaletteSource::Fixed(p) => p,
        };
        let chain = self.chain.clone();
        let use_gpu = self.gpu;
        let (aspect, pixel, intensity, speed, animated, rounded, fps) = (
            self.aspect,
            self.pixel,
            self.intensity,
            self.speed,
            self.animated,
            self.rounded,
            self.fps,
        );
        let mut parts: Vec<u64> = [pal.bg, pal.fg, pal.a, pal.b, pal.c]
            .iter()
            .flat_map(|v| [v.x, v.y, v.z])
            .map(|f| f.to_bits() as u64)
            .collect();
        parts.push(intensity.to_bits() as u64);
        for s in &chain {
            parts.extend(s.id.bytes().map(u64::from));
            parts.push(0);
        }
        let cfg = hash_cfg(&parts);
        let canvas_el = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, cx| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let visible = w.content_mask().bounds.intersects(&b);
                let now = Instant::now();
                let (image, old) = slot.update(cx, |s, _| {
                    let start = *s.start.get_or_insert(now);
                    let secs = (now - start).as_secs_f32();
                    // Off-screen views keep their last picture and cost nothing.
                    if !visible && s.image.is_some() {
                        return (s.image.clone().expect("checked"), None);
                    }
                    #[cfg(feature = "gpu")]
                    if use_gpu && !chain.is_empty() {
                        let scale = w.scale_factor();
                        if let Some(res) = gpu_frame(
                            s,
                            &chain,
                            &pal,
                            (bw, bh, scale),
                            (pixel, intensity, secs * speed, fps, animated),
                            cfg,
                            now,
                        ) {
                            return match res {
                                None => (s.image.clone().expect("rendered earlier"), None),
                                Some(img) => {
                                    let old = s.image.replace(img.clone());
                                    (img, old)
                                }
                            };
                        }
                    }
                    let pl = s
                        .player
                        .get_or_insert_with(|| Player::new(chain.clone(), pal));
                    pl.chain.clone_from(&chain);
                    pl.palette = pal;
                    pl.intensity = intensity;
                    pl.speed = speed;
                    pl.pixel = pixel;
                    pl.max_fps = if animated { fps } else { 0.0 };
                    let (frame, fresh) = pl.tick(secs, bw, bh, cfg);
                    if !fresh && s.image.is_some() {
                        return (s.image.clone().expect("checked"), None);
                    }
                    let (fw, fh) = (frame.w, frame.h);
                    let buf = image::RgbaImage::from_raw(fw, fh, frame.to_bgra8()).expect("size");
                    let img = Arc::new(RenderImage::new(vec![image::Frame::new(buf)]));
                    let old = s.image.replace(img.clone());
                    (img, old)
                });
                let _ = w.paint_image(b, Corners::all(px(rounded)), image, 0, false);
                // Free the replaced GPU texture so animation does not leak.
                if let Some(old) = old {
                    let _ = w.drop_image(old);
                }
                if animated && visible {
                    w.request_animation_frame();
                }
            },
        )
        .absolute()
        .size_full();
        let mut root = div().id(self.id).relative().w_full().child(canvas_el);
        root.style().aspect_ratio = Some(aspect);
        root
    }
}
