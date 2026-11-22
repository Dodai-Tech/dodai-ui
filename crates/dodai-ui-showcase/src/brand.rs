//! The app icon next to the "Dodai UI" title (sidebar and overview hero).

use std::sync::{Arc, OnceLock};

use gpui::{img, px, Image, ImageFormat, IntoElement, Styled};

/// The 128 px icon, generated from `assets/icons/dodai-ui.svg` by `just icons`.
const ICON_PNG: &[u8] = include_bytes!("../../../assets/icons/dodai-ui-128.png");

/// The icon as a gpui image, decoded once and shared by every frame.
fn icon() -> Arc<Image> {
    static ICON: OnceLock<Arc<Image>> = OnceLock::new();
    ICON.get_or_init(|| Arc::new(Image::from_bytes(ImageFormat::Png, ICON_PNG.to_vec())))
        .clone()
}

/// The app icon, `size` px square.
pub(crate) fn mark(size: f32) -> impl IntoElement {
    img(icon()).size(px(size)).flex_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_is_a_png() {
        assert!(ICON_PNG.starts_with(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn icon_is_built_once() {
        assert!(Arc::ptr_eq(&icon(), &icon()));
    }
}
