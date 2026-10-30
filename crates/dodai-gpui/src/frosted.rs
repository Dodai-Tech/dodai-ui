//! Frosted glass: panels that show a *blurred* copy of the picture behind them, under the
//! design's translucent tint ([`Theme::frost`](dodai_core::Theme::frost)).
//!
//! `gpui` cannot blur what was already drawn, so a [`FrostedScene`] owns its backdrop: it
//! draws the sharp picture, and each panel clips a blurred copy of the same picture to its own
//! rectangle (with the panel's rounded corners). The blur is computed once per picture and
//! cached.

use crate::backdrop::render_image;
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    div, img, px, App, IntoElement, ObjectFit, ParentElement, RenderImage, RenderOnce,
    SharedString, Styled, StyledImage, Window,
};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

/// A showcase picture with hard edges (so the blur is visible), in the theme's colours: a dusk
/// sky with a low sun, stars and three ridges of mountains, each layer darker than the last.
fn mesh_pixels(t: &Theme, w: usize, h: usize) -> Vec<u8> {
    let p = &t.palette;
    let (top, horizon) = (p.primary.mix(p.text, 0.25), p.warning.mix(p.danger, 0.3));
    let sun = p.warning.mix(p.surface_raised, 0.6);
    // Far to near: lighter and cooler first, darkest last.
    let ridges = [
        (0.56, 0.12, 1.7, 0.4, p.info.mix(p.primary, 0.4)),
        (0.68, 0.14, 2.6, 2.1, p.primary.mix(p.success, 0.35)),
        (0.84, 0.12, 3.9, 4.0, p.success.mix(p.text, 0.55)),
    ];
    let mut out = vec![0u8; w * h * 4];
    let (fw, fh) = (w as f32, h as f32);
    for y in 0..h {
        for x in 0..w {
            let (u, v) = (x as f32 / fw, y as f32 / fh);
            // Sky: a vertical blend, pulled toward the horizon.
            let mut c = top.mix(horizon, v.powf(1.4));
            // Hard-edged sun and a few stars (a hash picks the cells that hold one).
            if ((u - 0.5) * fw / fh).hypot(v - 0.42) < 0.13 {
                c = sun;
            }
            let (cx, cy) = (x / 14, y / 14);
            let hash = (cx * 73_856_093) ^ (cy * 19_349_663);
            if v < 0.4 && hash % 17 == 0 && (x % 14).abs_diff(7) < 2 && (y % 14).abs_diff(7) < 2 {
                c = p.surface_raised;
            }
            // Ridges: sharp peaks from folded sines.
            for (base, amp, freq, phase, col) in ridges {
                let peak = 1.0 - (u * freq * std::f32::consts::TAU + phase).sin().abs();
                let wobble = (u * freq * 3.1 + phase).sin() * 0.25 + 0.75;
                if v > base - amp * peak * wobble {
                    c = col;
                }
            }
            let o = (y * w + x) * 4;
            out[o..o + 4].copy_from_slice(&[c.r, c.g, c.b, 255]);
        }
    }
    out
}

/// Cached pictures (and panel crops) kept before the cache is emptied, so switching themes
/// does not grow it without bound.
const CACHE_LIMIT: usize = 32;

/// A picture and its blurred, reduced copy.
#[derive(Clone)]
struct Entry {
    sharp: Arc<RenderImage>,
    soft: Arc<dodai_frost::Reduced>,
}

fn cache() -> &'static Mutex<HashMap<u64, Entry>> {
    static C: OnceLock<Mutex<HashMap<u64, Entry>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

fn crops() -> &'static Mutex<HashMap<u64, Arc<RenderImage>>> {
    static C: OnceLock<Mutex<HashMap<u64, Arc<RenderImage>>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

enum Source {
    Mesh,
    Pixels(Arc<Vec<u8>>),
}

/// A picture with frosted-glass panels over it.
#[derive(IntoElement)]
pub struct FrostedScene {
    id: SharedString,
    size: (f32, f32),
    source: Source,
    radius: Option<f32>,
    panels: Vec<(f32, f32, f32, f32, gpui::AnyElement)>,
}

