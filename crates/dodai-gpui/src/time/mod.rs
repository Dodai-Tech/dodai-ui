//! Time components: [`Clock`] (analog + digital), [`Timer`] (countdown),
//! [`Stopwatch`] (chronometer with laps) and [`TapTempo`] (measure the BPM of
//! music by tapping, with a visual metronome).
//!
//! All state lives in GPUI keyed state, so give each one a unique id. They
//! redraw every frame while running and are idle otherwise.

use gpui::{App, ElementId, Window};
use std::rc::Rc;

mod clock;
mod helpers;
mod stopwatch;
mod tap;
mod timer;

pub use clock::{Clock, ClockFace};
pub use helpers::*;
pub use stopwatch::Stopwatch;
pub use tap::TapTempo;
pub use timer::Timer;

pub(super) type VoidCb = Rc<dyn Fn(&mut Window, &mut App)>;
pub(super) type Cb<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;

pub(super) fn id2(id: &ElementId, s: &'static str) -> ElementId {
    (id.clone(), s).into()
}

#[cfg(test)]
mod tests;
