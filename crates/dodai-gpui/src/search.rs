//! [`SearchView`] (text input + live results), [`RichTooltip`] (title, body,
//! optional action) and [`IconBadge`] (badge pinned to a child's corner).

use crate::command::fuzzy_filter;
use crate::data::Badge;
use crate::interact::{track, use_interact};
use crate::list::list_style;
use crate::paint::{shaped, shaped_id, ShapeStyle};
use crate::text::first_family;
use crate::text_input::TextInput;
use crate::{hsla, theme_of};
use dodai_core::Tone;
use gpui::{
    deferred, div, px, AnyElement, AnyView, App, AppContext as _, Context, ElementId, Entity,
    InteractiveElement, IntoElement, ParentElement, Render, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

type IndexFn = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type VoidFn = Rc<dyn Fn(&mut Window, &mut App)>;

// ───────────────────────────── SearchView ─────────────────────────────

/// Split `text` into `(segment, matched)` runs for highlighting the fuzzy
/// subsequence match of `query` (case-insensitive). Pure.
pub fn highlight_runs(text: &str, query: &str) -> Vec<(String, bool)> {
    let q: Vec<char> = query.to_lowercase().chars().collect();
    let mut runs: Vec<(String, bool)> = Vec::new();
    let mut qi = 0;
    for c in text.chars() {
        let hit = qi < q.len() && c.to_lowercase().next() == Some(q[qi]);
        if hit {
            qi += 1;
        }
        match runs.last_mut() {
            Some((s, m)) if *m == hit => s.push(c),
            _ => runs.push((c.to_string(), hit)),
        }
    }
    if qi < q.len() {
        // Not a full match: no highlighting.
        return vec![(text.to_string(), false)];
    }
    runs
}

/// Full-text search: input, clear button and a fuzzy-filtered result list
/// with matched characters highlighted. Owns no state: pass the
/// [`TextInput`] entity and an `open` flag.
#[derive(IntoElement)]
pub struct SearchView {
    id: ElementId,
    input: Entity<TextInput>,
    items: Vec<SharedString>,
    open: bool,
    max_rows: usize,
    on_pick: Option<IndexFn>,
    on_clear: Option<VoidFn>,
}

impl SearchView {
    pub fn new(id: impl Into<ElementId>, input: Entity<TextInput>) -> Self {
        Self {
            id: id.into(),
            input,
            items: Vec::new(),
            open: false,
            max_rows: 6,
            on_pick: None,
            on_clear: None,
        }
    }
    pub fn items<S: Into<SharedString>>(mut self, i: impl IntoIterator<Item = S>) -> Self {
        self.items = i.into_iter().map(Into::into).collect();
        self
    }
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    pub fn max_rows(mut self, n: usize) -> Self {
        self.max_rows = n.max(1);
        self
    }
    /// Called with the index into the original `items`.
    pub fn on_pick(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(f));
        self
    }
    /// Show a ✕ button while the query is non-empty.
    pub fn on_clear(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_clear = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for SearchView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let query = self.input.read(cx).text().to_string();
        let mut head = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.sm))
            .child(
                div()
                    .text_color(hsla(t.palette.text_muted))
                    .child(SharedString::from("⌕")),
            )
            .child(div().flex_1().child(self.input.clone()));
        if let (false, Some(f)) = (query.is_empty(), self.on_clear.clone()) {
            head = head.child(
                div()
                    .id((self.id.clone(), "clear"))
                    .cursor_pointer()
                    .text_color(hsla(t.palette.text_muted))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(SharedString::from("✕")),
            );
        }
        let mut root = div().relative().w_full().child(head);
        if self.open {
            let hits = fuzzy_filter(&query, &self.items);
            let p = t.panel(false);
            let mut menu = shaped(
                &t.shape,
                ShapeStyle::from_tokens(&t, Some(p.background), Some(p.border), false, true),
            )
            .absolute()
            .top(px(t.shape.control_height + 4.0))
            .left_0()
            .w_full()
            .flex()
            .flex_col()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md));
            if hits.is_empty() {
                menu = menu.child(
                    div()
                        .px(px(t.spacing.md))
                        .py(px(t.spacing.sm))
                        .text_color(hsla(t.palette.text_muted))
                        .child(SharedString::from("No results")),
                );
            }
            for &i in hits.iter().take(self.max_rows) {
                let id: ElementId = (self.id.clone(), format!("r{i}")).into();
                let st = use_interact(&id, window, cx);
                let hovered = st.read(cx).hovered;
                let (style, _, _) = list_style(&t, false, hovered);
                let mut row = track(shaped_id(id, &t.shape, style), &st)
                    .h(px(t.shape.control_height * 0.9))
                    .px(px(t.spacing.md))
                    .flex()
                    .items_center()
                    .cursor_pointer();
                for (seg, hit) in highlight_runs(&self.items[i], &query) {
                    row = row.child(
                        div()
                            .text_color(hsla(if hit {
                                t.palette.primary
                            } else {
                                t.palette.text
                            }))
                            .child(seg),
                    );
                }
                if let Some(f) = self.on_pick.clone() {
                    row = row.on_click(move |_, w, cx| f(i, w, cx));
                }
                menu = menu.child(row);
            }
            root = root.child(deferred(menu).with_priority(50));
        }
        root
    }
}

