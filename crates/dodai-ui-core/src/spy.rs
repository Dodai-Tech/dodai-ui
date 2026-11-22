//! Scroll-spy rule shared by every toolkit's sidebar: which section of a scrolling page is
//! "current".
//!
//! The naive rule ("last section whose top has reached the viewport top") fails twice: sections
//! near the end of a short page never reach the top, and a clicked entry that can't be scrolled
//! to the top isn't highlighted. So at the end of the scroll range the last section wins, and a
//! pinned (clicked) section wins until the user scrolls.

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
        assert_eq!(pick(&T, Some('d'), 0.0, false, 0.0), Some('d'));
        assert_eq!(pick(&T, None, 0.0, false, 0.0), Some('b'));
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
