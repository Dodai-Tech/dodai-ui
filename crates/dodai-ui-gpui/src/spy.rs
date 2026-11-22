//! [`ScrollSpy`]: which section of a scrolling page is "current", for a
//! sidebar / table-of-contents highlight.
//!
//! The naive rule ("last section whose top has reached the viewport top")
//! fails in two ways: sections near the end of a short page can never reach
//! the top, so the highlight sticks on the first item; and a clicked entry
//! that can't be scrolled to the top isn't highlighted. `ScrollSpy` fixes
//! both: at the end of the scroll range the last section wins, and an
//! explicit [`pin`](ScrollSpy::pin) (a click) wins until the user scrolls.

use gpui::{Pixels, ScrollHandle};

pub use dodai_ui_core::spy::pick;

/// Scroll-spy state: just the optional pinned (clicked) key.
#[derive(Clone, Copy, Debug)]
pub struct ScrollSpy<K> {
    pinned: Option<K>,
}

impl<K> Default for ScrollSpy<K> {
    fn default() -> Self {
        Self { pinned: None }
    }
}

impl<K: Copy + PartialEq> ScrollSpy<K> {
    /// Create a new [`ScrollSpy`].
    pub fn new() -> Self {
        Self::default()
    }
    /// Force `key` current (call on click) until [`release`](Self::release).
    pub fn pin(&mut self, key: K) {
        self.pinned = Some(key);
    }
    /// Go back to following the scroll position (call on user scroll or when
    /// the page changes).
    pub fn release(&mut self) {
        self.pinned = None;
    }
    /// The current key. `items` is `(key, child index)` in document order;
    /// positions come from the previous frame's layout of `handle`.
    pub fn current(&self, handle: &ScrollHandle, items: &[(K, usize)], slack: f32) -> Option<K> {
        let off = handle.offset().y;
        let tops: Vec<(K, f32)> = items
            .iter()
            .map(|(k, ix)| {
                let y = handle
                    .bounds_for_item(*ix)
                    .map_or(f32::INFINITY, |b| f32::from(b.top() + off));
                (*k, y)
            })
            .collect();
        let max: Pixels = handle.max_offset().height;
        let at_end = f32::from(max) > 1.0 && f32::from(-off) >= f32::from(max) - 1.0;
        pick(
            &tops,
            self.pinned,
            f32::from(handle.bounds().top()),
            at_end,
            slack,
        )
    }
}
