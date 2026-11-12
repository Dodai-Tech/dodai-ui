//! Pictures for the image demos, decoded once: RGBA pixels for toolkits that draw images
//! themselves, a `data:` URL for the web toolkits.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use base64::Engine as _;

/// A decoded picture.
#[derive(Debug, Clone, PartialEq)]
pub struct Picture {
    /// Width px.
    pub width: u32,
    /// Height px.
    pub height: u32,
    /// 4-byte RGBA pixels, row by row.
    pub rgba: Arc<Vec<u8>>,
}

/// Decodes PNG bytes to RGBA (`None` for anything that is not a readable PNG).
pub fn decode_png(bytes: &[u8]) -> Option<Picture> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    let px = &buf[..info.buffer_size()];
    let rgba: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => px.to_vec(),
        png::ColorType::Rgb => px
            .chunks_exact(3)
            .flat_map(|c| [c[0], c[1], c[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => px
            .chunks_exact(2)
            .flat_map(|c| [c[0], c[0], c[0], c[1]])
            .collect(),
        png::ColorType::Grayscale => px.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => return None,
    };
    Some(Picture {
        width: info.width,
        height: info.height,
        rgba: Arc::new(rgba),
    })
}

/// File bytes by path (`None` = unreadable), read once.
type FileCache = Mutex<HashMap<String, Option<Arc<Vec<u8>>>>>;

fn bytes(path: &str) -> Option<Arc<Vec<u8>>> {
    static CACHE: OnceLock<FileCache> = OnceLock::new();
    let mut c = CACHE.get_or_init(Default::default).lock().ok()?;
    c.entry(path.to_string())
        .or_insert_with(|| std::fs::read(path).ok().map(Arc::new))
        .clone()
}

/// The picture at `path`, decoded once.
pub fn picture(path: &str) -> Option<Picture> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<Picture>>>> = OnceLock::new();
    let mut c = CACHE.get_or_init(Default::default).lock().ok()?;
    c.entry(path.to_string())
        .or_insert_with(|| decode_png(&bytes(path)?))
        .clone()
}

/// The file at `path` as a `data:image/png;base64,…` URL.
pub fn data_url(path: &str) -> Option<String> {
    let b = bytes(path)?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(b.as_slice())
    ))
}

/// The Dodai UI logo (`assets/icons/dodai-ui.svg`) as a 256 px PNG.
pub const LOGO_PNG: &[u8] = include_bytes!("../../../assets/icons/dodai-ui-256.png");

/// The Dodai UI logo ([`LOGO_PNG`]), decoded once.
pub fn logo() -> Picture {
    static LOGO: OnceLock<Picture> = OnceLock::new();
    LOGO.get_or_init(|| decode_png(LOGO_PNG).expect("the logo is a PNG"))
        .clone()
}

/// The Dodai UI logo as a 128 px PNG: sharp up to 64 px on a 2× screen, and a third of the
/// bytes of [`LOGO_PNG`] for [`logo_data_url`], which web pages resend on every render.
pub const LOGO_PNG_128: &[u8] = include_bytes!("../../../assets/icons/dodai-ui-128.png");

/// The Dodai UI logo ([`LOGO_PNG_128`]) as a `data:image/png;base64,…` URL, made once.
pub fn logo_data_url() -> &'static str {
    static URL: OnceLock<String> = OnceLock::new();
    URL.get_or_init(|| {
        format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(LOGO_PNG_128)
        )
    })
}

/// Size of the frost demo's wallpaper, px.
pub const WALLPAPER_SIZE: (u32, u32) = (1200, 750);

/// The frost demo's wallpaper ([`dodai_core::glass_picture`]) in `theme`'s colours, made once
/// per theme.
pub fn wallpaper(theme: &dodai_core::Theme) -> Picture {
    static CACHE: OnceLock<Mutex<HashMap<&'static str, Picture>>> = OnceLock::new();
    let make = || {
        let (w, h) = WALLPAPER_SIZE;
        Picture {
            width: w,
            height: h,
            rgba: Arc::new(dodai_core::glass_picture(
                &theme.palette,
                w as usize,
                h as usize,
            )),
        }
    };
    match CACHE.get_or_init(Default::default).lock() {
        Ok(mut c) => c.entry(theme.id()).or_insert_with(make).clone(),
        Err(_) => make(),
    }
}

/// [`wallpaper`] as a `data:image/png;base64,…` URL, made once per theme.
pub fn wallpaper_url(theme: &dodai_core::Theme) -> String {
    static CACHE: OnceLock<Mutex<HashMap<&'static str, String>>> = OnceLock::new();
    let make = || {
        let p = wallpaper(theme);
        format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(encode_png(&p))
        )
    };
    match CACHE.get_or_init(Default::default).lock() {
        Ok(mut c) => c.entry(theme.id()).or_insert_with(make).clone(),
        Err(_) => make(),
    }
}

