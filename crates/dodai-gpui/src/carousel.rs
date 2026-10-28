//! [`Carousel`]: horizontally paged items with arrows and dot indicators.

use crate::surfaces::IconButton;
use crate::{hsla, theme_of};
use gpui::{div, px, AnyElement, App, ElementId, IntoElement, ParentElement, Styled, Window};
use std::rc::Rc;

/// First visible index when `active` is selected out of `count` items showing
/// `visible` at a time (keeps the window inside the list).
pub fn window_start(active: usize, count: usize, visible: usize) -> usize {
    let v = visible.max(1);
    active
        .min(count.saturating_sub(1))
        .min(count.saturating_sub(v))
}

/// How items share the row (Material 3 carousel layouts).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselLayout {
    /// Equal fixed-width items ([`Carousel::item_width`]).
    #[default]
    Uncontained,
    /// One large, one medium and one small item.
    MultiBrowse,
    /// One large item and a small peek of the next.
    Hero,
    /// A large item centred between two small ones.
    CenterHero,
}

/// Smallest item width of the contained layouts (the "peek" items).
pub const SMALL_ITEM: f32 = 56.0;

/// Item widths for `layout` filling exactly `total` px with `gap` between items. Pure.
/// [`CarouselLayout::Uncontained`] returns an empty list (it uses a fixed width instead).
pub fn layout_widths(layout: CarouselLayout, total: f32, gap: f32) -> Vec<f32> {
    let small = SMALL_ITEM;
    match layout {
        CarouselLayout::Uncontained => Vec::new(),
        CarouselLayout::Hero => vec![(total - gap - small).max(small), small],
        CarouselLayout::CenterHero => {
            let large = (total - 2.0 * gap - 2.0 * small).max(small);
            vec![small, large, small]
        }
        CarouselLayout::MultiBrowse => {
            let rest = (total - 2.0 * gap - small).max(2.0 * small);
            let large = rest * 0.62;
            vec![large, rest - large, small]
        }
    }
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
    layout: CarouselLayout,
    width: f32,
    item_height: f32,
    fill: bool,
    on_select: Option<IndexFn>,
}

impl Carousel {
    /// Create a new [`Carousel`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            active: 0,
            visible: 3,
            item_width: 160.0,
            layout: CarouselLayout::Uncontained,
            width: 560.0,
            item_height: 160.0,
            fill: false,
            on_select: None,
        }
    }
    /// Add an item.
    pub fn item(mut self, e: impl IntoElement) -> Self {
        self.items.push(e.into_any_element());
        self
    }
    /// The selected item (controlled).
    pub fn selected(mut self, i: usize) -> Self {
        self.active = i;
        self
    }
    /// Items shown at once (default 3).
    pub fn visible(mut self, n: usize) -> Self {
        self.visible = n.max(1);
        self
    }
    /// Width of each item in px.
    pub fn item_width(mut self, w: f32) -> Self {
        self.item_width = w;
        self
    }
    /// Item arrangement; the contained layouts ignore `visible` / `item_width`.
    pub fn layout(mut self, l: CarouselLayout) -> Self {
        self.layout = l;
        self
    }
    /// Total width of the item row for the contained layouts.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    /// Fill the parent's width: items (uncontained) or the item row (contained layouts) are
    /// sized from the measured space between the arrows, ignoring `item_width` and `width`.
    pub fn fill(mut self, on: bool) -> Self {
        self.fill = on;
        self
    }
    /// Item height for the contained layouts.
    pub fn item_height(mut self, h: f32) -> Self {
        self.item_height = h;
        self
    }
    crate::callback!(
        /// Called when an item is chosen, with the index.
        on_select, usize
    );
}

