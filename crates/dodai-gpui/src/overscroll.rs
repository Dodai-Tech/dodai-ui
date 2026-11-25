//! gpui side of [`Overscroll`]: wheel deltas and scroll-handle edges. The physics is
//! [`dodai_core::overscroll`], shared with the other toolkits.

use gpui::{Pixels, ScrollDelta, ScrollHandle};

pub use dodai_core::overscroll::{resist, Overscroll, OverscrollFeel};

/// Vertical content delta of a wheel event in px, platform-neutral. Smooth
/// (touchpad) deltas pass through; discrete mouse-wheel notches (`Lines`) are
/// scaled down and capped so one notch gives a small bump, not a lurch.
pub fn wheel_dy(d: &ScrollDelta) -> f32 {
    match d {
        ScrollDelta::Pixels(p) => f32::from(p.y),
        ScrollDelta::Lines(l) => (l.y * LINE_PX).clamp(-LINE_CAP, LINE_CAP),
    }
}

const LINE_PX: f32 = 8.0;
const LINE_CAP: f32 = 24.0;

/// `(at_top, at_bottom)` of a scroll handle (a container with no range is
/// both, so either direction stretches it).
pub fn edges(handle: &ScrollHandle) -> (bool, bool) {
    let off = f32::from(handle.offset().y);
    let max: Pixels = handle.max_offset().height;
    let max = f32::from(max);
    (off >= -0.5, -off >= max - 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_notches_are_tamed_but_touchpad_pixels_pass_through() {
        use gpui::{point, px};
        assert_eq!(
            wheel_dy(&ScrollDelta::Pixels(point(px(0.0), px(37.0)))),
            37.0
        );
        assert_eq!(wheel_dy(&ScrollDelta::Lines(point(0.0, 1.0))), 8.0);
        assert_eq!(
            wheel_dy(&ScrollDelta::Lines(point(0.0, -50.0))),
            -24.0,
            "capped"
        );
    }
}
