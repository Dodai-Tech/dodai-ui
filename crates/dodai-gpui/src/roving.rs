//! Roving keyboard focus for a row or column of items: the arrow, Home and End keys pick the
//! next enabled item. One rule shared by `Tabs`, `Pagination`, `Stepper`, `List`, `Menu`,
//! `TreeView`, `Carousel` and `Slider`; the pure part ([`next`]) is unit-tested.

/// Which arrow keys move through the items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// Left and right.
    Horizontal,
    /// Up and down.
    Vertical,
}

/// The item a key moves to from `at` among `n` items, skipping any `enabled` says no to.
/// `None`: not a navigation key, or nothing enabled lies that way (the focus stays).
/// Home and End jump to the first and last enabled item; the arrows stop at the ends.
pub fn next(
    key: &str,
    axis: Axis,
    at: usize,
    n: usize,
    enabled: impl Fn(usize) -> bool,
) -> Option<usize> {
    if n == 0 {
        return None;
    }
    let (back, fwd) = match axis {
        Axis::Horizontal => ("left", "right"),
        Axis::Vertical => ("up", "down"),
    };
    let step = |from: usize, up: bool| {
        if up {
            (from + 1..n).find(|i| enabled(*i))
        } else {
            (0..from).rev().find(|i| enabled(*i))
        }
    };
    let to = match key {
        k if k == fwd => step(at, true),
        k if k == back => step(at, false),
        "home" => (0..n).find(|i| enabled(*i)),
        "end" => (0..n).rev().find(|i| enabled(*i)),
        _ => return None,
    };
    to.filter(|t| *t != at)
}

#[cfg(test)]
mod tests {
    use super::*;

    const H: Axis = Axis::Horizontal;

    #[test]
    fn arrows_step_and_stop_at_the_ends() {
        let all = |_| true;
        assert_eq!(next("right", H, 0, 3, all), Some(1));
        assert_eq!(next("left", H, 1, 3, all), Some(0));
        assert_eq!(next("right", H, 2, 3, all), None);
        assert_eq!(next("left", H, 0, 3, all), None);
    }

    #[test]
    fn disabled_items_are_skipped() {
        let en = |i| i != 1;
        assert_eq!(next("right", H, 0, 3, en), Some(2));
        assert_eq!(next("left", H, 2, 3, en), Some(0));
        assert_eq!(next("home", H, 2, 3, |i| i != 0), Some(1));
        assert_eq!(next("end", H, 0, 3, |i| i != 2), Some(1));
    }

    #[test]
    fn nothing_enabled_that_way_keeps_the_focus() {
        assert_eq!(next("right", H, 0, 3, |i| i == 0), None);
        assert_eq!(next("home", H, 0, 3, |_| true), None);
    }

    #[test]
    fn the_axis_picks_the_keys() {
        assert_eq!(next("down", Axis::Vertical, 0, 2, |_| true), Some(1));
        assert_eq!(next("right", Axis::Vertical, 0, 2, |_| true), None);
        assert_eq!(next("down", H, 0, 2, |_| true), None);
    }

    #[test]
    fn empty_and_unknown_keys_do_nothing() {
        assert_eq!(next("right", H, 0, 0, |_| true), None);
        assert_eq!(next("a", H, 0, 3, |_| true), None);
    }
}
