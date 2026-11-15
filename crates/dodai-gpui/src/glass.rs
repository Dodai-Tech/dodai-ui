//! Frosted glass on any component: a blurred copy of the app's backdrop under a see-through fill.
//!
//! `gpui` cannot blur what was already drawn, so the glass blurs a picture it knows: the
//! wallpaper the app set with [`set_wallpaper`], or else the design's canvas mesh
//! ([`backdrop`](crate::backdrop())). Both are drawn stretched over the whole window, so the
//! part under a component is found from where the component sits in the window. Components
//! over other components still show the backdrop, not what lies between (`gpui` has no way to
//! read back or blur what it already drew), so surfaces that float over content (popups, menus,
//! tooltips, toasts) use [`ShapeStyle::with_floating_surface`](crate::ShapeStyle), which keeps
//! their fill at least [`FLOATING_ALPHA`](crate::paint::FLOATING_ALPHA) opaque.
//!
//! Turn it on with the surface mix (`ResolvedSurface::frost`, containers and input fields) or per component
//! with `Surface::new().frost(1.0)`.

use crate::lru::Lru;
use crate::paint::FrostLayer;
use dodai_core::{Shape, ShapeKind};
use gpui::{px, App, Bounds, Corners, Pixels, RenderImage, Window};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

/// Blurred backdrops kept (one per wallpaper and blur).
const SOFT_LIMIT: usize = 4;
/// Component crops kept; scrolling makes new ones, the oldest are freed.
const CROP_LIMIT: usize = 96;
/// A crop is sampled at a fraction of the component's size: the backdrop is blurred, so the
/// GPU's scaling hides it. Shapes cut by a mask get twice as many pixels for a cleaner edge.
const CROP_STEP: f32 = 4.0;
const CROP_STEP_MASKED: f32 = 2.0;

/// The wallpaper the app set: its pixels and the drawable copy.
struct Wall {
    id: u64,
    rgba: Arc<Vec<u8>>,
    size: (usize, usize),
    image: Arc<RenderImage>,
}

/// A blurred backdrop: RGBA pixels of `size`.
#[derive(Clone)]
struct Soft {
    rgba: Arc<Vec<u8>>,
    size: (usize, usize),
}

#[derive(Clone)]
enum Slot {
    Pending,
    Ready(Soft),
}

struct Store {
    wall: Option<Wall>,
    next_id: u64,
    soft: Lru<u64, Slot>,
    crops: Lru<u64, Arc<RenderImage>>,
    /// Textures no longer used, freed before the next frame paints.
    freed: Vec<Arc<RenderImage>>,
}

fn store() -> &'static Mutex<Store> {
    static S: OnceLock<Mutex<Store>> = OnceLock::new();
    S.get_or_init(|| {
        Mutex::new(Store {
            wall: None,
            next_id: 1,
            soft: Lru::new(SOFT_LIMIT),
            crops: Lru::new(CROP_LIMIT),
            freed: Vec::new(),
        })
    })
}

fn locked() -> std::sync::MutexGuard<'static, Store> {
    store().lock().unwrap_or_else(|e| e.into_inner())
}

/// Draw `rgba` (4-byte pixels, `width`×`height`) behind the app instead of the design's canvas
/// mesh, and frost components over it. Ignored when the size does not match the pixels.
pub fn set_wallpaper(cx: &mut App, rgba: Vec<u8>, width: usize, height: usize) {
    if width == 0 || height == 0 || rgba.len() != width * height * 4 {
        return;
    }
    let Some(image) = crate::backdrop::render_image(rgba.clone(), width, height) else {
        return;
    };
    {
        let mut s = locked();
        let id = s.next_id;
        s.next_id += 1;
        if let Some(old) = s.wall.replace(Wall {
            id,
            rgba: Arc::new(rgba),
            size: (width, height),
            image,
        }) {
            s.freed.push(old.image);
        }
    }
    cx.refresh_windows();
}

