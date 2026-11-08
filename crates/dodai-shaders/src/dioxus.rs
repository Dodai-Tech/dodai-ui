//! dioxus (feature `dioxus`): a shader as an `<img>` carrying a PNG data URL.
//!
//! The component redraws on a tokio timer, so it needs a tokio runtime (liveview and server
//! rendering have one). It reads its spec when it mounts: give it a `key` to change it.
//!
//! ```ignore
//! rsx! { ShaderView { spec: Spec::new("neon_grid").fx("crt"), width: 480.0 } }
//! ```

use crate::player::{Live, Poll, Spec};
use base64::Engine;
use dioxus::prelude::*;

/// PNG of a BGRA frame as a `data:` URL.
fn data_url(f: crate::player::Frame) -> Option<String> {
    let (w, h) = (f.width, f.height);
    let buf = image::RgbaImage::from_raw(w, h, f.rgba())?;
    let mut png = std::io::Cursor::new(Vec::new());
    buf.write_to(&mut png, image::ImageFormat::Png).ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png.into_inner())
    ))
}

/// Live shader, `width` logical px wide.
#[component]
pub fn ShaderView(spec: Spec, width: f32) -> Element {
    let mut src = use_signal(String::new);
    let aspect = spec.aspect;
    use_future(move || {
        let spec = spec.clone();
        async move {
            let mut live = Live::new(spec);
            loop {
                if let Poll::New(f) = live.tick(width, 1.0) {
                    if let Some(url) = data_url(f) {
                        src.set(url);
                    }
                }
                let wait = live
                    .next_in()
                    .unwrap_or(std::time::Duration::from_millis(250));
                tokio::time::sleep(wait.min(std::time::Duration::from_millis(250))).await;
            }
        }
    });
    rsx! {
        img {
            src: "{src}",
            style: "width:{width}px;aspect-ratio:{aspect};display:block;",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Frame;

    #[test]
    fn frames_become_png_data_urls() {
        let f = Frame {
            width: 2,
            height: 1,
            bgra: vec![0, 0, 255, 255, 255, 0, 0, 255],
        };
        let url = data_url(f).expect("png");
        let b64 = url.strip_prefix("data:image/png;base64,").expect("prefix");
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .expect("base64");
        assert_eq!(&bytes[1..4], b"PNG");
    }
}