impl FrostedScene {
    /// A scene over a generated picture in the theme's colours. `id` keys the cache.
    pub fn mesh(id: impl Into<SharedString>, width: f32, height: f32) -> Self {
        Self::with(id, width, height, Source::Mesh)
    }
    /// A scene over an picture of 4-byte pixels of exactly `width`×`height` pixels.
    pub fn pixels(
        id: impl Into<SharedString>,
        width: u32,
        height: u32,
        rgba: Arc<Vec<u8>>,
    ) -> Self {
        Self::with(id, width as f32, height as f32, Source::Pixels(rgba))
    }
    fn with(id: impl Into<SharedString>, w: f32, h: f32, source: Source) -> Self {
        Self {
            id: id.into(),
            size: (w, h),
            source,
            radius: None,
            panels: Vec::new(),
        }
    }
    /// Corner radius of the panels (default: the theme's medium spacing).
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = Some(r);
        self
    }
    /// A glass panel at `(x, y)` of `(width, height)` px, over the picture, holding `content`
    /// (inset by the theme's medium spacing).
    pub fn panel(
        mut self,
        (x, y): (f32, f32),
        (width, height): (f32, f32),
        content: impl IntoElement,
    ) -> Self {
        self.panels
            .push((x, y, width, height, content.into_any_element()));
        self
    }
}

impl RenderOnce for FrostedScene {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let frost = t.frost();
        let (sw, sh) = self.size;
        let mut hs = std::collections::hash_map::DefaultHasher::new();
        (
            self.id.as_ref(),
            sw.to_bits(),
            sh.to_bits(),
            frost.blur.to_bits(),
            frost.saturation.to_bits(),
        )
            .hash(&mut hs);
        match &self.source {
            Source::Mesh => {
                let p = &t.palette;
                [
                    p.background,
                    p.surface_raised,
                    p.primary,
                    p.info,
                    p.success,
                    p.warning,
                    p.danger,
                    p.text,
                ]
                .hash(&mut hs);
                (t.surface_style.wash, t.surface_style.backdrop.to_bits()).hash(&mut hs);
            }
            Source::Pixels(b) => (b.len(), b.first(), b.last()).hash(&mut hs),
        }
        let key = hs.finish();
        let cached = cache().lock().ok().and_then(|c| c.get(&key).cloned());
        let pair = cached.or_else(|| {
            let (w, h) = (sw as usize, sh as usize);
            let rgba = match &self.source {
                Source::Mesh => mesh_pixels(&t, w, h),
                Source::Pixels(b) => b.to_vec(),
            };
            let soft = dodai_frost::blurred(&rgba, w, h, frost.blur);
            let e = Entry {
                sharp: render_image(rgba, w, h)?,
                soft: Arc::new(soft),
            };
            if let Ok(mut c) = cache().lock() {
                if c.len() >= CACHE_LIMIT {
                    c.clear();
                }
                c.insert(key, e.clone());
            }
            Some(e)
        });
        let r = self.radius.unwrap_or(t.spacing.md);
        let mut root = div().relative().w(px(sw)).h(px(sh)).overflow_hidden();
        let Some(entry) = pair else {
            return root;
        };
        root = root.child(
            img(entry.sharp.clone())
                .object_fit(ObjectFit::Fill)
                .absolute()
                .left_0()
                .top_0()
                .w(px(sw))
                .h(px(sh)),
        );
        for (x, y, w, h, content) in self.panels {
            let mut hs = std::collections::hash_map::DefaultHasher::new();
            (key, x.to_bits(), y.to_bits(), w.to_bits(), h.to_bits()).hash(&mut hs);
            let ck = hs.finish();
            let cached = crops().lock().ok().and_then(|c| c.get(&ck).cloned());
            let crop = cached.or_else(|| {
                let (cw, ch) = (w as usize, h as usize);
                let mut pixels = entry.soft.crop(x as usize, y as usize, cw, ch);
                dodai_frost::saturate(&mut pixels, frost.saturation);
                let i = render_image(pixels, cw, ch)?;
                if let Ok(mut c) = crops().lock() {
                    if c.len() >= CACHE_LIMIT {
                        c.clear();
                    }
                    c.insert(ck, i.clone());
                }
                Some(i)
            });
            let mut glass = div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .w(px(w))
                .h(px(h))
                .rounded(px(r));
            if let Some(c) = crop {
                glass = glass.child(
                    img(c)
                        .object_fit(ObjectFit::Fill)
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .rounded(px(r)),
                );
            }
            root = root.child(
                glass
                    .child({
                        let mut tint = div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full()
                            .rounded(px(r))
                            .bg(hsla(frost.tint));
                        if let Some((c, wd)) = frost.edge {
                            tint = tint.border(px(wd)).border_color(hsla(c));
                        }
                        tint
                    })
                    .child(
                        div()
                            .relative()
                            .size_full()
                            .p(px(t.spacing.md))
                            .child(content),
                    ),
            );
        }
        root
    }
}
