//! Frosted glass: panels that show a *blurred* copy of the picture behind them, under the
//! design's translucent tint ([`Theme::frost`](dodai_core::Theme::frost)).
//!
//! `gpui` cannot blur what was already drawn, so a [`FrostedScene`] owns its backdrop: it
//! draws the sharp picture, and each panel clips a blurred copy of the same picture to its own
//! rectangle (with the panel's rounded corners). The blur is computed once per picture and
//! cached.

use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    div, img, px, App, IntoElement, ObjectFit, ParentElement, RenderImage, RenderOnce,
    SharedString, Styled, StyledImage, Window,
};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

/// Gaussian-like blur (three box passes) of an RGBA image, edges clamped. `sigma` is in pixels.
fn blur_pixels(src: &[u8], w: usize, h: usize, sigma: f32) -> Vec<u8> {
    assert_eq!(src.len(), w * h * 4);
    if sigma <= 0.0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    let wi = (12.0 * sigma * sigma / 3.0 + 1.0).sqrt();
    let r = (((wi - 1.0) / 2.0).round() as usize).max(1);
    let mut a = src.to_vec();
    let mut b = vec![0u8; a.len()];
    for _ in 0..3 {
        box_pass(&a, &mut b, w, h, r, true);
        box_pass(&b, &mut a, w, h, r, false);
    }
    a
}

/// One box-blur pass of radius `r` along rows (`horizontal`) or columns.
fn box_pass(src: &[u8], dst: &mut [u8], w: usize, h: usize, r: usize, horizontal: bool) {
    let (len, lines, step, line_step) = if horizontal {
        (w, h, 4, w * 4)
    } else {
        (h, w, w * 4, 4)
    };
    let win = (2 * r + 1) as u32;
    for line in 0..lines {
        let base = line * line_step;
        let at = |i: isize| base + i.clamp(0, len as isize - 1) as usize * step;
        for c in 0..4 {
            let mut sum: u32 = 0;
            for i in -(r as isize)..=(r as isize) {
                sum += src[at(i) + c] as u32;
            }
            for i in 0..len {
                dst[base + i * step + c] = ((sum + win / 2) / win) as u8;
                sum += src[at(i as isize + r as isize + 1) + c] as u32;
                sum -= src[at(i as isize - r as isize) + c] as u32;
            }
        }
    }
}

/// Average `f`×`f` blocks (`f` ≥ 1): the picture at `1/f` size.
fn downscale(src: &[u8], w: usize, h: usize, f: usize) -> (Vec<u8>, usize, usize) {
    if f <= 1 {
        return (src.to_vec(), w, h);
    }
    let (nw, nh) = ((w / f).max(1), (h / f).max(1));
    let mut out = vec![0u8; nw * nh * 4];
    for y in 0..nh {
        for x in 0..nw {
            let mut acc = [0u32; 4];
            let mut n = 0;
            for dy in 0..f {
                for dx in 0..f {
                    let (sx, sy) = ((x * f + dx).min(w - 1), (y * f + dy).min(h - 1));
                    for (c, a) in acc.iter_mut().enumerate() {
                        *a += src[(sy * w + sx) * 4 + c] as u32;
                    }
                    n += 1;
                }
            }
            for (c, a) in acc.iter().enumerate() {
                out[(y * nw + x) * 4 + c] = (a / n) as u8;
            }
        }
    }
    (out, nw, nh)
}

/// The blurred picture, at reduced size (it is scaled back up when drawn).
fn blurred(src: &[u8], w: usize, h: usize, sigma: f32) -> (Vec<u8>, usize, usize) {
    let f = (sigma / 6.0).ceil().max(1.0) as usize;
    let (small, sw, sh) = downscale(src, w, h, f);
    (blur_pixels(&small, sw, sh, sigma / f as f32), sw, sh)
}

/// A showcase picture with hard edges (so the blur is visible), in the theme's colours.
fn mesh_pixels(t: &Theme, w: usize, h: usize) -> Vec<u8> {
    let p = &t.palette;
    let colours = [p.primary, p.info, p.success, p.warning, p.danger];
    let mut out = vec![0u8; w * h * 4];
    let (fw, fh) = (w as f32, h as f32);
    for y in 0..h {
        for x in 0..w {
            let (u, v) = (x as f32 / fw, y as f32 / fh);
            let mut c = p.background.mix(p.surface_raised, 0.5);
            // Diagonal stripes.
            if ((u + v) * 14.0).fract() < 0.22 {
                c = c.mix(p.text, 0.18);
            }
            // Hard-edged discs.
            for (i, col) in colours.iter().enumerate() {
                let (cx, cy) = (0.12 + 0.19 * i as f32, if i % 2 == 0 { 0.35 } else { 0.7 });
                let d = ((u - cx) * fw / fh).hypot(v - cy);
                if d < 0.17 {
                    c = *col;
                }
            }
            let o = (y * w + x) * 4;
            out[o..o + 4].copy_from_slice(&[c.r, c.g, c.b, 255]);
        }
    }
    out
}