/// `p` as PNG bytes.
pub fn encode_png(p: &Picture) -> Vec<u8> {
    let mut out = Vec::new();
    let mut e = png::Encoder::new(&mut out, p.width, p.height);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    if let Ok(mut w) = e.write_header() {
        // Writing to a Vec cannot fail; a bad size would only leave the PNG empty.
        w.write_image_data(&p.rgba).ok();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = crate::sections::data::SAMPLE_IMAGE;

    #[test]
    fn the_wallpaper_round_trips_through_png() {
        let t = dodai_core::themes::all().remove(0);
        let p = wallpaper(&t);
        assert_eq!(decode_png(&encode_png(&p)).as_ref(), Some(&p));
        assert!(wallpaper_url(&t).starts_with("data:image/png;base64,"));
    }

    #[test]
    fn the_logo_is_a_square_256_px_picture_and_a_data_url() {
        let p = logo();
        assert_eq!((p.width, p.height), (256, 256));
        assert_eq!(p.rgba.len(), 256 * 256 * 4);
        assert!(logo_data_url().starts_with("data:image/png;base64,"));
    }

    #[test]
    fn the_overview_hero_shows_the_logo() {
        fn has_logo(n: &crate::Node) -> bool {
            match n {
                crate::Node::Logo(size) => *size > 0.0,
                crate::Node::Stack(s) => s.items.iter().any(has_logo),
                crate::Node::Framed { body, .. } => has_logo(body),
                _ => false,
            }
        }
        let t = dodai_core::themes::all().remove(0);
        assert!(has_logo(&crate::overview::overview(&t, "gpui")));
    }

    #[test]
    fn the_sample_decodes_to_rgba_of_its_size() {
        let p = picture(SAMPLE).expect("sample decodes");
        assert_eq!(p.rgba.len(), (p.width * p.height * 4) as usize);
        assert!(data_url(SAMPLE)
            .unwrap()
            .starts_with("data:image/png;base64,"));
    }

    #[test]
    fn missing_or_broken_files_give_none() {
        assert!(picture("/nonexistent/x.png").is_none());
        assert!(data_url("/nonexistent/x.png").is_none());
        assert!(decode_png(b"not a png").is_none());
    }
}

/// The pixels of a frosted scene: the sharp picture and, per panel, the blurred and tinted
/// copy of the picture behind it.
#[derive(Debug, Clone, PartialEq)]
pub struct FrostedScene {
    /// The picture.
    pub sharp: Picture,
    /// One pane per panel rectangle.
    pub panes: Vec<Picture>,
}

/// The picture of a `w`×`h` frosted scene in `theme`'s colours ([`dodai_core::frost_scene`])
/// and its panes for `rects` (`x, y, width, height` px), under the design's frost: blur,
/// saturation and tint. Made once per theme, size and panels.
pub fn frosted(theme: &dodai_core::Theme, w: u32, h: u32, rects: &[[f32; 4]]) -> FrostedScene {
    type Key = (&'static str, u32, u32, Vec<[u32; 4]>);
    static CACHE: OnceLock<Mutex<HashMap<Key, FrostedScene>>> = OnceLock::new();
    let key: Key = (
        theme.id(),
        w,
        h,
        rects.iter().map(|r| r.map(f32::to_bits)).collect(),
    );
    let make = || {
        let (uw, uh) = (w as usize, h as usize);
        let sharp = dodai_core::frost_scene(&theme.palette, uw, uh);
        let f = theme.frost();
        let soft = dodai_frost::blurred(&sharp, uw, uh, f.blur);
        let a = u32::from(f.tint.a);
        let panes = rects
            .iter()
            .map(|r| {
                let (x, y) = (r[0].max(0.0) as usize, r[1].max(0.0) as usize);
                let pw = (r[2] as usize).min(uw.saturating_sub(x)).max(1);
                let ph = (r[3] as usize).min(uh.saturating_sub(y)).max(1);
                let mut px = soft.crop(x, y, pw, ph);
                dodai_frost::saturate(&mut px, f.saturation);
                let tint = [f.tint.r, f.tint.g, f.tint.b];
                for p in px.chunks_exact_mut(4) {
                    for (c, t) in p.iter_mut().zip(tint) {
                        *c = ((u32::from(t) * a + u32::from(*c) * (255 - a) + 127) / 255) as u8;
                    }
                }
                Picture {
                    width: pw as u32,
                    height: ph as u32,
                    rgba: Arc::new(px),
                }
            })
            .collect();
        FrostedScene {
            sharp: Picture {
                width: w,
                height: h,
                rgba: Arc::new(sharp),
            },
            panes,
        }
    };
    match CACHE.get_or_init(Default::default).lock() {
        Ok(mut c) => c.entry(key).or_insert_with(make).clone(),
        Err(_) => make(),
    }
}

/// `p` as a `data:image/png;base64,…` URL, made once per picture (by its pixels' address, so
/// for pictures this module keeps alive, like [`frosted`]'s).
pub fn picture_url(p: &Picture) -> String {
    static CACHE: OnceLock<Mutex<HashMap<usize, String>>> = OnceLock::new();
    let make = || {
        format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(encode_png(p))
        )
    };
    match CACHE.get_or_init(Default::default).lock() {
        Ok(mut c) => c
            .entry(Arc::as_ptr(&p.rgba) as usize)
            .or_insert_with(make)
            .clone(),
        Err(_) => make(),
    }
}

#[cfg(test)]
mod frost_tests {
    use super::*;

    #[test]
    fn a_scene_has_one_pane_per_panel_of_the_panel_size() {
        let t = dodai_core::Theme::default();
        let s = frosted(
            &t,
            120,
            80,
            &[[10.0, 10.0, 40.0, 30.0], [100.0, 60.0, 50.0, 50.0]],
        );
        assert_eq!(s.sharp.rgba.len(), 120 * 80 * 4);
        assert_eq!((s.panes[0].width, s.panes[0].height), (40, 30));
        // Clipped to the picture.
        assert_eq!((s.panes[1].width, s.panes[1].height), (20, 20));
        assert_eq!(s.panes[1].rgba.len(), 20 * 20 * 4);
    }
}
