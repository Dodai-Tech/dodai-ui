//! Reading direction. Right-to-left (Arabic, Hebrew) mirrors what has a side: the arrow keys of
//! a row of items, the accent marker and the indent of a list row. The direction is one
//! setting per thread, read while drawing. Shared by every toolkit.
//!
//! Not mirrored yet: text input editing (caret and selection are left-to-right), sliders and
//! chevron icons. Text itself is shaped by the platform, which orders mixed scripts.

use std::cell::Cell;

/// Which way text and rows run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    /// Left to right (default).
    #[default]
    Ltr,
    /// Right to left.
    Rtl,
}

thread_local! {
    pub(crate) static CURRENT: Cell<Direction> = const { Cell::new(Direction::Ltr) };
}

/// Sets the reading direction (the toolkit repaints afterwards).
pub fn set_direction(d: Direction) {
    CURRENT.with(|c| c.set(d));
}

/// The reading direction.
pub fn direction() -> Direction {
    CURRENT.with(Cell::get)
}

/// Whether the reading direction is right to left.
pub fn is_rtl() -> bool {
    direction() == Direction::Rtl
}

/// `left` and `right` swapped when `rtl`: the key that means "further along" for a row.
pub fn mirror_key(key: &str, rtl: bool) -> &str {
    match (rtl, key) {
        (true, "left") => "right",
        (true, "right") => "left",
        _ => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtl_swaps_the_horizontal_arrows_only() {
        assert_eq!(mirror_key("left", true), "right");
        assert_eq!(mirror_key("right", true), "left");
        assert_eq!(mirror_key("left", false), "left");
        assert_eq!(mirror_key("up", true), "up");
        assert_eq!(mirror_key("home", true), "home");
    }
}
