//! Toggler, checkbox, radio, progress bar and slider styles.

use iced::{
    border::Radius,
    widget::{checkbox, progress_bar, radio, slider, toggler},
    Background, Border, Theme as IcedBaseTheme,
};
use dodai_core::{ControlState, ShapeKind, Tone};

use crate::{color, IcedTheme};

impl IcedTheme {
    /// Toggler / switch style (`theme.toggle` tokens; corner radius from shape).
    pub fn toggler(&self) -> impl Fn(&IcedBaseTheme, toggler::Status) -> toggler::Style + 'static {
        let this = self.clone();
        move |_t, status| {
            let (on, st) = match status {
                toggler::Status::Active { is_toggled } => (is_toggled, ControlState::Idle),
                toggler::Status::Hovered { is_toggled } => (is_toggled, ControlState::Hovered),
                toggler::Status::Disabled { is_toggled } => (is_toggled, ControlState::Disabled),
            };
            let t = this.theme.toggle(on, st);
            toggler::Style {
                background: Background::Color(color(t.track)),
                background_border_width: this.theme.shape.border_width,
                background_border_color: color(t.border),
                foreground: Background::Color(color(t.thumb)),
                foreground_border_width: 0.0,
                foreground_border_color: iced::Color::TRANSPARENT,
                text_color: Some(color(if st == ControlState::Disabled {
                    this.theme.palette.text_disabled
                } else {
                    this.theme.palette.text
                })),
                border_radius: Some(Radius::from(this.radius())),
                padding_ratio: 0.2,
            }
        }
    }

    /// Checkbox style.
    pub fn checkbox(
        &self,
    ) -> impl Fn(&IcedBaseTheme, checkbox::Status) -> checkbox::Style + 'static {
        let this = self.clone();
        move |_t, status| {
            let (on, st) = match status {
                checkbox::Status::Active { is_checked } => (is_checked, ControlState::Idle),
                checkbox::Status::Hovered { is_checked } => (is_checked, ControlState::Hovered),
                checkbox::Status::Disabled { is_checked } => (is_checked, ControlState::Disabled),
            };
            let t = this.theme.toggle(on, st);
            checkbox::Style {
                background: Background::Color(color(t.track)),
                icon_color: color(t.thumb),
                border: this.border(t.border),
                text_color: Some(color(if st == ControlState::Disabled {
                    this.theme.palette.text_disabled
                } else {
                    this.theme.palette.text
                })),
            }
        }
    }

    /// Radio style (round for rounded themes, square otherwise: radios follow
    /// the shape token, so a chamfered/square theme gets square radios).
    pub fn radio(&self) -> impl Fn(&IcedBaseTheme, radio::Status) -> radio::Style + 'static {
        let this = self.clone();
        move |_t, status| {
            let (sel, st) = match status {
                radio::Status::Active { is_selected } => (is_selected, ControlState::Idle),
                radio::Status::Hovered { is_selected } => (is_selected, ControlState::Hovered),
            };
            let t = this.theme.toggle(sel, st);
            let off = this.theme.toggle(false, st);
            radio::Style {
                background: Background::Color(color(off.track)),
                dot_color: color(this.theme.toggle(true, ControlState::Idle).track),
                border_width: this.theme.shape.border_width,
                border_color: color(t.border),
                text_color: Some(color(this.theme.palette.text)),
            }
        }
    }

    /// Progress bar in a status tone (`None` = brand colour).
    pub fn progress(
        &self,
        tone: Option<Tone>,
    ) -> impl Fn(&IcedBaseTheme) -> progress_bar::Style + 'static {
        let t = self.theme.progress(tone);
        let border = self.border(t.border);
        move |_| progress_bar::Style {
            background: Background::Color(color(t.track)),
            bar: Background::Color(color(t.fill)),
            border,
        }
    }

    /// Slider style: rail from progress tokens, handle round for rounded
    /// themes and a bar otherwise.
    pub fn slider(&self) -> impl Fn(&IcedBaseTheme, slider::Status) -> slider::Style + 'static {
        let this = self.clone();
        move |_t, status| {
            let p = this.theme.progress(None);
            let st = match status {
                slider::Status::Active => ControlState::Idle,
                slider::Status::Hovered => ControlState::Hovered,
                slider::Status::Dragged => ControlState::Pressed,
            };
            let handle = this.theme.toggle(true, st);
            let shape = if this.theme.shape.kind() == ShapeKind::Rounded {
                slider::HandleShape::Circle { radius: 8.0 }
            } else {
                slider::HandleShape::Rectangle {
                    width: 10,
                    border_radius: Radius::from(0.0),
                }
            };
            slider::Style {
                rail: slider::Rail {
                    backgrounds: (
                        Background::Color(color(p.fill)),
                        Background::Color(color(p.track)),
                    ),
                    width: 4.0,
                    border: Border {
                        radius: this.radius().min(2.0).into(),
                        ..Border::default()
                    },
                },
                handle: slider::Handle {
                    shape,
                    background: Background::Color(color(handle.track)),
                    border_width: this.theme.shape.border_width,
                    border_color: color(handle.border),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;
    use dodai_core::Theme;

    fn tog(t: Theme, on: bool) -> toggler::Style {
        let lt = IcedTheme::new(t);
        lt.toggler()(&lt.iced_theme(), toggler::Status::Active { is_toggled: on })
    }

    #[test]
    fn toggler_uses_toggle_tokens() {
        let t = cyberpunk();
        let want = t.toggle(true, ControlState::Idle);
        let s = tog(t, true);
        assert_eq!(s.background, Background::Color(color(want.track)));
        assert_eq!(s.foreground, Background::Color(color(want.thumb)));
        let off = tog(cyberpunk(), false);
        assert_ne!(off.background, s.background);
    }

    #[test]
    fn toggler_radius_adapts() {
        assert_eq!(
            tog(rounded_light(), true).border_radius,
            Some(Radius::from(8.0))
        );
        assert_eq!(
            tog(hard_square(), true).border_radius,
            Some(Radius::from(0.0))
        );
        assert_eq!(tog(hard_square(), true).background_border_width, 2.0);
    }

    #[test]
    fn checkbox_states() {
        let lt = IcedTheme::new(rounded_light());
        let f = lt.checkbox();
        let on = f(
            &lt.iced_theme(),
            checkbox::Status::Active { is_checked: true },
        );
        let want = lt.theme().toggle(true, ControlState::Idle);
        assert_eq!(on.background, Background::Color(color(want.track)));
        assert_eq!(on.border.radius, 8.0.into());
        let dis = f(
            &lt.iced_theme(),
            checkbox::Status::Disabled { is_checked: true },
        );
        assert_eq!(
            dis.text_color,
            Some(color(lt.theme().palette.text_disabled))
        );
    }

    #[test]
    fn radio_selected_border_differs() {
        let lt = IcedTheme::new(cyberpunk());
        let f = lt.radio();
        let a = f(
            &lt.iced_theme(),
            radio::Status::Active { is_selected: true },
        );
        let b = f(
            &lt.iced_theme(),
            radio::Status::Active { is_selected: false },
        );
        assert_ne!(a.border_color, b.border_color);
    }

    #[test]
    fn progress_tone_and_radius() {
        let lt = IcedTheme::new(rounded_light());
        let s = lt.progress(Some(Tone::Danger))(&lt.iced_theme());
        assert_eq!(s.bar, Background::Color(color(lt.theme().palette.danger)));
        assert_eq!(s.border.radius, 8.0.into());
        let d = lt.progress(None)(&lt.iced_theme());
        assert_eq!(d.bar, Background::Color(color(lt.theme().palette.primary)));
    }

    #[test]
    fn slider_handle_shape_per_kind() {
        let h = |t: Theme| {
            let lt = IcedTheme::new(t);
            lt.slider()(&lt.iced_theme(), slider::Status::Active)
                .handle
                .shape
        };
        assert!(matches!(
            h(rounded_light()),
            slider::HandleShape::Circle { .. }
        ));
        assert!(matches!(
            h(hard_square()),
            slider::HandleShape::Rectangle { .. }
        ));
        assert!(matches!(
            h(cyberpunk()),
            slider::HandleShape::Rectangle { .. }
        ));
    }
}