/// Go back to the design's canvas mesh behind the app.
pub fn clear_wallpaper(cx: &mut App) {
    {
        let mut s = locked();
        if let Some(old) = s.wall.take() {
            s.freed.push(old.image);
        }
    }
    cx.refresh_windows();
}

/// Whether the app set a wallpaper.
pub fn has_wallpaper() -> bool {
    locked().wall.is_some()
}

/// The wallpaper the app set, ready to draw.
pub(crate) fn wallpaper_image() -> Option<Arc<RenderImage>> {
    locked().wall.as_ref().map(|w| w.image.clone())
}

/// Free textures dropped since the last frame. Called before painting, never while a texture
/// may still be drawn in the frame being built.
pub(crate) fn free_unused(window: &mut Window, cx: &mut App) {
    let freed = std::mem::take(&mut locked().freed);
    for image in freed {
        cx.drop_image(image, Some(window));
    }
}

/// Paint the glass under a component's fill: the part of the blurred backdrop under `bounds`,
/// in `shape`. Nothing while the backdrop is still being blurred, or when there is none.
pub(crate) fn paint_under(
    window: &mut Window,
    cx: &mut App,
    bounds: Bounds<Pixels>,
    shape: &Shape,
    layer: &FrostLayer,
) {
    let view = window.viewport_size();
    let (vw, vh): (f32, f32) = (view.width.into(), view.height.into());
    let (x, y): (f32, f32) = (bounds.origin.x.into(), bounds.origin.y.into());
    let (w, h): (f32, f32) = (bounds.size.width.into(), bounds.size.height.into());
    let usable = |v: f32| v.is_finite() && v >= 1.0;
    if !(usable(vw) && usable(vh) && usable(w) && usable(h) && x.is_finite() && y.is_finite()) {
        return;
    }
    let Some((key, soft)) = soft_backdrop(cx, layer, vw) else {
        return;
    };
    let masked = shape.kind() == ShapeKind::Chamfered;
    let step = if masked { CROP_STEP_MASKED } else { CROP_STEP };
    let out = (
        (w / step).ceil().max(2.0) as usize,
        (h / step).ceil().max(2.0) as usize,
    );
    let mut hs = std::collections::hash_map::DefaultHasher::new();
    (
        key,
        [x, y, w, h, vw, vh].map(|v| v.round() as i32),
        layer.amount.to_bits(),
        layer.saturation.to_bits(),
        masked.then(|| (shape.radius.to_bits(), shape.outline(1.0, 1.0).len())),
    )
        .hash(&mut hs);
    let ck = hs.finish();
    let cached = locked().crops.get(ck);
    let image = match cached {
        Some(i) => i,
        None => {
            let outline = masked.then(|| shape.outline(w, h));
            let pixels = crop(
                &soft,
                (vw, vh),
                (x, y, w, h),
                out,
                Look {
                    saturation: layer.saturation,
                    amount: layer.amount,
                    mask: outline.as_deref(),
                },
            );
            let Some(i) = crate::backdrop::render_image(pixels, out.0, out.1) else {
                return;
            };
            let mut s = locked();
            let evicted = s.crops.insert(ck, i.clone());
            s.freed.extend(evicted);
            i
        }
    };
    let r = match shape.kind() {
        ShapeKind::Rounded => shape.radius.min(w.min(h) / 2.0).max(0.0),
        _ => 0.0,
    };
    let _ = window.paint_image(bounds, Corners::all(px(r)), image, 0, false);
}

