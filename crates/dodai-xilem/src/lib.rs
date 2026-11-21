//! # dodai-xilem
//!
//! Turns a [`dodai_core::Theme`] into [xilem](https://docs.rs/xilem) view styling (masonry
//! properties, drawn by vello). Nothing here looks at a theme id: colour, radius, border width
//! and sizes all come from tokens, so new themes work automatically.
//!
//! Scaffold: [`color`] and [`button`]. More components follow `docs/PARITY.md`.

use dodai_core::{ButtonVariant, ControlState, Theme};
use xilem::style::Style;
use xilem::view::{button as masonry_button, label};
use xilem::{Color, WidgetView};

/// A token colour as a vello/peniko colour.
pub fn color(c: dodai_core::Color) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

/// An optional token colour; none is transparent.
fn fill(c: Option<dodai_core::Color>) -> Color {
    c.map_or(Color::TRANSPARENT, color)
}

/// Themed button: the variant's fill, border, radius and padding from tokens. `on_press` runs on
/// a click or on Enter/Space while focused.
pub fn button<State: 'static>(
    theme: &Theme,
    text: impl Into<String>,
    variant: ButtonVariant,
    on_press: impl Fn(&mut State) + Send + Sync + 'static,
) -> impl WidgetView<State> {
    let idle = theme.button(variant, ControlState::Idle);
    let pressed = theme.button(variant, ControlState::Pressed);
    let off = theme.button(variant, ControlState::Disabled);
    let text = if theme.typography.uppercase_labels {
        text.into().to_uppercase()
    } else {
        text.into()
    };
    masonry_button::<State, (), _>(
        label(text)
            .text_size(theme.typography.size_md)
            .color(color(idle.foreground)),
        on_press,
    )
    .background_color(fill(idle.background))
    .active_background_color(fill(pressed.background))
    .disabled_background_color(fill(off.background))
    .border(fill(idle.border), f64::from(theme.shape.border_width))
    .corner_radius(f64::from(theme.shape.radius))
    .padding(f64::from(theme.button_padding()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_keeps_every_channel() {
        let c = color(dodai_core::Color {
            r: 1,
            g: 2,
            b: 3,
            a: 4,
        });
        assert_eq!(c.to_rgba8().to_u8_array(), [1, 2, 3, 4]);
    }

    #[test]
    fn a_button_builds_for_every_variant() {
        let t = Theme::default();
        for v in [
            ButtonVariant::Primary,
            ButtonVariant::Secondary,
            ButtonVariant::Outline,
            ButtonVariant::OutlineSecondary,
            ButtonVariant::Danger,
            ButtonVariant::Scrim,
            ButtonVariant::Ghost,
        ] {
            let _ = button(&t, "Go", v, |_: &mut ()| {});
        }
    }
}