impl Carousel {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let own = crate::own::Own::new(window, cx, self.id.clone(), self.active);
        self.active = own.get();
        let (state, inner) = (own.setter(), self.on_select.take());
        self.on_select = Some(std::rc::Rc::new(move |i, w, cx| {
            crate::own::choose(&state, cx, i);
            if let Some(f) = &inner {
                f(i, w, cx);
            }
        }));
        let t = theme_of(cx);
        let n = self.items.len();
        let gap = if t.look().tonal { 8.0 } else { t.spacing.md };
        // The parent's width, as painted last frame; the row is laid out again once it is known.
        let measured = self.fill.then(|| {
            window
                .use_keyed_state((self.id.clone(), "fit"), cx, |_, _| {
                    Rc::new(std::cell::Cell::new(0.0f32))
                })
                .read(cx)
                .clone()
        });
        let avail = measured.as_ref().map_or(0.0, |m| m.get());
        let arrows = 2.0 * (t.shape.control_height + t.spacing.sm);
        let row_w = if self.fill && avail > arrows + 1.0 {
            avail - arrows
        } else {
            self.width
        };
        let widths = layout_widths(self.layout, row_w, gap);
        let visible = if widths.is_empty() {
            self.visible
        } else {
            widths.len()
        };
        let start = window_start(self.active, n, visible);
        let max_start = n.saturating_sub(visible);
        let mk = |suffix: &'static str, glyph: &'static str, to: usize, enabled: bool| {
            let mut b = IconButton::new((self.id.clone(), suffix), glyph).disabled(!enabled);
            if let (true, Some(f)) = (enabled, self.on_select.clone()) {
                b = b.on_click(move |_, w, cx| f(to, w, cx));
            }
            b
        };
        let prev = mk("prev", "‹", start.saturating_sub(1), start > 0);
        let next = mk("next", "›", (start + 1).min(max_start), start < max_start);
        let w = if self.fill && avail > arrows + 1.0 {
            ((row_w - (visible as f32 - 1.0).max(0.0) * gap) / visible as f32).max(1.0)
        } else {
            self.item_width
        };
        let radius = if t.look().tonal {
            t.role_shape(dodai_core::ShapeRole::Carousel).radius
        } else {
            0.0
        };
        let height = self.item_height;
        // Uncontained rows slide to the new window; contained layouts step (slot widths differ).
        let slide = if widths.is_empty() {
            crate::motion::smooth(window, cx, self.id.clone(), start as f32)
                .clamp(0.0, max_start as f32)
        } else {
            start as f32
        };
        let first = slide.floor() as usize;
        let frac = slide - first as f32;
        let take = if frac > 0.001 { visible + 1 } else { visible };
        let shown = self
            .items
            .into_iter()
            .enumerate()
            .skip(first)
            .take(take)
            .enumerate()
            .map(|(k, (_, e))| match widths.get(k) {
                // Contained layouts: clipped, rounded slots of computed width.
                Some(&cw) => div()
                    .w(px(cw))
                    .h(px(height))
                    .flex_none()
                    .overflow_hidden()
                    .rounded(px(radius))
                    .child(e),
                None => div().w(px(w)).flex_none().child(e),
            });
        let pages = max_start + 1;
        let dot = t.sizes().bar_thickness;
        let dots = (0..pages).map(|i| {
            let on = i == start;
            div()
                .w(px(if on { dot * 3.0 } else { dot }))
                .h(px(dot))
                .bg(hsla(if on {
                    t.palette.primary
                } else {
                    t.palette.border_strong
                }))
        });
        let mut root = div().relative().flex().flex_col().gap(px(t.spacing.sm));
        if let Some(m) = measured {
            let wide = avail;
            root = root.w_full().child(
                gpui::canvas(
                    move |b, w, _| {
                        let now: f32 = b.size.width.into();
                        if (now - m.get()).abs() > 0.5 {
                            m.set(now);
                            w.refresh();
                        }
                    },
                    |_, _, _, _| (),
                )
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h(px(0.0)),
            );
            if wide <= 0.0 {
                crate::motion::next_frame(window);
            }
        }
        root.child(
            div()
                .flex()
                .items_center()
                .gap(px(t.spacing.sm))
                .child(prev)
                .child(if widths.is_empty() {
                    div()
                        .w(px(
                            visible as f32 * w + (visible as f32 - 1.0).max(0.0) * gap
                        ))
                        .overflow_hidden()
                        .child(
                            div()
                                .flex()
                                .gap(px(gap))
                                .ml(px(-frac * (w + gap)))
                                .children(shown),
                        )
                } else {
                    div().flex().gap(px(gap)).children(shown)
                })
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

crate::entry_render!(Carousel);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_fill_the_row_exactly() {
        for l in [
            CarouselLayout::Hero,
            CarouselLayout::CenterHero,
            CarouselLayout::MultiBrowse,
        ] {
            let w = layout_widths(l, 600.0, 8.0);
            let sum: f32 = w.iter().sum::<f32>() + 8.0 * (w.len() as f32 - 1.0);
            assert!((sum - 600.0).abs() < 0.01, "{l:?}: {sum}");
            assert!(w.iter().all(|&x| x >= SMALL_ITEM));
            let max = w.iter().cloned().fold(0.0, f32::max);
            assert!(max > SMALL_ITEM * 2.0, "{l:?} has a large item");
        }
        assert!(layout_widths(CarouselLayout::Uncontained, 600.0, 8.0).is_empty());
    }

    #[test]
    fn window_stays_in_bounds() {
        assert_eq!(window_start(0, 8, 3), 0);
        assert_eq!(window_start(4, 8, 3), 4);
        assert_eq!(window_start(7, 8, 3), 5);
        assert_eq!(window_start(3, 2, 3), 0);
        assert_eq!(window_start(0, 0, 3), 0);
    }
}
