//! Frosted glass on plain RGBA8 pixels (row-major, 4 bytes per pixel). No dependencies, no
//! toolkit, no theme: see the crate README.
//!
//! [`blur`], [`blurred`] and [`glass`] run on the CPU. With the `gpu` feature, a [`Blurrer`]
//! runs the same blur on the GPU (`wgpu`) whenever a real adapter exists, and on the CPU
//! otherwise. The GPU path is bit-identical to the CPU one.

#[cfg(feature = "gpu")]
mod gpu;
#[cfg(feature = "gpu")]
pub use gpu::GpuBlur;

/// Radius of each of the three box passes that approximate a Gaussian of `sigma`.
fn box_radius(sigma: f32) -> usize {
    let wi = (12.0 * sigma * sigma / 3.0 + 1.0).sqrt();
    (((wi - 1.0) / 2.0).round() as usize).max(1)
}

/// Gaussian-like blur (three box passes) of an RGBA image, edges clamped. `sigma` is in pixels.
pub fn blur(src: &[u8], w: usize, h: usize, sigma: f32) -> Vec<u8> {
    assert_eq!(src.len(), w * h * 4);
    if sigma <= 0.0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    let r = box_radius(sigma);
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

/// A blurred picture kept at reduced size (it is scaled back up when drawn or cropped).
#[derive(Debug, Clone, PartialEq)]
pub struct Reduced {
    /// RGBA pixels, `size.0 * size.1 * 4` bytes.
    pub pixels: Vec<u8>,
    /// Width and height of the reduced copy.
    pub size: (usize, usize),
    /// Width and height of the picture it was made from.
    pub full: (usize, usize),
}

/// Blur the picture on a copy reduced by `ceil(sigma / 6)`, which is much cheaper than blurring
/// at full size and indistinguishable once scaled back up.
pub fn blurred(src: &[u8], w: usize, h: usize, sigma: f32) -> Reduced {
    let f = (sigma / 6.0).ceil().max(1.0) as usize;
    let (small, sw, sh) = downscale(src, w, h, f);
    Reduced {
        pixels: blur(&small, sw, sh, sigma / f as f32),
        size: (sw, sh),
        full: (w, h),
    }
}

impl Reduced {
    /// The `w`×`h` rectangle at `(x, y)` of the full-size picture, sampled bilinearly so it lines
    /// up with the sharp picture.
    pub fn crop(&self, x: usize, y: usize, w: usize, h: usize) -> Vec<u8> {
        crop_bilinear(
            &self.pixels,
            self.size,
            (self.full.0 as f32, self.full.1 as f32),
            (x, y, w, h),
        )
    }
}

/// A frosted pane: blur radius, a translucent tint laid over the blur, and an optional edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glass {
    /// RGBA tint composited over the blurred backdrop (its alpha is the tint opacity).
    pub tint: [u8; 4],
    /// Blur sigma in pixels.
    pub blur: f32,
    /// Hard outline `(RGBA, width px)` drawn inside the pane's border.
    pub edge: Option<([u8; 4], f32)>,
    /// Colour saturation of the blurred backdrop before the tint (1 = unchanged, above 1 richer).
    pub saturation: f32,
}

/// Blurs on the GPU when it can, else on the CPU, reusing GPU resources between calls.
pub struct Blurrer {
    #[cfg(feature = "gpu")]
    gpu: Option<GpuBlur>,
}

impl Default for Blurrer {
    fn default() -> Self {
        Self::new()
    }
}

impl Blurrer {
    /// A blurrer on the GPU when a real adapter exists, else on the CPU. The GPU is opened
    /// here (once), not per call. Use the free functions ([`blur`], ...) for plain CPU calls.
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "gpu")]
            gpu: GpuBlur::new(),
        }
    }

    /// Whether blurs run on the GPU.
    pub fn uses_gpu(&self) -> bool {
        #[cfg(feature = "gpu")]
        if self.gpu.is_some() {
            return true;
        }
        false
    }

    /// [`blur`] on the chosen backend.
    pub fn blur(&mut self, src: &[u8], w: usize, h: usize, sigma: f32) -> Vec<u8> {
        #[cfg(feature = "gpu")]
        if self.uses_gpu() && sigma > 0.0 && w > 0 && h > 0 {
            if let Some(out) = self
                .gpu
                .as_mut()
                .and_then(|g| g.blur(src, w, h, box_radius(sigma)))
            {
                return out;
            }
        }
        blur(src, w, h, sigma)
    }

    /// [`blurred`] on the chosen backend (the reduced copy is what gets blurred).
    pub fn blurred(&mut self, src: &[u8], w: usize, h: usize, sigma: f32) -> Reduced {
        let f = (sigma / 6.0).ceil().max(1.0) as usize;
        let (small, sw, sh) = downscale(src, w, h, f);
        Reduced {
            pixels: self.blur(&small, sw, sh, sigma / f as f32),
            size: (sw, sh),
            full: (w, h),
        }
    }

    /// [`glass`] on the chosen backend.
    pub fn glass(&mut self, backdrop: &[u8], w: usize, h: usize, g: &Glass) -> Vec<u8> {
        let blurred = self.blur(backdrop, w, h, g.blur);
        tint_and_edge(blurred, w, h, g)
    }
}

