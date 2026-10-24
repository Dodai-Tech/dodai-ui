//! `ShaderView`: shows a `dodai-shaders` chain as a live, fluid-width image.

use crate::theme_of;
use dodai_shaders::{by_id, Family, ShaderInfo, ShaderPalette};
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
    /// A fixed palette.
    Fixed(ShaderPalette),
}

#[derive(Default)]
struct Slot {
    gpu_last: Option<(Instant, u64, (u32, u32))>,
    start: Option<Instant>,
    image: Option<Arc<RenderImage>>,
    /// When the picture last changed (paces the redraws).
    fresh_at: Option<Instant>,
    /// A wake-up timer is pending.
    waking: Arc<std::sync::atomic::AtomicBool>,
}

impl Slot {
    fn wake_state(&self) -> (Option<Instant>, Arc<std::sync::atomic::AtomicBool>) {
        (self.fresh_at, self.waking.clone())
    }
}

/// Re-render the current view after `secs`, once at a time per `flag`.
fn wake_in(w: &mut Window, cx: &mut App, secs: f32, flag: Arc<std::sync::atomic::AtomicBool>) {
    use std::sync::atomic::Ordering;
    if flag.swap(true, Ordering::AcqRel) {
        return;
    }
    let view = w.current_view();
    let exec = cx.background_executor().clone();
    w.spawn(cx, async move |acx| {
        exec.timer(std::time::Duration::from_secs_f32(secs)).await;
        flag.store(false, Ordering::Release);
        let _ = acx.update(|_, cx| cx.notify(view));
    })
    .detach();
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
    /// Where the shader palette comes from.
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
    /// Corner radius in px.
    pub fn rounded(mut self, r: f32) -> Self {
        self.rounded = r;
        self
    }
}

/// Where shader views render.
enum Backend {
    /// The GPU is still opening (100-200 ms, on a background thread so the first frame does not
    /// stall): views keep what they show.
    Starting,
    /// The shared GPU.
    Gpu(&'static std::sync::Mutex<dodai_shaders::gpu::Gpu>),
    /// No real adapter (or `DODAI_GPU=0`): the CPU renderer.
    Cpu,
}

/// GPU first: opened once, off the UI thread. The CPU renders only when there is no adapter.
fn backend() -> Backend {
    static GPU: std::sync::OnceLock<Option<std::sync::Mutex<dodai_shaders::gpu::Gpu>>> =
        std::sync::OnceLock::new();
    static START: std::sync::Once = std::sync::Once::new();
    START.call_once(|| {
        let _ = std::thread::Builder::new()
            .name("dodai-gpu-init".into())
            .spawn(|| {
                let g = dodai_shaders::gpu::Gpu::new();
                if let Some(g) = &g {
                    // Pipelines compile on demand (see `gpu_frame`); compiling
                    // the whole catalogue up front would hog the CPU for
                    // seconds for shaders nobody looks at.
                    g.prewarm(["pack".to_string()]);
                }
                let _ = GPU.set(g.map(std::sync::Mutex::new));
            });
    });
    match GPU.get() {
        None => Backend::Starting,
        Some(Some(g)) => Backend::Gpu(g),
        Some(None) => Backend::Cpu,
    }
}

/// The CPU renderer, shared by all views.
fn cpu_renderer() -> &'static std::sync::Mutex<dodai_shaders::Renderer> {
    static R: std::sync::OnceLock<std::sync::Mutex<dodai_shaders::Renderer>> =
        std::sync::OnceLock::new();
    R.get_or_init(|| std::sync::Mutex::new(dodai_shaders::Renderer::from_gpu(None)))
}

