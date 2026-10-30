//! The sidebar as its own cached view.
//!
//! A scroll re-renders the page, and the page used to rebuild and lay out the whole sidebar
//! (every category, sub-group and jump row) on each scroll frame. The sidebar does not move
//! when the page scrolls, so as a cached view gpui replays its last paint; it rebuilds on a
//! real state change, and on a scroll only when the scroll-spy moves its highlight.

use super::*;

pub(crate) struct SidebarView {
    showcase: Entity<Showcase>,
    _observe: gpui::Subscription,
}

impl SidebarView {
    pub(crate) fn new(showcase: Entity<Showcase>, cx: &mut Context<Self>) -> Self {
        // Scrolling and demo steps change nothing here (the scroll-spy notifies directly).
        let _observe = cx.observe(&showcase, |_, sc, cx| {
            let s = sc.read(cx);
            if !s.quiet && !s.demo_only {
                cx.notify();
            }
        });
        Self { showcase, _observe }
    }
}

impl Render for SidebarView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let narrow = window.viewport_size().width < px(760.0);
        self.showcase.update(cx, |s, cx| {
            let me = cx.entity();
            let (side, need) = s.sidebar(&me, narrow, cx);
            if (need - s.side_need).abs() > 0.5 {
                // The page sizes the sidebar column: let it re-measure (no section rebuild).
                s.side_need = need;
                s.quiet = true;
                cx.notify();
            }
            side
        })
    }
}