/// Scale the colour saturation of RGBA pixels in place: 1 leaves them, 0 turns them grey,
/// above 1 makes them richer (frosted glass over a vivid backdrop looks more saturated than the
/// backdrop itself). Alpha is untouched. Pure.
pub fn saturate(px: &mut [u8], s: f32) {
    if (s - 1.0).abs() < 1e-3 || !s.is_finite() {
        return;
    }
    let s = s.max(0.0);
    for p in px.chunks_exact_mut(4) {
        let l = 0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2]);
        for c in &mut p[..3] {
            *c = (l + (f32::from(*c) - l) * s).round().clamp(0.0, 255.0) as u8;
        }
    }
}

/// The finished pane for a whole backdrop: blur, then tint, then edge. The result is opaque
/// where the backdrop is.
pub fn glass(backdrop: &[u8], w: usize, h: usize, g: &Glass) -> Vec<u8> {
    tint_and_edge(blur(backdrop, w, h, g.blur), w, h, g)
}

fn tint_and_edge(mut out: Vec<u8>, w: usize, h: usize, g: &Glass) -> Vec<u8> {
    saturate(&mut out, g.saturation);
    let a = g.tint[3] as u32;
    for px in out.chunks_exact_mut(4) {
        for (p, &t) in px.iter_mut().zip(&g.tint).take(3) {
            *p = ((t as u32 * a + *p as u32 * (255 - a) + 127) / 255) as u8;
        }
    }
    if let Some((col, width)) = g.edge {
        let t = width.ceil().max(1.0) as usize;
        for y in 0..h {
            for x in 0..w {
                if x < t || y < t || x + t >= w || y + t >= h {
                    out[(y * w + x) * 4..][..4].copy_from_slice(&col);
                }
            }
        }
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blur_keeps_flat_colour_and_spreads_an_edge() {
        let (w, h) = (16, 4);
        let flat = vec![200u8; w * h * 4];
        assert!(blur(&flat, w, h, 3.0).iter().all(|&v| v == 200));
        let mut edge = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in w / 2..w {
                edge[(y * w + x) * 4..][..4].copy_from_slice(&[255; 4]);
            }
        }
        let out = blur(&edge, w, h, 2.0);
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
        let r = blurred(&src, 64, 32, 12.0);
        assert_eq!((r.size, r.full), ((32, 16), (64, 32)));
        assert_eq!(r.crop(8, 4, 16, 8).len(), 16 * 8 * 4);
    }

    #[test]
    fn glass_tints_the_blur_and_draws_the_edge() {
        let src = vec![100u8; 8 * 8 * 4];
        let g = Glass {
            tint: [200, 200, 200, 255],
            blur: 2.0,
            edge: Some(([1, 2, 3, 255], 1.0)),
            saturation: 1.0,
        };
        let out = glass(&src, 8, 8, &g);
        assert_eq!(
            &out[(4 * 8 + 4) * 4..][..3],
            &[200, 200, 200],
            "opaque tint wins"
        );
        assert_eq!(&out[..4], &[1, 2, 3, 255], "edge on the border");
        let clear = Glass {
            tint: [0, 0, 0, 0],
            blur: 2.0,
            edge: None,
            saturation: 1.0,
        };
        assert_eq!(glass(&src, 8, 8, &clear), src, "clear tint leaves the blur");
    }

    #[test]
    fn saturate_scales_chroma_and_keeps_grey_and_alpha() {
        let mut px = [200u8, 100, 50, 77, 90, 90, 90, 255];
        saturate(&mut px, 1.5);
        assert!(px[0] > 200 && px[2] < 50, "richer: {px:?}");
        assert_eq!(px[3], 77);
        assert_eq!(&px[4..], &[90, 90, 90, 255], "grey stays grey");
        let mut g = [200u8, 100, 50, 255];
        saturate(&mut g, 0.0);
        assert!(g[0].abs_diff(g[1]) <= 1 && g[1].abs_diff(g[2]) <= 1);
        let before = g;
        saturate(&mut g, f32::NAN);
        assert_eq!(g, before);
    }

    #[test]
    fn blurrer_matches_the_free_functions() {
        let (w, h) = (300, 260);
        let src: Vec<u8> = (0..w * h * 4).map(|i| (i * 13 % 255) as u8).collect();
        let g = Glass {
            tint: [10, 20, 30, 120],
            blur: 6.0,
            edge: None,
            saturation: 1.0,
        };
        {
            let mut b = Blurrer::new();
            assert_eq!(b.blur(&src, w, h, 6.0), blur(&src, w, h, 6.0),);
            assert_eq!(b.blurred(&src, w, h, 12.0), blurred(&src, w, h, 12.0),);
            assert_eq!(b.glass(&src, w, h, &g), glass(&src, w, h, &g),);
        }
    }
}
