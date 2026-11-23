//! Frosted glass: panels that show a *blurred* copy of the picture behind them, under the
//! design's translucent tint ([`Theme::frost`](dodai_core::Theme::frost)).
//!
//! `gpui` cannot blur what was already drawn, so a [`FrostedScene`] owns its backdrop: it
//! draws the sharp picture, and each panel clips a blurred copy of the same picture to its own
//! rectangle (with the panel's rounded corners). The blur is computed once per picture and
//! cached.

use crate::backdrop::render_image;
use crate::lru::Lru;
use crate::{hsla, theme_of};
use dodai_core::{Palette, Theme};
use gpui::{
    div, img, px, App, IntoElement, ObjectFit, ParentElement, RenderImage, RenderOnce,
    SharedString, Styled, StyledImage, Window,
};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

/// Scenes and panel crops kept; the least recently used is dropped (and its texture freed).
const CACHE_LIMIT: usize = 32;

/// A picture and its blurred, reduced copy. `_pin` keeps caller pixels alive so the address
/// they are keyed by cannot be reused by another picture while the entry exists.
#[derive(Clone)]
struct Entry {
    sharp: Arc<RenderImage>,
    soft: Arc<dodai_frost::Reduced>,
    _pin: Option<Arc<Vec<u8>>>,
}

/// A scene's picture: being blurred off the UI thread, ready, or impossible (bad size).
#[derive(Clone)]
enum Slot {
    Pending,
    Ready(Entry),
    Failed,
}

struct Store {
    scenes: Lru<u64, Slot>,
    crops: Lru<u64, Arc<RenderImage>>,
    /// Evicted textures, freed on the next render (that is where a `Window` is at hand).
    freed: Vec<Arc<RenderImage>>,
}

fn store() -> &'static Mutex<Store> {
    static S: OnceLock<Mutex<Store>> = OnceLock::new();
    S.get_or_init(|| {
        Mutex::new(Store {
            scenes: Lru::new(CACHE_LIMIT),
            crops: Lru::new(CACHE_LIMIT),
            freed: Vec::new(),
        })
    })
}

/// One blurrer for the app: GPU first, CPU without a real adapter. Opened lazily on the
/// first background blur, never on the UI thread.
pub(crate) fn blurrer() -> &'static Mutex<dodai_frost::Blurrer> {
    static B: OnceLock<Mutex<dodai_frost::Blurrer>> = OnceLock::new();
    B.get_or_init(|| Mutex::new(dodai_frost::Blurrer::new()))
}

fn sharp_images(slots: Vec<Slot>) -> impl Iterator<Item = Arc<RenderImage>> {
    slots.into_iter().filter_map(|s| match s {
        Slot::Ready(e) => Some(e.sharp),
        _ => None,
    })
}

/// The scene's picture and its blur; `None` when the pixels do not match the size.
fn build(
    palette: &Palette,
    pixels: Option<Arc<Vec<u8>>>,
    (w, h): (usize, usize),
    blur: f32,
) -> Option<Entry> {
    let rgba = match &pixels {
        None => dodai_core::frost_scene(palette, w, h),
        Some(b) if b.len() == w * h * 4 => b.to_vec(),
        Some(_) => return None,
    };
    let soft = blurrer()
        .lock()
        .map(|mut b| b.blurred(&rgba, w, h, blur))
        .unwrap_or_else(|e| e.into_inner().blurred(&rgba, w, h, blur));
    Some(Entry {
        sharp: render_image(rgba, w, h)?,
        soft: Arc::new(soft),
        _pin: pixels,
    })
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

/// The cache key of a scene: its id, size, the frost it is blurred with, and its picture (the
/// theme colours for the generated one; the pixels' address for caller pixels, which the cache
/// keeps alive so the address stays unique).
fn scene_key(
    id: &str,
    size: (f32, f32),
    frost: &dodai_core::Frost,
    source: &Source,
    t: &Theme,
) -> u64 {
    let mut hs = std::collections::hash_map::DefaultHasher::new();
    (
        id,
        size.0.to_bits(),
        size.1.to_bits(),
        frost.blur.to_bits(),
        frost.saturation.to_bits(),
    )
        .hash(&mut hs);
    match source {
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
        }
        Source::Pixels(b) => (Arc::as_ptr(b) as usize, b.len()).hash(&mut hs),
    }
    hs.finish()
}

