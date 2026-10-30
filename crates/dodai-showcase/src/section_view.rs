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
        // A demo step rebuilds only the sections that show the playing values.
        let _observe = cx.observe(&showcase, move |_, sc, cx| {
            let s = sc.read(cx);
            if !s.quiet && (!s.demo_only || s.live_sections.contains(title)) {
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
        let started = std::time::Instant::now();
        let el = showcase.update(cx, |s, cx| s.take_section(title, cx));
        div()
            .w_full()
            .on_children_prepainted(move |bounds, window, cx| {
                section_prof(title, started);
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

/// Perf probe (`DODAI_PROF`): time from a section's render to its prepaint, summed per
/// section; the costliest are printed once a second.
fn section_prof(title: &'static str, started: std::time::Instant) {
    type Acc = (
        Option<std::time::Instant>,
        std::collections::HashMap<&'static str, (f32, u32)>,
    );
    thread_local! {
        static ACC: std::cell::RefCell<Acc> = Default::default();
    }
    if std::env::var_os("DODAI_PROF").is_none() {
        return;
    }
    ACC.with(|a| {
        let mut a = a.borrow_mut();
        let ms = started.elapsed().as_secs_f32() * 1000.0;
        if ms > 3.0 {
            eprintln!("slow section {title}: {ms:.1} ms");
        }
        let e = a.1.entry(title).or_default();
        *e = (e.0 + ms, e.1 + 1);
        let t0 = *a.0.get_or_insert_with(std::time::Instant::now);
        if t0.elapsed().as_secs_f32() >= 1.0 {
            let mut v: Vec<_> = a.1.drain().collect();
            v.sort_by(|p, q| q.1 .0.total_cmp(&p.1 .0));
            for (t, (ms, n)) in v.iter().take(5) {
                eprintln!("section {t}: {ms:.1} ms over {n} builds");
            }
            a.0 = None;
        }
    });
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
            let stamps = (self.demo_seen, self.inst_seen);
            let built = self.build_group(cat, group, cx);
            let live = (self.demo_seen, self.inst_seen) != stamps;
            for (_, ti, el, _) in built {
                if live {
                    self.live_sections.insert(ti);
                } else {
                    self.live_sections.remove(ti);
                }
                self.built.insert(ti, el);
            }
        }
        let el = self.built.remove(title)?;
        let t = theme_of(cx);
        let me = cx.entity();
        Some(self.with_code(title, Some(key), el, &t, &me))
    }
}
