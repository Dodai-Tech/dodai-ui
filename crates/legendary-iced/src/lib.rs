//! # legendary-iced
//!
//! Turns a [`legendary_core::Theme`] into Iced styling: a native
//! [`iced::Theme`], style closures for buttons / containers / text inputs /
//! text, a glowing hard-edged look, and a canvas [`ChamferFrame`] for
//! cut-corner panels that plain `border.radius` cannot express.
//!
//! ```no_run
//! use iced::widget::{button, text};
//! use legendary_core::{ButtonVariant, Theme};
//! use legendary_iced::IcedTheme;
//!
//! let lt = IcedTheme::new(Theme::default());
//! # #[derive(Clone)] enum Msg { Go }
//! let _btn: iced::Element<Msg> = button(text("JACK IN"))
//!     .style(lt.button(ButtonVariant::Primary))
//!     .on_press(Msg::Go)
//!     .into();
//! // app.theme() => lt.iced_theme()
//! ```

mod chamfer;

pub use chamfer::{chamfer_frame, chamfer_points, ChamferFrame};

use iced::{
    border::Border,
    theme::Palette,
    widget::{button, container, text, text_input},
    Background, Color, Font, Shadow, Theme as IcedBaseTheme, Vector,
};
use legendary_core::{ButtonTokens, ButtonVariant, Color as LColor, ControlState, Theme, Tone};

/// Token colour → Iced colour.
pub fn color(c: LColor) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a as f32 / 255.0)
}

/// Iced view of a [`Theme`].
#[derive(Debug, Clone)]
pub struct IcedTheme {
    theme: Theme,
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

    /// Body font (first family in the token list; falls back to the default font).
    pub fn font(&self) -> Font {
        Font::DEFAULT
    }

    /// Monospace font.
    pub fn font_mono(&self) -> Font {
        Font::MONOSPACE
    }

    fn border(&self, c: LColor) -> Border {
        Border {
            color: color(c),
            width: self.theme.shape.border_width,
            radius: self.theme.shape.radius.into(),
        }
    }

    fn glow(&self, on: bool) -> Shadow {
        let e = &self.theme.effects;
        if on && e.glow_blur > 0.0 {
            Shadow {
                color: color(e.glow),
                offset: Vector::ZERO,
                blur_radius: e.glow_blur,
            }
        } else {
            Shadow::default()
        }
    }

    /// Button style closure for `.style(...)`.
    pub fn button(
        &self,
        variant: ButtonVariant,
    ) -> impl Fn(&IcedBaseTheme, button::Status) -> button::Style + 'static {
        let tok = |s| self.theme.button(variant, s);
        let (idle, hover, press, off) = (
            tok(ControlState::Idle),
            tok(ControlState::Hovered),
            tok(ControlState::Pressed),
            tok(ControlState::Disabled),
        );
        let this = self.clone();
        move |_t, status| {
            let (t, lit): (ButtonTokens, bool) = match status {
                button::Status::Active => (idle, false),
                button::Status::Hovered => (hover, true),
                button::Status::Pressed => (press, true),
                button::Status::Disabled => (off, false),
            };
            button::Style {
                background: t.background.map(|c| Background::Color(color(c))),
                text_color: color(t.foreground),
                border: match t.border {
                    Some(c) => this.border(c),
                    None => Border {
                        radius: this.theme.shape.radius.into(),
                        ..Border::default()
                    },
                },
                shadow: this.glow(lit),
                snap: true,
            }
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

    /// Bordered panel / card.
    pub fn panel(&self, focused: bool) -> impl Fn(&IcedBaseTheme) -> container::Style + 'static {
        let t = self.theme.panel(focused);
        let fg = color(self.theme.palette.text);
        let border = self.border(t.border);
        let shadow = self.glow(focused);
        move |_| container::Style {
            background: Some(Background::Color(color(t.background))),
            text_color: Some(fg),
            border,
            shadow,
            snap: true,
        }
    }

    /// Text input style.
    pub fn text_input(
        &self,
    ) -> impl Fn(&IcedBaseTheme, text_input::Status) -> text_input::Style + 'static {
        let idle = self.theme.input(false);
        let focus = self.theme.input(true);
        let off = self.theme.palette.text_disabled;
        let this = self.clone();
        move |_t, status| {
            let (t, disabled) = match status {
                text_input::Status::Focused { .. } => (focus, false),
                text_input::Status::Disabled => (idle, true),
                _ => (idle, false),
            };
            text_input::Style {
                background: Background::Color(color(t.background)),
                border: this.border(t.border),
                icon: color(t.placeholder),
                placeholder: color(t.placeholder),
                value: color(if disabled { off } else { t.text }),
                selection: color(t.selection),
            }
        }
    }

    /// Coloured status text.
    pub fn tone(&self, tone: Tone) -> impl Fn(&IcedBaseTheme) -> text::Style + 'static {
        let c = color(self.theme.palette.tone(tone));
        move |_| text::Style { color: Some(c) }
    }

    /// Muted secondary text.
    pub fn muted(&self) -> impl Fn(&IcedBaseTheme) -> text::Style + 'static {
        let c = color(self.theme.palette.text_muted);
        move |_| text::Style { color: Some(c) }
    }

    /// Brand-coloured heading text.
    pub fn heading(&self) -> impl Fn(&IcedBaseTheme) -> text::Style + 'static {
        let c = color(self.theme.palette.primary);
        move |_| text::Style { color: Some(c) }
    }

    /// Apply the theme's label casing (UPPERCASE for cyberpunk).
    pub fn label(&self, s: &str) -> String {
        if self.theme.typography.uppercase_labels {
            s.to_uppercase()
        } else {
            s.to_string()
        }
    }

    /// A [`ChamferFrame`] filled/stroked from this theme's panel tokens.
    pub fn chamfer(&self, focused: bool) -> ChamferFrame {
        let t = self.theme.panel(focused);
        ChamferFrame {
            fill: color(t.background),
            stroke: color(t.border),
            width: self.theme.shape.border_width,
            cut: self.theme.shape.chamfer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_converts() {
        let c = color(LColor::hex(0xfcee0a));
        assert_eq!(c, Color::from_rgb8(0xfc, 0xee, 0x0a));
    }

    #[test]
    fn primary_button_is_solid_yellow() {
        let lt = IcedTheme::new(Theme::default());
        let f = lt.button(ButtonVariant::Primary);
        let s = f(&lt.iced_theme(), button::Status::Active);
        assert_eq!(
            s.background,
            Some(Background::Color(color(LColor::hex(0xfcee0a))))
        );
        assert_eq!(s.shadow, Shadow::default());
        let h = f(&lt.iced_theme(), button::Status::Hovered);
        assert!(h.shadow.blur_radius > 0.0);
    }

    #[test]
    fn outline_fills_on_hover() {
        let lt = IcedTheme::new(Theme::default());
        let f = lt.button(ButtonVariant::Outline);
        assert!(f(&lt.iced_theme(), button::Status::Active)
            .background
            .is_none());
        assert!(f(&lt.iced_theme(), button::Status::Hovered)
            .background
            .is_some());
    }

    #[test]
    fn iced_theme_uses_primary() {
        let lt = IcedTheme::new(Theme::default());
        assert_eq!(
            lt.iced_theme().palette().primary,
            color(LColor::hex(0xfcee0a))
        );
    }

    #[test]
    fn label_uppercases() {
        assert_eq!(IcedTheme::new(Theme::default()).label("ok"), "OK");
    }
}
