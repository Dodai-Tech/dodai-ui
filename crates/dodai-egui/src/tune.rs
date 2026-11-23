//! App-wide shape tuning (roundness, borders, hard shadows), the egui side of
//! [`dodai_core::ShapeTune`]. Roundness, border width and shadow offset are applied to the theme
//! ([`dodai_core::Theme::tuned`]); the border colour, outline-all and shadow direction are read
//! while painting.

use dodai_core::ShapeTune;
use std::cell::Cell;

thread_local! {
    static TUNE: Cell<ShapeTune> = const { Cell::new(ShapeTune::DEFAULT) };
}

/// The tuning applied to every control.
pub fn shape_tune() -> ShapeTune {
    TUNE.with(Cell::get)
}

/// Set the tuning read while painting. [`crate::ThemeController`] calls this for you.
pub fn set_shape_tune(tune: ShapeTune) {
    TUNE.with(|c| c.set(tune));
}
