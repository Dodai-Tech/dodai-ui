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

/// Pure resolution rule (no gpui types, unit-tested).
///
/// `tops` are screen-space section tops in document order, `view_top` the
/// viewport top, `at_end` whether the scroll range is exhausted (and there is
/// a range), `slack` how far below the top counts as "reached".
pub fn pick<K: Copy + PartialEq>(
    tops: &[(K, f32)],
    pinned: Option<K>,
    view_top: f32,
    at_end: bool,
    slack: f32,
) -> Option<K> {
    if let Some(p) = pinned {
        if tops.iter().any(|(k, _)| *k == p) {
            return Some(p);
        }
    }
    if at_end {
        return tops.last().map(|(k, _)| *k);
    }
    tops.iter()
        .rfind(|(_, y)| *y <= view_top + slack)
        .or(tops.first())
        .map(|(k, _)| *k)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    const T: [(char, f32); 4] = [('a', -300.0), ('b', -20.0), ('c', 140.0), ('d', 500.0)];

    #[test]
    fn follows_the_last_section_that_reached_the_top() {
        assert_eq!(pick(&T, None, 0.0, false, 0.0), Some('b'));
        assert_eq!(pick(&T, None, 0.0, false, 150.0), Some('c'));
    }

    #[test]
    fn before_everything_falls_back_to_first() {
        let t = [('a', 200.0), ('b', 400.0)];
        assert_eq!(pick(&t, None, 0.0, false, 0.0), Some('a'));
    }

    #[test]
    fn end_of_range_selects_last_even_if_it_never_reached_the_top() {
        assert_eq!(pick(&T, None, 0.0, true, 0.0), Some('d'));
    }

    #[test]
    fn pin_wins_until_released() {
        let mut s = ScrollSpy::new();
        s.pin('d');
        assert_eq!(pick(&T, s.pinned, 0.0, false, 0.0), Some('d'));
        s.release();
        assert_eq!(pick(&T, s.pinned, 0.0, false, 0.0), Some('b'));
    }

    #[test]
    fn stale_pin_for_a_missing_section_is_ignored() {
        assert_eq!(pick(&T, Some('z'), 0.0, false, 0.0), Some('b'));
    }

    #[test]
    fn empty_page_has_no_current() {
        assert_eq!(pick::<char>(&[], None, 0.0, true, 0.0), None);
    }

    #[test]
    fn unlaid_out_sections_do_not_count_as_reached() {
        let t = [('a', -5.0), ('b', f32::INFINITY)];
        assert_eq!(pick(&t, None, 0.0, false, 0.0), Some('a'));
    }
}