/// Renders on the GPU, or on the CPU without one: `None` while it is not ready, `Some(None)` to keep the
/// current picture, `Some(Some(img))` for a new one.
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
    let backend = backend();
    if matches!(backend, Backend::Starting) {
        return None;
    }
    // `pixel` is device pixels per shader pixel on a 2x display, so the default
    // 2 is one image pixel per CSS pixel: a quarter of the work of full retina
    // resolution, and shaders are smooth enough that it does not show.
    let dev_px = (pixel * scale / 2.0).max(1.0);
    let (mut w, mut h) = ((bw * scale / dev_px) as u32, (bh * scale / dev_px) as u32);
    // Keep very large views to about 1080p; the image is scaled up.
    let big = (w as f32 / 1920.0).max(h as f32 / 1080.0).max(1.0);
    w = ((w as f32 / big) as u32).max(16);
    h = ((h as f32 / big) as u32).max(9);
    let ids: Vec<&str> = chain.iter().map(|i| i.id).collect();
    if let Backend::Gpu(g) = &backend {
        // Never stall a frame compiling: show the old picture until it is ready.
        let gpu = g.lock().ok()?;
        if !gpu.is_ready(&ids) {
            gpu.warm_chain(&ids);
            return None;
        }
    }
    let due = match s.gpu_last {
        None => true,
        Some((_, k, sz)) if k != cfg || sz != (w, h) || s.image.is_none() => true,
        Some((at, _, _)) => animated && (now - at).as_secs_f32() >= 1.0 / fps,
    };
    // One GPU render at a time per frame, spaced by what the last one cost:
    // each blocks while the picture is read back, so several in one frame
    // would drop it. Views that miss out keep their picture a frame longer.
    if !due || !pace_allows(now, s.image.is_none()) {
        return Some(None);
    }
    let t = if animated { t } else { 1.5 };
    let took = Instant::now();
    let bytes = match &backend {
        Backend::Gpu(g) => {
            let mut gpu = g.lock().ok()?;
            match gpu.render(&ids, w, h, t, intensity, pal) {
                Ok(b) => b.to_vec(),
                // A failed GPU render still shows a picture, from the CPU.
                Err(_) => dodai_shaders::cpu::render(&ids, w, h, t, intensity, pal).ok()?,
            }
        }
        _ => cpu_renderer()
            .lock()
            .ok()?
            .render(&ids, w, h, t, intensity, pal)
            .ok()?
            .to_vec(),
    };
    pace_record(took.elapsed().as_secs_f32());
    let buf = image::RgbaImage::from_raw(w, h, bytes)?;
    s.gpu_last = Some((now, cfg, (w, h)));
    Some(Some(Arc::new(RenderImage::new(vec![image::Frame::new(
        buf,
    )]))))
}

/// Earliest time the next GPU render may start, and the cost of the last one.
fn pace() -> &'static std::sync::Mutex<(Option<Instant>, f32)> {
    static P: std::sync::Mutex<(Option<Instant>, f32)> = std::sync::Mutex::new((None, 0.0));
    &P
}

/// May a GPU render start now? A view with no picture yet always may.
fn pace_allows(now: Instant, first: bool) -> bool {
    first
        || pace()
            .lock()
            .map_or(true, |p| p.0.is_none_or(|at| now >= at))
}

/// Record a render that took `secs`: the next one waits as long again, which
/// keeps the GPU's share of each frame at about half.
fn pace_record(secs: f32) {
    if let Ok(mut p) = pace().lock() {
        *p = (
            Some(Instant::now() + std::time::Duration::from_secs_f32(secs)),
            secs,
        );
    }
}

/// What `ShaderView` renders on: `"GPU · <adapter>"`, `"CPU (no GPU)"`, or a note while the GPU
/// starts.
pub fn shader_backend() -> String {
    match backend() {
        Backend::Gpu(g) => g
            .lock()
            .map_or_else(|_| "GPU".into(), |g| format!("GPU · {}", g.adapter)),
        Backend::Cpu => "CPU (no GPU)".into(),
        Backend::Starting => "starting GPU…".into(),
    }
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
                let shown = slot.update(cx, |s, _| {
                    let start = *s.start.get_or_insert(now);
                    let secs = (now - start).as_secs_f32();
                    // Off-screen views keep their last picture and cost nothing.
                    if !visible && s.image.is_some() {
                        return s.image.clone().map(|i| (i, None));
                    }
                    if !chain.is_empty() {
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
                                None => s.image.clone().map(|i| (i, None)),
                                Some(img) => {
                                    s.fresh_at = Some(now);
                                    let old = s.image.replace(img.clone());
                                    Some((img, old))
                                }
                            };
                        }
                    }
                    s.image.clone().map(|i| (i, None))
                });
                if let Some((image, old)) = shown {
                    let _ = w.paint_image(b, Corners::all(px(rounded)), image, 0, false);
                    // Free the replaced GPU texture so animation does not leak.
                    if let Some(old) = old {
                        let _ = w.drop_image(old);
                    }
                }
                if animated && visible {
                    // Redraw when the next picture is due, not every frame: a
                    // 30 fps shader needs 30 redraws a second, not 120.
                    let (at, waking) = slot.read(cx).wake_state();
                    let wait = at.map_or(0.0, |at| (1.0 / fps) - at.elapsed().as_secs_f32());
                    if wait <= 0.006 {
                        crate::motion::next_frame(w);
                    } else {
                        wake_in(w, cx, wait, waking);
                    }
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
