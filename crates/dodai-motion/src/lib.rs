//! Toolkit-agnostic motion primitives. See the crate README.

mod curve;
mod easing;
mod motion;
#[cfg(test)]
mod tests;
mod tween;

pub use curve::{lerp, reveal, stagger_window, Curve};
pub use easing::Easing;
pub use motion::{EntryKind, EntrySet, Motion};
pub use tween::Tween;