/// The blurred backdrop and its key, starting the blur off the UI thread when it is missing.
fn soft_backdrop(cx: &mut App, layer: &FrostLayer, view_w: f32) -> Option<(u64, Soft)> {
    let mut s = locked();
    let wall = s.wall.as_ref().map(|w| (w.id, w.rgba.clone(), w.size));
    if let Some((id, rgba, size)) = wall {
        // Blur in picture pixels: the picture is stretched over the window.
        let sigma = (layer.blur * size.0 as f32 / view_w).max(0.5);
        let key = {
            let mut hs = std::collections::hash_map::DefaultHasher::new();
            (id, (sigma * 4.0).round() as i32).hash(&mut hs);
            hs.finish()
        };
        return match s.soft.get(key) {
            Some(Slot::Ready(soft)) => Some((key, soft)),
            Some(Slot::Pending) => None,
            None => {
                let _ = s.soft.insert(key, Slot::Pending);
                drop(s);
                let exec = cx.background_executor().clone();
                cx.spawn(async move |acx| {
                    let reduced = exec
                        .spawn(async move {
                            crate::frosted::blurrer()
                                .lock()
                                .map(|mut b| b.blurred(&rgba, size.0, size.1, sigma))
                                .unwrap_or_else(|e| {
                                    e.into_inner().blurred(&rgba, size.0, size.1, sigma)
                                })
                        })
                        .await;
                    let _ = locked().soft.insert(
                        key,
                        Slot::Ready(Soft {
                            rgba: Arc::new(reduced.pixels),
                            size: reduced.size,
                        }),
                    );
                    let _ = acx.update(|cx| cx.refresh_windows());
                })
                .detach();
                None
            }
        };
    }
    // The canvas mesh is tiny and smooth already: it is its own blur.
    let (canvas, wash, strength) = layer.mesh?;
    let key = {
        let mut hs = std::collections::hash_map::DefaultHasher::new();
        (canvas, wash, strength.to_bits()).hash(&mut hs);
        hs.finish()
    };
    if let Some(Slot::Ready(soft)) = s.soft.get(key) {
        return Some((key, soft));
    }
    let (w, h) = crate::backdrop::MESH_SIZE;
    let soft = Soft {
        rgba: Arc::new(crate::backdrop::pixels(canvas, wash, strength, w, h)),
        size: (w, h),
    };
    let _ = s.soft.insert(key, Slot::Ready(soft.clone()));
    Some((key, soft))
}

/// How a crop is finished: saturation, opacity and the shape it is cut to.
struct Look<'a> {
    saturation: f32,
    amount: f32,
    /// The component's outline in its own px, for shapes the GPU cannot round.
    mask: Option<&'a [(f32, f32)]>,
}

/// The part of `soft` (stretched over a `view` px window) under `rect` (window px), sampled at
/// `out` pixels, then saturated, faded to `look.amount` and cut to `look.mask`.
fn crop(
    soft: &Soft,
    view: (f32, f32),
    (x, y, w, h): (f32, f32, f32, f32),
    out: (usize, usize),
    look: Look,
) -> Vec<u8> {
    let (sw, sh) = soft.size;
    let (fx, fy) = (sw as f32 / view.0, sh as f32 / view.1);
    let at = |px: usize, py: usize| -> [f32; 4] {
        let o = (py * sw + px) * 4;
        let p = &soft.rgba[o..o + 4];
        [p[0], p[1], p[2], p[3]].map(f32::from)
    };
    let sample = |u: f32, v: f32| -> [f32; 4] {
        let u = u.clamp(0.0, (sw - 1) as f32);
        let v = v.clamp(0.0, (sh - 1) as f32);
        let (x0, y0) = (u.floor() as usize, v.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(sw - 1), (y0 + 1).min(sh - 1));
        let (tx, ty) = (u - x0 as f32, v - y0 as f32);
        let (a, b, c, d) = (at(x0, y0), at(x1, y0), at(x0, y1), at(x1, y1));
        std::array::from_fn(|i| {
            let top = a[i] + (b[i] - a[i]) * tx;
            let bottom = c[i] + (d[i] - c[i]) * tx;
            top + (bottom - top) * ty
        })
    };
    let (ow, oh) = out;
    let (cw, ch) = (w / ow as f32, h / oh as f32);
    let amount = look.amount.clamp(0.0, 1.0);
    let mut pixels = Vec::with_capacity(ow * oh * 4);
    for j in 0..oh {
        for i in 0..ow {
            let (lx, ly) = ((i as f32 + 0.5) * cw, (j as f32 + 0.5) * ch);
            let c = sample((x + lx) * fx - 0.5, (y + ly) * fy - 0.5);
            let cover = look.mask.map_or(1.0, |m| coverage(m, lx, ly, cw, ch));
            let alpha = c[3] * amount * cover;
            pixels.extend([c[0], c[1], c[2], alpha].map(|v| v.round().clamp(0.0, 255.0) as u8));
        }
    }
    dodai_frost::saturate(&mut pixels, look.saturation);
    pixels
}

