//! [`lazy`]: lay out and paint a child only while it is near the window.
//!
//! gpui lays out and paints every element it is given, even those scrolled far
//! out of view. For a long page that is the whole frame budget. `lazy` wraps
//! one block (a section, a grid of icons, a chart card…): once the block has
//! been laid out and measured, it is replaced by a spacer of the same height
//! whenever it is further than `margin` px outside the window. The child is
//! built either way (builders are cheap), but never laid out or painted, so
//! its cost is paid only while it can be seen.
//!
//! Position comes from the previous frame, so the child is realised one frame
//! after it comes within `margin`; keep `margin` above the distance a frame
//! can scroll. A block that has never been measured is always realised.

use gpui::{
    div, px, AnyElement, App, Bounds, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Styled, Window,
};
use std::collections::HashMap;

/// Last painted bounds of every lazy block, by key.
#[derive(Default)]
struct Store(HashMap<ElementId, Bounds<Pixels>>);
impl gpui::Global for Store {}

/// Whether a block at `top..top+height` (window coordinates) is within
/// `margin` of a window `viewport` tall. Pure, so it is unit-tested.
pub fn near(top: f32, height: f32, viewport: f32, margin: f32) -> bool {
    top + height > -margin && top < viewport + margin
}

/// Wrap `child` so it is only laid out and painted near the window. `key` must
/// be unique and stable across frames.
pub fn lazy(key: impl Into<ElementId>, child: impl IntoElement) -> Lazy {
    Lazy {
        key: key.into(),
        child: child.into_any_element(),
        spacer: None,
        margin: 500.0,
        enabled: std::env::var_os("DODAI_NOCULL").is_none(),
    }
}

/// See [`lazy`].
pub struct Lazy {
    key: ElementId,
    child: AnyElement,
    spacer: Option<AnyElement>,
    margin: f32,
    enabled: bool,
}

impl Lazy {
    /// Distance outside the window (px) within which the child is realised.
    pub fn margin(mut self, px: f32) -> Self {
        self.margin = px;
        self
    }
}

impl IntoElement for Lazy {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl gpui::Element for Lazy {
    /// `true` while the child is realised.
    type RequestLayoutState = bool;
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, bool) {
        let seen = cx
            .try_global::<Store>()
            .and_then(|s| s.0.get(&self.key).copied());
        let vh: f32 = window.viewport_size().height.into();
        if let Some(b) = seen.filter(|_| self.enabled) {
            let (top, h) = (f32::from(b.origin.y), f32::from(b.size.height));
            if !near(top, h, vh, self.margin) {
                let mut s = div().flex_none().w_full().h(px(h)).into_any_element();
                let id = s.request_layout(window, cx);
                self.spacer = Some(s);
                return (id, false);
            }
        }
        (self.child.request_layout(window, cx), true)
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        live: &mut bool,
        window: &mut Window,
        cx: &mut App,
    ) {
        if *live {
            self.child.prepaint(window, cx);
        } else if let Some(s) = self.spacer.as_mut() {
            s.prepaint(window, cx);
        }
        cx.default_global::<Store>()
            .0
            .insert(self.key.clone(), bounds);
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        live: &mut bool,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        if *live {
            self.child.paint(window, cx);
        } else if let Some(s) = self.spacer.as_mut() {
            s.paint(window, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::near;

    #[test]
    fn near_covers_the_window_and_the_margin() {
        assert!(near(100.0, 50.0, 900.0, 0.0), "inside");
        assert!(!near(1000.0, 50.0, 900.0, 0.0), "below");
        assert!(near(1000.0, 50.0, 900.0, 200.0), "below but within margin");
        assert!(!near(-500.0, 100.0, 900.0, 200.0), "well above");
        assert!(near(-250.0, 100.0, 900.0, 200.0), "above but within margin");
        assert!(
            near(-100.0, 5000.0, 900.0, 0.0),
            "tall block spanning the window"
        );
    }
}
