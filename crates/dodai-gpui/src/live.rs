//! A live sample window shared between a data source and the charts that draw it. Push samples
//! from anywhere; the chart slides the window left between pushes, so the motion is continuous
//! rather than one step per sample.

use std::cell::RefCell;
use std::rc::Rc;

use dodai_charts_core::stream::RingBuffer;

struct Inner {
    buf: RingBuffer,
    /// Animation time of the last push.
    last: f32,
    /// Smoothed seconds between pushes.
    interval: f32,
    next_due: f32,
}

/// A fixed-size window of recent samples. Clones share the same data.
#[derive(Clone)]
pub struct Live(Rc<RefCell<Inner>>);

impl Live {
    /// A window of `capacity` samples, starting flat at zero.
    pub fn new(capacity: usize) -> Self {
        Self(Rc::new(RefCell::new(Inner {
            buf: RingBuffer::filled(capacity, 0.0),
            last: crate::motion::time(),
            interval: 1.0,
            next_due: 0.0,
        })))
    }

    /// A window already full of `sample(0..capacity)`, so a chart doesn't start flat.
    pub fn prefilled(capacity: usize, mut sample: impl FnMut(u64) -> f32) -> Self {
        let live = Self::new(capacity);
        {
            let mut s = live.0.borrow_mut();
            for _ in 0..capacity.max(2) {
                let v = sample(s.buf.pushed());
                s.buf.push(v);
            }
        }
        live
    }

    /// Adds a sample now.
    pub fn push(&self, v: f32) {
        let now = crate::motion::time();
        let mut s = self.0.borrow_mut();
        let dt = (now - s.last).clamp(0.02, 10.0);
        s.interval = 0.8 * s.interval + 0.2 * dt;
        s.last = now;
        s.buf.push(v);
    }

    /// Calls `sample(index)` and pushes the result once every `every` seconds of animation time
    /// (up to three catch-up samples after a stall). Call it every frame.
    pub fn advance(&self, every: f32, mut sample: impl FnMut(u64) -> f32) {
        let now = crate::motion::time();
        let every = every.max(0.01);
        let mut due = self.0.borrow().next_due;
        if due == 0.0 || now - due > 3.0 * every {
            due = now;
        }
        let mut guard = 0;
        while now >= due && guard < 3 {
            let i = self.0.borrow().buf.pushed();
            let v = sample(i);
            self.push(v);
            due += every;
            guard += 1;
        }
        self.0.borrow_mut().next_due = due;
    }

    /// The samples, oldest first.
    pub fn values(&self) -> Vec<f32> {
        self.0.borrow().buf.to_vec()
    }

    /// How far through the current sample interval the window is, in 0..1: how much a chart
    /// slides left. Zero when animation is off.
    pub fn scroll(&self) -> f32 {
        if !crate::motion::enabled() {
            return 0.0;
        }
        let s = self.0.borrow();
        ((crate::motion::time() - s.last) / s.interval.max(0.02)).clamp(0.0, 1.0)
    }
}

/// A smooth wander in `-1..=1` on the animation clock; each `key` takes its own path. It
/// pauses with the animations and runs at their speed.
pub fn wander(key: f32) -> f32 {
    dodai_core::motion::wander(crate::motion::time(), key)
}

/// [`wander`] mapped onto `lo..=hi`: a demo value that drifts while animations run.
pub fn wander_between(key: f32, lo: f32, hi: f32) -> f32 {
    dodai_core::motion::wander_between(crate::motion::time(), key, lo, hi)
}

/// A live stream shaped like `base`, sliding left on the animation clock (see
/// [`dodai_core::motion::drift`]).
pub fn drift(base: &[f32], key: f32) -> Vec<f32> {
    dodai_core::motion::drift(base, crate::motion::time(), key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefilled_window_is_full_of_samples() {
        let l = Live::prefilled(5, |i| i as f32);
        assert_eq!(l.values(), vec![5.0, 6.0, 7.0, 8.0, 9.0]);
    }

    #[test]
    fn push_drops_the_oldest() {
        let l = Live::prefilled(3, |_| 1.0);
        l.push(2.0);
        assert_eq!(l.values(), vec![1.0, 1.0, 2.0]);
    }
}
