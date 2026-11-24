//! gpui (feature `gpui`): `ShaderView`: shows a `dodai-shaders` chain as a live, fluid-width image.

use crate::player::{config_hash, Player, Poll, Request};
use crate::{by_id, Family, ShaderInfo, ShaderPalette};
use dodai_gpui::theme_of;

pub use crate::player::backend_label as shader_backend;
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
    player: Player,
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
    transparent: bool,
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
            transparent: false,
        }
    }
    /// A ready-made chain from [`crate::presets`].
    pub fn preset(id: impl Into<SharedString>, p: &crate::Preset) -> Self {
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
    /// Draw light on nothing: the darker a pixel, the more see-through it is, so a shader that is
    /// light on black (waves, rain, sparks) lays over whatever is behind the view as an overlay.
    pub fn transparent(mut self, on: bool) -> Self {
        self.transparent = on;
        self
    }
    /// Corner radius in px.
    pub fn rounded(mut self, r: f32) -> Self {
        self.rounded = r;
        self
    }
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
        let transparent = self.transparent;
        let (aspect, pixel, intensity, speed, animated, rounded, fps) = (
            self.aspect,
            self.pixel,
            self.intensity,
            self.speed,
            self.animated,
            self.rounded,
            self.fps,
        );
        let cfg = config_hash(&chain, &pal, intensity, transparent);
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
                        let poll = s.player.poll(&Request {
                            chain: &chain,
                            palette: &pal,
                            size: (bw, bh),
                            scale,
                            pixel,
                            intensity,
                            time: secs * speed,
                            fps,
                            animated,
                            transparent,
                            config: cfg,
                            now,
                        });
                        match poll {
                            Poll::Pending => {}
                            Poll::Keep => return s.image.clone().map(|i| (i, None)),
                            Poll::New(f) => {
                                if let Some(buf) =
                                    image::RgbaImage::from_raw(f.width, f.height, f.bgra)
                                {
                                    let img =
                                        Arc::new(RenderImage::new(vec![image::Frame::new(buf)]));
                                    s.fresh_at = Some(now);
                                    let old = s.image.replace(img.clone());
                                    return Some((img, old));
                                }
                            }
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
                        dodai_gpui::motion::next_frame(w);
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

/// A solved [`radiance`](crate::radiance) scene as a gpui image, for `gpui::img`.
/// `None` only when the scene is empty.
pub fn radiance_image(
    scene: &crate::radiance::Scene,
    light: &[crate::math::Vec3],
    exposure: f32,
) -> Option<Arc<RenderImage>> {
    let rgb = crate::radiance::picture(scene, light, exposure);
    // gpui images are BGRA
    let bgra: Vec<u8> = rgb
        .chunks(3)
        .flat_map(|p| [p[2], p[1], p[0], 255])
        .collect();
    let buf = image::RgbaImage::from_raw(scene.width(), scene.height(), bgra)?;
    (scene.width() > 0 && scene.height() > 0)
        .then(|| Arc::new(RenderImage::new(vec![image::Frame::new(buf)])))
}
