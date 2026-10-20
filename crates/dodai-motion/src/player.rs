//! Playing a clip against the host's clock.

use crate::{Clip, Motion, Pose};

/// A [`Clip`] started at some moment. Pass the host's current time (seconds) to read it.
#[derive(Clone)]
pub struct Player<T> {
    clip: Clip<T>,
    start: f32,
}

impl<T: 'static> Player<T> {
    pub fn new(clip: Clip<T>, now: f32) -> Self {
        Self { clip, start: now }
    }
    pub fn value(&self, now: f32) -> T {
        self.clip.at(now - self.start)
    }
    pub fn done(&self, now: f32) -> bool {
        self.clip.done(now - self.start)
    }
    pub fn restart(&mut self, now: f32) {
        self.start = now;
    }
    /// Swap in another clip and start it now.
    pub fn play(&mut self, clip: Clip<T>, now: f32) {
        self.clip = clip;
        self.start = now;
    }
}

/// The entry animation for a widget, in three calls: [`frame`](Animator::frame) to paint,
/// [`done`](Animator::done) to know whether to ask for another frame, and
/// [`restart`](Animator::restart) to replay.
///
/// ```
/// use dodai_motion::{Animator, Motion};
/// let a = Animator::new(Motion::default(), 0.0);
/// let pose = a.frame(0.05);          // paint with pose.opacity / pose.dy
/// let again = !a.done(0.05);         // if so, request a redraw
/// assert!(again && pose.opacity < 1.0);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Animator {
    motion: Motion,
    start: f32,
}

impl Animator {
    pub fn new(motion: Motion, now: f32) -> Self {
        Self { motion, start: now }
    }
    /// The pose `now` (resting once finished).
    pub fn frame(&self, now: f32) -> Pose {
        let (opacity, dx, dy) = self.motion.entry_frame(now - self.start);
        Pose {
            opacity,
            dx,
            dy,
            ..Pose::IDENTITY
        }
    }
    pub fn done(&self, now: f32) -> bool {
        let total = self.motion.entry.duration(self.motion.duration) + self.motion.delay;
        now - self.start >= total.as_secs_f32()
    }
    pub fn restart(&mut self, now: f32) {
        self.start = now;
    }
}
