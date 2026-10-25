//! One showcase section as its own cached view.
//!
//! gpui re-renders a view's ancestors whenever something inside it asks for a frame (a spinner,
//! a progress wave, a hover tween). With every section built inside `Showcase::render`, one
//! spinner rebuilt the whole page 60+ times a second. Here each section is a separate cached
//! view: a frame request re-renders only the section that animates, the others replay their
//! last paint. gpui sizes a cached view from its style alone, so the section reports its
//! measured height back and `Showcase::render` uses it as the style height.

use super::*;

pub(crate) struct SectionView {
    showcase: Entity<Showcase>,
    title: &'static str,
    _observe: gpui::Subscription,
}

impl SectionView {
    pub(crate) fn new(
        showcase: Entity<Showcase>,
        title: &'static str,
        cx: &mut Context<Self>,
    ) -> Self {
        // Any real state change on the showcase (a click, a slider) rebuilds the section;
        // scrolling only repaints the page, so it is skipped.
        let _observe = cx.observe(&showcase, |_, sc, cx| {
            if !sc.read(cx).quiet {
                cx.notify();
            }
        });
        Self {
            showcase,
            title,
            _observe,
        }
    }
}

impl Render for SectionView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title = self.title;
        let showcase = self.showcase.clone();
        let el = showcase.update(cx, |s, cx| s.take_section(title, cx));
        div()
            .w_full()
            .on_children_prepainted(move |bounds, window, cx| {
                let Some(b) = bounds.first() else { return };
                let h: f32 = b.size.height.into();
                let changed = showcase.update(cx, |s, _| {
                    let old = s.real_h.insert(title, h);
                    s.fresh.insert(title);
                    s.heights.insert(title, h);
                    old.is_none_or(|o| (o - h).abs() > 0.5)
                });
                if changed {
                    window.request_animation_frame();
                }
            })
            .children(el)
    }
}

impl Showcase {
    /// The element of one section (with its code toggle), building its group if needed.
    pub(crate) fn take_section(
        &mut self,
        title: &'static str,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        let (cat, gi, key) = *self.section_group.get(title)?;
        if !self.built.contains_key(title) {
            let group = Self::groups_of(cat).get(gi).copied()?;
            for (_, ti, el, _) in self.build_group(cat, group, cx) {
                self.built.insert(ti, el);
            }
        }
        let el = self.built.remove(title)?;
        let t = theme_of(cx);
        let me = cx.entity();
        Some(self.with_code(title, Some(key), el, &t, &me))
    }
}
