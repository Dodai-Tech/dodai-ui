//! Icons: Material Symbols / Cyberpunk packs as SVG for freya's `SvgViewer`.

use dodai_ui_core::{Color, Theme};

/// SVG for `icon` in `theme`, painted `color` (default: the theme text colour).
pub fn icon_svg(
    theme: &Theme,
    icon: dodai_ui_icons::IconId,
    size: u32,
    filled: bool,
    color: Option<Color>,
) -> Option<String> {
    let g = dodai_ui_icons::glyph_for_theme(icon, theme, filled)?;
    Some(g.svg(size, &color.unwrap_or(theme.palette.text).to_hex()))
}

#[cfg(feature = "ui")]
pub use component::DodaiIcon;

#[cfg(feature = "ui")]
mod component {
    use super::icon_svg;
    use crate::theme::use_dodai_theme;
    use freya::prelude::*;

    /// Theme-aware icon (the pack follows the theme's shape).
    #[derive(PartialEq)]
    pub struct DodaiIcon {
        icon: dodai_ui_icons::IconId,
        size: u32,
        filled: bool,
        color: Option<dodai_ui_core::Color>,
    }

    impl DodaiIcon {
        /// 24 px outline icon in the theme text colour.
        pub fn new(icon: dodai_ui_icons::IconId) -> Self {
            Self {
                icon,
                size: 24,
                filled: false,
                color: None,
            }
        }
        /// Edge length in px.
        pub fn size(mut self, size: u32) -> Self {
            self.size = size;
            self
        }
        /// Use the filled glyph.
        pub fn filled(mut self, filled: bool) -> Self {
            self.filled = filled;
            self
        }
        /// Paint with `color` instead of the theme text colour.
        pub fn color(mut self, color: dodai_ui_core::Color) -> Self {
            self.color = Some(color);
            self
        }
    }

    impl Component for DodaiIcon {
        fn render(&self) -> impl IntoElement {
            let t = use_dodai_theme();
            let px = Size::px(self.size as f32);
            match icon_svg(&t, self.icon, self.size, self.filled, self.color) {
                Some(svg) => SvgViewer::new(Bytes::from(svg.into_bytes()))
                    .show_loader(false)
                    .width(px.clone())
                    .height(px)
                    .into_element(),
                None => rect().width(px.clone()).height(px).into_element(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_is_painted_with_the_given_or_theme_colour() {
        let t = dodai_ui_core::themes::material::light();
        use dodai_ui_icons::IconId;
        let s = icon_svg(&t, IconId::Home, 20, false, None).unwrap();
        assert!(s.contains(&t.palette.text.to_hex()) && s.contains("width=\"20\""));
        let red = icon_svg(&t, IconId::Home, 20, false, Some(Color::hex(0xff0000))).unwrap();
        assert!(red.contains("#ff0000"));
    }
}