fn render_image(rgba: Vec<u8>, w: usize, h: usize) -> Option<Arc<RenderImage>> {
    // gpui stores frames as BGRA.
    let mut rgba = rgba;
    for px in rgba.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    let buf = image::RgbaImage::from_raw(w as u32, h as u32, rgba)?;
    Some(Arc::new(RenderImage::new(vec![image::Frame::new(buf)])))
}

/// Cached pictures (and panel crops) kept before the cache is emptied, so switching themes
/// does not grow it without bound.
const CACHE_LIMIT: usize = 32;

/// A picture and its blurred, reduced copy.
#[derive(Clone)]
struct Entry {
    sharp: Arc<RenderImage>,
    soft: Arc<Vec<u8>>,
    soft_size: (usize, usize),
}

fn cache() -> &'static Mutex<HashMap<u64, Entry>> {
    static C: OnceLock<Mutex<HashMap<u64, Entry>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

fn crops() -> &'static Mutex<HashMap<u64, Arc<RenderImage>>> {
    static C: OnceLock<Mutex<HashMap<u64, Arc<RenderImage>>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

/// The `w`×`h` rectangle at `(x, y)` of the picture (`full` px wide) cut from its reduced blurred
/// copy `soft` (`sw`×`sh`) by bilinear sampling, so it lines up with the sharp picture.
fn crop_bilinear(
    soft: &[u8],
    (sw, sh): (usize, usize),
    full: (f32, f32),
    (x, y, w, h): (usize, usize, usize, usize),
) -> Vec<u8> {
    let (kx, ky) = (sw as f32 / full.0, sh as f32 / full.1);
    let mut out = vec![0u8; w * h * 4];
    for j in 0..h {
        for i in 0..w {
            let fx = ((x + i) as f32 + 0.5) * kx - 0.5;
            let fy = ((y + j) as f32 + 0.5) * ky - 0.5;
            let (x0, y0) = (fx.floor(), fy.floor());
            let (tx, ty) = (fx - x0, fy - y0);
            let at = |xx: f32, yy: f32, c: usize| {
                let (xx, yy) = (
                    (xx.max(0.0) as usize).min(sw - 1),
                    (yy.max(0.0) as usize).min(sh - 1),
                );
                soft[(yy * sw + xx) * 4 + c] as f32
            };
            for c in 0..4 {
                let top = at(x0, y0, c) * (1.0 - tx) + at(x0 + 1.0, y0, c) * tx;
                let bot = at(x0, y0 + 1.0, c) * (1.0 - tx) + at(x0 + 1.0, y0 + 1.0, c) * tx;
                out[(j * w + i) * 4 + c] = (top * (1.0 - ty) + bot * ty).round() as u8;
            }
        }
    }
    out
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
            let (bl, bw, bh) = blurred(&rgba, w, h, frost.blur);
            let e = Entry {
                sharp: render_image(rgba, w, h)?,
                soft: Arc::new(bl),
                soft_size: (bw, bh),
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
                let pixels = crop_bilinear(
                    &entry.soft,
                    entry.soft_size,
                    (sw, sh),
                    (x as usize, y as usize, cw, ch),
                );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blur_keeps_flat_colour_and_spreads_an_edge() {
        let (w, h) = (16, 4);
        let flat = vec![200u8; w * h * 4];
        assert!(blur_pixels(&flat, w, h, 3.0).iter().all(|&v| v == 200));
        let mut edge = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in w / 2..w {
                edge[(y * w + x) * 4..][..4].copy_from_slice(&[255; 4]);
            }
        }
        let out = blur_pixels(&edge, w, h, 2.0);
        let at = |x: usize| out[x * 4];
        assert!(at(w / 2 - 1) > 0 && at(w / 2) < 255, "edge softens");
        assert!(at(0) < at(w / 2) && at(w / 2) < at(w - 1), "monotonic");
    }

    #[test]
    fn crop_lines_up_with_the_picture() {
        let (sw, sh) = (4, 2);
        let mut soft = vec![255u8; sw * sh * 4];
        for y in 0..sh {
            for x in 0..sw {
                soft[(y * sw + x) * 4] = (x * 60) as u8;
            }
        }
        let c = crop_bilinear(&soft, (sw, sh), (8.0, 4.0), (2, 1, 4, 2));
        assert_eq!(c.len(), 4 * 2 * 4);
        assert!(c[0] < c[4 * 3], "ramp keeps rising across the crop");
    }

    #[test]
    fn blurred_picture_is_reduced_by_the_sigma() {
        let src = vec![9u8; 64 * 32 * 4];
        let (_, w, h) = blurred(&src, 64, 32, 12.0);
        assert_eq!((w, h), (32, 16));
    }
}
