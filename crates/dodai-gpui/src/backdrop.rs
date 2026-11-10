//! The canvas mesh: a design whose [`SurfaceStyle::backdrop`](dodai_core::SurfaceStyle) is
//! above zero lights its canvas with soft blobs of its wash colours, and translucent panels
//! let it show through. The picture is tiny and smooth, so scaling it up is the blur.

use dodai_core::Theme;
use gpui::{img, IntoElement, ObjectFit, ParentElement, RenderImage, Styled, StyledImage};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

const W: usize = dodai_core::MESH_SIZE.0;
const H: usize = dodai_core::MESH_SIZE.1;
/// Width and height of the mesh picture.
#[cfg(feature = "frost")]
pub(crate) const MESH_SIZE: (usize, usize) = (W, H);
const CACHE_LIMIT: usize = 16;

/// A tiny RGBA picture as a gpui image.
pub(crate) fn render_image(rgba: Vec<u8>, w: usize, h: usize) -> Option<Arc<RenderImage>> {
    // gpui stores frames as BGRA.
    let mut rgba = rgba;
    for px in rgba.as_chunks_mut::<4>().0 {
        px.swap(0, 2);
    }
    let buf = image::RgbaImage::from_raw(w as u32, h as u32, rgba)?;
    Some(Arc::new(RenderImage::new(vec![image::Frame::new(buf)])))
}

fn cache() -> &'static Mutex<HashMap<u64, Arc<RenderImage>>> {
    static C: OnceLock<Mutex<HashMap<u64, Arc<RenderImage>>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

/// The mesh pixels (see [`dodai_core::mesh_pixels`]).
pub(crate) use dodai_core::mesh_pixels as pixels;

/// The canvas mesh of `t`, to place as the first child of the (relative) root; `None` for a
/// design with a flat canvas. With the `frost` feature, the wallpaper the app set with
/// [`set_wallpaper`](crate::set_wallpaper) is drawn instead, for any design.
pub fn backdrop(t: &Theme) -> Option<impl IntoElement> {
    #[cfg(feature = "frost")]
    if let Some(image) = crate::glass::wallpaper_image() {
        return Some(cover(image));
    }
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
    Some(cover(image))
}

/// `image` stretched over its (relative) parent.
fn cover(image: Arc<RenderImage>) -> gpui::Div {
    gpui::div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .child(img(image).object_fit(ObjectFit::Fill).w_full().h_full())
}
