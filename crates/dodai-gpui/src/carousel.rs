//! [`Carousel`]: horizontally paged items with arrows and dot indicators.

use crate::surfaces::IconButton;
use crate::{hsla, theme_of};
use gpui::{
    div, px, AnyElement, App, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window,
};
use std::rc::Rc;

/// First visible index when `active` is selected out of `count` items showing
/// `visible` at a time (keeps the window inside the list).
pub fn window_start(active: usize, count: usize, visible: usize) -> usize {
    let v = visible.max(1);
    active
        .min(count.saturating_sub(1))
        .min(count.saturating_sub(v))
}

type IndexFn = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Controlled carousel; each item gets a fixed `item_width`.
#[derive(IntoElement)]
pub struct Carousel {
    id: ElementId,
    items: Vec<AnyElement>,
    active: usize,
    visible: usize,
    item_width: f32,
    on_select: Option<IndexFn>,
}

impl Carousel {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            active: 0,
            visible: 3,
            item_width: 160.0,
            on_select: None,
        }
    }
    pub fn item(mut self, e: impl IntoElement) -> Self {
        self.items.push(e.into_any_element());
        self
    }
    pub fn active(mut self, i: usize) -> Self {
        self.active = i;
        self
    }
    /// Items shown at once (default 3).
    pub fn visible(mut self, n: usize) -> Self {
        self.visible = n.max(1);
        self
    }
    pub fn item_width(mut self, w: f32) -> Self {
        self.item_width = w;
        self
    }
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl Carousel {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let n = self.items.len();
        let start = window_start(self.active, n, self.visible);
        let max_start = n.saturating_sub(self.visible);
        let mk = |suffix: &'static str, glyph: &'static str, to: usize, enabled: bool| {
            let mut b = IconButton::new((self.id.clone(), suffix), glyph).disabled(!enabled);
            if let (true, Some(f)) = (enabled, self.on_select.clone()) {
                b = b.on_click(move |_, w, cx| f(to, w, cx));
            }
            b
        };
        let prev = mk("prev", "‹", start.saturating_sub(1), start > 0);
        let next = mk("next", "›", (start + 1).min(max_start), start < max_start);
        let w = self.item_width;
        let shown = self
            .items
            .into_iter()
            .enumerate()
            .skip(start)
            .take(self.visible)
            .map(|(_, e)| div().w(px(w)).flex_none().child(e));
        let pages = max_start + 1;
        let dots = (0..pages).map(|i| {
            let on = i == start;
            div()
                .w(px(if on { 18.0 } else { 6.0 }))
                .h(px(6.0))
                .bg(hsla(if on {
                    t.palette.primary
                } else {
                    t.palette.border_strong
                }))
        });
        div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(t.spacing.sm))
                    .child(prev)
                    .child(div().flex().gap(px(t.spacing.md)).children(shown))
                    .child(next),
            )
            .child(
                div()
                    .flex()
                    .justify_center()
                    .gap(px(t.spacing.xs))
                    .children(dots),
            )
    }
}

impl RenderOnce for Carousel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "Carousel", motion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_stays_in_bounds() {
        assert_eq!(window_start(0, 8, 3), 0);
        assert_eq!(window_start(4, 8, 3), 4);
        assert_eq!(window_start(7, 8, 3), 5);
        assert_eq!(window_start(3, 2, 3), 0);
        assert_eq!(window_start(0, 0, 3), 0);
    }
}