/// Share of the cell `cw`×`ch` around `(x, y)` inside `poly` (2×2 samples).
fn coverage(poly: &[(f32, f32)], x: f32, y: f32, cw: f32, ch: f32) -> f32 {
    let offsets = [-0.25, 0.25];
    let inside = offsets
        .iter()
        .flat_map(|dx| offsets.iter().map(move |dy| (x + dx * cw, y + dy * ch)))
        .filter(|&(px, py)| contains(poly, px, py))
        .count();
    inside as f32 / 4.0
}

/// Even-odd point-in-polygon test.
fn contains(poly: &[(f32, f32)], x: f32, y: f32) -> bool {
    let mut inside = false;
    let mut j = poly.len().wrapping_sub(1);
    for (i, &(xi, yi)) in poly.iter().enumerate() {
        let (xj, yj) = poly[j];
        if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2×1 backdrop: red on the left, blue on the right.
    fn red_blue() -> Soft {
        Soft {
            rgba: Arc::new(vec![255, 0, 0, 255, 0, 0, 255, 255]),
            size: (2, 1),
        }
    }

    fn plain(amount: f32) -> Look<'static> {
        Look {
            saturation: 1.0,
            amount,
            mask: None,
        }
    }

    #[test]
    fn a_crop_shows_the_part_of_the_backdrop_under_it() {
        let soft = red_blue();
        // Window 200 px wide: the left quarter is red, the right quarter blue.
        let left = crop(
            &soft,
            (200.0, 10.0),
            (0.0, 0.0, 40.0, 10.0),
            (2, 2),
            plain(1.0),
        );
        let right = crop(
            &soft,
            (200.0, 10.0),
            (160.0, 0.0, 40.0, 10.0),
            (2, 2),
            plain(1.0),
        );
        assert!(left[0] > 240 && left[2] < 15, "left is red: {left:?}");
        assert!(right[2] > 240 && right[0] < 15, "right is blue: {right:?}");
    }

    #[test]
    fn amount_sets_the_opacity_and_a_mask_cuts_the_corners() {
        let soft = red_blue();
        let half = crop(
            &soft,
            (10.0, 10.0),
            (0.0, 0.0, 10.0, 10.0),
            (2, 2),
            plain(0.5),
        );
        assert!(half.chunks(4).all(|p| p[3] == 128));
        // A triangle over the top-left half: the bottom-right cell is outside.
        let tri = [(0.0, 0.0), (10.0, 0.0), (0.0, 10.0)];
        let cut = crop(
            &soft,
            (10.0, 10.0),
            (0.0, 0.0, 10.0, 10.0),
            (2, 2),
            Look {
                saturation: 1.0,
                amount: 1.0,
                mask: Some(&tri),
            },
        );
        assert_eq!(cut[3], 255, "top left inside");
        assert_eq!(cut[15], 0, "bottom right outside");
    }

    #[test]
    fn a_crop_off_the_window_clamps_to_the_edge() {
        let soft = red_blue();
        let off = crop(
            &soft,
            (10.0, 10.0),
            (-50.0, -50.0, 10.0, 10.0),
            (1, 1),
            plain(1.0),
        );
        assert_eq!(off.len(), 4);
        assert!(off[0] > 240, "the nearest edge pixel (red)");
    }
}
