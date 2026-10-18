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
            pixel: 3.0,
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
    pub fn palette(mut self, p: PaletteSource) -> Self {
        self.palette = p;
        self
    }
    /// Width ÷ height (default 16:9).
    pub fn aspect(mut self, a: f32) -> Self {
        self.aspect = a.max(0.1);
        self
    }
    /// Device pixels per shader pixel (default 3).
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
    pub fn rounded(mut self, r: f32) -> Self {
        self.rounded = r;
        self
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
                let (image, old) = slot.update(cx, |s, _| {
                    let start = *s.start.get_or_insert(now);
                    let secs = (now - start).as_secs_f32();
                    let pl = s
                        .player
                        .get_or_insert_with(|| Player::new(chain.clone(), pal));
                    pl.chain.clone_from(&chain);
                    pl.palette = pal;
                    pl.intensity = intensity;
                    pl.speed = speed;
                    pl.pixel = pixel;
                    pl.max_fps = if animated { fps } else { 0.0 };
                    // Off-screen views keep their last picture and cost nothing.
                    if !visible && s.image.is_some() {
                        return (s.image.clone().expect("checked"), None);
                    }
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
