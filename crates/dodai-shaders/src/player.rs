//! Toolkit-agnostic shader playback: opens the GPU once (off the UI thread), falls back to the CPU,
//! sizes the picture, paces the renders and returns packed BGRA pixels. A toolkit only turns
//! [`Frame`] into its own image type and decides when to ask again.

use crate::{ShaderInfo, ShaderPalette};
use std::time::Instant;

/// Where shader views render.
enum Backend {
    /// The GPU is still opening (100-200 ms, on a background thread so the first frame does not
    /// stall): views keep what they show.
    Starting,
    /// The shared GPU.
    Gpu(&'static std::sync::Mutex<crate::gpu::Gpu>),
    /// No real adapter (or `DODAI_GPU=0`): the CPU renderer.
    Cpu,
}

/// GPU first: opened once, off the UI thread. The CPU renders only when there is no adapter.
fn backend() -> Backend {
    static GPU: std::sync::OnceLock<Option<std::sync::Mutex<crate::gpu::Gpu>>> =
        std::sync::OnceLock::new();
    static START: std::sync::Once = std::sync::Once::new();
    START.call_once(|| {
        let _ = std::thread::Builder::new()
            .name("dodai-gpu-init".into())
            .spawn(|| {
                let g = crate::gpu::Gpu::new();
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
fn cpu_renderer() -> &'static std::sync::Mutex<crate::Renderer> {
    static R: std::sync::OnceLock<std::sync::Mutex<crate::Renderer>> = std::sync::OnceLock::new();
    R.get_or_init(|| std::sync::Mutex::new(crate::Renderer::from_gpu(None)))
}

/// What one [`Player::poll`] asks for.
pub struct Request<'a> {
    /// A generator, then post effects.
    pub chain: &'a [&'static ShaderInfo],
    /// Colours.
    pub palette: &'a ShaderPalette,
    /// Box size in logical px.
    pub size: (f32, f32),
    /// Device pixels per logical px.
    pub scale: f32,
    /// Device pixels per shader pixel on a 2x display (2 = one image pixel per logical px).
    pub pixel: f32,
    /// Effect strength.
    pub intensity: f32,
    /// Shader clock in seconds.
    pub time: f32,
    /// Most pictures per second.
    pub fps: f32,
    /// Moves (false = one still frame).
    pub animated: bool,
    /// Light on black: the brightest channel becomes the opacity.
    pub transparent: bool,
    /// Hash of everything but size and time (see [`config_hash`]): a change forces a render.
    pub config: u64,
    /// Now.
    pub now: Instant,
}

/// A rendered picture, packed BGRA8.
pub struct Frame {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height * 4` bytes, blue first.
    pub bgra: Vec<u8>,
}

/// The answer to [`Player::poll`].
pub enum Poll {
    /// Not ready (GPU starting or pipeline compiling): keep showing what there is.
    Pending,
    /// Nothing new is due: keep the current picture.
    Keep,
    /// A new picture.
    New(Frame),
}

/// One shader view's render state. Cheap; the GPU and CPU renderers are shared by all players.
#[derive(Default)]
pub struct Player {
    last: Option<(Instant, u64, (u32, u32))>,
}

impl Player {
    /// A player that has shown nothing yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders on the GPU, or on the CPU without one, when a picture is due.
    pub fn poll(&mut self, r: &Request) -> Poll {
        let backend = backend();
        if matches!(backend, Backend::Starting) {
            return Poll::Pending;
        }
        // `pixel` is device pixels per shader pixel on a 2x display, so the default
        // 2 is one image pixel per logical pixel: a quarter of the work of full retina
        // resolution, and shaders are smooth enough that it does not show.
        let dev_px = (r.pixel * r.scale / 2.0).max(1.0);
        let (mut w, mut h) = (
            (r.size.0 * r.scale / dev_px) as u32,
            (r.size.1 * r.scale / dev_px) as u32,
        );
        // Keep very large views to about 1080p; the toolkit scales the picture up.
        let big = (w as f32 / 1920.0).max(h as f32 / 1080.0).max(1.0);
        w = ((w as f32 / big) as u32).max(16);
        h = ((h as f32 / big) as u32).max(9);
        let ids: Vec<&str> = r.chain.iter().map(|i| i.id).collect();
        if let Backend::Gpu(g) = &backend {
            // Never stall a frame compiling: show the old picture until it is ready.
            let Ok(gpu) = g.lock() else {
                return Poll::Pending;
            };
            if !gpu.is_ready(&ids) {
                gpu.warm_chain(&ids);
                return Poll::Pending;
            }
        }
        let due = match self.last {
            None => true,
            Some((_, k, sz)) if k != r.config || sz != (w, h) => true,
            Some((at, _, _)) => r.animated && (r.now - at).as_secs_f32() >= 1.0 / r.fps,
        };
        // One GPU render at a time per frame, spaced by what the last one cost: each blocks while
        // the picture is read back, so several in one frame would drop it.
        if !due || !pace_allows(r.now, self.last.is_none()) {
            return Poll::Keep;
        }
        let t = if r.animated { r.time } else { 1.5 };
        let took = Instant::now();
        let rendered = match &backend {
            Backend::Gpu(g) => g.lock().ok().and_then(|mut gpu| {
                match gpu.render(&ids, w, h, t, r.intensity, r.palette) {
                    Ok(b) => Some(b.to_vec()),
                    // A failed GPU render still shows a picture, from the CPU.
                    Err(_) => crate::cpu::render(&ids, w, h, t, r.intensity, r.palette).ok(),
                }
            }),
            _ => cpu_renderer().lock().ok().and_then(|mut c| {
                c.render(&ids, w, h, t, r.intensity, r.palette)
                    .ok()
                    .map(|b| b.to_vec())
            }),
        };
        let Some(mut bgra) = rendered else {
            return Poll::Pending;
        };
        pace_record(took.elapsed().as_secs_f32());
        if r.transparent {
            // Light on black: the brightest channel is the opacity, and the colour stays as it is,
            // which is already premultiplied by it.
            for px in bgra.as_chunks_mut::<4>().0 {
                px[3] = px[0].max(px[1]).max(px[2]);
            }
        }
        self.last = Some((r.now, r.config, (w, h)));
        Poll::New(Frame {
            width: w,
            height: h,
            bgra,
        })
    }
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

/// What shader views render on: `"GPU · <adapter>"`, `"CPU (no GPU)"`, or a note while the GPU
/// starts.
pub fn backend_label() -> String {
    match backend() {
        Backend::Gpu(g) => g
            .lock()
            .map_or_else(|_| "GPU".into(), |g| format!("GPU · {}", g.adapter)),
        Backend::Cpu => "CPU (no GPU)".into(),
        Backend::Starting => "starting GPU…".into(),
    }
}

/// Hash of what makes a picture differ besides its size and clock: the palette, the intensity,
/// transparency and the chain. Put it in [`Request::config`].
pub fn config_hash(
    chain: &[&'static ShaderInfo],
    palette: &ShaderPalette,
    intensity: f32,
    transparent: bool,
) -> u64 {
    let mut parts: Vec<u64> = [palette.bg, palette.fg, palette.a, palette.b, palette.c]
        .iter()
        .flat_map(|v| [v.x, v.y, v.z])
        .map(|f| f.to_bits() as u64)
        .collect();
    parts.push(intensity.to_bits() as u64);
    parts.push(u64::from(transparent));
    for s in chain {
        parts.extend(s.id.bytes().map(u64::from));
        parts.push(0);
    }
    parts.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, p| {
        (h ^ p).wrapping_mul(0x100_0000_01b3)
    })
}

/// What a shader view shows and how: the same description for every toolkit.
#[derive(Clone, Debug)]
pub struct Spec {
    /// A generator, then post effects.
    pub chain: Vec<&'static ShaderInfo>,
    /// Colours.
    pub palette: ShaderPalette,
    /// Width ÷ height.
    pub aspect: f32,
    /// Device pixels per shader pixel on a 2x display (bigger is chunkier and cheaper).
    pub pixel: f32,
    /// Post-effect strength `0..=1`.
    pub intensity: f32,
    /// Clock multiplier.
    pub speed: f32,
    /// Most pictures per second.
    pub fps: f32,
    /// Moves (false = one still frame).
    pub animated: bool,
    /// Light on black becomes opacity.
    pub transparent: bool,
}

impl PartialEq for Spec {
    fn eq(&self, o: &Self) -> bool {
        self.chain.len() == o.chain.len()
            && self.chain.iter().zip(&o.chain).all(|(a, b)| a.id == b.id)
            && self.palette == o.palette
            && self.aspect == o.aspect
            && self.pixel == o.pixel
            && self.intensity == o.intensity
            && self.speed == o.speed
            && self.fps == o.fps
            && self.animated == o.animated
            && self.transparent == o.transparent
    }
}

impl Spec {
    /// `shader` is a catalogue id such as `"neon_grid"` (unknown ids render nothing), drawn in
    /// the shader family's own palette.
    pub fn new(shader: &str) -> Self {
        let chain: Vec<_> = crate::by_id(shader).into_iter().collect();
        let palette = chain
            .first()
            .map_or(crate::Family::Cyberpunk, |s| s.family)
            .palette();
        Self {
            chain,
            palette,
            aspect: 16.0 / 9.0,
            pixel: 2.0,
            intensity: 1.0,
            speed: 1.0,
            fps: 30.0,
            animated: true,
            transparent: false,
        }
    }

    /// A ready-made chain from [`crate::presets`], in its own palette.
    pub fn preset(p: &crate::Preset) -> Self {
        Self {
            chain: p.shaders(),
            palette: (p.palette)(),
            ..Self::new("")
        }
    }

    /// Append a post effect by id.
    pub fn fx(mut self, id: &str) -> Self {
        self.chain.extend(crate::by_id(id));
        self
    }

    /// Colours.
    pub fn palette(mut self, p: ShaderPalette) -> Self {
        self.palette = p;
        self
    }

    /// Colours from a Dodai theme, so the shader follows the design system.
    pub fn themed(self, theme: &dodai_core::Theme) -> Self {
        self.palette(ShaderPalette::from_theme(theme))
    }

    /// Width ÷ height.
    pub fn aspect(mut self, a: f32) -> Self {
        self.aspect = a.max(0.1);
        self
    }

    /// Device pixels per shader pixel.
    pub fn pixel(mut self, p: f32) -> Self {
        self.pixel = p.max(1.0);
        self
    }

    /// Post-effect strength.
    pub fn intensity(mut self, i: f32) -> Self {
        self.intensity = i.clamp(0.0, 1.0);
        self
    }

    /// Clock multiplier.
    pub fn speed(mut self, s: f32) -> Self {
        self.speed = s;
        self
    }

    /// Most pictures per second.
    pub fn fps(mut self, f: f32) -> Self {
        self.fps = f.clamp(1.0, 120.0);
        self
    }

    /// Moves, or one still frame.
    pub fn animated(mut self, on: bool) -> Self {
        self.animated = on;
        self
    }

    /// Light on black becomes opacity.
    pub fn transparent(mut self, on: bool) -> Self {
        self.transparent = on;
        self
    }
}

/// A [`Spec`] being played: owns the clock and the [`Player`]. The toolkit views wrap this.
pub struct Live {
    spec: Spec,
    player: Player,
    start: Option<Instant>,
}

impl Live {
    /// Starts the clock at the first [`tick`](Self::tick).
    pub fn new(spec: Spec) -> Self {
        Self {
            spec,
            player: Player::new(),
            start: None,
        }
    }

    /// What is playing.
    pub fn spec(&self) -> &Spec {
        &self.spec
    }

    /// Replace the spec; the next tick draws it (nothing happens if it is unchanged).
    pub fn set_spec(&mut self, spec: Spec) {
        if spec != self.spec {
            self.spec = spec;
        }
    }

    /// Time until the next picture is due: how long a toolkit may sleep before calling
    /// [`tick`](Self::tick) again (`None` for a still shader).
    pub fn next_in(&self) -> Option<std::time::Duration> {
        self.spec
            .animated
            .then(|| std::time::Duration::from_secs_f32(1.0 / self.spec.fps))
    }

    /// Ask for a picture `width` logical px wide (the height follows the aspect) at `scale`
    /// device pixels per logical px.
    pub fn tick(&mut self, width: f32, scale: f32) -> Poll {
        let now = Instant::now();
        let start = *self.start.get_or_insert(now);
        let sp = &self.spec;
        if sp.chain.is_empty() || width < 1.0 {
            return Poll::Keep;
        }
        let config = config_hash(&sp.chain, &sp.palette, sp.intensity, sp.transparent);
        self.player.poll(&Request {
            chain: &sp.chain,
            palette: &sp.palette,
            size: (width, width / sp.aspect),
            scale,
            pixel: sp.pixel,
            intensity: sp.intensity,
            time: (now - start).as_secs_f32() * sp.speed,
            fps: sp.fps,
            animated: sp.animated,
            transparent: sp.transparent,
            config,
            now,
        })
    }
}

impl Frame {
    /// The pixels as RGBA8 (blue and red swapped).
    pub fn rgba(mut self) -> Vec<u8> {
        for px in self.bgra.as_chunks_mut::<4>().0 {
            px.swap(0, 2);
        }
        self.bgra
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::by_id;

    #[test]
    fn config_hash_follows_what_changes_the_picture() {
        let grid = [by_id("neon_grid").expect("shader")];
        let pal = ShaderPalette::cyberpunk();
        let base = config_hash(&grid, &pal, 1.0, false);
        assert_eq!(base, config_hash(&grid, &pal, 1.0, false));
        assert_ne!(base, config_hash(&grid, &pal, 0.5, false));
        assert_ne!(base, config_hash(&grid, &pal, 1.0, true));
        assert_ne!(
            base,
            config_hash(
                &grid,
                &ShaderPalette::from_theme(&dodai_core::themes::neutral()),
                1.0,
                false
            )
        );
        assert_ne!(base, config_hash(&[], &pal, 1.0, false));
    }

    #[test]
    fn a_player_renders_a_sized_frame_then_keeps_it() {
        let grid = [by_id("neon_grid").expect("shader")];
        let pal = ShaderPalette::cyberpunk();
        let config = config_hash(&grid, &pal, 1.0, false);
        let mut player = Player::new();
        let req = |now| Request {
            chain: &grid,
            palette: &pal,
            size: (64.0, 36.0),
            scale: 2.0,
            pixel: 2.0,
            intensity: 1.0,
            time: 0.0,
            fps: 30.0,
            animated: false,
            transparent: false,
            config,
            now,
        };
        // The GPU opens in the background and pipelines compile on demand: poll until a frame.
        let deadline = Instant::now() + std::time::Duration::from_secs(60);
        let frame = loop {
            match player.poll(&req(Instant::now())) {
                Poll::New(f) => break f,
                _ => {
                    assert!(Instant::now() < deadline, "no frame in 60 s");
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        };
        assert_eq!(frame.bgra.len(), (frame.width * frame.height * 4) as usize);
        assert_eq!((frame.width, frame.height), (64, 36));
        // Not animated, same size and config: nothing new is due.
        assert!(matches!(player.poll(&req(Instant::now())), Poll::Keep));
    }
}

#[cfg(test)]
mod spec_tests {
    use super::*;

    #[test]
    fn frames_swap_to_rgba() {
        let f = Frame {
            width: 1,
            height: 1,
            bgra: vec![1, 2, 3, 4],
        };
        assert_eq!(f.rgba(), vec![3, 2, 1, 4]);
    }

    #[test]
    fn specs_compare_by_shader_id_and_settings() {
        let a = Spec::new("neon_grid").fx("crt");
        assert_eq!(a, Spec::new("neon_grid").fx("crt"));
        assert_ne!(a, Spec::new("neon_grid"));
        assert_ne!(a, a.clone().intensity(0.3));
        assert!(Spec::new("nope").chain.is_empty());
        assert_eq!(Spec::new("neon_grid").fps(500.0).fps, 120.0);
    }

    #[test]
    fn a_still_shader_has_no_next_tick() {
        let live = Live::new(Spec::new("neon_grid").animated(false));
        assert!(live.next_in().is_none());
        let live = Live::new(Spec::new("neon_grid").fps(10.0));
        let wait = live.next_in().expect("animated");
        assert!((wait.as_secs_f32() - 0.1).abs() < 1e-4);
    }
}
