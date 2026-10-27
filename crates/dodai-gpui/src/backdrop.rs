//! The canvas mesh: a design whose [`SurfaceStyle::backdrop`](dodai_core::SurfaceStyle) is
//! above zero lights its canvas with soft blobs of its wash colours, and translucent panels
//! let it show through. The picture is tiny and smooth, so scaling it up is the blur.

use dodai_core::{Color, Theme};
use gpui::{img, IntoElement, ObjectFit, ParentElement, RenderImage, Styled, StyledImage};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

const W: usize = 96;
const H: usize = 60;
const CACHE_LIMIT: usize = 16;

/// Blob centres `(x, y, radius)` in unit coordinates, one per wash colour, plus a faint fourth.
const BLOBS: [(f32, f32, f32); 4] = [
    (0.10, 0.05, 0.70),
    (0.92, 0.30, 0.65),
    (0.45, 1.00, 0.75),
    (0.60, 0.10, 0.40),
];

/// A tiny RGBA picture as a gpui image.
pub(crate) fn render_image(rgba: Vec<u8>, w: usize, h: usize) -> Option<Arc<RenderImage>> {
    // gpui stores frames as BGRA.
    let mut rgba = rgba;
    for px in rgba.chunks_exact_mut(4) {
        px.swap(0, 2);
    }
    let buf = image::RgbaImage::from_raw(w as u32, h as u32, rgba)?;
    Some(Arc::new(RenderImage::new(vec![image::Frame::new(buf)])))
}

fn cache() -> &'static Mutex<HashMap<u64, Arc<RenderImage>>> {
    static C: OnceLock<Mutex<HashMap<u64, Arc<RenderImage>>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

/// The mesh pixels: the canvas with the three wash colours (and the second again, fainter)
/// blended in with a smooth falloff of `strength`.
pub(crate) fn pixels(
    canvas: Color,
    wash: [Color; 3],
    strength: f32,
    w: usize,
    h: usize,
) -> Vec<u8> {
    let colours = [wash[0], wash[1], wash[2], wash[1]];
    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let (u, v) = (x as f32 / w as f32, y as f32 / h as f32);
            let mut c = canvas;
            for (i, (cx, cy, r)) in BLOBS.iter().enumerate() {
                // The picture is wider than tall: keep the blobs round.
                let d = ((u - cx) * (w as f32 / h as f32)).hypot(v - cy) / r;
                let f = (1.0 - d).clamp(0.0, 1.0);
                let k = f * f * (3.0 - 2.0 * f) * strength * if i == 3 { 0.5 } else { 1.0 };
                c = c.mix(colours[i], k.min(1.0));
            }
            let o = (y * w + x) * 4;
            out[o..o + 4].copy_from_slice(&[c.r, c.g, c.b, 255]);
        }
    }
    out
}

/// The canvas mesh of `t`, to place as the first child of the (relative) root; `None` for a
/// design with a flat canvas.
pub fn backdrop(t: &Theme) -> Option<impl IntoElement> {
    let strength = t.surface_style.backdrop;
    if strength <= 0.0 {
        return None;
    }
    let wash = t.surface_style.wash;
    let mut hs = std::collections::hash_map::DefaultHasher::new();
    (t.palette.background, wash, strength.to_bits()).hash(&mut hs);
    let key = hs.finish();
    let cached = cache().lock().ok().and_then(|c| c.get(&key).cloned());
    let image = cached.or_else(|| {
        let i = render_image(pixels(t.palette.background, wash, strength, W, H), W, H)?;
        if let Ok(mut c) = cache().lock() {
            if c.len() >= CACHE_LIMIT {
                c.clear();
            }
            c.insert(key, i.clone());
        }
        Some(i)
    })?;
    Some(
        gpui::div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .child(img(image).object_fit(ObjectFit::Fill).w_full().h_full()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_where_a_blob_reaches_and_a_flat_canvas_at_zero() {
        let canvas = Color::rgba(250, 250, 252, 255);
        let wash = [
            Color::rgba(120, 60, 220, 255),
            Color::rgba(40, 80, 230, 255),
            Color::rgba(20, 190, 190, 255),
        ];
        let px = pixels(canvas, wash, 0.5, W, H);
        let o = (2 * W + 2) * 4;
        assert_ne!(px[o..o + 3], [250, 250, 252]);
        let flat = pixels(canvas, wash, 0.0, W, H);
        assert!(flat.chunks_exact(4).all(|p| p[..3] == [250, 250, 252]));
    }
}
