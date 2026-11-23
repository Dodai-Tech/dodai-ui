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
//! | charts | [`chart`](IcedTheme::chart), [`sparkline`](IcedTheme::sparkline) |
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

mod anchored;
mod button;
mod charts;
mod controls;
mod feedback;
mod frame;
mod geometry;
mod gizmo;
mod glass;
#[cfg(feature = "icons")]
mod icon;
mod inputs;
mod kit;
mod knob;
mod list;
mod panel;
mod range_slider;
mod scanlines;
mod shaped_button;
mod surface_fx;
mod tabs;
mod tag;
mod text;

#[cfg(test)]
mod testutil;

pub use anchored::{anchored, Anchored};
pub use charts::{ChartKind, Prim};
pub use frame::{shaped_frame, BoxStyle, ShapedFrame};
pub use geometry::ShapeGeom;
pub use gizmo::{GizmoState, GIZMO_HOME};
pub use kit::segment;
pub use knob::{wheel_value, KnobDrag, KnobRange};
pub use range_slider::{RangeDrag, RangeSpec, SliderLook};
pub use shaped_button::{ButtonState, ShapedButton, ShapedColors, Tick};
pub use surface_fx::SurfaceFxProgram;
pub use tabs::{tab_key, TabsLook};

use dodai_core::{Color as LColor, Theme};
use iced::{
    border::Border, theme::Palette, widget::container, Background, Color, Font, Shadow,
    Theme as IcedBaseTheme, Vector,
};

/// The first family of a CSS font stack (`"DM Sans, Inter, sans-serif"` → `DM Sans`), without
/// quotes; generic families (`sans-serif`, `monospace`, `system-ui`) are not names, so `""`.
pub fn first_family(stack: &'static str) -> &'static str {
    let first = stack
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches(['"', '\'']);
    match first {
        "sans-serif" | "serif" | "monospace" | "system-ui" | "cursive" | "fantasy" => "",
        f => f,
    }
}

/// Token colour → Iced colour.
pub fn color(c: LColor) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a as f32 / 255.0)
}

/// Iced view of a [`Theme`].
#[derive(Debug, Clone)]
pub struct IcedTheme {
    pub(crate) theme: Theme,
    pub(crate) glass: glass::GlassState,
    /// A busy shimmer swept over the buttons built from this: the band's left edge as a
    /// fraction of the width (see [`IcedTheme::with_shimmer`]).
    pub(crate) shimmer: Option<f32>,
}

impl IcedTheme {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            glass: glass::GlassState::default(),
            shimmer: None,
        }
        .with_mesh()
    }

    /// With the design's canvas mesh as the frost backdrop.
    #[cfg(feature = "frost")]
    fn with_mesh(mut self) -> Self {
        self.refresh_mesh();
        self
    }

    #[cfg(not(feature = "frost"))]
    fn with_mesh(self) -> Self {
        self
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        #[cfg(feature = "frost")]
        self.refresh_mesh();
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

    /// Body font: the first family of the design's body stack, from the installed fonts
    /// (iced falls back to its default when the family is missing).
    pub fn font(&self) -> Font {
        match first_family(self.theme.typography.body) {
            "" => Font::DEFAULT,
            name => Font::with_name(name),
        }
    }

    /// Display font (headings): the first family of the design's display stack.
    pub fn font_display(&self) -> Font {
        match first_family(self.theme.typography.display) {
            "" => self.font(),
            name => Font::with_name(name),
        }
    }

    /// Monospace font: the first family of the design's mono stack.
    pub fn font_mono(&self) -> Font {
        match first_family(self.theme.typography.mono) {
            "" => Font::MONOSPACE,
            name => Font::with_name(name),
        }
    }

    /// Border of `c` with the theme's width and (native) radius.
    pub(crate) fn border(&self, c: LColor) -> Border {
        let (c, width) = self
            .theme
            .outline(Some(c), false)
            .unwrap_or((c, self.theme.shape.border_width));
        Border {
            color: color(c),
            width,
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
        } else if let Some((dx, dy)) = self.theme.hard_offset().filter(|_| hard) {
            Shadow {
                color: color(self.theme.hard_shadow_color()),
                offset: Vector::new(dx, dy),
                blur_radius: 0.0,
            }
        } else {
            Shadow::default()
        }
    }

    /// Shadow of a lifted box (a raised button, a floating bar): the design's hard shadow, or
    /// a soft drop shadow on designs without one.
    pub(crate) fn lift_shadow(&self) -> Shadow {
        let e = &self.theme.effects;
        if e.hard_shadow > 0.0 {
            self.shadow(false, true)
        } else {
            Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.28),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            }
        }
    }

    /// The app root: the canvas colour, the design's mesh (with the `frost` feature) under
    /// `content`, and the scanlines over it. This is the default way to build a window's view,
    /// so a design with a gradient canvas (Dodai) always shows it.
    pub fn root<'a, Message: 'a>(
        &self,
        content: impl Into<iced::Element<'a, Message>>,
    ) -> iced::Element<'a, Message> {
        let canvas = container("")
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(self.background());
        let mut layers = iced::widget::stack![canvas];
        #[cfg(feature = "frost")]
        if let Some(b) = self.backdrop_view() {
            layers = layers.push(b);
        }
        let page = container(content)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);
        layers.push(page).push(self.scanlines()).into()
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
        let lt = IcedTheme::new(dodai_core::themes::cyberpunk::yellow());
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
    fn the_first_family_of_a_stack_is_the_font_name() {
        assert_eq!(first_family("DM Sans, Inter, sans-serif"), "DM Sans");
        assert_eq!(first_family("'Space Mono', Menlo"), "Space Mono");
        assert_eq!(first_family("Inter"), "Inter");
        assert_eq!(first_family("sans-serif, Inter"), "");
        assert_eq!(first_family(""), "");
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
            "button", "controls", "frame", "icon", "inputs", "kit", "list", "panel", "tabs", "tag",
            "text",
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
