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

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = crate::sections::data::SAMPLE_IMAGE;

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
