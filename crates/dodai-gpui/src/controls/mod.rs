//! Form controls: [`Toggle`] (switch), [`Checkbox`], [`Radio`], [`ProgressBar`], [`Slider`].

use crate::paint::ShapeStyle;
use dodai_core::{ControlState, Theme};
use gpui::{App, Window};
use std::rc::Rc;

pub use dodai_core::slider_math::{
    clamp01, key_value, nearest_thumb, range_update, slider_fraction, slider_value, tick_fractions,
};

/// Paint style of a toggle/checkbox/radio box (pure).
pub fn toggle_style(theme: &Theme, on: bool, state: ControlState) -> ShapeStyle {
    let k = theme.toggle(on, state);
    ShapeStyle::from_tokens(
        theme,
        Some(k.track),
        Some(k.border),
        on && state != ControlState::Disabled,
        false,
    )
}

/// Where a slider's value bubble sits relative to the thumb.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BubblePlacement {
    /// Above the thumb.
    Above,
    /// Below the thumb.
    Below,
}

type FloatHandler = Rc<dyn Fn(f32, &mut Window, &mut App)>;

mod bars;
mod choice;
mod slider;

pub use bars::*;
pub use choice::*;
pub use slider::*;

#[cfg(test)]
mod tests;
