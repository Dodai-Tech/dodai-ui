//! Reading direction. The rule lives in `dodai-core`; this adds the repaint.

use gpui::App;

pub use dodai_core::direction::{direction, is_rtl, mirror_key, Direction};

/// Sets the reading direction and repaints.
pub fn set_direction(cx: &mut App, d: Direction) {
    dodai_core::direction::set_direction(d);
    cx.refresh_windows();
}
