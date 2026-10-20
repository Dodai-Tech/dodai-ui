//! Toolkit-agnostic motion primitives. See the crate README.
//!
//! Nothing here reads a clock: hosts pass elapsed seconds in, so the crate runs on every
//! platform (including wasm) and is `Send + Sync`.

mod clip;
mod curve;
mod easing;
mod lerp;
mod mixins;
mod motion;
mod physics;
#[cfg(test)]
mod physics_tests;
mod player;
mod pose;
#[cfg(test)]
mod tests;
mod tween;

pub use clip::Clip;
pub use curve::{lerp, reveal, stagger_window, Curve};
pub use easing::Easing;
pub use lerp::Lerp;
pub use mixins::{
    attention_bounce, check_draw, collapse, count_up, flash_on_change, flip, indeterminate,
    magnetic, marquee, nudge, overscroll, page_snap, pop_in, pull_to_refresh, shake,
    skeleton_shimmer, swipe_release, tilt, typewriter, wobble, Personality, Reaction, Reactions,
    Reactive, Release, Trigger,
};
pub use motion::{EntryKind, EntrySet, Motion};
pub use physics::{damp, nearest, rubber_band, Bounce, Decay, Shake, Spring, SpringState};
pub use player::{Animator, Player};
pub use pose::Pose;
pub use tween::Tween;