// ───────────────────────────── RichTooltip ─────────────────────────────

/// Tooltip with a title, body text and optional action label:
/// `.tooltip(RichTooltip::build("Title", "Body", Some("Learn more")))`.
pub struct RichTooltip {
    title: SharedString,
    body: SharedString,
    action: Option<SharedString>,
}

impl RichTooltip {
    /// Builder for gpui's `.tooltip(...)`.
    pub fn build(
        title: impl Into<SharedString>,
        body: impl Into<SharedString>,
        action: Option<&str>,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
        let (title, body) = (title.into(), body.into());
        let action = action.map(|a| SharedString::from(a.to_string()));
        move |_, cx| {
            let (title, body, action) = (title.clone(), body.clone(), action.clone());
            cx.new(|_| RichTooltip {
                title,
                body,
                action,
            })
            .into()
        }
    }
}

impl Render for RichTooltip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme_of(cx);
        let style = ShapeStyle::from_tokens(
            &t,
            Some(t.palette.overlay),
            Some(t.palette.border_strong),
            false,
            true,
        );
        let mut el = shaped(&t.shape, style)
            .max_w(px(280.0))
            .p(px(t.spacing.md))
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .font_family(first_family(t.typography.body))
            .child(
                div()
                    .text_size(px(t.typography.size_md))
                    .text_color(hsla(t.palette.text))
                    .child(self.title.clone()),
            )
            .child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(self.body.clone()),
            );
        if let Some(a) = self.action.clone() {
            el = el.child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.primary))
                    .child(a),
            );
        }
        el
    }
}

// ────────────────────────────── IconBadge ──────────────────────────────

/// Pins a [`Badge`] to the top-right corner of `child` (icon, avatar, …).
#[derive(IntoElement)]
pub struct IconBadge {
    child: AnyElement,
    badge: Option<Badge>,
}

impl IconBadge {
    pub fn new(child: impl IntoElement) -> Self {
        Self {
            child: child.into_any_element(),
            badge: None,
        }
    }
    /// Counter badge; shows `99+` past 99, hidden when `n == 0`.
    pub fn count(mut self, n: u32, tone: Tone) -> Self {
        self.badge = (n > 0).then(|| Badge::new(count_label(n), tone));
        self
    }
    /// Status dot.
    pub fn dot(mut self, tone: Tone) -> Self {
        self.badge = Some(Badge::dot(tone));
        self
    }
}

/// `n` as badge text (`99+` cap). Pure.
pub fn count_label(n: u32) -> String {
    if n > 99 {
        "99+".into()
    } else {
        n.to_string()
    }
}

impl IconBadge {
    fn build(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut el = div().relative().child(self.child);
        if let Some(b) = self.badge {
            el = el.child(div().absolute().top(px(-6.0)).right(px(-6.0)).child(b));
        }
        el
    }
}

impl RenderOnce for IconBadge {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "IconBadge", motion)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_marks_subsequence() {
        let r = highlight_runs("Open File", "of");
        assert_eq!(r[0], ("O".to_string(), true));
        assert!(r.iter().any(|(s, m)| *m && s == "F"));
        let joined: String = r.iter().map(|(s, _)| s.as_str()).collect();
        assert_eq!(joined, "Open File");
    }

    #[test]
    fn highlight_no_match_is_plain() {
        assert_eq!(
            highlight_runs("abc", "xyz"),
            vec![("abc".to_string(), false)]
        );
        assert_eq!(highlight_runs("abc", ""), vec![("abc".to_string(), false)]);
    }

    #[test]
    fn count_caps() {
        assert_eq!(count_label(7), "7");
        assert_eq!(count_label(100), "99+");
    }
}
