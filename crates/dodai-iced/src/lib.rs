//! # dodai-iced
//!
//! A complete, stylable component library for [iced] 0.14 driven **only** by
//! [`dodai_core::Theme`] tokens. There is no branching on theme id/name:
//! shape ([`ShapeKind`](dodai_core::ShapeKind)), glow, hard shadow,
//! typography and spacing all come from tokens, so any theme added to
//! `dodai-core` (rounded, square, light, no-glow…) just works.
//!
//! Chamfered shapes cannot be expressed by iced's `border.radius`, so those
//! use small canvas widgets ([`ShapedButton`], [`ShapedFrame`]); rounded and
//! square shapes use native widgets with style functions.
//!
//! | Area | API on [`IcedTheme`] |
//! |------|----------------------|
//! | buttons | [`button_view`](IcedTheme::button_view), [`shaped_button`](IcedTheme::shaped_button), [`button`](IcedTheme::button) |
//! | panel / card | [`panel_view`](IcedTheme::panel_view), [`panel`](IcedTheme::panel), [`shaped_box`](IcedTheme::shaped_box) |
//! | tag / badge | [`tag`](IcedTheme::tag) |
//! | tabs | [`tabs`](IcedTheme::tabs), [`tab_style`](IcedTheme::tab_style) |
//! | toggler / checkbox / radio | [`toggler`](IcedTheme::toggler), [`checkbox`](IcedTheme::checkbox), [`radio`](IcedTheme::radio) |
//! | progress / slider | [`progress`](IcedTheme::progress), [`slider`](IcedTheme::slider) |
//! | inputs | [`text_input`](IcedTheme::text_input), [`pick_list`](IcedTheme::pick_list), [`menu`](IcedTheme::menu) |
//! | lists | [`list_item`](IcedTheme::list_item), [`list_item_style`](IcedTheme::list_item_style) |
//! | misc | [`divider`](IcedTheme::divider), [`scrollable`](IcedTheme::scrollable), [`tooltip_style`](IcedTheme::tooltip_style), [`tooltip`](IcedTheme::tooltip) |
//! | text | [`heading_text`](IcedTheme::heading_text), [`label_text`](IcedTheme::label_text), [`body_text`](IcedTheme::body_text), [`muted_text`](IcedTheme::muted_text), [`mono_text`](IcedTheme::mono_text) |
//! | effects | [`scanlines`](IcedTheme::scanlines) |
//!
//! ```no_run
//! use iced::widget::{button, text};
//! use dodai_core::{ButtonVariant, Theme};
//! use dodai_iced::IcedTheme;
//!
//! let lt = IcedTheme::new(Theme::default());
//! # #[derive(Clone)] enum Msg { Go }
//! let _btn: iced::Element<Msg> = lt.button_view("Continue", ButtonVariant::Primary, Some(Msg::Go));
//! let _native: iced::Element<Msg> = button(text("Native"))
//!     .style(lt.button(ButtonVariant::Outline))
//!     .on_press(Msg::Go)
//!     .into();
//! // app.theme() => lt.iced_theme()
//! ```
//!
//! Note: iced has no letter-spacing support, so `typography.label_tracking`
//! is not applied; fonts stay on iced defaults (family names in tokens are not
//! loaded).
//!
//! [iced]: https://iced.rs

mod button;
mod controls;
mod frame;
mod geometry;
mod inputs;
mod list;
mod panel;
mod scanlines;
mod shaped_button;
mod tabs;
mod tag;
mod text;

#[cfg(test)]
mod testutil;

pub use frame::{shaped_frame, BoxStyle, ShapedFrame};
pub use geometry::ShapeGeom;
pub use shaped_button::{ButtonState, ShapedButton, ShapedColors};

use dodai_core::{Color as LColor, Theme};
use iced::{
    border::Border, theme::Palette, widget::container, Background, Color, Font, Shadow,
    Theme as IcedBaseTheme, Vector,
};

/// Token colour → Iced colour.
pub fn color(c: LColor) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a as f32 / 255.0)
}

/// Iced view of a [`Theme`].
#[derive(Debug, Clone)]
pub struct IcedTheme {
    pub(crate) theme: Theme,
}

impl IcedTheme {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    /// Native Iced theme so built-in widgets inherit the palette.
    pub fn iced_theme(&self) -> IcedBaseTheme {
        let p = &self.theme.palette;
        IcedBaseTheme::custom(
            self.theme.name().to_string(),
            Palette {
                background: color(p.background),
                text: color(p.text),
                primary: color(p.primary),
                success: color(p.success),
                warning: color(p.warning),
                danger: color(p.danger),
            },
        )
    }

    /// Body font (iced default; token family names are not loaded).
    pub fn font(&self) -> Font {
        Font::DEFAULT
    }

    /// Monospace font.
    pub fn font_mono(&self) -> Font {
        Font::MONOSPACE
    }

    /// Border of `c` with the theme's width and (native) radius.
    pub(crate) fn border(&self, c: LColor) -> Border {
        Border {
            color: color(c),
            width: self.theme.shape.border_width,
            radius: self.radius().into(),
        }
    }

    /// Native shadow: glow if `glow` and enabled, else hard shadow if `hard`
    /// and `effects.hard_shadow > 0`, else none.
    pub(crate) fn shadow(&self, glow: bool, hard: bool) -> Shadow {
        let e = &self.theme.effects;
        if glow && e.glow_enabled() {
            Shadow {
                color: color(e.glow_color()),
                offset: Vector::ZERO,
                blur_radius: e.glow_radius(),
            }
        } else if hard && e.hard_shadow > 0.0 {
            Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: Vector::new(e.hard_shadow, e.hard_shadow),
                blur_radius: 0.0,
            }
        } else {
            Shadow::default()
        }
    }

    /// App background container.
    pub fn background(&self) -> impl Fn(&IcedBaseTheme) -> container::Style + 'static {
        let p = &self.theme.palette;
        let (bg, fg) = (color(p.background), color(p.text));
        move |_| container::Style {
            background: Some(Background::Color(bg)),
            text_color: Some(fg),
            ..container::Style::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn color_converts() {
        let c = color(LColor::hex(0xfcee0a));
        assert_eq!(c, Color::from_rgb8(0xfc, 0xee, 0x0a));
    }

    #[test]
    fn iced_theme_uses_primary() {
        let lt = IcedTheme::new(dodai::themes::cyberpunk::yellow());
        assert_eq!(
            lt.iced_theme().palette().primary,
            color(LColor::hex(0xfcee0a))
        );
        let light = IcedTheme::new(rounded_light());
        assert_eq!(
            light.iced_theme().palette().primary,
            color(LColor::hex(0x1f5fd6))
        );
    }

    #[test]
    fn radius_per_shape_kind() {
        assert_eq!(IcedTheme::new(cyberpunk()).radius(), 0.0);
        assert_eq!(IcedTheme::new(rounded_light()).radius(), 8.0);
        assert_eq!(IcedTheme::new(hard_square()).radius(), 0.0);
    }

    #[test]
    fn no_theme_id_branching() {
        for f in [
            "button", "controls", "frame", "inputs", "list", "panel", "tabs", "tag", "text",
        ] {
            let src = std::fs::read_to_string(format!("{}/src/{f}.rs", env!("CARGO_MANIFEST_DIR")))
                .unwrap();
            // Strip the test module so asserting on ids there is allowed.
            let body = src.split("#[cfg(test)]").next().unwrap();
            assert!(!body.contains(".id()") && !body.contains("meta.id"), "{f}");
            assert!(!body.contains(".name()"), "{f}");
        }
    }
}
