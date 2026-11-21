//! # dodai-xilem
//!
//! Turns a [`dodai_core::Theme`] into [xilem](https://docs.rs/xilem) view styling (masonry
//! properties, drawn by vello). Nothing here looks at a theme id: colour, radius, border width
//! and sizes all come from tokens, so new themes work automatically.
//!
//! Scaffold: [`color`] and [`button`]. More components follow `docs/PARITY.md`.

pub mod foundation;
mod generated;
pub use foundation::{Colors, Face, Foundation, Metrics, Size, Type};
pub use generated::{PanelStyle, TagStyle};

use dodai_core::{ButtonVariant, ControlState, ItemState, Theme, Tone};
use xilem::style::Style;
use xilem::view::{button as masonry_button, flex_row, label, sized_box};
use xilem::{Color, WidgetView};

/// Applies the shape every themed box shares (fill, border, radius, padding) to a masonry view.
/// A macro, not a function, because each view type returns its own wrapper types.
macro_rules! shaped {
    ($view:expr, $theme:expr, fill: $fill:expr, border: $border:expr, pad: $pad:expr) => {
        $view
            .background_color($fill)
            .border($border, f64::from($theme.shape.border_width))
            .corner_radius(f64::from($theme.shape.radius))
            .padding(f64::from($pad))
    };
}

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
    let text = display(theme, text.into());
    let view = masonry_button::<State, (), _>(
        label(text)
            .text_size(theme.typography.size_md)
            .color(color(idle.foreground)),
        on_press,
    );
    shaped!(view, theme, fill: fill(idle.background), border: fill(idle.border), pad: theme.button_padding())
        .active_background_color(fill(pressed.background))
        .disabled_background_color(fill(off.background))
}

/// Status tag: solid or outlined in a [`Tone`].
pub fn tag<State: 'static>(
    theme: &Theme,
    text: impl Into<String>,
    tone: Tone,
    solid: bool,
) -> impl WidgetView<State> {
    let tok = TagStyle::of(theme, tone, solid);
    let view = sized_box(
        label(display(theme, text.into()))
            .text_size(theme.typography.size_xs)
            .color(tok.foreground),
    );
    shaped!(view, theme, fill: tok.background, border: tok.border, pad: theme.spacing.sm)
}

/// Panel: the surface, border and radius of a container around `child`.
pub fn panel<State: 'static, V: WidgetView<State>>(
    theme: &Theme,
    focused: bool,
    child: V,
) -> impl WidgetView<State> {
    let tok = PanelStyle::of(theme, focused);
    shaped!(sized_box(child), theme, fill: tok.background, border: tok.border, pad: theme.spacing.md)
}

/// Tab strip: one button per label, the active one filled from the tab tokens. `on_select`
/// gets the index of the clicked tab.
pub fn tabs<State: 'static>(
    theme: &Theme,
    labels: &[&str],
    active: usize,
    on_select: impl Fn(&mut State, usize) + Clone + Send + Sync + 'static,
) -> impl WidgetView<State> {
    let items: Vec<_> = labels
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let tok = theme.tab(ItemState::of(i == active, false));
            let on_select = on_select.clone();
            let view = masonry_button::<State, (), _>(
                label(display(theme, (*text).to_string()))
                    .text_size(theme.typography.size_md)
                    .color(color(tok.foreground)),
                move |state: &mut State| on_select(state, i),
            );
            shaped!(view, theme, fill: fill(tok.background), border: fill(tok.indicator), pad: theme.spacing.md)
        })
        .collect();
    flex_row(items).gap(xilem::masonry::properties::types::Length::px(f64::from(
        theme.spacing.xs,
    )))
}

/// A label as the design writes it (some designs set labels in capitals).
fn display(theme: &Theme, text: String) -> String {
    if theme.typography.uppercase_labels {
        text.to_uppercase()
    } else {
        text
    }
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
    fn tag_panel_and_tabs_build() {
        let t = Theme::default();
        let _ = tag::<()>(&t, "New", Tone::Info, true);
        let _ = panel::<(), _>(&t, false, label("x"));
        let _ = tabs(&t, &["a", "b"], 1, |_: &mut (), _| {});
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
