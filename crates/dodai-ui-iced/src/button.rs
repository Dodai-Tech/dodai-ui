//! Buttons.

use dodai_ui_core::{ButtonTokens, ButtonVariant, ControlState, ShapeKind};
use iced::{
    alignment,
    widget::{button, text},
    Background, Border, Element, Length, Theme as IcedBaseTheme,
};

use crate::{color, IcedTheme, ShapedButton, ShapedColors};

impl IcedTheme {
    /// Native button style closure for `.style(...)` (rounded / square shapes;
    /// chamfered themes get square corners — use [`IcedTheme::button_view`]).
    pub fn button(
        &self,
        variant: ButtonVariant,
    ) -> impl Fn(&IcedBaseTheme, button::Status) -> button::Style + 'static {
        let tok = |s| self.button_colours(variant, s);
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
                        radius: this.radius().into(),
                        ..Border::default()
                    },
                },
                shadow: this.shadow(lit, false),
                snap: true,
            }
        }
    }

    /// Button shaped per theme tokens, drawn on a canvas (works for every
    /// [`ShapeKind`]). `None` renders it disabled.
    pub fn shaped_button<Message: Clone>(
        &self,
        label: &str,
        variant: ButtonVariant,
        on_press: Option<Message>,
    ) -> ShapedButton<Message> {
        let mut b =
            self.shaped_button_sized(label, variant, dodai_ui_core::ControlSize::Medium, on_press);
        // The classic button: the design's control height and large padding.
        b.size = self.theme.typography.size_md;
        b.height = self.theme.shape.control_height;
        b.padding_x = self.theme.spacing.lg;
        b
    }

    /// A [`ShapedButton`] at a [`dodai_ui_core::ControlSize`] step.
    pub fn shaped_button_sized<Message: Clone>(
        &self,
        label: &str,
        variant: ButtonVariant,
        size: dodai_ui_core::ControlSize,
        on_press: Option<Message>,
    ) -> ShapedButton<Message> {
        let m = self.theme.control_metrics(size);
        let e = &self.theme.effects;
        let colors = |st| {
            let t = self.button_colours(variant, st);
            let lit = matches!(st, ControlState::Hovered | ControlState::Pressed);
            ShapedColors {
                fill: t.background.map(color),
                stroke: t.border.or(t.background).map(color),
                text: color(t.foreground),
                glow: (lit && e.glow_enabled()).then(|| color(e.glow_color())),
            }
        };
        ShapedButton {
            label: self.label(label),
            font: self.font(),
            size: m.text_size,
            shape: self.theme.shape.clone(),
            border_width: self.theme.shape.border_width,
            idle: colors(ControlState::Idle),
            hover: colors(ControlState::Hovered),
            press: colors(ControlState::Pressed),
            disabled: colors(ControlState::Disabled),
            glow_radius: e.glow_radius(),
            on_press,
            backdrop: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.6),
            height: m.height,
            padding_x: m.padding,
            tick: None,
            shimmer: self.shimmer.map(|x| (self.shimmer_colour(variant), x)),
        }
    }

    /// Ready-to-use button: native for rounded / square themes, canvas
    /// [`ShapedButton`] for chamfered themes. `None` renders it disabled.
    pub fn button_view<'a, Message: Clone + 'a>(
        &self,
        label: &str,
        variant: ButtonVariant,
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        if self.theme.shape.kind() == ShapeKind::Chamfered {
            return self.shaped_button(label, variant, on_press).into();
        }
        let content = text(self.label(label))
            .size(self.theme.typography.size_md)
            .align_y(alignment::Vertical::Center)
            .height(Length::Fill);
        button(content)
            .padding([0.0, self.theme.spacing.lg])
            .height(self.theme.shape.control_height)
            .style(self.button(variant))
            .on_press_maybe(on_press)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;
    use dodai_ui_core::{Color as LColor, Theme};
    use iced::Shadow;

    fn st(t: Theme, v: ButtonVariant, s: button::Status) -> button::Style {
        let lt = IcedTheme::new(t);
        lt.button(v)(&lt.iced_theme(), s)
    }

    #[test]
    fn primary_button_is_solid_yellow() {
        let s = st(cyberpunk(), ButtonVariant::Primary, button::Status::Active);
        assert_eq!(
            s.background,
            Some(Background::Color(color(LColor::hex(0xfcee0a))))
        );
        assert_eq!(s.shadow, Shadow::default());
        let h = st(cyberpunk(), ButtonVariant::Primary, button::Status::Hovered);
        assert!(h.shadow.blur_radius > 0.0);
    }

    #[test]
    fn outline_fills_on_hover() {
        assert!(
            st(cyberpunk(), ButtonVariant::Outline, button::Status::Active)
                .background
                .is_none()
        );
        assert!(
            st(cyberpunk(), ButtonVariant::Outline, button::Status::Hovered)
                .background
                .is_some()
        );
    }

    #[test]
    fn disabled_uses_disabled_tokens() {
        let t = cyberpunk();
        let want = t.button(ButtonVariant::Primary, ControlState::Disabled);
        let s = st(t, ButtonVariant::Primary, button::Status::Disabled);
        assert_eq!(s.text_color, color(want.foreground));
    }

    #[test]
    fn radius_and_glow_adapt_to_theme() {
        let r = st(
            rounded_light(),
            ButtonVariant::Primary,
            button::Status::Hovered,
        );
        assert_eq!(r.border.radius, 8.0.into());
        assert_eq!(r.shadow.blur_radius, 0.0); // no glow token
        let q = st(
            hard_square(),
            ButtonVariant::Outline,
            button::Status::Active,
        );
        assert_eq!(q.border.radius, 0.0.into());
        assert_eq!(q.border.width, 2.0);
        let c = st(cyberpunk(), ButtonVariant::Outline, button::Status::Active);
        assert_eq!(c.border.width, cyberpunk().shape.border_width);
    }

    #[test]
    fn glow_intensity_scales_hover_glow() {
        let a = st(
            cyberpunk().with_glow_intensity(1.0),
            ButtonVariant::Primary,
            button::Status::Hovered,
        );
        let b = st(
            cyberpunk().with_glow_intensity(2.0),
            ButtonVariant::Primary,
            button::Status::Hovered,
        );
        assert!(b.shadow.blur_radius > a.shadow.blur_radius);
    }

    #[test]
    fn shaped_button_geometry_from_tokens() {
        let cp = IcedTheme::new(cyberpunk()).shaped_button("go", ButtonVariant::Primary, Some(()));
        assert_eq!(cp.label, "GO");
        assert_eq!(cp.height, cyberpunk().shape.control_height);
        assert!(cp.idle.glow.is_none() && cp.hover.glow.is_some());
        assert!(cp.margin() > 0.0);
        let sq =
            IcedTheme::new(hard_square()).shaped_button("go", ButtonVariant::Primary, Some(()));
        assert_eq!(sq.label, "go");
        assert_eq!(sq.margin(), 0.0);
        assert_eq!(sq.height, 32.0);
        assert!(sq.hover.glow.is_none());
        let (w, h) = sq.outer_size();
        assert_eq!(h, 32.0);
        assert!(w >= 80.0);
        let off =
            IcedTheme::new(hard_square()).shaped_button::<()>("go", ButtonVariant::Primary, None);
        assert!(off.on_press.is_none());
    }
}