impl RenderOnce for FrostedScene {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let frost = t.frost();
        let (sw, sh) = self.size;
        let key = scene_key(self.id.as_ref(), self.size, &frost, &self.source, &t);
        let (slot, freed) = {
            let mut s = store().lock().unwrap_or_else(|e| e.into_inner());
            let slot = s.scenes.get(key);
            if slot.is_none() {
                let out = s.scenes.insert(key, Slot::Pending);
                s.freed.extend(sharp_images(out));
            }
            (slot, std::mem::take(&mut s.freed))
        };
        for image in freed {
            cx.drop_image(image, Some(window));
        }
        if slot.is_none() && sw >= 1.0 && sh >= 1.0 {
            // Blur off the UI thread (on the GPU when there is one), then repaint this view.
            let view = window.current_view();
            let exec = cx.background_executor().clone();
            let palette = t.palette.clone();
            let pixels = match &self.source {
                Source::Mesh => None,
                Source::Pixels(b) => Some(b.clone()),
            };
            let (size, blur) = ((sw as usize, sh as usize), frost.blur);
            window
                .spawn(cx, async move |acx| {
                    let built = exec
                        .spawn(async move { build(&palette, pixels, size, blur) })
                        .await;
                    if let Ok(mut s) = store().lock() {
                        let out = s
                            .scenes
                            .insert(key, built.map_or(Slot::Failed, Slot::Ready));
                        s.freed.extend(sharp_images(out));
                    }
                    let _ = acx.update(|_, cx| cx.notify(view));
                })
                .detach();
        }
        let r = self.radius.unwrap_or(t.spacing.md);
        let mut root = div().relative().w(px(sw)).h(px(sh)).overflow_hidden();
        let entry = match slot {
            Some(Slot::Ready(e)) => Some(e),
            _ => None,
        };
        if let Some(e) = &entry {
            root = root.child(
                img(e.sharp.clone())
                    .object_fit(ObjectFit::Fill)
                    .absolute()
                    .left_0()
                    .top_0()
                    .w(px(sw))
                    .h(px(sh)),
            );
        }
        for (x, y, w, h, content) in self.panels {
            let crop = entry.as_ref().and_then(|e| {
                let mut hs = std::collections::hash_map::DefaultHasher::new();
                (key, x.to_bits(), y.to_bits(), w.to_bits(), h.to_bits()).hash(&mut hs);
                let ck = hs.finish();
                let mut s = store().lock().unwrap_or_else(|e| e.into_inner());
                if let Some(i) = s.crops.get(ck) {
                    return Some(i);
                }
                let (cw, ch) = (w as usize, h as usize);
                let mut pixels = e.soft.crop(x as usize, y as usize, cw, ch);
                dodai_frost::saturate(&mut pixels, frost.saturation);
                let i = render_image(pixels, cw, ch)?;
                let out = s.crops.insert(ck, i.clone());
                s.freed.extend(out);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caller_pixels_are_keyed_by_identity_not_by_their_edges() {
        let t = Theme::default();
        let frost = t.frost();
        let a = Arc::new(vec![9u8; 16]);
        let mut inner = vec![9u8; 16];
        inner[5] = 200;
        let b = Arc::new(inner);
        let key = |s: &Source| scene_key("x", (2.0, 2.0), &frost, s, &t);
        let (ka, kb) = (key(&Source::Pixels(a.clone())), key(&Source::Pixels(b)));
        assert_ne!(
            ka, kb,
            "same length and same first/last byte must not collide"
        );
        assert_eq!(ka, key(&Source::Pixels(a)));
    }

    #[test]
    fn pixels_of_the_wrong_size_fail_instead_of_panicking() {
        let p = Theme::default().palette;
        assert!(build(&p, Some(Arc::new(vec![0u8; 10])), (2, 2), 4.0).is_none());
        let ok = build(&p, Some(Arc::new(vec![128u8; 16])), (2, 2), 4.0);
        assert!(ok.is_some());
        assert!(build(&p, None, (8, 4), 2.0).is_some());
    }
}
