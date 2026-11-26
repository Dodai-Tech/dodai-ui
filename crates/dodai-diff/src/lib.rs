//! Compares two renderings of the same component (a toolkit capture and the design file's export)
//! and says how far apart they are, as numbers, so no one has to look at the pictures.
//!
//! Two measures, because a font or an anti-aliasing difference moves many pixels a little but
//! moves no layout:
//! - `pixels`: share of pixels whose colour differs by more than `tolerance` in any channel;
//! - `blocks`: share of `BLOCK`-pixel cells whose mean colour differs by more than `tolerance`
//!   (layout, colour and size of shapes; blind to glyph detail).
//!
//! Images are laid on white (transparency is flattened) and padded to the larger size; padding
//! counts as a difference, so a frame of the wrong size fails.

use std::io::BufWriter;
use std::path::Path;

/// Side of a comparison cell in pixels.
pub const BLOCK: usize = 8;

/// A decoded picture: RGB, flattened on white.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    /// Width in pixels.
    pub width: usize,
    /// Height in pixels.
    pub height: usize,
    /// `width * height` RGB triples.
    pub rgb: Vec<[u8; 3]>,
}

/// What can go wrong reading a picture.
#[derive(Debug)]
pub enum Error {
    /// The file could not be opened or decoded.
    Read(String),
    /// The file could not be written.
    Write(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Read(m) | Error::Write(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for Error {}

impl Picture {
    /// A picture of one colour.
    pub fn solid(width: usize, height: usize, rgb: [u8; 3]) -> Self {
        Self {
            width,
            height,
            rgb: vec![rgb; width * height],
        }
    }

    /// Decodes a PNG (any colour type of 8 bits) and flattens it on white.
    pub fn open(path: &Path) -> Result<Self, Error> {
        Self::open_on(path, [255, 255, 255])
    }

    /// Like [`Picture::open`], with transparency flattened on `bg`.
    pub fn open_on(path: &Path, bg: [u8; 3]) -> Result<Self, Error> {
        let file = std::fs::File::open(path)
            .map_err(|e| Error::Read(format!("{}: {e}", path.display())))?;
        let mut dec = png::Decoder::new(std::io::BufReader::new(file));
        dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = dec
            .read_info()
            .map_err(|e| Error::Read(format!("{}: {e}", path.display())))?;
        let mut buf = vec![
            0;
            reader
                .output_buffer_size()
                .ok_or_else(|| Error::Read("too large".into()))?
        ];
        let info = reader
            .next_frame(&mut buf)
            .map_err(|e| Error::Read(format!("{}: {e}", path.display())))?;
        let n = info.color_type.samples();
        let flat =
            |c: u8, a: u8, b: u8| ((c as u32 * a as u32 + b as u32 * (255 - a as u32)) / 255) as u8;
        let rgb = buf[..info.buffer_size()]
            .chunks_exact(n)
            .map(|p| match n {
                1 => [p[0]; 3],
                2 => [flat(p[0], p[1], bg[0]); 3],
                3 => [p[0], p[1], p[2]],
                _ => [
                    flat(p[0], p[3], bg[0]),
                    flat(p[1], p[3], bg[1]),
                    flat(p[2], p[3], bg[2]),
                ],
            })
            .collect();
        Ok(Self {
            width: info.width as usize,
            height: info.height as usize,
            rgb,
        })
    }

    /// The colour at `(x, y)`, if inside.
    pub fn at(&self, x: usize, y: usize) -> Option<[u8; 3]> {
        (x < self.width && y < self.height).then(|| self.rgb[y * self.width + x])
    }

    /// The `w` x `h` part at `(x, y)`; what lies outside the picture is white.
    pub fn crop(&self, x: usize, y: usize, w: usize, h: usize) -> Self {
        let mut out = Self::solid(w, h, [255, 255, 255]);
        for j in 0..h {
            for i in 0..w {
                if let Some(c) = self.at(x + i, y + j) {
                    out.rgb[j * w + i] = c;
                }
            }
        }
        out
    }

    /// Writes this picture as a PNG.
    pub fn save(&self, path: &Path) -> Result<(), Error> {
        let werr = |e: &dyn std::fmt::Display| Error::Write(format!("{}: {e}", path.display()));
        let file = std::fs::File::create(path).map_err(|e| werr(&e))?;
        let mut enc =
            png::Encoder::new(BufWriter::new(file), self.width as u32, self.height as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().map_err(|e| werr(&e))?;
        let bytes: Vec<u8> = self.rgb.iter().flatten().copied().collect();
        w.write_image_data(&bytes).map_err(|e| werr(&e))
    }
}

/// How the two pictures are compared.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// Largest per-channel difference (0..=255) that still counts as the same colour.
    pub tolerance: u8,
}

impl Default for Options {
    fn default() -> Self {
        Self { tolerance: 24 }
    }
}

/// The result of a comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// Size of the first picture.
    pub a: (usize, usize),
    /// Size of the second picture.
    pub b: (usize, usize),
    /// Percent of pixels that differ.
    pub pixels: f32,
    /// Percent of cells whose mean colour differs.
    pub blocks: f32,
}

impl Report {
    /// Whether the two pictures have the same size.
    pub fn same_size(&self) -> bool {
        self.a == self.b
    }

    /// One JSON object (no dependency needed for flat numbers).
    pub fn json(&self, name: &str) -> String {
        format!(
            "{{\"name\":\"{}\",\"a\":[{},{}],\"b\":[{},{}],\"pixels\":{:.2},\"blocks\":{:.2}}}",
            name.replace('"', "'"),
            self.a.0,
            self.a.1,
            self.b.0,
            self.b.1,
            self.pixels,
            self.blocks
        )
    }
}

fn far(p: [u8; 3], q: [u8; 3], tol: u8) -> bool {
    p.iter().zip(q).any(|(a, b)| a.abs_diff(b) > tol)
}

/// Compares `a` with `b`. The second value is a picture of the larger size: matching pixels are
/// faded, differing ones red, padding blue.
pub fn compare(a: &Picture, b: &Picture, opt: Options) -> (Report, Picture) {
    let (w, h) = (a.width.max(b.width), a.height.max(b.height));
    let mut out = Picture::solid(w, h, [255, 255, 255]);
    let mut bad = 0usize;
    for y in 0..h {
        for x in 0..w {
            let px = match (a.at(x, y), b.at(x, y)) {
                (Some(p), Some(q)) if !far(p, q, opt.tolerance) => {
                    let g = ((p[0] as u32 + p[1] as u32 + p[2] as u32) / 3) as u8;
                    let f = 200 + g / 5;
                    [f, f, f]
                }
                (Some(_), Some(_)) => {
                    bad += 1;
                    [230, 30, 30]
                }
                _ => {
                    bad += 1;
                    [60, 90, 230]
                }
            };
            out.rgb[y * w + x] = px;
        }
    }
    let mean = |p: &Picture, bx: usize, by: usize| -> Option<[u8; 3]> {
        let mut s = [0u32; 3];
        let mut n = 0u32;
        for y in by * BLOCK..((by + 1) * BLOCK).min(h) {
            for x in bx * BLOCK..((bx + 1) * BLOCK).min(w) {
                let c = p.at(x, y)?;
                for i in 0..3 {
                    s[i] += c[i] as u32;
                }
                n += 1;
            }
        }
        (n > 0).then(|| [(s[0] / n) as u8, (s[1] / n) as u8, (s[2] / n) as u8])
    };
    let (cx, cy) = (w.div_ceil(BLOCK), h.div_ceil(BLOCK));
    let mut cells_bad = 0usize;
    for by in 0..cy {
        for bx in 0..cx {
            match (mean(a, bx, by), mean(b, bx, by)) {
                (Some(p), Some(q)) if !far(p, q, opt.tolerance) => {}
                _ => cells_bad += 1,
            }
        }
    }
    let pct = |n: usize, d: usize| {
        if d == 0 {
            0.0
        } else {
            n as f32 * 100.0 / d as f32
        }
    };
    (
        Report {
            a: (a.width, a.height),
            b: (b.width, b.height),
            pixels: pct(bad, w * h),
            blocks: pct(cells_bad, cx * cy),
        },
        out,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_pictures_do_not_differ() {
        let a = Picture::solid(20, 20, [10, 20, 30]);
        let (r, _) = compare(&a, &a.clone(), Options::default());
        assert_eq!((r.pixels, r.blocks), (0.0, 0.0));
        assert!(r.same_size());
    }

    #[test]
    fn a_small_colour_drift_is_inside_the_tolerance() {
        let a = Picture::solid(16, 16, [100, 100, 100]);
        let b = Picture::solid(16, 16, [110, 100, 90]);
        assert_eq!(compare(&a, &b, Options::default()).0.pixels, 0.0);
        assert_eq!(compare(&a, &b, Options { tolerance: 5 }).0.pixels, 100.0);
    }

    #[test]
    fn a_moved_block_shows_in_both_measures() {
        let a = Picture::solid(32, 32, [255, 255, 255]);
        let mut b = a.clone();
        for y in 0..16 {
            for x in 0..16 {
                b.rgb[y * 32 + x] = [0, 0, 0];
            }
        }
        let (r, d) = compare(&a, &b, Options::default());
        assert_eq!(r.pixels, 25.0);
        assert_eq!(r.blocks, 25.0);
        assert_eq!(d.rgb[0], [230, 30, 30]);
    }

    #[test]
    fn crop_takes_the_part_and_pads_with_white() {
        let mut a = Picture::solid(4, 4, [0, 0, 0]);
        a.rgb[5] = [9, 9, 9];
        let c = a.crop(1, 1, 5, 2);
        assert_eq!(c.rgb[0], [9, 9, 9]);
        assert_eq!(c.rgb[4], [255, 255, 255]);
        assert_eq!((c.width, c.height), (5, 2));
    }

    #[test]
    fn the_wrong_size_counts_as_a_difference() {
        let a = Picture::solid(16, 16, [0, 0, 0]);
        let b = Picture::solid(16, 32, [0, 0, 0]);
        let (r, _) = compare(&a, &b, Options::default());
        assert!(!r.same_size());
        assert_eq!(r.pixels, 50.0);
    }

    #[test]
    fn save_then_open_round_trips() {
        let dir = std::env::temp_dir().join(format!("dodai-diff-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("a.png");
        let a = Picture::solid(9, 5, [1, 2, 3]);
        a.save(&p).unwrap();
        assert_eq!(Picture::open(&p).unwrap(), a);
    }
}
