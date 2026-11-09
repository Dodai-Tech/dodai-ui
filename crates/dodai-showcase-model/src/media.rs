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
