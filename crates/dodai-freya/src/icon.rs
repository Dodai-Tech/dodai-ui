//! Icons: Material Symbols / Cyberpunk pack as SVG for freya's `svg` element.

use dodai_core::{Color, Theme};

/// SVG for icon `name` in `theme`, painted `color` (default: the theme text
/// colour). `None` for unknown names.
pub fn icon_svg(
    theme: &Theme,
    name: &str,
    size: u32,
    filled: bool,
    color: Option<Color>,
) -> Option<String> {
    let g = dodai::icons::glyph(name, theme, filled)?;
    Some(g.svg(size, &color.unwrap_or(theme.palette.text).to_hex()))
}

#[cfg(feature = "ui")]
pub use component::*;

#[cfg(feature = "ui")]
mod component {
    use super::icon_svg;
    use crate::theme::use_dodai_theme;
    use freya::prelude::*;

    #[derive(Props, Clone, PartialEq)]
    pub struct IconProps {
        #[props(into)]
        pub name: String,
        #[props(default = 24)]
        pub size: u32,
        #[props(default)]
        pub filled: bool,
        #[props(default)]
        pub color: Option<dodai_core::Color>,
    }

    /// Theme-aware icon (rounded themes: Material Symbols; others: Cyberpunk).
    #[component]
    pub fn DodaiIcon(props: IconProps) -> Element {
        let t = use_dodai_theme();
        match icon_svg(&t, &props.name, props.size, props.filled, props.color) {
            Some(svg) => rsx! {
                svg {
                    width: "{props.size}",
                    height: "{props.size}",
                    svg_content: "{svg}",
                }
            },
            None => rsx! { rect { width: "{props.size}", height: "{props.size}" } },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_is_painted_with_the_given_or_theme_colour() {
        let t = dodai::themes::material::light();
        let s = icon_svg(&t, "home", 20, false, None).unwrap();
        assert!(s.contains(&t.palette.text.to_hex()) && s.contains("width=\"20\""));
        let red = icon_svg(&t, "home", 20, false, Some(Color::hex(0xff0000))).unwrap();
        assert!(red.contains("#ff0000"));
        assert!(icon_svg(&t, "nope_nope", 20, false, None).is_none());
    }
}
