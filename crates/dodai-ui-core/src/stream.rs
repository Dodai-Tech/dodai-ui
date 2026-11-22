//! A fixed-capacity sample window for live charts: push at one end, the oldest falls off the
//! other. Pure, so every toolkit streams the same way.

use std::collections::VecDeque;

/// The most recent `capacity` samples, oldest first.
#[derive(Clone, Debug, PartialEq)]
pub struct RingBuffer {
    cap: usize,
    data: VecDeque<f32>,
    pushed: u64,
}

impl RingBuffer {
    /// An empty buffer holding up to `capacity` samples (at least 2).
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(2);
        Self {
            cap,
            data: VecDeque::with_capacity(cap),
            pushed: 0,
        }
    }

    /// A buffer filled with `capacity` copies of `v`, so a chart starts full.
    pub fn filled(capacity: usize, v: f32) -> Self {
        let mut b = Self::new(capacity);
        for _ in 0..b.cap {
            b.push(v);
        }
        b
    }

    /// Adds a sample; non-finite values become 0. Drops the oldest when full.
    pub fn push(&mut self, v: f32) {
        if self.data.len() == self.cap {
            self.data.pop_front();
        }
        self.data.push_back(if v.is_finite() { v } else { 0.0 });
        self.pushed += 1;
    }

    /// The samples, oldest first.
    pub fn to_vec(&self) -> Vec<f32> {
        self.data.iter().copied().collect()
    }

    /// How many samples are held.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether no sample was pushed yet.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// The window size.
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Samples pushed since creation; a change means the window moved.
    pub fn pushed(&self) -> u64 {
        self.pushed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_the_oldest_when_full() {
        let mut b = RingBuffer::new(3);
        for v in [1.0, 2.0, 3.0, 4.0] {
            b.push(v);
        }
        assert_eq!(b.to_vec(), vec![2.0, 3.0, 4.0]);
        assert_eq!(b.pushed(), 4);
    }

    #[test]
    fn hostile_samples_become_zero() {
        let mut b = RingBuffer::new(2);
        b.push(f32::NAN);
        b.push(f32::INFINITY);
        assert_eq!(b.to_vec(), vec![0.0, 0.0]);
    }

    #[test]
    fn filled_starts_full() {
        let b = RingBuffer::filled(4, 1.5);
        assert_eq!(b.len(), 4);
        assert!(!b.is_empty());
    }
}
