//! Form controls: [`Toggle`] (switch), [`Checkbox`], [`Radio`], [`ProgressBar`], [`Slider`].

use crate::paint::ShapeStyle;
use dodai_core::{ControlState, Theme};
use gpui::{App, Window};
use std::rc::Rc;

/// Clamp to `0.0..=1.0` (NaN → 0).
pub fn clamp01(v: f32) -> f32 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(0.0, 1.0)
    }
}

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

/// Position of a value in `min..=max` as a 0..1 fraction.
pub fn slider_fraction(value: f32, min: f32, max: f32) -> f32 {
    if max <= min {
        0.0
    } else {
        clamp01((value - min) / (max - min))
    }
}

/// Where a slider's value bubble sits relative to the thumb.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BubblePlacement {
    /// Above the thumb.
    Above,
    /// Below the thumb.
    Below,
}

/// Tick positions (`0..=1`) for `min..=max` at multiples of `step`;
/// empty when there would be more than 64 (too dense to draw). Pure.
pub fn tick_fractions(min: f32, max: f32, step: Option<f32>) -> Vec<f32> {
    let Some(step) = step.filter(|s| *s > 0.0 && max > min) else {
        return Vec::new();
    };
    let n = ((max - min) / step).round() as usize;
    if n == 0 || n > 64 {
        return Vec::new();
    }
    (0..=n)
        .map(|i| ((i as f32 * step) / (max - min)).clamp(0.0, 1.0))
        .collect()
}

/// Map a pointer x to a slider value. `left`/`width` are the control's bounds,
/// `thumb` the thumb diameter (the thumb centre travels `width - thumb`).
pub fn slider_value(
    x: f32,
    left: f32,
    width: f32,
    thumb: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
) -> f32 {
    let span = (width - thumb).max(1.0);
    let f = clamp01((x - left - thumb / 2.0) / span);
    let mut v = min + f * (max - min);
    if let Some(s) = step.filter(|s| *s > 0.0) {
        v = min + ((v - min) / s).round() * s;
    }
    v.clamp(min.min(max), max.max(min))
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
