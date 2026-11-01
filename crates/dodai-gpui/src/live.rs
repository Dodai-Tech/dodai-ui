//! A live sample window shared between a data source and the charts that draw it. Push samples
//! from anywhere; the chart slides the window left between pushes, so the motion is continuous
//! rather than one step per sample.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use dodai_charts_core::stream::RingBuffer;

struct Inner {
    buf: RingBuffer,
    /// Animation time of the last push.
    last: f32,
    /// Smoothed seconds between pushes.
    interval: f32,
    next_due: f32,
    /// Samples pushed from other threads, taken in on every read.
    feed: Option<Feed>,
}

/// Samples waiting for a [`Live`] window, pushed from any thread (a socket reader, a sensor
/// loop). Clones share the queue; attach it with [`Live::attach`].
#[derive(Clone, Default)]
pub struct Feed(Arc<Mutex<Vec<f32>>>);

impl Feed {
    /// An empty feed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues a sample. Cheap and callable from any thread.
    pub fn push(&self, v: f32) {
        if let Ok(mut q) = self.0.lock() {
            q.push(v);
        }
    }

    /// Queues several samples, oldest first.
    pub fn push_all(&self, vs: impl IntoIterator<Item = f32>) {
        if let Ok(mut q) = self.0.lock() {
            q.extend(vs);
        }
    }

    /// Takes every queued sample, oldest first.
    fn take(&self) -> Vec<f32> {
        self.0
            .lock()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default()
    }
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
            feed: None,
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

    /// Draws samples from `feed` too: whatever other threads queue lands in the window the next
    /// time it is read, so a chart can follow a background source with no polling code.
    pub fn attach(self, feed: &Feed) -> Self {
        self.0.borrow_mut().feed = Some(feed.clone());
        self
    }

    /// Moves queued feed samples into the window (a burst larger than the window keeps its
    /// newest samples).
    fn pull(&self) {
        let queued = self.0.borrow().feed.as_ref().map(Feed::take);
        let Some(queued) = queued else { return };
        let keep = self.0.borrow().buf.capacity();
        let skip = queued.len().saturating_sub(keep);
        for v in queued.into_iter().skip(skip) {
            self.push(v);
        }
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
        self.pull();
        self.0.borrow().buf.to_vec()
    }

    /// How far through the current sample interval the window is, in 0..1: how much a chart
    /// slides left. Zero when animation is off.
    pub fn scroll(&self) -> f32 {
        self.pull();
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

#[cfg(test)]
mod feed_tests {
    use super::*;

    #[test]
    fn samples_pushed_from_another_thread_reach_the_window() {
        let feed = Feed::new();
        let live = Live::new(4).attach(&feed);
        let f = feed.clone();
        std::thread::spawn(move || f.push_all([1.0, 2.0]))
            .join()
            .unwrap();
        assert_eq!(live.values(), vec![0.0, 0.0, 1.0, 2.0]);
        // A burst larger than the window keeps the newest samples.
        feed.push_all((1..=10).map(|v| v as f32));
        assert_eq!(live.values(), vec![7.0, 8.0, 9.0, 10.0]);
        // Nothing queued: nothing changes.
        assert_eq!(live.values(), vec![7.0, 8.0, 9.0, 10.0]);
    }
}
